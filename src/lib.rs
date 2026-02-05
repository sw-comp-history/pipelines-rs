//! # pipelines-rs
//!
//! A mainframe-style 80-byte record pipeline processing library.
//!
//! This library demonstrates historical batch processing patterns used on
//! mainframe systems, where data was processed as fixed-width 80-byte records
//! (matching the width of punch cards).
//!
//! ## Overview
//!
//! Mainframe batch processing typically involved:
//! - **Fixed-width records**: 80 bytes per record (punch card width)
//! - **Sequential processing**: Records processed one at a time
//! - **Field-based operations**: Extracting and manipulating columns
//! - **Pipeline stages**: Filter, merge, split, reformat operations
//!
//! ## Example
//!
//! ```
//! use pipelines_rs::{Pipeline, Record};
//!
//! // Record layout: Last(8) First(10) Dept(10) Salary(8)
//! let records = vec![
//!     Record::from_str("SMITH   JOHN      SALES     00050000"),
//!     Record::from_str("JONES   MARY      ENGINEER  00075000"),
//!     Record::from_str("DOE     JANE      SALES     00060000"),
//! ];
//!
//! let result: Vec<Record> = Pipeline::new(records.into_iter())
//!     .filter(|r| r.field(18, 10).trim() == "SALES")
//!     .collect();
//!
//! assert_eq!(result.len(), 2);
//! ```

pub mod debug_trace;
pub mod dsl;
pub mod error;
pub mod executor;
pub mod pipeline;
pub mod record;
pub mod record_stage;
pub mod stage;

pub use debug_trace::{FlushTrace, RatDebugTrace, RecordTrace};
pub use dsl::{
    Command, DebugCallbacks, DebugInfo, execute_pipeline, execute_pipeline_debug,
    execute_pipeline_rat, execute_pipeline_rat_debug, parse_commands,
};
pub use error::PipelineError;
pub use executor::{execute_rat, execute_rat_traced};
pub use pipeline::{Pipeline, from_lines, from_strings};
pub use record::{RECORD_WIDTH, Record};
pub use record_stage::{RecordStage, command_to_record_stage};
pub use stage::{Filter, Inspect, Map, Reformat, Select, Stage};
