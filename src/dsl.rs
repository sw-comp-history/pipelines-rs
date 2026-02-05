//! DSL parser and executor for pipeline commands.
//!
//! Pipeline format (CMS Pipelines style):
//! ```text
//! PIPE CONSOLE
//! | FILTER 18,10 = "SALES"
//! | SELECT 0,8,0; 28,8,8
//! | CONSOLE
//! ?
//! ```
//!
//! - `PIPE CONSOLE` starts pipeline, reading from input
//! - `| <stage>` continues to next stage
//! - `| CONSOLE` writes to output
//! - `?` on its own line marks end of pipeline
//!
//! Stage position rules:
//! - First stage must be a source: CONSOLE, LITERAL, or HOLE
//! - Any stage can be in the middle (CONSOLE passes through while printing)
//! - Any stage can be last (output discarded if not a sink like CONSOLE)
//!
//! Supported stages:
//! - `CONSOLE` - Read from input (first), pass through (middle), or write to output (last)
//! - `FILTER pos,len = "value"` - Keep records where field equals value
//! - `FILTER pos,len != "value"` - Omit records where field equals value
//! - `HOLE` - Discard all input, output nothing (like /dev/null)
//! - `SELECT p1,l1,d1; p2,l2,d2; ...` - Select and reposition fields
//! - `TAKE n` - Keep first n records
//! - `SKIP n` - Skip first n records
//! - `LOCATE "pattern"` - Keep records containing pattern (grep-like)
//! - `LOCATE pos,len "pattern"` - Keep records where field contains pattern
//! - `NLOCATE "pattern"` - Keep records NOT containing pattern
//! - `COUNT` - Count records and emit summary (e.g., "COUNT=42")
//! - `CHANGE "old" "new"` - Replace occurrences of old with new (sed-like)
//! - `LITERAL "text"` - Append a literal record to the stream
//! - `UPPER` - Convert records to uppercase
//! - `LOWER` - Convert records to lowercase
//! - `REVERSE` - Reverse characters in each record
//! - `DUPLICATE n` - Repeat each record n times
//! - Lines starting with `#` are comments

use crate::{Pipeline, Record};

/// Callback type for stage start events: `(stage_index, stage_name)`.
type StageStartCallback = Box<dyn Fn(usize, &str) + 'static>;
/// Callback type for stage complete events: `(stage_index, output_count)`.
type StageCompleteCallback = Box<dyn Fn(usize, usize) + 'static>;

/// Debug callback information for stage-by-stage execution.
#[derive(Default)]
pub struct DebugCallbacks {
    pub on_stage_start: Option<StageStartCallback>,
    pub on_stage_complete: Option<StageCompleteCallback>,
}

impl DebugCallbacks {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Debug information for a single pipeline stage execution.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugInfo {
    pub stage_name: String,
    pub input_count: usize,
    pub output_count: usize,
    pub input_records: Option<Vec<Record>>,
    pub output_records: Option<Vec<Record>>,
}

impl DebugInfo {
    pub fn new(stage_name: String, input_count: usize, output_count: usize) -> Self {
        Self {
            stage_name,
            input_count,
            output_count,
            input_records: None,
            output_records: None,
        }
    }

    pub fn with_records(
        stage_name: String,
        input_count: usize,
        output_count: usize,
        input_records: Vec<Record>,
        output_records: Vec<Record>,
    ) -> Self {
        Self {
            stage_name,
            input_count,
            output_count,
            input_records: Some(input_records),
            output_records: Some(output_records),
        }
    }
}

/// Execute a pipeline defined by DSL text on input records.
///
/// Returns (output_text, input_count, output_count) on success.
pub fn execute_pipeline(
    input_text: &str,
    pipeline_text: &str,
) -> Result<(String, usize, usize), String> {
    // Parse pipeline commands
    let commands = parse_commands(pipeline_text)?;

    // Validate pipeline structure
    if commands.is_empty() {
        return Err("Pipeline is empty".to_string());
    }

    // Need at least 2 stages (source and something to receive output)
    if commands.len() < 2 {
        return Err("Pipeline must have at least 2 stages".to_string());
    }

    // Check first stage can be first (source)
    let first = commands.first().unwrap();
    if !first.can_be_first() {
        return Err(format!(
            "{} cannot be the first stage (try CONSOLE, LITERAL, or HOLE)",
            first.name()
        ));
    }

    // Any stage can be last - if not a sink, output is simply discarded
    // Any stage can be in the middle - CONSOLE passes through while printing

    // Get initial records based on first stage type
    let input_records: Vec<Record> = match first {
        Command::Console => {
            // CONSOLE reads from input text
            input_text
                .lines()
                .filter(|line| !line.is_empty())
                .map(Record::from_str)
                .collect()
        }
        Command::Literal { text } => {
            // LITERAL generates a single record
            vec![Record::from_str(text)]
        }
        Command::Hole => {
            // HOLE generates an empty stream
            vec![]
        }
        _ => {
            // Other can_be_first stages would be handled here
            return Err(format!("Unhandled source stage: {}", first.name()));
        }
    };

    let input_count = input_records.len();

    // Apply all commands after the first (source)
    // Any stage can be last - it transforms and the result is output
    let remaining_commands = &commands[1..];
    let output_records = apply_commands(input_records, remaining_commands)?;

    let output_count = output_records.len();

    // Format output (CONSOLE writes to output)
    let output_text = output_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count))
}

/// Execute a pipeline with debug callbacks for stage-by-stage inspection.
///
/// Returns (output_text, input_count, output_count, debug_info) on success.
pub fn execute_pipeline_debug(
    input_text: &str,
    pipeline_text: &str,
    debug: &Option<DebugCallbacks>,
) -> Result<(String, usize, usize, Vec<DebugInfo>), String> {
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
            "{} cannot be first stage (try CONSOLE, LITERAL, or HOLE)",
            first.name()
        ));
    }

    let input_records: Vec<Record> = match first {
        Command::Console => input_text
            .lines()
            .filter(|line| !line.is_empty())
            .map(Record::from_str)
            .collect(),
        Command::Literal { text } => {
            vec![Record::from_str(text)]
        }
        Command::Hole => {
            vec![]
        }
        _ => {
            return Err(format!("Unhandled source stage: {}", first.name()));
        }
    };

    let input_count = input_records.len();
    let mut debug_info: Vec<DebugInfo> = Vec::new();
    let mut current_records = input_records;

    // Record debug info for the source stage (stage 0).
    // The source was already evaluated above; don't re-apply it.
    {
        let source_name = first.name().to_string();
        let source_output_count = current_records.len();

        if let Some(debug) = debug
            && let Some(on_start) = &debug.on_stage_start
        {
            on_start(0, &source_name);
        }

        if let Some(debug) = debug
            && let Some(on_complete) = &debug.on_stage_complete
        {
            on_complete(0, source_output_count);
        }

        let info = if debug.is_some() {
            DebugInfo::with_records(
                source_name,
                0,
                source_output_count,
                vec![],
                current_records.clone(),
            )
        } else {
            DebugInfo::new(source_name, 0, source_output_count)
        };
        debug_info.push(info);
    }

    // Apply remaining commands (after source stage)
    for (idx, cmd) in commands.iter().enumerate().skip(1) {
        let stage_name = cmd.name().to_string();

        if let Some(debug) = debug
            && let Some(on_start) = &debug.on_stage_start
        {
            on_start(idx, &stage_name);
        }

        let input_count_stage = current_records.len();
        let input_records_clone = debug.as_ref().map(|_| current_records.clone());

        current_records = apply_command(current_records, cmd)?;

        let output_count_stage = current_records.len();
        let output_records_clone = debug.as_ref().map(|_| current_records.clone());

        if let Some(debug) = debug
            && let Some(on_complete) = &debug.on_stage_complete
        {
            on_complete(idx, output_count_stage);
        }

        let info = if let Some(input_recs) = input_records_clone {
            DebugInfo::with_records(
                stage_name,
                input_count_stage,
                output_count_stage,
                input_recs,
                output_records_clone.unwrap_or_default(),
            )
        } else {
            DebugInfo::new(stage_name, input_count_stage, output_count_stage)
        };

        debug_info.push(info);
    }

    let output_count = current_records.len();
    let output_text = current_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count, debug_info))
}

/// Parsed pipeline command.
#[derive(Debug, Clone)]
pub enum Command {
    /// CONSOLE - Read from input or write to output
    Console,
    /// FILTER pos,len = "value"
    FilterEq {
        pos: usize,
        len: usize,
        value: String,
    },
    /// FILTER pos,len != "value"
    FilterNe {
        pos: usize,
        len: usize,
        value: String,
    },
    /// SELECT p1,l1,d1; p2,l2,d2; ...
    Select { fields: Vec<(usize, usize, usize)> },
    /// TAKE n
    Take { n: usize },
    /// SKIP n
    Skip { n: usize },
    /// LOCATE "pattern" - keep records containing pattern
    Locate {
        pattern: String,
        /// Optional field restriction (pos, len)
        field: Option<(usize, usize)>,
    },
    /// NLOCATE "pattern" - keep records NOT containing pattern
    Nlocate {
        pattern: String,
        /// Optional field restriction (pos, len)
        field: Option<(usize, usize)>,
    },
    /// COUNT - count records and emit summary
    Count,
    /// CHANGE "old" "new" - replace occurrences
    Change { old: String, new: String },
    /// LITERAL "text" - append a literal record
    Literal { text: String },
    /// UPPER - convert to uppercase
    Upper,
    /// LOWER - convert to lowercase
    Lower,
    /// REVERSE - reverse characters in record
    Reverse,
    /// DUPLICATE n - repeat each record n times
    Duplicate { n: usize },
    /// HOLE - discard all input, output nothing (like /dev/null)
    Hole,
}

impl Command {
    /// Can this stage be the first stage in a pipeline (source)?
    /// Sources generate or read records without needing upstream input.
    pub fn can_be_first(&self) -> bool {
        // CONSOLE reads from input, LITERAL generates a record, HOLE generates empty stream
        matches!(
            self,
            Command::Console | Command::Literal { .. } | Command::Hole
        )
    }

    /// Get the stage name for error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Console => "CONSOLE",
            Command::FilterEq { .. } | Command::FilterNe { .. } => "FILTER",
            Command::Select { .. } => "SELECT",
            Command::Take { .. } => "TAKE",
            Command::Skip { .. } => "SKIP",
            Command::Locate { .. } => "LOCATE",
            Command::Nlocate { .. } => "NLOCATE",
            Command::Count => "COUNT",
            Command::Change { .. } => "CHANGE",
            Command::Literal { .. } => "LITERAL",
            Command::Upper => "UPPER",
            Command::Lower => "LOWER",
            Command::Reverse => "REVERSE",
            Command::Duplicate { .. } => "DUPLICATE",
            Command::Hole => "HOLE",
        }
    }
}

/// Parse DSL text into commands.
pub fn parse_commands(text: &str) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle "PIPE COMMAND" - extract command after PIPE
        let line = if line.to_uppercase().starts_with("PIPE ") {
            line[5..].trim()
        } else if line.eq_ignore_ascii_case("PIPE") {
            // Skip standalone PIPE declaration
            continue;
        } else {
            line
        };

        // Handle continuation lines: "| COMMAND ..."
        let line = if let Some(stripped) = line.strip_prefix('|') {
            stripped.trim()
        } else {
            line
        };

        // Remove trailing pipe delimiter (legacy format)
        let line = line.trim_end_matches('|').trim();

        // Remove trailing ? (explicit end of pipeline)
        let line = line.trim_end_matches('?').trim();

        // Skip if line is now empty
        if line.is_empty() {
            continue;
        }

        let cmd = parse_command(line).map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
        commands.push(cmd);
    }

    Ok(commands)
}

/// Parse a single command line.
fn parse_command(line: &str) -> Result<Command, String> {
    let upper = line.to_uppercase();

    if upper == "CONSOLE" || upper.starts_with("CONSOLE ") {
        Ok(Command::Console)
    } else if upper.starts_with("FILTER") {
        parse_filter(line)
    } else if upper.starts_with("SELECT") {
        parse_select(line)
    } else if upper.starts_with("TAKE") {
        parse_take(line)
    } else if upper.starts_with("SKIP") {
        parse_skip(line)
    } else if upper.starts_with("NLOCATE") {
        parse_nlocate(line)
    } else if upper.starts_with("LOCATE") {
        parse_locate(line)
    } else if upper == "COUNT" || upper.starts_with("COUNT ") {
        Ok(Command::Count)
    } else if upper.starts_with("CHANGE") {
        parse_change(line)
    } else if upper.starts_with("LITERAL") {
        parse_literal(line)
    } else if upper == "UPPER" || upper.starts_with("UPPER ") {
        Ok(Command::Upper)
    } else if upper == "LOWER" || upper.starts_with("LOWER ") {
        Ok(Command::Lower)
    } else if upper == "REVERSE" || upper.starts_with("REVERSE ") {
        Ok(Command::Reverse)
    } else if upper.starts_with("DUPLICATE") {
        parse_duplicate(line)
    } else if upper == "HOLE" || upper.starts_with("HOLE ") {
        Ok(Command::Hole)
    } else {
        Err(format!(
            "Unknown command: {}",
            line.split_whitespace().next().unwrap_or(line)
        ))
    }
}

/// Parse FILTER command.
fn parse_filter(line: &str) -> Result<Command, String> {
    // FILTER pos,len = "value" or FILTER pos,len != "value"
    let rest = line[6..].trim(); // Skip "FILTER"

    // Find the operator
    let (field_part, op, value) = if let Some(idx) = rest.find("!=") {
        let field_part = rest[..idx].trim();
        let value_part = rest[idx + 2..].trim();
        (field_part, "!=", value_part)
    } else if let Some(idx) = rest.find('=') {
        let field_part = rest[..idx].trim();
        let value_part = rest[idx + 1..].trim();
        (field_part, "=", value_part)
    } else {
        return Err("FILTER requires = or != operator".to_string());
    };

    // Parse pos,len
    let parts: Vec<&str> = field_part.split(',').collect();
    if parts.len() != 2 {
        return Err("FILTER requires pos,len before operator".to_string());
    }

    let pos: usize = parts[0]
        .trim()
        .parse()
        .map_err(|_| "Invalid position number")?;
    let len: usize = parts[1]
        .trim()
        .parse()
        .map_err(|_| "Invalid length number")?;

    // Parse quoted value
    let value = parse_quoted_string(value)?;

    if op == "!=" {
        Ok(Command::FilterNe { pos, len, value })
    } else {
        Ok(Command::FilterEq { pos, len, value })
    }
}

/// Parse SELECT command.
fn parse_select(line: &str) -> Result<Command, String> {
    // SELECT p1,l1,d1; p2,l2,d2; ...
    let rest = line[6..].trim(); // Skip "SELECT"

    let mut fields = Vec::new();

    for field_spec in rest.split(';') {
        let field_spec = field_spec.trim();
        if field_spec.is_empty() {
            continue;
        }

        let parts: Vec<&str> = field_spec.split(',').collect();
        if parts.len() != 3 {
            return Err(format!(
                "SELECT field '{}' requires src_pos,len,dest_pos",
                field_spec
            ));
        }

        let src_pos: usize = parts[0]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid source position in '{}'", field_spec))?;
        let len: usize = parts[1]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid length in '{}'", field_spec))?;
        let dest_pos: usize = parts[2]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid destination position in '{}'", field_spec))?;

        fields.push((src_pos, len, dest_pos));
    }

    if fields.is_empty() {
        return Err("SELECT requires at least one field specification".to_string());
    }

    Ok(Command::Select { fields })
}

/// Parse TAKE command.
fn parse_take(line: &str) -> Result<Command, String> {
    let rest = line[4..].trim(); // Skip "TAKE"
    let n: usize = rest.parse().map_err(|_| "TAKE requires a number")?;
    Ok(Command::Take { n })
}

/// Parse SKIP command.
fn parse_skip(line: &str) -> Result<Command, String> {
    let rest = line[4..].trim(); // Skip "SKIP"
    let n: usize = rest.parse().map_err(|_| "SKIP requires a number")?;
    Ok(Command::Skip { n })
}

/// Parse a delimited string using CMS Pipelines convention.
/// The first non-blank character is the delimiter, and the string
/// continues until the next occurrence of that delimiter.
/// Returns (extracted_string, rest_of_input).
fn parse_delimited_string(s: &str) -> Result<(String, &str), String> {
    let s = s.trim_start();
    if s.is_empty() {
        return Err("Expected delimited string".to_string());
    }

    // First character is the delimiter
    let delim = s.chars().next().unwrap();
    let after_delim = &s[delim.len_utf8()..];

    // Find the closing delimiter
    if let Some(end) = after_delim.find(delim) {
        let extracted = after_delim[..end].to_string();
        let rest = &after_delim[end + delim.len_utf8()..];
        Ok((extracted, rest))
    } else {
        Err(format!("Unclosed delimiter '{}'", delim))
    }
}

/// Parse a quoted string value (legacy helper, delegates to parse_delimited_string).
fn parse_quoted_string(s: &str) -> Result<String, String> {
    let (result, _) = parse_delimited_string(s)?;
    Ok(result)
}

/// Parse LOCATE command.
/// CMS Pipelines: Uses first non-blank char as delimiter.
/// Formats:
///   LOCATE /pattern/       - search entire record (/ is delimiter)
///   LOCATE "pattern"       - search entire record (" is delimiter)
///   LOCATE .pattern.       - search entire record (. is delimiter)
///   LOCATE pos,len /pattern/ - search specific field
fn parse_locate(line: &str) -> Result<Command, String> {
    let rest = line[6..].trim(); // Skip "LOCATE"

    if rest.is_empty() {
        return Err("LOCATE requires a pattern".to_string());
    }

    // If first char is a digit, parse field spec first
    if rest.chars().next().unwrap().is_ascii_digit() {
        // Find where the field spec ends (after the comma and second number)
        // Format: pos,len <delimited-pattern>
        let mut parts = rest.splitn(2, |c: char| !c.is_ascii_digit() && c != ',');
        let field_spec = parts.next().unwrap_or("");
        let pattern_part = parts.next().unwrap_or("").trim_start();

        let field_parts: Vec<&str> = field_spec.split(',').collect();
        if field_parts.len() != 2 {
            return Err("LOCATE field spec requires pos,len".to_string());
        }

        let pos: usize = field_parts[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid position number")?;
        let len: usize = field_parts[1]
            .trim()
            .parse()
            .map_err(|_| "Invalid length number")?;

        let (pattern, _) = parse_delimited_string(pattern_part)?;
        Ok(Command::Locate {
            pattern,
            field: Some((pos, len)),
        })
    } else {
        // No field spec, just the delimited pattern
        let (pattern, _) = parse_delimited_string(rest)?;
        Ok(Command::Locate {
            pattern,
            field: None,
        })
    }
}

/// Parse NLOCATE command.
/// CMS Pipelines: Uses first non-blank char as delimiter (same as LOCATE).
fn parse_nlocate(line: &str) -> Result<Command, String> {
    let rest = line[7..].trim(); // Skip "NLOCATE"

    if rest.is_empty() {
        return Err("NLOCATE requires a pattern".to_string());
    }

    // If first char is a digit, parse field spec first
    if rest.chars().next().unwrap().is_ascii_digit() {
        let mut parts = rest.splitn(2, |c: char| !c.is_ascii_digit() && c != ',');
        let field_spec = parts.next().unwrap_or("");
        let pattern_part = parts.next().unwrap_or("").trim_start();

        let field_parts: Vec<&str> = field_spec.split(',').collect();
        if field_parts.len() != 2 {
            return Err("NLOCATE field spec requires pos,len".to_string());
        }

        let pos: usize = field_parts[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid position number")?;
        let len: usize = field_parts[1]
            .trim()
            .parse()
            .map_err(|_| "Invalid length number")?;

        let (pattern, _) = parse_delimited_string(pattern_part)?;
        Ok(Command::Nlocate {
            pattern,
            field: Some((pos, len)),
        })
    } else {
        let (pattern, _) = parse_delimited_string(rest)?;
        Ok(Command::Nlocate {
            pattern,
            field: None,
        })
    }
}

/// Parse CHANGE command.
/// CMS Pipelines: Uses first non-blank char as delimiter.
/// Both strings must use the SAME delimiter.
/// Format: CHANGE /old/ /new/ or CHANGE "old" "new"
fn parse_change(line: &str) -> Result<Command, String> {
    let rest = line[6..].trim(); // Skip "CHANGE"

    if rest.is_empty() {
        return Err("CHANGE requires two delimited strings".to_string());
    }

    // Parse first delimited string
    let (old, after_first) = parse_delimited_string(rest)?;

    // Parse second delimited string (must use same delimiter as first)
    let (new, _) = parse_delimited_string(after_first)?;

    Ok(Command::Change { old, new })
}

/// Parse LITERAL command.
/// CMS Pipelines: LITERAL does NOT use delimiters.
/// Everything after "LITERAL " is the literal text.
/// LITERAL FOO    -> "FOO"
/// LITERAL "FOO"  -> "\"FOO\"" (quotes are part of text)
/// LITERAL /FOO/  -> "/FOO/"  (slashes are part of text)
fn parse_literal(line: &str) -> Result<Command, String> {
    let rest = line[7..].trim_start(); // Skip "LITERAL", keep leading spaces in text
    if rest.is_empty() {
        return Err("LITERAL requires text".to_string());
    }
    // Trim trailing whitespace from the text
    let text = rest.trim_end().to_string();
    Ok(Command::Literal { text })
}

/// Parse DUPLICATE command.
/// Format: DUPLICATE n
fn parse_duplicate(line: &str) -> Result<Command, String> {
    let rest = line[9..].trim(); // Skip "DUPLICATE"
    let n: usize = rest.parse().map_err(|_| "DUPLICATE requires a number")?;
    if n == 0 {
        return Err("DUPLICATE count must be at least 1".to_string());
    }
    Ok(Command::Duplicate { n })
}

/// Apply commands to records.
fn apply_commands(records: Vec<Record>, commands: &[Command]) -> Result<Vec<Record>, String> {
    // We need to collect and re-create pipeline for each command
    // because the Pipeline type changes with each operation
    let mut current: Vec<Record> = records;

    for cmd in commands {
        current = apply_command(current, cmd)?;
    }

    Ok(current)
}

/// Apply a single command to records.
fn apply_command(records: Vec<Record>, cmd: &Command) -> Result<Vec<Record>, String> {
    match cmd {
        Command::Console => {
            // Console in the middle of pipeline just passes through
            Ok(records)
        }
        Command::FilterEq { pos, len, value } => {
            let pos = *pos;
            let len = *len;
            let value = value.clone();
            Ok(Pipeline::new(records.into_iter())
                .filter(move |r| r.field_eq(pos, len, &value))
                .collect())
        }
        Command::FilterNe { pos, len, value } => {
            let pos = *pos;
            let len = *len;
            let value = value.clone();
            Ok(Pipeline::new(records.into_iter())
                .filter(move |r| !r.field_eq(pos, len, &value))
                .collect())
        }
        Command::Select { fields } => {
            let fields = fields.clone();
            Ok(Pipeline::new(records.into_iter()).select(fields).collect())
        }
        Command::Take { n } => Ok(Pipeline::new(records.into_iter()).take(*n).collect()),
        Command::Skip { n } => Ok(Pipeline::new(records.into_iter()).skip(*n).collect()),
        Command::Locate { pattern, field } => {
            let pattern = pattern.clone();
            match field {
                Some((pos, len)) => {
                    let pos = *pos;
                    let len = *len;
                    Ok(Pipeline::new(records.into_iter())
                        .filter(move |r| r.field_contains(pos, len, &pattern))
                        .collect())
                }
                None => {
                    // Search entire record
                    Ok(Pipeline::new(records.into_iter())
                        .filter(move |r| r.as_str().contains(pattern.as_str()))
                        .collect())
                }
            }
        }
        Command::Nlocate { pattern, field } => {
            let pattern = pattern.clone();
            match field {
                Some((pos, len)) => {
                    let pos = *pos;
                    let len = *len;
                    Ok(Pipeline::new(records.into_iter())
                        .filter(move |r| !r.field_contains(pos, len, &pattern))
                        .collect())
                }
                None => Ok(Pipeline::new(records.into_iter())
                    .filter(move |r| !r.as_str().contains(pattern.as_str()))
                    .collect()),
            }
        }
        Command::Count => {
            // Count records and emit a single summary record
            let count = records.len();
            let summary = format!("COUNT={count}");
            Ok(vec![Record::from_str(&summary)])
        }
        Command::Change { old, new } => {
            // Replace all occurrences of old with new in each record
            let old = old.clone();
            let new = new.clone();
            Ok(Pipeline::new(records.into_iter())
                .map(move |r| {
                    let content = r.as_str().replace(&old, &new);
                    Record::from_str(&content)
                })
                .collect())
        }
        Command::Literal { text } => {
            // CMS Pipelines: LITERAL is a "prefix" filter.
            // It outputs its literal text FIRST, then passes through all input records.
            let mut result = vec![Record::from_str(text)];
            result.extend(records);
            Ok(result)
        }
        Command::Upper => {
            // Convert all records to uppercase
            Ok(Pipeline::new(records.into_iter())
                .map(|r| Record::from_str(&r.as_str().to_uppercase()))
                .collect())
        }
        Command::Lower => {
            // Convert all records to lowercase
            Ok(Pipeline::new(records.into_iter())
                .map(|r| Record::from_str(&r.as_str().to_lowercase()))
                .collect())
        }
        Command::Reverse => {
            // Reverse characters in each record (trim first to avoid reversing trailing spaces)
            Ok(Pipeline::new(records.into_iter())
                .map(|r| {
                    let reversed: String = r.as_str().trim_end().chars().rev().collect();
                    Record::from_str(&reversed)
                })
                .collect())
        }
        Command::Duplicate { n } => {
            // Repeat each record n times
            let n = *n;
            Ok(records
                .into_iter()
                .flat_map(|r| std::iter::repeat_n(r, n))
                .collect())
        }
        Command::Hole => {
            // Discard all input records, output nothing (like /dev/null)
            // Consume records but produce empty output
            drop(records);
            Ok(vec![])
        }
    }
}

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

    let mut stages: Vec<Box<dyn crate::record_stage::RecordStage>> = commands[1..]
        .iter()
        .map(crate::record_stage::command_to_record_stage)
        .collect();

    let output_records = crate::executor::execute_rat(input_records, &mut stages);
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
) -> Result<(String, usize, usize, crate::debug_trace::RatDebugTrace), String> {
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

    let mut stages: Vec<Box<dyn crate::record_stage::RecordStage>> = commands[1..]
        .iter()
        .map(crate::record_stage::command_to_record_stage)
        .collect();

    let (output_records, trace) = crate::executor::execute_rat_traced(input_records, &mut stages);
    let output_count = output_records.len();

    let output_text = output_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count, trace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_eq() {
        let cmd = parse_command(r#"FILTER 18,10 = "SALES""#).unwrap();
        match cmd {
            Command::FilterEq { pos, len, value } => {
                assert_eq!(pos, 18);
                assert_eq!(len, 10);
                assert_eq!(value, "SALES");
            }
            _ => panic!("Expected FilterEq"),
        }
    }

    #[test]
    fn test_parse_filter_ne() {
        let cmd = parse_command(r#"FILTER 18,10 != "SALES""#).unwrap();
        match cmd {
            Command::FilterNe { pos, len, value } => {
                assert_eq!(pos, 18);
                assert_eq!(len, 10);
                assert_eq!(value, "SALES");
            }
            _ => panic!("Expected FilterNe"),
        }
    }

    #[test]
    fn test_parse_select() {
        let cmd = parse_command("SELECT 0,8,0; 28,8,8").unwrap();
        match cmd {
            Command::Select { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0], (0, 8, 0));
                assert_eq!(fields[1], (28, 8, 8));
            }
            _ => panic!("Expected Select"),
        }
    }

    #[test]
    fn test_parse_take() {
        let cmd = parse_command("TAKE 5").unwrap();
        match cmd {
            Command::Take { n } => assert_eq!(n, 5),
            _ => panic!("Expected Take"),
        }
    }

    #[test]
    fn test_parse_console() {
        let cmd = parse_command("CONSOLE").unwrap();
        assert!(matches!(cmd, Command::Console));
    }

    #[test]
    fn test_execute_pipeline() {
        let input = "SMITH   JOHN      SALES     00050000\nJONES   MARY      ENGINEER  00075000";
        let pipeline = r#"PIPE CONSOLE
| FILTER 18,10 = "SALES"
| CONSOLE
?"#;

        let (output, input_count, output_count) = execute_pipeline(input, pipeline).unwrap();

        assert_eq!(input_count, 2);
        assert_eq!(output_count, 1);
        assert!(output.contains("SMITH"));
        assert!(!output.contains("JONES"));
    }

    #[test]
    fn test_pipeline_requires_source_first() {
        let input = "SMITH   JOHN      SALES     00050000";

        // FILTER cannot be first stage
        let pipeline = r#"PIPE FILTER 18,10 = "SALES"
| CONSOLE"#;
        let result = execute_pipeline(input, pipeline);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("FILTER cannot be the first stage")
        );
    }

    #[test]
    fn test_any_stage_can_be_last_filter() {
        // FILTER as last stage - output is discarded
        let pipeline = r#"PIPE CONSOLE
| FILTER 18,10 = "SALES""#;
        let result = execute_pipeline("SMITH   JOHN      SALES     00050000", pipeline);
        // Should succeed - output just discarded
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_locate() {
        let input = "SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
DOE     JANE      SALES     00060000";
        let pipeline = r#"PIPE CONSOLE
| LOCATE "SALES"
| CONSOLE
?"#;

        let (output, input_count, output_count) = execute_pipeline(input, pipeline).unwrap();

        assert_eq!(input_count, 3);
        assert_eq!(output_count, 2);
        assert!(output.contains("SMITH"));
        assert!(output.contains("DOE"));
        assert!(!output.contains("JONES"));
    }

    #[test]
    fn test_execute_count() {
        let input = "SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
DOE     JANE      SALES     00060000";
        let pipeline = r#"PIPE CONSOLE
| COUNT
| CONSOLE
?"#;

        let (output, input_count, output_count) = execute_pipeline(input, pipeline).unwrap();

        assert_eq!(input_count, 3);
        assert_eq!(output_count, 1);
        assert_eq!(output, "COUNT=3");
    }

    #[test]
    fn test_literal_as_first_stage() {
        // LITERAL can be the first stage, generating a single record
        // CMS-style: no delimiters, everything after LITERAL is the text
        let pipeline = r#"PIPE LITERAL HELLO, WORLD
| CONSOLE
?"#;

        let (output, input_count, output_count) = execute_pipeline("", pipeline).unwrap();

        assert_eq!(input_count, 1); // LITERAL generates 1 record
        assert_eq!(output_count, 1);
        assert_eq!(output, "HELLO, WORLD");
    }

    #[test]
    fn test_literal_first_with_middle_stages() {
        // CMS-style: no delimiters
        let pipeline = r#"PIPE LITERAL hello world
| UPPER
| CONSOLE
?"#;

        let (output, _input_count, output_count) = execute_pipeline("", pipeline).unwrap();

        assert_eq!(output_count, 1);
        assert_eq!(output, "HELLO WORLD");
    }

    #[test]
    fn test_filter_cannot_be_first() {
        let pipeline = r#"PIPE FILTER 0,5 = "TEST"
| CONSOLE"#;
        let result = execute_pipeline("", pipeline);
        assert!(result.is_err(), "Expected error but got: {:?}", result);
        let err = result.unwrap_err();
        assert!(
            err.contains("FILTER cannot be the first stage"),
            "Got: {}",
            err
        );
    }

    #[test]
    fn test_any_stage_can_be_last() {
        // Output is discarded if last stage isn't a sink
        // CMS-style: LITERAL without delimiters
        let pipeline = r#"PIPE CONSOLE
| LITERAL END"#;
        let result = execute_pipeline("test", pipeline);
        // Should succeed - LITERAL as last just means output is discarded
        assert!(result.is_ok(), "Expected success but got: {:?}", result);
    }

    #[test]
    fn test_console_in_middle() {
        // CONSOLE in middle passes through (useful for debugging)
        let pipeline = r#"PIPE CONSOLE
| CONSOLE
| CONSOLE"#;
        let (output, input_count, output_count) = execute_pipeline("test", pipeline).unwrap();
        assert_eq!(input_count, 1);
        assert_eq!(output_count, 1);
        assert_eq!(output, "test");
    }

    #[test]
    fn test_hole_as_first() {
        // HOLE as first generates empty stream
        // CMS-style: LITERAL without delimiters
        let pipeline = r#"PIPE HOLE
| LITERAL No input
| CONSOLE"#;
        let (output, input_count, output_count) = execute_pipeline("ignored", pipeline).unwrap();
        assert_eq!(input_count, 0); // HOLE generates 0 records
        assert_eq!(output_count, 1); // LITERAL adds 1
        assert_eq!(output, "No input");
    }

    #[test]
    fn test_hole_in_middle() {
        // HOLE discards upstream, downstream gets nothing (before LITERAL)
        // CMS-style: LITERAL without delimiters
        let pipeline = r#"PIPE CONSOLE
| COUNT
| HOLE
| LITERAL Discarded
| CONSOLE"#;
        let (output, input_count, output_count) = execute_pipeline("A\nB\nC", pipeline).unwrap();
        assert_eq!(input_count, 3);
        assert_eq!(output_count, 1);
        assert_eq!(output, "Discarded");
    }

    #[test]
    fn test_hole_as_last() {
        // HOLE as last discards all output
        let pipeline = r#"PIPE CONSOLE
| HOLE"#;
        let (output, input_count, output_count) = execute_pipeline("test", pipeline).unwrap();
        assert_eq!(input_count, 1); // CONSOLE read 1 record
        assert_eq!(output_count, 0); // HOLE discarded it
        assert!(output.is_empty());
    }

    #[test]
    fn test_debug_info_new() {
        let info = DebugInfo::new("CONSOLE".to_string(), 3, 2);
        assert_eq!(info.stage_name, "CONSOLE");
        assert_eq!(info.input_count, 3);
        assert_eq!(info.output_count, 2);
        assert!(info.input_records.is_none());
        assert!(info.output_records.is_none());
    }

    #[test]
    fn test_debug_info_with_records() {
        let input_records = vec![
            Record::from_str("ABC"),
            Record::from_str("DEF"),
            Record::from_str("GHI"),
        ];
        let output_records = vec![Record::from_str("ABC"), Record::from_str("DEF")];
        let info = DebugInfo::with_records(
            "FILTER".to_string(),
            3,
            2,
            input_records.clone(),
            output_records.clone(),
        );
        assert_eq!(info.stage_name, "FILTER");
        assert_eq!(info.input_count, 3);
        assert_eq!(info.output_count, 2);
        assert!(info.input_records.is_some());
        assert!(info.output_records.is_some());
        assert_eq!(info.input_records.unwrap(), input_records);
        assert_eq!(info.output_records.unwrap(), output_records);
    }

    #[test]
    fn test_execute_pipeline_debug_without_callbacks() {
        let pipeline = r#"PIPE CONSOLE
| CONSOLE"#;
        let input = "A\nB\nC";
        let (output, input_count, output_count, debug_info) =
            execute_pipeline_debug(input, pipeline, &None).unwrap();
        assert_eq!(input_count, 3);
        assert_eq!(output_count, 3);
        assert_eq!(output, "A\nB\nC");
        assert_eq!(debug_info.len(), 2);
        assert_eq!(debug_info[0].stage_name, "CONSOLE");
        assert_eq!(debug_info[1].stage_name, "CONSOLE");
        assert!(debug_info[0].input_records.is_none());
        assert!(debug_info[0].output_records.is_none());
    }
}
