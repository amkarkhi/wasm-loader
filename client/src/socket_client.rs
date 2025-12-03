use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};
use uuid::Uuid;

use wasm_shared::*;

pub struct SocketClient {
    framed: Framed<UnixStream, LinesCodec>,
}

impl SocketClient {
    pub async fn connect() -> Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH)
            .await
            .context("Failed to connect to server. Is wasm-core running?")?;
        let framed = Framed::new(stream, LinesCodec::new());
        Ok(Self { framed })
    }

    async fn send_command(&mut self, command: Command) -> Result<Response> {
        let json = serde_json::to_string(&command)?;
        self.framed.send(json).await?;
        let line = self.framed.next().await.context("Connection closed")??;
        let response: Response = serde_json::from_str(&line)?;
        Ok(response)
    }

    pub async fn load_binary(&mut self, path: String) -> Result<LoadBinaryResponse> {
        let command = Command::LoadBinary(LoadBinaryRequest { path });
        let response = self.send_command(command).await?;
        match response {
            Response::LoadBinary(Ok(resp)) => Ok(resp),
            Response::LoadBinary(Err(e)) => Err(anyhow::anyhow!(e)),
            Response::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn execute(
        &mut self,
        binary_id: Uuid,
        input: String,
        config: Option<ExecutionConfig>,
    ) -> Result<ExecuteResponse> {
        let command = Command::Execute(ExecuteRequest {
            binary_id,
            input,
            config,
        });
        let response = self.send_command(command).await?;
        match response {
            Response::Execute(Ok(resp)) => Ok(resp),
            Response::Execute(Err(e)) => Err(anyhow::anyhow!(e)),
            Response::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn execute_chain(
        &mut self,
        binary_ids: Vec<Uuid>,
        input: String,
        config: Option<ExecutionConfig>,
    ) -> Result<ExecuteChainResponse> {
        let command = Command::ExecuteChain(ExecuteChainRequest {
            binary_ids,
            input,
            config,
        });
        let response = self.send_command(command).await?;
        match response {
            Response::ExecuteChain(Ok(resp)) => Ok(resp),
            Response::ExecuteChain(Err(e)) => Err(anyhow::anyhow!(e)),
            Response::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn list_binaries(&mut self) -> Result<ListBinariesResponse> {
        let command = Command::ListBinaries;
        let response = self.send_command(command).await?;
        match response {
            Response::ListBinaries(Ok(resp)) => Ok(resp),
            Response::ListBinaries(Err(e)) => Err(anyhow::anyhow!(e)),
            Response::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn unload_binary(&mut self, binary_id: Uuid) -> Result<UnloadBinaryResponse> {
        let command = Command::UnloadBinary(UnloadBinaryRequest { binary_id });
        let response = self.send_command(command).await?;
        match response {
            Response::UnloadBinary(Ok(resp)) => Ok(resp),
            Response::UnloadBinary(Err(e)) => Err(anyhow::anyhow!(e)),
            Response::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }
}
