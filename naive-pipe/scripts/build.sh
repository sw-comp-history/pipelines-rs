#!/bin/bash
set -euo pipefail

# Build naive-pipe library and RAT WASM UI
# Local-only build (no GitHub Pages deployment)

cd "$(dirname "$0")/.."

echo "Building naive-pipe library..."
cargo build

echo ""
echo "Building RAT WASM UI with trunk..."
cd wasm-ui
trunk build

echo ""
echo "Build complete!"
echo "Run ./scripts/serve.sh to start the server on port 9953"
