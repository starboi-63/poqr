use std::net::Ipv4Addr;

#[derive(Debug)]
/// An RIP entry in a RIP message
pub struct RIPEntry {
    /// The cost to reach the destination
    pub cost: u32,
    /// The IP address of the destination
    pub address: u32,
    /// A bit mask representing the subnet mask for the destination (e.g. 0xffffff00 for a /24 subnet)
    pub mask: u32,
}

impl RIPEntry {
    /// Get the address of the RIP entry
    pub fn address(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            (self.address >> 24) as u8,
            (self.address >> 16) as u8,
            (self.address >> 8) as u8,
            self.address as u8,
        )
    }

    /// Get the length of the mask of the RIP entry
    pub fn mask_length(&self) -> u8 {
        let mut mask_length = 0;
        let mut mask = self.mask;

        while mask != 0 {
            mask_length += 1;
            mask <<= 1;
        }

        mask_length
    }
}

#[derive(Debug)]
/// A RIP message
pub struct RipMessage {
    /// The command of the RIP message (e.g. 1 for request, 2 for response)
    pub command: u16,
    /// The number of entries in the RIP message
    pub num_entries: u16,
    /// The entries in the RIP message
    pub entries: Vec<RIPEntry>,
}
