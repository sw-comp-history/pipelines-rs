# pipelines-rs

A Rust library demonstrating historical mainframe-style batch processing with 80-byte fixed-width records.

## Overview

This project shows how data pipelines worked on mainframe systems, where data was processed as fixed-width 80-byte records (matching the width of punch cards). Operations like FILTER, SELECT, REFORMAT, and MERGE were used to transform datasets in batch processing jobs.

**Live Demo:** [https://softwarewrighter.github.io/pipelines-rs/](https://softwarewrighter.github.io/pipelines-rs/)

![pipelines-rs Web UI](images/screenshot.png?ts=1770065141000)

## Status

**Version**: 0.1.0 (pre-release)
**Edition**: Rust 2024

Core pipeline functionality is complete. See [Status](documentation/status.md) for details.

## Features

- **80-byte fixed-width records** - Historical punch card format
- **Type-safe pipeline composition** - Fluent builder API
- **Field-based operations** - Extract and manipulate columns by position
- **Mainframe-style stages** - Filter, Select, Reformat operations

## Quick Start

```rust
use pipelines_rs::{Pipeline, Record};

fn main() {
    // Record layout: Last(8) First(10) Dept(10) Salary(8)
    let records = vec![
        Record::from_str("SMITH   JOHN      SALES     00050000"),
        Record::from_str("JONES   MARY      ENGINEER  00075000"),
        Record::from_str("DOE     JANE      SALES     00060000"),
    ];

    // Filter SALES department and select name + salary
    let result: Vec<Record> = Pipeline::new(records.into_iter())
        .filter(|r| r.field_eq(18, 10, "SALES"))
        .select(vec![
            (0, 8, 0),   // Last name -> position 0
            (28, 8, 8),  // Salary -> position 8
        ])
        .collect();

    for record in &result {
        println!("{} ${}",
            record.field(0, 8).trim(),
            record.field(8, 8).trim()
        );
    }
    // Output:
    // SMITH $00050000
    // DOE $00060000
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [User Manual](documentation/user-manual.md) | Usage guide with examples |
| [Architecture](documentation/architecture.md) | System design and component overview |
| [PRD](documentation/prd.md) | Product requirements and goals |
| [Design](documentation/design.md) | Technical design decisions |
| [Plan](documentation/plan.md) | Development roadmap and milestones |
| [Status](documentation/status.md) | Current project status |
| [Process](documentation/process.md) | Development workflow and standards |
| [Tools](documentation/tools.md) | Development tools reference |
| [AI Agent Instructions](documentation/ai_agent_instructions.md) | Guidelines for AI coding agents |

## Web UI

A browser-based demo is available using Yew/WASM:

```bash
# Build library and WASM UI (ALWAYS use this script!)
./scripts/build.sh

# Serve locally (port 9952)
./scripts/serve.sh
# Then open http://localhost:9952
```

**Important:** Always use `./scripts/build.sh` to build the WASM UI. Do not run `trunk` or `wasm-pack` commands directly - the build script handles compilation and deploys to the correct folder for serving. After rebuilding, just refresh your browser - no server restart needed.

The UI provides:
- **Input Panel** - Enter/edit 80-byte records with column ruler
- **Pipeline Panel** - Write DSL commands (FILTER, SELECT, TAKE, SKIP)
- **Output Panel** - View processed results with record counts

## Building

```bash
# Build library
cargo build

# Run tests
cargo test

# Run with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Run CLI demo
cargo run
```

## Development

This project follows Test-Driven Development (TDD) with strict quality gates. See [Process](documentation/process.md) for the complete development workflow.

### Pre-commit Checklist

- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Code formatted
- [ ] Documentation updated

## License

MIT License - Copyright (c) 2026 Michael A Wright

See [LICENSE](LICENSE) for full text.

## Contributing

See [Plan](documentation/plan.md) for current priorities.
