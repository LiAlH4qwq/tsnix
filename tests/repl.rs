//! Tests for the REPL, including the Lix-compatible ENQ protocol and the
//! newline-delimited JSON protocol.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
use serde_json::Value;

fn tsnix() -> Command {
    Command::new(cargo_bin("tsnix"))
}

fn run_repl(args: &[&str], input: &str) -> (String, String, i32) {
    let output = tsnix()
        .arg("repl")
        .args(args)
        .write_stdin(input.to_owned())
        .output()
        .expect("run tsnix repl");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

#[test]
fn interactive_piped_evaluates_and_persists_bindings() {
    let (stdout, _stderr, code) = run_repl(
        &[],
        "1 + 1\nx = 41;\nx + 1\n{ a = 1; b = [ 2 3 ]; }\n:quit\n",
    );
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines[0], "2");
    assert_eq!(lines[1], "42");
    assert_eq!(lines[2], "{ a = 1; b = [ 2 3 ]; }");
}

#[test]
fn multiline_input_is_buffered() {
    let (stdout, _stderr, code) = run_repl(&[], "{\n  a = 1;\n  b = 2;\n}\n");
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "{ a = 1; b = 2; }");
}

#[test]
fn agent_protocol_emits_enq_readiness() {
    let (stdout, _stderr, code) = run_repl(&["--agent"], "1 + 1\n:quit\n");
    assert_eq!(code, 0);
    let bytes = stdout.as_bytes();
    // Before the first read.
    assert_eq!(bytes.first(), Some(&0x05));
    assert!(stdout.contains("2\n"), "missing value: {stdout:?}");
    assert!(
        stdout.ends_with('\u{5}'),
        "missing trailing ENQ: {stdout:?}"
    );
}

#[test]
fn empty_agent_input_exits_cleanly() {
    let (stdout, _stderr, code) = run_repl(&["--agent"], "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "\u{5}");
}

#[test]
fn json_protocol_round_trips() {
    let input = concat!(
        "{\"expr\":\"1 + 1\"}\n",
        "{\"command\":\"type\",\"arg\":\"builtins.map\"}\n",
        "{\"expr\":\"{ a = 1; }\"}\n",
    );
    let (stdout, _stderr, code) = run_repl(&["--protocol", "json"], input);
    assert_eq!(code, 0);

    let responses: Vec<Value> = stdout
        .lines()
        .map(|line| line.trim_start_matches('\u{5}').trim())
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect("valid JSON response"))
        .collect();

    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0]["value"], Value::from("2"));
    assert_eq!(responses[0]["format"], Value::from("nix"));
    assert!(responses[1]["message"].as_str().unwrap().contains("lambda"));
    assert_eq!(responses[2]["value"], Value::from("{ a = 1; }"));
}

#[test]
fn json_protocol_reports_errors() {
    let (stdout, _stderr, code) =
        run_repl(&["--protocol", "json"], "{\"expr\":\"derivation {}\"}\n");
    assert_eq!(code, 0);
    let line = stdout
        .lines()
        .map(|line| line.trim_start_matches('\u{5}').trim())
        .rfind(|line| !line.is_empty())
        .unwrap();
    let response: Value = serde_json::from_str(line).unwrap();
    assert_eq!(response["ok"], Value::Bool(false));
    assert_eq!(
        response["diagnostics"][0]["code"],
        Value::from("TSNIX-NO-STORE")
    );
}

#[test]
fn store_builtins_are_rejected_in_repl() {
    let (_stdout, stderr, code) = run_repl(&[], "derivation { name = \"x\"; }\n:quit\n");
    assert_eq!(code, 0);
    assert!(
        stderr.contains("TSNIX-NO-STORE"),
        "expected TSNIX-NO-STORE, got: {stderr}"
    );
}
