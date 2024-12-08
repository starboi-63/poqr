use super::{RIPEntry, RipMessage, TestMessage};
use crate::network_stacks::tcp::TcpPacket;
use etherparse::{IpNumber, Ipv4Header, TcpHeader};
use std::mem;
use std::net::Ipv4Addr;

// Command numbers
pub const TEST_COMMAND: u16 = 0;
pub const RIP_REQUEST: u16 = 1;
pub const RIP_RESPONSE: u16 = 2;

// Protocol numbers
pub const RIP_PROTOCOL: u8 = 200;
pub const TEST_PROTOCOL: u8 = 0;
pub const TCP_PROTOCOL: u8 = 6;

#[derive(Debug)]
/// The data contained in a packet
pub enum PacketData {
    RipMessage(RipMessage),
    TestMessage(TestMessage),
    TcpMessage(TcpPacket),
}

#[derive(Debug)]
/// A packet within the virtual network
pub struct IpPacket {
    pub header: Ipv4Header,
    pub data: PacketData,
}

impl IpPacket {
    /// Create a new packet with the given source, destination, protocol, and data
    pub fn new(source: Ipv4Addr, destination: Ipv4Addr, protocol: u8, data: PacketData) -> Self {
        let data_length = match &data {
            PacketData::RipMessage(rip_message) => {
                (mem::size_of::<u16>() * 2)
                    + (mem::size_of::<RIPEntry>() * rip_message.num_entries as usize)
            }
            PacketData::TestMessage(test_message) => {
                (mem::size_of::<u16>() * 1) + test_message.data.len()
            }
            PacketData::TcpMessage(tcp_packet) => {
                tcp_packet.header.header_len() + tcp_packet.data.len()
            }
        };

        let mut header = Ipv4Header {
            source: source.octets(),
            destination: destination.octets(),
            time_to_live: 16,
            total_len: Ipv4Header::MIN_LEN as u16 + data_length as u16,
            protocol: IpNumber::from(protocol),
            header_checksum: 0,
            ..Default::default()
        };

        // Initialize the header checksum
        header.header_checksum = header.calc_header_checksum();

        IpPacket { header, data }
    }

    /// Serialize the packet into a byte array which can be sent over the network
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Serialize the header
        buf.extend_from_slice(&self.header.to_bytes());

        // Serialize the data based on the protocol
        match &self.data {
            PacketData::RipMessage(rip_message) => {
                buf.extend_from_slice(&rip_message.command.to_be_bytes());
                buf.extend_from_slice(&rip_message.num_entries.to_be_bytes());

                for entry in &rip_message.entries {
                    buf.extend_from_slice(&entry.cost.to_be_bytes());
                    buf.extend_from_slice(&entry.address.to_be_bytes());
                    buf.extend_from_slice(&entry.mask.to_be_bytes());
                }
            }
            PacketData::TestMessage(test_message) => {
                buf.extend_from_slice(&test_message.command.to_be_bytes());
                buf.extend_from_slice(&test_message.data);
            }
            PacketData::TcpMessage(tcp_packet) => {
                buf.extend_from_slice(&tcp_packet.header.to_bytes());
                buf.extend_from_slice(&tcp_packet.data);
            }
        }

        buf
    }

    /// Deserialize a byte array into an IPPacket struct
    pub fn deserialize(buf: &[u8]) -> IpPacket {
        let (header, _) = Ipv4Header::from_slice(&buf[..Ipv4Header::MIN_LEN]).unwrap();
        let protocol = header.protocol.0;

        let data = match protocol {
            RIP_PROTOCOL => {
                let command =
                    u16::from_be_bytes([buf[Ipv4Header::MIN_LEN], buf[Ipv4Header::MIN_LEN + 1]]);
                let num_entries = u16::from_be_bytes([
                    buf[Ipv4Header::MIN_LEN + 2],
                    buf[Ipv4Header::MIN_LEN + 3],
                ]);
                let mut entries = Vec::new();

                let mut offset = Ipv4Header::MIN_LEN + (mem::size_of::<u16>() * 2);
                for _ in 0..num_entries {
                    let cost = u32::from_be_bytes([
                        buf[offset],
                        buf[offset + 1],
                        buf[offset + 2],
                        buf[offset + 3],
                    ]);
                    let address = u32::from_be_bytes([
                        buf[offset + 4],
                        buf[offset + 5],
                        buf[offset + 6],
                        buf[offset + 7],
                    ]);
                    let mask = u32::from_be_bytes([
                        buf[offset + 8],
                        buf[offset + 9],
                        buf[offset + 10],
                        buf[offset + 11],
                    ]);

                    entries.push(RIPEntry {
                        cost,
                        address,
                        mask,
                    });

                    offset += mem::size_of::<RIPEntry>();
                }

                PacketData::RipMessage(RipMessage {
                    command,
                    num_entries,
                    entries,
                })
            }
            TEST_PROTOCOL => {
                let command =
                    u16::from_be_bytes([buf[Ipv4Header::MIN_LEN], buf[Ipv4Header::MIN_LEN + 1]]);
                let data = buf[Ipv4Header::MIN_LEN + 2..].to_vec();

                PacketData::TestMessage(TestMessage { command, data })
            }
            TCP_PROTOCOL => {
                let (header, _) =
                    TcpHeader::from_slice(&buf[Ipv4Header::MIN_LEN..Ipv4Header::MIN_LEN + 20])
                        .unwrap();
                let data = buf[Ipv4Header::MIN_LEN + header.header_len() as usize..].to_vec();

                PacketData::TcpMessage(TcpPacket { header, data })
            }
            _ => panic!("Unknown protocol: {}", protocol),
        };

        IpPacket { header, data }
    }

    /// Decrement the time-to-live of the packet by 1 (recomputes the header checksum)
    pub fn decrement_ttl(&mut self) {
        self.header.time_to_live -= 1;

        // Update the header checksum (since the TTL has changed)
        self.header.header_checksum = 0; // Reset the checksum to 0 before recalculating
        self.header.header_checksum = self.header.calc_header_checksum();
    }

    /// Get the source ip-address of the packet
    pub fn source(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            self.header.source[0],
            self.header.source[1],
            self.header.source[2],
            self.header.source[3],
        )
    }

    /// Get the destination ip-address of the packet
    pub fn destination(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            self.header.destination[0],
            self.header.destination[1],
            self.header.destination[2],
            self.header.destination[3],
        )
    }

    /// Get the time-to-live of the packet
    pub fn time_to_live(&self) -> u8 {
        self.header.time_to_live
    }

    /// Get the protocol of the packet
    pub fn protocol(&self) -> u8 {
        self.header.protocol.0
    }

    /// Validate the checksum of the packet upon receipt. Returns true if the checksum is valid.
    pub fn validate_checksum(&self) -> bool {
        let mut sum = 0u32;
        let buf = self.header.to_bytes();
        let mut chunks = buf.chunks_exact(2);

        // Add all the 16-bit words together, wrapping around on overflow
        for chunk in &mut chunks {
            let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            sum = sum.wrapping_add(word);
        }

        // Add the last byte if the number of chunks is not a multiple of 2
        if let Some(&last_byte) = chunks.remainder().first() {
            let word = (last_byte as u32) << 8; // Pad with zeros on the right
            sum = sum.wrapping_add(word);
        }

        // Add carry bits until the result is a 16-bit number
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        // The checksum is valid if the sum is 0xFFFF (i.e. all bits are 1)
        !(sum as u16) == 0
    }

    /// Print a packet to the console
    pub fn print(&self) {
        match &self.data {
            PacketData::TestMessage(test_msg) => {
                if let Ok(message) = String::from_utf8(test_msg.data.clone()) {
                    println!(
                        "Received test packet: Src: {}, Dst: {}, TTL: {}, Data: {}",
                        self.source(),
                        self.destination(),
                        self.time_to_live(),
                        message
                    );
                } else {
                    eprintln!("Error: TestMessage data is not valid UTF-8.");
                }
            }
            PacketData::RipMessage(rip_msg) => {
                println!(
                    "Received RIP packet: Src: {}, Dst: {}, TTL: {}, Command: {}, Num entries: {}",
                    self.source(),
                    self.destination(),
                    self.time_to_live(),
                    rip_msg.command,
                    rip_msg.num_entries
                );

                for entry in &rip_msg.entries {
                    println!(
                        "Cost: {}, Address: {}, Mask: {}",
                        entry.cost,
                        entry.address(),
                        entry.mask_length()
                    );
                }
            }
            PacketData::TcpMessage(tcp_packet) => {
                println!(
                    "Received TCP packet: Src: {}, Dst: {}, TTL: {}, Seq: {}, Ack: {}, Data: {}",
                    self.source(),
                    self.destination(),
                    self.time_to_live(),
                    tcp_packet.header.sequence_number,
                    tcp_packet.header.acknowledgment_number,
                    String::from_utf8(tcp_packet.data.clone())
                        .unwrap_or_else(|_| "Invalid UTF-8".into())
                );
            }
        }
    }
}
