use crate::nodes::Relay;
use std::collections::HashSet;

pub struct Directory {
    /// Returns the list of available relays in the directory
    relays: HashSet<Relay>,
}
