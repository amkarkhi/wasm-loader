use anyhow::{anyhow, Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use wasmtime::{Engine, Module};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryMetadata {
    pub id: Uuid,
    pub path: PathBuf,
    pub size: usize,
    pub loaded_at: std::time::SystemTime,
}

#[derive(Clone)]
pub struct LoadedBinary {
    pub metadata: BinaryMetadata,
    pub module: Module,
}

#[derive(Clone)]
pub struct BinaryRegistry {
    binaries: Arc<DashMap<Uuid, LoadedBinary>>,
    engine: Engine,
}

impl BinaryRegistry {
    pub fn new(engine: Engine) -> Self {
        Self {
            binaries: Arc::new(DashMap::new()),
            engine,
        }
    }

    pub async fn load_binary(&self, path: impl AsRef<Path>) -> Result<Uuid> {
        let path = path.as_ref();
        
        // Check if a binary with the same path already exists
        if let Some(existing_id) = self.find_binary_by_path(path) {
            tracing::info!(
                "Binary with path {} already exists (id: {}), updating...",
                path.display(),
                existing_id
            );
            
            // Read and compile the new WASM file
            let wasm_bytes = tokio::fs::read(path)
                .await
                .with_context(|| format!("Failed to read WASM file: {}", path.display()))?;
            let size = wasm_bytes.len();
            let module = Module::from_binary(&self.engine, &wasm_bytes)
                .context("Failed to compile WASM module")?;
            
            // Update the existing entry with the same UUID
            let metadata = BinaryMetadata {
                id: existing_id,
                path: path.to_path_buf(),
                size,
                loaded_at: std::time::SystemTime::now(),
            };
            let loaded = LoadedBinary {
                metadata: metadata.clone(),
                module,
            };
            self.binaries.insert(existing_id, loaded);
            
            tracing::info!(
                "Binary updated successfully: {} (size: {} bytes, id: {})",
                path.display(),
                size,
                existing_id
            );
            self.save()?;
            return Ok(existing_id);
        }
        
        // No existing binary found, create a new one
        tracing::info!("Loading new binary from: {}", path.display());
        let wasm_bytes = tokio::fs::read(path)
            .await
            .with_context(|| format!("Failed to read WASM file: {}", path.display()))?;
        let size = wasm_bytes.len();
        let module = Module::from_binary(&self.engine, &wasm_bytes)
            .context("Failed to compile WASM module")?;
        let id = Uuid::new_v4();
        let metadata = BinaryMetadata {
            id,
            path: path.to_path_buf(),
            size,
            loaded_at: std::time::SystemTime::now(),
        };
        let loaded = LoadedBinary {
            metadata: metadata.clone(),
            module,
        };
        self.binaries.insert(id, loaded);
        tracing::info!(
            "Binary loaded successfully: {} (size: {} bytes, id: {})",
            path.display(),
            size,
            id
        );
        self.save()?;
        Ok(id)
    }

    pub fn get_binary(&self, id: &Uuid) -> Result<LoadedBinary> {
        self.binaries
            .get(id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| anyhow!("Binary not found: {}", id))
    }

    pub fn find_binary_by_path(&self, path: impl AsRef<Path>) -> Option<Uuid> {
        let path = path.as_ref();
        self.binaries
            .iter()
            .find(|entry| entry.value().metadata.path == path)
            .map(|entry| entry.value().metadata.id)
    }

    pub fn unload_binary(&self, id: &Uuid) -> Result<()> {
        self.binaries
            .remove(id)
            .ok_or_else(|| anyhow!("Binary not found: {}", id))?;
        tracing::info!("Binary unloaded: {}", id);
        self.save()?;
        Ok(())
    }

    pub fn list_binaries(&self) -> Vec<BinaryMetadata> {
        self.binaries
            .iter()
            .map(|entry| entry.value().metadata.clone())
            .collect()
    }

    pub fn print_binaries(&self) -> Result<()> {
        tracing::info!("Loaded Binaries:");
        self.list_binaries().iter().for_each(|meta| {
            println!(
                "ID: {}, Path: {}, Size: {} bytes, Loaded At: {:?}",
                meta.id,
                meta.path.display(),
                meta.size,
                meta.loaded_at
            );
        });
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.binaries.len()
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn save(&self) -> Result<()> {
        let metadata: Vec<_> = self
            .binaries
            .iter()
            .map(|entry| entry.metadata.clone())
            .collect();
        let json = serde_json::to_string(&metadata).context("Failed to serialize metadata")?;
        std::fs::write("metadata.json", json).context("Failed to write metadata file")?;
        tracing::info!("Binary registry metadata saved");
        Ok(())
    }

    pub fn load(&self) -> Result<()> {
        let data =
            std::fs::read_to_string("metadata.json").context("Failed to read metadata file")?;
        let metadata: Vec<BinaryMetadata> =
            serde_json::from_str(&data).context("Failed to deserialize metadata")?;
        for meta in metadata {
            let wasm_bytes = std::fs::read(&meta.path)
                .with_context(|| format!("Failed to read WASM file: {}", meta.path.display()))?;
            let module = Module::from_binary(&self.engine, &wasm_bytes)
                .context("Failed to compile WASM module")?;
            let loaded = LoadedBinary {
                metadata: meta.clone(),
                module,
            };
            self.binaries.insert(meta.id, loaded);
        }
        tracing::info!("Binary registry metadata loaded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;

    #[tokio::test]
    async fn test_binary_registry() {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config).unwrap();

        let registry = BinaryRegistry::new(engine);

        // Initially empty
        assert_eq!(registry.count(), 0);

        // Load a binary (would need a real WASM file for this to work)
        // let id = registry.load_binary("test.wasm").await.unwrap();
        // assert_eq!(registry.count(), 1);

        // Get binary
        // let binary = registry.get_binary(&id).unwrap();
        // assert_eq!(binary.metadata.id, id);

        // Unload
        // registry.unload_binary(&id).unwrap();
        // assert_eq!(registry.count(), 0);
    }
}
