#!/bin/bash
set -euo pipefail

# Build pipelines-rs library and WASM UI

cd "$(dirname "$0")/.."

echo "Building Rust library..."
cargo build

echo ""
echo "Building WASM UI..."
cd wasm-ui
wasm-pack build --target web --out-dir pkg

# Copy index.html to pkg
cp index.html pkg/

echo ""
echo "Build complete!"
echo "Run ./scripts/serve.sh to start the server"
