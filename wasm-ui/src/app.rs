//! Main application component.

use gloo::timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, HtmlAnchorElement, HtmlInputElement, HtmlSelectElement, Url};
use yew::prelude::*;

use crate::components::{InputPanel, OutputPanel, PipelinePanel};
use crate::dsl::execute_pipeline;

/// Render CSS-animated countdown with cycling dots.
fn countdown_html(countdown: u32, prefix: &str, suffix: &str) -> Html {
    html! {
        <span class="countdown-anim">
            <span class="frame f0">{ format!("{}...{}{}", prefix, countdown, suffix) }</span>
            <span class="frame f1">{ format!("{}..{}.{}", prefix, countdown, suffix) }</span>
            <span class="frame f2">{ format!("{}.{}..{}", prefix, countdown, suffix) }</span>
            <span class="frame f3">{ format!("{}{}...{}", prefix, countdown, suffix) }</span>
        </span>
    }
}

/// Tutorial phase tracking.
#[derive(Clone, PartialEq, Debug)]
pub enum TutorialPhase {
    /// No tutorial active.
    None,
    /// Showing description dialog.
    ShowingDialog,
    /// Showing tooltip over Run button.
    ShowingRunTooltip,
    /// Pipeline was run, showing Next/Cancel in output.
    ShowingOutputButtons,
}

/// Tutorial content for each command.
#[derive(Clone, PartialEq)]
pub struct TutorialStep {
    pub name: &'static str,
    pub description: &'static str,
    pub example_pipeline: &'static str,
}

const TUTORIALS: &[TutorialStep] = &[
    TutorialStep {
        name: "PIPE/CONSOLE",
        description: "The simplest pipeline - like Unix 'cat'!\n\n\
            PIPE CONSOLE reads from the Input Records panel.\n\
            | CONSOLE writes to the Output Records panel.\n\
            ? marks the end of the pipeline.\n\n\
            This passes all records through unchanged.\n\
            All pipelines follow this basic structure.",
        example_pipeline: "# Simplest pipeline (like Unix cat)\n# Pass all records through unchanged\nPIPE CONSOLE\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "FILTER",
        description: "FILTER selects records based on field values.\n\n\
            Syntax: FILTER pos,len = \"value\" (keep matches)\n\
            Syntax: FILTER pos,len != \"value\" (keep non-matches)\n\n\
            pos: starting column (0-based)\n\
            len: number of characters to compare\n\
            value: text to match\n\n\
            Records are 80-byte fixed-width, so positions matter!",
        example_pipeline: "# Filter: keep only SALES department\nPIPE CONSOLE\n| FILTER 18,10 = \"SALES\"\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "SELECT",
        description: "SELECT extracts and rearranges fields from records.\n\n\
            Syntax: SELECT src,len,dest; src,len,dest; ...\n\n\
            src: source position (0-based)\n\
            len: number of characters to copy\n\
            dest: destination position in output\n\n\
            Output is padded to 80 bytes. Use multiple field specs separated by semicolons.",
        example_pipeline: "# Select: extract name and salary fields\nPIPE CONSOLE\n| SELECT 0,8,0; 28,8,10\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "TAKE",
        description: "TAKE passes only the first N records.\n\n\
            Syntax: TAKE n\n\n\
            Useful for:\n\
            - Limiting output size\n\
            - Getting a sample of data\n\
            - Pagination (with SKIP)",
        example_pipeline: "# Take: get first 3 records only\nPIPE CONSOLE\n| TAKE 3\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "SKIP",
        description: "SKIP discards the first N records.\n\n\
            Syntax: SKIP n\n\n\
            Combine with TAKE for pagination:\n\
            - SKIP 10 | TAKE 10 = records 11-20\n\
            - SKIP 20 | TAKE 10 = records 21-30",
        example_pipeline: "# Skip: discard first 2 records\nPIPE CONSOLE\n| SKIP 2\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "LOCATE",
        description: "LOCATE finds records containing a pattern.\n\n\
            Syntax: LOCATE \"pattern\"\n\
            Syntax: LOCATE \"pattern\" pos,len (search specific field)\n\n\
            Case-sensitive substring search.\n\
            Only matching records pass through.",
        example_pipeline: "# Locate: find records containing ENGINEER\nPIPE CONSOLE\n| LOCATE \"ENGINEER\"\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "NLOCATE",
        description: "NLOCATE is the opposite of LOCATE.\n\n\
            Syntax: NLOCATE \"pattern\"\n\
            Syntax: NLOCATE \"pattern\" pos,len\n\n\
            Records that do NOT contain the pattern pass through.\n\
            Useful for filtering out unwanted records.",
        example_pipeline: "# Nlocate: exclude SALES records\nPIPE CONSOLE\n| NLOCATE \"SALES\"\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "COUNT",
        description: "COUNT outputs a single record with the count of input records.\n\n\
            Syntax: COUNT\n\n\
            The count is right-justified in an 80-byte record.\n\
            Useful after FILTER or LOCATE to count matches.",
        example_pipeline: "# Count: how many ENGINEER records?\nPIPE CONSOLE\n| LOCATE \"ENGINEER\"\n| COUNT\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "CHANGE",
        description: "CHANGE replaces text in records.\n\n\
            Syntax: CHANGE \"old\" \"new\"\n\n\
            Replaces ALL occurrences in each record.\n\
            Output is padded/truncated to 80 bytes.\n\
            Useful for data normalization.",
        example_pipeline: "# Change: rename SALES to REVENUE\nPIPE CONSOLE\n| CHANGE \"SALES\" \"REVENUE\"\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "LITERAL",
        description: "LITERAL inserts a fixed record into the stream.\n\n\
            Syntax: LITERAL \"text\"\n\n\
            The literal is inserted BEFORE each input record.\n\
            Text is padded to 80 bytes.\n\
            Useful for adding headers or separators.",
        example_pipeline: "# Literal: add a header line\nPIPE CONSOLE\n| TAKE 1\n| LITERAL \"=== EMPLOYEE RECORD ===\"\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "UPPER",
        description: "UPPER converts records to uppercase.\n\n\
            Syntax: UPPER\n\n\
            Converts all alphabetic characters to uppercase.\n\
            Non-alphabetic characters unchanged.",
        example_pipeline: "# Upper: convert to uppercase\nPIPE CONSOLE\n| TAKE 2\n| UPPER\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "LOWER",
        description: "LOWER converts records to lowercase.\n\n\
            Syntax: LOWER\n\n\
            Converts all alphabetic characters to lowercase.\n\
            Non-alphabetic characters unchanged.",
        example_pipeline: "# Lower: convert to lowercase\nPIPE CONSOLE\n| TAKE 2\n| LOWER\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "REVERSE",
        description: "REVERSE reverses the characters in each record.\n\n\
            Syntax: REVERSE\n\n\
            Reverses the entire 80-byte record.\n\
            Trailing spaces become leading spaces.",
        example_pipeline: "# Reverse: mirror the text\nPIPE CONSOLE\n| TAKE 2\n| REVERSE\n| CONSOLE\n?",
    },
    TutorialStep {
        name: "DUPLICATE",
        description: "DUPLICATE outputs each record multiple times.\n\n\
            Syntax: DUPLICATE n\n\n\
            Each input record is output n times.\n\
            Useful for testing or data generation.",
        example_pipeline: "# Duplicate: triple each record\nPIPE CONSOLE\n| TAKE 2\n| DUPLICATE 3\n| CONSOLE\n?",
    },
];

/// Main application state.
#[derive(Clone, PartialEq)]
pub struct AppState {
    /// Input records (one per line).
    pub input_text: String,
    /// Pipeline DSL commands.
    pub pipeline_text: String,
    /// Output records after processing.
    pub output_text: String,
    /// Error message, if any.
    pub error: Option<String>,
    /// Record count stats.
    pub stats: String,
    /// Current tutorial step index (None = no tutorial).
    pub tutorial_step: Option<usize>,
    /// Current tutorial phase.
    pub tutorial_phase: TutorialPhase,
    /// Auto-run mode (automatically advances through tutorials).
    pub auto_mode: bool,
    /// Tutorial delay in seconds (1-15).
    pub tutorial_delay: u32,
    /// Countdown seconds remaining (for auto mode display).
    pub countdown: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_text: DEFAULT_INPUT.to_string(),
            pipeline_text: DEFAULT_PIPELINE.to_string(),
            output_text: String::new(),
            error: None,
            stats: String::new(),
            tutorial_step: None,
            tutorial_phase: TutorialPhase::None,
            auto_mode: false,
            tutorial_delay: 5,
            countdown: 0,
        }
    }
}

const DEFAULT_INPUT: &str = r#"SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
DOE     JANE      SALES     00060000
WILSON  ROBERT    MARKETING 00055000
CHEN    LISA      ENGINEER  00080000
GARCIA  CARLOS    SALES     00045000
TAYLOR  SUSAN     MARKETING 00065000
BROWN   MICHAEL   ENGINEER  00090000"#;

const DEFAULT_PIPELINE: &str = r#"PIPE CONSOLE
| FILTER 18,10 = "SALES"
| SELECT 0,8,0; 28,8,8
| CONSOLE
?"#;

/// Main application component.
#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(AppState::default);

    let on_input_change = {
        let state = state.clone();
        Callback::from(move |text: String| {
            let mut new_state = (*state).clone();
            new_state.input_text = text;
            state.set(new_state);
        })
    };

    let on_pipeline_change = {
        let state = state.clone();
        Callback::from(move |text: String| {
            let mut new_state = (*state).clone();
            new_state.pipeline_text = text;
            state.set(new_state);
        })
    };

    let on_run = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();

            match execute_pipeline(&new_state.input_text, &new_state.pipeline_text) {
                Ok((output, input_count, output_count)) => {
                    new_state.output_text = output;
                    new_state.error = None;
                    new_state.stats = format!(
                        "Input: {} records | Output: {} records",
                        input_count, output_count
                    );
                }
                Err(e) => {
                    new_state.output_text.clear();
                    new_state.error = Some(e);
                    new_state.stats.clear();
                }
            }

            // If in tutorial mode showing Run tooltip, advance to output buttons phase
            if new_state.tutorial_phase == TutorialPhase::ShowingRunTooltip {
                new_state.tutorial_phase = TutorialPhase::ShowingOutputButtons;
                if new_state.auto_mode {
                    new_state.countdown = new_state.tutorial_delay;
                }
            }

            state.set(new_state);
        })
    };

    let on_load = {
        let state = state.clone();
        Callback::from(move |e: web_sys::Event| {
            let state = state.clone();
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();

                    let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(text) = result.as_string() {
                                let mut new_state = (*state).clone();
                                new_state.pipeline_text = text;
                                state.set(new_state);
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();

                    let _ = reader.read_as_text(&file);
                }
            }
            // Clear the input so the same file can be loaded again
            input.set_value("");
        })
    };

    let on_save = {
        let state = state.clone();
        Callback::from(move |_| {
            let text = state.pipeline_text.clone();
            let array = js_sys::Array::new();
            array.push(&JsValue::from_str(&text));

            let blob = Blob::new_with_str_sequence(&array).unwrap();
            let url = Url::create_object_url_with_blob(&blob).unwrap();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let anchor: HtmlAnchorElement =
                document.create_element("a").unwrap().dyn_into().unwrap();

            anchor.set_href(&url);
            anchor.set_download("pipeline.pipe");
            anchor.click();

            let _ = Url::revoke_object_url(&url);
        })
    };

    // Tutorial dropdown change handler
    let on_tutorial_select = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let value = target.value();
            let mut new_state = (*state).clone();

            if value.is_empty() {
                // "Select Tutorial" placeholder
                new_state.tutorial_step = None;
                new_state.tutorial_phase = TutorialPhase::None;
                new_state.auto_mode = false;
            } else if value == "auto" {
                // Auto-run all tutorials
                new_state.tutorial_step = Some(0);
                new_state.tutorial_phase = TutorialPhase::ShowingDialog;
                new_state.auto_mode = true;
                new_state.countdown = new_state.tutorial_delay;
            } else if let Ok(idx) = value.parse::<usize>() {
                new_state.tutorial_step = Some(idx);
                new_state.tutorial_phase = TutorialPhase::ShowingDialog;
                new_state.auto_mode = false;
            }

            state.set(new_state);
        })
    };

    // Tutorial speed slider change handler
    let on_speed_change = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            if let Ok(delay) = target.value().parse::<u32>() {
                let mut new_state = (*state).clone();
                new_state.tutorial_delay = delay;
                state.set(new_state);
            }
        })
    };

    // Dialog Next button - load example and show Run tooltip
    let on_dialog_next = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*state).clone();
            if let Some(idx) = new_state.tutorial_step {
                if let Some(tutorial) = TUTORIALS.get(idx) {
                    new_state.pipeline_text = tutorial.example_pipeline.to_string();
                }
            }
            new_state.tutorial_phase = TutorialPhase::ShowingRunTooltip;
            if new_state.auto_mode {
                new_state.countdown = new_state.tutorial_delay;
            }
            state.set(new_state);
        })
    };

    // Dialog/tooltip Cancel or dismiss (MouseEvent version for onclick)
    let on_tutorial_cancel_click = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*state).clone();
            new_state.tutorial_step = None;
            new_state.tutorial_phase = TutorialPhase::None;
            new_state.auto_mode = false;
            state.set(new_state);
        })
    };

    // Dialog/tooltip Cancel or dismiss (unit version for component props)
    let on_tutorial_cancel = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            let mut new_state = (*state).clone();
            new_state.tutorial_step = None;
            new_state.tutorial_phase = TutorialPhase::None;
            new_state.auto_mode = false;
            state.set(new_state);
        })
    };

    // Output panel Next Tutorial button
    let on_next_tutorial = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            let mut new_state = (*state).clone();
            // Clear output when advancing to next tutorial
            new_state.output_text.clear();
            new_state.error = None;
            new_state.stats.clear();
            if let Some(idx) = new_state.tutorial_step {
                let next_idx = idx + 1;
                if next_idx < TUTORIALS.len() {
                    new_state.tutorial_step = Some(next_idx);
                    new_state.tutorial_phase = TutorialPhase::ShowingDialog;
                    if new_state.auto_mode {
                        new_state.countdown = new_state.tutorial_delay;
                    }
                } else {
                    // End of tutorials
                    new_state.tutorial_step = None;
                    new_state.tutorial_phase = TutorialPhase::None;
                    new_state.auto_mode = false;
                }
            }
            state.set(new_state);
        })
    };

    // Clear output button
    let on_clear = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            let mut new_state = (*state).clone();
            new_state.output_text.clear();
            new_state.error = None;
            new_state.stats.clear();
            state.set(new_state);
        })
    };

    // Handle clicking on overlay to dismiss dialog/tooltip
    let on_overlay_click = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*state).clone();
            new_state.tutorial_step = None;
            new_state.tutorial_phase = TutorialPhase::None;
            new_state.auto_mode = false;
            state.set(new_state);
        })
    };

    // Auto-mode timer: uses Timeout that re-schedules via state changes
    // Each time countdown changes, a new timeout is scheduled
    {
        let state = state.clone();
        let on_run = on_run.clone();
        let auto_mode = state.auto_mode;
        let phase = state.tutorial_phase.clone();
        let countdown = state.countdown;
        let tutorial_step = state.tutorial_step;
        let tutorial_delay = state.tutorial_delay;

        use_effect_with(
            (auto_mode, phase.clone(), countdown),
            move |(auto_mode, phase, countdown)| {
                let timeout_handle: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

                if *auto_mode && *phase != TutorialPhase::None {
                    let state = state.clone();
                    let on_run = on_run.clone();
                    let current_countdown = *countdown;
                    let current_phase = phase.clone();

                    let handle = Timeout::new(1000, move || {
                        let mut new_state = (*state).clone();

                        if current_countdown > 1 {
                            // Still counting down - decrement
                            new_state.countdown = current_countdown - 1;
                            state.set(new_state);
                        } else {
                            // Countdown reached 1 -> 0, trigger phase transition
                            match current_phase {
                                TutorialPhase::ShowingDialog => {
                                    // Load example and show Run tooltip
                                    if let Some(idx) = tutorial_step {
                                        if let Some(tutorial) = TUTORIALS.get(idx) {
                                            new_state.pipeline_text =
                                                tutorial.example_pipeline.to_string();
                                        }
                                    }
                                    new_state.tutorial_phase = TutorialPhase::ShowingRunTooltip;
                                    new_state.countdown = tutorial_delay;
                                    state.set(new_state);
                                }
                                TutorialPhase::ShowingRunTooltip => {
                                    // Run the pipeline (callback will set phase and countdown)
                                    on_run.emit(());
                                }
                                TutorialPhase::ShowingOutputButtons => {
                                    // Clear output and advance to next tutorial
                                    new_state.output_text.clear();
                                    new_state.error = None;
                                    new_state.stats.clear();
                                    if let Some(idx) = tutorial_step {
                                        let next_idx = idx + 1;
                                        if next_idx < TUTORIALS.len() {
                                            new_state.tutorial_step = Some(next_idx);
                                            new_state.tutorial_phase = TutorialPhase::ShowingDialog;
                                            new_state.countdown = tutorial_delay;
                                        } else {
                                            // End of tutorials
                                            new_state.tutorial_step = None;
                                            new_state.tutorial_phase = TutorialPhase::None;
                                            new_state.auto_mode = false;
                                        }
                                    }
                                    state.set(new_state);
                                }
                                TutorialPhase::None => {}
                            }
                        }
                    });
                    *timeout_handle.borrow_mut() = Some(handle);
                }

                let cleanup_handle = timeout_handle.clone();
                move || {
                    if let Some(handle) = cleanup_handle.borrow_mut().take() {
                        handle.cancel();
                    }
                }
            },
        );
    }

    // Get current tutorial info for rendering
    let current_tutorial = state.tutorial_step.and_then(|idx| TUTORIALS.get(idx));
    let next_tutorial_name = state
        .tutorial_step
        .and_then(|idx| TUTORIALS.get(idx + 1))
        .map(|t| t.name);

    html! {
        <div class="app">
            <header class="header">
                <div class="header-left">
                    <h1>{ "pipelines-rs" }</h1>
                    <p class="subtitle">{ "Mainframe-Style 80-Byte Record Processing" }</p>
                </div>
                <div class="header-right">
                    if state.tutorial_step.is_some() {
                        <div class="speed-control">
                            <label class="speed-label">{ "Delay:" }</label>
                            <input
                                type="range"
                                class="speed-slider"
                                min="1"
                                max="15"
                                value={state.tutorial_delay.to_string()}
                                oninput={on_speed_change}
                            />
                            <span class="speed-value">{ format!("{}s", state.tutorial_delay) }</span>
                        </div>
                    }
                    <select class="tutorial-select" onchange={on_tutorial_select}>
                        <option value="" selected={state.tutorial_step.is_none() && !state.auto_mode}>{ "Tutorial" }</option>
                        <option value="auto" selected={state.auto_mode}>{ "Run All (auto)" }</option>
                        { for TUTORIALS.iter().enumerate().map(|(idx, t)| {
                            html! {
                                <option value={idx.to_string()} selected={state.tutorial_step == Some(idx) && !state.auto_mode}>
                                    { t.name }
                                </option>
                            }
                        })}
                    </select>
                </div>
            </header>

            <main class="main">
                <div class="panels">
                    <InputPanel
                        value={state.input_text.clone()}
                        on_change={on_input_change}
                    />

                    <PipelinePanel
                        value={state.pipeline_text.clone()}
                        on_change={on_pipeline_change}
                        on_run={on_run.clone()}
                        on_load={on_load}
                        on_save={on_save}
                        show_run_tooltip={state.tutorial_phase == TutorialPhase::ShowingRunTooltip}
                        on_tooltip_dismiss={on_tutorial_cancel.clone()}
                        auto_mode={state.auto_mode}
                        countdown={state.countdown}
                    />

                    <OutputPanel
                        value={state.output_text.clone()}
                        error={state.error.clone()}
                        stats={state.stats.clone()}
                        show_tutorial_buttons={state.tutorial_phase == TutorialPhase::ShowingOutputButtons}
                        next_tutorial_name={next_tutorial_name.map(|s| s.to_string())}
                        on_next_tutorial={on_next_tutorial}
                        on_cancel_tutorial={on_tutorial_cancel.clone()}
                        auto_mode={state.auto_mode}
                        countdown={state.countdown}
                        on_clear={on_clear}
                    />
                </div>
            </main>

            // Tutorial dialog overlay
            if state.tutorial_phase == TutorialPhase::ShowingDialog {
                if let Some(tutorial) = current_tutorial {
                    <div class="modal-overlay" onclick={on_overlay_click.clone()}>
                        <div class="modal-dialog" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                            <h3 class="modal-title">{ format!("{} Tutorial", tutorial.name) }</h3>
                            <div class="modal-content">
                                <pre class="modal-description">{ tutorial.description }</pre>
                            </div>
                            <div class="modal-buttons">
                                <button class="modal-button cancel" onclick={on_tutorial_cancel_click.clone()}>
                                    { "Cancel" }
                                </button>
                                <div class="next-button-container">
                                    <button class="modal-button next" onclick={on_dialog_next}>
                                        { "Next" }
                                    </button>
                                    if state.auto_mode {
                                        <div class="modal-next-tooltip">
                                            { countdown_html(state.countdown, "Auto-next in ", "") }
                                        </div>
                                    }
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }

            <footer class="footer">
                <div class="footer-row">
                    <span>{ "80-byte fixed-width records | ASCII | Punch card format" }</span>
                </div>
                <div class="footer-row">
                    <span class="footer-left">
                        <a href="https://github.com/softwarewrighter/pipelines-rs" target="_blank">{ "GitHub" }</a>
                        { " | MIT License | " }
                        { "\u{00A9} 2026 Michael A Wright" }
                    </span>
                    <span class="footer-build">
                        { format!("Build: {}@{} {}", env!("BUILD_HOST"), env!("BUILD_COMMIT"), env!("BUILD_TIMESTAMP")) }
                    </span>
                </div>
            </footer>
        </div>
    }
}
