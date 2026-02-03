//! Visual debugger component for stage-by-stage pipeline inspection.
//!
//! Layout:
//! - Top: Debug controls (reset, play/pause, step forward)
//! - Left: Stage list with click-to-toggle breakpoints
//! - Right: Inspectors (next input, watches, next output)

use yew::prelude::*;

use crate::dsl::{DebugInfo, DebugCallbacks};

/// Debugger state.
#[derive(Clone, PartialEq, Default)]
pub struct DebuggerState {
    pub current_stage: usize,
    pub paused: bool,
    pub breakpoints: HashSet<usize>,
    pub debug_info: Vec<DebugInfo>,
    pub expand_all: bool,
}

impl DebuggerState {
    pub fn new() -> Self {
        Self {
            current_stage: 0,
            paused: true,
            breakpoints: HashSet::new(),
            debug_info: Vec::new(),
            expand_all: false,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DebuggerProps {
    pub input_text: String,
    pub pipeline_text: String,
    pub on_run: Callback<()>,
}

/// Visual debugger panel component.
#[function_component(DebuggerPanel)]
pub fn debugger_panel(props: &DebuggerProps) -> Html {
    let state = use_state(DebuggerState::new);
    
    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.current_stage = 0;
            new_state.paused = true;
            new_state.debug_info.clear();
            state.set(new_state);
        })
    };
    
    let on_step_forward = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            if new_state.current_stage < new_state.debug_info.len() {
                new_state.current_stage += 1;
                new_state.paused = true;
                state.set(new_state);
            }
        })
    };
    
    let on_toggle_breakpoint = {
        let state = state.clone();
        Callback::from(move |stage_idx: usize| {
            let mut new_state = (*state).clone();
            if new_state.breakpoints.contains(&stage_idx) {
                new_state.breakpoints.remove(&stage_idx);
            } else {
                new_state.breakpoints.insert(stage_idx);
            }
            state.set(new_state);
        })
    };
    
    html! {
        <div class="debugger-layout">
            {if state.debug_info.is_empty() {
                html! {
                    <div class="debugger-empty">
                        <h2>{"No debug info available"}</h2>
                        <p>{"Run the pipeline to see stage-by-stage execution."}</p>
                        <button
                            class="debug-btn"
                            onclick={props.on_run.clone()}
                        >
                            {"▶ Run Pipeline"}
                        </button>
                    </div>
                }
            } else {
                html! {
                    <div class="debug-controls">
                        <button class="debug-btn" onclick={on_reset}>
                            {"⏮ Reset"}
                        </button>
                        <button class="debug-btn" onclick={Callback::from(|_| {})}>
                            {if state.paused { "▶ Play" } else { "⏸ Pause" }}
                        </button>
                        <button class="debug-btn" onclick={on_step_forward}>
                            {"⏭ Step Forward"}
                        </button>
                        <span class="current-stage-display">
                            {"Stage: "}{state.current_stage + 1} / {state.debug_info.len()}
                        </span>
                    </div>
                    
                    <div class="debugger-content">
                        <div class="stage-list">
                            <h3>{"Stages (Click to toggle breakpoint)"}</h3>
                            {state.debug_info.iter().enumerate().map(|(idx, info)| {
                                let is_current = idx == state.current_stage;
                                let is_breakpoint = state.breakpoints.contains(&idx);
                                
                                html! {
                                    <div class={classes![
                                        "stage-item",
                                        if is_current { "current" },
                                        if is_breakpoint { "breakpoint" },
                                    ]}>
                                        <input
                                            type="checkbox"
                                            checked={is_breakpoint}
                                            onclick={on_toggle_breakpoint.reform(move |()| idx)}
                                        />
                                        <span class="stage-name">
                                            {info.stage_name.clone()}
                                        </span>
                                        <span class="record-count">
                                            {"→ "}{info.output_count} recs"
                                        </span>
                                    </div>
                                }
                            }).collect::<Html>()}
                        </div>
                        
                        <div class="inspectors">
                            <div class="inspector-box">
                                <h3>{"Next Input"}</h3>
                                {if let Some(ref info) = state.debug_info.get(state.current_stage) {
                                    html! {
                                        <div class="inspector-content compact">
                                            {info.input_records.as_ref().and_then(|recs| recs.first()).map(|rec| {
                                                html! {<pre>{rec.as_str()}</pre>}
                                            })}
                                        </div>
                                    }
                                } else {
                                    html! {<span class="no-data">{"No input yet"}</span>}
                                }}
                            </div>
                            
                            <div class="inspector-box">
                                <h3>{"Stage Watches"}</h3>
                                <div class="watches-list">
                                    <label>
                                        <input type="checkbox" checked={true} />
                                        {"Watch: Stage inputs"}
                                    </label>
                                    <label>
                                        <input type="checkbox" checked={true} />
                                        {"Watch: Stage outputs"}
                                    </label>
                                </div>
                            </div>
                            
                            <div class="inspector-box">
                                <h3>{"Next Output Line"}</h3>
                                {if let Some(ref info) = state.debug_info.get(state.current_stage) {
                                    html! {
                                        <div class="inspector-content compact">
                                            {info.output_records.as_ref().and_then(|recs| recs.first()).map(|rec| {
                                                html! {<pre>{rec.as_str()}</pre>}
                                            })}
                                        </div>
                                    }
                                } else {
                                    html! {<span class="no-data">{"No output yet"}</span>}
                                }}
                            </div>
                        </div>
                    </div>
                    
                    <button class="expand-all-btn">
                        Expand All Input/Output
                    </button>
                }
            }}
        </div>
    }
}
