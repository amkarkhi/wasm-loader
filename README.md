# WASM Core: Multi-Tenant WebAssembly Execution Service

A high-performance WebAssembly execution service built in Rust with:

- **In-Memory Binary Caching** - Load once, execute many times
- **Smart Updates** - Automatically update existing binaries without creating duplicates
- **Concurrent Execution** - Multiple clients, zero blocking
- **Binary Chaining** - Create complex pipelines
- **Async Runtime** - Built on Tokio for maximum performance
- **Complete Safety** - Timeouts, memory limits, sandboxing

---

## Quick Start

```bash
# 1. Build everything
cargo build --release

# 2. Build plugins
./build-all-plugins.sh

# 3. Start server
cargo run -p wasm-core

# 4. Use client (in another terminal)
cargo run -p wasm-client -- load --path ./plugins/env-reader.wasm
# Returns UUID

cargo run -p wasm-client -- execute --binary-id <uuid> --input "hello"
# Output: HELLO

# Reload same file - updates existing binary, keeps same UUID
cargo run -p wasm-client -- load --path ./plugins/env-reader.wasm
# Returns same UUID
```

---

## Features

### 1. Smart Binary Management

```rust
// First load - creates new entry with UUID
let id = registry.load_binary("plugin.wasm").await?;

// Execute multiple times (cached, instant!)
executor.execute(id, "input1", config).await?;
executor.execute(id, "input2", config).await?;

// Reload same file - updates existing binary, keeps same UUID
let same_id = registry.load_binary("plugin.wasm").await?;
assert_eq!(id, same_id);
```

**Benefits**:

- No duplicate entries for the same file path
- Automatic updates when reloading
- UUIDs remain stable across updates
- First load: ~50ms, subsequent: ~1ms

---

### 2. Binary Chaining

Chain binaries into pipelines:

```bash
# Load plugins
wasm-client load --path ./plugin-rot13/target/.../plugin_rot13.wasm
# UUID: aaaa-1111

wasm-client load --path ./plugin-uppercase/target/.../plugin_uppercase.wasm
# UUID: bbbb-2222

# Chain them
wasm-client chain --binary-ids aaaa-1111,bbbb-2222 --input "hello"
# Step 1: uryyb (rot13)
# Step 2: URYYB (uppercase)
```

---

### 3. Complete Safety

```rust
let config = ExecutionConfig {
    timeout_ms: 5000,      // 5 second timeout
    memory_limit_mb: 64,   // 64 MB max memory
};
```

- Timeouts via fuel-based limits
- Memory caps per execution
- Complete sandboxing
- Isolated failure handling

---

## API

### Commands

```bash
# Load binary (creates new or updates existing)
wasm-client load --path <wasm-file>

# Execute binary
wasm-client execute --binary-id <uuid> --input <string> [--timeout <ms>] [--memory <mb>]

# Execute chain
wasm-client chain --binary-ids <uuid1>,<uuid2> --input <string>

# List loaded binaries
wasm-client list

# Unload binary
wasm-client unload --binary-id <uuid>
```

---

## Plugin Development

Write plugins that implement the process function:

```rust
#[no_mangle]
pub extern "C" fn process(
    input_ptr: i32,
    input_len: i32,
    env_ptr: i32,
    env_len: i32,
) -> i32 {
    // Your logic here
    0 // Return code
}
```

Use host functions for logging:

```rust
#[link(wasm_import_module = "host")]
extern "C" {
    fn log(ptr: *const u8, len: usize);
}
```

Build:

```bash
cargo build --target wasm32-unknown-unknown --release
```

See included plugins: `plugin-uppercase`, `plugin-rot13`, `plugin-counter`, `plugin-env-reader`

---

## Project Structure

```
creatingly-wasm/
├── core/                    # Core server
│   ├── src/
│   │   ├── binary_registry.rs  # Binary management with smart updates
│   │   ├── executor.rs         # Async execution engine
│   │   ├── server.rs           # Server API
│   │   └── socket_core.rs      # Unix socket communication
│   └── Cargo.toml
├── client/                  # Client CLI
│   └── src/
│       ├── main.rs
│       └── socket_client.rs
├── shared/                  # Shared types
│   └── src/lib.rs
├── plugin-*/                # Example plugins
└── tests/                   # Integration tests
```

---

## Architecture

**Binary Registry**: Thread-safe (DashMap) cache of compiled WASM modules

- Fast UUID lookups
- Path-based deduplication
- No recompilation needed

**Async Executor**: Tokio-based concurrent execution

- Fuel-based timeouts
- Memory limits
- Binary chaining support

**Communication**: Unix domain sockets for IPC

---

## License

This project is provided as an educational example for building WebAssembly execution services in Rust.
