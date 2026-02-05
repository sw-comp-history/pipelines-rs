//! Web UI for pipelines-rs
//!
//! A Yew-based web interface for demonstrating mainframe-style
//! 80-byte record pipeline processing.

mod app;
mod components;
mod debugger;
mod dsl;

use wasm_bindgen::prelude::*;

/// Entry point for the WASM application.
#[wasm_bindgen(start)]
pub fn run_app() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Mount the Yew app
    yew::Renderer::<app::App>::new().render();
}
