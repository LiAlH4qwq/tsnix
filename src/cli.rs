//! Command-line interface.
//!
//! The CLI is deliberately not a Nix clone: results go to stdout as pure
//! payloads, diagnostics go to stderr, and a machine-readable contract is
//! available via `tsnix schema`.

use std::io::{IsTerminal, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde_json::json;

use tsnix::{Arg, CheckOptions, Diagnostic, EvalError, EvalOptions, Format, IoMode, Mode, Source};

#[cfg(feature = "repl")]
use tsnix::repl::{Protocol as ReplProtocol, ReplOptions};

/// A minimal Nix-language evaluator (no store).
#[derive(Debug, Parser)]
#[command(name = "tsnix", version, about, long_about = None, disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Evaluate a Nix expression or file.
    Eval(EvalArgs),
    /// Parse and compile without evaluating.
    Check(CheckArgs),
    /// Start a Nix-language REPL (friendly or agent mode).
    #[cfg(feature = "repl")]
    Repl(ReplArgs),
    /// Print a machine-readable description of this CLI.
    Schema(SchemaArgs),
    /// Print the `std` standard library catalogue as JSON.
    #[cfg(feature = "stdlib")]
    Libdoc(LibdocArgs),
}

/// Variable bindings and I/O policy, shared by all commands.
#[derive(Debug, Args)]
struct BindingArgs {
    /// Bind a name to a parsed Nix expression (`--arg NAME EXPR`).
    #[arg(long, num_args = 2, action = ArgAction::Append, value_names = ["NAME", "EXPR"], allow_hyphen_values = true)]
    arg: Vec<String>,

    /// Bind a name to a string (`--argstr NAME STRING`).
    #[arg(long, num_args = 2, action = ArgAction::Append, value_names = ["NAME", "STRING"], allow_hyphen_values = true)]
    argstr: Vec<String>,

    /// Allow local file I/O (`import`, `readFile`, …).
    #[arg(long, value_enum, default_value_t = CliIo::None)]
    io: CliIo,

    /// Diagnostics format.
    #[arg(long, value_enum, default_value_t = ErrorFormat::Auto)]
    error_format: ErrorFormat,

    /// Suppress non-essential output.
    #[arg(short = 'q', long)]
    quiet: bool,
}

impl BindingArgs {
    fn args(&self) -> Vec<Arg> {
        let mut args = Vec::new();
        for pair in self.arg.chunks_exact(2) {
            args.push(Arg::Nix {
                name: pair[0].clone(),
                expr: pair[1].clone(),
            });
        }
        for pair in self.argstr.chunks_exact(2) {
            args.push(Arg::Str {
                name: pair[0].clone(),
                value: pair[1].clone(),
            });
        }
        args
    }

    fn io(&self) -> IoMode {
        match self.io {
            CliIo::None => IoMode::None,
            CliIo::Local => IoMode::Local,
        }
    }
}

#[derive(Debug, Args)]
struct CommonArgs {
    /// Path to a .nix file, or `-` for stdin.
    #[arg(value_name = "SOURCE")]
    source: Option<PathBuf>,

    /// Evaluate an inline expression.
    #[arg(
        short = 'e',
        long,
        value_name = "EXPR",
        conflicts_with = "source",
        allow_hyphen_values = true
    )]
    expr: Option<String>,

    /// Evaluate a file.
    #[arg(short = 'f', long, value_name = "PATH", conflicts_with_all = ["source", "expr"])]
    file: Option<PathBuf>,

    #[command(flatten)]
    bindings: BindingArgs,
}

impl CommonArgs {
    fn source(&self) -> Source {
        if let Some(expr) = &self.expr {
            Source::Expr(expr.clone())
        } else if let Some(file) = &self.file {
            Source::File(file.clone())
        } else if let Some(source) = &self.source {
            Source::File(source.clone())
        } else {
            Source::Stdin
        }
    }
}

#[derive(Debug, Args)]
struct EvalArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// Output format.
    #[arg(short = 'F', long, value_enum, default_value_t = CliFormat::Json)]
    format: CliFormat,

    /// Pretty-print JSON output (best effort).
    #[arg(long)]
    pretty: bool,

    /// Top-level forcing mode.
    #[arg(long, value_enum, default_value_t = CliMode::Strict)]
    mode: CliMode,

    /// `NIX_PATH` for `<...>` lookups (requires `--io local`).
    #[arg(long, value_name = "SPEC")]
    nix_path: Option<String>,

    /// Write the result to a file instead of stdout.
    #[arg(short = 'o', long, value_name = "PATH")]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct CheckArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[cfg(feature = "repl")]
#[derive(Debug, Args)]
struct ReplArgs {
    #[command(flatten)]
    bindings: BindingArgs,

    /// Output format for displayed values.
    #[arg(short = 'F', long, value_enum, default_value_t = CliFormat::Nix)]
    format: CliFormat,

    /// Forcing mode for displayed values.
    #[arg(long, value_enum, default_value_t = CliMode::Strict)]
    mode: CliMode,

    /// Load a file on startup (repeatable).
    #[arg(short = 'l', long = "load", value_name = "PATH")]
    load: Vec<PathBuf>,

    /// Input/output protocol.
    #[arg(long, value_enum, default_value_t = CliProtocol::Interactive)]
    protocol: CliProtocol,

    /// Shorthand for `--protocol enq` (Lix repl-automation).
    #[arg(long, visible_alias = "automation")]
    agent: bool,
}

#[derive(Debug, Args)]
struct SchemaArgs {
    /// Pretty-print the schema.
    #[arg(long)]
    pretty: bool,
}

#[cfg(feature = "stdlib")]
#[derive(Debug, Args)]
struct LibdocArgs {
    /// Pretty-print the catalogue.
    #[arg(long)]
    pretty: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliFormat {
    Json,
    Raw,
    Nix,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliIo {
    None,
    Local,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMode {
    Strict,
    Lazy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliProtocol {
    Interactive,
    Enq,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ErrorFormat {
    Auto,
    Text,
    Json,
}

impl ErrorFormat {
    fn resolve(self) -> Self {
        match self {
            ErrorFormat::Auto => {
                if std::io::stderr().is_terminal() {
                    ErrorFormat::Text
                } else {
                    ErrorFormat::Json
                }
            }
            explicit => explicit,
        }
    }
}

/// Parse arguments and run.
pub fn main() -> ExitCode {
    let command = Cli::parse().command;
    match command {
        Command::Eval(args) => run_eval(args),
        Command::Check(args) => run_check(args),
        #[cfg(feature = "repl")]
        Command::Repl(args) => run_repl(args),
        Command::Schema(args) => run_schema(args),
        #[cfg(feature = "stdlib")]
        Command::Libdoc(args) => run_libdoc(args),
    }
}

fn run_eval(args: EvalArgs) -> ExitCode {
    let options = EvalOptions {
        source: args.common.source(),
        format: to_format(args.format),
        io: args.common.bindings.io(),
        mode: to_mode(args.mode),
        args: args.common.bindings.args(),
        pretty: args.pretty,
        nix_path: args.nix_path,
    };

    let diagnostics = args.common.bindings.error_format.resolve();
    match tsnix::evaluate(&options) {
        Ok(output) => {
            if !args.common.bindings.quiet {
                emit_diagnostics(&output.warnings, diagnostics);
            }
            match write_output(&output.text, args.output.as_deref()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    emit_error(
                        &EvalError::single(Diagnostic::internal(
                            "TSNIX-IO",
                            format!("cannot write output: {error}"),
                        )),
                        diagnostics,
                    );
                    ExitCode::from(1)
                }
            }
        }
        Err(error) => {
            emit_error(&error, diagnostics);
            ExitCode::from(1)
        }
    }
}

fn run_check(args: CheckArgs) -> ExitCode {
    let options = CheckOptions {
        source: args.common.source(),
        io: args.common.bindings.io(),
        args: args.common.bindings.args(),
    };
    let diagnostics = args.common.bindings.error_format.resolve();
    match tsnix::check(&options) {
        Ok(warnings) => {
            if !args.common.bindings.quiet {
                emit_diagnostics(&warnings, diagnostics);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            emit_error(&error, diagnostics);
            ExitCode::from(1)
        }
    }
}

#[cfg(feature = "repl")]
fn run_repl(args: ReplArgs) -> ExitCode {
    let protocol = if args.agent && args.protocol == CliProtocol::Interactive {
        CliProtocol::Enq
    } else {
        args.protocol
    };
    let error_format = args.bindings.error_format.resolve();
    let options = ReplOptions {
        io: args.bindings.io(),
        mode: to_mode(args.mode),
        format: to_format(args.format),
        args: args.bindings.args(),
        load: args.load,
        protocol: match protocol {
            CliProtocol::Interactive => ReplProtocol::Interactive,
            CliProtocol::Enq => ReplProtocol::Enq,
            CliProtocol::Json => ReplProtocol::Json,
        },
        errors_json: error_format == ErrorFormat::Json,
        quiet: args.bindings.quiet,
    };
    match tsnix::repl::run(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            emit_error(&error, error_format);
            ExitCode::from(1)
        }
    }
}

fn run_schema(args: SchemaArgs) -> ExitCode {
    let value = tsnix::schema();
    let text = if args.pretty {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    };
    match text {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("tsnix: cannot serialise schema: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(feature = "stdlib")]
fn run_libdoc(args: LibdocArgs) -> ExitCode {
    let value = tsnix::stdlib_catalogue();
    let text = if args.pretty {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    };
    match text {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("tsnix: cannot serialise catalogue: {error}");
            ExitCode::from(1)
        }
    }
}

fn to_format(format: CliFormat) -> Format {
    match format {
        CliFormat::Json => Format::Json,
        CliFormat::Raw => Format::Raw,
        CliFormat::Nix => Format::Nix,
    }
}

fn to_mode(mode: CliMode) -> Mode {
    match mode {
        CliMode::Strict => Mode::Strict,
        CliMode::Lazy => Mode::Lazy,
    }
}

fn emit_diagnostics(diagnostics: &[Diagnostic], format: ErrorFormat) {
    if diagnostics.is_empty() {
        return;
    }
    match format {
        ErrorFormat::Json => {
            let value = json!({ "version": 1, "ok": true, "diagnostics": diagnostics });
            let _ = writeln!(std::io::stderr(), "{value}");
        }
        _ => {
            for diagnostic in diagnostics {
                let text = if diagnostic.rendered.is_empty() {
                    diagnostic.message.as_str()
                } else {
                    diagnostic.rendered.as_str()
                };
                eprintln!("{text}");
            }
        }
    }
}

fn emit_error(error: &EvalError, format: ErrorFormat) {
    match format {
        ErrorFormat::Json => {
            let value = json!({ "version": 1, "ok": false, "diagnostics": error.diagnostics });
            let _ = writeln!(std::io::stderr(), "{value}");
        }
        _ => {
            for diagnostic in &error.diagnostics {
                let text = if diagnostic.rendered.is_empty() {
                    diagnostic.message.as_str()
                } else {
                    diagnostic.rendered.as_str()
                };
                eprintln!("{text}");
            }
        }
    }
}

fn write_output(text: &str, path: Option<&std::path::Path>) -> std::io::Result<()> {
    match path {
        Some(path) => std::fs::write(path, format!("{text}\n")),
        None => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            lock.write_all(text.as_bytes())?;
            lock.write_all(b"\n")
        }
    }
}
