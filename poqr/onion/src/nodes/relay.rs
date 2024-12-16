use crate::tables::ChannelTable;
use ntru::NtruKeyPair;

pub struct Relay {
    port: u16,
    forward_channels: ChannelTable,
    backward_channels: ChannelTable,
    identity_key: NtruKeyPair,
    onion_key: NtruKeyPair,
}
