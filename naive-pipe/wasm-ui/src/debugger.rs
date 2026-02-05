//! Visual debugger for record-at-a-time pipeline inspection.
//!
//! Stepping is per-pipe-point: each step reveals the next pipe point
//! in a record's journey through the pipeline. When a record is filtered
//! (empty pipe point), the next step moves to the next record. After all
//! records, flush traces are stepped similarly.
//!
//! As records reach the final stage, their output appears progressively
//! in the output panel (not buffered until the end).
//!
//! **Indexing note:** `execute_pipeline_rat_debug` handles the source
//! stage separately. `trace.stage_names` excludes the source;
//! `pipe_points[0]` is the input to the first processing stage.
//! `pipeline_lines` includes ALL stages (source at index 0). The pipe
//! point between pipeline stage `i` and `i+1` maps to `pipe_points[i]`.

use naive_pipe::RatDebugTrace;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::app::TUTORIALS;
use crate::dsl::PipelineLine;

const DOTS: &str = "\u{00B7}\u{00B7}\u{00B7}";

/// A watch placed at a pipe point between stages.
#[derive(Clone, PartialEq)]
pub struct Watch {
    /// Display label: "w1", "w2", etc.
    pub label: String,
    /// Pipeline stage index of the stage above this pipe point.
    pub stage_index: usize,
}

/// A breakpoint at a pipe point between stages.
#[derive(Clone, PartialEq)]
pub struct Breakpoint {
    pub stage_index: usize,
}

/// Debugger state (stored in AppState).
#[derive(Clone, PartialEq)]
pub struct DebuggerState {
    pub active: bool,
    pub trace: Option<RatDebugTrace>,
    /// Global step counter (0 = initial, 1..=total_steps).
    pub current_step: usize,
    /// Total granular steps across all records and flushes.
    pub total_steps: usize,
    /// Index into record_traces or flush_traces.
    pub trace_idx: usize,
    /// Pipe points revealed for current trace entry (0 = none).
    pub visible_pp: usize,
    /// True when stepping through flush traces.
    pub in_flush_phase: bool,
    /// Output accumulated so far (records that reached the sink).
    pub accumulated_output: String,
    pub watches: Vec<Watch>,
    pub next_watch_id: usize,
    pub breakpoints: Vec<Breakpoint>,
    pub hit_breakpoint: Option<usize>,
    pub stage_count: usize,
    /// Full pipeline output (computed up front for run-all).
    pub output_text: String,
    pub input_count: usize,
    pub output_count: usize,
    pub pipeline_lines: Vec<PipelineLine>,
    pub error: Option<String>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self {
            active: false,
            trace: None,
            current_step: 0,
            total_steps: 0,
            trace_idx: 0,
            visible_pp: 0,
            in_flush_phase: false,
            accumulated_output: String::new(),
            watches: Vec::new(),
            next_watch_id: 1,
            breakpoints: Vec::new(),
            hit_breakpoint: None,
            stage_count: 0,
            output_text: String::new(),
            input_count: 0,
            output_count: 0,
            pipeline_lines: Vec::new(),
            error: None,
        }
    }
}

/// Max UI pipe points to reveal for a record trace.
/// Stops at the first empty pipe point (filter) + 1.
fn max_pp_for_record(rt: &naive_pipe::RecordTrace) -> usize {
    // Last pipe_point is final output (not a UI pipe point).
    let ui_count = rt.pipe_points.len().saturating_sub(1);
    for i in 0..ui_count {
        if rt.pipe_points[i].is_empty() {
            return i + 1;
        }
    }
    ui_count
}

/// Max UI pipe points to reveal for a flush trace.
fn max_pp_for_flush(ft: &naive_pipe::FlushTrace, num_ui_pp: usize) -> usize {
    let start = ft.stage_index + 1;
    let viewable = num_ui_pp.saturating_sub(start).min(ft.pipe_points.len());
    for i in 0..viewable {
        if ft.pipe_points[i].is_empty() {
            return i + 1;
        }
    }
    viewable
}

impl DebuggerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_watch(&mut self, stage_index: usize) {
        if let Some(pos) = self
            .watches
            .iter()
            .position(|w| w.stage_index == stage_index)
        {
            self.watches.remove(pos);
        } else {
            let label = format!("w{}", self.next_watch_id);
            self.next_watch_id += 1;
            self.watches.push(Watch { label, stage_index });
        }
    }

    pub fn remove_watch(&mut self, label: &str) {
        self.watches.retain(|w| w.label != label);
    }

    pub fn watches_at(&self, stage_index: usize) -> Vec<&Watch> {
        self.watches
            .iter()
            .filter(|w| w.stage_index == stage_index)
            .collect()
    }

    pub fn toggle_breakpoint(&mut self, stage_index: usize) {
        if let Some(pos) = self
            .breakpoints
            .iter()
            .position(|b| b.stage_index == stage_index)
        {
            self.breakpoints.remove(pos);
        } else {
            self.breakpoints.push(Breakpoint { stage_index });
        }
    }

    pub fn has_breakpoint(&self, stage_index: usize) -> bool {
        self.breakpoints
            .iter()
            .any(|b| b.stage_index == stage_index)
    }

    fn record_count(&self) -> usize {
        self.trace
            .as_ref()
            .map(|t| t.record_traces.len())
            .unwrap_or(0)
    }

    fn flush_count(&self) -> usize {
        self.trace
            .as_ref()
            .map(|t| t.flush_traces.len())
            .unwrap_or(0)
    }

    fn num_ui_pipe_points(&self) -> usize {
        self.pipeline_lines.len().saturating_sub(1)
    }

    fn current_max_pp(&self) -> usize {
        let trace = match &self.trace {
            Some(t) => t,
            None => return 0,
        };
        if !self.in_flush_phase {
            trace
                .record_traces
                .get(self.trace_idx)
                .map(max_pp_for_record)
                .unwrap_or(0)
        } else {
            let num_ui = self.num_ui_pipe_points();
            trace
                .flush_traces
                .get(self.trace_idx)
                .map(|ft| max_pp_for_flush(ft, num_ui))
                .unwrap_or(0)
        }
    }

    /// Compute total granular steps for all traces.
    pub fn compute_total_steps(&self) -> usize {
        let trace = match &self.trace {
            Some(t) => t,
            None => return 0,
        };
        let num_ui = self.num_ui_pipe_points();
        let record_steps: usize = trace.record_traces.iter().map(max_pp_for_record).sum();
        let flush_steps: usize = trace
            .flush_traces
            .iter()
            .map(|ft| max_pp_for_flush(ft, num_ui))
            .sum();
        record_steps + flush_steps
    }

    /// Returns the pipe point index most recently revealed.
    pub fn currently_revealed_pipe_point(&self) -> Option<usize> {
        if self.current_step == 0 || self.visible_pp == 0 {
            return None;
        }
        if !self.in_flush_phase {
            Some(self.visible_pp - 1)
        } else {
            let ft = self.trace.as_ref()?.flush_traces.get(self.trace_idx)?;
            Some(ft.stage_index + self.visible_pp)
        }
    }

    /// Advance one granular step. Collects output when a trace completes.
    /// Returns `true` if a breakpoint was hit.
    pub fn advance(&mut self) -> bool {
        if self.current_step >= self.total_steps {
            return false;
        }
        let max_pp = self.current_max_pp();
        if self.visible_pp < max_pp {
            self.visible_pp += 1;
            if self.visible_pp == max_pp {
                self.collect_output();
            }
        } else {
            let rc = self.record_count();
            if !self.in_flush_phase {
                self.trace_idx += 1;
                if self.trace_idx >= rc {
                    self.in_flush_phase = true;
                    self.trace_idx = 0;
                }
            } else {
                self.trace_idx += 1;
            }
            self.visible_pp = 1;
        }
        self.current_step += 1;
        if let Some(pp) = self.currently_revealed_pipe_point() {
            if self.has_breakpoint(pp) {
                self.hit_breakpoint = Some(pp);
                return true;
            }
        }
        false
    }

    /// Collect output records from the current trace entry's final pipe point.
    fn collect_output(&mut self) {
        let records_text: Vec<String> = {
            let trace = match &self.trace {
                Some(t) => t,
                None => return,
            };
            let final_pp = if !self.in_flush_phase {
                trace
                    .record_traces
                    .get(self.trace_idx)
                    .and_then(|rt| rt.pipe_points.last())
            } else {
                trace
                    .flush_traces
                    .get(self.trace_idx)
                    .and_then(|ft| ft.pipe_points.last())
            };
            match final_pp {
                Some(records) if !records.is_empty() => records
                    .iter()
                    .map(|r| r.as_str().trim_end().to_string())
                    .collect(),
                _ => return,
            }
        };
        for text in &records_text {
            if !self.accumulated_output.is_empty() {
                self.accumulated_output.push('\n');
            }
            self.accumulated_output.push_str(text);
        }
    }

    /// Step counter label: "Record 2 of 8 (1/3)" or "Flush 1 of 2 (1/1)".
    fn step_label(&self) -> String {
        if !self.active || self.total_steps == 0 || self.current_step == 0 {
            return String::new();
        }
        let max_pp = self.current_max_pp();
        let prefix = if self.hit_breakpoint.is_some() {
            "[BP] "
        } else {
            ""
        };
        if !self.in_flush_phase {
            let rc = self.record_count();
            format!(
                "{prefix}Record {} of {} ({}/{})",
                self.trace_idx + 1,
                rc,
                self.visible_pp,
                max_pp
            )
        } else {
            let fc = self.flush_count();
            format!(
                "{prefix}Flush {} of {} ({}/{})",
                self.trace_idx + 1,
                fc,
                self.visible_pp,
                max_pp
            )
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DebuggerProps {
    pub state: DebuggerState,
    pub on_run: Callback<()>,
    pub on_step: Callback<()>,
    pub on_reset: Callback<()>,
    pub on_toggle_watch: Callback<usize>,
    pub on_toggle_breakpoint: Callback<usize>,
    pub on_remove_watch: Callback<String>,
    pub on_load_example: Callback<usize>,
    pub on_load_file: Callback<web_sys::Event>,
}

#[function_component(DebuggerPanel)]
pub fn debugger_panel(props: &DebuggerProps) -> Html {
    let state = &props.state;
    let file_input_ref = use_node_ref();

    let on_load_select = {
        let cb_example = props.on_load_example.clone();
        let file_ref = file_input_ref.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let value = target.value();
            if value == "upload" {
                if let Some(input) = file_ref.cast::<web_sys::HtmlInputElement>() {
                    input.click();
                }
            } else if let Ok(idx) = value.parse::<usize>() {
                cb_example.emit(idx);
            }
            // Reset select back to placeholder
            target.set_value("");
        })
    };

    let on_file_change = {
        let cb = props.on_load_file.clone();
        Callback::from(move |e: Event| cb.emit(e))
    };

    let on_run = {
        let cb = props.on_run.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_step = {
        let cb = props.on_step.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_reset = {
        let cb = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let step_label = state.step_label();
    let run_disabled = state.active && state.current_step >= state.total_steps;
    let step_disabled = !state.active || state.current_step >= state.total_steps;
    let reset_disabled = !state.active || state.current_step == 0;

    html! {
        <div class="panel debugger-panel">
            <div class="panel-header">
                <h2>{"Debug"}</h2>
                <div class="debug-controls">
                    <select class="debug-load-select" onchange={on_load_select}
                        title="Load an example or upload a .pipe file">
                        <option value="" disabled=true selected=true>{"Load example..."}</option>
                        <optgroup label="Examples">
                            { for TUTORIALS.iter().enumerate().map(|(idx, t)| {
                                html! {
                                    <option value={idx.to_string()}>{t.name}</option>
                                }
                            })}
                        </optgroup>
                        <option value="upload">{"Upload .pipe file..."}</option>
                    </select>
                    <input type="file" accept=".pipe" ref={file_input_ref}
                        style="display:none" onchange={on_file_change} />
                    <button class="debug-btn debug-btn-run" onclick={on_run}
                        disabled={run_disabled}
                        title="Run pipeline">
                        {"Run"}
                    </button>
                    <button class="debug-btn debug-btn-step"
                        onclick={on_step}
                        disabled={step_disabled}
                        title="Step to next pipe point"
                    >
                        {"Step \u{25B6}"}
                    </button>
                    <button class="debug-btn debug-btn-reset"
                        onclick={on_reset}
                        disabled={reset_disabled}
                        title="Reset to step 0"
                    >
                        {"Reset"}
                    </button>
                    <span class="step-counter">{step_label}</span>
                </div>
            </div>
            <div class="panel-content debugger-content">
                { render_error(state) }
                { render_stage_list(state, &props.on_toggle_watch, &props.on_toggle_breakpoint) }
                { render_watch_list(state, &props.on_remove_watch) }
            </div>
        </div>
    }
}

fn render_error(state: &DebuggerState) -> Html {
    if let Some(err) = &state.error {
        html! { <div class="error">{err}</div> }
    } else {
        html! {}
    }
}

fn render_stage_list(
    state: &DebuggerState,
    on_toggle_watch: &Callback<usize>,
    on_toggle_breakpoint: &Callback<usize>,
) -> Html {
    if !state.active {
        return html! {
            <div class="debugger-placeholder">
                <p>{"Click "}<strong>{"Run"}</strong>{" to load the pipeline, then "}<strong>{"Step"}</strong>{" through records."}</p>
                <p class="hint">{"Each step advances a record one stage further."}</p>
            </div>
        };
    }

    let lines = &state.pipeline_lines;
    html! {
        <div class="stage-list">
            { for lines.iter().enumerate().map(|(i, line)| {
                let stage_idx = line.stage_index;
                html! {
                    <>
                        <div class={classes!("stage-line", stage_class(state, stage_idx))}>
                            <span class="stage-prefix">
                                { if stage_idx == 0 { "PIPE" } else { "|" } }
                            </span>
                            <span class="stage-text">{&line.text}</span>
                            <span class="stage-number">{format!("stage {stage_idx}")}</span>
                        </div>
                        { if i < lines.len() - 1 {
                            render_pipe_point(state, stage_idx, on_toggle_watch, on_toggle_breakpoint)
                        } else {
                            html! {}
                        }}
                    </>
                }
            })}
        </div>
    }
}

/// Stage is "completed" when the step has progressed past it.
fn stage_class(state: &DebuggerState, stage_idx: usize) -> &'static str {
    if state.current_step == 0 {
        return "stage-pending";
    }
    if !state.in_flush_phase {
        if stage_idx < state.visible_pp {
            "stage-completed"
        } else {
            "stage-pending"
        }
    } else {
        let trace = match &state.trace {
            Some(t) => t,
            None => return "stage-pending",
        };
        if let Some(ft) = trace.flush_traces.get(state.trace_idx) {
            let flush_start = ft.stage_index + 1;
            if stage_idx >= flush_start && (stage_idx - flush_start) < state.visible_pp {
                "stage-completed"
            } else {
                "stage-pending"
            }
        } else {
            "stage-pending"
        }
    }
}

fn render_pipe_point(
    state: &DebuggerState,
    stage_index: usize,
    on_toggle_watch: &Callback<usize>,
    on_toggle_breakpoint: &Callback<usize>,
) -> Html {
    let watches = state.watches_at(stage_index);
    let record_info = pipe_point_info(state, stage_index);
    let has_watch = !watches.is_empty();
    let has_bp = state.has_breakpoint(stage_index);
    let is_bp_hit = state.hit_breakpoint == Some(stage_index);

    let on_watch_click = {
        let cb = on_toggle_watch.clone();
        let idx = stage_index;
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            cb.emit(idx);
        })
    };

    let on_bp_click = {
        let cb = on_toggle_breakpoint.clone();
        let idx = stage_index;
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            cb.emit(idx);
        })
    };

    let has_data = state.current_step > 0 && !record_info.starts_with('\u{00B7}');
    let base_class = if has_data {
        "pipe-point pipe-reached"
    } else {
        "pipe-point pipe-pending"
    };
    let watch_class = if has_watch {
        "pipe-watch-icon active"
    } else {
        "pipe-watch-icon"
    };
    let bp_class = if has_bp {
        "pipe-bp-icon active"
    } else {
        "pipe-bp-icon"
    };

    html! {
        <div class={classes!(base_class, is_bp_hit.then_some("pipe-bp-hit"))}>
            <span class={watch_class} onclick={on_watch_click} title="Toggle watch">
                {"\u{24E6}"}
            </span>
            <span class={bp_class} onclick={on_bp_click} title="Toggle breakpoint">
                {"\u{24B7}"}
            </span>
            { for watches.iter().map(|w| {
                html! { <span class="watch-label">{&w.label}</span> }
            })}
            <span class="pipe-info">{record_info}</span>
        </div>
    }
}

/// Pipe point info: only shows data for revealed pipe points.
fn pipe_point_info(state: &DebuggerState, stage_index: usize) -> String {
    if state.current_step == 0 {
        return DOTS.to_string();
    }
    let trace = match &state.trace {
        Some(t) => t,
        None => return DOTS.to_string(),
    };
    if !state.in_flush_phase {
        if let Some(rt) = trace.record_traces.get(state.trace_idx) {
            if stage_index < state.visible_pp && stage_index < rt.pipe_points.len() {
                format_pipe_point_records(&rt.pipe_points[stage_index])
            } else {
                DOTS.to_string()
            }
        } else {
            DOTS.to_string()
        }
    } else if let Some(ft) = trace.flush_traces.get(state.trace_idx) {
        let flush_start = ft.stage_index + 1;
        if stage_index >= flush_start {
            let offset = stage_index - flush_start;
            if offset < state.visible_pp && offset < ft.pipe_points.len() {
                format_pipe_point_records(&ft.pipe_points[offset])
            } else {
                DOTS.to_string()
            }
        } else {
            DOTS.to_string()
        }
    } else {
        DOTS.to_string()
    }
}

fn format_pipe_point_records(records: &[pipelines_rs::Record]) -> String {
    if records.is_empty() {
        DOTS.to_string()
    } else if records.len() == 1 {
        let preview = records[0].as_str().trim_end();
        let truncated = if preview.len() > 30 {
            format!("{}...", &preview[..27])
        } else {
            preview.to_string()
        };
        format!("1 rec: {truncated}")
    } else {
        let preview = records[0].as_str().trim_end();
        let truncated = if preview.len() > 25 {
            format!("{}...", &preview[..22])
        } else {
            preview.to_string()
        };
        format!("{} recs: {truncated}", records.len())
    }
}

fn render_watch_list(state: &DebuggerState, on_remove_watch: &Callback<String>) -> Html {
    if !state.active {
        return html! {};
    }
    html! {
        <div class="watch-list">
            <h3 class="watch-list-header">{"Watches"}</h3>
            if state.watches.is_empty() {
                <p class="watch-hint">{"Click \u{24E6} to toggle a watch"}</p>
            } else {
                { for state.watches.iter().map(|watch| {
                    render_watch_item(state, watch, on_remove_watch)
                })}
            }
        </div>
    }
}

fn render_watch_item(
    state: &DebuggerState,
    watch: &Watch,
    on_remove_watch: &Callback<String>,
) -> Html {
    let stage_name = state
        .pipeline_lines
        .iter()
        .find(|l| l.stage_index == watch.stage_index)
        .map(|l| l.text.split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_default();

    let next_stage_name = state
        .pipeline_lines
        .iter()
        .find(|l| l.stage_index == watch.stage_index + 1)
        .map(|l| l.text.split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_else(|| "END".to_string());

    let description = format!("after {stage_name} \u{2192} {next_stage_name}");

    let on_delete = {
        let cb = on_remove_watch.clone();
        let label = watch.label.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            cb.emit(label.clone());
        })
    };

    html! {
        <div class="watch-item">
            <div class="watch-item-header">
                <span class="watch-item-label">{&watch.label}</span>
                <span class="watch-item-desc">{description}</span>
                <button class="watch-delete" onclick={on_delete} title="Remove watch">
                    {"\u{1F5D1}"}
                </button>
            </div>
            <div class="watch-records">
                { render_watch_records(state, watch.stage_index) }
            </div>
        </div>
    }
}

fn render_watch_records(state: &DebuggerState, stage_index: usize) -> Html {
    if state.current_step == 0 {
        return html! {
            <span class="watch-not-reached">{"step to see data"}</span>
        };
    }
    let trace = match &state.trace {
        Some(t) => t,
        None => {
            return html! {
                <span class="watch-empty">{"no data"}</span>
            };
        }
    };

    let records = if !state.in_flush_phase {
        trace.record_traces.get(state.trace_idx).and_then(|rt| {
            if stage_index < state.visible_pp {
                rt.pipe_points.get(stage_index)
            } else {
                None
            }
        })
    } else {
        trace.flush_traces.get(state.trace_idx).and_then(|ft| {
            let flush_start = ft.stage_index + 1;
            if stage_index >= flush_start {
                let offset = stage_index - flush_start;
                if offset < state.visible_pp {
                    ft.pipe_points.get(offset)
                } else {
                    None
                }
            } else {
                None
            }
        })
    };

    match records {
        Some(recs) if recs.is_empty() => html! {},
        Some(recs) => {
            let count = recs.len();
            html! {
                <>
                    { for recs.iter().take(20).map(|r| {
                        html! {
                            <div class="watch-record">{r.as_str().trim_end()}</div>
                        }
                    })}
                    if count > 20 {
                        <div class="watch-record-more">
                            {format!("... ({count} total)")}
                        </div>
                    }
                </>
            }
        }
        None => html! {},
    }
}
