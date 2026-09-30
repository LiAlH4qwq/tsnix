//! A friendly, agent-compatible REPL.
//!
//! Three protocols are supported:
//!
//! * [`Protocol::Interactive`] — a `rustyline` line editor with history.
//! * [`Protocol::Enq`] — Lix's `repl-automation`: an ENQ byte (U+0005) is
//!   written and flushed before every read, and commands are newline-delimited.
//! * [`Protocol::Json`] — ENQ readiness plus newline-delimited JSON requests
//!   and responses.
//!
//! The engine (`engine::Session`) is independent of the input source, so it is
//! testable without a terminal.

mod engine;
mod interact;

use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use crate::diagnostic::EvalError;
use crate::{Arg, Format, IoMode, Mode};

use engine::{Input, Outcome, Session};
use interact::{EnqInteracter, Interacter, PlainInteracter, ReadlineInteracter};

/// How the REPL reads input and writes output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Interactive line editor with history and prompts.
    Interactive,
    /// Lix `repl-automation`: ENQ before each read, newline-delimited commands.
    Enq,
    /// ENQ before each read, newline-delimited JSON requests/responses.
    Json,
}

/// Options for [`run`].
#[derive(Debug, Clone)]
pub struct ReplOptions {
    /// Filesystem access policy.
    pub io: IoMode,
    /// Forcing mode for displayed values.
    pub mode: Mode,
    /// Display format.
    pub format: Format,
    /// Initial `--arg`/`--argstr` bindings.
    pub args: Vec<Arg>,
    /// Files to load on startup.
    pub load: Vec<PathBuf>,
    /// Input/output protocol.
    pub protocol: Protocol,
    /// Render errors as JSON diagnostics on stderr.
    pub errors_json: bool,
    /// Suppress the banner and non-essential output.
    pub quiet: bool,
}

/// Run a REPL session until end of input or `:quit`.
pub fn run(options: ReplOptions) -> Result<(), EvalError> {
    let mut session = Session::new(&options)?;

    if options.protocol == Protocol::Interactive && !options.quiet && std::io::stdin().is_terminal()
    {
        println!(
            "tsnix {} — Nix language REPL (no store). Type :help for help.",
            env!("CARGO_PKG_VERSION")
        );
    }

    for path in &options.load {
        let command = format!(":load {}", path.display());
        if let Outcome::Error(error) = session.command(&command) {
            return Err(error);
        }
    }

    match options.protocol {
        Protocol::Json => run_json(&mut session),
        Protocol::Interactive => {
            let mut interacter: Box<dyn Interacter> = if std::io::stdin().is_terminal() {
                Box::new(ReadlineInteracter::new())
            } else {
                Box::new(PlainInteracter::new())
            };
            run_stream(&mut session, interacter.as_mut(), &options)
        }
        Protocol::Enq => {
            let mut interacter = EnqInteracter::new();
            run_stream(&mut session, &mut interacter, &options)
        }
    }
}

fn run_stream(
    session: &mut Session,
    interacter: &mut dyn Interacter,
    options: &ReplOptions,
) -> Result<(), EvalError> {
    let mut buffer = String::new();
    loop {
        let first = buffer.is_empty();
        match interacter.read(&mut buffer, first) {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => return Err(io_error(error)),
        }

        if matches!(session.classify(&buffer), Input::Incomplete) {
            continue;
        }

        let outcome = session.execute(&buffer);
        buffer.clear();

        match outcome {
            Outcome::Quit => break,
            Outcome::Value { text, .. } => println!("{text}"),
            Outcome::Message(message) => println!("{message}"),
            Outcome::Bound { .. } | Outcome::Nothing => {}
            Outcome::Error(error) => emit(&error, options),
        }
    }
    interacter.finish();
    Ok(())
}

fn run_json(session: &mut Session) -> Result<(), EvalError> {
    use std::io::BufRead;

    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        {
            let mut out = std::io::stdout();
            out.write_all(b"\x05").map_err(io_error)?;
            out.flush().map_err(io_error)?;
        }

        let Some(line) = lines.next() else {
            break;
        };
        let line = line.map_err(io_error)?;
        if line.trim().is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                write_json(&serde_json::json!({ "ok": false, "error": error.to_string() }))?;
                continue;
            }
        };

        let (response, quit) = session.handle_json(&request);
        write_json(&response)?;
        if quit {
            break;
        }
    }
    Ok(())
}

fn write_json(value: &serde_json::Value) -> Result<(), EvalError> {
    let mut out = std::io::stdout();
    writeln!(out, "{value}").map_err(io_error)?;
    out.flush().map_err(io_error)
}

fn emit(error: &EvalError, options: &ReplOptions) {
    if options.errors_json {
        let value = serde_json::json!({
            "version": 1,
            "ok": false,
            "diagnostics": error.diagnostics,
        });
        let _ = writeln!(std::io::stderr(), "{value}");
        return;
    }
    for diagnostic in &error.diagnostics {
        let text = if diagnostic.rendered.is_empty() {
            diagnostic.message.as_str()
        } else {
            diagnostic.rendered.as_str()
        };
        eprintln!("{text}");
    }
}

fn io_error(error: std::io::Error) -> EvalError {
    EvalError::single(crate::diagnostic::Diagnostic::internal(
        "TSNIX-IO",
        error.to_string(),
    ))
}
