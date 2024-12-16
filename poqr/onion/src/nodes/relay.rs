use crate::tables::ChannelTable;
use ntru::NtruKeyPair;

pub struct Relay {
    id: u32,
    port: u16,
    forward_channels: ChannelTable,
    backward_channels: ChannelTable,
    pub onion_key: NtruKeyPair,
}

impl Relay {
    pub fn new(id: u32, port: u16) -> Relay {
        Relay {
            id,
            port,
            forward_channels: ChannelTable::new(),
            backward_channels: ChannelTable::new(),
            onion_key: NtruKeyPair::new(),
        }
    }
}
