# Demos

This project includes two web UIs and 24 CLI demo scripts demonstrating mainframe-style 80-byte fixed-width record pipeline processing.

## Web UIs

### Batched Pipeline

Stage-at-a-time execution: all records flow through each stage before moving to the next. Includes 15 interactive tutorials covering all pipeline commands.

**Live:** [https://sw-comp-history.github.io/pipelines-rs/batched/](https://sw-comp-history.github.io/pipelines-rs/batched/)

### Record-at-a-Time Debugger

Record-at-a-time execution: each record flows through the entire pipeline before the next is read. Visual debugger with step, watch, and breakpoints.

**Live:** [https://sw-comp-history.github.io/pipelines-rs/rat/](https://sw-comp-history.github.io/pipelines-rs/rat/)

## Interactive Tutorials

Both web UIs include 15 built-in tutorials with guided walkthroughs:

| # | Tutorial | Command |
|---|----------|---------|
| 1 | Hello World | `LITERAL` |
| 2 | PIPE/CONSOLE | `PIPE CONSOLE \| CONSOLE` |
| 3 | FILTER | `FILTER pos,len = "value"` |
| 4 | SELECT | `SELECT src,len,dest; ...` |
| 5 | TAKE | `TAKE n` |
| 6 | SKIP | `SKIP n` |
| 7 | LOCATE | `LOCATE /pattern/` |
| 8 | NLOCATE | `NLOCATE /pattern/` |
| 9 | COUNT | `COUNT` |
| 10 | CHANGE | `CHANGE /old/new/` |
| 11 | LITERAL | `LITERAL text` |
| 12 | UPPER | `UPPER` |
| 13 | LOWER | `LOWER` |
| 14 | REVERSE | `REVERSE` |
| 15 | DUPLICATE | `DUPLICATE n` |

## CLI Demo Scripts

The `demos/` directory contains 24 shell scripts that run pipeline specs against a shared 80-byte fixed-width employee dataset (`specs/input-fixed-80.data`).

| Script | Description |
|--------|-------------|
| `demo-all.sh` | Run all demos in sequence |
| `change-rename` | Rename "SALES" to "MKTG" with CHANGE |
| `change-strip-prefix` | Strip "ERROR: " prefix |
| `count-filtered` | Count records containing "SALES" |
| `count-records` | Count total input records |
| `duplicate-double` | Double each record |
| `duplicate-triple` | Triple each record |
| `engineers-only` | Filter for ENGINEERING department |
| `filter-sales` | Filter for SALES department |
| `literal-footer` | Append footer record |
| `literal-header-footer` | Append header and footer |
| `locate-errors` | Find records containing "ERROR" |
| `locate-field` | Find "SALES" in specific field |
| `lower-case` | Convert all text to lowercase |
| `multi-filter-count` | LOCATE + UPPER + COUNT pipeline |
| `multi-locate-select` | LOCATE + SELECT pipeline |
| `multi-transform` | UPPER + CHANGE pipeline |
| `nlocate-exclude` | Exclude records with "DEBUG" |
| `non-marketing` | Show non-MARKETING employees |
| `reverse-text` | Reverse characters in each record |
| `sales-report` | Filter SALES + extract name/salary |
| `skip-take-window` | SKIP 2 + TAKE 3 (pagination) |
| `top-five` | SELECT fields + TAKE 5 |
| `upper-case` | Convert all text to uppercase |
