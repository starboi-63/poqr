use crate::Channel;
use std::collections::HashMap;

type CircuitId = u32;

pub struct ChannelTable {
    /// Map of circuit id to an encryption key and channel
    channels: HashMap<CircuitId, Channel>,
}

impl ChannelTable {
    pub fn new() -> ChannelTable {
        ChannelTable {
            channels: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: CircuitId, channel: Channel) {
        self.channels.insert(id, channel);
    }

    pub fn get(&self, id: CircuitId) -> Option<&Channel> {
        self.channels.get(&id)
    }

    pub fn get_mut(&mut self, id: CircuitId) -> Option<&mut Channel> {
        self.channels.get_mut(&id)
    }

    pub fn remove(&mut self, id: CircuitId) -> Option<Channel> {
        self.channels.remove(&id)
    }

    pub fn contains_key(&self, id: CircuitId) -> bool {
        self.channels.contains_key(&id)
    }
}
