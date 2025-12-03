mod socket_client;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use uuid::Uuid;
use wasm_shared::ExecutionConfig;

use socket_client::*;

#[derive(Parser)]
#[command(name = "wasm-client")]
#[command(about = "Client for WASM Core Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Load {
        #[arg(short, long)]
        path: PathBuf,
    },

    Execute {
        #[arg(short, long)]
        binary_id: Uuid,

        #[arg(short, long)]
        input: String,

        #[arg(short, long, default_value = "5000")]
        timeout: u64,

        #[arg(short, long, default_value = "64")]
        memory: u64,
    },

    Chain {
        #[arg(short, long, value_delimiter = ',')]
        binary_ids: Vec<Uuid>,

        #[arg(short, long)]
        input: String,

        #[arg(short, long, default_value = "5000")]
        timeout: u64,

        #[arg(short, long, default_value = "64")]
        memory: u64,
    },

    List,

    Unload {
        #[arg(short, long)]
        binary_id: Uuid,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut client = SocketClient::connect().await?;

    match cli.command {
        Commands::Load { path } => {
            println!("?? Loading binary: {}", path.display());
            println!();

            match client.load_binary(path.to_string_lossy().to_string()).await {
                Ok(response) => {
                    println!("? Binary loaded successfully!");
                    println!("Binary ID: {}", response.binary_id);
                    println!("Size: {} bytes", response.size);
                    println!();
                    println!("Use this ID to execute the binary:");
                    println!(
                        "  cargo run -p wasm-client -- execute --binary-id {} --input \"Hello\"",
                        response.binary_id
                    );
                }
                Err(e) => {
                    eprintln!("? Failed to load binary: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Execute {
            binary_id,
            input,
            timeout,
            memory,
        } => {
            println!("?? Executing binary: {}", binary_id);
            println!("Input: \"{}\"", input);
            println!("Timeout: {}ms", timeout);
            println!("Memory: {}MB", memory);
            println!();

            let config = Some(ExecutionConfig {
                timeout_ms: timeout,
                memory_limit_mb: memory,
            });

            match client.execute(binary_id, input, config).await {
                Ok(response) => {
                    println!("? Execution completed!");
                    println!("Return code: {}", response.result.return_code);
                    if !response.result.output.is_empty() {
                        println!("Output:");
                        println!("{}", response.result.output);
                    }
                    println!("Execution time: {}ms", response.result.execution_time_ms);
                    println!("Fuel consumed: {}", response.result.fuel_consumed);
                }
                Err(e) => {
                    eprintln!("? Execution failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Chain {
            binary_ids,
            input,
            timeout,
            memory,
        } => {
            println!("??  Executing chain: {} binaries", binary_ids.len());
            println!("Binary IDs:");
            for (i, id) in binary_ids.iter().enumerate() {
                println!("  {}. {}", i + 1, id);
            }
            println!();
            println!("Initial input: \"{}\"", input);
            println!("Timeout: {}ms", timeout);
            println!("Memory: {}MB", memory);
            println!();

            let config = Some(ExecutionConfig {
                timeout_ms: timeout,
                memory_limit_mb: memory,
            });

            match client.execute_chain(binary_ids, input, config).await {
                Ok(response) => {
                    println!("? Chain execution completed!");
                    println!();
                    for (i, result) in response.results.iter().enumerate() {
                        println!("Step {}: {}", i + 1, result.binary_id);
                        println!("  Return code: {}", result.return_code);
                        if !result.output.is_empty() {
                            println!("  Output: {}", result.output);
                        }
                        println!("  Execution time: {}ms", result.execution_time_ms);
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("? Chain execution failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::List => {
            println!("?? Loaded binaries:");
            println!();
            match client.list_binaries().await {
                Ok(response) => {
                    if response.binaries.is_empty() {
                        println!("No binaries loaded yet.");
                        println!();
                        println!("Load a binary with:");
                        println!(
                            "  cargo run -p wasm-client -- load --path ./plugins/transform.wasm"
                        );
                    } else {
                        println!("Found {} binaries:", response.binaries.len());
                        println!();
                        for binary in response.binaries {
                            println!("ID: {}", binary.id);
                            println!("  Path: {}", binary.path);
                            println!("  Size: {} bytes", binary.size);
                            let datetime =
                                chrono::DateTime::from_timestamp(binary.loaded_at as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| binary.loaded_at.to_string());
                            println!("  Loaded at: {}", datetime);
                            println!();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("? Failed to list binaries: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Unload { binary_id } => {
            println!("???  Unloading binary: {}", binary_id);
            println!();
            match client.unload_binary(binary_id).await {
                Ok(response) => {
                    println!("? {}", response.message);
                }
                Err(e) => {
                    eprintln!("? Failed to unload binary: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
