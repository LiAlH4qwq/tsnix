# CLI 参考

`tsnix schema` 是本文档的权威机器可读版本。

## 概要

```
tsnix <COMMAND>

Commands:
  eval    求值 Nix 表达式或文件
  check   仅解析与编译，不求值
  repl    启动 REPL（友好 / 智能体模式）
  schema  以 JSON 打印 CLI 契约
```

## `eval` / `check` 通用选项

| 选项 | 说明 |
| --- | --- |
| `[SOURCE]` | `.nix` 文件路径，或 `-` 表示 stdin。 |
| `-e, --expr <EXPR>` | 求值内联表达式。 |
| `-f, --file <PATH>` | 求值文件。 |
| `--arg <NAME> <EXPR>` | 将 `NAME` 绑定为解析后的 Nix 表达式。可重复。 |
| `--argstr <NAME> <STRING>` | 将 `NAME` 绑定为字符串。可重复。 |
| `--io <none\|local>` | 是否允许本地文件 I/O。默认 `none`。 |
| `--error-format <auto\|text\|json>` | 诊断格式。默认 `auto`（TTY 上为 text，否则 json）。 |
| `-q, --quiet` | 抑制警告。 |

必须且只能选择一种输入；`-`（或全部省略）读取 stdin。`--arg`/`--argstr`
的名字必须是合法 Nix 标识符。

## 仅 `eval`

| 选项 | 说明 |
| --- | --- |
| `-F, --format <json\|raw\|nix>` | 输出格式。默认 `json`。 |
| `--pretty` | 用 `serde_json` 美化 JSON（尽力而为）。 |
| `--mode <strict\|lazy>` | 顶层强制模式。默认 `strict`。 |
| `--nix-path <SPEC>` | 用于 `<...>` 查找的 `NIX_PATH`（需 `--io local`）。 |
| `-o, --output <PATH>` | 写入文件而非 stdout。 |

## `repl` 选项

| 选项 | 说明 |
| --- | --- |
| `--protocol <interactive\|enq\|json>` | 输入输出协议。默认 `interactive`。 |
| `--agent` | 等价于 `--protocol enq`（Lix `repl-automation`：ENQ 就绪字节）。 |
| `-F, --format <json\|raw\|nix>` | 展示格式。默认 `nix`。 |
| `--mode <strict\|lazy>` | 强制模式。默认 `strict`。 |
| `-l, --load <PATH>` | 启动时载入文件。可重复。 |

## `schema`

| 选项 | 说明 |
| --- | --- |
| `--pretty` | 美化输出。 |

## 退出码

| 码 | 含义 |
| --- | --- |
| `0` | 成功 |
| `1` | 求值 / 编译错误 |
| `2` | 用法错误（未知选项、缺少参数等） |
