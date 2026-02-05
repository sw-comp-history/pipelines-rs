#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/demo-lib.sh
run_demo "batched" "pipe-run" "specs/reverse-text.pipe" "specs/input-fixed-80.data" "work/sample-pipe-outputs/reverse-text.out"
