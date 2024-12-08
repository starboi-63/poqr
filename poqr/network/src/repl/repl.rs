use super::repl_handler::ReplHandler;
use crate::network_stacks::ip::IpRepl;
use crate::network_stacks::tcp::TcpRepl;
use std::any::Any;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{io, thread};

/// A trait for implementing a Read-Eval-Print Loop (REPL) on an object
pub trait Repl: Send + Sync + Any {
    /// Wrap `self` with an `Arc` for use in threads
    fn to_arc(&self) -> Arc<dyn Repl>;

    /// Convert the REPL to an IP-Stack REPL if possible
    fn as_ip_repl(&self) -> Option<Arc<dyn IpRepl>>;

    /// Convert the REPL to a TCP-Stack REPL if possible
    fn as_tcp_repl(&self) -> Option<Arc<dyn TcpRepl>>;

    /// Get the object's REPL handlers
    fn repl_handlers(&self) -> Arc<Mutex<HashMap<String, ReplHandler>>>;

    /// Register a new handler function for a specific command
    fn register_repl_handler(&self, command: &str, handler: ReplHandler) {
        let repl_handlers = self.repl_handlers();
        let mut repl_handlers = repl_handlers.lock().unwrap();
        repl_handlers.insert(command.to_string(), handler);
    }

    /// Start the object's user-facing REPL
    fn start_repl(&self, name: &str) {
        let repl = self.to_arc();

        // Register the exit command
        repl.register_repl_handler("exit", |_, _| {
            println!("Exiting REPL...");
            std::process::exit(0);
        });

        println!("Welcome to the {} REPL. Type 'exit' to quit.", name);

        thread::spawn(move || loop {
            // Print a prompt
            print!("> ");
            io::stdout().flush().unwrap();

            // Read user input
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            // Extract the arguments from the input
            let lowercased_input = input.trim().to_ascii_lowercase();
            let args = lowercased_input.split_whitespace().collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            // Handle the input using all registered REPL handlers
            let repl_handlers = repl.repl_handlers();
            let repl_handlers = repl_handlers.lock().unwrap();

            if let Some(handler) = repl_handlers.get(args[0]) {
                handler(repl.clone(), args[1..].to_vec());
            } else {
                eprintln!("Unknown command: {}", args[0]);
            }
        });
    }
}
