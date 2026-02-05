//! Debug trace types for the record-at-a-time executor.
//!
//! These types capture the journey of each record through the pipeline,
//! enabling visualization of record-at-a-time execution flow.

use pipelines_rs::Record;

/// Trace of one input record's journey through the pipeline.
///
/// `pipe_points[0]` is the input (single record), `pipe_points[i]` is the
/// output after stage `i-1`. Length is `num_stages + 1`.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTrace {
    /// Records present at each pipe point between stages.
    pub pipe_points: Vec<Vec<Record>>,
}

/// Trace of one stage's flush output journey through downstream stages.
///
/// `stage_index` identifies which stage produced the flush output.
/// `pipe_points[0]` is the flush output, `pipe_points[i]` is after
/// passing through `i` downstream stages.
#[derive(Debug, Clone, PartialEq)]
pub struct FlushTrace {
    /// Index of the stage that produced this flush output.
    pub stage_index: usize,
    /// Records at each pipe point from flush source through downstream stages.
    pub pipe_points: Vec<Vec<Record>>,
}

/// Complete debug trace of a record-at-a-time pipeline execution.
#[derive(Debug, Clone, PartialEq)]
pub struct RatDebugTrace {
    /// Names of each stage in the pipeline.
    pub stage_names: Vec<String>,
    /// One trace per input record, showing its journey through all stages.
    pub record_traces: Vec<RecordTrace>,
    /// One trace per stage that produced flush output.
    pub flush_traces: Vec<FlushTrace>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_trace_structure() {
        let trace = RecordTrace {
            pipe_points: vec![
                vec![Record::from_str("input")],
                vec![Record::from_str("output")],
            ],
        };
        assert_eq!(trace.pipe_points.len(), 2);
        assert_eq!(trace.pipe_points[0].len(), 1);
        assert_eq!(trace.pipe_points[1].len(), 1);
    }

    #[test]
    fn test_flush_trace_structure() {
        let trace = FlushTrace {
            stage_index: 1,
            pipe_points: vec![vec![Record::from_str("3")]],
        };
        assert_eq!(trace.stage_index, 1);
        assert_eq!(trace.pipe_points.len(), 1);
    }

    #[test]
    fn test_rat_debug_trace_structure() {
        let trace = RatDebugTrace {
            stage_names: vec!["FILTER".to_string(), "COUNT".to_string()],
            record_traces: vec![],
            flush_traces: vec![],
        };
        assert_eq!(trace.stage_names.len(), 2);
        assert!(trace.record_traces.is_empty());
        assert!(trace.flush_traces.is_empty());
    }

    #[test]
    fn test_record_trace_with_expansion() {
        // DUPLICATE produces multiple records at a pipe point
        let trace = RecordTrace {
            pipe_points: vec![
                vec![Record::from_str("A")],
                vec![Record::from_str("A"), Record::from_str("A")],
            ],
        };
        assert_eq!(trace.pipe_points[0].len(), 1);
        assert_eq!(trace.pipe_points[1].len(), 2);
    }

    #[test]
    fn test_record_trace_with_filter() {
        // FILTER can produce zero records at a pipe point
        let trace = RecordTrace {
            pipe_points: vec![vec![Record::from_str("rejected")], vec![]],
        };
        assert_eq!(trace.pipe_points[0].len(), 1);
        assert!(trace.pipe_points[1].is_empty());
    }
}
