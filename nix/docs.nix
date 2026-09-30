{
  inputs,
  lib,
  ...
}:
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

      # The milieuim theme ships the stock mdBook 0.4 assets. nixpkgs ships
      # mdBook 0.5, so port the theme to its DOM: drop 0.4's index.hbs/book.js
      # in favour of 0.5's built-ins, prefix the element ids 0.5 renamed with
      # `mdbook-` (e.g. `#menu-bar` -> `#mdbook-menu-bar`) and replace the
      # FontAwesome icon-font selectors with 0.5's inline `.fa-svg` SVGs.
      # `mdbook-0.5-compat.css` adds the rules 0.4 never had (SVG sizing, the
      # new copy button, per-theme copy-button filters).
      theme = pkgs.runCommandLocal "milieuim-theme" { } ''
        mkdir -p $out
        cp -r ${inputs.mdbook-theme-milieuim}/* $out/
        chmod -R u+w $out
        rm -f $out/index.hbs $out/book.js $out/highlight.js

        substituteInPlace $out/css/chrome.css \
          --replace-fail '#menu-bar i' '#menu-bar .fa-svg' \
          --replace-fail '.icon-button i' '.icon-button .fa-svg' \
          --replace-fail '.menu-bar a i' '.menu-bar a .fa-svg' \
          --replace-fail '.menu-bar i:hover' '.menu-bar .fa-svg:hover' \
          --replace-fail '.mobile-nav-chapters i:hover' '.mobile-nav-chapters .fa-svg:hover' \
          --replace-fail 'pre > .buttons i' 'pre > .buttons .fa-svg' \
          --replace-fail '.no-js' ':not(.js)' \
          --replace-fail '#menu-bar' '#mdbook-menu-bar' \
          --replace-fail '#sidebar' '#mdbook-sidebar' \
          --replace-fail '#body-container' '#mdbook-body-container' \
          --replace-fail '#searchbar' '#mdbook-searchbar' \
          --replace-fail '#searchresults' '#mdbook-searchresults'

        substituteInPlace $out/css/general.css \
          --replace-fail '.no-js' ':not(.js)'

        substituteInPlace $out/css/print.css \
          --replace-fail '#menu-bar' '#mdbook-menu-bar' \
          --replace-fail '#sidebar' '#mdbook-sidebar' \
          --replace-fail '#page-wrapper' '#mdbook-page-wrapper' \
          --replace-fail '#content' '#mdbook-content'

        substituteInPlace $out/css/variables.css \
          --replace-fail '.no-js' ':not(.js)'

        cat ${./mdbook-0.5-compat.css} >> $out/css/general.css
      '';
    in
    {
      # Bilingual mdBook site using the milieuim theme.
      packages.docs =
        pkgs.runCommandLocal "tsnix-docs"
          {
            nativeBuildInputs = [ pkgs.mdbook ];
          }
          ''
            cp -r ${../docs} book
            chmod -R u+w book
            cp -r ${theme} book/theme
            cp -r ${theme} book/zh/theme
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
