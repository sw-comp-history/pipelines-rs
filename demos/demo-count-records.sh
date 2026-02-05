#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/demo-lib.sh
run_demo "batched" "pipe-run" "specs/count-records.pipe" "specs/input-fixed-80.data" "work/sample-pipe-outputs/count-records.out"
