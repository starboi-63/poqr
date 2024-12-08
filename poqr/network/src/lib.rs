// Private to /src directory
mod network_nodes;
mod parser;
mod repl;
// Exposed to /bin directory
pub mod network_stacks;
pub use network_nodes::{Host, Router};
pub use parser::IPConfig;
