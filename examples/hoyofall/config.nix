# Example: split a hoyofall-style config into files and render JSON with tsnix.
#
#   tsnix eval -f examples/hoyofall/config.nix --io local --pretty
#
# This exercises relative imports, attrsets, lists and string interpolation
# without any Nix store. To inject values from the command line, reference
# variables bound by `--arg`/`--argstr` (they are ordinary top-level names).
{
  convert = {
    emitBuiltinOutbounds = false;
    proxyNameFormat = "{sub}-{name}";
  };

  output = {
    file = {
      enabled = true;
      path = "hoyofall.json";
      pretty = true;
    };
    http = {
      enabled = true;
      listen = {
        host = "127.0.0.1";
        port = 9090;
      };
    };
  };

  subscriptions = import ./subscriptions.nix;
}
