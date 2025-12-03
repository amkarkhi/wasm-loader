#!/bin/bash

echo "Testing plugin builds..."
echo ""

for plugin in plugin-example plugin-uppercase plugin-counter plugin-rot13 plugin-env-reader; do
    echo "=== Testing $plugin ==="
    cd "$plugin"
    
    if cargo build --target wasm32-unknown-unknown --release 2>&1 | tee /tmp/build-$plugin.log; then
        echo "✅ $plugin built successfully"
    else
        echo "❌ $plugin build failed"
        echo "Error output:"
        cat /tmp/build-$plugin.log
    fi
    
    cd ..
    echo ""
done
