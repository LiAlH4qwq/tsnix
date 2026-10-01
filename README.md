# tsnix

A minimal, embeddable **Nix-language evaluator** built solely on
[`snix-eval`](https://snix.dev). It contains **no Nix store**: it evaluates the
Nix language and emits JSON/raw/Nix output, so you can generate configuration
where the `nix` binary is unavailable or too heavy — for example an Android
Magisk/KernelSU module or an embedded device.

`tsnix` is deliberately **not** a `nix` clone. It has a small, modern,
agent-friendly interface:

- **stdout is the payload** (JSON by default), nothing else.
- **stderr is diagnostics** (text or structured JSON).
- **stable machine-readable error codes** such as `TSNIX-NO-STORE`.
- a `tsnix schema` command that prints the whole contract as JSON.

- English | [简体中文](README.zh-CN.md)
- Agent-oriented guide: [AGENTS.md](AGENTS.md)

## Why

`snix-eval` is a fast, modular Rust implementation of Nix evaluation. Because
it is library-first, we can embed just the evaluator and skip the entire store,
daemon and build machinery. That makes `tsnix` tiny enough for embedded targets
while remaining compatible with Nix's evaluation semantics (verified against
real `nix` in the test suite).

Priority order: **correctness > performance > readability > elegance**. Store
operations are therefore never faked — they are rejected with a clear error.

## What is supported

| Area | Support |
| --- | --- |
| Pure Nix expressions, attrsets, lists, functions, `let`, `with`, `rec`, `inherit`, string interpolation, `builtins.*` data functions | ✅ |
| `--format json` (via `builtins.toJSON`) | ✅ exact Nix behaviour |
| `--format raw` (via `builtins.toString`) | ✅ |
| `--format nix` (best-effort pretty printer) | ✅ (paths become strings, functions rejected) |
| `--arg` / `--argstr` top-level bindings | ✅ |
| `std` standard library (pure Nix; merge, paths, lists, strings, case conversion) | ✅ default; off with `--no-default-features` |
| Local file I/O: `import`, `readFile`, `readDir`, `pathExists`, `getEnv`, … (`--io local`) | ✅ |
| REPL: friendly (`nix repl`-style bindings, `:commands`) and agent modes (Lix ENQ, JSONL) | ✅ |
| `derivation`, `fetchurl`, `writeFile`, `storePath`, `storeDir`, `toFile`, … | ❌ rejected (`TSNIX-NO-STORE`) |

The complete rejected/optional list is in `tsnix schema`.

## Install

### Cargo

```console
$ cargo build --release          # or: cargo install --path .
$ ./target/release/tsnix --help
```

> `snix-eval` is not published on crates.io yet, so `tsnix` depends on a pinned
> git revision. See `Cargo.toml`.

### Nix

```console
$ nix build                 # host binary
$ nix build .#static        # static musl binary for embedded targets
$ nix build .#docs          # bilingual documentation site (mdBook)
$ nix build .#wasm          # wasm32-wasip1 build (experimental)
$ nix run . -- eval -e '1 + 1'
$ nix develop               # dev shell with the pinned toolchain, hooks and mdBook
```

## Usage

```console
# JSON by default
$ tsnix eval -e '{ a = 1; b = [ true null ]; }'
{"a":1,"b":[true,null]}

# Pretty-print
$ tsnix eval -e '{ a = 1; }' --pretty
{
  "a": 1
}

# Evaluate a file, importing other local files
$ tsnix eval -f config.nix --io local

# Bind top-level variables like nix --arg/--argstr
$ tsnix eval -e '{ url = "http://${host}:${toString port}"; }' \
    --argstr host example.com --arg port 8080
{"url":"http://example.com:8080"}

# Raw string output
$ tsnix eval -e '"hello ${builtins.currentSystem}"' --format raw

# Nix pretty printing
$ tsnix eval -e '{ a = 1; b = [ 1 2 ]; }' --format nix
{ a = 1; b = [ 1 2 ]; }

# The `std` standard library (pure Nix, no store, no I/O)
$ tsnix eval -e 'std.merge { a = { x = 1; }; } { a = { y = 2; }; }'
{"a":{"x":1,"y":2}}
$ tsnix eval -e 'std.toSnakeCase "HTTPServerConfig"'
"http_server_config"

# Discover the library as JSON
$ tsnix libdoc --pretty

# Read from stdin
$ echo '{ a = 1; }' | tsnix eval -

# Validate without evaluating
$ tsnix check -f config.nix

# Interactive REPL (bindings, :commands, history)
$ tsnix repl
tsnix> x = 21;
tsnix> x * 2
42

# Agent-friendly REPL: Lix repl-automation (ENQ readiness) or JSON lines
$ printf '1 + 1\n:quit\n' | tsnix repl --agent
$ tsnix repl --protocol json

# The machine-readable contract
$ tsnix schema --pretty
```

## The CLI contract

| | |
| --- | --- |
| stdout | the result in `--format` (nothing else) |
| stderr | diagnostics: `--error-format text` (default on a TTY) or `json` |
| exit `0` | success |
| exit `1` | evaluation/compilation error |
| exit `2` | usage error (clap) |

Input sources are mutually exclusive: a positional `SOURCE`, `-e/--expr`, or
`-f/--file` (use `-` for stdin; omitting everything also reads stdin).

Output formatting:

- `json` — `builtins.toJSON` of the value (compact, exact). `--pretty` asks
  `serde_json` to re-indent it.
- `raw` — `builtins.toString` of the value.
- `nix` — renders the JSON projection as Nix source.

## I/O and store policy

By default `tsnix` is **pure**: `import`, `readFile`, `readDir`, `pathExists`,
`getEnv`, `hashFile` and `currentTime` raise `TSNIX-NO-IO`. Pass `--io local` to
enable local filesystem access through `snix-eval`'s `StdIO` (relative imports
resolve against the source file). No I/O ever touches a store path.

Store-dependent builtins are **always** rejected with `TSNIX-NO-STORE` and a
bilingual message, for example:

```console
$ tsnix eval -e 'derivation { name = "x"; }'
error[TSNIX-NO-STORE]: tsnix::no_store: `derivation` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store，无法构建 derivation。)
  --> <expr>:1:1
  |
 1 | derivation { name = "x"; }
  | ^~~~~~~~~~~~~~~~~~~~~~~~~~
```

## Diagnostics

`--error-format json` emits a stable document on stderr:

```json
{
  "version": 1,
  "ok": false,
  "diagnostics": [
    {
      "severity": "error",
      "code": "TSNIX-NO-STORE",
      "kind": "CatchableError",
      "message": "tsnix::no_store: …",
      "span": {
        "file": "config.nix",
        "start": { "line": 3, "column": 1 },
        "end": { "line": 3, "column": 12 }
      },
      "rendered": "error[TSNIX-NO-STORE]: …\n  --> config.nix:3:1\n…"
    }
  ]
}
```

Error codes are stable: `TSNIX-NO-STORE`, `TSNIX-NO-IO`, `TSNIX-NO-LOCAL-IO`,
`TSNIX-EVAL`, `TSNIX-USAGE`, `TSNIX-IO`, `TSNIX-WARN`. See
[docs/errors.md](docs/errors.md).

## Embedding

`tsnix` is a library plus a thin CLI:

```rust
use tsnix::{evaluate, EvalOptions, Format, IoMode, Mode, Source};

let output = evaluate(&EvalOptions {
    source: Source::Expr("{ a = 1; }".to_string()),
    format: Format::Json,
    io: IoMode::None,
    mode: Mode::Strict,
    args: Vec::new(),
    pretty: false,
    nix_path: None,
})?;
println!("{}", output.text);
# Ok::<(), tsnix::EvalError>(())
```

See [docs/architecture.md](docs/architecture.md) for the design.

## Building for embedded / wasm

- `--no-default-features` removes local I/O (`snix-eval`'s `impure` feature),
  the REPL and `std`, building a pure evaluator with no filesystem dependency;
  `--io local` then reports `TSNIX-NO-LOCAL-IO`.
- `--no-default-features --features stdlib` keeps the pure evaluator and `std`
  (the library has no I/O dependency).
- `no-leak` forwards `snix-eval/no_leak` to reduce peak memory at some speed
  cost.
- The release profile uses `lto = "fat"`, `codegen-units = 1`, `opt-level = "s"`
  and `strip`.

Static musl builds are produced by the flake (`packages.<system>.static`).

## Testing

```console
$ cargo test
```

The suite includes a golden cross-check that runs the same expressions through
real `nix eval --json` when `nix` is available.

## License

GPL-3.0-only, matching `snix-eval`. See [LICENSE](LICENSE).
