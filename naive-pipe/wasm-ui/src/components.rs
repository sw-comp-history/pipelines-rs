//! UI Components for the pipeline demo.

use yew::prelude::*;

/// Input panel for entering records.
#[derive(Properties, PartialEq)]
pub struct InputPanelProps {
    pub value: String,
    pub on_change: Callback<String>,
}

#[function_component(InputPanel)]
pub fn input_panel(props: &InputPanelProps) -> Html {
    let on_input = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let target: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            on_change.emit(target.value());
        })
    };

    html! {
        <div class="panel input-panel">
            <div class="panel-header">
                <h2>{ "Input Records" }</h2>
                <span class="hint">{ "One 80-byte record per line" }</span>
            </div>
            <div class="panel-content">
                <div class="column-ruler">
                    { "0---------1---------2---------3---------4---------5---------6---------7---------" }
                </div>
                <textarea
                    class="record-input"
                    value={props.value.clone()}
                    oninput={on_input}
                    spellcheck="false"
                    wrap="off"
                    rows="12"
                />
            </div>
        </div>
    }
}

/// Pipeline DSL panel.
#[derive(Properties, PartialEq)]
pub struct PipelinePanelProps {
    pub value: String,
    pub on_change: Callback<String>,
    pub on_run: Callback<()>,
    pub on_load: Callback<web_sys::Event>,
    pub on_save: Callback<()>,
    #[prop_or(false)]
    pub show_run_tooltip: bool,
    #[prop_or_default]
    pub on_tooltip_dismiss: Callback<()>,
    #[prop_or(false)]
    pub auto_mode: bool,
    #[prop_or(0)]
    pub countdown: u32,
}

#[function_component(PipelinePanel)]
pub fn pipeline_panel(props: &PipelinePanelProps) -> Html {
    let on_input = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let target: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            on_change.emit(target.value());
        })
    };

    let on_run_click = {
        let on_run = props.on_run.clone();
        Callback::from(move |_| {
            on_run.emit(());
        })
    };

    let on_load_change = {
        let on_load = props.on_load.clone();
        Callback::from(move |e: web_sys::Event| {
            on_load.emit(e);
        })
    };

    let on_save_click = {
        let on_save = props.on_save.clone();
        Callback::from(move |_| {
            on_save.emit(());
        })
    };

    let on_dismiss = {
        let on_tooltip_dismiss = props.on_tooltip_dismiss.clone();
        Callback::from(move |_| {
            on_tooltip_dismiss.emit(());
        })
    };

    html! {
        <div class="panel pipeline-panel">
            <div class="panel-header">
                <h2>{ "Pipeline" }</h2>
                <div class="button-group">
                    <label class="file-button">
                        { "Load" }
                        <input type="file" accept=".pipe" onchange={on_load_change} />
                    </label>
                    <button class="save-button" onclick={on_save_click}>
                        { "Save" }
                    </button>
                    <div class="run-button-container">
                        <button class="run-button" onclick={on_run_click}>
                            { "Run" }
                        </button>
                        if props.show_run_tooltip {
                            <div class="run-tooltip">
                                if props.auto_mode {
                                    { countdown_html(props.countdown, "Auto-run in ", "") }
                                } else {
                                    <>
                                        <span>{ "Click Run to execute" }</span>
                                        <button class="tooltip-dismiss" onclick={on_dismiss}>{ "\u{00D7}" }</button>
                                    </>
                                }
                            </div>
                        }
                    </div>
                </div>
            </div>
            <div class="panel-content">
                <textarea
                    class="pipeline-input"
                    value={props.value.clone()}
                    oninput={on_input}
                    spellcheck="false"
                    wrap="off"
                    rows="8"
                    placeholder="Enter pipeline commands..."
                />
                <div class="dsl-help">
                    <details>
                        <summary>{ "DSL Reference" }</summary>
                        <pre>{r#"PIPE CONSOLE             - Start: read from Input Records
| <stage>                  - Apply transformation stage
| CONSOLE                  - End: write to Output Records
?                          - End of pipeline
# comment                  - Comments ignored

CHANGE /old/new/           - Replace text (any delimiter)
CONSOLE                    - Pass through (middle), debug output
COUNT                      - Output record count
DUPLICATE n                - Repeat each record n times
FILTER pos,len = "v"       - Keep matching records
FILTER pos,len != "v"      - Omit matching records
HOLE                       - Discard all input (like /dev/null)
LITERAL text               - Prefix literal record
LOCATE /pattern/           - Keep records containing pattern
LOCATE pos,len /pattern/   - Keep if field contains pattern
LOWER                      - Convert to lowercase
NLOCATE /pattern/          - Keep records NOT containing pattern
REVERSE                    - Reverse characters in record
SELECT p,l,d; p,l,d        - Select fields (src,len,dest)
SKIP n                     - Skip first n records
TAKE n                     - Keep first n records
UPPER                      - Convert to uppercase"#}</pre>
                    </details>
                </div>
            </div>
        </div>
    }
}

/// Output panel for displaying results.
#[derive(Properties, PartialEq)]
pub struct OutputPanelProps {
    pub value: String,
    pub error: Option<String>,
    pub stats: String,
    #[prop_or(false)]
    pub show_tutorial_buttons: bool,
    #[prop_or_default]
    pub next_tutorial_name: Option<String>,
    #[prop_or_default]
    pub on_next_tutorial: Callback<()>,
    #[prop_or_default]
    pub on_cancel_tutorial: Callback<()>,
    #[prop_or(false)]
    pub auto_mode: bool,
    #[prop_or(0)]
    pub countdown: u32,
    #[prop_or_default]
    pub on_clear: Callback<()>,
}

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

#[function_component(OutputPanel)]
pub fn output_panel(props: &OutputPanelProps) -> Html {
    let on_next_click = {
        let on_next_tutorial = props.on_next_tutorial.clone();
        Callback::from(move |_| {
            on_next_tutorial.emit(());
        })
    };

    let on_cancel_click = {
        let on_cancel_tutorial = props.on_cancel_tutorial.clone();
        Callback::from(move |_| {
            on_cancel_tutorial.emit(());
        })
    };

    let on_clear_click = {
        let on_clear = props.on_clear.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            on_clear.emit(());
        })
    };

    html! {
        <div class="panel output-panel">
            <div class="panel-header">
                <h2>{ "Output Records" }</h2>
                <div class="header-right-group">
                    if !props.stats.is_empty() {
                        <span class="stats">{ &props.stats }</span>
                    }
                    <button class="clear-button" onclick={on_clear_click}>
                        { "Clear" }
                    </button>
                </div>
            </div>
            <div class="panel-content">
                <div class="column-ruler">
                    { "0---------1---------2---------3---------4---------5---------6---------7---------" }
                </div>
                if let Some(error) = &props.error {
                    <div class="error">
                        { error }
                    </div>
                } else {
                    <pre class="record-output">{ &props.value }</pre>
                }
                if props.show_tutorial_buttons {
                    <div class="tutorial-buttons">
                        <div class="next-tutorial-container">
                            <button class="tutorial-button next" onclick={on_next_click}>
                                { "Next Tutorial" }
                            </button>
                            if let Some(next_name) = &props.next_tutorial_name {
                                <div class="next-tooltip">
                                    if props.auto_mode {
                                        { countdown_html(props.countdown, &format!("Next: {} in ", next_name), "") }
                                    } else {
                                        { format!("Next: {}", next_name) }
                                    }
                                </div>
                            }
                        </div>
                        <button class="tutorial-button cancel" onclick={on_cancel_click}>
                            { "Cancel" }
                        </button>
                    </div>
                }
            </div>
        </div>
    }
}
