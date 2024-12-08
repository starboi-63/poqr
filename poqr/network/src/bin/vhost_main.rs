use network::{Host, IPConfig};
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] != "--config" {
        eprintln!("Usage: vhost --config <lnx file>");
        process::exit(1);
    }

    let lnx_file = (&args[2]).to_string();
    let ip_config = IPConfig::new(lnx_file);
    let host = Host::new(ip_config);

    host.start();
}
