#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../.."
source scripts/demo-lib.sh
run_demo "rat" "pipe-run-rat" "specs/multi-transform.pipe" "specs/input-fixed-80.data" "naive-pipe/work/sample-pipe-outputs/multi-transform.out"
