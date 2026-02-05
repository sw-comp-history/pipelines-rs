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

---

## Next Major Feature: Multi-Stage Pipeline Specifications (NEW - CURRENT FOCUS)

This milestone enables defining and running multiple interconnected pipelines in a single specification file, following CMS Pipelines pattern of using `?` as a pipeline separator. This provides a simpler, Unix/Linux-adapted approach focused on practical data processing workflows.

See [Multi-Stage Pipeline Design](multi-stage-pipes-design.md) for detailed design and implementation notes.

### Overview

**Key Features:**
- **Chained Pipelines**: Output of one pipeline becomes input to the next
- **Independent Pipelines**: Write to intermediate files with FILE stage
- **File I/O**: FILE source (read) and FILE sink (write) stages
- **Unix-Style Stages**: SORT, SPLIT, UNIQ adapted for record-based processing

### Phases

1. **Core Multi-Pipeline Parser**
   - Parse ?-separated pipeline specifications
   - Support chained execution (output→input)
   - Support independent pipelines with FILE I/O
   
2. **File I/O Stages**
   - FILE source stage (read from files)
   - FILE sink stage (write to files)
   - Working directory context for relative paths
   
3. **Unix-Style Stages (Basic Set)**
   - SORT stage (field-based sorting)
   - SPLIT stage (delimiter-based splitting)
   - UNIQ stage (duplicate removal)
   - Enhanced FILTER operators (>, <, CONTAINS, STARTSWITH)
   
4. **WASM UI Enhancements**
   - Tutorial menu with submenus (single/multi/examples)
   - Enhanced LOAD button (upload + canned examples)
   - Input data file loading (.f80 files)
   - Example library with commented production pipelines
   
5. **Demo Scripts**
   - Multi-pipeline demo scripts
   - Sample multi-stage .pipe files

### Tasks
- [ ] Step forward one stage
- [ ] Run to completion
- [ ] Add breakpoints (pause at specific stage)

**Inspector Panel UI**:
```
Input: 8 records
Pipeline 1: Chained (3 stages)
    ↓
Pipeline 2: Independent (2 stages)
```

### Tasks

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

### Milestone 3: Multi-Stage Pipeline Specifications (NEW - CURRENT FOCUS)
See [Multi-Stage Pipeline Design](multi-stage-pipes-design.md) for detailed design and implementation notes.

### Overview
- Chained pipelines (output feeds next input)
- Independent pipelines with FILE I/O
- File I/O: FILE source (read) and FILE sink (write) stages
- Unix-style stages: SORT, SPLIT, UNIQ adapted for record-based processing

### Phases
1. Core Multi-Pipeline Parser
   - Parse ?-separated pipeline specifications
   - Support chained execution (output→input)
   - Support independent pipelines with FILE I/O
   
2. File I/O Stages
   - FILE source stage (read from files)
   - FILE sink stage (write to files)
   - Working directory context for relative paths
   
3. Unix-Style Stages (Basic Set)
   - SORT stage (field-based sorting, asc/desc)
   - SPLIT stage (delimiter-based splitting)
   - UNIQ stage (duplicate removal)
   - Enhanced FILTER operators (>, <, CONTAINS, STARTSWITH)
   
4. WASM UI Enhancements
   - Tutorial menu with submenus (single/multi/examples/canned)
   - Enhanced LOAD button (upload + canned examples)
   - Input data file loading (.f80 files)
   - Example library with commented production pipelines
   
5. Demo Scripts
   - Multi-pipeline demo scripts
   - Sample multi-stage .pipe files

### Tasks
- [x] Core Multi-Pipeline Parser (parse ?-separated pipelines, implement chaining)
- [x] File I/O stages (implement FILE source/sink)
- [x] Unix-style stages (implement SORT, SPLIT, UNIQ)
- [x] WASM UI enhancements (tutorial submenus, load examples, .f80 support)
- [x] Demo scripts (create multi-pipeline examples)
- [x] Documentation updates (user manual, examples)

### Milestone 4: Visual Pipeline Debugger - COMPLETE
Record-at-a-time visual debugger in `wasm-ui-rat/`:
- [x] Tabbed debugger view separate from main interface
- [x] Pipeline flow visualization with stage-by-stage execution
- [x] Stage controls (Run, Step, Reset)
- [x] Per-pipe-point stepping with record and flush phases
- [x] Progressive output (records appear as they reach the sink)
- [x] Watch points with toggle on/off and data panel
- [x] Breakpoints at pipe points (Run pauses, red highlight, `[BP]` label)
- [x] Load dropdown with auto-initialization (examples + file upload)
- [x] Color-based icon visibility (gold watches, red breakpoints)

See [naive-pipe/docs/debugger-manual.md](../naive-pipe/docs/debugger-manual.md) for usage.

### Future: Crate Workspace Restructure

Planned structure for full separation of concerns:

```
crates/
  shared/     - Common types (Record, Command, parse_commands, error types)
  batched/    - Batch/iterator-based pipeline executor (current src/)
  raat/       - Record-at-a-time executor (current naive-pipe/)
  saat/       - Stage-at-a-time executor (future)
```

Each crate depends on shared/. Each has its own wasm-ui/ for web interface.

### Milestone 5: Advanced Multi-Stage Features (Future)
- SPLIT stage with conditional routing (match-based subpipes)
- Conditional pipelines (IF...THEN...ELSE)
- REFORMAT stage (like CMS REFORMAT)
- LOOKUP stage (key-value matching)
- Variable substitution in pipelines

## Risk Register

- [Architecture](architecture.md) - System design
- [Product Requirements](prd.md) - Feature requirements
- [Design Document](design.md) - Technical decisions
- [Status](status.md) - Current progress
- [User Manual](user-manual.md) - Usage guide
