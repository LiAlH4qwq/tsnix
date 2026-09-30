//! REPL session state: parsing submissions, evaluating them against a shared
//! globals map, and executing `:commands`.

use std::rc::Rc;

use rnix::ast::HasEntry;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use snix_eval::{GlobalsMap, SourceCode, Value};

use crate::diagnostic::{Diagnostic, EvalError};
use crate::eval::{base_builder, to_eval_mode};
use crate::{Arg, Format, IoMode, Mode};

use super::ReplOptions;

/// A parsed binding of the form `name = value;`.
pub(crate) struct Binding {
    pub name: String,
    pub value: String,
}

/// Classification of a complete REPL submission.
pub(crate) enum Input {
    Empty,
    Command(String),
    Bindings(Vec<Binding>),
    Expr(String),
    Incomplete,
}

/// The result of executing a submission.
pub(crate) enum Outcome {
    Value { text: String, format: Format },
    Bound { names: Vec<String> },
    Message(String),
    Error(EvalError),
    Quit,
    Nothing,
}

/// A persistent REPL session.
pub struct Session {
    io: IoMode,
    mode: Mode,
    format: Format,
    globals: Rc<GlobalsMap>,
    source_map: SourceCode,
    env: FxHashMap<SmolStr, Value>,
}

impl Session {
    /// Create a session, seeding `--arg`/`--argstr` bindings.
    pub fn new(options: &ReplOptions) -> Result<Self, EvalError> {
        let seed = base_builder(options.io)?.build();
        let mut session = Session {
            io: options.io,
            mode: options.mode,
            format: options.format,
            globals: seed.globals(),
            source_map: seed.source_map(),
            env: FxHashMap::default(),
        };

        for arg in &options.args {
            match arg {
                Arg::Str { name, value } => {
                    session
                        .env
                        .insert(SmolStr::new(name.as_str()), Value::from(value.clone()));
                }
                Arg::Nix { name, expr } => {
                    let value = session.eval_value(expr, Mode::Lazy)?;
                    session.env.insert(SmolStr::new(name.as_str()), value);
                }
            }
        }
        Ok(session)
    }

    /// Evaluate an expression against the shared globals and current bindings.
    fn eval_value(&self, code: &str, mode: Mode) -> Result<Value, EvalError> {
        let evaluation = base_builder(self.io)?
            .with_globals(self.globals.clone())
            .with_source_map(self.source_map.clone())
            .env(Some(&self.env))
            .mode(to_eval_mode(mode))
            .build();
        let result = evaluation.evaluate(code, None);
        if !result.errors.is_empty() {
            let diagnostics = result
                .errors
                .iter()
                .map(|error| Diagnostic::from_snix_error(error, 0, code))
                .collect();
            return Err(EvalError {
                diagnostics,
                warnings: Vec::new(),
            });
        }
        result.value.ok_or_else(|| {
            EvalError::single(Diagnostic::internal(
                "TSNIX-NO-VALUE",
                "evaluation produced no value",
            ))
        })
    }

    /// Classify a (possibly multi-line) submission.
    pub fn classify(&self, input: &str) -> Input {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Input::Empty;
        }
        if trimmed.starts_with(':') {
            return Input::Command(trimmed.to_string());
        }
        if let Some(bindings) = parse_bindings(input) {
            return Input::Bindings(bindings);
        }
        let parsed = rnix::ast::Root::parse(input);
        let errors: Vec<String> = parsed.errors().iter().map(|e| e.to_string()).collect();
        if errors.is_empty() {
            return Input::Expr(input.to_string());
        }
        if is_incomplete(input, &errors) {
            return Input::Incomplete;
        }
        // Let the evaluator produce a proper parse/type error.
        Input::Expr(input.to_string())
    }

    /// Execute a complete submission.
    pub fn execute(&mut self, input: &str) -> Outcome {
        match self.classify(input) {
            Input::Empty => Outcome::Nothing,
            Input::Command(command) => self.command(&command),
            Input::Bindings(bindings) => self.bind(bindings),
            Input::Expr(expr) => self.render_expr(&expr),
            Input::Incomplete => Outcome::Error(EvalError::single(Diagnostic::internal(
                "TSNIX-INCOMPLETE",
                "incomplete input",
            ))),
        }
    }

    fn bind(&mut self, bindings: Vec<Binding>) -> Outcome {
        let mut names = Vec::with_capacity(bindings.len());
        for binding in bindings {
            match self.eval_value(&binding.value, Mode::Lazy) {
                Ok(value) => {
                    self.env.insert(SmolStr::new(binding.name.as_str()), value);
                    names.push(binding.name);
                }
                Err(error) => return Outcome::Error(error),
            }
        }
        Outcome::Bound { names }
    }

    fn render_expr(&self, expr: &str) -> Outcome {
        match self.format {
            Format::Nix => match self.eval_value(expr, self.mode) {
                Ok(value) => Outcome::Value {
                    text: display(&value),
                    format: Format::Nix,
                },
                Err(error) => Outcome::Error(error),
            },
            Format::Json => {
                match self.eval_value(&format!("builtins.toJSON ({expr})"), self.mode) {
                    Ok(value) => match string_of(&value) {
                        Ok(text) => Outcome::Value {
                            text,
                            format: Format::Json,
                        },
                        Err(error) => Outcome::Error(error),
                    },
                    Err(error) => Outcome::Error(error),
                }
            }
            Format::Raw => match self.eval_value(&format!("builtins.toString ({expr})"), self.mode)
            {
                Ok(value) => match string_of(&value) {
                    Ok(text) => Outcome::Value {
                        text,
                        format: Format::Raw,
                    },
                    Err(error) => Outcome::Error(error),
                },
                Err(error) => Outcome::Error(error),
            },
        }
    }

    /// Execute a `:command`.
    pub fn command(&mut self, command: &str) -> Outcome {
        let mut parts = command
            .trim_start_matches(':')
            .splitn(2, char::is_whitespace);
        let name = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        match name {
            "quit" | "q" => Outcome::Quit,
            "help" | "?" => Outcome::Message(HELP.to_string()),
            "format" => {
                if rest.is_empty() {
                    return Outcome::Message(format!("format = {}", format_name(self.format)));
                }
                match parse_format(rest) {
                    Some(format) => {
                        self.format = format;
                        Outcome::Message(format!("format = {}", format_name(format)))
                    }
                    None => Outcome::Message(format!(
                        "unknown format `{rest}` (expected nix, json or raw)"
                    )),
                }
            }
            "mode" => {
                if rest.is_empty() {
                    return Outcome::Message(format!("mode = {}", mode_name(self.mode)));
                }
                match parse_mode(rest) {
                    Some(mode) => {
                        self.mode = mode;
                        Outcome::Message(format!("mode = {}", mode_name(mode)))
                    }
                    None => {
                        Outcome::Message(format!("unknown mode `{rest}` (expected strict or lazy)"))
                    }
                }
            }
            "bindings" => {
                let mut names: Vec<String> = self
                    .env
                    .keys()
                    .map(|key| key.as_str().to_string())
                    .collect();
                names.sort();
                if names.is_empty() {
                    Outcome::Message("no bindings".to_string())
                } else {
                    Outcome::Message(names.join("  "))
                }
            }
            "clear" => {
                self.env.clear();
                Outcome::Message("bindings cleared".to_string())
            }
            "type" | "t" => match self.eval_value(rest, self.mode) {
                Ok(value) => Outcome::Message(format!("{} :: {}", rest, value.type_of())),
                Err(error) => Outcome::Error(error),
            },
            "print" | "p" => self.render_expr(rest),
            "doc" => match self.eval_value(rest, self.mode) {
                Ok(value) => Outcome::Message(explain(&value)),
                Err(error) => Outcome::Error(error),
            },
            "load" | "l" => self.load(rest),
            other => Outcome::Message(format!("unknown command `:{other}` (try `:help`)")),
        }
    }

    fn load(&mut self, path: &str) -> Outcome {
        if path.is_empty() {
            return Outcome::Message("usage: :load <path>".to_string());
        }
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                return Outcome::Error(EvalError::single(Diagnostic::internal(
                    "TSNIX-IO",
                    format!("cannot read `{path}`: {error}"),
                )));
            }
        };
        match self.classify(&text) {
            Input::Bindings(bindings) => {
                let outcome = self.bind(bindings);
                match outcome {
                    Outcome::Bound { names } => {
                        Outcome::Message(format!("added {} binding(s)", names.len()))
                    }
                    other => other,
                }
            }
            Input::Expr(_) => match self.eval_value(&format!("({text})"), Mode::Lazy) {
                Ok(Value::Attrs(attrs)) => {
                    let mut added = 0;
                    for (key, value) in attrs.iter() {
                        if let Ok(key) = key.as_str() {
                            self.env.insert(SmolStr::new(key), value.clone());
                            added += 1;
                        }
                    }
                    Outcome::Message(format!("added {added} binding(s)"))
                }
                Ok(other) => Outcome::Value {
                    text: display(&other),
                    format: Format::Nix,
                },
                Err(error) => Outcome::Error(error),
            },
            _ => Outcome::Message("nothing to load".to_string()),
        }
    }

    /// Handle a JSON request, returning `(response, quit)`.
    pub fn handle_json(&mut self, request: &serde_json::Value) -> (serde_json::Value, bool) {
        use serde_json::json;

        let outcome = match request {
            serde_json::Value::String(input) => self.execute(input),
            serde_json::Value::Object(map) => {
                if let Some(expr) = map.get("expr").and_then(|v| v.as_str()) {
                    self.execute(expr)
                } else if let Some(command) = map.get("command").and_then(|v| v.as_str()) {
                    let arg = map.get("arg").and_then(|v| v.as_str()).unwrap_or("");
                    let command = if arg.is_empty() {
                        format!(":{command}")
                    } else {
                        format!(":{command} {arg}")
                    };
                    self.command(&command)
                } else if let Some(input) = map.get("input").and_then(|v| v.as_str()) {
                    self.execute(input)
                } else {
                    return (
                        json!({ "ok": false, "error": "expected {\"expr\"|\"command\"|\"input\"}" }),
                        false,
                    );
                }
            }
            _ => {
                return (
                    json!({ "ok": false, "error": "expected a string or object" }),
                    false,
                );
            }
        };

        match outcome {
            Outcome::Value { text, format } => (
                json!({ "ok": true, "format": format_name(format), "value": text }),
                false,
            ),
            Outcome::Bound { names } => (json!({ "ok": true, "bound": names }), false),
            Outcome::Message(message) => (json!({ "ok": true, "message": message }), false),
            Outcome::Nothing => (json!({ "ok": true }), false),
            Outcome::Quit => (json!({ "ok": true, "quit": true }), true),
            Outcome::Error(error) => (
                json!({ "ok": false, "diagnostics": error.diagnostics }),
                false,
            ),
        }
    }
}

const HELP: &str = "\
commands:
  :help, :?            show this help
  :quit, :q            leave the repl
  :type, :t EXPR       print the type of EXPR
  :print, :p EXPR      evaluate and print EXPR
  :doc EXPR            show documentation for a value
  :load, :l FILE       evaluate FILE; an attrset is added to scope
  :format [FMT]        get/set output format (nix, json, raw)
  :mode [MODE]         get/set forcing mode (strict, lazy)
  :bindings            list current bindings
  :clear               drop all bindings

bindings: `name = expr;` adds `name` to scope.";

fn parse_bindings(input: &str) -> Option<Vec<Binding>> {
    let mut text = input.trim().to_string();
    if text.is_empty() {
        return None;
    }
    if !text.ends_with(';') {
        text.push(';');
    }
    let wrapped = format!("{{ {text} }}");
    let parsed = rnix::ast::Root::parse(&wrapped);
    if !parsed.errors().is_empty() {
        return None;
    }
    let rnix::ast::Expr::AttrSet(set) = parsed.tree().expr()? else {
        return None;
    };

    let mut bindings = Vec::new();
    for entry in set.entries() {
        let rnix::ast::Entry::AttrpathValue(entry) = entry else {
            return None;
        };
        let attrpath = entry.attrpath()?;
        let mut attrs = attrpath.attrs();
        let attr = attrs.next()?;
        if attrs.next().is_some() {
            return None;
        }
        let name = attr.to_string();
        if !is_identifier(&name) {
            return None;
        }
        let value = entry.value()?;
        bindings.push(Binding {
            name,
            value: value.to_string(),
        });
    }
    (!bindings.is_empty()).then_some(bindings)
}

fn is_incomplete(input: &str, errors: &[String]) -> bool {
    if errors.iter().any(|error| error.contains("end of file")) {
        return true;
    }
    let trimmed = input.trim_end();
    let last = trimmed.chars().last();
    if matches!(
        last,
        Some(
            '=' | '+'
                | '-'
                | '*'
                | '/'
                | '<'
                | '>'
                | '!'
                | '&'
                | '|'
                | ','
                | '.'
                | '@'
                | '('
                | '['
                | '{'
                | ':'
        )
    ) {
        return true;
    }
    [
        " in", " let", " then", " else", " with", " assert", " inherit", " or", " rec",
    ]
    .iter()
    .any(|keyword| trimmed.ends_with(keyword))
}

pub(crate) fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '\'' | '-'))
}

fn display(value: &Value) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| format!("{value}")))
        .unwrap_or_else(|_| "<error>".to_string())
}

fn string_of(value: &Value) -> Result<String, EvalError> {
    let string = value.to_str().map_err(|kind| {
        EvalError::single(Diagnostic::internal(
            "TSNIX-EXPECTED-STRING",
            format!("expected a string result: {kind}"),
        ))
    })?;
    string
        .as_str()
        .map(|text| text.to_owned())
        .map_err(|error| EvalError::single(Diagnostic::internal("TSNIX-UTF8", error.to_string())))
}

fn explain(value: &Value) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match value {
        Value::Builtin(builtin) => match builtin.documentation() {
            Some(docs) => docs.to_string(),
            None => format!("builtin `{}`", builtin.name()),
        },
        _ => value.explain(),
    }))
    .unwrap_or_else(|_| display(value))
}

fn parse_format(text: &str) -> Option<Format> {
    match text {
        "nix" => Some(Format::Nix),
        "json" => Some(Format::Json),
        "raw" => Some(Format::Raw),
        _ => None,
    }
}

fn format_name(format: Format) -> &'static str {
    match format {
        Format::Nix => "nix",
        Format::Json => "json",
        Format::Raw => "raw",
    }
}

fn parse_mode(text: &str) -> Option<Mode> {
    match text {
        "strict" => Some(Mode::Strict),
        "lazy" => Some(Mode::Lazy),
        _ => None,
    }
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Strict => "strict",
        Mode::Lazy => "lazy",
    }
}
