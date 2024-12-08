use super::Host;
use crate::network_stacks::tcp::{TcpSocketTable, TcpStack};
use std::sync::{Arc, Mutex};

impl TcpStack for Host {
    fn to_arc(&self) -> Arc<dyn TcpStack> {
        Arc::new(self.clone())
    }

    fn socket_table(&self) -> Arc<Mutex<TcpSocketTable>> {
        self.socket_table.clone()
    }
}
