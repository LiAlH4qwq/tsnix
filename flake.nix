{
  description = "tsnix — a minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { config, withSystem, ... }:
      {
        # Deliberately not `import inputs.systems` (nix-systems/default):
        # nixpkgs 26.11 dropped x86_64-darwin support, so keeping it in the
        # list would make `nix flake show --all-systems` / `flake check` fail.
        systems = [
          "aarch64-linux"
          "x86_64-linux"
          "aarch64-darwin"
        ];

        imports = [
          # Development tooling (docs, git hooks, generated GitHub Actions)
          # lives in the `dev` partition so it never reaches a consumer's
          # lock file.
          inputs.flake-parts.flakeModules.partitions
          ./nix/wasm.nix
        ];

        partitionedAttrs = {
          # `docs`/`rustdoc` and their mdBook theme input live in the dev
          # partition; the root `packages` set is re-exported from there.
          packages = "dev";
          devShells = "dev";
          apps = "dev";
          checks = "dev";
        };

        partitions.dev = {
          extraInputsFlake = ./dev;
          module.imports = [ ./dev/flake-module.nix ];
        };

        perSystem =
          { system, config, ... }:
          let
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ (import inputs.rust-overlay) ];
            };

            # NAR hash of the pinned snix-eval git source (see Cargo.toml). Both
            # `snix-eval` and its in-repo `snix-eval-builtin-macros` path
            # dependency come from the same checkout, so they share a hash.
            #
            # To refresh after bumping the revision in Cargo.toml:
            #   nix build .#tsnix 2>&1 | grep 'got:'
            inherit (import ./nix/constants.nix) snixSourceHash;

            tsnix = pkgs.callPackage ./nix/package.nix { inherit snixSourceHash; };

            # Fully static musl build: no dynamic loader, no Termux prefix.
            # Runnable from a normal Android root shell (Magisk/KernelSU) and
            # from Termux, as well as any Linux. Static for embedded targets.
            tsnixStatic = pkgs.pkgsStatic.callPackage ./nix/package.nix {
              inherit snixSourceHash;
            };
          in
          {
            _module.args.pkgs = pkgs;

            packages.tsnix = tsnix;
            packages.default = config.packages.tsnix;
            packages.static = tsnixStatic;
            # Android/Termux targets. Both are the same self-contained static
            # binary; `android` is named for a plain root shell, `termux` for
            # the Termux prefix. Neither needs the Android NDK or Bionic.
            packages.android = tsnixStatic;
            packages.termux = tsnixStatic;

            apps.default = {
              type = "app";
              program = "${config.packages.tsnix}/bin/tsnix";
            };

            formatter = pkgs.nixfmt;
          };

        flake.overlays.tsnix = final: _prev: {
          tsnix = withSystem final.stdenv.hostPlatform.system (ps: ps.config.packages.tsnix);
        };

        flake.overlays.default = config.flake.overlays.tsnix;
      }
    );
}
