{
  lib,
  ...
}:
let
  constants = import ./constants.nix;
in
{
  perSystem =
    { pkgs, ... }:
    let
      # A toolchain with the wasip1 target installed.
      rustWasm = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "wasm32-wasip1" ];
      };
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustWasm;
        rustc = rustWasm;
      };

      src = lib.cleanSourceWith {
        src = ../.;
        filter =
          path: _type:
          !(builtins.elem (baseNameOf (toString path)) [
            "target"
            "result"
            "book"
            ".direnv"
            ".git"
          ]);
      };
    in
    {
      # Pure evaluator compiled to wasm32-wasip1 (runnable with e.g. wasmtime).
      # No local I/O and no REPL in this build.
      packages.wasm = rustPlatform.buildRustPackage {
        pname = "tsnix-wasm";
        version = "0.2.0";
        inherit src;
        cargoLock = {
          lockFile = ../Cargo.lock;
          outputHashes = {
            "snix-eval-0.1.0" = constants.snixSourceHash;
            "snix-eval-builtin-macros-0.0.1" = constants.snixSourceHash;
          };
        };
        cargoBuildFlags = [
          "--target"
          "wasm32-wasip1"
          "--no-default-features"
        ];
        doCheck = false;
        installPhase = ''
          runHook preInstall
          mkdir -p $out/bin
          cp target/wasm32-wasip1/release/tsnix.wasm $out/bin/tsnix.wasm
          runHook postInstall
        '';
        meta = {
          description = "tsnix compiled to wasm32-wasip1 (pure evaluator, no store)";
          license = lib.licenses.gpl3Only;
          # Built on any host with a wasm32-wasip1-capable toolchain.
          platforms = lib.platforms.all;
        };
      };
    };
}
