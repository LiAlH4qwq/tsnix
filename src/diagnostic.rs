//! Structured and human-readable diagnostics.
//!
//! Diagnostics are the primary contract between `tsnix` and its callers: a
//! failed evaluation yields a [`EvalError`] carrying one or more
//! [`Diagnostic`] values. Diagnostics are serialisable so agents can consume
//! them directly, and carry a rendered snippet for humans.

use std::fmt;

use serde::Serialize;
use snix_eval::{Error as SnixError, EvalWarning, SourceCode};

/// A source position (1-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
}

/// A source range within a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    /// The file the span belongs to, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Inclusive start position.
    pub start: Position,
    /// Exclusive end position.
    pub end: Position,
}

/// Severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A fatal problem; evaluation did not produce a result.
    Error,
    /// A non-fatal problem; the result is still usable.
    Warning,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// A single structured problem produced during evaluation or compilation.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Whether this is an error or a warning.
    pub severity: Severity,
    /// A stable machine-readable code, e.g. `TSNIX-NO-STORE`.
    pub code: String,
    /// The underlying snix error or warning variant, e.g. `TypeError`.
    pub kind: String,
    /// A human-readable message.
    pub message: String,
    /// The source location, if it could be determined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// String contexts attached to the value (paths, derivations, …).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<String>,
    /// A colourless rendered snippet for humans.
    pub rendered: String,
}

impl Diagnostic {
    /// Build a diagnostic from an arbitrary message, with a stable code.
    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        let message = message.into();
        Diagnostic {
            severity: Severity::Error,
            code: code.into(),
            kind: "Internal".to_string(),
            rendered: format!("error: {message}"),
            message,
            span: None,
            contexts: Vec::new(),
        }
    }

    pub(crate) fn from_snix_error(err: &SnixError, line_offset: usize, source_text: &str) -> Self {
        // Runtime errors are frequently wrapped in `NativeError`/`BytecodeError`
        // frames; report the innermost, most specific cause.
        let err = root_error(err);
        let code = code_for(&err.kind);
        let message = detailed_message(&err.kind);
        let span = positions(&err.source, err.span, line_offset);
        let rendered = render_text(Severity::Error, &code, &message, span.as_ref(), source_text);
        Diagnostic {
            severity: Severity::Error,
            code,
            kind: short_debug(&err.kind),
            message,
            span,
            contexts: err.contexts.clone(),
            rendered,
        }
    }

    pub(crate) fn from_warning(
        warning: &EvalWarning,
        source: &SourceCode,
        line_offset: usize,
        source_text: &str,
    ) -> Self {
        let message = short_debug(&warning.kind);
        let code = "TSNIX-WARN".to_string();
        let span = positions(source, warning.span, line_offset);
        let rendered = render_text(
            Severity::Warning,
            &code,
            &message,
            span.as_ref(),
            source_text,
        );
        Diagnostic {
            severity: Severity::Warning,
            code,
            kind: message.clone(),
            message,
            span,
            contexts: Vec::new(),
            rendered,
        }
    }
}

/// A failed evaluation or compilation.
#[derive(Debug, Clone)]
pub struct EvalError {
    /// Fatal diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Non-fatal diagnostics that were still produced.
    pub warnings: Vec<Diagnostic>,
}

impl EvalError {
    /// Create an error from a single diagnostic.
    pub fn single(diagnostic: Diagnostic) -> Self {
        EvalError {
            diagnostics: vec![diagnostic],
            warnings: Vec::new(),
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for diagnostic in &self.diagnostics {
            if !first {
                writeln!(f)?;
            }
            first = false;
            write!(f, "{}", diagnostic.rendered)?;
        }
        Ok(())
    }
}

impl std::error::Error for EvalError {}

fn code_for(kind: &snix_eval::ErrorKind) -> String {
    if let Some(message) = thrown_message(kind)
        && let Some(rest) = message.strip_prefix("tsnix::")
        && let Some((tag, _)) = rest.split_once(':')
    {
        return format!(
            "TSNIX-{}",
            tag.trim().to_ascii_uppercase().replace('_', "-")
        );
    }
    "TSNIX-EVAL".to_string()
}

/// Walk through `NativeError`/`BytecodeError` frames to the underlying cause.
fn root_error(err: &SnixError) -> &SnixError {
    let mut current = err;
    loop {
        match &current.kind {
            snix_eval::ErrorKind::NativeError { err, .. } => current = err,
            snix_eval::ErrorKind::BytecodeError(inner) => current = inner,
            _ => return current,
        }
    }
}

/// Extract the message of a `builtins.throw` (which arrives as a catchable
/// error) or `builtins.abort`.
fn thrown_message(kind: &snix_eval::ErrorKind) -> Option<String> {
    match kind {
        snix_eval::ErrorKind::Abort(message) => Some(message.clone()),
        snix_eval::ErrorKind::CatchableError(snix_eval::CatchableErrorKind::Throw(message)) => {
            message.as_str().map(str::to_owned).ok()
        }
        _ => None,
    }
}

fn detailed_message(kind: &snix_eval::ErrorKind) -> String {
    match kind {
        snix_eval::ErrorKind::ParseErrors(errors) => {
            let mut message = String::from("failed to parse Nix code:");
            for error in errors {
                message.push_str("\n  ");
                message.push_str(&error.to_string());
            }
            message
        }
        snix_eval::ErrorKind::ImportParseError { path, errors, .. } => {
            let mut message = format!("failed to parse `{}`:", path.display());
            for error in errors {
                message.push_str("\n  ");
                message.push_str(&error.to_string());
            }
            message
        }
        snix_eval::ErrorKind::ImportCompilerError { path, errors } => {
            let mut message = format!("failed to compile `{}`:", path.display());
            for error in errors {
                message.push_str("\n  ");
                message.push_str(&error.to_string());
            }
            message
        }
        other => thrown_message(other).unwrap_or_else(|| other.to_string()),
    }
}

/// A concise name for an error or warning variant, e.g. `TypeError` rather than
/// the full nested `Debug` representation.
fn short_debug(value: &impl fmt::Debug) -> String {
    let debug = format!("{value:?}");
    debug
        .split(['(', '{'])
        .next()
        .unwrap_or(&debug)
        .trim()
        .to_string()
}

/// Translate a snix span into our own structured [`Span`], shifting lines by
/// `line_offset` to account for the synthesised prelude/wrapper.
///
/// Span lookup panics if a span does not belong to any known file, which should
/// never happen for spans produced by evaluation; we still guard against it so
/// that a malformed error can never abort the process.
fn positions(source: &SourceCode, span: codemap::Span, line_offset: usize) -> Option<Span> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let codemap = source.codemap();
        let loc = codemap.look_up_span(span);
        let start = shift(loc.begin.line, loc.begin.column, line_offset);
        let end = shift(loc.end.line, loc.end.column, line_offset);
        let file = loc.file.name().to_string();
        Span {
            file: (!file.is_empty()).then_some(file),
            start,
            end,
        }
    }))
    .ok()
}

fn shift(line: usize, col: usize, line_offset: usize) -> Position {
    Position {
        line: (line + 1).saturating_sub(line_offset).max(1),
        column: col + 1,
    }
}

fn render_text(
    severity: Severity,
    code: &str,
    message: &str,
    span: Option<&Span>,
    source_text: &str,
) -> String {
    let mut out = format!("{}[{}]: {}", severity.as_str(), code, message);
    let Some(span) = span else {
        return out;
    };
    let file = span.file.as_deref().unwrap_or("<input>");
    out.push_str(&format!(
        "\n  --> {}:{}:{}",
        file, span.start.line, span.start.column
    ));

    let Some(line) = source_text.lines().nth(span.start.line.saturating_sub(1)) else {
        return out;
    };
    let start = span.start.column.saturating_sub(1);
    let end = if span.end.line == span.start.line {
        span.end.column.saturating_sub(1).max(start + 1)
    } else {
        line.chars().count().max(start + 1)
    };
    out.push_str("\n  |");
    out.push_str(&format!("\n{:>2} | {}", span.start.line, line));
    let mut caret = String::new();
    caret.push_str(&" ".repeat(start));
    caret.push('^');
    caret.push_str(&"~".repeat(end.saturating_sub(start + 1)));
    out.push_str(&format!("\n  | {caret}"));
    out
}
