//! Visual debugger component for stage-by-stage pipeline inspection.

use pipelines_rs::DebugInfo;
use yew::prelude::*;

use crate::dsl::PipelineLine;

/// A watch placed at a pipe point between stages.
#[derive(Clone, PartialEq)]
pub struct Watch {
    /// Display label: "w1", "w2", etc.
    pub label: String,
    /// Which pipe point (output of this stage index).
    pub stage_index: usize,
}

/// Debugger state (stored in AppState).
#[derive(Clone, PartialEq)]
pub struct DebuggerState {
    /// Whether the debugger has run at least once.
    pub active: bool,
    /// Debug info from pipeline execution.
    pub debug_info: Vec<DebugInfo>,
    /// Current step: 0 = before any stage, N = after stage N-1.
    pub current_step: usize,
    /// Ordered list of watches.
    pub watches: Vec<Watch>,
    /// Counter for generating watch labels.
    pub next_watch_id: usize,
    /// Total number of stages.
    pub stage_count: usize,
    /// Pipeline output text.
    pub output_text: String,
    /// Input record count.
    pub input_count: usize,
    /// Output record count.
    pub output_count: usize,
    /// Pipeline lines for display.
    pub pipeline_lines: Vec<PipelineLine>,
    /// Error from pipeline execution.
    pub error: Option<String>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self {
            active: false,
            debug_info: Vec::new(),
            current_step: 0,
            watches: Vec::new(),
            next_watch_id: 1,
            stage_count: 0,
            output_text: String::new(),
            input_count: 0,
            output_count: 0,
            pipeline_lines: Vec::new(),
            error: None,
        }
    }
}

impl DebuggerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a watch at the given pipe point.
    pub fn add_watch(&mut self, stage_index: usize) {
        let label = format!("w{}", self.next_watch_id);
        self.next_watch_id += 1;
        self.watches.push(Watch { label, stage_index });
    }

    /// Remove a watch by label.
    pub fn remove_watch(&mut self, label: &str) {
        self.watches.retain(|w| w.label != label);
    }

    /// Get watches at a specific pipe point.
    pub fn watches_at(&self, stage_index: usize) -> Vec<&Watch> {
        self.watches
            .iter()
            .filter(|w| w.stage_index == stage_index)
            .collect()
    }
}

#[derive(Properties, PartialEq)]
pub struct DebuggerProps {
    pub state: DebuggerState,
    pub on_run: Callback<()>,
    pub on_step: Callback<()>,
    pub on_run_all: Callback<()>,
    pub on_reset: Callback<()>,
    pub on_add_watch: Callback<usize>,
    pub on_remove_watch: Callback<String>,
}

/// Visual debugger panel component.
#[function_component(DebuggerPanel)]
pub fn debugger_panel(props: &DebuggerProps) -> Html {
    let state = &props.state;

    let on_run = {
        let cb = props.on_run.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_step = {
        let cb = props.on_step.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_run_all = {
        let cb = props.on_run_all.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_reset = {
        let cb = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let step_label = if state.active {
        format!("{} / {}", state.current_step, state.stage_count)
    } else {
        String::new()
    };

    html! {
        <div class="panel debugger-panel">
            <div class="panel-header">
                <h2>{"Visual Debugger"}</h2>
                <div class="debug-controls">
                    <button class="debug-btn" onclick={on_run} title="Run pipeline">
                        {"Run"}
                    </button>
                    <button class="debug-btn"
                        onclick={on_step}
                        disabled={!state.active || state.current_step >= state.stage_count}
                        title="Step forward one stage"
                    >
                        {"Step \u{25B6}"}
                    </button>
                    <button class="debug-btn"
                        onclick={on_run_all}
                        disabled={!state.active || state.current_step >= state.stage_count}
                        title="Run all remaining stages"
                    >
                        {"Run All"}
                    </button>
                    <button class="debug-btn"
                        onclick={on_reset}
                        disabled={!state.active}
                        title="Reset to step 0"
                    >
                        {"Reset"}
                    </button>
                    if !step_label.is_empty() {
                        <span class="step-counter">{step_label}</span>
                    }
                </div>
            </div>
            <div class="panel-content debugger-content">
                { render_error(state) }
                { render_stage_list(state, &props.on_add_watch) }
                { render_watch_list(state, &props.on_remove_watch) }
            </div>
        </div>
    }
}

fn render_error(state: &DebuggerState) -> Html {
    if let Some(err) = &state.error {
        html! {
            <div class="error">{err}</div>
        }
    } else {
        html! {}
    }
}

fn render_stage_list(state: &DebuggerState, on_add_watch: &Callback<usize>) -> Html {
    if !state.active {
        return html! {
            <div class="debugger-placeholder">
                <p>{"Click "}<strong>{"Run"}</strong>{" to load the pipeline, then "}<strong>{"Step"}</strong>{" through stages."}</p>
                <p class="hint">{"Add watches by clicking pipe points between stages."}</p>
            </div>
        };
    }

    let lines = &state.pipeline_lines;

    html! {
        <div class="stage-list">
            { for lines.iter().enumerate().map(|(i, line)| {
                let stage_idx = line.stage_index;
                let status = stage_status(state.current_step, stage_idx, state.stage_count);

                html! {
                    <>
                        // Stage row
                        <div class={classes!("stage-line", status_class(status))}>
                            <span class="stage-prefix">
                                { if stage_idx == 0 { "PIPE" } else { "|" } }
                            </span>
                            <span class="stage-text">{&line.text}</span>
                            <span class="stage-number">{format!("stage {stage_idx}")}</span>
                        </div>
                        // Pipe point between stages (not after last)
                        { if i < lines.len() - 1 {
                            render_pipe_point(state, stage_idx, on_add_watch)
                        } else {
                            html! {}
                        }}
                    </>
                }
            })}
        </div>
    }
}

/// Stage execution status.
#[derive(Clone, Copy, PartialEq)]
enum StageStatus {
    Completed,
    Current,
    Pending,
}

fn stage_status(current_step: usize, stage_index: usize, _stage_count: usize) -> StageStatus {
    if current_step > stage_index {
        StageStatus::Completed
    } else if current_step == stage_index {
        StageStatus::Current
    } else {
        StageStatus::Pending
    }
}

fn status_class(status: StageStatus) -> &'static str {
    match status {
        StageStatus::Completed => "stage-completed",
        StageStatus::Current => "stage-current",
        StageStatus::Pending => "stage-pending",
    }
}

fn render_pipe_point(
    state: &DebuggerState,
    stage_index: usize,
    on_add_watch: &Callback<usize>,
) -> Html {
    let reached = state.current_step > stage_index;
    let watches = state.watches_at(stage_index);

    let record_info = if reached {
        if let Some(info) = state.debug_info.get(stage_index) {
            format!("{} records", info.output_count)
        } else {
            String::new()
        }
    } else {
        "\u{00B7}\u{00B7}\u{00B7}".to_string()
    };

    let on_click = {
        let cb = on_add_watch.clone();
        let idx = stage_index;
        Callback::from(move |_: MouseEvent| cb.emit(idx))
    };

    let pipe_class = if reached {
        "pipe-point pipe-reached"
    } else {
        "pipe-point pipe-pending"
    };

    // \u{24E6} = ⓦ (circled w)
    html! {
        <div class={pipe_class} onclick={on_click} title="Click to add watch">
            <span class="pipe-watch-icon">{"\u{24E6}"}</span>
            { for watches.iter().map(|w| {
                html! {
                    <span class="watch-label">{&w.label}</span>
                }
            })}
            <span class="pipe-info">{record_info}</span>
        </div>
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
                <p class="watch-hint">{"Click a pipe point to add a watch"}</p>
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
    let reached = state.current_step > watch.stage_index;

    let stage_name = state
        .debug_info
        .get(watch.stage_index)
        .map(|d| d.stage_name.clone())
        .unwrap_or_default();

    let next_stage_name = state
        .pipeline_lines
        .iter()
        .find(|l| l.stage_index == watch.stage_index + 1)
        .map(|l| l.text.split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_else(|| "END".to_string());

    let description = format!("after {} \u{2192} {}", stage_name, next_stage_name);

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
                { if reached {
                    render_watch_records(state, watch.stage_index)
                } else {
                    html! {
                        <span class="watch-not-reached">{"not yet reached"}</span>
                    }
                }}
            </div>
        </div>
    }
}

fn render_watch_records(state: &DebuggerState, stage_index: usize) -> Html {
    if let Some(info) = state.debug_info.get(stage_index) {
        if let Some(records) = &info.output_records {
            if records.is_empty() {
                return html! {
                    <span class="watch-empty">{"(0 records)"}</span>
                };
            }
            let count = records.len();
            html! {
                <>
                    { for records.iter().take(20).map(|r| {
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
        } else {
            html! {
                <span class="watch-empty">{format!("{} records (details not captured)", info.output_count)}</span>
            }
        }
    } else {
        html! {
            <span class="watch-empty">{"no data"}</span>
        }
    }
}
