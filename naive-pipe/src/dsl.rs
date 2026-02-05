//! RAT-specific pipeline execution wrappers.
//!
//! Provides `execute_pipeline_rat` and `execute_pipeline_rat_debug` which
//! parse DSL text and execute using the record-at-a-time executor.

use pipelines_rs::{Command, Record, parse_commands};

use crate::debug_trace::RatDebugTrace;
use crate::executor::{execute_rat, execute_rat_traced};
use crate::record_stage::{RecordStage, command_to_record_stage};

/// Execute a pipeline in record-at-a-time mode.
///
/// Returns (output_text, input_count, output_count) on success.
/// Produces identical output to `execute_pipeline` for all pipelines.
pub fn execute_pipeline_rat(
    input_text: &str,
    pipeline_text: &str,
) -> Result<(String, usize, usize), String> {
    let commands = parse_commands(pipeline_text)?;

    if commands.is_empty() {
        return Err("Pipeline is empty".to_string());
    }
    if commands.len() < 2 {
        return Err("Pipeline must have at least 2 stages".to_string());
    }

    let first = commands.first().unwrap();
    if !first.can_be_first() {
        return Err(format!(
            "{} cannot be the first stage (try CONSOLE, LITERAL, or HOLE)",
            first.name()
        ));
    }

    let input_records: Vec<Record> = match first {
        Command::Console => input_text
            .lines()
            .filter(|line| !line.is_empty())
            .map(Record::from_str)
            .collect(),
        Command::Literal { text } => vec![Record::from_str(text)],
        Command::Hole => vec![],
        _ => return Err(format!("Unhandled source stage: {}", first.name())),
    };

    let input_count = input_records.len();

    let mut stages: Vec<Box<dyn RecordStage>> =
        commands[1..].iter().map(command_to_record_stage).collect();

    let output_records = execute_rat(input_records, &mut stages);
    let output_count = output_records.len();

    let output_text = output_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count))
}

/// Execute a pipeline in record-at-a-time mode with debug tracing.
///
/// Returns (output_text, input_count, output_count, trace) on success.
pub fn execute_pipeline_rat_debug(
    input_text: &str,
    pipeline_text: &str,
) -> Result<(String, usize, usize, RatDebugTrace), String> {
    let commands = parse_commands(pipeline_text)?;

    if commands.is_empty() {
        return Err("Pipeline is empty".to_string());
    }
    if commands.len() < 2 {
        return Err("Pipeline must have at least 2 stages".to_string());
    }

    let first = commands.first().unwrap();
    if !first.can_be_first() {
        return Err(format!(
            "{} cannot be the first stage (try CONSOLE, LITERAL, or HOLE)",
            first.name()
        ));
    }

    let input_records: Vec<Record> = match first {
        Command::Console => input_text
            .lines()
            .filter(|line| !line.is_empty())
            .map(Record::from_str)
            .collect(),
        Command::Literal { text } => vec![Record::from_str(text)],
        Command::Hole => vec![],
        _ => return Err(format!("Unhandled source stage: {}", first.name())),
    };

    let input_count = input_records.len();

    let mut stages: Vec<Box<dyn RecordStage>> =
        commands[1..].iter().map(command_to_record_stage).collect();

    let (output_records, trace) = execute_rat_traced(input_records, &mut stages);
    let output_count = output_records.len();

    let output_text = output_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count, trace))
}
