# Architecture

`tsnix` is a thin, opinionated shell around `snix-eval`. Everything store- or
I/O-related is either rejected or opt-in.

```
   CLI (src/cli.rs, clap)
        |
        v
   lib API (src/lib.rs)
        |
   eval.rs  --- program.rs (input, --arg prelude, wrapper)
        |   \-- stubs.rs   (store/IO rejection as Nix source)
        |   \-- output.rs  (JSON -> Nix printer)
        v
   snix-eval (git, pinned)
```

## Evaluation pipeline

1. **Read** the input (`Source::Expr` / `File` / `Stdin`) into a string, and pick
   a `location` used for relative-path resolution and error reporting.
2. **Compose** (`program.rs`) a program:
   - an optional `let …; in` prelude for `--arg`/`--argstr` bindings, then
   - a wrapper:
     - `builtins.toJSON (…)` for `json` and `nix`,
     - `builtins.toString (…)` for `raw`,
     - `(…)` for `check`.
3. **Build** an evaluator with `Evaluation::builder_pure()` and install stubs.
4. **Evaluate** in `EvalMode::Strict` (deep-forces the result) or `Lazy`.
5. **Render** the resulting `Value` to a string, or convert errors to
   diagnostics.

### Why `builtins.toJSON` instead of `Value::into_contextful_json`

`into_contextful_json` is a generator that requires driving the VM's private
request loop, so it is not usable from downstream crates. `builtins.toJSON` uses
exactly that logic internally and returns a plain Nix string, so wrapping the
user expression in `builtins.toJSON` gives byte-for-byte Nix-compatible JSON
through the public API. `--format nix` reuses the same JSON projection and then
prints it as Nix, which avoids unforceable thunks and keeps semantics uniform.

### Wrapper and spans

The synthesised prelude and wrapper put the user's code after a newline, so user
line numbers are shifted by a known `line_offset` and columns are unaffected.
`diagnostic.rs` subtracts the offset and renders its own colourless snippet, so
structured `span`s and human text both point at the user's real lines.

## Store and I/O

`snix-eval` registers no store builtins in pure mode. `tsnix` additionally adds
Nix-source stubs (`stubs.rs`) via `add_src_builtin`, which appends after the
defaults and therefore overrides or supplies them:

- **Always:** `derivation`, `derivationStrict`, `fetchurl`, `fetchTarball`,
  `fetchGit`, `fetchMercurial`, `fetchTree`, `writeFile`, `toFile`, `storePath`,
  `path`, `filterSource`, `placeholder`, `scopedImport`, `storeDir`. They
  `throw "tsnix::no_store: …"`.
- **When `--io none`:** `import`, `readFile`, `readDir`, `readFileType`,
  `pathExists`, `getEnv`, `hashFile`, `currentTime`. They `throw "tsnix::no_io: …"`.

When `--io local` is requested (and the `local-io` feature is built in), the
evaluator is upgraded with `enable_impure(Some(Box::new(StdIO)))`: real local
filesystem access and the impure builtins, but still no store. `StdIO::store_dir`
returns `None`.

Because source builtins are compiled lazily, a malformed stub would panic at
first use; `src/stubs.rs` has a unit test that parses every stub with `rnix`.

## Diagnostics

`snix-eval` errors are wrapped in `NativeError`/`BytecodeError` frames at
runtime; `diagnostic.rs` walks to the innermost cause to report a useful
`message`, maps `builtins.throw` payloads to stable codes, computes 1-based
spans, and renders a compact snippet. Span lookup is guarded with `catch_unwind`
so an unexpected span can never abort the process.

## Embedding

The public API (`lib.rs`) is independent of the CLI:

- `EvalOptions` / `CheckOptions`
- `evaluate` / `check`
- `Output`, `Diagnostic`, `EvalError`
- `schema`

The CLI (`src/cli.rs`) is part of the binary crate only, so library consumers do
not pull in clap plumbing into their API surface (clap is still linked in this
package for the binary).

## Build variants

- default: `local-io` (snix-eval `impure`) enabled.
- `--no-default-features`: pure; `--io local` yields `TSNIX-NO-LOCAL-IO`.
- `no-leak`: forwards `snix-eval/no_leak` to lower peak memory on embedded.
