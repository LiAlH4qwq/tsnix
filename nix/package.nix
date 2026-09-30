{
  lib,
  rustPlatform,
  snixSourceHash,
}:

rustPlatform.buildRustPackage {
  pname = "tsnix";
  version = "0.1.0";

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

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "snix-eval-0.1.0" = snixSourceHash;
      "snix-eval-builtin-macros-0.0.1" = snixSourceHash;
    };
  };

  meta = {
    description = "A minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";
    homepage = "https://git.snix.dev/snix/snix";
    license = lib.licenses.gpl3Only;
    mainProgram = "tsnix";
    platforms = lib.platforms.unix;
  };
}
