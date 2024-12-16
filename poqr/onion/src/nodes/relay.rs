use crate::{ChannelTable, Directory};
use ntru::NtruKeyPair;
use std::sync::{Arc, RwLock};

pub struct Relay {
    id: u32,
    port: u16,
    forward_channels: ChannelTable,
    backward_channels: ChannelTable,
    pub onion_key: NtruKeyPair,
    directory: Arc<RwLock<Directory>>,
}

impl Relay {
    pub fn new(id: u32, port: u16, directory: Arc<RwLock<Directory>>) -> Relay {
        Relay {
            id,
            port,
            forward_channels: ChannelTable::new(),
            backward_channels: ChannelTable::new(),
            onion_key: NtruKeyPair::new(),
            directory,
        }
    }
}
