# tsnix

`tsnix` evaluates the **Nix language** and prints the result. It is built
solely on [`snix-eval`](https://snix.dev) and contains **no Nix store**, so it
can turn Nix expressions into JSON where the `nix` binary is unavailable or too
heavy — for example an Android module or an embedded device.

It is deliberately **not** a `nix` clone:

- **stdout is the payload** (JSON by default) and nothing else.
- **stderr is diagnostics** (text or structured JSON).
- stable machine-readable error codes such as `TSNIX-NO-STORE`.
- a `tsnix schema` command that prints the whole contract as JSON.

```console
$ tsnix eval -e '{ a = 1; b = [ true null ]; }'
{"a":1,"b":[true,null]}
```

## Chapters

- [CLI reference](cli.md) — commands, options and exit codes.
- [Standard library](stdlib.md) — the pure-Nix `std` namespace.
- [Diagnostics](errors.md) — the JSON diagnostic schema and stable codes.
- [REPL](repl.md) — the friendly and agent-friendly interactive modes.
- [Architecture](architecture.md) — how evaluation, stubs and output work.
- [WebAssembly](wasm.md) — building and running the wasm version.

## Non-goals

Building derivations, fetching, writing store paths, a daemon, or emulating the
`nix` CLI. Store-dependent builtins are rejected with a clear error rather than
faked.

## License

GPL-3.0-only, matching `snix-eval`.
