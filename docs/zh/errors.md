# 诊断

`tsnix` 将结果与诊断分离：

- **stdout** —— `--format` 指定的结果。
- **stderr** —— `--error-format` 指定的诊断（`text` 或 `json`）。
- 失败时 stdout 不输出任何负载。

`--error-format auto`（默认）在 stderr 是终端时选 `text`，否则选 `json`，
因此管道与智能体默认得到结构化输出。

## JSON 文档

```json
{
  "version": 1,
  "ok": false,
  "diagnostics": [
    {
      "severity": "error",
      "code": "TSNIX-NO-STORE",
      "kind": "CatchableError",
      "message": "tsnix::no_store: `derivation` requires a Nix store, …",
      "span": {
        "file": "config.nix",
        "start": { "line": 3, "column": 1 },
        "end": { "line": 3, "column": 12 }
      },
      "contexts": [],
      "rendered": "error[TSNIX-NO-STORE]: …\n  --> config.nix:3:1\n…"
    }
  ]
}
```

- `severity` —— `error` 或 `warning`。
- `code` —— 稳定，优先据此分支。
- `kind` —— 底层 `snix-eval` 错误变体（如 `TypeError`、`ParseErrors`、
  `CatchableError`）。
- `span` —— 能确定位置时存在；行列均从 1 开始，指向用户源码。
- `contexts` —— Nix 字符串上下文（通常为空）。
- `rendered` —— 供人类阅读的彩色无关片段。

## 稳定错误码

| 码 | 含义 | 常见修复 |
| --- | --- | --- |
| `TSNIX-NO-STORE` | 调用了依赖 store 的 builtin。 | 去掉 store 操作。 |
| `TSNIX-NO-IO` | 在 `--io none` 下使用文件系统 builtin。 | 改用 `--io local`。 |
| `TSNIX-NO-LOCAL-IO` | 在未启用 `local-io` 的构建中使用 `--io local`。 | 使用完整构建。 |
| `TSNIX-EVAL` | 求值 / 编译错误。 | 查看 `message`/`span`。 |
| `TSNIX-USAGE` | 非法的 `--arg`/`--argstr` 名字。 | 使用 Nix 标识符。 |
| `TSNIX-IO` | 读取输入或写入 `-o` 失败。 | 检查路径 / 权限。 |
| `TSNIX-WARN` | 非致命警告。 | 可选。 |

`tsnix::no_store:` / `tsnix::no_io:` 前缀会产生对应错误码；用户 `throw`
保持 `TSNIX-EVAL`。
