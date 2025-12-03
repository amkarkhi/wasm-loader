mod binary_registry;
mod executor;
mod server;
mod socket_core;

use anyhow::Result;
use wasmtime::{Config, Engine};

use crate::binary_registry::BinaryRegistry;
use crate::executor::Executor;
use crate::server::Server;
use crate::socket_core::SocketServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("?? Starting WASM Core Server");
    tracing::info!("??????????????????????????????????????????");

    // Initialize Wasmtime engine
    let mut config = Config::new();
    config.async_support(true);
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    tracing::info!("? Wasmtime engine initialized");

    // Create binary registry
    let registry = BinaryRegistry::new(engine);
    tracing::info!("? Binary registry created");

    // Load existing binaries from metadata
    if let Err(e) = registry.load() {
        tracing::warn!("No existing metadata found: {}", e);
    } else {
        tracing::info!("? Loaded {} existing binaries", registry.count());
        registry.print_binaries()?;
    }

    // Create executor
    let executor = Executor::new(registry.clone());
    tracing::info!("? Executor created");

    // Create server
    let server = Server::new(registry, executor);
    tracing::info!("? Server created");

    let socket_server = SocketServer::new(server);
    tracing::info!("? Socket server initialized");

    tracing::info!("??????????????????????????????????????????");
    tracing::info!("?? Server listening on /tmp/wasm-core.sock");
    tracing::info!("?? Use wasm-client to interact with the server");
    tracing::info!("??????????????????????????????????????????");

    // Start listening
    socket_server.listen().await?;

    Ok(())
}
