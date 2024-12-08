// Module: messages
mod rip_message;
mod test_message;
// Exported from messages module
pub use rip_message::{RIPEntry, RipMessage};
pub use test_message::TestMessage;
