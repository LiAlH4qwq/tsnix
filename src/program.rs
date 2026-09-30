//! Turning a [`Source`] plus `--arg`/`--argstr` bindings and an output format
//! into a single Nix program, and reading inputs from disk or stdin.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::diagnostic::{Diagnostic, EvalError};
use crate::{Arg, Source};

/// The wrapper inserted around the user's code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wrap {
    /// `(\n…\n)`: evaluate and return the value (for `check`).
    Plain,
    /// `builtins.toJSON (\n…\n)`.
    ToJson,
    /// `builtins.toString (\n…\n)`.
    ToString,
}

/// A composed program ready for evaluation.
pub(crate) struct Program {
    /// The full Nix source, including any synthesised prelude and wrapper.
    pub code: String,
    /// How many lines precede the user's own source.
    pub line_offset: usize,
    /// The path used for relative-path resolution and error reporting.
    pub location: Option<PathBuf>,
}

/// Read the raw user source, along with its location and display label.
pub(crate) fn read_source(source: &Source) -> Result<(String, Option<PathBuf>, String), EvalError> {
    match source {
        Source::Expr(expr) => {
            let location = synthetic_location("<expr>");
            Ok((expr.clone(), Some(location), "<expr>".to_string()))
        }
        Source::Stdin => {
            let content = read_stdin()?;
            let location = synthetic_location("<stdin>");
            Ok((content, Some(location), "<stdin>".to_string()))
        }
        Source::File(path) if path.to_str() == Some("-") => {
            let content = read_stdin()?;
            let location = synthetic_location("<stdin>");
            Ok((content, Some(location), "<stdin>".to_string()))
        }
        Source::File(path) => {
            let content = std::fs::read_to_string(path).map_err(|err| {
                EvalError::single(Diagnostic::internal(
                    "TSNIX-IO",
                    format!("cannot read `{}`: {err}", path.display()),
                ))
            })?;
            Ok((content, Some(path.clone()), path.display().to_string()))
        }
    }
}

fn read_stdin() -> Result<String, EvalError> {
    use std::io::Read as _;
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|err| {
            EvalError::single(Diagnostic::internal(
                "TSNIX-IO",
                format!("cannot read stdin: {err}"),
            ))
        })?;
    Ok(buffer)
}

fn synthetic_location(name: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(name)
}

/// Compose the user source with the `--arg` prelude and, depending on the
/// output format, a `builtins.toJSON`/`builtins.toString` wrapper.
///
/// The synthesised prefix ends with a newline, so the user's line numbers are
/// merely shifted by [`Program::line_offset`] and columns stay exact.
pub(crate) fn compose(base: &str, args: &[Arg], wrap: Wrap) -> Result<Program, EvalError> {
    let mut prelude = String::new();
    for arg in args {
        let (name, value) = match arg {
            Arg::Nix { name, expr } => (name, format!("({expr})")),
            Arg::Str { name, value } => (name, format!("\"{}\"", escape_nix_string(value))),
        };
        validate_ident(name)?;
        writeln!(prelude, "  {name} = {value};").expect("writing to String never fails");
    }

    let prefix = if args.is_empty() {
        String::new()
    } else {
        format!("let\n{prelude}in\n")
    };
    let prefix_lines = prefix.matches('\n').count();

    let open = match wrap {
        Wrap::Plain => "(\n",
        Wrap::ToJson => "builtins.toJSON (\n",
        Wrap::ToString => "builtins.toString (\n",
    };
    let close = "\n)";

    let code = format!("{prefix}{open}{base}{close}");
    let line_offset = prefix_lines + open.matches('\n').count();

    Ok(Program {
        code,
        line_offset,
        location: None,
    })
}

fn validate_ident(name: &str) -> Result<(), EvalError> {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '\'' | '-'));
    if valid {
        Ok(())
    } else {
        Err(EvalError::single(Diagnostic::internal(
            "TSNIX-USAGE",
            format!("invalid argument name `{name}` (expected a Nix identifier)"),
        )))
    }
}

/// Escape a Rust string so it can be embedded in a Nix double-quoted string.
fn escape_nix_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }
    out
}
