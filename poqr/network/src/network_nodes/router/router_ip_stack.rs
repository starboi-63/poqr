use super::Router;
use crate::network_stacks::ip::{
    ForwardingTable, Interface, IpHandler, IpPacket, IpStack, RoutingTable,
};
use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

impl IpStack for Router {
    fn to_arc(&self) -> Arc<dyn IpStack> {
        Arc::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn forwarding_table(&self) -> Arc<Mutex<ForwardingTable>> {
        self.forwarding_table.clone()
    }

    fn routing_table(&self) -> Arc<Mutex<RoutingTable>> {
        self.routing_table.clone()
    }

    fn interfaces(&self) -> Arc<Mutex<HashMap<String, Arc<Mutex<Interface>>>>> {
        self.interfaces.clone()
    }

    fn packet_receiver(&self) -> Arc<Mutex<mpsc::Receiver<IpPacket>>> {
        self.packet_receiver.clone()
    }

    fn protocol_handlers(&self) -> Arc<Mutex<HashMap<u8, IpHandler>>> {
        self.protocol_handlers.clone()
    }
}
