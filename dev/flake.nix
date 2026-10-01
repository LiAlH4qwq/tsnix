{
  description = "Development-only inputs for tsnix. Used by the dev partition of the top-level flake; they never appear in a consumer's lock file.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    github-actions-nix = {
      url = "github:synapdeck/github-actions-nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
    };

    # mdBook theme used by the documentation site. It is an asset tree, not a
    # flake, and only the dev partition (docs) needs it. Pinned to the current
    # main revision; bump it here to refresh the theme.
    mdbook-theme-milieuim = {
      url = "github:milieuim/mdbook-theme-milieuim/ec716d732fb77907dfeaf4d23e754813ad1688c3";
      flake = false;
    };
  };

  # This flake only contributes its inputs to the dev partition.
  outputs = _: { };
}
