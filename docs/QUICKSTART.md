# Quick Start Guide

Get up and running with WASM Core in 5 minutes.

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Cargo

## Installation

```bash
# Add wasm32 target if you don't have it
rustup target add wasm32-unknown-unknown

# Optional: Install cargo-make for easier builds
cargo install cargo-make
```

## Build Everything

```bash
# Using cargo-make (recommended)
cargo make build

# Or manually
cargo build --release -p wasm-core
cargo build --release -p wasm-client
./build-all-plugins.sh
```

## Quick Start

### Step 1: Start the Server

Open a terminal and start the core server:

```bash
cargo run -p wasm-core
```

You should see:
```
🚀 Starting WASM Core Server
━━━━━━━━━━━━━━━━━━━━━━━━
✓ Wasmtime engine initialized
✓ Binary registry created
✓ IPC initialized
🎯 Server listening on /tmp/wasm-core.sock
```

### Step 2: Load a Plugin

In a new terminal, load a WASM plugin:

```bash
cargo run -p wasm-client -- load --path ./plugins/uppercase.wasm
```

Output:
```
✓ Binary loaded successfully
Binary ID: 550e8400-e29b-41d4-a716-446655440000
```

**Save this UUID!** You'll need it for execution.

### Step 3: Execute the Plugin

```bash
cargo run -p wasm-client -- execute \
  --binary-id 550e8400-e29b-41d4-a716-446655440000 \
  --input "hello world"
```

Output:
```
✓ Execution completed!
Return code: 0
Output: HELLO WORLD
Execution time: 2ms
```

## Available Commands

### Load a Binary
```bash
cargo run -p wasm-client -- load --path <path-to-wasm>
```

### Execute a Binary
```bash
cargo run -p wasm-client -- execute \
  --binary-id <uuid> \
  --input <string>
```

### Chain Multiple Binaries
```bash
cargo run -p wasm-client -- chain \
  --binary-ids <uuid1>,<uuid2>,<uuid3> \
  --input <string>
```

### List Loaded Binaries
```bash
cargo run -p wasm-client -- list
```

### Unload a Binary
```bash
cargo run -p wasm-client -- unload --binary-id <uuid>
```

## Example: Chain Plugins

Load multiple plugins and chain them together:

```bash
# Load plugins
ID1=$(cargo run -p wasm-client -- load --path ./plugins/uppercase.wasm | grep "Binary ID:" | awk '{print $3}')
ID2=$(cargo run -p wasm-client -- load --path ./plugins/reverser.wasm | grep "Binary ID:" | awk '{print $3}')

# Chain them: uppercase → reverse
cargo run -p wasm-client -- chain \
  --binary-ids $ID1,$ID2 \
  --input "hello"

# Output:
# Step 1: HELLO (uppercased)
# Step 2: OLLEH (reversed)
```

## Available Plugins

The project includes several example plugins:

- **uppercase.wasm** - Converts text to uppercase
- **reverser.wasm** - Reverses text
- **rot13.wasm** - ROT13 cipher
- **counter.wasm** - Stateful counter (demonstrates state management)

## Next Steps

- [Architecture Overview](ARCHITECTURE.md) - Understand how it works
- [Plugin Development](PLUGIN_DEVELOPMENT.md) - Build your own plugins
- [API Reference](API_REFERENCE.md) - Complete API documentation

## Troubleshooting

### Server won't start
```bash
# Clean the socket file
rm /tmp/wasm-core.sock

# Restart server
cargo run -p wasm-core
```

### Build fails
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Plugin not found
```bash
# Build all plugins
./build-all-plugins.sh

# Or build a specific plugin
cd plugin-uppercase
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/plugin_uppercase.wasm ../plugins/uppercase.wasm
```
