# WebAssembly

> **Status: builds, but does not run yet (upstream blocker).** CMake-free
> `wasm32-wasip1` builds succeed (`nix build .#wasm`), but the module traps at
> startup: `snix-eval`'s `systems.rs` panics with `unrecognized triple
> wasm32-wasip1` while constructing `builtins.currentSystem`. Until upstream
> handles wasm triples, the wasm build is a build-only artifact. The native CLI
> is the supported target.

`tsnix` can be compiled to WebAssembly because it only depends on `snix-eval`
and never touches a store. Two flavours are considered:

| Target | Purpose | Local I/O | REPL |
| --- | --- | --- | --- |
| `wasm32-wasip1` | a CLI runnable with `wasmtime` | no | no |
| `wasm32-unknown-unknown` | browser embedding (scaffold) | no | no |

## wasm32-wasip1

The pure evaluator (no local I/O, no REPL) is built with the `local-io` and
`repl` features disabled:

```console
$ nix build .#wasm          # produces result/bin/tsnix.wasm
$ wasmtime run result/bin/tsnix.wasm eval -e '{ a = 1; }'
# currently: panicked at snix-eval/src/systems.rs: unrecognized triple wasm32-wasip1
```

Equivalently, with a toolchain that has the target installed:

```console
$ cargo build --release --target wasm32-wasip1 --no-default-features
```

Without the `local-io` feature, `--io local` reports `TSNIX-NO-LOCAL-IO`, and
store builtins still report `TSNIX-NO-STORE`. The runtime failure is entirely
inside `snix-eval` before any expression is evaluated.

## Browser scaffold

`src/wasm.rs` (feature `wasm`) exposes a small `wasm-bindgen` API:

```rust
#[wasm_bindgen(js_name = evalJson)]
pub fn eval_json(expr: &str) -> Result<String, JsValue>;
```

Intended build (not wired into the flake yet):

```console
$ wasm-pack build --no-default-features --features wasm --target web
```

Browsers have no synchronous filesystem, so only the pure evaluator is used and
`--io local` is unavailable. Whether `snix-eval` compiles for
`wasm32-unknown-unknown` depends on its transitive dependencies; if it does not,
this remains future work.

## Limitations

- No store, ever (`TSNIX-NO-STORE`).
- No local file I/O in the wasm build (`TSNIX-NO-LOCAL-IO`).
- No REPL in the wasm build.
- Browser integration requires a JavaScript harness for input/output.
