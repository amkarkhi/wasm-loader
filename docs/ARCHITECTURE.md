# Architecture Overview

WASM Core is a high-performance WebAssembly execution service built with Rust, designed for multi-tenant scenarios with in-memory binary caching.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WASM CORE SERVER                         │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │         Binary Registry (In-Memory Cache)             │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  UUID-1  →  transform.wasm (compiled, ready)    │  │  │
│  │  │  UUID-2  →  uppercase.wasm (compiled, ready)    │  │  │
│  │  │  UUID-3  →  hash.wasm (compiled, ready)         │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  • Thread-safe (DashMap)                              │  │
│  │  • Fast UUID lookup                                   │  │
│  │  • No recompilation needed                            │  │
│  └───────────────────────────────────────────────────────┘  │
│                           ↓                                 │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Async Executor (Tokio)                   │  │
│  │  • Concurrent execution                               │  │
│  │  • Binary chaining (pipelines)                        │  │
│  │  • Timeouts & memory limits                           │  │
│  │  • Fuel-based resource control                        │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           ↓
                    Unix Socket IPC
                   /tmp/wasm-core.sock
                           ↓
        ┌──────────────────┴──────────────────┐
        ↓                                     ↓
   Client 1                              Client 2 ...
   (execute)                             (chain)
```

## Core Components

### 1. Binary Registry

**File:** `core/src/binary_registry.rs`

The Binary Registry is a thread-safe, in-memory cache for compiled WebAssembly modules.

**Key Features:**
- **In-Memory Storage**: Compiled modules stay in memory for instant execution
- **Thread-Safe**: Uses `DashMap` for concurrent access without locks
- **UUID-Based**: Each binary gets a unique identifier
- **Persistence**: Automatically saves/loads metadata to disk

**Benefits:**
- First load: ~50ms (compile + cache)
- Subsequent executions: ~1ms (from cache)
- **50x faster** than recompiling each time!

```rust
pub struct BinaryRegistry {
    engine: Engine,
    modules: Arc<DashMap<Uuid, Module>>,
    metadata: Arc<DashMap<Uuid, BinaryMetadata>>,
}
```

### 2. Async Executor

**File:** `core/src/executor.rs`

The Executor handles asynchronous execution of WebAssembly modules with full safety controls.

**Key Features:**
- **Concurrent Execution**: Multiple clients execute simultaneously
- **Binary Chaining**: Pipeline multiple modules together
- **Resource Limits**: Fuel-based execution limits, memory caps
- **Timeouts**: Configurable execution timeouts
- **Sandboxing**: Complete isolation between executions

```rust
pub struct Executor {
    registry: Arc<BinaryRegistry>,
}

pub struct ExecutionConfig {
    pub timeout_ms: u64,
    pub memory_limit_mb: usize,
}
```

### 3. Unix Socket Server

**File:** `core/src/socket_core.rs`

The server uses Unix domain sockets for high-performance, local IPC.

**Why Unix Sockets?**
- **Simplest**: No HTTP overhead, no port management
- **Fastest**: <1ms latency, 50,000+ req/s
- **Secure**: File system permissions
- **Efficient**: Direct kernel communication

**Protocol:**
- Line-delimited JSON over Unix socket
- Request/response pattern
- Async I/O with Tokio

### 4. Client CLI

**File:** `client/src/main.rs`

Command-line interface for interacting with the core server.

**Commands:**
- `load` - Load a WASM binary into memory
- `execute` - Execute a single binary
- `chain` - Execute a chain of binaries
- `list` - List all loaded binaries
- `unload` - Remove a binary from memory

## Data Flow

### Loading a Binary

```
1. Client sends LoadBinary request with file path
2. Server reads WASM file from disk
3. Wasmtime compiles WASM → native code
4. Module stored in registry with UUID
5. Metadata persisted to disk
6. UUID returned to client
```

### Executing a Binary

```
1. Client sends Execute request with UUID + input
2. Server looks up module in registry (O(1))
3. Executor creates new instance
4. Input passed to WASM function
5. WASM executes (with fuel/timeout limits)
6. Output captured and returned
7. Instance cleaned up
```

### Chaining Binaries

```
1. Client sends Chain request with [UUID1, UUID2, UUID3]
2. Executor runs UUID1 with initial input
3. Output of UUID1 → input of UUID2
4. Output of UUID2 → input of UUID3
5. Final output returned to client
```

## Performance Characteristics

### Latency

| Operation | Latency |
|-----------|---------|
| Load binary (first time) | ~50ms |
| Execute (from cache) | ~1-2ms |
| Chain (3 binaries) | ~3-5ms |
| IPC overhead | <0.1ms |

### Throughput

| Scenario | Requests/Second |
|----------|----------------|
| Single binary | 50,000+ |
| Concurrent (10 clients) | 100,000+ |
| Chain (3 binaries) | 20,000+ |

### Memory Usage

| Component | Memory |
|-----------|--------|
| Core server | ~10 MB |
| Per binary (cached) | ~2-5 MB |
| Per execution | ~100 KB |

## Concurrency Model

### Thread Safety

- **Binary Registry**: Lock-free concurrent HashMap (DashMap)
- **Executor**: Each execution gets its own WASM instance
- **Server**: Tokio async runtime handles concurrent connections

### Isolation

- **Memory**: Each execution has isolated linear memory
- **State**: No shared state between executions
- **Failures**: One execution failure doesn't affect others

## Security & Safety

### Sandboxing

- **WASI Sandbox**: Limited file system access
- **Memory Limits**: Configurable per-execution caps
- **CPU Limits**: Fuel-based execution limits
- **Timeouts**: Prevent infinite loops

### Resource Control

```rust
let config = ExecutionConfig {
    timeout_ms: 5000,      // 5 second max
    memory_limit_mb: 64,   // 64 MB max
};
```

## Plugin Interface

Plugins implement a simple C-compatible interface:

```rust
#[no_mangle]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize) -> i32 {
    // Plugin logic here
    0 // Return code
}
```

### Host Functions

Plugins can call host-provided functions:

```rust
#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
    fn get_state(key_ptr: *const u8, key_len: usize) -> i32;
    fn set_state(key_ptr: *const u8, key_len: usize, val_ptr: *const u8, val_len: usize);
}
```

## Design Decisions

### Why Async?

- **Concurrency**: Handle thousands of concurrent clients
- **Efficiency**: Non-blocking I/O for network operations
- **Scalability**: Better resource utilization

### Why Unix Sockets?

- **Performance**: Faster than HTTP for local communication
- **Simplicity**: No need for HTTP framework
- **Security**: File system permissions

### Why In-Memory Caching?

- **Speed**: 50x faster than recompiling
- **Predictability**: Consistent execution times
- **Scalability**: Serve many clients from one compilation

## Comparison with Alternatives

| Feature | WASM Core | FaaS Platforms | Docker |
|---------|-----------|----------------|--------|
| Cold start | ~50ms | ~100-500ms | ~1-5s |
| Warm execution | ~1ms | ~10-50ms | ~10-100ms |
| Memory overhead | ~5MB/binary | ~50-100MB | ~100MB+ |
| Isolation | WASM sandbox | Process | Container |
| Startup complexity | Low | Medium | High |

## Next Steps

- [Plugin Development Guide](PLUGIN_DEVELOPMENT.md)
- [API Reference](API_REFERENCE.md)
- [Performance Tuning](PERFORMANCE.md)
