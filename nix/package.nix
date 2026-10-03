{
  lib,
  rustPlatform,
  snixSourceHash,
}:

let
  # Derive the `outputHashes` keys from Cargo.lock so an upstream version bump
  # (e.g. snix-eval 0.1.0 -> 0.2.0) does not require editing this file. Both
  # crates come from the same pinned snix checkout, hence the shared hash.
  lock = builtins.fromTOML (builtins.readFile ../Cargo.lock);

  outputHashFor =
    name:
    let
      pkg = lib.findFirst (
        p: p.name == name
      ) (throw "nix/package.nix: ${name} is not in Cargo.lock") lock.package;
    in
    {
      "${pkg.name}-${pkg.version}" = snixSourceHash;
    };
in
rustPlatform.buildRustPackage {
  pname = "tsnix";
  version = "0.2.0";

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
    outputHashes = outputHashFor "snix-eval" // outputHashFor "snix-eval-builtin-macros";
  };

  meta = {
    description = "A minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";
    homepage = "https://git.snix.dev/snix/snix";
    license = lib.licenses.gpl3Only;
    mainProgram = "tsnix";
    platforms = lib.platforms.unix;
  };
}
