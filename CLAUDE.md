# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

pipelines-rs is a Rust library demonstrating historical mainframe-style batch processing with 80-byte fixed-width records (punch card format). It includes a WASM-based web UI built with Yew.

**Live Demo:** https://softwarewrighter.github.io/pipelines-rs/

## Build Commands

```bash
# Build library
cargo build

# Run tests
cargo test

# Run single test
cargo test test_name

# Linting (MANDATORY - zero warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Validate markdown
markdown-checker -f "**/*.md"

# Run CLI demo
cargo run

# Build library + WASM UI (ALWAYS use this script!)
./scripts/build.sh

# Serve web UI locally (port 9952)
./scripts/serve.sh
```

## WASM UI Build Process

**CRITICAL: Always use `./scripts/build.sh` to build the WASM UI.**

- Do NOT run `trunk build`, `wasm-pack`, or other WASM commands directly
- The build script compiles to `wasm-ui/dist/` then copies to `pages/` for serving
- The local server (port 9952) serves from `pages/` via symlink
- GitHub Pages also serves from `pages/`

After making WASM UI changes:
1. Run `./scripts/build.sh`
2. Refresh browser (shift-reload if needed)
3. **No server restart needed** - the server serves via symlink to `pages/`

## Architecture

### Core Components (src/)

- **Record** (`record.rs`): 80-byte fixed-width record type with field access methods
- **Pipeline** (`pipeline.rs`): Generic iterator-based pipeline with fluent API
- **Stage** (`stage.rs`): Trait + implementations (Filter, Select, Reformat, Map, Inspect)
- **Error** (`error.rs`): Error types using thiserror

### WASM UI (wasm-ui/src/)

- **app.rs**: Yew application component
- **dsl.rs**: Parser for CMS Pipelines-style DSL commands
- **components.rs**: UI panels (Input, Pipeline, Output)

### Key Patterns

1. **Pull-based data flow** - Iterator-style lazy evaluation
2. **Fluent builder API** - Chainable pipeline operations
3. **Stage trait abstraction** - Extensible transform/filter operations
4. **80-byte records** - All data as fixed-width ASCII (EBCDIC compatibility simulation)

## Development Process

This project follows strict TDD (Red/Green/Refactor) with mandatory pre-commit gates.

### Pre-Commit Checklist (ALL MUST PASS)

1. `cargo test` - All tests pass
2. `cargo clippy --all-targets --all-features -- -D warnings` - Zero warnings
3. `cargo fmt --all` - Code formatted
4. `markdown-checker -f "**/*.md"` - Markdown validated

### Code Quality Rules

- **Never use `#[allow(...)]`** to suppress clippy warnings - fix the actual issue
- **Files under 500 lines** (prefer 200-300)
- **Functions under 50 lines** (prefer 10-30)
- **Max 3 TODO comments per file** - never commit FIXMEs
- **Inline format args**: `format!("{name}")` not `format!("{}", name)`
- **Doc comments**: `//!` for modules, `///` for items

### Rust/WASM Specific

- Keep JavaScript to absolute minimum - all business logic in Rust
- Use `wasm-bindgen` for JS interop, `web-sys` for DOM
- Write tests in Rust using `wasm-bindgen-test`, not JavaScript

## Workspace Structure

```
Cargo.toml        # Workspace root with 2 members: . and wasm-ui
src/              # Main library
wasm-ui/          # WASM web UI
docs/             # Documentation (architecture, design, process, etc.)
pages/            # GitHub Pages (built WASM UI)
scripts/          # Build and serve scripts
specs/            # Example pipeline specifications
```

## Key Documentation

- `docs/ai_agent_instructions.md` - Full AI agent guidelines
- `docs/process.md` - Development workflow
- `docs/architecture.md` - System design
- `docs/user-manual.md` - Usage examples
