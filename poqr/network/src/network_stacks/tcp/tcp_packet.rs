use super::TcpConnection;
use etherparse::TcpHeader;

/// The type of a TCP packet.
pub enum TcpPacketType {
    Syn,
    SynAck,
    Ack,
    Fin,
    FinAck,
}

impl TcpPacketType {
    /// Get the packet type from the given header flags.
    pub fn from_flags(syn: bool, ack: bool, fin: bool) -> Result<Self, String> {
        match (syn, ack, fin) {
            (true, false, false) => Ok(TcpPacketType::Syn),
            (true, true, false) => Ok(TcpPacketType::SynAck),
            (false, true, false) => Ok(TcpPacketType::Ack),
            (false, false, true) => Ok(TcpPacketType::Fin),
            (false, true, true) => Ok(TcpPacketType::FinAck),
            _ => Err("Invalid TCP flags".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
/// A TCP packet.
pub struct TcpPacket {
    pub header: TcpHeader,
    pub data: Vec<u8>,
}

impl TcpPacket {
    /// Create a new TCP packet with the given parameters.
    pub fn new(
        packet_type: TcpPacketType,
        connection: TcpConnection,
        sequence_number: u32,
        acknowledgment_number: u32,
        window_size: u16,
        data: Option<Vec<u8>>,
    ) -> Self {
        // Create the TCP header.
        let mut header = TcpHeader::default();
        header.source_port = connection.local_address.port();
        header.destination_port = connection.remote_address.port();
        header.sequence_number = sequence_number;
        header.acknowledgment_number = acknowledgment_number;
        header.window_size = window_size;

        // Set the flags based on the packet type.
        (header.syn, header.ack, header.fin) = match packet_type {
            TcpPacketType::Syn => (true, false, false),
            TcpPacketType::SynAck => (true, true, false),
            TcpPacketType::Ack => (false, true, false),
            TcpPacketType::Fin => (false, false, true),
            TcpPacketType::FinAck => (false, true, true),
        };

        // Create the TCP packet.
        let data = data.unwrap_or_else(Vec::new);

        // Calculate the checksum.
        match header.calc_checksum_ipv4_raw(
            connection.local_address.ip().octets(),
            connection.remote_address.ip().octets(),
            &data,
        ) {
            Ok(checksum) => header.checksum = checksum,
            Err(e) => {
                eprintln!("Error calculating TCP checksum: {}", e);
                header.checksum = 0; // Fallback to default.
            }
        }

        Self { header, data }
    }

    /// Validate the checksum of a received TCP packet.
    pub fn validate_checksum(&mut self, connection: TcpConnection) -> Result<bool, String> {
        // Save the received checksum and zero it out.
        let received_checksum = self.header.checksum;
        self.header.checksum = 0;

        // Recalculate the checksum.
        match self.header.calc_checksum_ipv4_raw(
            connection.local_address.ip().octets(),
            connection.remote_address.ip().octets(),
            &self.data,
        ) {
            Ok(calculated_checksum) => Ok(calculated_checksum == received_checksum),
            Err(e) => Err(format!("Error validating checksum: {}", e)),
        }
    }
}
