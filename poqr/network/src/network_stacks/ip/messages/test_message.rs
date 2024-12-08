#[derive(Debug)]
/// A test message (used for testing the IP stack)
pub struct TestMessage {
    pub command: u16,
    pub data: Vec<u8>,
}
