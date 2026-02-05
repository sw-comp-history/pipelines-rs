# Shared helper for verbose demo scripts.
# Source this file; do not execute directly.
#
# Usage (batched):
#   source "$(dirname "$0")/../scripts/demo-lib.sh"
#   run_demo "batched" "pipe-run" "specs/NAME.pipe" "specs/input-fixed-80.data" "work/sample-pipe-outputs/NAME.out"
#
# Usage (RAT):
#   source "$(dirname "$0")/../../scripts/demo-lib.sh"
#   run_demo "rat" "pipe-run-rat" "specs/NAME.pipe" "specs/input-fixed-80.data" "naive-pipe/work/sample-pipe-outputs/NAME.out"

run_demo() {
    local mode="$1"       # "batched" or "rat"
    local bin="$2"        # binary name
    local pipe_file="$3"  # path to .pipe file (relative to repo root)
    local input_file="$4" # path to .data file (relative to repo root)
    local output_file="$5" # path to output file (relative to repo root)
    local name
    name=$(basename "$pipe_file" .pipe)

    local label
    if [ "$mode" = "rat" ]; then
        label="Record-at-a-Time"
    else
        label="Batched"
    fi

    echo "================================================================"
    echo "  Demo: $name ($label)"
    echo "================================================================"
    echo
    echo "--- Input Data ($input_file) ---"
    cat "$input_file"
    echo
    echo "--- Pipeline ($pipe_file) ---"
    cat "$pipe_file"
    echo
    echo "--- Running: $bin $pipe_file $input_file ---"

    mkdir -p "$(dirname "$output_file")"

    if [ "$mode" = "rat" ]; then
        cargo run -p naive-pipe --bin "$bin" --release -- \
            "$pipe_file" "$input_file" -o "$output_file" 2>&1
    else
        cargo run --bin "$bin" --release -- \
            "$pipe_file" "$input_file" -o "$output_file" 2>&1
    fi

    echo
    echo "--- Output ($output_file) ---"
    cat "$output_file"
    echo
    echo
}
