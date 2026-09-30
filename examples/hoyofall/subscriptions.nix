# Imported by config.nix. Keeping this in a separate file demonstrates that
# `--io local` resolves relative imports.
{
  example = {
    name = "example";
    url = "https://example.com/subscribe?token=REPLACE_ME";
    intervalSeconds = 3600;
    format = "auto";
    onUnsupported = "skip";
  };
}
