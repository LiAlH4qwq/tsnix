//! Rendering a JSON value back into Nix source.
//!
//! This is a best-effort pretty printer used only by `--format nix`. It works
//! on the JSON projection of the value (see `builtins.toJSON`), which keeps the
//! behaviour consistent with `--format json` and avoids unforceable thunks.
//! A consequence is that paths render as strings and functions are rejected by
//! `builtins.toJSON` before reaching this module.

use serde_json::Value;

/// Render a JSON value as Nix source.
pub(crate) fn render_nix(json: &Value) -> String {
    let mut out = String::new();
    write(&mut out, json);
    out
}

fn write(out: &mut String, json: &Value) {
    match json {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                out.push_str(&integer.to_string());
            } else if let Some(integer) = number.as_u64() {
                out.push_str(&integer.to_string());
            } else if let Some(float) = number.as_f64() {
                out.push_str(&format_float(float));
            } else {
                out.push_str(&number.to_string());
            }
        }
        Value::String(string) => write_string(out, string),
        Value::Array(items) => {
            out.push('[');
            for item in items {
                out.push(' ');
                write(out, item);
            }
            out.push_str(" ]");
        }
        Value::Object(entries) => {
            out.push('{');
            for (key, value) in entries {
                out.push(' ');
                write_key(out, key);
                out.push_str(" = ");
                write(out, value);
                out.push(';');
            }
            out.push_str(" }");
        }
    }
}

fn write_key(out: &mut String, key: &str) {
    if is_identifier(key) {
        out.push_str(key);
    } else {
        write_string(out, key);
    }
}

fn write_string(out: &mut String, string: &str) {
    out.push('"');
    let mut chars = string.chars().peekable();
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
    out.push('"');
}

fn is_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '\'' | '-'))
}

fn format_float(float: f64) -> String {
    if float.is_nan() {
        return "nan".to_string();
    }
    if float.is_infinite() {
        return if float.is_sign_positive() {
            "inf"
        } else {
            "-inf"
        }
        .to_string();
    }
    let rendered = format!("{float}");
    if rendered.chars().any(|c| matches!(c, '.' | 'e' | 'E')) {
        rendered
    } else {
        format!("{rendered}.0")
    }
}
