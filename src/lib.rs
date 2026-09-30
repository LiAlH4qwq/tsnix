//! Minimal, embeddable Nix-language evaluator built solely on `snix-eval`.
//!
//! This crate intentionally contains **no Nix store**. Evaluation is pure: any
//! function that would require a store (`derivation`, `fetchurl`, `writeFile`,
//! …) is replaced by a builtin that raises a stable Nix error. Local file I/O
//! (`import`, `readFile`, …) is opt-in via [`IoMode::Local`].
//!
//! # Quick start
//!
//! ```
//! use tsnix::{evaluate, EvalOptions, Format, IoMode, Mode, Source};
//!
//! let options = EvalOptions {
//!     source: Source::Expr("{ a = 1; b = [ true null ]; }".to_string()),
//!     format: Format::Json,
//!     io: IoMode::None,
//!     mode: Mode::Strict,
//!     args: Vec::new(),
//!     pretty: false,
//!     nix_path: None,
//! };
//! let output = evaluate(&options).unwrap();
//! assert_eq!(output.text, r#"{"a":1,"b":[true,null]}"#);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod diagnostic;
mod eval;
mod output;
mod program;
#[cfg(feature = "repl")]
pub mod repl;
mod schema;
mod stubs;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use diagnostic::{Diagnostic, EvalError, Position, Severity, Span};
pub use eval::{check, evaluate};
pub use schema::schema;

/// The output format for a successful evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// `builtins.toJSON` of the value.
    Json,
    /// `builtins.toString` of the value.
    Raw,
    /// Best-effort Nix pretty printing; functions are rejected.
    Nix,
}

/// How much filesystem access evaluation is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IoMode {
    /// No I/O: `import`, `readFile`, … raise a stable error.
    None,
    /// Local file I/O is enabled.
    Local,
}

/// Top-level forcing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Deeply force the result.
    Strict,
    /// Return the result as-is.
    Lazy,
}

/// Where the Nix source comes from.
#[derive(Debug, Clone)]
pub enum Source {
    /// An inline expression.
    Expr(String),
    /// A file path (or `-` for stdin).
    File(PathBuf),
    /// Standard input.
    Stdin,
}

/// A top-level variable binding, mirroring Nix's `--arg`/`--argstr`.
#[derive(Debug, Clone)]
pub enum Arg {
    /// `--arg NAME NIX_EXPR`: the value is the parsed expression.
    Nix {
        /// The variable name (must be a valid Nix identifier).
        name: String,
        /// The Nix expression source.
        expr: String,
    },
    /// `--argstr NAME STRING`: the value is a string.
    Str {
        /// The variable name (must be a valid Nix identifier).
        name: String,
        /// The string value.
        value: String,
    },
}

/// Options for [`evaluate`].
#[derive(Debug, Clone)]
pub struct EvalOptions {
    /// The input source.
    pub source: Source,
    /// The output format.
    pub format: Format,
    /// Filesystem access policy.
    pub io: IoMode,
    /// Forcing mode.
    pub mode: Mode,
    /// Top-level variable bindings.
    pub args: Vec<Arg>,
    /// Pretty-print JSON output (best effort).
    pub pretty: bool,
    /// Optional `NIX_PATH` used for `<...>` lookups (with [`IoMode::Local`]).
    pub nix_path: Option<String>,
}

/// Options for [`check`].
#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// The input source.
    pub source: Source,
    /// Filesystem access policy.
    pub io: IoMode,
    /// Top-level variable bindings.
    pub args: Vec<Arg>,
}

/// A successful evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct EvalOutput {
    /// The format the text is rendered in.
    pub format: Format,
    /// The rendered result.
    pub text: String,
    /// Non-fatal diagnostics.
    pub warnings: Vec<Diagnostic>,
}
