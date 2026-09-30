//! End-to-end tests for the `tsnix` CLI.
//!
//! These exercise the public contract: stdout is the payload, stderr carries
//! diagnostics, exit codes are stable, and store access is rejected.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use serde_json::Value;

fn tsnix() -> Command {
    Command::new(cargo_bin("tsnix"))
}

fn eval_json(expr: &str) -> Value {
    let output = tsnix()
        .args(["eval", "-e", expr, "--error-format", "json"])
        .output()
        .expect("run tsnix");
    assert!(
        output.status.success(),
        "tsnix failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

#[test]
fn json_scalars_and_containers() {
    assert_eq!(eval_json("null"), Value::Null);
    assert_eq!(eval_json("true"), Value::Bool(true));
    assert_eq!(eval_json("42"), Value::from(42));
    assert_eq!(eval_json("-1.5"), Value::from(-1.5));
    assert_eq!(eval_json(r#""x""#), Value::from("x"));
    assert_eq!(eval_json("[ 1 2 3 ]"), serde_json::json!([1, 2, 3]));
    assert_eq!(
        eval_json("{ b = 2; a = 1; }"),
        serde_json::json!({ "a": 1, "b": 2 })
    );
}

#[test]
fn string_escapes_round_trip() {
    assert_eq!(eval_json(r#""a\nb\t\"c\"""#), Value::from("a\nb\t\"c\""));
}

#[test]
fn pretty_output_is_multiline() {
    tsnix()
        .args(["eval", "-e", "{ a = 1; }", "--pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n"));
}

#[test]
fn raw_format_coerces_to_string() {
    tsnix()
        .args(["eval", "-e", "builtins.toString 42", "-F", "raw"])
        .assert()
        .success()
        .stdout("42\n");
}

#[test]
fn nix_format_round_trips() {
    tsnix()
        .args([
            "eval",
            "-e",
            "{ a = 1; b = [ 1 2 ]; s = \"x\"; }",
            "-F",
            "nix",
        ])
        .assert()
        .success()
        .stdout("{ a = 1; b = [ 1 2 ]; s = \"x\"; }\n");
}

#[test]
fn arg_and_argstr_bind_globals() {
    let output = tsnix()
        .args([
            "eval",
            "-e",
            "{ host = h; n = v + 1; }",
            "--arg",
            "v",
            "41",
            "--argstr",
            "h",
            "example.com",
        ])
        .output()
        .expect("run tsnix");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json, serde_json::json!({ "host": "example.com", "n": 42 }));
}

#[test]
fn invalid_arg_name_is_rejected() {
    tsnix()
        .args(["eval", "-e", "1", "--argstr", "not valid", "x"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("TSNIX-USAGE"));
}

fn diagnostic_code(stderr: &[u8]) -> String {
    let json: Value = serde_json::from_slice(stderr).expect("stderr is JSON diagnostics");
    json["diagnostics"][0]["code"].as_str().unwrap().to_string()
}

#[test]
fn store_builtins_are_rejected() {
    for expr in [
        "derivation { name = \"x\"; }",
        "builtins.fetchurl \"https://example.com\"",
        "builtins.toFile \"x\" \"y\"",
        "builtins.storePath \"/nix/store/x\"",
        "builtins.storeDir",
        "builtins.writeFile \"x\" \"y\"",
    ] {
        let output = tsnix()
            .args(["eval", "-e", expr, "--error-format", "json"])
            .output()
            .expect("run tsnix");
        assert!(!output.status.success(), "`{expr}` should fail");
        assert_eq!(
            diagnostic_code(&output.stderr),
            "TSNIX-NO-STORE",
            "unexpected code for `{expr}`"
        );
    }
}

#[test]
fn io_is_disabled_by_default() {
    let output = tsnix()
        .args([
            "eval",
            "-e",
            "builtins.readFile ./Cargo.toml",
            "--error-format",
            "json",
        ])
        .output()
        .expect("run tsnix");
    assert!(!output.status.success());
    assert_eq!(diagnostic_code(&output.stderr), "TSNIX-NO-IO");
}

#[test]
fn io_local_allows_reads_and_imports() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("value.nix");
    std::fs::write(&file, "{ a = 1; }").unwrap();

    tsnix()
        .args([
            "eval",
            "-e",
            &format!("import {}", file.display()),
            "--io",
            "local",
        ])
        .assert()
        .success()
        .stdout(r#"{"a":1}"#.to_owned() + "\n");

    tsnix()
        .args([
            "eval",
            "-e",
            &format!("builtins.readFile {}", file.display()),
            "--io",
            "local",
            "-F",
            "raw",
        ])
        .assert()
        .success()
        .stdout("{ a = 1; }\n");
}

#[test]
fn check_reports_parse_errors_and_succeeds_otherwise() {
    tsnix()
        .args(["check", "-e", "let x = 1; in x"])
        .assert()
        .success();

    let output = tsnix()
        .args(["check", "-e", "let x = ; in x", "--error-format", "json"])
        .output()
        .expect("run tsnix");
    assert!(!output.status.success());
    assert_eq!(diagnostic_code(&output.stderr), "TSNIX-EVAL");
}

#[test]
fn stdin_is_supported() {
    tsnix()
        .args(["eval", "-"])
        .write_stdin("{ a = 1; }")
        .assert()
        .success()
        .stdout(r#"{"a":1}"#.to_owned() + "\n");
}

#[test]
fn output_file_flag() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.json");
    tsnix()
        .args(["eval", "-e", "{ a = 1; }", "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "{\"a\":1}\n");
}

#[test]
fn schema_is_valid_json_with_commands() {
    let output = tsnix().args(["schema"]).output().expect("run tsnix");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("schema is JSON");
    let commands = json["commands"].as_object().unwrap();
    for name in ["eval", "check", "schema"] {
        assert!(commands.contains_key(name), "missing command {name}");
    }
    assert_eq!(json["store"]["available"], Value::Bool(false));
}

/// Optional cross-check against real Nix, when available.
#[test]
fn matches_nix_json_when_available() {
    if std::process::Command::new("nix")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipping: `nix` not available");
        return;
    }
    for expr in [
        "{ a = 1; b = [ true null ]; }",
        "builtins.map (x: x * 2) [ 1 2 3 ]",
        "let f = x: { y = x; }; in f 7",
    ] {
        let nix = std::process::Command::new("nix")
            .args(["eval", "--json", "--expr", expr])
            .output()
            .expect("run nix");
        assert!(nix.status.success(), "nix failed for {expr}");
        let expected: Value = serde_json::from_slice(&nix.stdout).unwrap();
        let actual = eval_json(expr);
        assert_eq!(actual, expected, "mismatch for {expr}");
    }
}
