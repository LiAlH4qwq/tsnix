{
  description = "tsnix — a minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    systems.url = "github:nix-systems/default";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # mdBook theme used by the documentation site (`flake = false`: it is an
    # asset tree, not a flake).
    mdbook-theme-milieuim = {
      url = "github:milieuim/mdbook-theme-milieuim";
      flake = false;
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { config, withSystem, ... }:
      {
        systems = import inputs.systems;

        imports = [
          # Development tooling (git hooks, generated GitHub Actions) lives in
          # the `dev` partition so it never reaches a consumer's lock file.
          inputs.flake-parts.flakeModules.partitions
          ./nix/docs.nix
          ./nix/wasm.nix
        ];

        partitionedAttrs = {
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
          in
          {
            _module.args.pkgs = pkgs;

            packages.tsnix = tsnix;
            packages.default = config.packages.tsnix;
            # Static musl build for embedded targets.
            packages.static = pkgs.pkgsStatic.callPackage ./nix/package.nix {
              inherit snixSourceHash;
            };

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
