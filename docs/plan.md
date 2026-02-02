# Development Plan

## Overview

This document outlines the implementation plan for pipelines-rs, broken into milestones with specific deliverables.

## Current Phase

**Phase**: CLI and Web UI Complete
**Status**: CLI, demos, and tutorial system functional

## Completed Work

### Milestone 1: Core Pipeline (Foundation) - COMPLETE

- [x] Project structure setup
  - [x] Cargo workspace (library + wasm-ui)
  - [x] Module organization
  - [x] Rust 2024 edition
- [x] Core types
  - [x] `Record` type (80-byte fixed-width)
  - [x] `Pipeline` struct with fluent API
  - [x] Iterator-based data flow
- [x] Basic stages
  - [x] `Filter` stage (= and != operators)
  - [x] `Select` stage (field extraction/repositioning)
  - [x] `Take` stage (limit records)
  - [x] `Skip` stage (skip records)
  - [x] `Map` stage (transform records)
  - [x] `Inspect` stage (debugging)
- [x] Web UI
  - [x] Yew/WASM application
  - [x] Three-panel layout (Input, Pipeline, Output)
  - [x] DSL parser for pipeline text
  - [x] Load/Save pipeline files
  - [x] 80-column display with ruler
  - [x] Interactive tutorial system with auto-run mode
  - [x] Clear button for output panel
- [x] Tests
  - [x] 59 tests (33 unit + 26 doc tests)
  - [x] Zero clippy warnings
- [x] Documentation
  - [x] API documentation
  - [x] User manual with examples

### Milestone 2: CLI Interface - COMPLETE

- [x] DSL module in main library
  - [x] `execute_pipeline()` function exposed
  - [x] Full DSL parser (FILTER, SELECT, TAKE, SKIP, LOCATE, COUNT, etc.)
- [x] `pipe-run` CLI binary
  - [x] Run .pipe files against input data
  - [x] Output to file or stdout
  - [x] Record count statistics
- [x] Demo scripts
  - [x] Individual `demo-<name>.sh` for each .pipe file
  - [x] `demo-all.sh` to run all demos
  - [x] Outputs to `work/sample-pipe-outputs/`
- [x] Sample data
  - [x] `specs/input-fixed-80.data` with 8 employee records
  - [x] 23 sample .pipe files in `specs/`

---

## Next Major Feature (Deferred)

### Milestone 3: Multi-Stream Processing

This milestone enables routing records to different subpipes based on selectors, with each subpipe potentially writing to different output files. This is the core feature that enables real mainframe-style batch processing patterns like master file updates.

#### Overview

Records from an input source can be routed to different processing paths based on field values:

```
PIPE CONSOLE
| a: SPLIT 18,10                    # Split on department field
|    = "SALES": sales_pipe          # Route SALES to subpipe
|    = "ENGINEER": eng_pipe         # Route ENGINEER to subpipe
|    OTHERWISE: other_pipe          # Route unmatched records
?

# Subpipe definitions
sales_pipe:
| SELECT 0,8,0; 28,8,8
| FILE sales-report.out
?

eng_pipe:
| UPPER
| FILE engineers.out
?

other_pipe:
| FILE unmatched.out
?
```

#### Prerequisites: Debugging Controls

To implement and test multi-stream processing, we first need pipeline debugging controls:

**Stage Inspection**:
- [ ] Add stage-by-stage execution mode (step through pipeline)
- [ ] Show intermediate results between stages
- [ ] Track and display record counts at each stage
- [ ] Highlight current stage in pipeline editor

**Pipeline Controls**:
- [ ] Reset pipeline to initial state
- [ ] Step forward one stage
- [ ] Run to completion
- [ ] Add breakpoints (pause at specific stage)

**Inspector Panel UI**:
```
[Input: 8 records]
    ↓
[a: SPLIT] ─┬─> [sales_pipe: 3 records]
            ├─> [eng_pipe: 3 records]
            └─> [other_pipe: 2 records]
```

#### Labels for Stages

Add label syntax for referencing stages:

```
PIPE CONSOLE
| a: FILTER 18,10 = "SALES"    # Label 'a' for this stage
| b: SELECT 0,8,0; 28,8,8      # Label 'b' for this stage
| CONSOLE
?
```

**Tasks**:
- [ ] Add label syntax (`label:` prefix) to DSL parser
- [ ] Store labels in parsed Command struct
- [ ] Display labels in UI debug panel
- [ ] Use labels for SPLIT routing targets

#### SPLIT Stage (Conditional Routing)

Route records to different subpipes based on field matching:

```
PIPE CONSOLE
| SPLIT 18,10                      # Field to match
|   = "SALES": sales_handler       # Exact match
|   = "ENGINEER": eng_handler      # Exact match
|   CONTAINS "MARK": marketing     # Partial match
|   OTHERWISE: default_handler     # Catch-all
?
```

**Tasks**:
- [ ] Design SPLIT syntax (field spec + routing rules)
- [ ] Implement SPLIT stage in DSL parser
- [ ] Add subpipe definition syntax
- [ ] Implement multi-output pipeline executor
- [ ] Route records to appropriate subpipes
- [ ] Update UI to show split outputs in separate panels

#### File I/O for Subpipes

Each subpipe can read from and write to files:

```
PIPE FILE master.dat              # Read from file
| SPLIT 0,1                       # Route by record type
|   = "U": update_pipe            # Updates
|   = "D": delete_pipe            # Deletes
|   = "A": add_pipe               # Additions
?

update_pipe:
| LOOKUP master-index.dat 0,8     # Find matching master
| MERGE                           # Merge update into master
| FILE master-new.dat             # Write updated master
?
```

**Tasks**:
- [ ] Add FILE stage for reading (line-by-line)
- [ ] Add FILE stage for writing (append or overwrite)
- [ ] Handle file errors gracefully
- [ ] Support relative and absolute paths
- [ ] Support multiple output files from one pipeline

#### Advanced Demo: Master File Update

Classic mainframe pattern - apply transactions to a master file:

```
# Transaction file has: A=Add, U=Update, D=Delete records
# Master file has existing employee records

PIPE FILE transactions.dat
| SPLIT 0,1
|   = "A": add_new
|   = "U": update_existing
|   = "D": mark_deleted
?

add_new:
| SKIP 1                          # Skip transaction code
| FILE master-adds.dat
?

update_existing:
| LOOKUP master.dat 1,8           # Match on employee ID
| SELECT ...                      # Merge fields
| FILE master-updates.dat
?
```

**Tasks**:
- [ ] Implement LOOKUP stage (key-based record matching)
- [ ] Create transaction file test data
- [ ] Create master file update demo
- [ ] Document master file update pattern

---

## Backlog (Lower Priority)

### SORT Stage

Sort records by field:

```
PIPE CONSOLE
| SORT 28,8 DESC
| CONSOLE
?
```

**Tasks**:
- [ ] Implement SORT stage
- [ ] Support ASC/DESC order
- [ ] Support multiple sort keys
- [ ] Consider external sort for large datasets

### MERGE Stage

Combine multiple sorted streams:

```
PIPE (
  FILE sales.dat | SORT 0,8
  FILE marketing.dat | SORT 0,8
)
| MERGE 0,8
| CONSOLE
?
```

**Tasks**:
- [ ] Design MERGE syntax for multiple inputs
- [ ] Implement sorted merge algorithm
- [ ] Consider memory-efficient streaming merge

### Additional FILTER Operators

```
FILTER 28,8 > "00050000"     # Greater than
FILTER 28,8 < "00070000"     # Less than
FILTER 0,8 CONTAINS "SMI"    # Contains substring
FILTER 0,8 STARTSWITH "S"    # Starts with
```

### REFORMAT Stage

Create new records with literal text and field references:

```
PIPE CONSOLE
| REFORMAT "Name: " 0,8 " Salary: $" 28,8
| CONSOLE
?
```

### Keyboard Shortcuts

- [ ] Ctrl+Enter to run pipeline
- [ ] Ctrl+S to save pipeline
- [ ] F5 to run, F6 to step

### CI/CD Pipeline

- [ ] GitHub Actions workflow
- [ ] Automated testing on PR
- [ ] Deploy to GitHub Pages on merge

---

## Long Term (Future Consideration)

#### Parallel Execution

- [ ] Design parallel execution model
- [ ] Implement parallel filter/map stages

#### External Data Sources

- [ ] Database source (SQL query)
- [ ] HTTP/REST source

#### Pipeline Composition

```
INCLUDE common-filters.pipe
PIPE CONSOLE
| CALL validate_record
| CONSOLE
?
```

#### Visual Pipeline Editor

- [ ] Node-based visual editor
- [ ] Generate DSL from visual layout

---

## Task Tracking

### Current Sprint

| Task | Status | Notes |
|------|--------|-------|
| CLI binary | Complete | `pipe-run` command |
| Demo scripts | Complete | 24 scripts in `demos/` |
| Tutorial system | Complete | Auto-run mode with countdown |

### Next Sprint (Deferred)

| Task | Status | Notes |
|------|--------|-------|
| Debug controls | Not Started | Prerequisite for SPLIT |
| Stage inspector | Not Started | UI for debugging |
| Labels | Not Started | Required for SPLIT targets |
| SPLIT stage | Not Started | Core routing feature |
| File I/O | Not Started | Required for subpipe outputs |

### Backlog (Prioritized)

1. **Debugging controls** (reset, step, inspect)
2. **Labels for stages** (reference targets)
3. **SPLIT stage** (conditional routing to subpipes)
4. **File I/O stages** (read/write files)
5. SORT stage
6. MERGE stage (combine streams)
7. LOOKUP stage (key-based matching)
8. Additional FILTER operators
9. REFORMAT stage
10. Keyboard shortcuts
11. CI/CD pipeline

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Complex SPLIT/MERGE syntax | Medium | High | Prototype early, get feedback |
| Performance with large files | Low | Medium | Streaming design, external sort |
| UI complexity with debug features | Medium | Medium | Progressive disclosure |

## Quality Standards

All code must meet these criteria before merge:
- All tests pass (`cargo test`)
- Zero clippy warnings (`cargo clippy -- -D warnings`)
- Formatted (`cargo fmt`)
- Documented (public items)
- User manual updated for new features

## Related Documentation

- [Architecture](architecture.md) - System design
- [Product Requirements](prd.md) - Feature requirements
- [Design Document](design.md) - Technical decisions
- [Status](status.md) - Current progress
- [User Manual](user-manual.md) - Usage guide
