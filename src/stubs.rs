//! Nix-source builtins that replace store- or I/O-dependent functionality.
//!
//! `tsnix` never registers a Nix store. Every store-dependent builtin is
//! replaced by a stub that raises a stable `tsnix::no_store:` error, and when
//! `--io` is `none` the filesystem builtins raise `tsnix::no_io:`. A message
//! consists of a machine-readable prefix, an English sentence and a Chinese
//! sentence.

/// Builtins that would require a Nix store. Always installed.
pub(crate) const STORE_STUBS: &[(&str, &str)] = &[
    (
        "derivation",
        r##"_args: throw "tsnix::no_store: `derivation` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store，无法构建 derivation。)""##,
    ),
    (
        "derivationStrict",
        r##"_args: throw "tsnix::no_store: `derivationStrict` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "fetchurl",
        r##"_args: throw "tsnix::no_store: `fetchurl` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store，无法下载固定输出。)""##,
    ),
    (
        "fetchTarball",
        r##"_args: throw "tsnix::no_store: `fetchTarball` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "fetchGit",
        r##"_args: throw "tsnix::no_store: `fetchGit` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "fetchMercurial",
        r##"_args: throw "tsnix::no_store: `fetchMercurial` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "fetchTree",
        r##"_args: throw "tsnix::no_store: `fetchTree` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "writeFile",
        r##"_args: throw "tsnix::no_store: `writeFile` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store，无法写入 store。)""##,
    ),
    (
        "toFile",
        r##"_args: throw "tsnix::no_store: `toFile` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "storePath",
        r##"_args: throw "tsnix::no_store: `storePath` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "path",
        r##"_args: throw "tsnix::no_store: `path` copies into a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "filterSource",
        r##"_args: throw "tsnix::no_store: `filterSource` copies into a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "placeholder",
        r##"_args: throw "tsnix::no_store: `placeholder` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
    (
        "scopedImport",
        r##"_args: throw "tsnix::no_store: `scopedImport` is unavailable; use `import` with `--io local` instead. (tsnix 不支持 scopedImport。)""##,
    ),
    (
        "storeDir",
        r##"throw "tsnix::no_store: `storeDir` requires a Nix store, which tsnix does not provide. (tsnix 不提供 Nix store。)""##,
    ),
];

/// Builtins that need local file I/O. Installed only when `--io none`.
pub(crate) const IO_STUBS: &[(&str, &str)] = &[
    (
        "import",
        r##"_path: throw "tsnix::no_io: `import` is disabled. Re-run with `--io local` to allow local file imports. (已禁用 import，请使用 --io local。)""##,
    ),
    (
        "readFile",
        r##"_path: throw "tsnix::no_io: `readFile` is disabled. Re-run with `--io local` to allow local file reads. (已禁用文件读取，请使用 --io local。)""##,
    ),
    (
        "readDir",
        r##"_path: throw "tsnix::no_io: `readDir` is disabled. Re-run with `--io local`. (已禁用目录读取，请使用 --io local。)""##,
    ),
    (
        "readFileType",
        r##"_path: throw "tsnix::no_io: `readFileType` is disabled. Re-run with `--io local`. (已禁用文件读取，请使用 --io local。)""##,
    ),
    (
        "pathExists",
        r##"_path: throw "tsnix::no_io: `pathExists` is disabled. Re-run with `--io local`. (已禁用文件探测，请使用 --io local。)""##,
    ),
    (
        "getEnv",
        r##"_key: throw "tsnix::no_io: `getEnv` is disabled. Re-run with `--io local`. (已禁用环境变量读取，请使用 --io local。)""##,
    ),
    (
        "hashFile",
        r##"_args: throw "tsnix::no_io: `hashFile` is disabled. Re-run with `--io local`. (已禁用文件哈希，请使用 --io local。)""##,
    ),
    (
        "currentTime",
        r##"throw "tsnix::no_io: `currentTime` is disabled; tsnix aims to be deterministic. (currentTime 已禁用以保证确定性。)""##,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// `snix-eval` panics if a source builtin fails to parse, so guard against
    /// a malformed stub at compile-test time.
    #[test]
    fn all_stubs_parse() {
        for &(name, source) in STORE_STUBS.iter().chain(IO_STUBS.iter()) {
            let parsed = rnix::ast::Root::parse(source);
            assert!(
                parsed.errors().is_empty(),
                "stub `{name}` has parse errors: {:?}",
                parsed.errors()
            );
            assert!(
                parsed.tree().expr().is_some(),
                "stub `{name}` has no top-level expression"
            );
        }
    }
}
