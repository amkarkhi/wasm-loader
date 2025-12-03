// Note: Tracer was added by AI
use anyhow::{anyhow, Context, Result};
use rand::random;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use wasm_shared::{ExecutionConfig, ExecutionResult};
use wasmtime::*;

use crate::binary_registry::{BinaryRegistry, LoadedBinary};
use crate::tracer::{ExecutionTrace, TraceEventType, Tracer};

pub struct Executor {
    registry: BinaryRegistry,
    tracer: Tracer,
}

impl Executor {
    pub fn new(registry: BinaryRegistry) -> Self {
        Self {
            registry,
            tracer: Tracer::default(),
        }
    }

    /// Create an executor with a custom tracer configuration
    /// This is useful for advanced use cases where you want to control tracing behavior
    #[allow(dead_code)]
    pub fn with_tracer(registry: BinaryRegistry, tracer: Tracer) -> Self {
        Self { registry, tracer }
    }

    /// Get a reference to the tracer for accessing execution traces
    #[allow(dead_code)]
    pub fn tracer(&self) -> &Tracer {
        &self.tracer
    }

    pub async fn execute(
        &self,
        binary_id: Uuid,
        input: String,
        config: ExecutionConfig,
    ) -> Result<ExecutionResult> {
        // Start tracing if enabled
        let mut trace = self.tracer.start_trace(binary_id).await;

        let start = std::time::Instant::now();
        tracing::info!("Executing binary: {}", binary_id);

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::ExecutionStart,
                format!("Starting execution of binary {}", binary_id),
                Some(serde_json::json!({
                    "input_length": input.len(),
                    "timeout_ms": config.timeout_ms,
                    "memory_limit_mb": config.memory_limit_mb,
                })),
            );
        }

        let binary = match self.registry.get_binary(&binary_id) {
            Ok(b) => {
                if let Some(ref mut t) = trace {
                    t.add_event(
                        TraceEventType::LoadComplete,
                        "Binary loaded from registry".to_string(),
                        None,
                    );
                }
                b
            }
            Err(e) => {
                let error_msg = format!("Failed to get binary: {}", e);
                tracing::error!("{}", error_msg);
                if let Some(mut t) = trace {
                    t.add_event(TraceEventType::LoadError, error_msg.clone(), None);
                    t.complete(false, Some(error_msg.clone()));
                    self.tracer.complete_trace(t).await;
                }
                return Err(e);
            }
        };

        let result = match timeout(
            Duration::from_millis(config.timeout_ms),
            self.execute_binary(binary, input, config, trace.as_mut()),
        )
        .await
        {
            Ok(Ok(result)) => {
                if let Some(ref mut t) = trace {
                    t.add_event(
                        TraceEventType::ExecutionComplete,
                        "Execution completed successfully".to_string(),
                        Some(serde_json::json!({
                            "return_code": result.return_code,
                            "fuel_consumed": result.fuel_consumed,
                        })),
                    );
                }
                result
            }
            Ok(Err(e)) => {
                let error_msg = format!("Execution error: {}", e);
                tracing::error!("{}", error_msg);
                if let Some(mut t) = trace {
                    t.add_event(TraceEventType::ExecutionError, error_msg.clone(), None);
                    t.complete(false, Some(error_msg.clone()));
                    self.tracer.complete_trace(t).await;
                }
                return Err(e);
            }
            Err(_) => {
                let error_msg = "Execution timeout";
                tracing::error!("{}", error_msg);
                if let Some(mut t) = trace {
                    t.add_event(TraceEventType::ExecutionError, error_msg.to_string(), None);
                    t.complete(false, Some(error_msg.to_string()));
                    self.tracer.complete_trace(t).await;
                }
                return Err(anyhow!("Execution timeout"));
            }
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;
        tracing::info!(
            "Execution completed: {} ({}ms, fuel: {})",
            binary_id,
            execution_time_ms,
            result.fuel_consumed
        );

        if let Some(mut t) = trace {
            t.complete(true, None);
            self.tracer.complete_trace(t).await;
        }

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

            // Extract the actual result for the next plugin in the chain
            current_input = Self::extract_result(&result.output);
            tracing::debug!(
                "Chain step {} extracted output: {}",
                index + 1,
                current_input
            );

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
        mut trace: Option<&mut ExecutionTrace>,
    ) -> Result<ExecutionResult> {
        let mut store = Store::new(self.registry.engine(), HostState::new());
        let fuel_limit = config.timeout_ms * 1_000_000;
        store.set_fuel(fuel_limit)?;

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::FuelCheckpoint,
                format!("Fuel limit set: {}", fuel_limit),
                Some(serde_json::json!({"fuel_limit": fuel_limit})),
            );
        }

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

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::HostFunctionCall,
                "Host function 'log' registered".to_string(),
                None,
            );
        }

        linker.allow_shadowing(true);

        let instance = linker
            .instantiate_async(&mut store, &binary.module)
            .await
            .map_err(|e| {
                tracing::error!("Instantiation error: {:?}", e);
                anyhow!("Failed to instantiate module: {}. Check that all required imports are satisfied.", e)
            })?;

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::FunctionCall,
                "Module instantiated successfully".to_string(),
                None,
            );
        }

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("Plugin must export 'memory'"))?;
        let memory_size_mb = (memory.size(&store) * 64 * 1024) / (1024 * 1024);

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::MemoryOp,
                format!("Memory size: {} MB", memory_size_mb),
                Some(serde_json::json!({
                    "memory_size_mb": memory_size_mb,
                    "memory_limit_mb": config.memory_limit_mb,
                })),
            );
        }

        if memory_size_mb > config.memory_limit_mb {
            let error = anyhow!(
                "Memory limit exceeded: {} MB > {} MB",
                memory_size_mb,
                config.memory_limit_mb
            );
            if let Some(ref mut t) = trace {
                t.add_event(
                    TraceEventType::ExecutionError,
                    format!(
                        "Memory limit exceeded: {} MB > {} MB",
                        memory_size_mb, config.memory_limit_mb
                    ),
                    None,
                );
            }
            return Err(error);
        }

        let input_bytes = input.as_bytes();
        memory
            .write(&mut store, 0, input_bytes)
            .context("Failed to write input to memory")?;

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::MemoryOp,
                format!("Input written to memory: {} bytes", input_bytes.len()),
                Some(serde_json::json!({"input_bytes": input_bytes.len()})),
            );
        }

        let env_json = Self::env_json().context("Failed to generate environment JSON")?;
        let env_bytes = env_json.as_bytes();
        memory
            .write(&mut store, input_bytes.len(), env_bytes)
            .context("Failed to write env JSON to memory")?;

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::MemoryOp,
                format!("Environment written to memory: {} bytes", env_bytes.len()),
                Some(serde_json::json!({"env_bytes": env_bytes.len()})),
            );
        }

        let process_func = instance
            .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "process")
            .context("Plugin must export 'process(i32, i32, i32, i32) -> i32'")?;

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::FunctionCall,
                "Calling 'process' function".to_string(),
                Some(serde_json::json!({
                    "input_ptr": 0,
                    "input_len": input_bytes.len(),
                    "env_ptr": input_bytes.len(),
                    "env_len": env_bytes.len(),
                })),
            );
        }

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

        if let Some(ref mut t) = trace {
            t.add_event(
                TraceEventType::FuelCheckpoint,
                format!("Execution completed with return code: {}", return_code),
                Some(serde_json::json!({
                    "return_code": return_code,
                    "fuel_consumed": fuel_consumed,
                    "fuel_remaining": store.get_fuel().unwrap_or(0),
                })),
            );
        }

        let output = store.data().logs.join("\n");

        // Log all plugin messages to trace
        if let Some(ref mut t) = trace {
            for log in &store.data().logs {
                t.add_event(TraceEventType::PluginLog, log.clone(), None);
            }
        }

        Ok(ExecutionResult {
            binary_id: binary.metadata.id,
            return_code,
            output,
            execution_time_ms: 0, // Will be set by caller
            fuel_consumed,
        })
    }

    /// Extract the actual result from plugin output
    /// Plugins may log multiple lines, but the result is typically after "Result = "
    /// If no "Result = " marker is found, return the last non-empty line
    fn extract_result(output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();

        // Look for "Result = " marker
        for (i, line) in lines.iter().enumerate() {
            if line.contains("Result = ") {
                // The result is typically on the next line
                if i + 1 < lines.len() {
                    let result = lines[i + 1].trim();
                    if !result.is_empty() {
                        return result.to_string();
                    }
                }
                // Or it might be on the same line after the marker
                if let Some(pos) = line.find("Result = ") {
                    let result = line[pos + 9..].trim();
                    if !result.is_empty() {
                        return result.to_string();
                    }
                }
            }
        }

        // Fallback: return the last non-empty line
        lines
            .iter()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
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
