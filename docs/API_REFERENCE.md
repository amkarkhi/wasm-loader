# API Reference

Complete reference for the WASM Core API.

## Client Commands

### load

Load a WebAssembly binary into memory.

**Usage:**
```bash
cargo run -p wasm-client -- load --path <path-to-wasm>
```

**Arguments:**
- `--path <PATH>` - Path to the WASM file (required)

**Returns:**
- Binary ID (UUID)

**Example:**
```bash
$ cargo run -p wasm-client -- load --path ./plugins/uppercase.wasm
✓ Binary loaded successfully
Binary ID: 550e8400-e29b-41d4-a716-446655440000
```

**Errors:**
- File not found
- Invalid WASM format
- Compilation error

---

### execute

Execute a loaded binary with input.

**Usage:**
```bash
cargo run -p wasm-client -- execute \
  --binary-id <uuid> \
  --input <string> \
  [--timeout <ms>] \
  [--memory <mb>]
```

**Arguments:**
- `--binary-id <UUID>` - Binary identifier (required)
- `--input <STRING>` - Input string (required)
- `--timeout <MS>` - Execution timeout in milliseconds (optional, default: 5000)
- `--memory <MB>` - Memory limit in megabytes (optional, default: 64)

**Returns:**
- Return code
- Output string
- Execution time
- Fuel consumed

**Example:**
```bash
$ cargo run -p wasm-client -- execute \
  --binary-id 550e8400-e29b-41d4-a716-446655440000 \
  --input "hello world"

✓ Execution completed!
Return code: 0
Output: HELLO WORLD
Execution time: 2ms
Fuel consumed: 12,345
```

**Errors:**
- Binary not found
- Execution timeout
- Out of memory
- Runtime error

---

### chain

Execute multiple binaries in sequence, passing output as input to the next.

**Usage:**
```bash
cargo run -p wasm-client -- chain \
  --binary-ids <uuid1>,<uuid2>,<uuid3> \
  --input <string> \
  [--timeout <ms>] \
  [--memory <mb>]
```

**Arguments:**
- `--binary-ids <UUID,UUID,...>` - Comma-separated list of binary IDs (required)
- `--input <STRING>` - Initial input string (required)
- `--timeout <MS>` - Per-binary timeout in milliseconds (optional, default: 5000)
- `--memory <MB>` - Per-binary memory limit in megabytes (optional, default: 64)

**Returns:**
- Results from each binary in the chain
- Total execution time

**Example:**
```bash
$ cargo run -p wasm-client -- chain \
  --binary-ids aaaa-1111,bbbb-2222,cccc-3333 \
  --input "hello"

✓ Chain execution completed!

Step 1 (aaaa-1111):
  Output: HELLO
  Time: 2ms

Step 2 (bbbb-2222):
  Output: OLLEH
  Time: 1ms

Step 3 (cccc-3333):
  Output: BYYRУ
  Time: 2ms

Total time: 5ms
```

**Errors:**
- Binary not found
- Execution timeout
- Chain interrupted (returns partial results)

---

### list

List all loaded binaries.

**Usage:**
```bash
cargo run -p wasm-client -- list
```

**Arguments:**
None

**Returns:**
- List of loaded binaries with metadata

**Example:**
```bash
$ cargo run -p wasm-client -- list

Loaded Binaries:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ID: 550e8400-e29b-41d4-a716-446655440000
Path: ./plugins/uppercase.wasm
Size: 1,841 bytes
Loaded: 2024-11-25 10:30:45

ID: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
Path: ./plugins/reverser.wasm
Size: 1,814 bytes
Loaded: 2024-11-25 10:31:12

Total: 2 binaries
```

---

### unload

Remove a binary from memory.

**Usage:**
```bash
cargo run -p wasm-client -- unload --binary-id <uuid>
```

**Arguments:**
- `--binary-id <UUID>` - Binary identifier (required)

**Returns:**
- Success confirmation

**Example:**
```bash
$ cargo run -p wasm-client -- unload \
  --binary-id 550e8400-e29b-41d4-a716-446655440000

✓ Binary unloaded successfully
```

**Errors:**
- Binary not found

---

## Request/Response Protocol

The core server uses a line-delimited JSON protocol over Unix sockets.

### Request Format

All requests follow this structure:

```json
{
  "type": "RequestType",
  "payload": { ... }
}
```

### Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": { ... }
}
```

Or for errors:

```json
{
  "success": false,
  "error": "Error message"
}
```

---

## Request Types

### LoadBinary

Load a WASM binary from disk.

**Request:**
```json
{
  "type": "LoadBinary",
  "payload": {
    "path": "./plugins/uppercase.wasm"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "binary_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

### Execute

Execute a loaded binary.

**Request:**
```json
{
  "type": "Execute",
  "payload": {
    "binary_id": "550e8400-e29b-41d4-a716-446655440000",
    "input": "hello world",
    "config": {
      "timeout_ms": 5000,
      "memory_limit_mb": 64
    }
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "return_code": 0,
    "output": "HELLO WORLD",
    "execution_time_ms": 2,
    "fuel_consumed": 12345
  }
}
```

---

### ExecuteChain

Execute multiple binaries in sequence.

**Request:**
```json
{
  "type": "ExecuteChain",
  "payload": {
    "binary_ids": [
      "aaaa-1111",
      "bbbb-2222",
      "cccc-3333"
    ],
    "input": "hello",
    "config": {
      "timeout_ms": 5000,
      "memory_limit_mb": 64
    }
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "results": [
      {
        "binary_id": "aaaa-1111",
        "return_code": 0,
        "output": "HELLO",
        "execution_time_ms": 2
      },
      {
        "binary_id": "bbbb-2222",
        "return_code": 0,
        "output": "OLLEH",
        "execution_time_ms": 1
      },
      {
        "binary_id": "cccc-3333",
        "return_code": 0,
        "output": "BYYRУ",
        "execution_time_ms": 2
      }
    ],
    "total_time_ms": 5
  }
}
```

---

### ListBinaries

List all loaded binaries.

**Request:**
```json
{
  "type": "ListBinaries",
  "payload": {}
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "binaries": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "path": "./plugins/uppercase.wasm",
        "size": 1841,
        "loaded_at": "2024-11-25T10:30:45Z"
      },
      {
        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "path": "./plugins/reverser.wasm",
        "size": 1814,
        "loaded_at": "2024-11-25T10:31:12Z"
      }
    ]
  }
}
```

---

### UnloadBinary

Remove a binary from memory.

**Request:**
```json
{
  "type": "UnloadBinary",
  "payload": {
    "binary_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {}
}
```

---

## Error Codes

| Error | Description |
|-------|-------------|
| `BinaryNotFound` | The specified binary ID doesn't exist |
| `FileNotFound` | The WASM file doesn't exist |
| `InvalidWasm` | The file is not valid WebAssembly |
| `CompilationError` | Failed to compile the WASM module |
| `ExecutionTimeout` | Execution exceeded timeout limit |
| `OutOfMemory` | Execution exceeded memory limit |
| `RuntimeError` | Error during WASM execution |
| `InvalidInput` | Invalid request parameters |

---

## Configuration

### ExecutionConfig

Controls execution behavior and resource limits.

```rust
pub struct ExecutionConfig {
    pub timeout_ms: u64,        // Execution timeout (default: 5000)
    pub memory_limit_mb: usize, // Memory limit (default: 64)
}
```

**Defaults:**
- `timeout_ms`: 5000 (5 seconds)
- `memory_limit_mb`: 64 MB

---

## Limits

| Resource | Default | Maximum |
|----------|---------|---------|
| Execution timeout | 5s | 60s |
| Memory per execution | 64 MB | 512 MB |
| Input size | - | 10 MB |
| Output size | - | 10 MB |
| Chain length | - | 10 binaries |
| Concurrent executions | - | 1000 |

---

## Next Steps

- [Plugin Development Guide](PLUGIN_DEVELOPMENT.md) - Build your own plugins
- [Architecture Overview](ARCHITECTURE.md) - Understand the system design
- [Quick Start](QUICKSTART.md) - Get started quickly
