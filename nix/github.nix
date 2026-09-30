{
  lib,
  ...
}:
let
  # GitHub Actions workflows, defined as typed Nix and serialised to YAML by
  # `github-actions-nix`. The `write-github` app copies the generated files into
  # the repository's `.github/` directory using Nushell.
  workflowDefinitions = {
    ci = {
      name = "CI";
      on = {
        push.branches = [ "main" ];
        pullRequest = { };
        workflowDispatch = { };
      };
      permissions.contents = "read";
      concurrency = {
        group = "ci-\${{ github.workflow }}-\${{ github.ref }}";
        cancelInProgress = true;
      };
      jobs.checks = {
        name = "Checks";
        runsOn = "ubuntu-latest";
        steps = [
          { uses = "actions/checkout@v4"; }
          {
            uses = "cachix/install-nix-action@v30";
            with_.extra_nix_config = "experimental-features = nix-command flakes";
          }
          {
            name = "nix flake check";
            run = "nix flake check -L";
          }
          {
            name = "Build documentation";
            run = "nix build .#docs --print-build-logs";
          }
        ];
      };
    };

    pages = {
      name = "Documentation";
      on = {
        push.branches = [ "main" ];
        workflowDispatch = { };
      };
      permissions = {
        contents = "read";
        pages = "write";
        id-token = "write";
      };
      concurrency = {
        group = "pages";
        cancelInProgress = false;
      };
      jobs.deploy = {
        name = "Build and deploy";
        runsOn = "ubuntu-latest";
        environment = {
          name = "github-pages";
          url = "\${{ steps.deployment.outputs.page_url }}";
        };
        steps = [
          { uses = "actions/checkout@v4"; }
          {
            uses = "cachix/install-nix-action@v30";
            with_.extra_nix_config = "experimental-features = nix-command flakes";
          }
          {
            name = "Build documentation";
            run = "nix build .#docs --print-build-logs";
          }
          { uses = "actions/configure-pages@v5"; }
          {
            uses = "actions/upload-pages-artifact@v3";
            with_.path = "result";
          }
          {
            id = "deployment";
            uses = "actions/deploy-pages@v4";
          }
        ];
      };
    };
  };
in
{
  perSystem =
    { config, pkgs, ... }:
    let
      targets = lib.mapAttrsToList (name: file: {
        destination = ".github/workflows/${name}";
        source = file;
      }) config.githubActions.workflowFiles;

      directories = lib.sort (a: b: a < b) (lib.unique (map (t: builtins.dirOf t.destination) targets));

      mkdirLines = lib.concatStringsSep "\n" (
        lib.imap0 (index: directory: ''
          let dir${toString index} = ($root | path join "${directory}")
          if not ($dir${toString index} | path exists) { mkdir $dir${toString index} }
        '') directories
      );

      copyLines = lib.concatStringsSep "\n" (
        map (t: "  cp --force ${t.source} ($root | path join \"${t.destination}\")") targets
      );

      writeGithub = pkgs.writers.writeNuBin "write-github" ''
        # Copy every generated .github file into the repository.
        def main [root: path = ".", --stage] {
        ${mkdirLines}
        ${copyLines}
          if $stage { ^git -C $root add .github }
        }
      '';

      checkGithub = pkgs.writers.writeNuBin "check-github" ''
        # Fail if the committed .github files differ from the generated ones.
        def main [root: path = "."] {
          let files = [
        ${lib.concatStringsSep "\n" (
          map (t: "    { destination: \"${t.destination}\", source: ${t.source} }") targets
        )}
          ]

          let stale = ($files | each {|file|
            let target = ($root | path join $file.destination)
            if ($target | path exists) {
              if (open --raw $target) == (open --raw $file.source) { null } else { $file.destination }
            } else { $file.destination }
          } | where {|path| $path != null })

          if ($stale | is-empty) {
            print "github files are up to date"
          } else {
            print $"stale .github files: ($stale | str join ', ')"
            print "run: nix run .#write-github -- . --stage"
            exit 1
          }
        }
      '';
    in
    {
      githubActions = {
        enable = true;
        workflows = workflowDefinitions;
      };

      apps.write-github = {
        type = "app";
        program = "${writeGithub}/bin/write-github";
      };

      apps.check-github = {
        type = "app";
        program = "${checkGithub}/bin/check-github";
      };

      checks.github-up-to-date = pkgs.runCommandLocal "github-up-to-date" { } ''
        ${checkGithub}/bin/check-github ${../.}
        touch $out
      '';
    };
}
