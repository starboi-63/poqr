use crate::Circuit;
use std::collections::HashMap;

pub struct CircuitTable {
    /// Map of destination id to circuit
    circuits: HashMap<u32, Circuit>,
}

impl CircuitTable {
    pub fn new() -> CircuitTable {
        CircuitTable {
            circuits: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u32, circuit: Circuit) {
        self.circuits.insert(id, circuit);
    }

    pub fn get(&self, id: u32) -> Option<&Circuit> {
        self.circuits.get(&id)
    }

    pub fn remove(&mut self, id: u32) -> Option<Circuit> {
        self.circuits.remove(&id)
    }
}
