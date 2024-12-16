use crate::nodes::Relay;

pub struct Circuit {
    ///  The unique identifier of this circuit for a given host
    id: u32,
    /// The list of relays in the circuit ; instantiated as vec with capacity of 3
    relays: Vec<Relay>,
}
