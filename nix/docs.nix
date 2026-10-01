{
  inputs,
  ...
}:
{
  perSystem =
    { config, pkgs, ... }:
    let
      # Rose Pine Dawn palette, shared by the landing page below and matching
      # the mdBook `rose-pine-dawn` theme vendored from the milieuim theme.
      rp = {
        base = "#faf4ed";
        surface = "#fffaf3";
        overlay = "#f2e9e1";
        text = "#575279";
        subtle = "#797593";
        muted = "#9893a5";
        love = "#b4637a";
        gold = "#ea9d34";
        pine = "#286983";
        foam = "#56949f";
        iris = "#907aa9";
        rose = "#d7827e";
      };

      # The language chooser served at the site root; both books are deployed
      # next to it. Self-contained: inline CSS, a little vanilla JS and inline
      # SVG, so it needs no network access at build or run time.
      landing = pkgs.writeText "index.html" ''
        <!DOCTYPE html>
        <html lang="en">
          <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta name="theme-color" content="${rp.base}" />
            <title>tsnix &mdash; the tiny snix-eval CLI</title>
            <style>
              :root {
                --rp-base: ${rp.base};
                --rp-surface: ${rp.surface};
                --rp-overlay: ${rp.overlay};
                --rp-text: ${rp.text};
                --rp-subtle: ${rp.subtle};
                --rp-muted: ${rp.muted};
                --rp-love: ${rp.love};
                --rp-gold: ${rp.gold};
                --rp-pine: ${rp.pine};
                --rp-foam: ${rp.foam};
                --rp-iris: ${rp.iris};
                --rp-rose: ${rp.rose};
                color-scheme: light;
              }

              * { box-sizing: border-box; }

              html, body { height: 100%; }

              body {
                margin: 0;
                font-family: ui-sans-serif, system-ui, -apple-system, "Segoe UI",
                  Roboto, "Helvetica Neue", sans-serif;
                color: var(--rp-text);
                background: var(--rp-base);
                overflow-x: hidden;
              }

              /* Soft, slowly drifting aurora blobs in the background. */
              .bg { position: fixed; inset: 0; z-index: -1; overflow: hidden; }
              .bg span {
                position: absolute;
                display: block;
                width: 46vmax;
                height: 46vmax;
                border-radius: 50%;
                filter: blur(70px);
                opacity: 0.5;
                animation: drift 26s ease-in-out infinite alternate;
              }
              .bg span:nth-child(1) {
                top: -18vmax; left: -12vmax;
                background: radial-gradient(circle, var(--rp-rose), transparent 70%);
              }
              .bg span:nth-child(2) {
                bottom: -22vmax; right: -14vmax;
                background: radial-gradient(circle, var(--rp-iris), transparent 70%);
                animation-delay: -8s;
              }
              .bg span:nth-child(3) {
                top: 30%; left: 55%;
                background: radial-gradient(circle, var(--rp-foam), transparent 70%);
                animation-delay: -16s;
              }
              @keyframes drift {
                from { transform: translate3d(0, 0, 0) scale(1); }
                to   { transform: translate3d(6vmax, -4vmax, 0) scale(1.15); }
              }

              .wrap {
                min-height: 100vh;
                display: flex;
                flex-direction: column;
                gap: 1.5rem;
                align-items: center;
                justify-content: center;
                padding: 3rem 1.25rem 4rem;
                text-align: center;
              }

              @font-face {
                font-family: 'Quicksand';
                font-style: normal;
                font-weight: 600;
                font-display: swap;
                src: url('tsnix-600.woff2') format('woff2');
              }
              h1 {
                margin: 0;
                font-family: 'Quicksand', 'ui-rounded', system-ui, sans-serif;
                font-weight: 600;
                font-size: clamp(2.75rem, 9vw, 4.5rem);
                letter-spacing: -0.02em;
                color: var(--rp-text);
                animation: pop 0.7s cubic-bezier(0.22, 1, 0.36, 1) both;
              }
              @keyframes pop {
                from { transform: scale(0.9); opacity: 0; }
                to   { transform: scale(1); opacity: 1; }
              }

              .tagline {
                margin: 0;
                min-height: 1.6em;
                font-size: clamp(1rem, 2.6vw, 1.3rem);
                color: var(--rp-subtle);
              }
              .caret {
                display: inline-block;
                width: 0.6ch;
                margin-left: 0.15ch;
                border-right: 2px solid var(--rp-rose);
                animation: blink 1s steps(2, start) infinite;
                vertical-align: -0.1em;
              }
              @keyframes blink { 50% { border-color: transparent; } }

              .terminal {
                width: min(640px, 100%);
                text-align: left;
                background: var(--rp-surface);
                border: 1px solid var(--rp-overlay);
                border-radius: 12px;
                box-shadow: 0 18px 40px -24px rgba(87, 82, 121, 0.55);
                overflow: hidden;
                font-family: ui-monospace, "SFMono-Regular", "Source Code Pro",
                  Menlo, Consolas, monospace;
                font-size: 0.9rem;
              }
              .bar {
                display: flex;
                gap: 0.5rem;
                padding: 0.7rem 0.9rem;
                background: var(--rp-overlay);
              }
              .bar i {
                width: 11px; height: 11px; border-radius: 50%;
                display: inline-block;
              }
              .bar i:nth-child(1) { background: var(--rp-love); }
              .bar i:nth-child(2) { background: var(--rp-gold); }
              .bar i:nth-child(3) { background: var(--rp-foam); }
              .term-body { padding: 1rem 1.1rem 1.2rem; }
              .line {
                white-space: pre-wrap;
                word-break: break-word;
                opacity: 0;
                transform: translateY(4px);
                animation: reveal 0.45s ease forwards;
              }
              .line + .line { margin-top: 0.55rem; }
              .line:nth-child(1) { animation-delay: 0.9s; }
              .line:nth-child(2) { animation-delay: 1.5s; }
              .line:nth-child(3) { animation-delay: 2.1s; }
              @keyframes reveal {
                to { opacity: 1; transform: translateY(0); }
              }
              .prompt { color: var(--rp-pine); }
              .str { color: var(--rp-gold); }
              .key { color: var(--rp-iris); }
              .cmt { color: var(--rp-muted); font-style: italic; }

              .features {
                display: flex;
                flex-wrap: wrap;
                gap: 0.5rem;
                justify-content: center;
                max-width: 640px;
              }
              .chip {
                padding: 0.35rem 0.8rem;
                border: 1px solid var(--rp-overlay);
                background: var(--rp-surface);
                border-radius: 999px;
                font-size: 0.82rem;
                color: var(--rp-subtle);
                transition: transform 0.18s ease, color 0.18s ease,
                  border-color 0.18s ease;
              }
              .chip:hover {
                transform: translateY(-2px);
                color: var(--rp-text);
                border-color: var(--rp-iris);
              }

              nav {
                display: flex;
                flex-wrap: wrap;
                gap: 0.75rem;
                justify-content: center;
              }
              a.btn {
                display: inline-flex;
                align-items: center;
                gap: 0.5rem;
                padding: 0.6rem 1.2rem;
                border: 1px solid currentColor;
                border-radius: 0.6rem;
                text-decoration: none;
                color: var(--rp-text);
                background: var(--rp-surface);
                transition: transform 0.18s ease, background 0.18s ease,
                  color 0.18s ease, box-shadow 0.18s ease;
              }
              a.btn:hover {
                transform: translateY(-2px);
                background: var(--rp-iris);
                border-color: var(--rp-iris);
                color: var(--rp-base);
                box-shadow: 0 12px 24px -14px var(--rp-iris);
              }
              a.btn svg { width: 1.1em; height: 1.1em; fill: currentColor; }
              a.btn.primary {
                background: var(--rp-iris);
                border-color: var(--rp-iris);
                color: var(--rp-base);
              }
              a.btn.primary:hover { background: var(--rp-pine); border-color: var(--rp-pine); }

              footer {
                margin-top: 0.5rem;
                font-size: 0.78rem;
                color: var(--rp-muted);
              }
              footer a { color: var(--rp-iris); text-decoration: none; }
              footer a:hover { text-decoration: underline; }

              @media (prefers-reduced-motion: reduce) {
                .bg span,
                h1,
                .caret { animation: none; }
                .line { opacity: 1; transform: none; animation: none; }
              }
            </style>
          </head>
          <body>
            <div class="bg" aria-hidden="true"><span></span><span></span><span></span></div>

            <div class="wrap">
              <h1>tsnix</h1>

              <p class="tagline">
                <span id="tagline">A minimal Nix-language evaluator.</span><span class="caret" aria-hidden="true"></span>
              </p>

              <div class="terminal" aria-label="tsnix example">
                <div class="bar" aria-hidden="true"><i></i><i></i><i></i></div>
                <div class="term-body">
                  <div class="line"><span class="prompt">$</span> tsnix eval -e '{ a = 1; b = [ true null ]; }'</div>
                  <div class="line">{<span class="key">"a"</span>:1,<span class="key">"b"</span>:[<span class="str">true</span>,<span class="str">null</span>]}</div>
                  <div class="line cmt"># no store, no daemon, just JSON</div>
                </div>
              </div>

              <div class="features">
                <span class="chip">stdout = payload</span>
                <span class="chip">stderr = diagnostics</span>
                <span class="chip">stable error codes</span>
                <span class="chip">pure std library</span>
                <span class="chip">embeddable</span>
              </div>

              <nav>
                <a class="btn primary" href="en/">English docs</a>
                <a class="btn" href="zh/">&#x7B80;&#x4F53;&#x4E2D;&#x6587;</a>
                <a class="btn" href="https://github.com/LiAlH4qwq/tsnix">
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .5C5.37.5 0 5.78 0 12.29c0 5.21 3.44 9.63 8.21 11.19.6.11.82-.25.82-.56 0-.28-.01-1.02-.02-2-3.34.71-4.04-1.58-4.04-1.58-.55-1.37-1.34-1.74-1.34-1.74-1.09-.73.08-.71.08-.71 1.2.08 1.84 1.22 1.84 1.22 1.07 1.8 2.81 1.28 3.5.98.11-.76.42-1.28.76-1.57-2.67-.3-5.47-1.31-5.47-5.84 0-1.29.47-2.34 1.24-3.17-.12-.3-.54-1.52.12-3.18 0 0 1.01-.32 3.3 1.21a11.6 11.6 0 0 1 3-.4c1.02 0 2.05.13 3 .4 2.28-1.53 3.29-1.21 3.29-1.21.66 1.66.24 2.88.12 3.18.77.83 1.24 1.88 1.24 3.17 0 4.54-2.81 5.53-5.49 5.82.43.37.82 1.1.82 2.22 0 1.6-.02 2.9-.02 3.29 0 .31.22.67.83.56A12.02 12.02 0 0 0 24 12.29C24 5.78 18.63.5 12 .5z"/></svg>
                  GitHub
                </a>
              </nav>

              <footer>
                GPL-3.0-only &middot; built on
                <a href="https://snix.dev">snix-eval</a>
                &middot;
                <a href="https://github.com/LiAlH4qwq/tsnix/issues">icon design wanted</a>
              </footer>
            </div>

            <script>
              (function () {
                var phrases = [
                  "A minimal Nix-language evaluator.",
                  "The tiny snix-eval CLI.",
                  "No store. No daemon. Just JSON.",
                  "Embeddable where nix cannot go.",
                  "Pure, deterministic, fast."
                ];
                var el = document.getElementById("tagline");
                var reduce = window.matchMedia
                  && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
                if (!el || reduce) { return; }

                var pi = 0, ci = phrases[0].length, deleting = true;
                el.textContent = phrases[0];

                function tick() {
                  var p = phrases[pi];
                  if (!deleting) {
                    ci++;
                    el.textContent = p.slice(0, ci);
                    if (ci >= p.length) {
                      deleting = true;
                      setTimeout(tick, 1800);
                      return;
                    }
                    setTimeout(tick, 55);
                  } else {
                    ci--;
                    el.textContent = p.slice(0, ci);
                    if (ci <= 0) {
                      deleting = false;
                      pi = (pi + 1) % phrases.length;
                      setTimeout(tick, 400);
                      return;
                    }
                    setTimeout(tick, 26);
                  }
                }
                setTimeout(tick, 1600);
              })();
            </script>
          </body>
        </html>
      '';

      # The milieuim theme targets mdBook 0.4, while nixpkgs ships mdBook 0.5.
      # Instead of porting that stylesheet (which keeps drifting as 0.5 adds
      # markup such as the help popup, the on-this-page list and the sidebar
      # resize handles), keep mdBook 0.5's own built-in CSS/JS/`index.hbs` and
      # take only the colour scheme, fonts and favicons from the theme. The
      # rose-pine-dawn palette lives in the theme's `rose-pine-dawn.css`, loaded
      # via `additional-css` and selected with `default-theme`.
      theme = pkgs.runCommandLocal "tsnix-mdbook-theme" { } ''
        mkdir -p $out
        cp -r ${inputs.mdbook-theme-milieuim}/* $out/
        chmod -R u+w $out
        rm -rf $out/css $out/index.hbs $out/book.js $out/highlight.js $out/highlight.css
      '';

    in
    {
      # Bilingual mdBook site using the vendored milieuim theme with the
      # rose-pine-dawn colour scheme, plus a themed landing page.
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
            # The landing page loads the wordmark webfont directly.
            cp ${../docs/fonts/tsnix-600.woff2} $out/tsnix-600.woff2
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
