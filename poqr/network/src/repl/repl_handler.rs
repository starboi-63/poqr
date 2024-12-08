use crate::repl::Repl;
use std::sync::Arc;

/// A REPL command handler function which takes a shared-reference to a REPL and optional string arguments
pub type ReplHandler = fn(Arc<dyn Repl>, Vec<&str>);
