{ inputs, ... }:
{
  imports = [
    inputs.github-actions-nix.flakeModules.default
    inputs.git-hooks.flakeModule
    ../nix/docs.nix
    ../nix/github.nix
    ../nix/hooks.nix
  ];

  perSystem = { config, ... }: {
    # git-hooks.nix builds a devshell with the hooks installed and every
    # `enabledPackages` entry on PATH.
    devShells.default = config.pre-commit.devShell;
  };
}
