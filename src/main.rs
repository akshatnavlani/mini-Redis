// Entry point file.
//
// Add imports for:
mod resp;
mod commands;
mod store;
mod server;
// - clap, to parse port, maximum-key, and sweep-interval options.
// - tokio, to create the async runtime.
// - tracing-subscriber, to configure structured logging.
// - the server module, to start accepting TCP clients.
//
// Function reference: MAIN-001
// Function name: main
// Input: process arguments, read through the CLI configuration type.
// Output: (), after starting the server until shutdown or fatal error.
// Description: Parse configuration, initialize logging, create the Tokio
// runtime, and hand control to the server module.
// Why it exists: Keep startup concerns in one place and keep networking logic
// testable outside the process entry point.
fn main(){
    println!("Hello, mini-Redis!");
}