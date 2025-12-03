use anyhow::{anyhow, Context, Result};
use rand::random;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use wasm_shared::{ExecutionConfig, ExecutionResult};
use wasmtime::*;

use crate::binary_registry::{BinaryRegistry, LoadedBinary};

pub struct Executor {
    registry: BinaryRegistry,
}

impl Executor {
    pub fn new(registry: BinaryRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        binary_id: Uuid,
        input: String,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        tracing::info!("Executing binary: {}", binary_id);
        let binary = self.registry.get_binary(&binary_id)?;
        let result = timeout(
            Duration::from_millis(config.timeout_ms),
            self.execute_binary(binary, input, config),
        )
        .await
        .context("Execution timeout")??;
        let execution_time_ms = start.elapsed().as_millis() as u64;
        tracing::info!(
            "Execution completed: {} ({}ms, fuel: {})",
            binary_id,
            execution_time_ms,
            result.fuel_consumed
        );
        Ok(ExecutionResult {
            binary_id,
            return_code: result.return_code,
            output: result.output,
            execution_time_ms,
            fuel_consumed: result.fuel_consumed,
        })
    }

    pub async fn execute_chain(
        &self,
        binary_ids: Vec<Uuid>,
        initial_input: String,
        config: ExecutionConfig,
    ) -> Result<Vec<ExecutionResult>> {
        tracing::info!("Executing binary chain: {} binaries", binary_ids.len());
        let mut results = Vec::new();
        let mut current_input = initial_input;
        for (index, binary_id) in binary_ids.iter().enumerate() {
            tracing::info!(
                "Chain step {}/{}: {}",
                index + 1,
                binary_ids.len(),
                binary_id
            );
            let result = self
                .execute(*binary_id, current_input.clone(), config.clone())
                .await?;
            current_input = result.output.clone();
            results.push(result);
        }
        tracing::info!("Chain execution completed: {} steps", results.len());
        Ok(results)
    }

    async fn execute_binary(
        &self,
        binary: LoadedBinary,
        input: String,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        let mut store = Store::new(self.registry.engine(), HostState::new());
        let fuel_limit = config.timeout_ms * 1_000_000;
        store.set_fuel(fuel_limit)?;
        let mut linker = Linker::new(self.registry.engine());
        linker.func_wrap_async(
            "host",
            "log",
            |mut caller: Caller<'_, HostState>, (ptr, len): (i32, i32)| {
                Box::new(async move {
                    let mem = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .ok_or_else(|| anyhow!("No memory export"))?;
                    let mut buf = vec![0u8; len as usize];
                    mem.read(&caller, ptr as usize, &mut buf)?;
                    let message = std::str::from_utf8(&buf).context("Invalid UTF-8")?;
                    caller.data_mut().logs.push(message.to_string());
                    tracing::debug!("[Plugin Log]: {}", message);
                    Ok(())
                })
            },
        )?;

        linker.allow_shadowing(true);

        let instance = linker
            .instantiate_async(&mut store, &binary.module)
            .await
            .map_err(|e| {
                tracing::error!("Instantiation error: {:?}", e);
                anyhow!("Failed to instantiate module: {}. Check that all required imports are satisfied.", e)
            })?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("Plugin must export 'memory'"))?;
        let memory_size_mb = (memory.size(&store) * 64 * 1024) / (1024 * 1024);
        if memory_size_mb > config.memory_limit_mb {
            return Err(anyhow!(
                "Memory limit exceeded: {} MB > {} MB",
                memory_size_mb,
                config.memory_limit_mb
            ));
        }
        let input_bytes = input.as_bytes();
        memory
            .write(&mut store, 0, input_bytes)
            .context("Failed to write input to memory")?;
        let env_json = Self::env_json().context("Failed to generate environment JSON")?;
        let env_bytes = env_json.as_bytes();
        memory
            .write(&mut store, input_bytes.len(), env_bytes)
            .context("Failed to write env JSON to memory")?;
        let process_func = instance
            .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "process")
            .context("Plugin must export 'process(i32, i32, i32, i32) -> i32'")?;
        let return_code = process_func
            .call_async(
                &mut store,
                (
                    0,
                    input_bytes.len() as i32,
                    input_bytes.len() as i32,
                    env_bytes.len() as i32,
                ),
            )
            .await
            .context("Plugin execution failed")?;
        let fuel_consumed = fuel_limit - store.get_fuel().unwrap_or(0);
        let output = store.data().logs.join("\n");
        Ok(ExecutionResult {
            binary_id: binary.metadata.id,
            return_code,
            output,
            execution_time_ms: 0, // Will be set by caller
            fuel_consumed,
        })
    }

    fn env_json() -> Result<String> {
        // Placeholder for environment JSON generation logic
        let now = std::time::SystemTime::now();
        let timestamp = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;

        let random_seed = random::<i64>();
        let mut env = serde_json::Map::new();
        env.insert("timestamp".to_string(), serde_json::json!(timestamp));
        env.insert("random_seed".to_string(), serde_json::json!(random_seed));
        let json = serde_json::to_string(&env).context("Failed to serialize env to JSON")?;
        Ok(json)
    }
}

#[derive(Default)]
struct HostState {
    logs: Vec<String>,
}

impl HostState {
    fn new() -> Self {
        Self::default()
    }
}
