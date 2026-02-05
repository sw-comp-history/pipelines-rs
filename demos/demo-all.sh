#!/bin/bash
set -euo pipefail

# Run all pipeline demos (batched executor)

cd "$(dirname "$0")/.."

echo "=== Running all Batched pipeline demos ==="
echo

cargo build --bin pipe-run --release 2>/dev/null

for pipe in specs/*.pipe; do
    name=$(basename "$pipe" .pipe)
    ./demos/demo-${name}.sh
done

echo "=== All Batched demos complete ==="
