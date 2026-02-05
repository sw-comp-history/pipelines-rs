#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../.."
source scripts/demo-lib.sh
run_demo "rat" "pipe-run-rat" "specs/literal-header-footer.pipe" "specs/input-fixed-80.data" "naive-pipe/work/sample-pipe-outputs/literal-header-footer.out"
