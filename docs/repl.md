# REPL

`tsnix repl` evaluates Nix expressions interactively against the same
store-less evaluator as `tsnix eval`. There are three protocols:

| Protocol | Flag | Purpose |
| --- | --- | --- |
| Interactive | *(default)* | a `rustyline` line editor with history |
| ENQ | `--agent` (alias `--automation`) | Lix `repl-automation` |
| JSON | `--protocol json` | newline-delimited JSON |

Common options: `--io none|local`, `--arg`/`--argstr`, `-F/--format json|raw|nix`
(default `nix`), `--mode strict|lazy` (default `strict`), `-l/--load FILE`.

## Interactive mode

```console
$ tsnix repl
tsnix 0.1.0 — Nix language REPL (no store). Type :help for help.
tsnix> 1 + 2
3
tsnix> x = 1 + 2;
tsnix> x * 10
30
tsnix> { a = 1; b = [ 2 3 ]; }
{ a = 1; b = [ 2 3 ]; }
tsnix> :quit
```

Features:

- **Bindings**: `name = expr;` adds `name` to scope for later inputs (evaluated
  lazily, like `nix repl`).
- **Multi-line input**: keep typing until the expression is complete; a
  continuation prompt is shown.
- **History** at `$XDG_DATA_HOME/tsnix/history`.
- When standard input is not a terminal, the REPL reads lines without a line
  editor (useful for scripting).

### Commands

| Command | Description |
| --- | --- |
| `:help`, `:?` | show help |
| `:quit`, `:q` | leave the REPL |
| `:type`, `:t EXPR` | print the type of `EXPR` |
| `:print`, `:p EXPR` | evaluate and print `EXPR` |
| `:doc EXPR` | show documentation for a value or builtin |
| `:load`, `:l FILE` | evaluate a file; an attribute set is added to scope |
| `:format [nix\|json\|raw]` | get or set the display format |
| `:mode [strict\|lazy]` | get or set the forcing mode |
| `:bindings` | list current bindings |
| `:clear` | drop all bindings |

## Agent mode (Lix `repl-automation`)

`tsnix repl --agent` mirrors Lix's `repl-automation` experimental feature:

1. Before **every** read — including continuation reads — write one `ENQ` byte
   (`U+0005`, `0x05`) to stdout and flush.
2. Read one newline-terminated command.
3. On end of input, exit `0`.

This lets a driving process know exactly when the REPL is ready and
synchronise without parsing prompts.

```console
$ printf '1 + 1\n:quit\n' | tsnix repl --agent | cat -v
^E2
^E
```

## JSON protocol

`tsnix repl --protocol json` emits the same ENQ readiness byte, then reads one
JSON request per line and writes one JSON response per line.

Requests:

```json
{"expr": "1 + 1"}
{"command": "type", "arg": "builtins.map"}
{"input": "x = 1;"}
```

Responses:

```json
{"ok": true, "format": "nix", "value": "2"}
{"ok": true, "message": "builtins.map :: lambda"}
{"ok": false, "diagnostics": [ { "code": "TSNIX-NO-STORE", "...": "..." } ]}
```

Diagnostics reuse the schema from [Diagnostics](errors.md), so the stable
`code` field is available to agents.
