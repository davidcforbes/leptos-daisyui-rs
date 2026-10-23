# pele assets (serve these from your own origin)

`MermaidDisplay` and `MarkdownView` draw Mermaid diagrams with
[pele](https://github.com/obsidianmd/pele) (MIT), run in a dedicated Web
Worker. Nothing is fetched from a CDN. A host serves this directory from its
own origin, at `/pele/` by default; anywhere else needs
`leptos_daisyui_rs::pele::set_pele_asset_base(Some("/your/path"))` before the
first diagram renders.

| File | What it is |
|---|---|
| `pele.global.js` | pele's browser bundle, built from the fixed fork commit below (banner `Pele 1.9.0`) |
| `ldui-pele-worker.js` | this crate's worker: `importScripts('pele.global.js')`, then one `pele.render` per message |
| `LICENSE-pele` | pele's MIT licence |

## Provenance of `pele.global.js` (ldui-o55o, 2026-10-22)

- **Source:** `https://github.com/davidcforbes/pele`, branch `perf-fixes`,
  commit `4d3aaf68e87f7a4c9146c8300fbb14ec82d1af2a`. That is 26 commits on
  upstream `obsidianmd/pele` tag `1.9.0` (`e7f092a`), and it fixes every
  hostile-input hang and robustness bug found by the pele review (pele beads
  epic `pele-qd0`). `package.json` and `package-lock.json` are unchanged
  from 1.9.0, so it adds no dependencies. This is **not** an npm release, so
  there is no npm integrity value or attestation. Switch back to the official
  npm build once upstream ships these fixes.
- **SHA-256** `f2c82fb8abe4768937735692d6e2ca42ffad6371615a2d031f1f04c4bc686868`
  (709,113 bytes), pinned in `src/pele.rs` (`PELE_BUNDLE_SHA256`, with
  `PELE_SOURCE` naming the commit) and checked by the gate's `test-lib` step.
  `.gitattributes` marks the file `-text`, so a CRLF checkout cannot change
  its bytes.
- **Reproduced independently:** a fresh clone at that commit, `npm ci` and
  `npm run build` (Node 26.12.1, Windows) produce this exact file. The pele
  session reproduced it on its own as well, and its earlier check of 1.8.0
  showed the build reproducing byte for byte across OS and Node versions,
  because esbuild is pinned by the lockfile.
- **Scanned:** like 1.8.0, the bundle contains no `eval`, `Function(`,
  `fetch`, `XMLHttpRequest`, `WebSocket`, `sendBeacon`, `import(`,
  `Worker`, storage or cookie access.
- **Reviewed:** six review agents, a 305-input injection sweep on both 1.8.0
  and this build, and 2,283 docs-corpus renders byte-identical to upstream.
  No injection path was found. The sanitiser in `src/pele.rs` is the
  recommended second layer, and the worker's time limit stays as defence in
  depth against hangs nobody has found yet.

The previously vendored 1.8.0 bundle (SHA-256 `9bf34b3a…53d0`, npm-attested,
reproduced from tag `1.8.0`) is in git history at commit `a85f8c9`.

## Content-Security-Policy

The worker needs `script-src 'self'` (a `worker-src` falls back to it) and
nothing else: pele uses no `eval`, `new Function`, `fetch` or dynamic
import. A `blob:` worker would need `blob:` in `script-src`, which is why
this is a served file. The drawn SVG uses inline `style="..."` attributes, so
a `style-src` without `'unsafe-inline'` breaks the colours but nothing else.

## Upgrading

Never edit `pele.global.js`. To move to another build:
1. Copy the new `dist/pele.global.js` and recompute its SHA-256.
2. Update `PELE_VERSION`, `PELE_SOURCE` and `PELE_BUNDLE_SHA256` together.
3. Repeat the reproduce-from-source check.
4. Run `cargo xtask test-mermaid-pele`. Its `data-pele-stripped="0"`
   assertions show whether the sanitiser allow-list (`SVG_ELEMENTS`,
   `SVG_ATTRIBUTES`) still covers what the new build draws.
