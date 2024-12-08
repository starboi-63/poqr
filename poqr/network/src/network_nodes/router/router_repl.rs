use super::Router;
use crate::network_stacks::ip::IpRepl;
use crate::network_stacks::tcp::TcpRepl;
use crate::repl::{Repl, ReplHandler};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl Repl for Router {
    fn to_arc(&self) -> Arc<dyn Repl> {
        Arc::new(self.clone())
    }

    fn as_ip_repl(&self) -> Option<Arc<dyn IpRepl>> {
        Some(Arc::new(self.clone()))
    }

    fn as_tcp_repl(&self) -> Option<Arc<dyn TcpRepl>> {
        None // Routers do not have TCP stacks
    }

    fn repl_handlers(&self) -> Arc<Mutex<HashMap<String, ReplHandler>>> {
        self.repl_handlers.clone()
    }
}

impl IpRepl for Router {} // One-liner trait implementation (no methods to implement)
