# AGENTS.md

Instructions for AI agents working with or on `tsnix`.

## What this is

`tsnix` evaluates the **Nix language** and prints the result. It plugs into
`snix-eval` only and never touches a Nix store. It is the right tool when you
need to turn Nix expressions into JSON (for example to generate a config file)
but cannot rely on the `nix` binary or a store.

**Non-goals:** building derivations, fetching, writing store paths, a daemon, a
REPL, or emulating the `nix` CLI.

## Recipes

```console
# Expression -> JSON on stdout (default)
tsnix eval -e '{ a = 1; b = [ true null ]; }'

# File -> JSON, allowing local imports/reads
tsnix eval -f config.nix --io local

# Variable bindings (like nix --arg/--argstr)
tsnix eval -f config.nix --argstr host example.com --arg port 8080

# Validate only (fast; no evaluation)
tsnix check -f config.nix

# Discover the full contract
tsnix schema
```

## Hard contract

- **stdout** is the payload and nothing else. It is safe to parse directly.
- **stderr** carries diagnostics. With `--error-format json` it is one JSON
  document; otherwise human text.
- Exit codes: `0` success, `1` evaluation/compilation error, `2` usage error.
- On failure there is **no** payload on stdout.

Always pass `--error-format json` when you need to programmatically inspect
failures, and read the `code` field first.

### Diagnostics schema (stderr, `--error-format json`)

```json
{
  "version": 1,
  "ok": false,
  "diagnostics": [
    {
      "severity": "error",          // "error" | "warning"
      "code": "TSNIX-NO-STORE",     // stable; switch on this
      "kind": "CatchableError",     // underlying snix variant
      "message": "…",               // one-line message (bilingual for our stubs)
      "span": {                     // omitted if unknown
        "file": "config.nix",
        "start": { "line": 3, "column": 1 },
        "end": { "line": 3, "column": 12 }
      },
      "contexts": [],               // string contexts (usually empty here)
      "rendered": "error[TSNIX-NO-STORE]: …\n  --> config.nix:3:1\n…"
    }
  ]
}
```

When `ok` is true and there are warnings, the same document appears on stderr
with `"ok": true` and `severity: "warning"` entries.

### Stable error codes

| Code | Meaning | Typical fix |
| --- | --- | --- |
| `TSNIX-NO-STORE` | A store-dependent builtin was called (`derivation`, `fetchurl`, `writeFile`, `storePath`, `storeDir`, `toFile`, `path`, …) | Remove the store operation; pass data via `--arg`/`--argstr` or a local file. |
| `TSNIX-NO-IO` | Filesystem builtin used while `--io none` (default) | Re-run with `--io local`. |
| `TSNIX-NO-LOCAL-IO` | `--io local` on a `--no-default-features` build | Use a build with the `local-io` feature. |
| `TSNIX-EVAL` | An evaluation/compile error (type error, parse error, `throw`, …) | Inspect `rendered` / `message` and `span`. |
| `TSNIX-USAGE` | Invalid `--arg`/`--argstr` name | Use a valid Nix identifier. |
| `TSNIX-IO` | Could not read an input or write `-o` | Check paths/permissions. |
| `TSNIX-WARN` | Non-fatal warning | Optional. |

## Input rules

- Choose **exactly one** input: positional `SOURCE`, `-e/--expr`, or `-f/--file`.
  `-` means stdin; omitting all three also reads stdin.
- `--arg NAME EXPR` binds `NAME` to a **parsed Nix expression**.
  `--argstr NAME STRING` binds `NAME` to a **string**. Names must match
  `[A-Za-z_][A-Za-z0-9_'-]*`.
- Bindings are available as ordinary top-level variables inside the expression.

## Output rules

- `--format json` (default): `builtins.toJSON` of the value. Compact and exact.
  `--pretty` re-indents via `serde_json`.
- `--format raw`: `builtins.toString` of the value.
- `--format nix`: the JSON projection rendered as Nix source (functions rejected,
  paths become strings).
- Use `-o PATH` to write to a file. A trailing newline is appended.

## Store and I/O policy

- No store, ever. Store builtins raise `TSNIX-NO-STORE`. Do not try to work
  around this by faking paths.
- `--io local` reads the real filesystem (and `getEnv`), but still never a
  store. Relative path literals resolve against the evaluated file.
- `--io local` is required for `import ./other.nix`.

## Example: generate JSON from a split config

`examples/hoyofall/config.nix`:

```console
$ tsnix eval -f examples/hoyofall/config.nix --io local --pretty > config.json
```

## Working on the code

```console
$ cargo build
$ cargo test                 # includes a cross-check against real nix if present
$ cargo fmt
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo build --no-default-features   # pure build (no local I/O)
```

Repository layout:

```
src/lib.rs          public embedding API
src/eval.rs         build the evaluator, install stubs, run
src/program.rs      input reading, --arg prelude, output wrapper
src/diagnostic.rs   structured diagnostics + rendering
src/stubs.rs        Nix-source stubs for store/IO builtins
src/output.rs       JSON -> Nix printer
src/schema.rs       `tsnix schema`
src/cli.rs          clap CLI (binary-only)
tests/cli.rs        end-to-end tests
```

### Constraints

- **License is GPL-3.0-only** (inherited from `snix-eval`). Keep it that way.
- The `snix-eval` git revision is pinned in `Cargo.toml`; changing it requires
  updating `Cargo.lock` and the flake's `outputHashes`.
- Do not add I/O or store behaviour outside `src/eval.rs` and `src/stubs.rs`.
- Priority is correctness > performance > readability > elegance. Prefer an
  accurate rejection over a convenient fabrication.
