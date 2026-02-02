# Project Status

## Current Status

**Project**: pipelines-rs
**Version**: 0.1.0 (pre-release)
**Last Updated**: 2026-02-02

### Overall Progress

| Milestone | Status | Progress |
|-----------|--------|----------|
| M1: Core Pipeline | Complete | 100% |
| M2: File I/O | Complete | 100% |
| M3: CLI Interface | Complete | 100% |
| M4: Advanced Features | Not Started | 0% |
| M5: Polish and Release | Not Started | 0% |

### Current Phase

**Phase**: CLI and Web UI Complete
**Focus**: Demo scripts and pipeline execution

## Recent Activity

### 2026-02-02

- [x] Implemented interactive tutorial system with auto-run mode
- [x] Added Clear button and output clearing between tutorials
- [x] Created DSL module in main library (moved from wasm-ui)
- [x] Implemented `pipe-run` CLI binary for running .pipe files
- [x] Created 23 demo scripts in `demos/` directory
- [x] Created `demo-all.sh` to run all demos
- [x] Sample outputs go to `work/sample-pipe-outputs/`
- [x] Added input data file `specs/input-fixed-80.data`
- [x] Added serve.sh redirect for localhost:9952 root

### 2026-02-01

- [x] Created project skeleton
- [x] Set up Cargo.toml with Rust 2024 edition
- [x] Created documentation structure
- [x] Implemented `Record` type (80-byte fixed-width)
- [x] Implemented `Stage` trait
- [x] Implemented `Pipeline` struct with builder pattern
- [x] Implemented stages: Filter, Select, Reformat, Map, Inspect
- [x] Added 33 unit tests + 26 doc tests
- [x] Created mainframe-style demo application
- [x] Zero clippy warnings, all tests passing
- [x] Implemented Yew/WASM web UI
- [x] Created DSL parser (FILTER, SELECT, TAKE, SKIP)
- [x] Built three-panel UI (Input, Pipeline, Output)
- [x] Added build/serve scripts for port 9952
- [x] Implemented CMS Pipelines-style DSL syntax (PIPE + | continuations)
- [x] Added optional `?` end-of-pipe terminator

## What's Working

- **Record type**: 80-byte fixed-width records with field access
- **Pipeline**: Fluent API for chaining operations
- **Stages**: Filter, Select, Reformat, Map, Inspect
- **Operations**: filter, omit, map, select, reformat, take, skip, chain, fold, any, all
- **CLI**: `pipe-run` binary for running .pipe files
- **Demo scripts**: 24 demo scripts in `demos/` directory
- **Web UI**: Yew/WASM interface at http://localhost:9952
- **Tutorial system**: Interactive tutorials with auto-run mode
- **DSL Parser**: Text-based pipeline commands (FILTER, SELECT, TAKE, SKIP, LOCATE, NLOCATE, COUNT, CHANGE, LITERAL, UPPER, LOWER, REVERSE, DUPLICATE)
- **Live demo**: https://softwarewrighter.github.io/pipelines-rs/

## What's Not Working

- Full merge/split with sorting (planned for M4)
- Labels for stages
- SORT stage

## Blockers

None currently.

## Next Steps

### Immediate (This Week)

1. [ ] Add labels for stages
2. [ ] Add SORT stage
3. [ ] Set up CI/CD with GitHub Actions

### Short Term (This Month)

1. [ ] Add proper Merge stage (sorted merge)
2. [ ] Add Split stage (multi-output)
3. [ ] Debug inspector panel

### Medium Term (Next Quarter)

1. [ ] Additional FILTER operators (>, <, CONTAINS)
2. [ ] Keyboard shortcuts
3. [ ] Initial user feedback

## Metrics

### Code Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Lines of Code | ~1200 | - |
| Test Coverage | High | >80% |
| Clippy Warnings | 0 | 0 |
| Doc Coverage | 100% | 100% |

### Quality Metrics

| Metric | Status |
|--------|--------|
| Tests Passing | 59/59 (33 unit + 26 doc) |
| Linting Clean | Yes (zero warnings) |
| Formatted | Yes |
| Documentation | Complete for current features |

## Known Issues

None currently.

## Technical Debt

None currently.

## Notes

### Decisions Made

1. Using Rust 2024 edition for latest features
2. Pull-based (iterator) data flow
3. Sync-first design with async compatibility planned
4. 80-byte fixed-width records (mainframe punch card format)
5. ASCII-only (simulating EBCDIC->ASCII conversion)

### Open Questions

1. Configuration file format (TOML vs YAML)
2. Parallel execution model
3. Plugin architecture design

## Related Documentation

- [Development Plan](plan.md) - Detailed roadmap
- [Architecture](architecture.md) - System design
- [Design Document](design.md) - Technical decisions
- [Product Requirements](prd.md) - Feature requirements
