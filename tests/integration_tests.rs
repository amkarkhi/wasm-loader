use anyhow::Result;
use serde_json::to_string;
use std::process::Command as cmd;
use std::process::{Child, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use wasm_shared::*;

#[allow(dead_code)]
struct CoreServer {
    process: Child,
}

#[allow(dead_code)]
impl CoreServer {
    fn start() -> Result<Self> {
        let process = cmd::new("cargo")
            .args(["run", "-p", "wasm-core"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self { process })
    }

    fn stop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Drop for CoreServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[allow(dead_code)]
async fn create_client() -> Result<SocketClient> {
    for _ in 0..10 {
        match SocketClient::connect().await {
            Ok(client) => return Ok(client),
            Err(_) => sleep(Duration::from_millis(500)).await,
        }
    }
    Err(anyhow::anyhow!("Failed to connect to server"))
}

#[tokio::test]
async fn test_server_connection() -> Result<()> {
    println!("?? Test: Server Connection");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let _ = create_client().await?;
    println!("? Connected to server");

    Ok(())
}

#[tokio::test]
async fn test_load_single_binary() -> Result<()> {
    println!("?? Test: Load Single Binary");
    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;
    let mut client = create_client().await?;
    let response = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?;
    println!("? Loaded binary: {}", response.binary_id);
    assert!(response.size > 0);

    Ok(())
}

#[tokio::test]
async fn test_execute_single_binary() -> Result<()> {
    println!("?? Test: Execute Single Binary");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load binary
    let load_resp = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?;
    let binary_id = load_resp.binary_id;

    // Execute
    let exec_resp = client.execute(binary_id, "hello".to_string(), None).await?;

    println!("? Output: {}", exec_resp.result.output);
    assert_eq!(exec_resp.result.output, "olleh");
    assert_eq!(exec_resp.result.return_code, 0);

    Ok(())
}

#[tokio::test]
async fn test_execute_chain() -> Result<()> {
    println!("?? Test: Execute Chain");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load binaries
    let uppercase_id = client
        .load_binary("./plugins/uppercase.wasm".to_string())
        .await?
        .binary_id;

    let reverser_id = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?
        .binary_id;

    // Execute chain
    let response = client
        .execute_chain(vec![uppercase_id, reverser_id], "hello".to_string(), None)
        .await?;

    println!("? Chain results:");
    for (i, result) in response.results.iter().enumerate() {
        println!("  Step {}: {}", i + 1, result.output);
    }

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].output, "HELLO");
    assert_eq!(response.results[1].output, "OLLEH");

    Ok(())
}

#[tokio::test]
async fn test_list_binaries() -> Result<()> {
    println!("?? Test: List Binaries");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load multiple binaries
    client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?;
    client
        .load_binary("./plugins/uppercase.wasm".to_string())
        .await?;

    // List binaries
    let response = client.list_binaries().await?;

    println!("? Found {} binaries", response.binaries.len());
    assert!(response.binaries.len() >= 2);

    Ok(())
}

#[tokio::test]
async fn test_unload_binary() -> Result<()> {
    println!("?? Test: Unload Binary");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load binary
    let binary_id = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?
        .binary_id;

    // Unload binary
    let response = client.unload_binary(binary_id).await?;

    println!("? {}", response.message);
    assert!(response.message.contains("unloaded"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_executions() -> Result<()> {
    println!("?? Test: Multiple Executions");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load binary
    let binary_id = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?
        .binary_id;

    // Execute multiple times
    for i in 0..5 {
        let input = format!("test{}", i);
        let response = client.execute(binary_id, input.clone(), None).await?;

        let expected: String = input.chars().rev().collect();
        assert_eq!(response.result.output, expected);
    }

    println!("? All 5 executions successful");

    Ok(())
}

#[tokio::test]
async fn test_timeout() -> Result<()> {
    println!("?? Test: Timeout");

    let _server = CoreServer::start()?;
    sleep(Duration::from_secs(2)).await;

    let mut client = create_client().await?;

    // Load binary
    let binary_id = client
        .load_binary("./plugins/reverser.wasm".to_string())
        .await?
        .binary_id;

    // Execute with very short timeout
    let config = Some(ExecutionConfig {
        timeout_ms: 1, // 1ms - very short
        memory_limit_mb: 64,
    });

    let result = client.execute(binary_id, "test".to_string(), config).await;

    // Should either timeout or succeed very quickly
    match result {
        Ok(resp) => {
            println!(
                "? Completed within timeout: {}ms",
                resp.result.execution_time_ms
            );
        }
        Err(e) => {
            println!("? Timed out as expected: {}", e);
        }
    }

    Ok(())
}

// Add socket client implementation for tests
use futures::{SinkExt, StreamExt};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

const SOCKET_PATH: &str = "/tmp/wasm-core.sock";

pub struct SocketClient {
    framed: Framed<UnixStream, LinesCodec>,
}

impl SocketClient {
    pub async fn connect() -> Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH).await?;
        let framed = Framed::new(stream, LinesCodec::new());
        Ok(Self { framed })
    }

    pub async fn load_binary(&mut self, path: String) -> Result<LoadBinaryResponse> {
        let command = Command::LoadBinary(LoadBinaryRequest { path });
        let response = {
            let this = &mut *self;
            async move {
                let json = to_string(&command)?;
                this.framed.send(json).await?;
                let line = match this.framed.next().await {
                    Some(Ok(v)) => v,
                    Some(Err(e)) => return Err(e.into()),
                    None => return Err(anyhow::anyhow!("Connection closed")),
                };
                let response: Response = serde_json::from_str(&line)?;
                Ok(response)
            }
        }
        .await?;
        match response {
            Response::LoadBinary(Ok(resp)) => Ok(resp),
            Response::LoadBinary(Err(e)) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
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
        let response = {
            let this = &mut *self;
            async move {
                let json = to_string(&command)?;
                this.framed.send(json).await?;
                let line = match this.framed.next().await {
                    Some(Ok(line)) => line,
                    Some(Err(e)) => return Err(anyhow::anyhow!("Codec error: {}", e)),
                    None => return Err(anyhow::anyhow!("Connection closed")),
                };
                let response: Response = serde_json::from_str(&line)?;
                Ok(response)
            }
        }
        .await?;
        match response {
            Response::Execute(Ok(resp)) => Ok(resp),
            Response::Execute(Err(e)) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
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
        let response = {
            let this = &mut *self;
            async move {
                let json = to_string(&command)?;
                this.framed.send(json).await?;
                let line = match this.framed.next().await {
                    Some(Ok(line)) => line,
                    Some(Err(e)) => return Err(anyhow::anyhow!("Codec error: {}", e)),
                    None => return Err(anyhow::anyhow!("Connection closed")),
                };
                let response: Response = serde_json::from_str(&line)?;
                Ok(response)
            }
        }
        .await?;
        match response {
            Response::ExecuteChain(Ok(resp)) => Ok(resp),
            Response::ExecuteChain(Err(e)) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub async fn list_binaries(&mut self) -> Result<ListBinariesResponse> {
        let command = Command::ListBinaries;
        let response = {
            let this = &mut *self;
            async move {
                let json = to_string(&command)?;
                this.framed.send(json).await?;
                let line = match this.framed.next().await {
                    Some(Ok(line)) => line,
                    Some(Err(e)) => return Err(anyhow::anyhow!("Codec error: {}", e)),
                    None => return Err(anyhow::anyhow!("Connection closed")),
                };
                let response: Response = serde_json::from_str(&line)?;
                Ok(response)
            }
        }
        .await?;
        match response {
            Response::ListBinaries(Ok(resp)) => Ok(resp),
            Response::ListBinaries(Err(e)) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub async fn unload_binary(&mut self, binary_id: Uuid) -> Result<UnloadBinaryResponse> {
        let command = Command::UnloadBinary(UnloadBinaryRequest { binary_id });
        let response = {
            let this = &mut *self;
            async move {
                let json = to_string(&command)?;
                this.framed.send(json).await?;
                let line = match this.framed.next().await {
                    Some(Ok(line)) => line,
                    Some(Err(e)) => return Err(anyhow::anyhow!("Codec error: {}", e)),
                    None => return Err(anyhow::anyhow!("Connection closed")),
                };
                let response: Response = serde_json::from_str(&line)?;
                Ok(response)
            }
        }
        .await?;
        match response {
            Response::UnloadBinary(Ok(resp)) => Ok(resp),
            Response::UnloadBinary(Err(e)) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
}
