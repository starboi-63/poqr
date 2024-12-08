use crate::network_stacks::ip::{IpPacket, IpStack};
use std::sync::Arc;

/// An IP handler function which takes a shared-reference to an IP stack and an IP packet
pub type IpHandler = fn(&Arc<dyn IpStack>, IpPacket);
