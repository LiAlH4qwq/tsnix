# 架构

`tsnix` 是 `snix-eval` 之上的一层薄封装。所有与 store / 文件系统相关的功能
要么被拒绝，要么显式开启。

```
   CLI (src/cli.rs, clap)
        |
        v
   lib API (src/lib.rs)
        |
   eval.rs  --- program.rs (输入、--arg 前奏、包装)
        |   \-- stubs.rs   (store/IO 拒绝，以 Nix 源码实现)
        |   \-- output.rs  (JSON -> Nix 打印)
        v
   snix-eval (git，固定 revision)
```

## 求值流程

1. **读取**（`Source::Expr` / `File` / `Stdin`），并确定用于相对路径解析与
   错误报告的 `location`。
2. **组合**（`program.rs`）出程序：
   - 可选的 `--arg`/`--argstr` 前奏，然后
   - 包装：`json`/`nix` 用 `builtins.toJSON (…)`，`raw` 用
     `builtins.toString (…)`，`check` 用 `(…)`。
3. **构建** `Evaluation::builder_pure()` 并安装 stub。
4. **求值**（`EvalMode::Strict` 深度强制，或 `Lazy`）。
5. **渲染**结果，或把错误转成诊断。

### 为什么用 `builtins.toJSON`

`Value::into_contextful_json` 是一个生成器，需要驱动 VM 私有的请求循环，
下游无法调用。`builtins.toJSON` 内部使用的正是这套逻辑并返回普通 Nix 字符串，
因此在公开 API 上包装 `builtins.toJSON` 可得到与 Nix 逐字节一致的 JSON。
`--format nix` 复用同一 JSON 投影再打印为 Nix，避免无法强制的 thunk。

### 包装与 span

合成的前奏与包装把用户代码放在换行之后，因此用户行号整体偏移已知的
`line_offset`，列号不受影响。`diagnostic.rs` 会减去偏移并自行渲染无彩色片段，
使结构化 `span` 与人类文本都指向用户真实位置。

## 标准库

启用 `stdlib` feature（默认）时，`eval.rs` 用
`add_src_builtin("std", …)` 注册内嵌的 `src/stdlib.nix`，因此可以
`builtins.std` 访问。`program.rs` 额外注入一条极小的前奏绑定
`std = builtins.std;`，并通过 `builtins.seq std (…)` 引用它，使该绑定被视为
“已使用”（不产生 `UnusedBinding` 警告）且不影响用户结果。用户传入
`--arg std …` 会抑制注入的绑定。REPL 会把同一个值种入其 `env`。

`stdlib.rs` 内嵌源码，并承载 `tsnix libdoc` 与 schema `stdlib` 段所需的 Rust
目录；单元测试会解析 `stdlib.nix` 并断言导出名与目录一致。

## store 与 I/O

纯模式下 `snix-eval` 不注册 store builtin。`tsnix` 另外通过
`add_src_builtin` 追加 Nix 源码 stub（`stubs.rs`），追加在默认项之后，因此会
覆盖或补齐：

- **始终**：`derivation`、`derivationStrict`、`fetchurl`、`fetchTarball`、
  `fetchGit`、`fetchMercurial`、`fetchTree`、`writeFile`、`toFile`、`storePath`、
  `path`、`filterSource`、`placeholder`、`scopedImport`、`storeDir`，抛出
  `tsnix::no_store:`。
- **`--io none` 时**：`import`、`readFile`、`readDir`、`readFileType`、
  `pathExists`、`getEnv`、`hashFile`、`currentTime`，抛出 `tsnix::no_io:`。

`--io local` 时用 `enable_impure(Some(Box::new(StdIO)))` 升级：真实本地文件
访问与 impure builtin，但仍无 store；`StdIO::store_dir` 返回 `None`。

源码 builtin 是惰性编译的，因此非法 stub 会在首次使用时 panic；
`src/stubs.rs` 有一个用 `rnix` 解析全部 stub 的单元测试来防止这一点。

## 诊断

运行时错误常被 `NativeError`/`BytecodeError` 包裹；`diagnostic.rs` 会走到最内层
原因来给出有用的 `message`，把 `builtins.throw` 负载映射为稳定错误码，计算
从 1 开始的 span，并渲染紧凑片段。span 查询用 `catch_unwind` 保护，避免异常
span 终止进程。

## 嵌入

公开 API（`lib.rs`）与 CLI 解耦：

- `EvalOptions` / `CheckOptions`
- `evaluate` / `check`
- `Output`、`Diagnostic`、`EvalError`
- `schema`
- `repl`（feature `repl`）

## 构建变体

- 默认：`local-io` + `repl`。
- `--no-default-features`：纯求值器；`--io local` 报 `TSNIX-NO-LOCAL-IO`，
  且不包含 REPL。
- `no-leak`：转发 `snix-eval/no_leak` 以降低峰值内存。
- `wasm`：浏览器 wasm-bindgen 脚手架（见 [WebAssembly](wasm.md)）。
