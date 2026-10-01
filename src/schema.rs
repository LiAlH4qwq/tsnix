//! A machine-readable description of the CLI, emitted by `tsnix schema`.
//!
//! This is the contract agents should rely on. It is intentionally static and
//! covered by a test that checks it does not drift from the clap definition.

use serde_json::{Value, json};

/// Return the CLI schema as a JSON value.
pub fn schema() -> Value {
    #[allow(unused_mut)]
    let mut value = json!({
        "name": "tsnix",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "A minimal Nix-language evaluator built solely on snix-eval. No Nix store.",
        "stdout": "The evaluation result, in the selected --format. Nothing else.",
        "stderr": "Diagnostics (warnings and errors), in --error-format.",
        "store": {
            "available": false,
            "policy": "Store-dependent builtins are replaced by stubs that raise a stable nix error.",
            "rejected": [
                "derivation", "derivationStrict", "fetchurl", "fetchTarball", "fetchGit",
                "fetchMercurial", "fetchTree", "writeFile", "toFile", "storePath", "storeDir",
                "path", "filterSource", "placeholder", "scopedImport"
            ],
            "io_disabled_by_default": [
                "import", "readFile", "readDir", "readFileType", "pathExists", "getEnv",
                "hashFile", "currentTime"
            ]
        },
        "exit_codes": {
            "0": "success",
            "1": "evaluation or compilation error",
            "2": "usage error (bad flags, invalid --arg name)"
        },
        "error_schema": {
            "version": 1,
            "shape": {
                "version": "int",
                "ok": "bool",
                "diagnostics": [{
                    "severity": "error | warning",
                    "code": "stable machine-readable code, e.g. TSNIX-NO-STORE",
                    "kind": "snix error variant, e.g. TypeError",
                    "message": "one-line human-readable message",
                    "span": {
                        "file": "string (optional)",
                        "start": { "line": "int, 1-based", "column": "int, 1-based" },
                        "end": { "line": "int", "column": "int" }
                    },
                    "contexts": ["string context elements (optional)"],
                    "rendered": "full colourless rendering"
                }]
            }
        },
        "formats": {
            "json": "builtins.toJSON of the value (exact Nix behaviour).",
            "raw": "builtins.toString of the value.",
            "nix": "best-effort Nix pretty printer; functions are rejected."
        },
        "commands": {
            "eval": {
                "about": "Evaluate a Nix expression or file.",
                "args": [
                    { "name": "[SOURCE]", "about": "Path to a .nix file, or `-` for stdin." },
                    { "name": "-e, --expr <EXPR>", "about": "Evaluate an inline expression." },
                    { "name": "-f, --file <PATH>", "about": "Evaluate a file." },
                    { "name": "--arg <NAME> <NIX_EXPR>", "about": "Bind a name to a parsed Nix expression. Repeatable." },
                    { "name": "--argstr <NAME> <STRING>", "about": "Bind a name to a string. Repeatable." },
                    { "name": "-F, --format <json|raw|nix>", "about": "Output format. Default: json." },
                    { "name": "--pretty", "about": "Pretty-print JSON (best effort)." },
                    { "name": "--mode <strict|lazy>", "about": "Top-level forcing mode. Default: strict." },
                    { "name": "--io <none|local>", "about": "Allow local file I/O. Default: none." },
                    { "name": "--nix-path <SPEC>", "about": "NIX_PATH for <...> lookups (with --io local)." },
                    { "name": "-o, --output <PATH>", "about": "Write the result to a file instead of stdout." },
                    { "name": "--error-format <text|json>", "about": "Diagnostics format. Default: text on a TTY, else json." }
                ]
            },
            "check": {
                "about": "Parse and compile without evaluating.",
                "args": [
                    { "name": "[SOURCE]", "about": "Path to a .nix file, or `-` for stdin." },
                    { "name": "--arg <NAME> <NIX_EXPR>", "about": "Bind a name to a parsed Nix expression." },
                    { "name": "--argstr <NAME> <STRING>", "about": "Bind a name to a string." },
                    { "name": "--io <none|local>", "about": "Allow local file I/O. Default: none." },
                    { "name": "--error-format <text|json>", "about": "Diagnostics format." }
                ]
            },
            "repl": {
                "about": "Start a Nix-language REPL (friendly or agent mode).",
                "args": [
                    { "name": "--protocol <interactive|enq|json>", "about": "Input/output protocol. Default: interactive." },
                    { "name": "--agent", "about": "Shorthand for --protocol enq (Lix repl-automation: ENQ readiness byte)." },
                    { "name": "-F, --format <json|raw|nix>", "about": "Display format. Default: nix." },
                    { "name": "--mode <strict|lazy>", "about": "Forcing mode. Default: strict." },
                    { "name": "-l, --load <PATH>", "about": "Load a file on startup. Repeatable." },
                    { "name": "--arg/--argstr", "about": "Seed bindings." },
                    { "name": "--io <none|local>", "about": "Allow local file I/O. Default: none." }
                ],
                "protocols": {
                    "interactive": "rustyline line editor with history and :commands.",
                    "enq": "Before each read, write ENQ (U+0005) and flush; read a newline-terminated line; EOF exits. Continuation reads emit ENQ again.",
                    "json": "ENQ readiness, then newline-delimited JSON: request {\"expr\"|\"command\"|\"input\"}, response {\"ok\",\"value\"|\"diagnostics\"}."
                },
                "commands": [ ":help/?", ":quit/q", ":type/t", ":print/p", ":doc", ":load/l", ":format", ":mode", ":bindings", ":clear" ]
            },
            "schema": {
                "about": "Print this document.",
                "args": [{ "name": "--pretty", "about": "Pretty-print the schema." }]
            }
        },
        "examples": [
            { "about": "Evaluate an inline expression to JSON", "run": "tsnix eval -e '{ a = 1; b = [ true null ]; }'" },
            { "about": "Evaluate a file and pretty-print", "run": "tsnix eval -f config.nix --pretty" },
            { "about": "Bind a string and import local files", "run": "tsnix eval -f config.nix --io local --argstr host example.com" },
            { "about": "Validate only", "run": "tsnix check -f config.nix" }
        ]
    });

    #[cfg(feature = "stdlib")]
    {
        value["commands"]["libdoc"] = json!({
            "about": "Print the `std` standard library catalogue as JSON.",
            "args": [{ "name": "--pretty", "about": "Pretty-print the catalogue." }]
        });
        value["stdlib"] = stdlib_section();
    }

    value
}

/// The `stdlib` section of the schema (present only when compiled in).
#[cfg(feature = "stdlib")]
fn stdlib_section() -> Value {
    json!({
        "available": true,
        "namespace": "std",
        "bindings": {
            "global": "std",
            "builtin": "builtins.std",
            "note": "`std` is injected as a top-level binding unless a user `--arg std` overrides it."
        },
        "prelude": crate::stdlib::PRELUDE,
        "functions": crate::stdlib::FUNCTIONS.iter().map(|function| json!({
            "name": function.name,
            "signature": function.signature,
            "about": function.about,
            "group": function.group,
            "nixpkgs": function.nixpkgs,
        })).collect::<Vec<_>>(),
    })
}

/// Return the catalogue of `std` functions as a JSON value.
///
/// This is the machine-readable contract behind `tsnix libdoc`.
#[cfg(feature = "stdlib")]
pub fn stdlib_catalogue() -> Value {
    let groups: Vec<&str> = {
        let mut groups: Vec<&str> = crate::stdlib::FUNCTIONS
            .iter()
            .map(|function| function.group)
            .collect();
        groups.dedup();
        groups
    };

    json!({
        "version": 1,
        "name": "std",
        "about": "The tsnix standard library: pure Nix, no store, no I/O.",
        "usage": "Available as `std` and `builtins.std`. Use `with std;` as a prelude.",
        "groups": groups,
        "prelude": crate::stdlib::PRELUDE,
        "functions": crate::stdlib::FUNCTIONS.iter().map(|function| json!({
            "name": function.name,
            "signature": function.signature,
            "about": function.about,
            "group": function.group,
            "nixpkgs": function.nixpkgs,
        })).collect::<Vec<_>>(),
    })
}
