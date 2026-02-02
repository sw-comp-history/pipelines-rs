//! Main application component.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, HtmlAnchorElement, HtmlInputElement, Url};
use yew::prelude::*;

use crate::components::{InputPanel, OutputPanel, PipelinePanel};
use crate::dsl::execute_pipeline;

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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_text: DEFAULT_INPUT.to_string(),
            pipeline_text: DEFAULT_PIPELINE.to_string(),
            output_text: String::new(),
            error: None,
            stats: String::new(),
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

const DEFAULT_PIPELINE: &str = r#"PIPE FILTER 18,10 = "SALES"
   | SELECT 0,8,0; 28,8,8?"#;

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
            let anchor: HtmlAnchorElement = document
                .create_element("a")
                .unwrap()
                .dyn_into()
                .unwrap();

            anchor.set_href(&url);
            anchor.set_download("pipeline.pipe");
            anchor.click();

            let _ = Url::revoke_object_url(&url);
        })
    };

    html! {
        <div class="app">
            <header class="header">
                <h1>{ "pipelines-rs" }</h1>
                <p class="subtitle">{ "Mainframe-Style 80-Byte Record Processing" }</p>
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
                        on_run={on_run}
                        on_load={on_load}
                        on_save={on_save}
                    />

                    <OutputPanel
                        value={state.output_text.clone()}
                        error={state.error.clone()}
                        stats={state.stats.clone()}
                    />
                </div>
            </main>

            <footer class="footer">
                <p>{ "80-byte fixed-width records | ASCII | Punch card format" }</p>
            </footer>
        </div>
    }
}
