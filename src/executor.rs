//! Record-at-a-time (RAT) pipeline executor.
//!
//! Executes a pipeline by pushing each input record through the entire
//! stage chain before reading the next input record. This contrasts with
//! the batch executor which processes all records through one stage before
//! moving to the next.

use crate::Record;
use crate::debug_trace::{FlushTrace, RatDebugTrace, RecordTrace};
use crate::record_stage::RecordStage;

/// Push records through a slice of stages, processing each record
/// through each stage in sequence.
fn push_through_stages(records: Vec<Record>, stages: &mut [Box<dyn RecordStage>]) -> Vec<Record> {
    let mut current = records;
    for stage in stages.iter_mut() {
        let mut next = Vec::new();
        for r in current {
            next.extend(stage.process(r));
        }
        current = next;
    }
    current
}

/// Execute a pipeline in record-at-a-time mode.
///
/// Each input record flows through the entire stage chain before the next
/// record is read. After all records are processed, stages are flushed
/// in order, with flush output propagated through downstream stages.
pub fn execute_rat(input: Vec<Record>, stages: &mut [Box<dyn RecordStage>]) -> Vec<Record> {
    let mut output = Vec::new();

    // Process each input record through the entire stage chain
    for record in input {
        output.extend(push_through_stages(vec![record], stages));
    }

    // Flush propagation: flush each stage and push output through remaining stages
    for i in 0..stages.len() {
        let flush_output = stages[i].flush();
        if !flush_output.is_empty() {
            output.extend(push_through_stages(flush_output, &mut stages[i + 1..]));
        }
    }

    output
}

/// Execute a pipeline in record-at-a-time mode with debug tracing.
///
/// Captures a `RatDebugTrace` showing each record's journey through
/// the pipeline and each stage's flush output.
pub fn execute_rat_traced(
    input: Vec<Record>,
    stages: &mut [Box<dyn RecordStage>],
) -> (Vec<Record>, RatDebugTrace) {
    let stage_names: Vec<String> = stages.iter().map(|s| s.name().to_string()).collect();
    let num_stages = stages.len();
    let mut output = Vec::new();
    let mut record_traces = Vec::new();
    let mut flush_traces = Vec::new();

    // Process each input record through the entire stage chain with tracing
    for record in input {
        let mut pipe_points: Vec<Vec<Record>> = Vec::with_capacity(num_stages + 1);
        pipe_points.push(vec![record.clone()]);

        let mut current = vec![record];
        for stage in stages.iter_mut() {
            let mut next = Vec::new();
            for r in current {
                next.extend(stage.process(r));
            }
            pipe_points.push(next.clone());
            current = next;
        }

        output.extend(current);
        record_traces.push(RecordTrace { pipe_points });
    }

    // Flush propagation with tracing
    for i in 0..num_stages {
        let flush_output = stages[i].flush();
        if !flush_output.is_empty() {
            let mut pipe_points: Vec<Vec<Record>> = Vec::new();
            pipe_points.push(flush_output.clone());

            let mut current = flush_output;
            for stage in stages[i + 1..].iter_mut() {
                let mut next = Vec::new();
                for r in current {
                    next.extend(stage.process(r));
                }
                pipe_points.push(next.clone());
                current = next;
            }

            output.extend(current);
            flush_traces.push(FlushTrace {
                stage_index: i,
                pipe_points,
            });
        }
    }

    let trace = RatDebugTrace {
        stage_names,
        record_traces,
        flush_traces,
    };

    (output, trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Command, execute_pipeline, parse_commands};
    use crate::record_stage::command_to_record_stage;
    use std::fs;
    use std::path::Path;

    /// Helper: run batch executor on a spec file and return trimmed output.
    fn run_batch(input: &str, pipeline: &str) -> String {
        let (output, _, _) = execute_pipeline(input, pipeline).unwrap();
        output
    }

    /// Helper: run RAT executor on a spec file and return trimmed output.
    fn run_rat(input: &str, pipeline: &str) -> String {
        let commands = parse_commands(pipeline).unwrap();
        assert!(commands.len() >= 2);

        let first = &commands[0];
        let input_records: Vec<Record> = match first {
            Command::Console => input
                .lines()
                .filter(|line| !line.is_empty())
                .map(Record::from_str)
                .collect(),
            Command::Literal { text } => vec![Record::from_str(text)],
            Command::Hole => vec![],
            _ => panic!("Unhandled source stage: {}", first.name()),
        };

        let mut stages: Vec<Box<dyn RecordStage>> =
            commands[1..].iter().map(command_to_record_stage).collect();

        let output_records = execute_rat(input_records, &mut stages);
        output_records
            .iter()
            .map(|r| r.as_str().trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Assert RAT and batch executors produce identical output for a spec file.
    fn assert_equivalence(spec_name: &str) {
        let spec_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("specs");
        let input = fs::read_to_string(spec_dir.join("input-fixed-80.data")).unwrap();
        let pipeline = fs::read_to_string(spec_dir.join(spec_name)).unwrap();

        let batch_output = run_batch(&input, &pipeline);
        let rat_output = run_rat(&input, &pipeline);

        assert_eq!(
            batch_output, rat_output,
            "RAT output differs from batch for {spec_name}"
        );
    }

    // --- Unit tests ---

    #[test]
    fn test_simple_passthrough() {
        let input = vec![Record::from_str("A"), Record::from_str("B")];
        let mut stages: Vec<Box<dyn RecordStage>> =
            vec![command_to_record_stage(&Command::Console)];
        let output = execute_rat(input, &mut stages);
        assert_eq!(output.len(), 2);
        assert_eq!(output[0].as_str().trim(), "A");
        assert_eq!(output[1].as_str().trim(), "B");
    }

    #[test]
    fn test_filter_and_count() {
        let input = vec![
            Record::from_str("SALES"),
            Record::from_str("ENGINEER"),
            Record::from_str("SALES"),
        ];
        let mut stages: Vec<Box<dyn RecordStage>> = vec![
            command_to_record_stage(&Command::Locate {
                pattern: "SALES".to_string(),
                field: None,
            }),
            command_to_record_stage(&Command::Count),
        ];
        let output = execute_rat(input, &mut stages);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].as_str().trim(), "COUNT=2");
    }

    #[test]
    fn test_literal_prepends() {
        let input = vec![Record::from_str("A"), Record::from_str("B")];
        let mut stages: Vec<Box<dyn RecordStage>> =
            vec![command_to_record_stage(&Command::Literal {
                text: "HEADER".to_string(),
            })];
        let output = execute_rat(input, &mut stages);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0].as_str().trim(), "HEADER");
        assert_eq!(output[1].as_str().trim(), "A");
        assert_eq!(output[2].as_str().trim(), "B");
    }

    #[test]
    fn test_literal_flush_on_empty() {
        let input: Vec<Record> = vec![];
        let mut stages: Vec<Box<dyn RecordStage>> =
            vec![command_to_record_stage(&Command::Literal {
                text: "ONLY".to_string(),
            })];
        let output = execute_rat(input, &mut stages);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].as_str().trim(), "ONLY");
    }

    #[test]
    fn test_duplicate_expansion() {
        let input = vec![Record::from_str("X")];
        let mut stages: Vec<Box<dyn RecordStage>> =
            vec![command_to_record_stage(&Command::Duplicate { n: 3 })];
        let output = execute_rat(input, &mut stages);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_traced_captures_pipe_points() {
        let input = vec![Record::from_str("A"), Record::from_str("B")];
        let mut stages: Vec<Box<dyn RecordStage>> = vec![command_to_record_stage(&Command::Upper)];
        let (output, trace) = execute_rat_traced(input, &mut stages);
        assert_eq!(output.len(), 2);
        assert_eq!(trace.stage_names, vec!["UPPER"]);
        assert_eq!(trace.record_traces.len(), 2);
        // Each trace has 2 pipe points: input and after UPPER
        assert_eq!(trace.record_traces[0].pipe_points.len(), 2);
        assert_eq!(trace.record_traces[0].pipe_points[0].len(), 1);
        assert_eq!(trace.record_traces[0].pipe_points[1].len(), 1);
    }

    #[test]
    fn test_traced_captures_flush() {
        let input = vec![Record::from_str("A")];
        let mut stages: Vec<Box<dyn RecordStage>> = vec![command_to_record_stage(&Command::Count)];
        let (output, trace) = execute_rat_traced(input, &mut stages);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].as_str().trim(), "COUNT=1");
        assert_eq!(trace.record_traces.len(), 1);
        // Record trace: input goes in, nothing comes out of COUNT
        assert_eq!(trace.record_traces[0].pipe_points[1].len(), 0);
        // Flush trace: COUNT emits on flush
        assert_eq!(trace.flush_traces.len(), 1);
        assert_eq!(trace.flush_traces[0].stage_index, 0);
        assert_eq!(trace.flush_traces[0].pipe_points[0].len(), 1);
    }

    #[test]
    fn test_traced_equivalence() {
        let input = vec![
            Record::from_str("SMITH   JOHN      SALES     00050000"),
            Record::from_str("JONES   MARY      ENGINEER  00075000"),
        ];
        let mut stages: Vec<Box<dyn RecordStage>> = vec![
            command_to_record_stage(&Command::FilterEq {
                pos: 18,
                len: 10,
                value: "SALES".to_string(),
            }),
            command_to_record_stage(&Command::Upper),
        ];
        let mut stages2: Vec<Box<dyn RecordStage>> = vec![
            command_to_record_stage(&Command::FilterEq {
                pos: 18,
                len: 10,
                value: "SALES".to_string(),
            }),
            command_to_record_stage(&Command::Upper),
        ];

        let plain = execute_rat(input.clone(), &mut stages);
        let (traced, _trace) = execute_rat_traced(input, &mut stages2);
        assert_eq!(plain, traced);
    }

    // --- Equivalence tests for all spec files ---

    macro_rules! equiv_test {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                assert_equivalence($file);
            }
        };
    }

    equiv_test!(equiv_change_rename, "change-rename.pipe");
    equiv_test!(equiv_change_strip_prefix, "change-strip-prefix.pipe");
    equiv_test!(equiv_count_filtered, "count-filtered.pipe");
    equiv_test!(equiv_count_records, "count-records.pipe");
    equiv_test!(equiv_duplicate_double, "duplicate-double.pipe");
    equiv_test!(equiv_duplicate_triple, "duplicate-triple.pipe");
    equiv_test!(equiv_engineers_only, "engineers-only.pipe");
    equiv_test!(equiv_filter_sales, "filter-sales.pipe");
    equiv_test!(equiv_literal_footer, "literal-footer.pipe");
    equiv_test!(equiv_literal_header_footer, "literal-header-footer.pipe");
    equiv_test!(equiv_locate_errors, "locate-errors.pipe");
    equiv_test!(equiv_locate_field, "locate-field.pipe");
    equiv_test!(equiv_lower_case, "lower-case.pipe");
    equiv_test!(equiv_multi_filter_count, "multi-filter-count.pipe");
    equiv_test!(equiv_multi_locate_select, "multi-locate-select.pipe");
    equiv_test!(equiv_multi_transform, "multi-transform.pipe");
    equiv_test!(equiv_nlocate_exclude, "nlocate-exclude.pipe");
    equiv_test!(equiv_non_marketing, "non-marketing.pipe");
    equiv_test!(equiv_reverse_text, "reverse-text.pipe");
    equiv_test!(equiv_sales_report, "sales-report.pipe");
    equiv_test!(equiv_skip_take_window, "skip-take-window.pipe");
    equiv_test!(equiv_top_five, "top-five.pipe");
    equiv_test!(equiv_upper_case, "upper-case.pipe");
}
