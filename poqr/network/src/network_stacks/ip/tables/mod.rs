// Module: tables
mod forwarding_table;
mod routing_table;
// Exported from tables module
pub use forwarding_table::ForwardingTable;
pub use routing_table::{Route, RoutingTable};
