# Diagnostics

`tsnix` separates the payload from its diagnostics:

- **stdout** — the evaluation result, in `--format`.
- **stderr** — diagnostics, in `--error-format` (`text` or `json`).
- On failure there is no payload on stdout.

`--error-format auto` (the default) selects `text` when stderr is a terminal and
`json` otherwise, so pipes and agents get structured output automatically.

## JSON document

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

- `severity` — `error` or `warning`.
- `code` — stable, switch on this first.
- `kind` — the underlying `snix-eval` error variant (e.g. `TypeError`,
  `ParseErrors`, `CatchableError`).
- `span` — present when a location could be determined. Line and column are
  1-based and refer to the user's own source, not the internal wrapper.
- `contexts` — Nix string contexts (usually empty; `tsnix` has no store).
- `rendered` — a colourless snippet for humans.

## Codes

| Code | Meaning | Typical fix |
| --- | --- | --- |
| `TSNIX-NO-STORE` | A store-dependent builtin was called. | Remove the store operation. |
| `TSNIX-NO-IO` | A filesystem builtin was used with `--io none`. | Re-run with `--io local`. |
| `TSNIX-NO-LOCAL-IO` | `--io local` on a build without the `local-io` feature. | Use a full build. |
| `TSNIX-EVAL` | Evaluation or compilation error. | Inspect `message`/`span`. |
| `TSNIX-USAGE` | Invalid `--arg`/`--argstr` name. | Use a Nix identifier. |
| `TSNIX-IO` | Could not read input or write `-o`. | Check paths/permissions. |
| `TSNIX-WARN` | Non-fatal warning. | Optional. |

The `tsnix::no_store:` / `tsnix::no_io:` message prefixes are what produce the
corresponding codes; user `throw`s keep `TSNIX-EVAL`.

## Text output

```console
$ tsnix eval -e '1 + "a"'
error[TSNIX-EVAL]: expected value of type 'number (either int or float)', but found a 'string'
  --> <expr>:1:1
  |
 1 | 1 + "a"
  | ^~~~~~~
```
