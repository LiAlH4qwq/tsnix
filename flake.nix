{
  description = "tsnix — a minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";

  inputs = {
    # If the GitHub API is rate-limited (unauthenticated shared IPs), override
    # the inputs, e.g.:
    #   nix build .#tsnix \
    #     --override-input nixpkgs path:/path/to/nixpkgs \
    #     --override-input rust-overlay \
    #       https://codeload.github.com/oxalica/rust-overlay/tar.gz/refs/heads/master
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      # NAR hash of the pinned snix-eval git source (see Cargo.toml). Both the
      # `snix-eval` crate and its in-repo `snix-eval-builtin-macros` path
      # dependency come from the same checkout, so they share a hash.
      #
      # To refresh after bumping the revision in Cargo.toml:
      #   nix build .#tsnix 2>&1 | grep 'got:'
      snixSourceHash = "sha256-TxtwwKnNdRXHdf4kr4c62R2jkTwgXlOiHfJiRmn0o24=";

      # Shared derivation arguments for a given nixpkgs/rustPlatform.
      mkTsnix =
        pkgs:
        let
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: _type:
              !(builtins.elem (baseNameOf (toString path)) [
                "target"
                "result"
                ".direnv"
                ".git"
              ]);
          };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "tsnix";
          version = "0.1.0";
          inherit src;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "snix-eval-0.1.0" = snixSourceHash;
              "snix-eval-builtin-macros-0.0.1" = snixSourceHash;
            };
          };
          meta = {
            description = "A minimal, embeddable Nix-language evaluator built solely on snix-eval (no store)";
            homepage = "https://git.snix.dev/snix/snix";
            license = pkgs.lib.licenses.gpl3Only;
            mainProgram = "tsnix";
            platforms = pkgs.lib.platforms.unix;
          };
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          tsnix = mkTsnix pkgs;
        in
        {
          default = tsnix;
          tsnix = tsnix;
          # Static musl build for embedded targets on the host's architecture.
          static = mkTsnix pkgs.pkgsStatic;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.tsnix}/bin/tsnix";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          rust = pkgs.rust-bin.stable.latest.default.override {
            components = [
              "rustfmt"
              "clippy"
            ];
            targets = [ "wasm32-wasip1" ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rust
              pkgs.pkg-config
              # `cc` for the linker and proc-macro build scripts.
              pkgs.stdenv.cc
            ];
          };
        }
      );

      checks = forAllSystems (system: {
        tsnix = self.packages.${system}.tsnix;
      });

      formatter = forAllSystems (system: (import nixpkgs { inherit system; }).nixfmt);
    };
}
