#!/bin/bash

# Build all WASM plugins

set -e

echo "🔧 Building all WASM plugins..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Ensure plugins directory exists
mkdir -p plugins

# Plugin 1: String Reverser
echo ""
echo "📦 Building plugin-example (String Reverser)..."
cargo build --target wasm32-unknown-unknown --release -p plugin-example
cp target/wasm32-unknown-unknown/release/plugin_example.wasm plugins/reverser.wasm
echo "✓ reverser.wasm → plugins/reverser.wasm"

# Plugin 2: Uppercase Converter
echo ""
echo "📦 Building plugin-uppercase..."
cargo build --target wasm32-unknown-unknown --release -p plugin-uppercase
cp target/wasm32-unknown-unknown/release/plugin_uppercase.wasm plugins/uppercase.wasm
echo "✓ uppercase.wasm → plugins/uppercase.wasm"

# Plugin 3: Character Counter
echo ""
echo "📦 Building plugin-counter..."
cargo build --target wasm32-unknown-unknown --release -p plugin-counter
cp target/wasm32-unknown-unknown/release/plugin_counter.wasm plugins/counter.wasm
echo "✓ counter.wasm → plugins/counter.wasm"

# Plugin 4: ROT13 Cipher
echo ""
echo "📦 Building plugin-rot13..."
cargo build --target wasm32-unknown-unknown --release -p plugin-rot13
cp target/wasm32-unknown-unknown/release/plugin_rot13.wasm plugins/rot13.wasm
echo "✓ rot13.wasm → plugins/rot13.wasm"

# Plugin 5: Env Reader 
echo ""
echo "📦 Building plugin-env-reader..."
cargo build --target wasm32-unknown-unknown --release -p plugin-env-reader
cp target/wasm32-unknown-unknown/release/plugin_env_reader.wasm plugins/env-reader.wasm
echo "✓ env-reader.wasm → plugins/env-reader.wasm"


echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All plugins built successfully!"
echo ""
echo "Available plugins:"
ls -lh plugins/*.wasm | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "🚀 Ready to use! Start the core server:"
echo "   cargo run -p wasm-core"
