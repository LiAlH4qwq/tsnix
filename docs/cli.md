# CLI reference

`tsnix schema` is the authoritative, machine-readable version of this document.

## Synopsis

```
tsnix <COMMAND>

Commands:
  eval    Evaluate a Nix expression or file
  check   Parse and compile without evaluating
  repl    Start a Nix-language REPL (friendly or agent mode)
  schema  Print a machine-readable description of this CLI
  libdoc  Print the `std` standard library catalogue as JSON
```

## Common options (`eval` and `check`)

| Option | Description |
| --- | --- |
| `[SOURCE]` | Path to a `.nix` file, or `-` for stdin. |
| `-e, --expr <EXPR>` | Evaluate an inline expression. |
| `-f, --file <PATH>` | Evaluate a file. |
| `--arg <NAME> <EXPR>` | Bind `NAME` to a parsed Nix expression. Repeatable. |
| `--argstr <NAME> <STRING>` | Bind `NAME` to a string. Repeatable. |
| `--io <none\|local>` | Allow local file I/O. Default `none`. |
| `--error-format <auto\|text\|json>` | Diagnostics format. Default `auto` (text on a TTY, otherwise JSON). |
| `-q, --quiet` | Suppress warnings. |

Exactly one input must be selected. `-` (or no input at all) reads stdin.
`--arg`/`--argstr` names must be valid Nix identifiers
(`[A-Za-z_][A-Za-z0-9_'-]*`).

## `eval` only

| Option | Description |
| --- | --- |
| `-F, --format <json\|raw\|nix>` | Output format. Default `json`. |
| `--pretty` | Re-indent JSON output with `serde_json` (best effort). |
| `--mode <strict\|lazy>` | Top-level forcing. Default `strict`. |
| `--nix-path <SPEC>` | `NIX_PATH` used for `<...>` lookups (requires `--io local`). |
| `-o, --output <PATH>` | Write the result to a file instead of stdout. |

## `schema`

| Option | Description |
| --- | --- |
| `--pretty` | Pretty-print the schema. |

## `libdoc`

Prints the `std` standard library catalogue as JSON: every function's name,
signature, description, group and closest nixpkgs `lib` name, plus the list of
re-exported prelude builtins. It is present only when the `stdlib` feature is
compiled in (the default).

| Option | Description |
| --- | --- |
| `--pretty` | Pretty-print the catalogue. |

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | success |
| `1` | evaluation or compilation error |
| `2` | usage error (unknown flags, missing values, …) |

## Examples

```console
$ tsnix eval -e '{ a = 1; }'
{"a":1}

$ tsnix eval -e '{ a = 1; }' --pretty
{
  "a": 1
}

$ tsnix eval -f examples/hoyofall/config.nix --io local -o config.json

$ tsnix check -f config.nix --error-format json
```
