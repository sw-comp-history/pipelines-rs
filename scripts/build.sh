#!/bin/bash
set -euo pipefail

# Build pipelines-rs library and both WASM UIs
# IMPORTANT: Always use this script to build - do not run trunk/wasm commands directly

cd "$(dirname "$0")/.."

echo "Building Rust library..."
cargo build

echo ""
echo "Building Batched WASM UI with trunk..."
cd wasm-ui
trunk build --public-url /pipelines-rs/batched/
cd ..

echo ""
echo "Building RAT WASM UI with trunk..."
cd naive-pipe/wasm-ui
trunk build --public-url /pipelines-rs/rat/
cd ../..

# Prepare pages/ directory
echo ""
echo "Copying to pages/ for GitHub Pages..."
# Remove old build artifacts but keep .nojekyll
rm -rf pages/batched pages/rat pages/*.js pages/*.wasm pages/*.ico
mkdir -p pages/batched pages/rat

# Copy batched UI
cp -r wasm-ui/dist/* pages/batched/

# Copy RAT UI
cp -r naive-pipe/wasm-ui/dist/* pages/rat/

# Copy landing page
cp pages-src/index.html pages/index.html

echo ""
echo "Build complete!"
echo "Run ./scripts/serve.sh to start the server on port 9952"
echo "GitHub Pages files are in ./pages/"
