use super::Router;
use crate::network_stacks::ip::Rip;
use std::sync::Arc;

impl Rip for Router {
    fn to_arc(&self) -> Arc<dyn Rip> {
        Arc::new(self.clone())
    }
}
