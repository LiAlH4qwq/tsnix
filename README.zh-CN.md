# tsnix

一个基于 [`snix-eval`](https://snix.dev) 构建的**极简、可嵌入的 Nix 语言求值器**。
它**不包含 Nix store**：只负责求值 Nix 语言并输出 JSON / 原始字符串 / Nix 源码，
因此可以在没有 `nix` 或 `nix` 过重的场景下生成配置——例如 Android Magisk/KernelSU
模块或嵌入式设备。

`tsnix` 刻意**不模仿 `nix` 的命令行界面**，而是提供小巧、现代、对智能体友好的接口：

- **stdout 只输出结果**（默认 JSON），没有其他内容。
- **stderr 只输出诊断**（文本或结构化 JSON）。
- 稳定的机器可读错误码，如 `TSNIX-NO-STORE`。
- `tsnix schema` 命令以 JSON 打印完整契约。

- [English](README.md) | 简体中文
- 面向智能体的说明：[AGENTS.md](AGENTS.md)

## 为什么

`snix-eval` 是 Nix 求值的一种快速、模块化的 Rust 实现。由于它「库优先」，
我们可以只嵌入求值器，跳过整个 store、daemon 与构建机制。这让 `tsnix` 小到
足以运行在嵌入式设备上，同时保持与 Nix 求值语义兼容（测试套件会在有 `nix`
时与真实 `nix` 对拍验证）。

优先级：**正确 > 性能 > 可读 > 优雅**。因此 store 操作绝不会被伪造，而是
以明确错误拒绝。

## 支持范围

| 范围 | 支持 |
| --- | --- |
| 纯 Nix 表达式、attrset、list、函数、`let`、`with`、`rec`、`inherit`、字符串插值、`builtins.*` 数据函数 | ✅ |
| `--format json`（经由 `builtins.toJSON`） | ✅ 与 Nix 完全一致 |
| `--format raw`（经由 `builtins.toString`） | ✅ |
| `--format nix`（尽力而为的打印器） | ✅（path 会变成字符串，函数会被拒绝） |
| `--arg` / `--argstr` 顶层变量绑定 | ✅ |
| 本地文件 I/O：`import`、`readFile`、`readDir`、`pathExists`、`getEnv` …（`--io local`） | ✅ |
| `derivation`、`fetchurl`、`writeFile`、`storePath`、`storeDir`、`toFile` … | ❌ 拒绝（`TSNIX-NO-STORE`） |

完整列表见 `tsnix schema`。

## 安装

### Cargo

```console
$ cargo build --release          # 或：cargo install --path .
$ ./target/release/tsnix --help
```

> `snix-eval` 尚未发布到 crates.io，因此 `tsnix` 依赖一个固定的 git revision，
> 见 `Cargo.toml`。

### Nix

```console
$ nix build                 # 本机二进制
$ nix build .#static        # 面向嵌入式目标的静态 musl 二进制
$ nix run . -- eval -e '1 + 1'
$ nix develop               # 带固定工具链的开发 shell
```

## 用法

```console
# 默认输出 JSON
$ tsnix eval -e '{ a = 1; b = [ true null ]; }'
{"a":1,"b":[true,null]}

# 美化输出
$ tsnix eval -e '{ a = 1; }' --pretty

# 求值文件，并导入其它本地文件
$ tsnix eval -f config.nix --io local

# 像 nix --arg/--argstr 一样绑定顶层变量
$ tsnix eval -e '{ url = "http://${host}:${toString port}"; }' \
    --argstr host example.com --arg port 8080
{"url":"http://example.com:8080"}

# 原始字符串输出
$ tsnix eval -e '"hello ${builtins.currentSystem}"' --format raw

# Nix 打印
$ tsnix eval -e '{ a = 1; b = [ 1 2 ]; }' --format nix
{ a = 1; b = [ 1 2 ]; }

# 从 stdin 读取
$ echo '{ a = 1; }' | tsnix eval -

# 只校验不求值
$ tsnix check -f config.nix

# 机器可读契约
$ tsnix schema --pretty
```

## CLI 契约

| | |
| --- | --- |
| stdout | `--format` 指定的结果（仅此） |
| stderr | 诊断：`--error-format text`（TTY 默认）或 `json` |
| 退出码 `0` | 成功 |
| 退出码 `1` | 求值/编译错误 |
| 退出码 `2` | 用法错误（clap） |

输入来源互斥：位置参数 `SOURCE`、`-e/--expr` 或 `-f/--file`（用 `-` 表示 stdin；
全部省略时也读取 stdin）。

输出格式：

- `json` —— `builtins.toJSON`（紧凑、精确）；`--pretty` 用 `serde_json` 重新缩进。
- `raw` —— `builtins.toString`。
- `nix` —— 将 JSON 投影渲染为 Nix 源码。

## I/O 与 store 策略

默认情况下 `tsnix` 是**纯**的：`import`、`readFile`、`readDir`、`pathExists`、
`getEnv`、`hashFile`、`currentTime` 都会抛出 `TSNIX-NO-IO`。使用 `--io local`
可通过 `snix-eval` 的 `StdIO` 启用本地文件访问（相对导入会相对源文件解析）。
任何 I/O 都不会触及 store 路径。

依赖 store 的 builtin **始终**以 `TSNIX-NO-STORE` 和一条中英双语消息拒绝：

```console
$ tsnix eval -e 'derivation { name = "x"; }'
error[TSNIX-NO-STORE]: tsnix::no_store: `derivation` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store，无法构建 derivation。)
  --> <expr>:1:1
  |
 1 | derivation { name = "x"; }
  | ^~~~~~~~~~~~~~~~~~~~~~~~~~
```

## 诊断

`--error-format json` 会在 stderr 输出稳定文档：

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

错误码稳定：`TSNIX-NO-STORE`、`TSNIX-NO-IO`、`TSNIX-NO-LOCAL-IO`、
`TSNIX-EVAL`、`TSNIX-USAGE`、`TSNIX-IO`、`TSNIX-WARN`。
详见 [docs/errors.md](docs/errors.md)。

## 嵌入

`tsnix` 既是库也是瘦 CLI：

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

设计见 [docs/architecture.md](docs/architecture.md)。

## 嵌入式 / wasm 构建

- `--no-default-features` 会去掉本地 I/O（`snix-eval` 的 `impure` feature），
  得到不依赖文件系统的纯求值器；此时 `--io local` 会报 `TSNIX-NO-LOCAL-IO`。
- `no-leak` 转发 `snix-eval/no_leak`，以一定速度代价降低峰值内存。
- release profile 使用 `lto = "fat"`、`codegen-units = 1`、`opt-level = "s"`、
  `strip`。

静态 musl 构建由 flake 提供（`packages.<system>.static`）。

## 测试

```console
$ cargo test
```

测试套件包含一项 golden 对拍：当系统存在 `nix` 时，用真实
`nix eval --json` 校验相同表达式。

## 许可证

GPL-3.0-only，与 `snix-eval` 保持一致。见 [LICENSE](LICENSE)。
