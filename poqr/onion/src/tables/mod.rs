// Module: tables
mod channel_table;
mod circuit_table;
// Exported from tables module
pub use channel_table::ChannelTable;
pub use circuit_table::{CircuitId, CircuitTable};
