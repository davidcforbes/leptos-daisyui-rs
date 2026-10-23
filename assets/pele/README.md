# pele assets (serve these from your own origin)

`MermaidDisplay` and `MarkdownView` draw Mermaid diagrams with
[pele](https://github.com/obsidianmd/pele) (MIT), run in a dedicated Web
Worker. Nothing is fetched from a CDN. A host serves this directory from its
own origin, at `/pele/` by default; anywhere else needs
`leptos_daisyui_rs::pele::set_pele_asset_base(Some("/your/path"))` before the
first diagram renders.

| File | What it is |
|---|---|
| `pele.global.js` | pele **1.8.0**, `dist/pele.global.js` from the npm package, byte for byte |
| `ldui-pele-worker.js` | this crate's worker: `importScripts('pele.global.js')`, then one `pele.render` per message |
| `LICENSE-pele` | pele's MIT licence |

## Provenance of `pele.global.js` (ldui-o55o, 2026-10-22)

- SHA-256 `9bf34b3a9a19351dc69d0d238e365560331a8ae392c41656df7f5090653253d0`,
  pinned in `src/pele.rs` (`PELE_BUNDLE_SHA256`) and checked by the gate's
  `test-lib` step. `.gitattributes` marks the file `-text` so a CRLF checkout
  cannot change its bytes.
- npm integrity `sha512-AYmHttE70cwNJVXdfPMCANGivFlmc3bGGpMG0GqH5S5vE3RgtEuLlkqbhRfe94Csv/ZEd1TUbv2dMUHpw4p1Bg==`;
  the release carries an npm provenance attestation
  (`https://registry.npmjs.org/-/npm/v1/attestations/pele@1.8.0`) from
  pele's GitHub Actions release workflow.
- **Reproduced from source**: tag `1.8.0` (commit `8fad23b`), `npm ci` and
  `npm run build` produce this exact file. On Windows the build's final
  `.d.ts` step fails (`npx` ENOENT), but only after the bundles are written.
- Byte-identical to editmark's vendored copies
  (`editmark-pele/vendor/pele.global.js`), which a gate test also checks
  when the sibling checkout is present.
- Reviewed for injection and stability by a dedicated session (pele beads
  epic `pele-qd0` in `C:\dev\pele`). No script-injection path was found. Several
  hostile inputs under pele's 50 KB limit run for 5 to 20+ seconds, which is
  why rendering runs in this worker under a time limit, and why
  `src/pele.rs` still passes the output through an allow-list sanitiser.

## Content-Security-Policy

The worker needs `script-src 'self'` (a `worker-src` falls back to it) and
nothing else: pele uses no `eval`, `new Function`, `fetch` or dynamic
import. A `blob:` worker would need `blob:` in `script-src`, which is why
this is a served file. The drawn SVG uses inline `style="..."` attributes, so
a `style-src` without `'unsafe-inline'` breaks the colours but nothing else.

## Upgrading

Never edit `pele.global.js`. To move to another release, copy the new
`dist/pele.global.js`, recompute its SHA-256, update `PELE_VERSION` and
`PELE_BUNDLE_SHA256` together, repeat the reproduce-from-tag check, and
re-check the sanitiser allow-list (1.9.0 adds `defs`, `linearGradient` and
`stop` for sankey gradients).
