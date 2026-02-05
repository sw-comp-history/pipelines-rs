#!/bin/bash
set -euo pipefail

# Serve the RAT WASM UI on port 9953

cd "$(dirname "$0")/.."

echo "Serving RAT UI at http://localhost:9953/"
echo "Press Ctrl+C to stop"
basic-http-server -a 0.0.0.0:9953 wasm-ui/dist
