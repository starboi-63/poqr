use super::IpNetwork;
use std::net::Ipv4Addr;

/// Shared behavior for different IP stack networking tables.
pub trait IpTable<V> {
    /// Check if the given destination IP is a part of the given subnet
    fn matches_prefix(&self, destination: Ipv4Addr, subnet: IpNetwork) -> bool {
        // Compute a bit-mask for the subnet
        let mask: u32 = subnet.bit_mask();

        // Check if the masked versions of the destination IP and network IP are equal
        let dest = u32::from(destination);
        let net = u32::from(subnet.address);
        (dest & mask) == (net & mask)
    }

    /// Get the entry with the longest prefix match for the given destination IP
    fn longest_prefix_match(&self, destination: Ipv4Addr) -> (Option<V>, u8);

    /// Search for an entry in the table
    fn get(&self, subnet: IpNetwork) -> Option<V>;

    /// Add a new entry to the table
    fn insert(&mut self, subnet: IpNetwork, value: V);

    /// Remove an entry from the table
    fn remove(&mut self, subnet: IpNetwork);
}
