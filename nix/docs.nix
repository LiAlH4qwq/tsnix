{ ... }:
{
  perSystem =
    { config, pkgs, ... }:
    let
      # The language chooser served at the site root; both books are deployed
      # next to it.
      landing = pkgs.writeText "index.html" ''
        <!DOCTYPE html>
        <html lang="en">
          <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <title>tsnix</title>
            <style>
              :root { color-scheme: light dark; }
              body {
                margin: 0;
                min-height: 100vh;
                display: flex;
                flex-direction: column;
                gap: 1.5rem;
                align-items: center;
                justify-content: center;
                font-family: system-ui, sans-serif;
              }
              h1 { margin: 0; font-size: 2.5rem; }
              p { margin: 0; opacity: 0.75; }
              nav { display: flex; gap: 1rem; }
              a {
                padding: 0.6rem 1.2rem;
                border: 1px solid currentColor;
                border-radius: 0.5rem;
                text-decoration: none;
                color: inherit;
              }
            </style>
          </head>
          <body>
            <h1>tsnix</h1>
            <p>A minimal Nix-language evaluator &mdash; no store</p>
            <nav>
              <a href="en/">English</a>
              <a href="zh/">&#x7B80;&#x4F53;&#x4E2D;&#x6587;</a>
            </nav>
          </body>
        </html>
      '';

    in
    {
      # Bilingual mdBook site using mdBook's built-in themes.
      packages.docs =
        pkgs.runCommandLocal "tsnix-docs"
          {
            nativeBuildInputs = [ pkgs.mdbook ];
          }
          ''
            cp -r ${../docs} book
            chmod -R u+w book
            mdbook build book -d $out/en
            mdbook build book/zh -d $out/zh
            cp ${landing} $out/index.html
          '';

      # API documentation built with cargo doc.
      packages.rustdoc = config.packages.tsnix.overrideAttrs (old: {
        pname = "tsnix-rustdoc";
        doCheck = false;
        buildPhase = "cargo doc --offline --no-deps --document-private-items";
        installPhase = ''
          mkdir -p $out/share/doc/tsnix
          cp -r target/doc/* $out/share/doc/tsnix/
        '';
      });
    };
}
