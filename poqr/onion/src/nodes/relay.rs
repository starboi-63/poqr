use crate::{ChannelTable, Directory};
use ntru::NtruKeyPair;
use std::sync::{Arc, RwLock};

pub struct Relay {
    id: u32,
    port: u16,
    channels: ChannelTable,
    pub identity_key: NtruKeyPair,
    directory: Arc<RwLock<Directory>>,
}

impl Relay {
    pub fn new(id: u32, port: u16, directory: Arc<RwLock<Directory>>) -> Relay {
        Relay {
            id,
            port,
            channels: ChannelTable::new(),
            identity_key: NtruKeyPair::new(),
            directory,
        }
    }
}
