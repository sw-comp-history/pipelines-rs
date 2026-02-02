#!/bin/bash
set -euo pipefail

# Build pipelines-rs library and WASM UI
# IMPORTANT: Always use this script to build - do not run trunk/wasm commands directly

cd "$(dirname "$0")/.."

echo "Building Rust library..."
cargo build

echo ""
echo "Building WASM UI with trunk..."
cd wasm-ui
trunk build --public-url /pipelines-rs/

# Copy dist/ to pages/ for GitHub Pages and local serving
echo ""
echo "Copying to pages/ for GitHub Pages..."
cd ..
rm -rf pages/*
cp -r wasm-ui/dist/* pages/

echo ""
echo "Build complete!"
echo "Run ./scripts/serve.sh to start the server on port 9952"
echo "GitHub Pages files are in ./pages/"
