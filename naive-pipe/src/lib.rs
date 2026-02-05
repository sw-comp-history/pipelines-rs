//! Record-at-a-time (RAT) pipeline executor.
//!
//! This crate provides the record-at-a-time execution model for pipelines-rs.
//! Each input record flows through the entire stage chain before the next
//! record is read, contrasting with the batch executor which processes all
//! records through one stage before moving to the next.

pub mod debug_trace;
pub mod dsl;
pub mod executor;
pub mod record_stage;

pub use debug_trace::{FlushTrace, RatDebugTrace, RecordTrace};
pub use dsl::{execute_pipeline_rat, execute_pipeline_rat_debug};
pub use executor::{execute_rat, execute_rat_traced};
pub use record_stage::{RecordStage, command_to_record_stage};
