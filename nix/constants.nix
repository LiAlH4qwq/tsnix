{
  # NAR hash of the pinned snix-eval git source (see Cargo.toml). Both
  # `snix-eval` and its in-repo `snix-eval-builtin-macros` path dependency come
  # from the same checkout, so they share a hash.
  #
  # To refresh after bumping the revision in Cargo.toml:
  #   nix build .#tsnix 2>&1 | grep 'got:'
  snixSourceHash = "sha256-Qgq9HK47tyyD2XhfDzGjXdz73tGVPZJ/iVghsqmorUA=";
}
