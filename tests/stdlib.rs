//! End-to-end tests for the `std` standard library and `tsnix libdoc`.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin;
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
    assert!(
        output.stderr.is_empty(),
        "unexpected diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

#[test]
fn std_is_available_bare_and_qualified() {
    assert_eq!(eval_json("std.id 7"), Value::from(7));
    assert_eq!(eval_json("builtins.std.id 7"), Value::from(7));
    assert_eq!(
        eval_json("with std; map (x: x + 1) [ 1 2 ]"),
        serde_json::json!([2, 3])
    );
}

#[test]
fn deep_merge_and_defaults() {
    assert_eq!(
        eval_json("std.merge { a = { x = 1; y = 2; }; b = 1; } { a = { y = 9; }; c = 3; }"),
        serde_json::json!({ "a": { "x": 1, "y": 9 }, "b": 1, "c": 3 })
    );
    assert_eq!(
        eval_json("std.defaults { a = 1; b = 2; } { b = 20; c = 30; }"),
        serde_json::json!({ "a": 1, "b": 20, "c": 30 })
    );
}

#[test]
fn path_helpers_accept_dotted_strings_and_lists() {
    assert_eq!(
        eval_json("std.getIn \"a.b\" { a = { b = 5; }; }"),
        Value::from(5)
    );
    assert_eq!(
        eval_json("std.getInOr 9 \"a.x\" { a = { b = 5; }; }"),
        Value::from(9)
    );
    assert_eq!(
        eval_json("std.assocIn \"a.b\" 7 { a = { c = 1; }; }"),
        serde_json::json!({ "a": { "c": 1, "b": 7 } })
    );
    assert_eq!(
        eval_json("std.removeIn \"a.b\" { a = { b = 1; c = 2; }; }"),
        serde_json::json!({ "a": { "c": 2 } })
    );
    assert_eq!(
        eval_json("std.selectKeys [ \"a.b\" ] { a = { b = 1; c = 2; }; d = 3; }"),
        serde_json::json!({ "a": { "b": 1 } })
    );
}

#[test]
fn map_entries_changes_name_and_value() {
    assert_eq!(
        eval_json("std.mapEntries (n: v: { name = \"${n}!\"; value = v + 1; }) { a = 1; b = 2; }"),
        serde_json::json!({ "a!": 2, "b!": 3 })
    );
    assert_eq!(
        eval_json("std.mapKeys (n: n + \"x\") { a = 1; }"),
        serde_json::json!({ "ax": 1 })
    );
}

#[test]
fn list_and_conditional_helpers() {
    assert_eq!(
        eval_json("std.compact [ 1 null 2 ]"),
        serde_json::json!([1, 2])
    );
    assert_eq!(
        eval_json("std.unique [ 1 2 1 3 ]"),
        serde_json::json!([1, 2, 3])
    );
    assert_eq!(eval_json("std.range 1 4"), serde_json::json!([1, 2, 3, 4]));
    assert_eq!(eval_json("std.sum [ 1 2 3 ]"), Value::from(6));
    assert_eq!(eval_json("std.optional true 5"), serde_json::json!([5]));
    assert_eq!(eval_json("std.optional false 5"), serde_json::json!([]));
    assert_eq!(
        eval_json("std.enumerate [ \"a\" ]"),
        serde_json::json!([{ "index": 0, "value": "a" }])
    );
}

#[test]
fn string_helpers_and_case_conversion() {
    assert_eq!(eval_json("std.trim \"  hi  \""), Value::from("hi"));
    assert_eq!(
        eval_json("std.hasPrefix \"foo\" \"foobar\""),
        Value::Bool(true)
    );
    assert_eq!(
        eval_json("std.splitString \".\" \"a.b.c\""),
        serde_json::json!(["a", "b", "c"])
    );
    assert_eq!(
        eval_json("std.join \"-\" [ \"a\" 1 true ]"),
        Value::from("a-1-1")
    );
    assert_eq!(
        eval_json("std.toSnakeCase \"HTTPServerConfig\""),
        Value::from("http_server_config")
    );
    assert_eq!(
        eval_json("std.toCamelCase \"foo_bar\""),
        Value::from("fooBar")
    );
    assert_eq!(
        eval_json("std.toKebabCase \"fooBar\""),
        Value::from("foo-bar")
    );
    assert_eq!(eval_json("std.toIntOr 0 \"007\""), Value::from(7));
    assert_eq!(eval_json("std.toIntOr 0 \"nope\""), Value::from(0));
}

#[test]
fn pipe_composes_transformations() {
    assert_eq!(
        eval_json("std.pipe { a = 1; } [ (x: x // { b = 2; }) (x: std.assocIn \"c\" 3 x) ]"),
        serde_json::json!({ "a": 1, "b": 2, "c": 3 })
    );
}

#[test]
fn user_binding_overrides_injected_std() {
    let output = tsnix()
        .args(["eval", "-e", "std", "--arg", "std", "5"])
        .output()
        .expect("run tsnix");
    assert!(
        output.status.success(),
        "tsnix failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap(), 5);
}

#[test]
fn libdoc_lists_functions() {
    let output = tsnix().args(["libdoc"]).output().expect("run tsnix");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("libdoc is JSON");
    let functions = json["functions"].as_array().expect("functions array");
    assert!(functions.len() > 40, "expected a broad catalogue");
    for name in ["merge", "mapEntries", "getIn", "toSnakeCase", "compact"] {
        assert!(
            functions
                .iter()
                .any(|function| function["name"].as_str() == Some(name)),
            "libdoc is missing `{name}`"
        );
    }
    let prelude = json["prelude"].as_array().unwrap();
    assert!(prelude.contains(&Value::from("map")));
}

#[test]
fn schema_advertises_stdlib() {
    let output = tsnix().args(["schema"]).output().expect("run tsnix");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("schema is JSON");
    assert_eq!(json["stdlib"]["available"], Value::Bool(true));
    assert_eq!(json["stdlib"]["namespace"], Value::from("std"));
}
