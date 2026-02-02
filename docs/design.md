# Design Document

## Overview

This document captures key design decisions for pipelines-rs, including rationale and alternatives considered.

## Core Abstractions

### Pipeline Stage Trait

**Decision**: Use a generic trait for pipeline stages

```rust
pub trait Stage<I, O> {
    type Error;
    fn process(&mut self, input: I) -> Result<O, Self::Error>;
}
```

**Rationale**:
- Type parameters ensure compile-time safety
- Associated error type allows stage-specific errors
- Mutable self allows stateful stages

**Alternatives Considered**:
- `Fn` closures: Simpler but less flexible for stateful operations
- Async trait: Adds complexity, deferred to future version

### Pipeline Composition

**Decision**: Use builder pattern for pipeline construction

```rust
let pipeline = Pipeline::new()
    .source(file_reader)
    .map(|x| x.to_uppercase())
    .filter(|x| !x.is_empty())
    .sink(file_writer);
```

**Rationale**:
- Fluent API is intuitive
- Each method returns owned self for chaining
- Type inference works well with closures

**Alternatives Considered**:
- Macro-based DSL: Higher learning curve
- Configuration files only: Less flexible

### Error Handling Strategy

**Decision**: Use custom error enum with thiserror

```rust
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("source error: {0}")]
    Source(#[from] SourceError),
    #[error("transform error: {0}")]
    Transform(#[from] TransformError),
    #[error("sink error: {0}")]
    Sink(#[from] SinkError),
}
```

**Rationale**:
- Clear error categorization
- Automatic From implementations
- Good error messages

**Alternatives Considered**:
- `anyhow`: Good for applications, less suitable for library
- `Box<dyn Error>`: Loses type information

## Data Flow Design

### Pull-Based vs Push-Based

**Decision**: Pull-based (iterator-style) for synchronous pipelines

**Rationale**:
- Natural fit for Rust iterators
- Easy to reason about
- Good backpressure by default

**Alternatives Considered**:
- Push-based: Better for async, but adds complexity
- Hybrid: Deferred to future version

### Streaming vs Batch

**Decision**: Support both, default to streaming

**Rationale**:
- Streaming is memory-efficient
- Batch needed for some operations (sort, group)
- User chooses based on use case

## Memory Management

### Ownership Model

**Decision**: Stages own their data, pass ownership through pipeline

```rust
// Data is moved through the pipeline
fn process(&mut self, input: I) -> Result<O, Self::Error>;
```

**Rationale**:
- Clear ownership semantics
- No lifetime complexity for users
- Compiler ensures correctness

### Buffer Management

**Decision**: Configurable internal buffers

**Rationale**:
- Balance memory usage vs throughput
- User can tune based on workload
- Reasonable defaults for common cases

## Concurrency Design

### Thread Safety

**Decision**: Pipeline is `!Sync`, stages may be `Send`

**Rationale**:
- Single pipeline runs in one thread
- Stages can be moved between threads
- Parallel execution is explicit

### Future Async Support

**Decision**: Design for sync-first, async-compatible

**Rationale**:
- Simpler initial implementation
- Can add async later without breaking changes
- Most use cases are sync

## API Design Principles

### 1. Progressive Disclosure

Simple cases should be simple:

```rust
// Simple case
Pipeline::new()
    .map(|x| x + 1)
    .collect()

// Advanced case
Pipeline::builder()
    .with_buffer_size(1024)
    .with_error_handler(|e| log::warn!("{e}"))
    .source(complex_source)
    .map_with_context(|ctx, x| transform(ctx, x))
    .sink(complex_sink)
    .build()
```

### 2. Type Inference

Leverage Rust's type inference:

```rust
// Types are inferred
let result: Vec<String> = Pipeline::new()
    .source(vec![1, 2, 3])
    .map(|x| x.to_string())
    .collect();
```

### 3. Fail Fast

Validate early, fail with clear errors:

```rust
// This should fail at build time, not runtime
Pipeline::new()
    .map(|x: i32| x.to_string())
    .map(|x: i32| x + 1)  // Type error: expected String
    .collect()
```

## Testing Strategy

### Unit Tests

- Test each stage in isolation
- Test error conditions
- Test edge cases (empty input, large input)

### Integration Tests

- Test complete pipelines
- Test file I/O
- Test CLI commands

### Property-Based Tests

- Test with random inputs
- Verify invariants hold

## Performance Considerations

### Hot Path Optimization

- Minimize allocations in transform stages
- Use `&str` where possible
- Consider SIMD for bulk operations (future)

### Benchmarking

- Benchmark each stage type
- Benchmark complete pipelines
- Compare against baseline implementations

## Open Questions

1. **Should pipelines be clonable?**
   - Pro: Easy to create variants
   - Con: Complicates stateful stages

2. **Should we support branching pipelines?**
   - Pro: More flexible workflows
   - Con: Significantly more complex

3. **Configuration file format?**
   - TOML: Simple, Rust ecosystem standard
   - YAML: More expressive, widely used

## DSL Syntax Design

**Decision**: CMS Pipelines-style syntax for web UI

```text
PIPE FILTER 18,10 = "SALES"
   | SELECT 0,8,0; 28,8,8
   | TAKE 10?
```

**Syntax Rules**:
- `PIPE` keyword with first stage on same line
- `|` (pipe) connects stages in sequence
- `?` marks end of pipeline (separator for chaining multiple pipelines)
- Continuation lines start with `|` for readability
- `#` starts a comment line
- Whitespace is flexible (leading/trailing trimmed)

**Supported Stages**:
- `FILTER pos,len = "value"` - Keep matching records
- `FILTER pos,len != "value"` - Omit matching records
- `SELECT src,len,dest; ...` - Select and reposition fields
- `TAKE n` - Keep first n records
- `SKIP n` - Skip first n records

**Rationale**:
- Familiar syntax for mainframe users
- Visual representation matches data flow concept
- Readable and self-documenting
- Compatible with CMS Pipelines heritage

**Alternatives Considered**:
- JSON configuration: Verbose, not user-friendly
- GUI-only: Less flexible, harder to share/reproduce
- Unix pipe syntax: Less familiar for target users

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-01 | Use Rust 2024 edition | Latest features, future-proof |
| 2026-02-01 | Pull-based data flow | Natural iterator fit |
| 2026-02-01 | Sync-first design | Simpler initial implementation |
| 2026-02-01 | CMS Pipelines-style DSL | Familiar syntax for mainframe users |

## Related Documentation

- [Architecture](architecture.md) - System overview
- [Product Requirements](prd.md) - Feature requirements
- [Development Plan](plan.md) - Implementation roadmap
