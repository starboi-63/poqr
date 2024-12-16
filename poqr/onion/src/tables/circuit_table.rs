use crate::nodes::Relay;
use std::collections::HashMap;

pub struct CircuitTable {
    circuits: HashMap<u32, Circuit>,
}

struct Circuit {
    id: u32,
    hops: Vec<Relay>,
}
