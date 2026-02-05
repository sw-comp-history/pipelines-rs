//! Thin wrapper around the library's RAT executor.
//!
//! Replaces the duplicated DSL parser with direct calls to the library's
//! `execute_pipeline_rat` and `execute_pipeline_rat_debug` functions.

use naive_pipe::RatDebugTrace;

/// Execute a pipeline using the record-at-a-time executor.
///
/// Returns (output_text, input_count, output_count) on success.
pub fn execute_pipeline(
    input_text: &str,
    pipeline_text: &str,
) -> Result<(String, usize, usize), String> {
    naive_pipe::execute_pipeline_rat(input_text, pipeline_text)
}

/// Execute a pipeline with debug tracing using the record-at-a-time executor.
///
/// Returns (output_text, input_count, output_count, trace) on success.
pub fn execute_pipeline_debug(
    input_text: &str,
    pipeline_text: &str,
) -> Result<(String, usize, usize, RatDebugTrace), String> {
    naive_pipe::execute_pipeline_rat_debug(input_text, pipeline_text)
}

/// A parsed pipeline line for debugger display.
#[derive(Clone, PartialEq)]
pub struct PipelineLine {
    /// The display text for this command (cleaned up).
    pub text: String,
    /// The stage index in the pipeline.
    pub stage_index: usize,
}

/// Parse pipeline text into display lines, each tagged with its stage index.
pub fn parse_pipeline_lines(pipeline_text: &str) -> Vec<PipelineLine> {
    let mut lines = Vec::new();
    let mut stage_index: usize = 0;

    for line in pipeline_text.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Extract command text (same logic as library parser)
        let cmd_text = if trimmed.to_uppercase().starts_with("PIPE ") {
            trimmed[5..].trim()
        } else if trimmed.eq_ignore_ascii_case("PIPE") {
            continue;
        } else {
            trimmed
        };

        let cmd_text = if let Some(stripped) = cmd_text.strip_prefix('|') {
            stripped.trim()
        } else {
            cmd_text
        };

        let cmd_text = cmd_text.trim_end_matches('|').trim();
        let cmd_text = cmd_text.trim_end_matches('?').trim();

        if cmd_text.is_empty() {
            continue;
        }

        lines.push(PipelineLine {
            text: cmd_text.to_string(),
            stage_index,
        });
        stage_index += 1;
    }

    lines
}
