use std::{collections::HashMap, net::TcpStream};

pub struct ChannelTable {
    table: HashMap<u32, Channel>,
}

struct Channel {
    conn: TcpStream,
}
