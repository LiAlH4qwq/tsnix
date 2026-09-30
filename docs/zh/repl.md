# REPL

`tsnix repl` 使用与 `tsnix eval` 相同的无 store 求值器进行交互式求值。共有
三种协议：

| 协议 | 参数 | 用途 |
| --- | --- | --- |
| 交互 | *(默认)* | 带历史的 `rustyline` 行编辑器 |
| ENQ | `--agent`（别名 `--automation`） | Lix `repl-automation` |
| JSON | `--protocol json` | 按行分隔的 JSON |

通用选项：`--io none|local`、`--arg`/`--argstr`、`-F/--format json|raw|nix`
（默认 `nix`）、`--mode strict|lazy`（默认 `strict`）、`-l/--load FILE`。

## 交互模式

```console
$ tsnix repl
tsnix 0.1.0 — Nix language REPL (no store). Type :help for help.
tsnix> 1 + 2
3
tsnix> x = 1 + 2;
tsnix> x * 10
30
tsnix> :quit
```

特性：

- **绑定**：`name = expr;` 将 `name` 加入作用域，供后续输入使用（惰性求值，
  与 `nix repl` 类似）。
- **多行输入**：表达式未结束时继续输入，会显示续行提示符。
- **历史**保存在 `$XDG_DATA_HOME/tsnix/history`。
- stdin 非终端时不使用行编辑器（便于脚本化）。

### 命令

| 命令 | 说明 |
| --- | --- |
| `:help`, `:?` | 显示帮助 |
| `:quit`, `:q` | 退出 |
| `:type`, `:t EXPR` | 打印 `EXPR` 的类型 |
| `:print`, `:p EXPR` | 求值并打印 `EXPR` |
| `:doc EXPR` | 显示值或 builtin 的文档 |
| `:load`, `:l FILE` | 求值文件；若是 attrset 则加入作用域 |
| `:format [nix\|json\|raw]` | 获取 / 设置展示格式 |
| `:mode [strict\|lazy]` | 获取 / 设置强制模式 |
| `:bindings` | 列出当前绑定 |
| `:clear` | 清空全部绑定 |

## 智能体模式（Lix `repl-automation`）

`tsnix repl --agent` 完全对齐 Lix 的 `repl-automation` 实验特性：

1. 在**每次**读取之前（包括续行读取）向 stdout 写入一个 `ENQ` 字节
   （`U+0005`，`0x05`）并 flush。
2. 读取一行以换行结尾的命令。
3. 输入结束时以 `0` 退出。

这让驱动进程能精确得知 REPL 何时就绪，无需解析提示符。

```console
$ printf '1 + 1\n:quit\n' | tsnix repl --agent | cat -v
^E2
^E
```

## JSON 协议

`tsnix repl --protocol json` 同样先输出 ENQ 就绪字节，然后每行读取一个 JSON
请求、每行写回一个 JSON 响应。

请求：

```json
{"expr": "1 + 1"}
{"command": "type", "arg": "builtins.map"}
{"input": "x = 1;"}
```

响应：

```json
{"ok": true, "format": "nix", "value": "2"}
{"ok": true, "message": "builtins.map :: lambda"}
{"ok": false, "diagnostics": [ { "code": "TSNIX-NO-STORE", "...": "..." } ]}
```

诊断复用 [诊断](errors.md) 的 schema，因此智能体可直接使用稳定的 `code` 字段。
