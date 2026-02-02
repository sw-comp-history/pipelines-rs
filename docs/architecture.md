# Architecture

## Overview

pipelines-rs is a Rust-based pipeline processing library that demonstrates historical mainframe batch processing patterns. It processes fixed-width 80-byte records (matching punch card width) through composable pipeline stages.

## System Architecture

```
+------------------+     +------------------+     +------------------+
|                  |     |                  |     |                  |
|   Input Source   | --> |  Pipeline Core   | --> |  Output Sink     |
|                  |     |                  |     |                  |
+------------------+     +------------------+     +------------------+
                               |
                               v
                    +------------------+
                    |                  |
                    |   Transformers   |
                    |                  |
                    +------------------+
```

## Core Components

### 1. Pipeline Core

**Responsibility**: Orchestrates data flow between stages

**Key Features**:
- Stage composition and chaining
- Error handling and recovery
- Backpressure management
- Resource lifecycle management

### 2. Input Sources

**Responsibility**: Data ingestion from various sources

**Planned Sources**:
- File readers (JSON, CSV, TOML)
- Standard input
- Network streams
- Database queries

### 3. Transformers

**Responsibility**: Data transformation operations

**Planned Transformers**:
- Map/Filter/Reduce operations
- Data validation
- Format conversion
- Aggregation

### 4. Output Sinks

**Responsibility**: Data output to destinations

**Planned Sinks**:
- File writers
- Standard output
- Network endpoints
- Database inserts

## Module Structure

```
src/
+-- main.rs          # Demo application
+-- lib.rs           # Library exports
+-- record.rs        # 80-byte fixed-width Record type
+-- pipeline.rs      # Pipeline struct with fluent API
+-- stage.rs         # Stage trait and implementations
+-- error.rs         # Error types
```

## Data Flow

1. **Initialization**: Pipeline configuration is parsed and validated
2. **Setup**: Sources and sinks are connected, resources allocated
3. **Execution**: Data flows through stages with backpressure control
4. **Completion**: Resources are released, final status reported

## Design Principles

### 1. Type Safety

- Strong typing for pipeline stages
- Compile-time validation where possible
- Generic stage interfaces

### 2. Zero-Copy Where Possible

- Minimize allocations in hot paths
- Use references and borrows effectively
- Stream processing over batch loading

### 3. Error Resilience

- Graceful degradation
- Retry mechanisms
- Clear error propagation

### 4. Composability

- Stages are independent and reusable
- Pipelines can be nested
- Configuration-driven behavior

## Technology Stack

- **Language**: Rust 2024 Edition
- **Async Runtime**: tokio (planned)
- **Serialization**: serde (planned)
- **CLI**: clap (planned)

## Future Considerations

### Scalability

- Parallel stage execution
- Distributed processing support
- Worker pool management

### Observability

- Metrics collection
- Logging integration
- Tracing support

### Extensibility

- Plugin architecture for custom stages
- Dynamic pipeline configuration
- Hot-reload capabilities

## Related Documentation

- [Design Document](design.md) - Detailed design decisions
- [Product Requirements](prd.md) - Feature requirements
- [Development Plan](plan.md) - Implementation roadmap
