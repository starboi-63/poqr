use crate::Channel;
use std::collections::HashMap;

pub struct ChannelTable {
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
}
