# tsnix

<p align="center"><img src="logo.svg" width="220" alt="tsnix logo"></p>

> **征集图标设计。** `tsnix` 目前只有字标，欢迎在
> [issue 区](https://github.com/LiAlH4qwq/tsnix/issues) 提交你的图标设计。

`tsnix` 是 **tiny snix-eval CLI** 的缩写，求值 **Nix 语言** 并输出结果。它仅基于
[`snix-eval`](https://snix.dev) 构建，**不包含 Nix store**，因此在无法使用
`nix` 或 `nix` 过重的场景下（例如 Android 模块或嵌入式设备）也能把 Nix
表达式转成 JSON。

它刻意**不模仿 `nix`**：

- **stdout 只输出结果**（默认 JSON）。
- **stderr 只输出诊断**（文本或结构化 JSON）。
- 稳定的机器可读错误码，如 `TSNIX-NO-STORE`。
- `tsnix schema` 以 JSON 打印完整契约。

```console
$ tsnix eval -e '{ a = 1; b = [ true null ]; }'
{"a":1,"b":[true,null]}
```

## 章节

- [CLI 参考](cli.md) —— 子命令、选项与退出码。
- [诊断](errors.md) —— JSON 诊断 schema 与稳定错误码。
- [REPL](repl.md) —— 友好交互模式与面向智能体的模式。
- [架构](architecture.md) —— 求值、stub 与输出的实现。
- [WebAssembly](wasm.md) —— wasm 版本的构建与运行。

## 非目标

构建 derivation、抓取、写入 store 路径、daemon，或模仿 `nix` CLI。
依赖 store 的 builtin 会以明确错误拒绝，而不是伪造。

## 许可证

GPL-3.0-only，与 `snix-eval` 一致。
