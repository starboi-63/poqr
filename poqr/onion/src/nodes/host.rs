use crate::tables::{ChannelTable, CircuitTable};

pub struct Host {
    port: u16,
    channel_table: ChannelTable,
    circuit_table: CircuitTable,
}

impl Host {
    pub fn new(port: u16) -> Host {
        Host {
            port,
            channel_table: ChannelTable::new(),
            circuit_table: CircuitTable::new(),
        }
    }

    pub fn create_circuit(&mut self, hops: Vec<u32>) {
        thread::spawn(move || {})
    }
}
