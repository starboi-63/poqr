use std::collections::{HashMap, HashSet};
pub type CircuitId = u32;

pub struct CircuitTable {
    /// Map of destination port to circuit
    pub circuits: HashMap<u16, CircuitId>,
    pub used_circuit_ids: HashSet<CircuitId>,
}

impl CircuitTable {
    pub fn new() -> CircuitTable {
        CircuitTable {
            circuits: HashMap::new(),
            used_circuit_ids: HashSet::new(),
        }
    }

    pub fn insert(&mut self, port: u16, circuit_id: CircuitId) {
        self.circuits.insert(port, circuit_id);
        self.used_circuit_ids.insert(circuit_id);
    }

    pub fn get(&self, port: u16) -> Option<&CircuitId> {
        self.circuits.get(&port)
    }

    pub fn remove(&mut self, port: u16) -> Option<CircuitId> {
        self.used_circuit_ids.remove(&self.circuits[&port]);
        self.circuits.remove(&port)
    }
}
