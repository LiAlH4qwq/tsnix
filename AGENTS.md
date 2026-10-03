# AGENTS.md

Instructions for AI agents working with or on `tsnix`.

## What this is

`tsnix` — short for **tiny snix-eval CLI** — evaluates the **Nix language** and
prints the result. It plugs into
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

# Interactive REPL (bindings persist; :commands)
tsnix repl

# Agent REPL: Lix repl-automation — ENQ (\x05) before each read, newline commands
tsnix repl --agent

# Agent REPL: ENQ readiness + newline-delimited JSON requests/responses
tsnix repl --protocol json

# Discover the full contract
tsnix schema

# Discover the standard library as JSON (present by default)
tsnix libdoc --pretty
```

## Standard library (`std`)

- Compiled in by default (cargo feature `stdlib`); `--no-default-features`
  removes it. `--no-default-features --features stdlib` keeps it.
- Always pure: no store, no I/O, deterministic. Safe under `builtins.toJSON`.
- Available as `std` (top-level binding) and `builtins.std`. A user `--arg std`
  overrides the binding; `builtins.std` remains.
- `with std;` works as a prelude (core list/attr builtins are re-exported).
- Naming is flat and cross-language (Clojure/Elixir/lodash/Haskell), not
  nixpkgs; `tsnix libdoc` records the closest nixpkgs `lib` name per function.
- The catalogue is generated from `src/stdlib.rs`; a unit test parses
  `src/stdlib.nix` and fails if the two drift.

## REPL contract

- `tsnix repl` (default): rustyline editor, `name = expr;` bindings, `:commands`
  (`:help`, `:quit`, `:type`, `:print`, `:doc`, `:load`, `:format`, `:mode`,
  `:bindings`, `:clear`). Not a TTY → plain line reader, no editor.
- `--agent` / `--automation`: before **every** read (including continuation),
  write one ENQ byte `0x05` to stdout and flush; read a newline-terminated
  command; EOF exits 0. This is Lix's `repl-automation` protocol verbatim.
- `--protocol json`: same ENQ readiness, then one JSON request per line and one
  JSON response per line. Request: `{"expr"|"command"|"input": ...}` (a command
  may carry `"arg"`). Response: `{"ok":true,"value"|"message"|"bound":...}` or
  `{"ok":false,"diagnostics":[...]}` (the schema below).
- REPL values render with `--format` (default `nix`, `<LAMBDA>` for functions).

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
src/program.rs      input reading, --arg/std prelude, output wrapper
src/diagnostic.rs   structured diagnostics + rendering
src/stubs.rs        Nix-source stubs for store/IO builtins
src/stdlib.nix      the pure-Nix `std` implementation (feature `stdlib`)
src/stdlib.rs       embedded source + catalogue (`libdoc`) + drift test
src/output.rs       JSON -> Nix printer
src/schema.rs       `tsnix schema` (embeds the `std` catalogue)
src/repl/           REPL engine + interacters (interactive/ENQ/JSON)
src/wasm.rs         wasm-bindgen scaffold (feature `wasm`)
src/cli.rs          clap CLI (binary-only)
tests/cli.rs        end-to-end tests
tests/repl.rs       REPL protocol tests
tests/stdlib.rs     `std` end-to-end tests
nix/                flake-parts modules (package, docs, hooks, github, wasm)
dev/                dev-only inputs + modules (docs theme, git hooks, GitHub
                    Actions generation); docs/`rustdoc` are built from the
                    `dev` partition, so `nix build .#docs` still works
docs/               bilingual mdBook sources (rose-pine-dawn mdBook theme)
.github/scripts/    CI helpers (update-snix.sh, used by the weekly bump)
```

### Constraints

- **License is GPL-3.0-only** (inherited from `snix-eval`). Keep it that way.
- The `snix-eval` git revision is pinned in `Cargo.toml`; changing it requires
  updating `Cargo.lock` and `nix/constants.nix` (`snixSourceHash`). The
  `outputHashes` keys are derived from `Cargo.lock`, so they need no editing.
- Do not add I/O or store behaviour outside `src/eval.rs` and `src/stubs.rs`.
- Priority is correctness > performance > readability > elegance. Prefer an
  accurate rejection over a convenient fabrication.

## Releases and automation

- **Weekly snix-eval bump.** `.github/workflows/update-snix.yml` (generated from
  `nix/github.nix`) runs Mondays 06:00 UTC. It follows the tip of the default
  branch (`canon`) of `https://git.snix.dev/snix/snix.git`, rewrites
  `Cargo.toml` + `Cargo.lock`, refreshes `snixSourceHash` in
  `nix/constants.nix`, self-tests with `nix build .#tsnix`, then opens/updates
  the `snix/update` PR via `.github/scripts/update-snix.sh`. Trigger it manually
  with `gh workflow run update-snix.yml`.
- Configure the repo secret `SNIX_UPDATE_TOKEN` (fine-grained PAT with
  *contents: write* and *pull requests: write*) so the bot PR triggers CI.
  Without it the fallback `github.token` still opens the PR, but GitHub
  suppresses `pull_request` runs for it; the workflow's own `nix build` still
  tests the bump.
- **Android / Termux.** `packages.<system>.android` and `packages.<system>.termux`
  are the fully static musl build (`.#static`): self-contained, no Termux prefix,
  runnable from a normal Android root shell (Magisk/KernelSU) and inside Termux.
- **Releases.** Pushing a `v*` tag runs `.github/workflows/release.yml`, which
  builds static musl `android` binaries (x86_64 + aarch64) plus `aarch64-darwin`
  and attaches them to the GitHub release.
- **Not publishable to crates.io.** `snix-eval` is a git-only dependency (the
  crates.io `snix-eval` is an empty `0.0.0-pre` placeholder), and Cargo refuses
  to package git dependencies for any registry. Distribute via GitHub Releases
  or `nix build .#android` instead.

### Versioning

`tsnix` follows [Semantic Versioning](https://semver.org) (`MAJOR.MINOR.PATCH`).
The version lives in `Cargo.toml` and is mirrored in `Cargo.lock`; keep both in
sync.

- `MAJOR`: breaking changes to the CLI contract, the embedding API, or the
  `std`/`schema` surface.
- `MINOR`: backwards-compatible additions (new subcommands, options, `std`
  functions, features — e.g. adding `std` moved 0.1.0 to 0.2.0).
- `PATCH`: backwards-compatible fixes and internal work.

Bump the version in the same commit that changes the main program: any change
to `src/` (or the CLI contract it implements) is a release and must update the
version. Docs-, tests- and packaging-only changes do not require a bump.
