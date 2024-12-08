#![warn(clippy::pedantic)]
use ipnet::Ipv4Net;
use std::{
    fmt, fs,
    net::{self, Ipv4Addr},
    num,
};

// NOTE: These data structures only represent structure of a
// configuration file.  In your implementation, you will still need to
// build your own data structures that store relevant information
// about your links, interfaces, etc. at runtime.
//
// These structs only represent the things in the config file--you
// will probably only parse these at startup in order to set up your own
// data structures.

#[derive(Debug, Clone)]
pub enum ParserError {
    Ipnet(ipnet::AddrParseError),
    Net(net::AddrParseError),
    ParseIntError(num::ParseIntError),
    MissingToken(String),
    Other(String),
    BadFormat,

    InvalidLine(String, Box<ParserError>),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::Ipnet(e) => write!(f, "IPNet error: {e}"),
            ParserError::Net(e) => write!(f, "Net error: {e}"),
            ParserError::ParseIntError(e) => write!(f, "ParseInt error: {e}"),
            ParserError::MissingToken(token) => write!(f, "Missing token: {token}"),
            ParserError::Other(e) => write!(f, "Error: {e}"),
            ParserError::BadFormat => write!(f, "Bad format"),
            ParserError::InvalidLine(line, e) => write!(f, "Invalid line: {line}\n{e}"),
        }
    }
}

impl From<ipnet::AddrParseError> for ParserError {
    fn from(e: ipnet::AddrParseError) -> Self {
        ParserError::Ipnet(e)
    }
}

impl From<net::AddrParseError> for ParserError {
    fn from(e: std::net::AddrParseError) -> Self {
        ParserError::Net(e)
    }
}

impl From<num::ParseIntError> for ParserError {
    fn from(e: num::ParseIntError) -> Self {
        ParserError::ParseIntError(e)
    }
}

fn str_to_udp(input: &str) -> (Ipv4Addr, u16) {
    let tokens = input.split(':').collect::<Vec<&str>>();
    (
        tokens[0].parse::<Ipv4Addr>().unwrap(),
        tokens[1].parse::<u16>().unwrap(),
    )
}

#[derive(Debug, PartialEq)]
pub enum RoutingType {
    None,
    Static,
    Rip,
}

impl Default for RoutingType {
    fn default() -> Self {
        Self::None
    }
}

impl TryFrom<&str> for RoutingType {
    type Error = ParserError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "static" => Ok(Self::Static),
            "rip" => Ok(Self::Rip),
            _ => Err(ParserError::Other(format!("Invalid routing type: {value}"))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InterfaceConfig {
    pub name: String,
    // Ip address of the interface + prefix
    pub assigned_prefix: Ipv4Net,
    pub assigned_ip: Ipv4Addr,
    pub udp_addr: Ipv4Addr,
    pub udp_port: u16,
}

impl TryFrom<Vec<&str>> for InterfaceConfig {
    type Error = ParserError;

    /// Create an `InterfaceConfig` from a vector of tokens
    /// Format: interface <name> <virtual IP address>/<prefix> <UDP address>:<UDP port>
    fn try_from(tokens: Vec<&str>) -> Result<Self, ParserError> {
        if tokens.len() != 4 {
            return Err(ParserError::BadFormat);
        }

        let name = String::from(tokens[1]);
        let assigned_prefix: Ipv4Net = tokens[2].parse()?;
        let (udp_addr, udp_port) = str_to_udp(tokens[3]);

        Ok(Self {
            name,
            assigned_prefix,
            assigned_ip: assigned_prefix.addr(),
            udp_addr,
            udp_port,
        })
    }
}

#[derive(Debug)]
pub struct NeighborConfig {
    pub dest_addr: Ipv4Addr,
    pub udp_addr: Ipv4Addr,
    pub udp_port: u16,
    pub interface_name: String,
}

impl TryFrom<Vec<&str>> for NeighborConfig {
    type Error = ParserError;

    /// Create a `NeighborConfig` from a vector of tokens
    /// Format: neighbor <virtual IP> at <UDP address>:<UDP port> via <interface>
    fn try_from(tokens: Vec<&str>) -> Result<Self, ParserError> {
        if tokens.len() != 6 {
            return Err(ParserError::BadFormat);
        }

        let dest_addr: Ipv4Addr = tokens[1].parse()?;
        if tokens[2] != "at" {
            return Err(ParserError::MissingToken(String::from("at")));
        }
        let (udp_addr, udp_port) = str_to_udp(tokens[3]);
        if tokens[4] != "via" {
            return Err(ParserError::MissingToken(String::from("via")));
        }

        Ok(Self {
            dest_addr,
            udp_addr,
            udp_port,
            interface_name: String::from(tokens[5]),
        })
    }
}

pub type StaticRoute = (Ipv4Net, Ipv4Addr);

/// `IPConfig` struct to hold all the parsed data
#[derive(Debug, Default)]
pub struct IPConfig {
    pub interfaces: Vec<InterfaceConfig>,
    pub neighbors: Vec<NeighborConfig>,

    pub routing_mode: RoutingType,

    pub static_routes: Vec<StaticRoute>, // prefix -> addr

    // ROUTERS ONLY: Timing parameters for RIP updates (in milliseconds)
    pub rip_neighbors: Option<Vec<Ipv4Addr>>,
    pub rip_periodic_update_rate: Option<u64>,
    pub rip_timeout_threshold: Option<u64>,

    // HOSTS ONLY: Timing parameters for TCP (in milliseconds)
    pub tcp_rto_min: Option<u64>,
    pub tcp_rto_max: Option<u64>,
}

impl IPConfig {
    /// Create a new `IPConfig` from a file path
    ///
    /// # Panics
    /// Panics if there is an issue reading the file
    #[must_use]
    pub fn new(file_path: String) -> Self {
        Self::try_new(file_path).unwrap()
    }

    /// Create a new `IPConfig` from a file path, returning a Result
    ///
    /// # Errors
    /// Returns an error if there is an issue reading the file
    pub fn try_new(file_path: String) -> Result<Self, ParserError> {
        // Open and read the file
        let file = fs::read_to_string(file_path);
        let file_content = match file {
            Ok(f) => f,
            Err(e) => return Err(ParserError::Other(format!("Error reading file: {e:?}"))),
        };

        let mut ip_config = IPConfig::default();

        // Parse the file contents
        ip_config.parse(&file_content)?;
        Ok(ip_config)
    }

    /// Parse a config based on its contents as a string
    ///
    /// # Errors
    /// Returns an error if there is an issue parsing the file
    pub fn parse(&mut self, config: &str) -> Result<(), ParserError> {
        for line in config.lines() {
            match self.parse_line(line) {
                Ok(()) | Err(ParserError::InvalidLine(_, _)) => {}
                Err(e) => return Err(ParserError::InvalidLine(String::from(line), Box::new(e))),
            };
        }
        Ok(())
    }

    /// Parse a single line of the config, updating the `IPConfig`
    fn parse_line(&mut self, line: &str) -> Result<(), ParserError> {
        let mut tokens = line.split_ascii_whitespace().collect::<Vec<&str>>();

        // Remove # and all tokens after it
        tokens.truncate(
            tokens
                .iter()
                .position(|&x| x == "#")
                .unwrap_or(tokens.len()),
        );

        if tokens.is_empty() {
            return Ok(());
        }

        let directive = tokens[0];

        // Invoke the appropriate parsing function based on the first token
        match directive {
            "interface" => self.interfaces.push(InterfaceConfig::try_from(tokens)?),
            "neighbor" => self.neighbors.push(NeighborConfig::try_from(tokens)?),
            "routing" => self.parse_routing(&tokens)?,
            "route" => self.parse_route(&tokens)?,
            "rip" => self.parse_rip(&tokens)?,
            "tcp" => self.parse_tcp(&tokens)?,
            _ => {
                return Err(ParserError::Other(format!(
                    "Invalid directive: {directive}",
                )))
            }
        }
        Ok(())
    }

    /// Parse a routing command
    /// Format: routing <mode>
    fn parse_routing(&mut self, tokens: &[&str]) -> Result<(), ParserError> {
        if tokens.len() != 2 {
            return Err(ParserError::BadFormat);
        }

        self.routing_mode = RoutingType::try_from(tokens[1])?;
        Ok(())
    }

    /// Parse a route command
    /// Format: route <prefix> via <addr>
    fn parse_route(&mut self, tokens: &[&str]) -> Result<(), ParserError> {
        if tokens.len() != 4 {
            return Err(ParserError::BadFormat);
        }

        let prefix: Ipv4Net = tokens[1].parse()?;
        if tokens[2] != "via" {
            return Err(ParserError::MissingToken(String::from("via")));
        }
        let addr: Ipv4Addr = tokens[3].parse()?;

        self.static_routes.push((prefix, addr));
        Ok(())
    }

    /// Parse a RIP command
    /// Format: rip <command> <value>
    fn parse_rip(&mut self, tokens: &[&str]) -> Result<(), ParserError> {
        if tokens.len() != 3 {
            return Err(ParserError::BadFormat);
        }

        let command = tokens[1];

        match command {
            "advertise-to" => {
                let addr: Ipv4Addr = tokens[2].parse()?;
                let matching_neighbor = self.neighbors.iter().find(|n| n.dest_addr == addr);
                match matching_neighbor {
                    Some(neighbor) => {
                        if let Some(rip_neighbors) = &mut self.rip_neighbors {
                            rip_neighbors.push(neighbor.dest_addr);
                        } else {
                            // If rip_neighbors is None, create a new vec with the
                            // neighbor's address
                            self.rip_neighbors = Some(vec![neighbor.dest_addr]);
                        }
                    }
                    None => {
                        return Err(ParserError::Other(format!(
                            "No neighbor with address {addr}"
                        )))
                    }
                }
            }
            "periodic-update-rate" => {
                let rate: u64 = tokens[2].parse()?;
                self.rip_periodic_update_rate = Some(rate);
            }
            "route-timeout-threshold" => {
                let threshold: u64 = tokens[2].parse()?;
                self.rip_timeout_threshold = Some(threshold);
            }
            _ => {
                return Err(ParserError::Other(format!("Invalid command: {command}")));
            }
        }
        Ok(())
    }

    /// Parse a TCP command
    /// Format: tcp <property> <value>
    fn parse_tcp(&mut self, tokens: &[&str]) -> Result<(), ParserError> {
        if tokens.len() != 3 {
            return Err(ParserError::BadFormat);
        }

        let property = tokens[1];
        let value: u64 = tokens[2].parse()?;

        match property {
            "rto-min" => self.tcp_rto_min = Some(value),
            "rto-max" => self.tcp_rto_max = Some(value),
            _ => {
                return Err(ParserError::Other(format!(
                    "Invalid TCP property: {property}"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_new_fields() {
        let config_str = "
# Auto-generated configuration for r2

interface if0 10.1.0.2/24 127.0.0.1:5003 # to network r1-r2
neighbor 10.1.0.1 at 127.0.0.1:5002 via if0 # r1

interface if1 10.2.0.1/24 127.0.0.1:5004 # to network r2-hosts
neighbor 10.2.0.2 at 127.0.0.1:5005 via if1 # h2
neighbor 10.2.0.3 at 127.0.0.1:5006 via if1 # h3


routing rip

# Neighbor routers that should be sent RIP messages
rip advertise-to 10.1.0.1

rip periodic-update-rate 5000
rip route-timeout-threshold 12000

tcp rto-min 250
tcp rto-max 3000
";
        let mut ip_config = IPConfig::default();
        ip_config.parse(config_str).unwrap();

        // These fields were added in F24
        assert_eq!(ip_config.rip_periodic_update_rate, Some(5000));
        assert_eq!(ip_config.rip_timeout_threshold, Some(12000));
        assert_eq!(ip_config.tcp_rto_min, Some(250));
        assert_eq!(ip_config.tcp_rto_max, Some(3000));
    }

    #[test]
    fn test_str_to_udp() {
        let (ip, port) = str_to_udp("192.168.1.1:8080");
        assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_interface_config_try_from() {
        let tokens = vec!["interface", "eth0", "192.168.1.1/24", "10.0.0.1:9000"];
        let config = InterfaceConfig::try_from(tokens).unwrap();

        assert_eq!(config.name, "eth0");
        assert_eq!(config.assigned_prefix.to_string(), "192.168.1.1/24");
        assert_eq!(config.assigned_ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(config.udp_addr, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(config.udp_port, 9000);
    }

    #[test]
    fn test_interface_config_invalid() {
        let tokens = vec!["interface", "eth0", "invalid_ip", "10.0.0.1:9000"];
        let result = InterfaceConfig::try_from(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_neighbor_config_try_from() {
        let tokens = vec![
            "neighbor",
            "192.168.1.2",
            "at",
            "10.0.0.2:9001",
            "via",
            "eth1",
        ];
        let config = NeighborConfig::try_from(tokens).unwrap();

        assert_eq!(config.dest_addr, Ipv4Addr::new(192, 168, 1, 2));
        assert_eq!(config.udp_addr, Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(config.udp_port, 9001);
        assert_eq!(config.interface_name, "eth1");
    }

    #[test]
    fn test_neighbor_config_invalid() {
        let tokens = vec![
            "neighbor",
            "192.168.1.2",
            "at",
            "10.0.0.2:9001",
            "invalid",
            "eth1",
        ];
        let result = NeighborConfig::try_from(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_routing() {
        let mut config = IPConfig::default();
        config.parse_routing(&["routing", "static"]).unwrap();
        assert_eq!(config.routing_mode, RoutingType::Static);
    }

    #[test]
    fn test_parse_route() {
        let mut config = IPConfig::default();
        config
            .parse_route(&["route", "192.168.1.0/24", "via", "10.0.0.1"])
            .unwrap();
        assert_eq!(config.static_routes.len(), 1);
        assert_eq!(config.static_routes[0].0.to_string(), "192.168.1.0/24");
        assert_eq!(config.static_routes[0].1, Ipv4Addr::new(10, 0, 0, 1));
    }

    #[test]
    fn test_parse_rip() {
        let mut config = IPConfig::default();
        config.neighbors.push(NeighborConfig {
            dest_addr: Ipv4Addr::new(192, 168, 1, 2),
            udp_addr: Ipv4Addr::new(10, 0, 0, 2),
            udp_port: 9001,
            interface_name: String::from("eth1"),
        });

        config
            .parse_rip(&["rip", "advertise-to", "192.168.1.2"])
            .unwrap();
        assert!(config.rip_neighbors.is_some());
        assert_eq!(
            config.rip_neighbors.unwrap()[0],
            Ipv4Addr::new(192, 168, 1, 2)
        );
    }

    #[test]
    fn test_parse_rip_no_matching_neighbor() {
        let mut config = IPConfig::default();
        let result = config.parse_rip(&["rip", "advertise-to", "192.168.1.2"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_line_comment() {
        let mut config = IPConfig::default();
        let result = config.parse_line("# This is a comment");
        assert!(result.is_ok());
        assert!(config.interfaces.is_empty());
    }

    #[test]
    fn test_parse_line_interface() {
        let mut config = IPConfig::default();
        config
            .parse_line("interface eth0 192.168.1.1/24 10.0.0.1:9000")
            .unwrap();
        assert_eq!(config.interfaces.len(), 1);
        assert_eq!(config.interfaces[0].name, "eth0");
    }
}
