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
//! Supported stages:
//! - `CONSOLE` - Read from input (first) or write to output (last)
//! - `FILTER pos,len = "value"` - Keep records where field equals value
//! - `FILTER pos,len != "value"` - Omit records where field equals value
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

    // Check first stage can be first (source)
    let first = commands.first().unwrap();
    if !first.can_be_first() {
        return Err(format!(
            "{} cannot be the first stage (try CONSOLE or LITERAL)",
            first.name()
        ));
    }

    // Check last stage can be last (sink)
    let last = commands.last().unwrap();
    if !last.can_be_last() {
        return Err(format!(
            "{} cannot be the last stage (try CONSOLE)",
            last.name()
        ));
    }

    // Need at least 2 stages (source and sink)
    if commands.len() < 2 {
        return Err("Pipeline must have at least a source and sink stage".to_string());
    }

    // Check middle stages can be in middle
    for cmd in &commands[1..commands.len() - 1] {
        if !cmd.can_be_middle() {
            return Err(format!(
                "{} cannot be in the middle of a pipeline",
                cmd.name()
            ));
        }
    }

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
        _ => {
            // Other can_be_first stages would be handled here
            return Err(format!("Unhandled source stage: {}", first.name()));
        }
    };

    let input_count = input_records.len();

    // Apply middle commands (skip first source and last sink)
    let middle_commands = &commands[1..commands.len() - 1];
    let output_records = apply_commands(input_records, middle_commands)?;

    let output_count = output_records.len();

    // Format output (CONSOLE writes to output)
    let output_text = output_records
        .iter()
        .map(|r| r.as_str().trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Ok((output_text, input_count, output_count))
}

/// Parsed pipeline command.
#[derive(Debug, Clone)]
enum Command {
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
}

impl Command {
    /// Can this stage be the first stage in a pipeline (source)?
    /// Sources generate or read records without needing upstream input.
    fn can_be_first(&self) -> bool {
        matches!(self, Command::Console | Command::Literal { .. })
    }

    /// Can this stage be the last stage in a pipeline (sink)?
    /// Sinks consume records without passing them downstream.
    fn can_be_last(&self) -> bool {
        matches!(self, Command::Console)
    }

    /// Can this stage be in the middle of a pipeline (filter)?
    /// Filters transform records, requiring both upstream and downstream.
    fn can_be_middle(&self) -> bool {
        // CONSOLE cannot be in middle - it's only a source or sink
        // LITERAL can be in middle - it appends its record after all upstream records
        !matches!(self, Command::Console)
    }

    /// Get the stage name for error messages.
    fn name(&self) -> &'static str {
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
        }
    }
}

/// Parse DSL text into commands.
fn parse_commands(text: &str) -> Result<Vec<Command>, String> {
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

/// Parse a quoted string value.
fn parse_quoted_string(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        Ok(s[1..s.len() - 1].to_string())
    } else if s.starts_with('/') && s.ends_with('/') && s.len() >= 2 {
        // Also accept /pattern/ delimiters (CMS style)
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Err(format!("Value must be quoted: {}", s))
    }
}

/// Parse LOCATE command.
/// Formats:
///   LOCATE "pattern"      - search entire record
///   LOCATE /pattern/      - search entire record (CMS style)
///   LOCATE pos,len "pattern" - search specific field
fn parse_locate(line: &str) -> Result<Command, String> {
    let rest = line[6..].trim(); // Skip "LOCATE"

    // Check if there's a field spec before the pattern
    // Field spec is pos,len followed by quoted string
    if let Some(quote_start) = rest.find('"').or_else(|| rest.find('/')) {
        let before_quote = rest[..quote_start].trim();

        if before_quote.is_empty() {
            // Just LOCATE "pattern"
            let pattern = parse_quoted_string(rest)?;
            Ok(Command::Locate {
                pattern,
                field: None,
            })
        } else {
            // LOCATE pos,len "pattern"
            let parts: Vec<&str> = before_quote.split(',').collect();
            if parts.len() != 2 {
                return Err("LOCATE field spec requires pos,len".to_string());
            }

            let pos: usize = parts[0]
                .trim()
                .parse()
                .map_err(|_| "Invalid position number")?;
            let len: usize = parts[1]
                .trim()
                .parse()
                .map_err(|_| "Invalid length number")?;

            let pattern = parse_quoted_string(&rest[quote_start..])?;
            Ok(Command::Locate {
                pattern,
                field: Some((pos, len)),
            })
        }
    } else {
        Err("LOCATE requires a quoted pattern".to_string())
    }
}

/// Parse NLOCATE command.
fn parse_nlocate(line: &str) -> Result<Command, String> {
    let rest = line[7..].trim(); // Skip "NLOCATE"

    // Check if there's a field spec before the pattern
    if let Some(quote_start) = rest.find('"').or_else(|| rest.find('/')) {
        let before_quote = rest[..quote_start].trim();

        if before_quote.is_empty() {
            let pattern = parse_quoted_string(rest)?;
            Ok(Command::Nlocate {
                pattern,
                field: None,
            })
        } else {
            let parts: Vec<&str> = before_quote.split(',').collect();
            if parts.len() != 2 {
                return Err("NLOCATE field spec requires pos,len".to_string());
            }

            let pos: usize = parts[0]
                .trim()
                .parse()
                .map_err(|_| "Invalid position number")?;
            let len: usize = parts[1]
                .trim()
                .parse()
                .map_err(|_| "Invalid length number")?;

            let pattern = parse_quoted_string(&rest[quote_start..])?;
            Ok(Command::Nlocate {
                pattern,
                field: Some((pos, len)),
            })
        }
    } else {
        Err("NLOCATE requires a quoted pattern".to_string())
    }
}

/// Parse CHANGE command.
/// Format: CHANGE "old" "new" or CHANGE /old/ /new/
fn parse_change(line: &str) -> Result<Command, String> {
    let rest = line[6..].trim(); // Skip "CHANGE"

    // Find the two quoted strings
    // First, find the delimiter (either " or /)
    let delim = if rest.starts_with('"') {
        '"'
    } else if rest.starts_with('/') {
        '/'
    } else {
        return Err("CHANGE requires two quoted strings".to_string());
    };

    // Parse first quoted string
    let after_first_delim = &rest[1..];
    let end_first = after_first_delim
        .find(delim)
        .ok_or("CHANGE: unclosed first string")?;
    let old = after_first_delim[..end_first].to_string();

    // Find second quoted string
    let after_first = after_first_delim[end_first + 1..].trim();
    if !after_first.starts_with(delim) {
        return Err("CHANGE requires two quoted strings".to_string());
    }

    let after_second_delim = &after_first[1..];
    let end_second = after_second_delim
        .find(delim)
        .ok_or("CHANGE: unclosed second string")?;
    let new = after_second_delim[..end_second].to_string();

    Ok(Command::Change { old, new })
}

/// Parse LITERAL command.
/// Format: LITERAL "text" or LITERAL /text/
fn parse_literal(line: &str) -> Result<Command, String> {
    let rest = line[7..].trim(); // Skip "LITERAL"
    let text = parse_quoted_string(rest)?;
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
            // Append a literal record to the stream
            let mut result = records;
            result.push(Record::from_str(text));
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
    }
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
    fn test_pipeline_requires_console() {
        let input = "SMITH   JOHN      SALES     00050000";

        // Missing starting CONSOLE
        let result = execute_pipeline(input, r#"FILTER 18,10 = "SALES" | CONSOLE"#);
        assert!(result.is_err());

        // Missing ending CONSOLE
        let result = execute_pipeline(input, r#"CONSOLE | FILTER 18,10 = "SALES""#);
        assert!(result.is_err());
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
        let pipeline = r#"PIPE LITERAL "HELLO, WORLD"
| CONSOLE
?"#;

        let (output, input_count, output_count) = execute_pipeline("", pipeline).unwrap();

        assert_eq!(input_count, 1); // LITERAL generates 1 record
        assert_eq!(output_count, 1);
        assert_eq!(output, "HELLO, WORLD");
    }

    #[test]
    fn test_literal_first_with_middle_stages() {
        let pipeline = r#"PIPE LITERAL "hello world"
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
        assert!(err.contains("FILTER cannot be the first stage"), "Got: {}", err);
    }

    #[test]
    fn test_literal_cannot_be_last() {
        let pipeline = r#"PIPE CONSOLE
| LITERAL "END""#;
        let result = execute_pipeline("test", pipeline);
        assert!(result.is_err(), "Expected error but got: {:?}", result);
        let err = result.unwrap_err();
        assert!(err.contains("LITERAL cannot be the last stage"), "Got: {}", err);
    }

    #[test]
    fn test_console_cannot_be_middle() {
        let pipeline = r#"PIPE CONSOLE
| CONSOLE
| CONSOLE"#;
        let result = execute_pipeline("test", pipeline);
        assert!(result.is_err(), "Expected error but got: {:?}", result);
        let err = result.unwrap_err();
        assert!(err.contains("CONSOLE cannot be in the middle"), "Got: {}", err);
    }
}
