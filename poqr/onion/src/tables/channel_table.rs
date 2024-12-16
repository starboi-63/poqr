use crate::Channel;
use std::collections::HashMap;

pub struct ChannelTable {
    /// Map of circuit id to an encryption key and channel
    channels: HashMap<u32, Channel>,
}

impl ChannelTable {
    pub fn new() -> ChannelTable {
        ChannelTable {
            channels: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u32, channel: Channel) {
        self.channels.insert(id, channel);
    }

    pub fn get(&self, id: u32) -> Option<&Channel> {
        self.channels.get(&id)
    }

    pub fn remove(&mut self, id: u32) -> Option<Channel> {
        self.channels.remove(&id)
    }

    pub fn contains_key(&self, id: &u32) -> bool {
        self.channels.contains_key(id)
    }
}
