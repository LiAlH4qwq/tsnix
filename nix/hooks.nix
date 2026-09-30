{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      pre-commit = {
        # The `clippy` hook needs to fetch and compile dependencies, which the
        # `nix flake check` sandbox forbids; disable the derived check.
        check.enable = false;

        settings = {
          hooks = {
            nixfmt.enable = true;
            rustfmt.enable = true;
            clippy.enable = true;
          };

          enabledPackages = with pkgs; [
            mdbook
            nixfmt
            nushell
          ];
        };
      };
    };
}
