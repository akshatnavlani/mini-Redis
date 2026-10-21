// Entry point file.
//
// Add imports for:
mod resp;
mod commands;
mod store;
mod server;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use clap::Parser;
// - clap, to parse port, maximum-key, and sweep-interval options.
// - tokio, to create the async runtime.
// - tracing-subscriber, to configure structured logging.
// - the server module, to start accepting TCP clients.
//
#[derive(Debug, Parser)]
#[command(name="mini-redis")]
#[command(about= "A small Redis-compatible server")]
struct Config{
    #[arg(long,default_value = "127.0.0.1:6380")]
    bind_address:SocketAddr,
    #[arg(long,default_value_t = 10_000)]
    max_keys:usize,
    #[arg(long,default_value_t = 100)]
    sweep_interval_ms: u64,
}
// Function reference: MAIN-001
// Function name: main
// Input: process arguments, read through the CLI configuration type.
// Output: (), after starting the server until shutdown or fatal error.
// Description: Parse configuration, initialize logging, create the Tokio
// runtime, and hand control to the server module.
// Why it exists: Keep startup concerns in one place and keep networking logic
// testable outside the process entry point.
#[tokio::main]
async fn main()-> io::Result<()>{
    let config = Config::parse();
    tracing_subscriber::fmt::init();
    if config.sweep_interval_ms==0{
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sweep-interval-ms must be greater than zero",
        ));
    }
    let store_handle = store::new_store(config.max_keys).map_err(|error| {
        io::Error::new(io::ErrorKind::InvalidInput,format!("invalid store configuration: {error:?}"),)
    })?;
    let sweep_interval = Duration::from_millis(config.sweep_interval_ms);
    server::run(config.bind_address,store_handle,sweep_interval).await

}