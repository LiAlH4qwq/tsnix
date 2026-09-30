//! Browser (`wasm32-unknown-unknown`) bindings.
//!
//! This is a scaffold: it exposes a tiny JSON-in/JSON-out API for embedding
//! `tsnix` in a web page. It uses the pure evaluator only (`IoMode::None`),
//! because browsers have no synchronous filesystem and `tsnix` never touches a
//! store. See `docs/wasm.md`.
//!
//! Build with:
//!
//! ```console
//! wasm-pack build --no-default-features --features wasm --target web
//! ```

use wasm_bindgen::prelude::*;

use crate::{EvalOptions, Format, IoMode, Mode, Source, evaluate};

fn run(expr: &str, format: Format) -> Result<String, JsValue> {
    let options = EvalOptions {
        source: Source::Expr(expr.to_string()),
        format,
        io: IoMode::None,
        mode: Mode::Strict,
        args: Vec::new(),
        pretty: false,
        nix_path: None,
    };
    evaluate(&options)
        .map(|output| output.text)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Evaluate `expr` and return compact JSON.
#[wasm_bindgen(js_name = evalJson)]
pub fn eval_json(expr: &str) -> Result<String, JsValue> {
    run(expr, Format::Json)
}

/// Evaluate `expr` and return its Nix rendering.
#[wasm_bindgen(js_name = evalNix)]
pub fn eval_nix(expr: &str) -> Result<String, JsValue> {
    run(expr, Format::Nix)
}
