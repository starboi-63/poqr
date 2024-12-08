use super::Host;
use crate::network_stacks::ip::IpRepl;
use crate::network_stacks::tcp::TcpRepl;
use crate::repl::{Repl, ReplHandler};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl Repl for Host {
    fn to_arc(&self) -> Arc<dyn Repl> {
        Arc::new(self.clone())
    }

    fn as_ip_repl(&self) -> Option<Arc<dyn IpRepl>> {
        Some(Arc::new(self.clone()))
    }

    fn as_tcp_repl(&self) -> Option<Arc<dyn TcpRepl>> {
        Some(Arc::new(self.clone()))
    }

    fn repl_handlers(&self) -> Arc<Mutex<HashMap<String, ReplHandler>>> {
        self.repl_handlers.clone()
    }
}

impl IpRepl for Host {} // One-liner trait implementation (no methods to implement)
impl TcpRepl for Host {} // One-liner trait implementation (no methods to implement)
