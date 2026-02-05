//! Record-at-a-time stage trait and implementations.
//!
//! Each `RecordStage` processes one record at a time, returning zero or more
//! output records. This enables the record-at-a-time (RAT) executor to show
//! individual record flow through the pipeline.

use pipelines_rs::Command;
use pipelines_rs::Record;

/// A pipeline stage that processes records one at a time.
///
/// Unlike the batch `Stage` trait, `RecordStage` returns `Vec<Record>`
/// to support stages that produce multiple outputs (DUPLICATE), no
/// output during processing (COUNT), or extra output (LITERAL).
pub trait RecordStage {
    /// Process a single input record, returning zero or more output records.
    fn process(&mut self, record: Record) -> Vec<Record>;

    /// Flush any accumulated state, returning final output records.
    ///
    /// Called after all input records have been processed. Stages like
    /// COUNT use this to emit their summary record.
    fn flush(&mut self) -> Vec<Record> {
        vec![]
    }

    /// The display name of this stage.
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Stage implementations
// ---------------------------------------------------------------------------

/// CONSOLE - passes records through unchanged.
pub struct ConsoleStage;

impl RecordStage for ConsoleStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        vec![record]
    }

    fn name(&self) -> &str {
        "CONSOLE"
    }
}

/// FILTER pos,len = "value" - keeps records where field equals value.
pub struct FilterEqStage {
    pos: usize,
    len: usize,
    value: String,
}

impl RecordStage for FilterEqStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        if record.field_eq(self.pos, self.len, &self.value) {
            vec![record]
        } else {
            vec![]
        }
    }

    fn name(&self) -> &str {
        "FILTER"
    }
}

/// FILTER pos,len != "value" - keeps records where field does not equal value.
pub struct FilterNeStage {
    pos: usize,
    len: usize,
    value: String,
}

impl RecordStage for FilterNeStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        if !record.field_eq(self.pos, self.len, &self.value) {
            vec![record]
        } else {
            vec![]
        }
    }

    fn name(&self) -> &str {
        "FILTER"
    }
}

/// SELECT - extracts and repositions fields.
pub struct SelectStage {
    fields: Vec<(usize, usize, usize)>,
}

impl RecordStage for SelectStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        let mut output = Record::new();
        for &(src, len, dest) in &self.fields {
            output.set_field(dest, len, record.field(src, len));
        }
        vec![output]
    }

    fn name(&self) -> &str {
        "SELECT"
    }
}

/// TAKE n - keeps the first n records, discards the rest.
pub struct TakeStage {
    n: usize,
    seen: usize,
}

impl RecordStage for TakeStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        if self.seen < self.n {
            self.seen += 1;
            vec![record]
        } else {
            vec![]
        }
    }

    fn name(&self) -> &str {
        "TAKE"
    }
}

/// SKIP n - skips the first n records, passes the rest.
pub struct SkipStage {
    n: usize,
    seen: usize,
}

impl RecordStage for SkipStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        if self.seen < self.n {
            self.seen += 1;
            vec![]
        } else {
            vec![record]
        }
    }

    fn name(&self) -> &str {
        "SKIP"
    }
}

/// LOCATE - keeps records containing a pattern.
pub struct LocateStage {
    pattern: String,
    field: Option<(usize, usize)>,
}

impl RecordStage for LocateStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        let matches = match self.field {
            Some((pos, len)) => record.field_contains(pos, len, &self.pattern),
            None => record.as_str().contains(self.pattern.as_str()),
        };
        if matches { vec![record] } else { vec![] }
    }

    fn name(&self) -> &str {
        "LOCATE"
    }
}

/// NLOCATE - keeps records NOT containing a pattern.
pub struct NlocateStage {
    pattern: String,
    field: Option<(usize, usize)>,
}

impl RecordStage for NlocateStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        let matches = match self.field {
            Some((pos, len)) => record.field_contains(pos, len, &self.pattern),
            None => record.as_str().contains(self.pattern.as_str()),
        };
        if matches { vec![] } else { vec![record] }
    }

    fn name(&self) -> &str {
        "NLOCATE"
    }
}

/// COUNT - counts records and emits summary on flush.
pub struct CountStage {
    count: usize,
}

impl RecordStage for CountStage {
    fn process(&mut self, _record: Record) -> Vec<Record> {
        self.count += 1;
        vec![]
    }

    fn flush(&mut self) -> Vec<Record> {
        vec![Record::from_str(&self.count.to_string())]
    }

    fn name(&self) -> &str {
        "COUNT"
    }
}

/// CHANGE "old" "new" - replaces occurrences in each record.
pub struct ChangeStage {
    old: String,
    new: String,
}

impl RecordStage for ChangeStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        let content = record.as_str().replace(&self.old, &self.new);
        vec![Record::from_str(&content)]
    }

    fn name(&self) -> &str {
        "CHANGE"
    }
}

/// LITERAL "text" - emits a literal record before the first input record.
///
/// On `flush()`, emits the literal if no input records were received
/// (matching batch behavior where LITERAL prepends to an empty stream).
pub struct LiteralStage {
    text: String,
    emitted: bool,
}

impl RecordStage for LiteralStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        if !self.emitted {
            self.emitted = true;
            vec![Record::from_str(&self.text), record]
        } else {
            vec![record]
        }
    }

    fn flush(&mut self) -> Vec<Record> {
        if !self.emitted {
            self.emitted = true;
            vec![Record::from_str(&self.text)]
        } else {
            vec![]
        }
    }

    fn name(&self) -> &str {
        "LITERAL"
    }
}

/// UPPER - converts records to uppercase.
pub struct UpperStage;

impl RecordStage for UpperStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        vec![Record::from_str(&record.as_str().to_uppercase())]
    }

    fn name(&self) -> &str {
        "UPPER"
    }
}

/// LOWER - converts records to lowercase.
pub struct LowerStage;

impl RecordStage for LowerStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        vec![Record::from_str(&record.as_str().to_lowercase())]
    }

    fn name(&self) -> &str {
        "LOWER"
    }
}

/// REVERSE - reverses characters in each record.
pub struct ReverseStage;

impl RecordStage for ReverseStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        let reversed: String = record.as_str().trim_end().chars().rev().collect();
        vec![Record::from_str(&reversed)]
    }

    fn name(&self) -> &str {
        "REVERSE"
    }
}

/// DUPLICATE n - repeats each record n times.
pub struct DuplicateStage {
    n: usize,
}

impl RecordStage for DuplicateStage {
    fn process(&mut self, record: Record) -> Vec<Record> {
        std::iter::repeat_n(record, self.n).collect()
    }

    fn name(&self) -> &str {
        "DUPLICATE"
    }
}

/// HOLE - discards all input, outputs nothing.
pub struct HoleStage;

impl RecordStage for HoleStage {
    fn process(&mut self, _record: Record) -> Vec<Record> {
        vec![]
    }

    fn name(&self) -> &str {
        "HOLE"
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create a `RecordStage` from a parsed `Command`.
pub fn command_to_record_stage(cmd: &Command) -> Box<dyn RecordStage> {
    match cmd {
        Command::Console => Box::new(ConsoleStage),
        Command::FilterEq { pos, len, value } => Box::new(FilterEqStage {
            pos: *pos,
            len: *len,
            value: value.clone(),
        }),
        Command::FilterNe { pos, len, value } => Box::new(FilterNeStage {
            pos: *pos,
            len: *len,
            value: value.clone(),
        }),
        Command::Select { fields } => Box::new(SelectStage {
            fields: fields.clone(),
        }),
        Command::Take { n } => Box::new(TakeStage { n: *n, seen: 0 }),
        Command::Skip { n } => Box::new(SkipStage { n: *n, seen: 0 }),
        Command::Locate { pattern, field } => Box::new(LocateStage {
            pattern: pattern.clone(),
            field: *field,
        }),
        Command::Nlocate { pattern, field } => Box::new(NlocateStage {
            pattern: pattern.clone(),
            field: *field,
        }),
        Command::Count => Box::new(CountStage { count: 0 }),
        Command::Change { old, new } => Box::new(ChangeStage {
            old: old.clone(),
            new: new.clone(),
        }),
        Command::Literal { text } => Box::new(LiteralStage {
            text: text.clone(),
            emitted: false,
        }),
        Command::Upper => Box::new(UpperStage),
        Command::Lower => Box::new(LowerStage),
        Command::Reverse => Box::new(ReverseStage),
        Command::Duplicate { n } => Box::new(DuplicateStage { n: *n }),
        Command::Hole => Box::new(HoleStage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_passthrough() {
        let mut stage = ConsoleStage;
        let r = Record::from_str("hello");
        let out = stage.process(r.clone());
        assert_eq!(out, vec![r]);
    }

    #[test]
    fn test_filter_eq_pass() {
        let mut stage = FilterEqStage {
            pos: 0,
            len: 5,
            value: "HELLO".to_string(),
        };
        let r = Record::from_str("HELLO");
        assert_eq!(stage.process(r).len(), 1);
    }

    #[test]
    fn test_filter_eq_reject() {
        let mut stage = FilterEqStage {
            pos: 0,
            len: 5,
            value: "HELLO".to_string(),
        };
        let r = Record::from_str("WORLD");
        assert!(stage.process(r).is_empty());
    }

    #[test]
    fn test_filter_ne_pass() {
        let mut stage = FilterNeStage {
            pos: 0,
            len: 5,
            value: "HELLO".to_string(),
        };
        let r = Record::from_str("WORLD");
        assert_eq!(stage.process(r).len(), 1);
    }

    #[test]
    fn test_filter_ne_reject() {
        let mut stage = FilterNeStage {
            pos: 0,
            len: 5,
            value: "HELLO".to_string(),
        };
        let r = Record::from_str("HELLO");
        assert!(stage.process(r).is_empty());
    }

    #[test]
    fn test_select_stage() {
        let mut stage = SelectStage {
            fields: vec![(0, 5, 0), (10, 5, 5)],
        };
        let out = stage.process(Record::from_str("ABCDE     FGHIJ"));
        assert_eq!(&out[0].as_str()[..10], "ABCDEFGHIJ");
    }

    #[test]
    fn test_take_stage() {
        let mut stage = TakeStage { n: 2, seen: 0 };
        assert_eq!(stage.process(Record::from_str("A")).len(), 1);
        assert_eq!(stage.process(Record::from_str("B")).len(), 1);
        assert!(stage.process(Record::from_str("C")).is_empty());
    }

    #[test]
    fn test_skip_stage() {
        let mut stage = SkipStage { n: 2, seen: 0 };
        assert!(stage.process(Record::from_str("A")).is_empty());
        assert!(stage.process(Record::from_str("B")).is_empty());
        assert_eq!(stage.process(Record::from_str("C")).len(), 1);
    }

    #[test]
    fn test_locate_whole_record() {
        let mut stage = LocateStage {
            pattern: "SALES".to_string(),
            field: None,
        };
        assert_eq!(
            stage
                .process(Record::from_str("SMITH   JOHN      SALES"))
                .len(),
            1
        );
        assert!(
            stage
                .process(Record::from_str("JONES   MARY      ENGINEER"))
                .is_empty()
        );
    }

    #[test]
    fn test_locate_field() {
        let mut stage = LocateStage {
            pattern: "SALES".to_string(),
            field: Some((18, 10)),
        };
        assert_eq!(
            stage
                .process(Record::from_str("SMITH   JOHN      SALES"))
                .len(),
            1
        );
        assert!(
            stage
                .process(Record::from_str("JONES   MARY      ENGINEER"))
                .is_empty()
        );
    }

    #[test]
    fn test_nlocate_stage() {
        let mut stage = NlocateStage {
            pattern: "SALES".to_string(),
            field: None,
        };
        assert!(
            stage
                .process(Record::from_str("SMITH   JOHN      SALES"))
                .is_empty()
        );
        assert_eq!(
            stage
                .process(Record::from_str("JONES   MARY      ENGINEER"))
                .len(),
            1
        );
    }

    #[test]
    fn test_count_stage() {
        let mut stage = CountStage { count: 0 };
        assert!(stage.process(Record::from_str("A")).is_empty());
        assert!(stage.process(Record::from_str("B")).is_empty());
        assert!(stage.process(Record::from_str("C")).is_empty());
        let flushed = stage.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].as_str().trim(), "3");
    }

    #[test]
    fn test_count_stage_zero() {
        let mut stage = CountStage { count: 0 };
        let flushed = stage.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].as_str().trim(), "0");
    }

    #[test]
    fn test_change_stage() {
        let mut stage = ChangeStage {
            old: "HELLO".to_string(),
            new: "WORLD".to_string(),
        };
        let out = stage.process(Record::from_str("HELLO THERE"));
        assert!(out[0].as_str().starts_with("WORLD THERE"));
    }

    #[test]
    fn test_literal_with_input() {
        let mut stage = LiteralStage {
            text: "HEADER".to_string(),
            emitted: false,
        };
        let out1 = stage.process(Record::from_str("A"));
        assert_eq!(out1.len(), 2);
        assert_eq!(out1[0].as_str().trim(), "HEADER");
        assert_eq!(out1[1].as_str().trim(), "A");

        let out2 = stage.process(Record::from_str("B"));
        assert_eq!(out2.len(), 1);
        assert_eq!(out2[0].as_str().trim(), "B");

        assert!(stage.flush().is_empty());
    }

    #[test]
    fn test_literal_no_input() {
        let mut stage = LiteralStage {
            text: "HEADER".to_string(),
            emitted: false,
        };
        let flushed = stage.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].as_str().trim(), "HEADER");
    }

    #[test]
    fn test_upper_stage() {
        let mut stage = UpperStage;
        let out = stage.process(Record::from_str("hello world"));
        assert_eq!(out[0].as_str().trim(), "HELLO WORLD");
    }

    #[test]
    fn test_lower_stage() {
        let mut stage = LowerStage;
        let out = stage.process(Record::from_str("HELLO WORLD"));
        assert_eq!(out[0].as_str().trim(), "hello world");
    }

    #[test]
    fn test_reverse_stage() {
        let mut stage = ReverseStage;
        let out = stage.process(Record::from_str("ABC"));
        assert_eq!(out[0].as_str().trim(), "CBA");
    }

    #[test]
    fn test_duplicate_stage() {
        let mut stage = DuplicateStage { n: 3 };
        let out = stage.process(Record::from_str("A"));
        assert_eq!(out.len(), 3);
        for r in &out {
            assert_eq!(r.as_str().trim(), "A");
        }
    }

    #[test]
    fn test_hole_stage() {
        let mut stage = HoleStage;
        assert!(stage.process(Record::from_str("A")).is_empty());
        assert!(stage.flush().is_empty());
    }

    #[test]
    fn test_factory_upper() {
        let cmd = Command::Upper;
        let mut stage = command_to_record_stage(&cmd);
        assert_eq!(stage.name(), "UPPER");
        let out = stage.process(Record::from_str("hello"));
        assert_eq!(out[0].as_str().trim(), "HELLO");
    }

    #[test]
    fn test_factory_count() {
        let cmd = Command::Count;
        let mut stage = command_to_record_stage(&cmd);
        assert_eq!(stage.name(), "COUNT");
        stage.process(Record::from_str("A"));
        stage.process(Record::from_str("B"));
        let flushed = stage.flush();
        assert_eq!(flushed[0].as_str().trim(), "2");
    }

    #[test]
    fn test_factory_duplicate() {
        let cmd = Command::Duplicate { n: 2 };
        let mut stage = command_to_record_stage(&cmd);
        assert_eq!(stage.name(), "DUPLICATE");
        let out = stage.process(Record::from_str("X"));
        assert_eq!(out.len(), 2);
    }
}
