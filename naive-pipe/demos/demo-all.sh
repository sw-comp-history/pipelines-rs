#!/bin/bash
set -euo pipefail

# Run all pipeline demos (record-at-a-time executor)

cd "$(dirname "$0")/../.."

echo "=== Running all RAT pipeline demos ==="
echo

cargo build -p naive-pipe --bin pipe-run-rat --release 2>/dev/null

for pipe in specs/*.pipe; do
    name=$(basename "$pipe" .pipe)
    ./naive-pipe/demos/demo-${name}.sh
done

echo "=== All RAT demos complete ==="
