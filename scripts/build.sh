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

# Copy index.html to pkg (for local development)
cp index.html pkg/

# Copy to docs/ for GitHub Pages
echo ""
echo "Copying to docs/ for GitHub Pages..."
cd ..
cp wasm-ui/pkg/wasm_ui.js docs/
cp wasm-ui/pkg/wasm_ui_bg.wasm docs/
cp wasm-ui/index.html docs/

echo ""
echo "Build complete!"
echo "Run ./scripts/serve.sh to start the server"
echo "GitHub Pages files are in ./docs/"
