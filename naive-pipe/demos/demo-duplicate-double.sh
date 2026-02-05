#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../.."
source scripts/demo-lib.sh
run_demo "rat" "pipe-run-rat" "specs/duplicate-double.pipe" "specs/input-fixed-80.data" "naive-pipe/work/sample-pipe-outputs/duplicate-double.out"
