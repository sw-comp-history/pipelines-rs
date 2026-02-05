#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/demo-lib.sh
run_demo "batched" "pipe-run" "specs/change-strip-prefix.pipe" "specs/input-fixed-80.data" "work/sample-pipe-outputs/change-strip-prefix.out"
