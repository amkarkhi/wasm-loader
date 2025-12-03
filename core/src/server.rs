use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_shared::*;

use crate::binary_registry::BinaryRegistry;
use crate::executor::Executor;

pub struct Server {
    registry: BinaryRegistry,
    executor: Arc<RwLock<Executor>>,
}

impl Server {
    pub fn new(registry: BinaryRegistry, executor: Executor) -> Self {
        Self {
            registry,
            executor: Arc::new(RwLock::new(executor)),
        }
    }

    pub async fn load_binary(&self, req: LoadBinaryRequest) -> Result<LoadBinaryResponse> {
        tracing::info!("Loading binary from: {}", req.path);
        let binary_id = self.registry.load_binary(&req.path).await?;
        let binary = self.registry.get_binary(&binary_id)?;
        Ok(LoadBinaryResponse {
            binary_id,
            size: binary.metadata.size,
        })
    }

    pub async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse> {
        tracing::info!("Executing binary: {}", req.binary_id);
        let config = req.config.unwrap_or_default();
        let executor = self.executor.read().await;
        let result = executor.execute(req.binary_id, req.input, config).await?;
        Ok(ExecuteResponse { result })
    }

    pub async fn execute_chain(&self, req: ExecuteChainRequest) -> Result<ExecuteChainResponse> {
        tracing::info!("Executing chain: {} binaries", req.binary_ids.len());
        let config = req.config.unwrap_or_default();
        let executor = self.executor.read().await;
        let results = executor
            .execute_chain(req.binary_ids, req.input, config)
            .await?;
        Ok(ExecuteChainResponse { results })
    }

    pub async fn list_binaries(&self, _req: ListBinariesRequest) -> Result<ListBinariesResponse> {
        let binaries = self
            .registry
            .list_binaries()
            .into_iter()
            .map(|meta| BinaryInfo {
                id: meta.id,
                path: meta.path.to_string_lossy().to_string(),
                size: meta.size,
                loaded_at: meta
                    .loaded_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
            .collect();
        Ok(ListBinariesResponse { binaries })
    }

    pub async fn unload_binary(&self, req: UnloadBinaryRequest) -> Result<UnloadBinaryResponse> {
        tracing::info!("Unloading binary: {}", req.binary_id);
        self.registry.unload_binary(&req.binary_id)?;
        Ok(UnloadBinaryResponse {
            message: format!("Binary {} unloaded successfully", req.binary_id),
        })
    }
}
