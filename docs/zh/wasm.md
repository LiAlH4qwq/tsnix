# WebAssembly

> **状态：可构建，但暂时无法运行（上游阻塞）。** `wasm32-wasip1` 构建成功
> （`nix build .#wasm`），但模块在启动时 trap：`snix-eval` 的 `systems.rs`
> 在构造 `builtins.currentSystem` 时 panic：`unrecognized triple
> wasm32-wasip1`。在上游支持 wasm triple 之前，wasm 仅是可构建产物。受支持的
> 目标是原生 CLI。

由于只依赖 `snix-eval` 且从不接触 store，`tsnix` 可以编译为 WebAssembly。
考虑两种形态：

| 目标 | 用途 | 本地 I/O | REPL |
| --- | --- | --- | --- |
| `wasm32-wasip1` | 可用 `wasmtime` 运行的 CLI | 否 | 否 |
| `wasm32-unknown-unknown` | 浏览器嵌入（脚手架） | 否 | 否 |

## wasm32-wasip1

关闭 `local-io` 与 `repl` feature，构建纯求值器：

```console
$ nix build .#wasm          # 产出 result/bin/tsnix.wasm
$ wasmtime run result/bin/tsnix.wasm eval -e '{ a = 1; }'
# 目前：panicked at snix-eval/src/systems.rs: unrecognized triple wasm32-wasip1
```

或使用已安装该 target 的工具链：

```console
$ cargo build --release --target wasm32-wasip1 --no-default-features
```

没有 `local-io` 时，`--io local` 报 `TSNIX-NO-LOCAL-IO`；store builtin 仍报
`TSNIX-NO-STORE`。该运行时失败完全发生在 `snix-eval` 内部，早于任何表达式求值。

## 浏览器脚手架

`src/wasm.rs`（feature `wasm`）暴露一个小型 `wasm-bindgen` API：

```rust
#[wasm_bindgen(js_name = evalJson)]
pub fn eval_json(expr: &str) -> Result<String, JsValue>;
```

预期构建方式（尚未接入 flake）：

```console
$ wasm-pack build --no-default-features --features wasm --target web
```

浏览器没有同步文件系统，因此只使用纯求值器，`--io local` 不可用。
`snix-eval` 能否为 `wasm32-unknown-unknown` 编译取决于其传递依赖；若不能，
则这部分留作未来工作。

## 限制

- 永远没有 store（`TSNIX-NO-STORE`）。
- wasm 构建没有本地文件 I/O（`TSNIX-NO-LOCAL-IO`）。
- wasm 构建没有 REPL。
- 浏览器集成需要一层 JavaScript 输入输出壳。
