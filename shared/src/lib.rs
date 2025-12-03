extern crate alloc;

pub mod plugin_helpers;

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SOCKET_PATH: &str = "/tmp/wasm-core.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub timeout_ms: u64,
    pub memory_limit_mb: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            memory_limit_mb: 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub binary_id: Uuid,
    pub return_code: i32,
    pub output: String,
    pub execution_time_ms: u64,
    pub fuel_consumed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub id: Uuid,
    pub path: String,
    pub size: usize,
    pub loaded_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBinaryRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBinaryResponse {
    pub binary_id: Uuid,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub binary_id: Uuid,
    pub input: String,
    #[serde(default)]
    pub config: Option<ExecutionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub result: ExecutionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteChainRequest {
    pub binary_ids: Vec<Uuid>,
    pub input: String,
    #[serde(default)]
    pub config: Option<ExecutionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteChainResponse {
    pub results: Vec<ExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBinariesRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBinariesResponse {
    pub binaries: Vec<BinaryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadBinaryRequest {
    pub binary_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadBinaryResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    LoadBinary(LoadBinaryRequest),
    Execute(ExecuteRequest),
    ExecuteChain(ExecuteChainRequest),
    ListBinaries,
    UnloadBinary(UnloadBinaryRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    LoadBinary(Result<LoadBinaryResponse, String>),
    Execute(Result<ExecuteResponse, String>),
    ExecuteChain(Result<ExecuteChainResponse, String>),
    ListBinaries(Result<ListBinariesResponse, String>),
    UnloadBinary(Result<UnloadBinaryResponse, String>),
    Error(String),
}
