use std::net::Ipv4Addr;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
/// Represents a subnet mask in the network
pub struct IpNetwork {
    /// The network address of the subnet
    pub address: Ipv4Addr,
    /// The length of the subnet mask (e.g. 24 for a /24 subnet)
    pub mask_length: u8,
}

impl IpNetwork {
    /// Create a new network IP representing a subnet. Bits after the mask length are set to 0.
    ///
    /// Example:
    /// ```
    /// let network_ip = IpNetwork::new(Ipv4Addr::new(10, 2, 0, 10), 24);
    ///
    /// assert_eq!(network_ip.address, Ipv4Addr::new(10, 2, 0, 0));
    /// assert_eq!(network_ip.mask_length, 24);
    /// ```
    pub fn new(address: Ipv4Addr, mask_length: u8) -> Self {
        let mut network_ip = IpNetwork {
            address,
            mask_length,
        };

        // Set the bits after the mask length to 0
        let mask: u32 = network_ip.bit_mask();
        network_ip.address = Ipv4Addr::from(u32::from(network_ip.address) & mask);

        network_ip
    }

    /// Compute a bit mask to validate whether an IP address is in the subnet
    ///
    /// Example:
    /// ```
    /// network_ip = IpNetwork {
    ///     address: Ipv4Addr::new(192, 168, 1, 0),
    ///     mask_length: 24,
    /// };
    ///
    /// assert_eq!(network_ip.get_bit_mask(), 0xffffff00);
    /// ```
    pub fn bit_mask(&self) -> u32 {
        match self.mask_length {
            0 => 0,
            1..=32 => (!0u32) << (32 - self.mask_length), // Left shift 0xffffffff by (32 - mask_length) bits
            _ => panic!("Invalid subnet mask length"),
        }
    }
}
