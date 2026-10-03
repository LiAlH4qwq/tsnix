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
            name = "Build and test tsnix";
            run = "nix build .#tsnix --print-build-logs";
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

    # Weekly bump of the pinned snix-eval revision. Runs the test build in this
    # job (self-test), then opens/updates a PR. Configure the
    # `SNIX_UPDATE_TOKEN` secret (contents + pull-requests write) so the PR
    # triggers CI; without it the fallback GITHUB_TOKEN opens the PR but GitHub
    # suppresses `pull_request` runs for it.
    "update-snix" = {
      name = "Update snix-eval";
      on = {
        schedule = [ { cron = "0 6 * * 1"; } ];
        workflowDispatch = { };
      };
      permissions = {
        contents = "write";
        pull-requests = "write";
      };
      concurrency = {
        group = "update-snix";
        cancelInProgress = false;
      };
      jobs.update = {
        name = "Bump snix-eval";
        runsOn = "ubuntu-latest";
        timeoutMinutes = 90;
        steps = [
          {
            uses = "actions/checkout@v4";
            with_.fetch-depth = 0;
          }
          {
            uses = "cachix/install-nix-action@v30";
            with_.extra_nix_config = "experimental-features = nix-command flakes";
          }
          # Reads rust-toolchain.toml; only `cargo update` needs it.
          { uses = "actions-rust-lang/setup-rust-toolchain@v1"; }
          {
            id = "bump";
            name = "Update the snix-eval pin";
            run = "./.github/scripts/update-snix.sh";
          }
          {
            name = "Open pull request";
            if_ = "steps.bump.outputs.changed == 'true'";
            uses = "peter-evans/create-pull-request@v7";
            with_ = {
              token = "\${{ secrets.SNIX_UPDATE_TOKEN || github.token }}";
              branch = "snix/update";
              delete-branch = true;
              commit-message = "chore: update snix-eval to \${{ steps.bump.outputs.rev }}";
              title = "chore: update snix-eval to \${{ steps.bump.outputs.rev }}";
              body = "Automated weekly bump of the pinned `snix-eval` revision.\n\n- revision: `\${{ steps.bump.outputs.rev }}`\n- refreshed `nix/constants.nix`\n- self-tested with `nix build .#tsnix`";
              labels = "dependencies";
            };
          }
        ];
      };
    };

    # Tagged releases (v*) publish prebuilt binaries. Cargo/crates.io publishing
    # is not possible while snix-eval is a git-only dependency, so this is the
    # distribution path. The static musl binaries double as the Android/Termux
    # targets: they run from a normal root shell and inside Termux.
    release = {
      name = "Release";
      on = {
        push.tags = [ "v*" ];
        workflowDispatch = { };
      };
      permissions.contents = "write";
      concurrency = {
        group = "release-\${{ github.ref }}";
        cancelInProgress = false;
      };
      jobs.build = {
        name = "Build \${{ matrix.asset }}";
        runsOn = "\${{ matrix.runner }}";
        strategy = {
          failFast = false;
          matrix.include = [
            {
              runner = "ubuntu-latest";
              package = "android";
              asset = "tsnix-x86_64-linux-android";
            }
            {
              runner = "ubuntu-24.04-arm";
              package = "android";
              asset = "tsnix-aarch64-linux-android";
            }
            {
              runner = "macos-14";
              package = "tsnix";
              asset = "tsnix-aarch64-darwin";
            }
          ];
        };
        steps = [
          { uses = "actions/checkout@v4"; }
          {
            uses = "cachix/install-nix-action@v30";
            with_.extra_nix_config = "experimental-features = nix-command flakes";
          }
          {
            name = "Build";
            run = "nix build .#\${{ matrix.package }} --print-build-logs";
          }
          {
            name = "Stage binary";
            run = "cp result/bin/tsnix \${{ matrix.asset }} && chmod +x \${{ matrix.asset }}";
          }
          {
            uses = "actions/upload-artifact@v4";
            with_ = {
              name = "\${{ matrix.asset }}";
              path = "\${{ matrix.asset }}";
            };
          }
        ];
      };
      jobs.publish = {
        name = "Publish release";
        runsOn = "ubuntu-latest";
        needs = [ "build" ];
        steps = [
          {
            uses = "actions/download-artifact@v4";
            with_ = {
              path = "dist";
              merge-multiple = true;
            };
          }
          {
            uses = "softprops/action-gh-release@v2";
            with_ = {
              files = "dist/*";
              generate_release_notes = true;
            };
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
