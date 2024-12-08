use network::{IPConfig, Router};
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] != "--config" {
        eprintln!("Usage: vrouter --config <lnx file>");
        process::exit(1);
    }

    let lnx_file = (&args[2]).to_string();
    let ip_config = IPConfig::new(lnx_file);
    let router = Router::new(ip_config);

    router.start();
}
