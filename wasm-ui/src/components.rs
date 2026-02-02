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
                    { "----+----1----+----2----+----3----+----4----+----5----+----6----+----7----+----8" }
                </div>
                <textarea
                    class="record-input"
                    value={props.value.clone()}
                    oninput={on_input}
                    spellcheck="false"
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
                    <button class="run-button" onclick={on_run_click}>
                        { "Run" }
                    </button>
                </div>
            </div>
            <div class="panel-content">
                <textarea
                    class="pipeline-input"
                    value={props.value.clone()}
                    oninput={on_input}
                    spellcheck="false"
                    rows="8"
                    placeholder="Enter pipeline commands..."
                />
                <div class="dsl-help">
                    <details>
                        <summary>{ "DSL Reference" }</summary>
                        <pre>{r#"PIPE <stage> | <stage>?  - Pipeline with stages, ? ends pipe
   | <next-stage>          - Continue to next stage (on new line)
FILTER pos,len = "v"       - Keep matching records
FILTER pos,len != "v"      - Omit matching records
SELECT p,l,d; p,l,d        - Select fields (src,len,dest)
TAKE n                     - Keep first n records
SKIP n                     - Skip first n records
# comment                  - Comments ignored"#}</pre>
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
}

#[function_component(OutputPanel)]
pub fn output_panel(props: &OutputPanelProps) -> Html {
    html! {
        <div class="panel output-panel">
            <div class="panel-header">
                <h2>{ "Output Records" }</h2>
                if !props.stats.is_empty() {
                    <span class="stats">{ &props.stats }</span>
                }
            </div>
            <div class="panel-content">
                <div class="column-ruler">
                    { "----+----1----+----2----+----3----+----4----+----5----+----6----+----7----+----8" }
                </div>
                if let Some(error) = &props.error {
                    <div class="error">
                        { error }
                    </div>
                } else {
                    <pre class="record-output">{ &props.value }</pre>
                }
            </div>
        </div>
    }
}
