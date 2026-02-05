# naive-pipe CLI Usage

The `pipe-run-rat` binary executes pipeline spec files using the
record-at-a-time (RAT) executor. It produces identical output to the
batched `pipe-run` for all pipelines.

## Building

```bash
# Debug build
cargo build -p naive-pipe --bin pipe-run-rat

# Release build (recommended for benchmarks)
cargo build -p naive-pipe --bin pipe-run-rat --release
```

## Running a Single Pipeline

```bash
cargo run -p naive-pipe --bin pipe-run-rat -- \
    specs/filter-sales.pipe \
    specs/input-fixed-80.data
```

Output goes to stdout. By default the command is quiet -- no extra
output is printed to stderr.

### Writing Output to a File

```bash
cargo run -p naive-pipe --bin pipe-run-rat -- \
    -o work/filter-sales.out \
    specs/filter-sales.pipe \
    specs/input-fixed-80.data
```

### Verbose Mode

Use `-v` / `--verbose` to print diagnostic info to stderr:

```bash
cargo run -p naive-pipe --bin pipe-run-rat -- \
    -v specs/filter-sales.pipe specs/input-fixed-80.data
```

Output:

```
Pipeline: specs/filter-sales.pipe
Input:    specs/input-fixed-80.data
Output:   (stdout)
Executor: record-at-a-time
SMITH   JOHN      SALES     00050000
DOE     JANE      SALES     00060000
GARCIA  CARLOS    SALES     00045000
Records:  8 in -> 3 out
```

### Reading from stdin

```bash
printf "HELLO\nWORLD\n" | cargo run -p naive-pipe --bin pipe-run-rat -- \
    specs/upper-case.pipe /dev/stdin
```

### Command-Line Reference

```
pipe-run-rat [OPTIONS] <PIPELINE> <INPUT>

Arguments:
  <PIPELINE>  Pipeline definition file (.pipe)
  <INPUT>     Input data file (80-byte fixed-width records, or /dev/stdin)

Options:
  -o, --output <OUTPUT>  Write output to file instead of stdout
  -v, --verbose          Show paths, executor, and record counts on stderr
  -h, --help             Print help
```

The batched equivalent `pipe-run` accepts the same arguments.

## Demo Scripts

Each spec file has a corresponding demo script in `naive-pipe/demos/`.
Demos are verbose: they print the input data, pipeline spec, execution
command, and output so you can see exactly what each pipeline does.

### Run a Single Demo

```bash
# From the repo root
naive-pipe/demos/demo-filter-sales.sh
```

Example output:

```
================================================================
  Demo: filter-sales (Record-at-a-Time)
================================================================

--- Input Data (specs/input-fixed-80.data) ---
SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
...

--- Pipeline (specs/filter-sales.pipe) ---
PIPE CONSOLE
| FILTER 18,10 = "SALES"
| CONSOLE
?

--- Running: pipe-run-rat specs/filter-sales.pipe specs/input-fixed-80.data ---
Pipeline: specs/filter-sales.pipe
Input:    specs/input-fixed-80.data
Output:   naive-pipe/work/sample-pipe-outputs/filter-sales.out
Executor: record-at-a-time
Records:  8 in -> 3 out

--- Output (naive-pipe/work/sample-pipe-outputs/filter-sales.out) ---
SMITH   JOHN      SALES     00050000
DOE     JANE      SALES     00060000
GARCIA  CARLOS    SALES     00045000
```

### Run All Demos

```bash
naive-pipe/demos/demo-all.sh
```

Outputs are written to `naive-pipe/work/sample-pipe-outputs/`.
Equivalent batched demos are in `demos/` at the repo root.

### Available Demos

| Demo | Pipeline | Description |
|------|----------|-------------|
| demo-change-rename | change-rename.pipe | Rename SALES to MKTG |
| demo-change-strip-prefix | change-strip-prefix.pipe | Strip leading zeros |
| demo-count-filtered | count-filtered.pipe | Count SALES records |
| demo-count-records | count-records.pipe | Count total records |
| demo-duplicate-double | duplicate-double.pipe | Duplicate each record 2x |
| demo-duplicate-triple | duplicate-triple.pipe | Duplicate each record 3x |
| demo-engineers-only | engineers-only.pipe | Filter for ENGINEER dept |
| demo-filter-sales | filter-sales.pipe | Filter for SALES dept |
| demo-literal-footer | literal-footer.pipe | Append a footer record |
| demo-literal-header-footer | literal-header-footer.pipe | Add header and footer |
| demo-locate-errors | locate-errors.pipe | Locate ERROR substring |
| demo-locate-field | locate-field.pipe | Locate SALES substring |
| demo-lower-case | lower-case.pipe | Convert to lowercase |
| demo-multi-filter-count | multi-filter-count.pipe | Locate + upper + count |
| demo-multi-locate-select | multi-locate-select.pipe | Locate + select fields |
| demo-multi-transform | multi-transform.pipe | Multiple transforms |
| demo-nlocate-exclude | nlocate-exclude.pipe | Exclude matching records |
| demo-non-marketing | non-marketing.pipe | Filter out MARKETING |
| demo-reverse-text | reverse-text.pipe | Reverse record text |
| demo-sales-report | sales-report.pipe | Filter SALES + select fields |
| demo-skip-take-window | skip-take-window.pipe | Skip 2, take 3 |
| demo-top-five | top-five.pipe | First 5 records |
| demo-upper-case | upper-case.pipe | Convert to uppercase |

## Running Tests

```bash
# All naive-pipe tests (60 tests: unit + equivalence)
cargo test -p naive-pipe

# All workspace tests (114 tests total)
cargo test

# Run a specific test
cargo test -p naive-pipe equiv_filter_sales

# Run all equivalence tests (compare batch vs RAT output)
cargo test -p naive-pipe equiv_
```

The 23 equivalence tests in `naive-pipe/src/executor.rs` automatically
verify that the RAT executor produces identical output to the batch
executor for every spec file in `specs/`.

## Benchmarking

Compare batched vs record-at-a-time performance:

```bash
# Default: 100 iterations per spec
./scripts/benchmark.sh

# Custom iteration count
./scripts/benchmark.sh 500
```

The benchmark script:
- Builds both `pipe-run` and `pipe-run-rat` in release mode
- Runs each spec file N times through both executors
- Measures wall-clock time per spec
- Verifies output equivalence (fails if any spec produces different output)
- Prints a comparison table with timing and ratio

Example output:

```
=== Batched vs RAT Benchmark ===
Iterations per spec: 100

Spec                            Batched        RAT    Ratio Match
---                                 ---        ---      --- ---
count-records                   350.1 ms    340.2 ms    0.97x ok
filter-sales                    355.8 ms    348.9 ms    0.98x ok
...
TOTAL                          9500.2 ms   9400.1 ms
                                                      0.99x

All outputs match between batched and RAT executors.
```

A ratio below 1.0 means RAT is faster; above 1.0 means batched is
faster. With the small 8-record test data, times are dominated by
process startup so both executors perform similarly.
