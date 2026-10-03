#!/usr/bin/env bash
# Bump the pinned snix-eval revision to the current tip of its default branch.
#
# Updates the three coupled places: Cargo.toml (rev), Cargo.lock (via cargo) and
# nix/constants.nix (fixed-output NAR hash). Leaves the tree ready to commit;
# the caller opens a pull request.
set -euo pipefail

REPO="https://git.snix.dev/snix/snix.git"
CARGO_TOML="Cargo.toml"
CONSTANTS="nix/constants.nix"

emit() {
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf '%s\n' "$1" >>"$GITHUB_OUTPUT"
  else
    printf '%s\n' "$1"
  fi
}

NEW_REV="$(git ls-remote "$REPO" HEAD | awk '{print $1}')"
OLD_REV="$(grep -oP 'rev\s*=\s*"\K[0-9a-f]{40}' "$CARGO_TOML" | head -n1)"

if [ -z "$NEW_REV" ]; then
  echo "could not determine the upstream snix-eval revision" >&2
  exit 1
fi

if [ "$NEW_REV" = "$OLD_REV" ]; then
  echo "snix-eval is already at $NEW_REV"
  emit "changed=false"
  exit 0
fi

echo "snix-eval: $OLD_REV -> $NEW_REV"
sed -i -E "s/rev = \"[0-9a-f]{40}\"/rev = \"$NEW_REV\"/" "$CARGO_TOML"

# Re-resolve the lock. snix-eval and its in-repo macros crate share one git
# source, so resolving either updates both.
cargo update -p snix-eval || cargo update

set_hash() {
  sed -i -E "s|snixSourceHash = \"[^\"]+\"|snixSourceHash = \"$1\"|" "$CONSTANTS"
}

# The fixed-output hash for the new revision is only known once Nix fetches it.
# The first build is expected to fail with a hash mismatch; recover the hash
# from the error, then build for real.
if ! nix build .#tsnix --no-link >/tmp/update-snix-build.log 2>&1; then
  NEW_HASH="$(grep -oP 'got:\s*\Ksha256-[A-Za-z0-9+/=]+' /tmp/update-snix-build.log | head -n1 || true)"
  if [ -z "$NEW_HASH" ]; then
    echo "build failed and no hash mismatch was found:" >&2
    cat /tmp/update-snix-build.log >&2
    exit 1
  fi
  echo "snixSourceHash -> $NEW_HASH"
  set_hash "$NEW_HASH"
fi

# Final self-test: compiles snix-eval and runs the tsnix test suite.
nix build .#tsnix --print-build-logs

emit "changed=true"
emit "rev=$NEW_REV"
