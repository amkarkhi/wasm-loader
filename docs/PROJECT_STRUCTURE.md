# Project Structure

Complete overview of the project organization and file structure.

## Directory Layout

```
creatingly-wasm/
├── core/                          # Core server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Server entry point
│       ├── binary_registry.rs    # Binary management & caching
│       ├── executor.rs           # Async execution engine
│       ├── server.rs             # Business logic
│       └── socket_core.rs        # Unix socket server
│
├── client/                        # Client CLI
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # CLI commands
│       └── socket_client.rs      # Unix socket client
│
├── shared/                        # Shared types & utilities
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                # Common types, traits
│
├── plugin-example/                # Example plugin (reverser)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── plugin-uppercase/              # Uppercase plugin
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── plugin-rot13/                  # ROT13 cipher plugin
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── plugin-counter/                # Stateful counter plugin
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── plugins/                       # Compiled WASM binaries
│   ├── reverser.wasm
│   ├── uppercase.wasm
│   ├── rot13.wasm
│   └── counter.wasm
│
├── tests/                         # Integration tests
│   ├── Cargo.toml
│   └── integration_tests.rs
│
├── docs/                          # Documentation
│   ├── QUICKSTART.md
│   ├── ARCHITECTURE.md
│   ├── PROJECT_STRUCTURE.md
│   ├── API_REFERENCE.md
│   └── PLUGIN_DEVELOPMENT.md
│
├── scripts/                       # Build & utility scripts
│   ├── build-all-plugins.sh
│   ├── test-plugins.sh
│   ├── test-plugin-build.sh
│   └── run-tests.sh
│
├── Cargo.toml                     # Workspace configuration
├── Makefile.toml                  # Cargo-make tasks
├── metadata.json                  # Binary metadata (runtime)
└── README.md                      # Main documentation
```

## Core Components

### Core Server (`core/`)

The main server that manages and executes WASM binaries.

**main.rs**
- Entry point for the server
- Initializes Wasmtime engine
- Creates registry and executor
- Starts Unix socket server

**binary_registry.rs**
- Thread-safe in-memory cache for compiled modules
- UUID-based binary management
- Persistence layer for metadata
- Uses `DashMap` for lock-free concurrent access

**executor.rs**
- Async execution engine
- Binary chaining support
- Resource limits (fuel, memory, timeouts)
- Sandboxing and isolation

**server.rs**
- Business logic layer
- Request handling
- Error management
- Coordination between registry and executor

**socket_core.rs**
- Unix domain socket server
- Line-delimited JSON protocol
- Async I/O with Tokio
- Connection management

### Client (`client/`)

Command-line interface for interacting with the server.

**main.rs**
- CLI argument parsing (using `clap`)
- Command implementations:
  - `load` - Load a WASM binary
  - `execute` - Execute a binary
  - `chain` - Chain multiple binaries
  - `list` - List loaded binaries
  - `unload` - Remove a binary

**socket_client.rs**
- Unix socket client implementation
- Request/response handling
- Connection management

### Shared Library (`shared/`)

Common types and utilities used by both core and client.

**lib.rs**
- Request/response types
- Execution configuration
- Binary metadata structures
- Shared traits and utilities

### Plugins

Example WebAssembly plugins demonstrating various capabilities.

**plugin-example** (reverser)
- Simple string reversal
- Demonstrates basic plugin interface

**plugin-uppercase**
- Converts text to uppercase
- Shows string manipulation

**plugin-rot13**
- ROT13 cipher implementation
- Example of transformation logic

**plugin-counter**
- Stateful counter
- Demonstrates state management with host functions

### Tests (`tests/`)

Integration tests for the entire system.

**integration_tests.rs**
- End-to-end testing
- Load, execute, chain scenarios
- Error handling tests

## File Counts

| Category | Files | Lines of Code |
|----------|-------|---------------|
| Core server | 5 | ~800 |
| Client | 2 | ~300 |
| Shared | 1 | ~100 |
| Plugins | 4 | ~400 |
| Tests | 1 | ~200 |
| **Total** | **13** | **~1,800** |

## Dependencies

### Core Server

```toml
[dependencies]
wasmtime = "15.0"          # WebAssembly runtime
tokio = { version = "1", features = ["full"] }
tokio-util = "0.7"         # Codec utilities
futures = "0.3"            # Async traits
dashmap = "5.5"            # Concurrent HashMap
uuid = { version = "1.6", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"            # Logging
tracing-subscriber = "0.3"
```

### Client

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-util = "0.7"
futures = "0.3"
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"             # Date formatting
```

### Plugins

```toml
[dependencies]
# No dependencies - pure Rust to WASM
```

## Build Artifacts

### Development Build

```
target/
├── debug/
│   ├── wasm-core          # Core server binary
│   ├── wasm-client        # Client binary
│   └── deps/              # Dependencies
└── wasm32-unknown-unknown/
    └── debug/
        └── *.wasm         # Plugin binaries
```

### Release Build

```
target/
└── release/
    ├── wasm-core          # Optimized server
    ├── wasm-client        # Optimized client
    └── ...
```

## Configuration Files

**Cargo.toml** (workspace)
- Workspace member definitions
- Shared dependencies
- Build profiles

**Makefile.toml**
- Cargo-make task definitions
- Build automation
- Test runners

**metadata.json**
- Runtime binary metadata
- Generated automatically
- Persists loaded binaries

## Communication Protocol

**Socket Path:** `/tmp/wasm-core.sock`

**Format:** Line-delimited JSON

**Request Types:**
- `LoadBinary` - Load a WASM file
- `Execute` - Execute a binary
- `ExecuteChain` - Chain multiple binaries
- `ListBinaries` - List loaded binaries
- `UnloadBinary` - Remove a binary

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Code size | ~1,800 LOC |
| Binary size (core) | ~8 MB (release) |
| Binary size (client) | ~5 MB (release) |
| Startup time | <100ms |
| Memory usage | ~10 MB (idle) |

## Development Workflow

### Build
```bash
cargo make build
```

### Test
```bash
cargo make test
```

### Run Server
```bash
cargo make server
```

### Run Client
```bash
cargo make client -- <command>
```

## Next Steps

- [Architecture Overview](ARCHITECTURE.md) - Understand the design
- [API Reference](API_REFERENCE.md) - Complete API documentation
- [Plugin Development](PLUGIN_DEVELOPMENT.md) - Build your own plugins
