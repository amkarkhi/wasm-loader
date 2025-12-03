use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed, LinesCodec};
use wasm_shared::{Command, ListBinariesRequest, Response, SOCKET_PATH};

use crate::server::Server;

pub struct SocketServer {
    server: Arc<Server>,
}

impl SocketServer {
    pub fn new(server: Server) -> Self {
        Self {
            server: Arc::new(server),
        }
    }

    pub async fn listen(&self) -> Result<()> {
        let _ = std::fs::remove_file(SOCKET_PATH);
        let listener = UnixListener::bind(SOCKET_PATH).context("Failed to bind Unix socket")?;
        tracing::info!("?? Socket server listening on {}", SOCKET_PATH);
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let server = Arc::clone(&self.server);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, server).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(stream: UnixStream, server: Arc<Server>) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new());
    while let Some(line) = framed.next().await {
        let line = line.context("Failed to read line")?;
        let command: Command = match serde_json::from_str(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                let response = Response::Error(format!("Invalid command: {}", e));
                let json = serde_json::to_string(&response)?;
                framed.send(json).await?;
                continue;
            }
        };
        let response = process_command(command, &server).await;
        let json = serde_json::to_string(&response)?;
        framed.send(json).await?;
    }
    Ok(())
}

async fn process_command(command: Command, server: &Server) -> Response {
    match command {
        Command::LoadBinary(req) => {
            let result = server.load_binary(req).await.map_err(|e| e.to_string());
            Response::LoadBinary(result)
        }
        Command::Execute(req) => {
            let result = server.execute(req).await.map_err(|e| e.to_string());
            Response::Execute(result)
        }
        Command::ExecuteChain(req) => {
            let result = server.execute_chain(req).await.map_err(|e| e.to_string());
            Response::ExecuteChain(result)
        }
        Command::ListBinaries => {
            let result = server
                .list_binaries(ListBinariesRequest {})
                .await
                .map_err(|e| e.to_string());
            Response::ListBinaries(result)
        }
        Command::UnloadBinary(req) => {
            let result = server.unload_binary(req).await.map_err(|e| e.to_string());
            Response::UnloadBinary(result)
        }
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}
