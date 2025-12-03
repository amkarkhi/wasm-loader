# Quick Reference

## Getting Started

### Install Cargo Make

```bash
cargo install cargo-make
```

### Build Everything

```bash
cargo make build
```

### Run Tests

```bash
cargo make test
```

### Start Server

```bash
cargo make server
```

---

## Common Commands

| Command | Description |
|---------|-------------|
| `cargo make build` | Build all components |
| `cargo make test` | Run all tests |
| `cargo make server` | Start core server |
| `cargo make client` | Run client |
| `cargo make all` | Build + test |
| `cargo make clean` | Clean everything |
| `cargo make help` | Show all tasks |

---

## Build Commands

```bash
# Build everything (release mode)
cargo make build

# Build in debug mode (faster)
cargo make build-dev

# Build specific components
cargo make build-plugins
cargo make build-core
cargo make build-client
```

---

## Test Commands

```bash
# Run unit tests
cargo make test

# Run full test suite (automated)
cargo make test-full

# Run integration tests
cargo make test-integration
```

---

## Run Commands

```bash
# Start server (release mode)
cargo make server

# Start server (debug mode, faster startup)
cargo make server-dev

# Run client
cargo make client

# Run client with args
cargo make client -- list
cargo make client -- load --path ./plugins/reverser.wasm
cargo make client -- execute --binary-id <uuid> --input "hello"
```

---

## Utility Commands

```bash
# Clean all build artifacts
cargo make clean

# Format code
cargo make format

# Run linter
cargo make lint

# Check code (no build)
cargo make check
```

---

## Quick Workflows

### First Time Setup

```bash
# Install cargo-make
cargo install cargo-make

# Build everything
cargo make build

# Run tests
cargo make test
```

### Development Workflow

```bash
# Terminal 1: Start server
cargo make server-dev

# Terminal 2: Test client
cargo make client -- load --path ./plugins/reverser.wasm
cargo make client -- list
```

### Full CI Pipeline

```bash
cargo make ci
# Runs: format, lint, build, test
```

### Create Release

```bash
cargo make release
# Creates ./release/ with binaries and plugins
```

---

## Examples

### Load and Execute Plugin

```bash
# Start server
cargo make server &

# Wait for startup
sleep 2

# Load plugin
cargo make client -- load --path ./plugins/reverser.wasm
# Copy the Binary ID

# Execute
cargo make client -- execute --binary-id <uuid> --input "hello"
```

### Chain Plugins

```bash
# Load plugins and get IDs
ID1=$(cargo make client -- load --path ./plugins/uppercase.wasm | grep "Binary ID:" | awk '{print $3}')
ID2=$(cargo make client -- load --path ./plugins/reverser.wasm | grep "Binary ID:" | awk '{print $3}')

# Chain them
cargo make client -- chain --binary-ids $ID1,$ID2 --input "hello"
```

---

## One-Liners

```bash
# Build and test
cargo make all

# Clean and rebuild
cargo make clean && cargo make build

# Format and lint
cargo make format && cargo make lint

# Full CI check
cargo make ci
```

---

## Troubleshooting

### Server Won't Start

```bash
# Clean socket
cargo make clean-socket

# Restart server
cargo make server
```

### Build Fails

```bash
# Clean and rebuild
cargo make clean
cargo make build
```

### Tests Fail

```bash
# Check code first
cargo make check

# Run tests with output
cargo test -- --nocapture
```

---

## Summary

### Install

```bash
cargo install cargo-make
```

### Use

```bash
cargo make build    # Build
cargo make test     # Test
cargo make server   # Run server
cargo make all      # Build + test
cargo make help     # Show all commands
```

**That's it!** No bash scripts needed.
