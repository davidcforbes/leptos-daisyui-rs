# Self-hosting / vendoring a JS LaTeX-math renderer (KaTeX, Temml, MathJax 4) under a strict CSP — as of 2026-10-23

Method note: version, date, size, licence and provenance figures come from the npm registry API and the jsDelivr file-listing API, both queried live on 2026-10-23. "Measured" findings come from fetching the published bundles from jsDelivr and running them in a Node/Bun `vm` context that has **no `document`/`window`**, which is a stand-in for a Web Worker. Advisory data comes from the GitHub REST API (`/repos/{repo}/security-advisories` and `/advisories`). Sizes are uncompressed bytes as published (B); 1 KiB = 1024 B.

## KaTeX: version, cadence, package contents, eval, MathML-only, worker use, inline styles, options, advisories, provenance, licence

### Takeaway
KaTeX's current version is **0.19.0 (2026-10-01)**. It is MIT-licensed, has **no eval/`new Function`**, and its `renderToString` runs with no DOM, so it fits the existing `assets/pele/` worker pattern directly. A full self-host is about **545 KiB** (JS 266 KiB, CSS 24 KiB, 20 woff2 fonts 254 KiB). The HTML output is full of inline `style="…"` attributes. `output:'mathml'` emits **no** style attributes and needs **no** CSS or fonts, but then layout is left to the browser's native MathML. Six GHSA advisories exist, and every one is fixed in ≥ 0.18.2. There is **no npm provenance** attestation.

### Cited Findings
**Version and release cadence**
- The npm `latest` tag is `katex@0.19.0`, published 2026-10-01; licence is `MIT`; the only runtime dependency is `commander ^15.0.0` (for the CLI) — [npm registry: katex](https://registry.npmjs.org/katex)
- Release cadence, counted from the npm `time` map ([npm registry: katex](https://registry.npmjs.org/katex)):
  - **2024**: 10 releases (0.16.10 on 2024-03-24 through 0.16.19 on 2024-12-29).
  - **2025**: 8 releases (0.16.20 on 2025-01-12 through 0.16.27 on 2025-12-07).
  - **2026 to 1 October**: 34 releases (0.16.28 through 0.16.47, then 0.17.0 on 2026-05-22, 0.18.0 on 2026-07-17, 0.18.1–0.18.11, and 0.19.0 on 2026-10-01).
  - So the project has been in a fast-moving phase in 2026, with three breaking minor bumps in five months.
- Breaking changes in 2026 ([KaTeX CHANGELOG](https://raw.githubusercontent.com/KaTeX/KaTeX/main/CHANGELOG.md)):
  - **0.17.0** (2026-05-22): the internal `__defineFunction` API changed.
  - **0.18.0** (2026-07-17): "prefix css classes (#4229)". The changelog says "users who apply custom styles or have **allowlists targeting KaTeX's internal classes must update their selectors**". This matters directly to an allow-list sanitiser.
  - **0.19.0** (2026-10-01): missing character metrics are now reported through `strict` with code `symbolNotInFont`. Strict callbacks that return `undefined`/`null` now warn.
- Measured 0.19.0 output shows prefixed classes such as `katex-base` and `katex-strut`, alongside still-unprefixed ones such as `mord` and `vlist-t`. The prefixing is therefore partial, and any class allow-list must be regenerated per KaTeX version — measured from [katex@0.19.0 katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- Other 2025–26 changelog items relevant to self-hosting ([KaTeX CHANGELOG](https://raw.githubusercontent.com/KaTeX/KaTeX/main/CHANGELOG.md)):
  - 0.16.25 (2025-10) added `katex-swap.css`, which uses `font-display: swap`.
  - 0.18.10 (2026-09) makes KaTeX "fallback to browser math font before Times New Roman".
  - 0.16.36 fixed "contrib esm bloat".

**Package contents and sizes (katex@0.19.0)**
- The npm tarball is 4,043,696 B unpacked, in 225 files — [npm registry: katex](https://registry.npmjs.org/katex)
- Per-file sizes ([jsDelivr listing katex@0.19.0](https://data.jsdelivr.com/v1/packages/npm/katex@0.19.0?structure=flat)):

| File | Size |
|---|---|
| `dist/katex.min.js` | **272,868 B** |
| `dist/katex.min.css` | **24,793 B** |
| `dist/katex.js` | 621,053 B |
| `dist/katex.mjs` | 602,889 B |
| `dist/katex.css` | 33,084 B |
| `contrib/auto-render.min.js` | 3,486 B |
| `contrib/mhchem.min.js` | 33,706 B |
| `contrib/copy-tex.min.js` | 1,671 B |

- Fonts, from the same [jsDelivr listing](https://data.jsdelivr.com/v1/packages/npm/katex@0.19.0?structure=flat):
  - `dist/fonts/` holds **20 font faces × 3 formats**: woff2 = 20 files totalling **259,792 B**; woff = 20 files, 303,116 B; ttf = 20 files, 513,664 B.
  - The largest woff2 files are KaTeX_AMS-Regular (28,076 B), KaTeX_Main-Regular (26,272 B) and KaTeX_Main-Bold (25,324 B).
  - The smallest is KaTeX_Size3-Regular (3,624 B).
- `katex.min.css` declares **20 `@font-face`** rules. Their `src` lists woff2, woff and ttf with **relative** URLs such as `url(fonts/KaTeX_AMS-Regular.woff2)`, so the CSS works unmodified if a `fonts/` directory sits next to it. No URL rewriting is needed unless the layout changes — measured from [katex.min.css](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.css)

**eval and `new Function`**
- Static scan of `katex.min.js` 0.19.0: **0** `eval(`, **0** `new Function(`, **0** `Function(` calls and **0** `innerHTML`. There are 15 `document.` references, all in the DOM-building `render()` path. `script-src 'self'` without `'unsafe-eval'` is therefore sufficient — measured from [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)

**Rendering in a Web Worker (no DOM)**
- Loading `katex.min.js` in a `vm` context with no `document`/`window`/`self` beyond a stub succeeded (`katex.version === "0.19.0"`). `renderToString` then produced output for every test formula, in both `htmlAndMathml` and `mathml` modes. So `renderToString` needs no DOM and will work after `importScripts()` in a worker; only `katex.render(el)` touches `document` — measured from [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- The exported API is `version`, `render`, `renderToString`, `ParseError` and `SETTINGS_SCHEMA`, plus `__`-prefixed internals — measured, same source.

**Inline styles in the HTML output (relevant to `style-src`)**
- Measured on 0.19.0 with default `htmlAndMathml` output:

| Formula | `style="…"` attributes |
|---|---|
| `x^2+\frac{a}{b}` | 16 |
| `\sqrt[3]{x}\left(\sum_{i=1}^n i\right)` | 24 |
| `\begin{pmatrix}…\end{pmatrix}` | 21 |

  - The CSS properties seen were `height`, `vertical-align`, `top`, `bottom`, `margin-left`, `margin-right`, `padding-left`, `min-width`, `width`, `border-bottom-width`, `border-top-width`, `border-right-width`, `color` and `background-color`.
  - Error spans (`throwOnError:false`) carry `style="color:#cc0000"` (the `errorColor`).
  - Source: [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- KaTeX's own non-minified source also calls `setAttribute("style", …)` in 3 places in its MathML builders. Examples are `text-shadow` on an `<mstyle>` for the fake-bold fallback, and `border: … solid …` for `\fbox`-type MathML. The HTML `toNode()` path builds elements with `document.createElement` and applies "inline styles" on the node object — measured from [katex.js 0.19.0](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.js)

**MathML-only output (`output: 'mathml'`)**
- `output:'mathml'` emits `<span class="katex"><math xmlns=…><semantics>…<annotation encoding="application/x-tex">…</annotation></semantics></math></span>`. Measured examples:
  - `x^2+\frac{a}{b}` is 266 characters, against 1,998 for HTML+MathML.
  - **No `style` attributes** appeared in any test formula.
  - Colour is expressed with MathML attributes (`mathcolor`, `mathbackground`), and `\colorbox` with `<mpadded … mathbackground="yellow">`.
  - Matrices use `<mtable rowspacing columnalign columnspacing>`.
  - Source: [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- The `output` option accepts `"htmlAndMathml"` (default), `"html"` or `"mathml"` (from `SETTINGS_SCHEMA`) — measured, same source.

**`trust`, `maxSize` and `maxExpand`**
- `trust` takes a boolean or a function and defaults to false. With `trust:false`, these all rendered as red error text rather than links, images or attributes: `\href{javascript:alert(1)}{x}`, `\htmlStyle{color:red}{x}`, `\includegraphics{http://e/x.png}` and `\htmlData{foo=bar}{x}`. With `trust:true`, `\href{javascript:alert(1)}` emitted `href="javascript:alert(1)"` (twice: once in the MathML and once in the HTML). `trust:true` does **not** filter dangerous protocols; only a `trust` *function* can — measured from [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- The documented `trust` function receives a context with `command`, `url` and `protocol`. For example, `trust: (context) => ['\\url','\\href'].includes(context.command) && ['http','https','_relative'].includes(context.protocol)` — [KaTeX Options](https://katex.org/docs/options)
- `maxSize` caps user-specified sizes in ems and defaults to Infinity. `maxExpand` limits macro expansions, defaults to 1000, and counts every token `\edef` expands — [KaTeX Options](https://katex.org/docs/options)
- A measured check: `\def\a{\a\a}\a` throws "Too many expansions: infinite loop or need to increase maxExpand setting" under the default settings — measured from [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js)
- KaTeX's own security page says: "Any HTML generated by KaTeX *should* be safe from `<script>` or other code injection attacks. Of course, it is always a good idea to sanitize the HTML, though you will need a rather generous whitelist (including some of SVG and MathML)". It also warns: "The error message thrown by KaTeX may contain unescaped LaTeX source code." — [KaTeX Security](https://katex.org/docs/security)

**Security advisories** (6 in the repo, 6 in the GitHub Advisory DB for npm `katex`)

| Advisory | CVE | Severity / published | Affected | Fixed in |
|---|---|---|---|---|
| GHSA-cvr6-37gx-v8wc: maxExpand bypassed by Unicode sub/superscripts | CVE-2024-28244 | medium, 2024-03-24 | 0.15.4 to < 0.16.10 | 0.16.10 |
| GHSA-64fm-8hw2-v72w: maxExpand bypassed by `\edef` | CVE-2024-28243 | medium | ≥ 0.10.0-beta (the global DB says ≥ 0.12.0) to < 0.16.10 | 0.16.10 |
| GHSA-3wc5-fcw2-2329: missing protocol normalisation in URLs bypasses forbidden protocols | CVE-2024-28246 | medium | ≥ 0.11.0 to < 0.16.10 | 0.16.10 |
| GHSA-f98w-7cxr-ff2h: `\includegraphics` does not escape filename | CVE-2024-28245 | medium | global DB: ≥ 0.11.0 to < 0.16.10 | 0.16.10 |
| GHSA-cg87-wmx4-v546: `\htmlData` does not validate attribute names | CVE-2025-23207 | medium, 2025-01-17 | ≥ 0.12.0 to ≤ 0.16.20 | 0.16.21 |
| GHSA-238p-pmpm-9mq7: existing prototype pollution can bypass KaTeX trust restrictions | CVE-2026-103923 | **low**; repo 2026-09-21, global DB 2026-10-05 | ≥ 0.11.0 to < 0.18.2 | 0.18.2 |

  - Sources: [GitHub API: KaTeX repo advisories](https://api.github.com/repos/KaTeX/KaTeX/security-advisories); [GitHub Advisory DB (npm katex)](https://api.github.com/advisories?ecosystem=npm&affects=katex)
  - GHSA-f98w: the repo-level advisory says "< 0.6.10" and patched "0.6.10", which is evidently a typo — [GitHub API](https://api.github.com/repos/KaTeX/KaTeX/security-advisories); the global DB says < 0.16.10 / patched 0.16.10 — [GitHub Advisory DB](https://api.github.com/advisories?ecosystem=npm&affects=katex)
  - GHSA-cg87 (`\htmlData`): with `trust` enabled, "malicious input using `\htmlData` … runs arbitrary JavaScript". The listed workarounds include "Sanitize HTML output from KaTeX" — [GHSA-cg87-wmx4-v546](https://github.com/advisories/GHSA-cg87-wmx4-v546)
  - GHSA-238p (prototype pollution): "an inherited `trust` value will be treated as though the application explicitly enabled trusted rendering … Exploitation requires the consuming application to insert KaTeX output into a web page without a separate sanitizer." The changelog entry for 0.18.2 (2026-08-08) is "prevent prototype pollution in settings (#4260)" — [GHSA-238p-pmpm-9mq7](https://github.com/advisories/GHSA-238p-pmpm-9mq7); [KaTeX CHANGELOG](https://raw.githubusercontent.com/KaTeX/KaTeX/main/CHANGELOG.md)
- Security contact: katex-security@mit.edu — [GHSA-cg87-wmx4-v546](https://github.com/advisories/GHSA-cg87-wmx4-v546)

**npm provenance and licence**
- `katex@0.19.0` has **no** `dist.attestations` (no npm provenance / Sigstore attestation); licence is MIT — [npm registry: katex](https://registry.npmjs.org/katex)

### Inferences
- **Worker pattern.** KaTeX fits the existing Mermaid/pele pattern: `importScripts('katex.min.js')` in a same-origin worker, then `renderToString(src, {output:'mathml', trust:false, maxSize:<n>, maxExpand:1000, throwOnError:false})`, then post the string back, re-parse it, and put it through a Rust allow-list. The CSP needs only `script-src 'self'` (or `worker-src 'self'`).
- **HTML mode vs `style-src`.** Choosing HTML mode means the main thread must insert markup carrying dozens of `style="…"` attributes per formula. Under a `style-src` (or `style-src-attr`) without `'unsafe-inline'`, style attributes parsed from markup are blocked, while styles set through the CSSOM (`el.style.x = …`) are not. Note that `katex.render()` on the main thread builds nodes rather than parsing strings. A worker-to-string pipeline would therefore need the sanitiser to re-apply allow-listed style properties through the CSSOM, or to choose `output:'mathml'`. (The consumer's `style-src` was not given; check it before relying on this.)
- **MathML-only mode** removes the CSS file, all 20 font files and every inline style. It produces the smallest allow-list surface: MathML Core elements plus `class`, `mathcolor`, `mathbackground`, `mpadded` attributes, `columnalign` and the like.
- **Version pinning.** All six advisories are irrelevant to a pinned 0.18.2+ / 0.19.0 with `trust:false` and no prototype pollution. Two caveats remain: the 0.18.0 class-prefix change shows that a class allow-list must be re-derived on every upgrade, and the cadence of roughly 3–4 releases a month in 2026 means the SHA-256 pin will be bumped deliberately rather than tracked.
- **Provenance.** Because npm provenance is absent, the existing SHA-256 pin in a unit test is the only integrity anchor. Pin the exact tarball bytes, or the jsDelivr file hashes, at vendor time.

### Gaps
- I did not visually compare KaTeX `output:'mathml'` rendering in Chrome, Firefox and Safari in this session, so "how good it looks" rests on the Temml comparison material and the native MathML caveats in the Temml section below.
- I did not confirm which exact CSSOM call KaTeX's `toNode()` uses for inline styles. A regex for `node.style[x] =` found none, so it may be `Object.assign`/`setProperty`; either way it is CSSOM, not markup.
- CVE-2026-103923 is reported exactly as the GitHub API returned it; I did not cross-check it on cve.org.

## Temml: version, size, fidelity, maintenance, licence, fonts, CSP, worker use

### Takeaway
Temml is a **0.14.0 (2026-10-08)**, MIT-licensed, dependency-free LaTeX→MathML-Core converter by Ron Kok, derived from KaTeX. A minimum install is **about 182 KiB**: `temml.min.js` 167,881 B, one CSS file of about 9 KB, and `Temml.woff2` at 9,404 B. It has no eval and runs DOM-free like KaTeX. Its output is almost entirely MathML, with only a handful of inline `style` attributes. It has **zero** published security advisories and steady maintenance (about 17 releases in 2025 and 5 in 2026 so far), but no npm provenance.

### Cited Findings
- Version and package:
  - `temml@0.14.0`, published 2026-10-08, MIT licence, **no dependencies**, 2,165,141 B unpacked in 119 files, **no** npm attestations — [npm registry: temml](https://registry.npmjs.org/temml)
  - Releases: 0.11.0 (2025-02-14) → 0.11.11 (2025-09-13) → 0.12.1 (2025-11-03) → 0.13.1 (2025-12-22) → 0.13.2–0.13.5 (2026-03 to 2026-08) → 0.14.0 — [npm registry: temml](https://registry.npmjs.org/temml)
- Temml 0.14.0's breaking change is "Fix order of `\prescript` arguments" — [Temml CHANGELOG](https://raw.githubusercontent.com/ronkok/Temml/main/CHANGELOG.md)
- Files in `temml@0.14.0` ([jsDelivr listing temml@0.14.0](https://data.jsdelivr.com/v1/packages/npm/temml@0.14.0?structure=flat)):
  - JS: `dist/temml.min.js` **167,881 B**, `dist/temml.js` 424,122 B, `dist/temml.mjs` 478,952 B, `dist/temml.cjs` 478,962 B.
  - Font: `dist/Temml.woff2` **9,404 B**.
  - Six alternative CSS files of about 8.8–9.3 KB each: `Temml-Local.css` (8,992), `Temml-Latin-Modern.css` (9,288), `Temml-STIX2.css` (8,794), `Temml-Asana.css`, `Temml-Libertinus.css` and `Temml-NotoSans.css`.
  - `dist/temmlPostProcess.js` (2,973 B).
  - Contrib: `mhchem.min.js` (34,762 B), `physics.js`, `texvc.js` and `copy-tex.min.js`.
- The minimum browser installation is "temml.min.js, Temml-Local.css, Temml.woff2", and "the css file and font file must be in the same folder" — [Temml Administration](https://temml.org/docs/en/administration)
- Fonts:
  - "**Latin Modern** will provide the best quality rendering … For best results, you must also serve a small (10kb) Temml.woff2 file."
  - "Temml-Local.css is the light-weight option. It calls three fonts: Cambria Math (Windows), STIX TWO (iOS/macOS as of Safari 16), or NotoSans Math (Android?) … Sadly, this option has rendering issues. Chrome and WebKit both botch extensible arrows."
  - Source: [Temml Administration](https://temml.org/docs/en/administration)
  - The README says Chromium "works better with the Latin Modern font, a 380 KB file" — [Temml README](https://raw.githubusercontent.com/ronkok/Temml/main/README.md)
  - The Latin Modern font file is **not** in the npm package; only `Temml.woff2` is — [jsDelivr listing](https://data.jsdelivr.com/v1/packages/npm/temml@0.14.0?structure=flat)
- The README's own size table (minified JS + CSS) is Temml 173 KB, KaTeX 280 KB, MathJax 2.7.5 338 KB and TeXZilla 168 KB. It also states "Temml's coverage of LaTeX functions is as good as MathJax, slightly better than KaTeX 0.16.0" — [Temml README](https://raw.githubusercontent.com/ronkok/Temml/main/README.md)
- Temml's comparison page compares MathML output from Temml 0.13.02, MathJax 4.0, KaTeX 0.16.27 and TeXZilla 1.0.2.0, "now readable in Chrome, Edge, Firefox, and Safari" — [Temml comparison](https://temml.org/docs/en/comparison)
- Browser support: "Temml works in browsers that support MathML. This includes Chrome, Edge, Firefox, and Safari. Temml will never work in Internet Explorer." — [Temml Administration](https://temml.org/docs/en/administration)
- Security:
  - Temml's security text mirrors KaTeX's ("should be safe from `<script>`…"), with `maxSize`, `maxExpand` and `trust` (gating `\includegraphics` and `\class`) — [Temml Administration](https://temml.org/docs/en/administration)
  - Changelog items include converting the URL protocol to lower case "to enable a URL whitelist", and "`maxSize` now takes an array of two numbers" — [Temml CHANGELOG](https://raw.githubusercontent.com/ronkok/Temml/main/CHANGELOG.md)
  - There are **0** repo advisories and **0** npm advisories — [GitHub API: Temml advisories](https://api.github.com/repos/ronkok/Temml/security-advisories); [GitHub Advisory DB (npm temml)](https://api.github.com/advisories?ecosystem=npm&affects=temml)
- Options worth noting:
  - `annotate` (default false) adds `<annotation>` with the TeX source, which `copy-tex` requires, and "Auto-linebreaks will not work if annotation is true".
  - `wrap` (`none`/`tex`/`=`) gives soft line breaks in Chromium and Firefox but not WebKit.
  - Source: [Temml Administration](https://temml.org/docs/en/administration)
- Measured on temml@0.14.0:
  - **0** `eval(`, **0** `new Function(`, **0** `innerHTML`.
  - It loads and runs `renderToString` in a context with no DOM. API keys: `version`, `render`, `renderToString`, `renderMathInElement`, `postProcess`, `ParseError`, `definePreamble`.
  - Output is compact: `x^2+\frac{a}{b}` comes out as `<math><mrow><msup><mi>x</mi><mn class="tml-sml-pad">2</mn></msup><mo>+</mo><mfrac>…</mfrac></mrow></math>`.
  - Source: [temml.min.js 0.14.0](https://cdn.jsdelivr.net/npm/temml@0.14.0/dist/temml.min.js)
- Inline styles that Temml still emits (measured):
  - `style="display:block math;"` on display-mode `<math>`.
  - `style="color:#ff0000;"` for `\color`, and `padding:0.3em;color:…` for `\colorbox`.
  - `padding-left/right:…pt` on matrix cells.
  - Error text styled `color:#b22222;white-space:pre-line;`.
  - Also measured: `\href{javascript:…}` and `\htmlStyle` with the default trust rendered as error-coloured text, not links.
  - Source: [temml.min.js 0.14.0](https://cdn.jsdelivr.net/npm/temml@0.14.0/dist/temml.min.js)
  - Its DOM path sets `.style[...]` through the CSSOM (`math-depth`, `font-style`, `font-family`, `font-weight`) — [temml.js 0.14.0](https://cdn.jsdelivr.net/npm/temml@0.14.0/dist/temml.js)

### Inferences
- **Payload.** Temml is the smallest-payload option, about 182 KiB against KaTeX's 545 KiB for HTML mode, or about 266 KiB for KaTeX in MathML-only mode (which needs no CSS or fonts).
- **CSS file.** Temml's CSS file is still needed, for `.tml-*` classes, `math-depth` and font selection, and it references `Temml.woff2` relatively.
- **Maintenance risk.** Temml is effectively a single-maintainer project (Ron Kok), so the bus factor is higher than KaTeX's. On the other hand it has no dependencies and no advisory history.
- **Fidelity** depends on the browser's MathML Core layout engine, which Temml's own docs admit is uneven: Chromium has problems with extensible arrows and system fonts. A worker plus allow-list design suits it, because the output is a small MathML tree. The few `style` attributes it emits (`display:block math`, colours, cell padding) could be mapped to MathML attributes or classes by the sanitiser, or dropped.

### Gaps
- I did not find an independent (non-Temml) fidelity benchmark of Temml against KaTeX MathML output in 2025–26.
- I did not measure the Latin Modern woff2 size; the README says about 380 KB.

## MathJax 4: size, self-hosting under `script-src 'self'` without unsafe-eval, output modes, fonts, accessibility

### Takeaway
MathJax 4 shipped **4.0.0 on 2025-08-04**, and the latest is **4.1.3 (2026-07-03)**. It is Apache-2.0 and **has npm SLSA provenance**. It is much heavier:
- `tex-chtml.js` is about 974 KiB, plus the default `mathjax-newcm` font package of 105 woff2 files (1.55 MiB in total, loaded on demand).
- `tex-svg.js` is 1.76 MiB.

Its only `new Function` is webpack's dead-path `globalThis` shim, so `'unsafe-eval'` is not needed in modern browsers. However, v4 has accessibility (semantic enrichment, speech and Braille, an explorer) enabled by default. That stack spawns a speech worker from a **`blob:` URL** that `importScripts` the SRE worker, and CHTML injects a `<style>` element. A strict CSP must therefore either allow `worker-src blob:` and a style mechanism, or disable those features.

### Cited Findings
- Versions, licence and provenance:
  - `mathjax@4.1.3` (2026-07-03), Apache-2.0, 19,971,291 B unpacked, 106 files. It depends on `@mathjax/mathjax-newcm-font ^4.1.3` and carries an **npm provenance attestation (SLSA v1)** — [npm registry: mathjax](https://registry.npmjs.org/mathjax)
  - Releases: 4.0.0 (2025-08-04), 4.1.0 (2025-12-16), 4.1.1 (2026-02-19), 4.1.2 (2026-05-03), 4.1.3 (2026-07-03) — [npm registry: mathjax](https://registry.npmjs.org/mathjax)
  - `@mathjax/src@4.1.3` also has SLSA provenance and depends on `speech-rule-engine 5.0.0-rc.4`, `mhchemparser` and `mj-context-menu` — [npm registry: @mathjax/src](https://registry.npmjs.org/@mathjax/src)
  - `@mathjax/mathjax-newcm-font@4.1.3` is 49,247,851 B unpacked in 1,124 files and has **no** attestations — [npm registry: font](https://registry.npmjs.org/@mathjax/mathjax-newcm-font)
- Combined component sizes in 4.1.3 ([jsDelivr listing mathjax@4.1.3](https://data.jsdelivr.com/v1/packages/npm/mathjax@4.1.3?structure=flat)):

| Component | Size |
|---|---|
| `tex-chtml.js` | 997,445 B |
| `tex-chtml-nofont.js` | 888,469 B |
| `tex-svg.js` | 1,849,625 B |
| `tex-svg-nofont.js` | 873,900 B |
| `tex-mml-chtml.js` | 997,558 B |
| `mml-chtml.js` | 850,354 B |
| `startup.js` | 17,116 B |
| `core.js` | 141,170 B |

- Font payload of `mathjax-newcm` 4.1.3 ([jsDelivr listing @mathjax/mathjax-newcm-font@4.1.3](https://data.jsdelivr.com/v1/packages/npm/@mathjax/mathjax-newcm-font@4.1.3?structure=flat)):
  - `chtml/woff2/`: **105 woff2 files, 1,623,040 B in total**.
  - `chtml/dynamic/`: 550,677 B.
  - `svg/dynamic/`: 9,968,318 B.
  - `chtml.js`: 110,114 B; `svg.js`: 976,702 B.
- In v4, fonts are separate packages: "Installing MathJax via npm … will get you the default `mathjax-newcm` font". A self-hoster configures `output: { font: 'mathjax-stix2', fontPath: '/mathjax-stix2-font' }` and copies the font package to the server — [MathJax: Hosting your own copy](https://docs.mathjax.org/en/latest/web/hosting.html)
- What's new in v4 includes extended font support, line-breaking, the CHTML/SVG output improvements, a promise-based API, ES6 modules and "Changes to Speech Generation" — [What's New in MathJax v4.0](https://docs.mathjax.org/en/latest/upgrading/whats-new-4.0.html)
- Speech generation: "the speech is now generated within a web worker or node worker thread, the speech-generation code is no longer in the main MathJax components, but is in a separate file that is run in the worker" — [MathJax v4 Breaking Changes](https://docs.mathjax.org/en/latest/upgrading/whats-new-4.0/breaking.html)
- Accessibility is on by default (from [MathJax Accessibility Options](https://docs.mathjax.org/en/latest/options/accessibility.html)):
  - Defaults: `enableSpeech: true`, `enableBraille: true`, `enableExplorer: true`, `enableExplorerHelp: true`, `a11y: { speech: true, braille: true, subtitles: true … }`.
  - Worker config: `worker: { path: <bundle>/a11y/sre, maps: <sre>/lib/mathmaps, worker: 'speech-worker.js' }`.
  - "In version 3, the assistive-mml extension was … active by default. That is no longer the case in v4, where the other accessibility tools are included and enabled by default."
- Measured in `tex-chtml.js` 4.1.3 ([tex-chtml.js](https://cdn.jsdelivr.net/npm/mathjax@4.1.3/tex-chtml.js); same pattern in [tex-svg-nofont.js](https://cdn.jsdelivr.net/npm/mathjax@4.1.3/tex-svg-nofont.js)):
  - **0** `eval(` and exactly **1** `new Function`. That one is webpack's `r.g = function(){ if(typeof globalThis=="object") return globalThis; try{ return this || new Function("return this")() } … }`, which modern browsers never reach.
  - `createWorker()` builds the speech worker as `URL.createObjectURL(new Blob(["self.maps='…'; importScripts('<path>/speech-worker.js')"]))` and then calls `new Worker(blobURL)`.
  - CHTML output creates one `<style>` element and calls `insertRule` 5 times; there are no `nonce` references.
- Historical CSP/eval baggage belongs to v2/v3:
  - MathJax 2.7.4 users reported needing `'unsafe-eval'` and `'unsafe-inline'` — [mathjax/MathJax#1988](https://github.com/mathjax/MathJax/issues/1988)
  - Two npm advisories affect `mathjax` 2.x only: CVE-2018-1999024 (macro running untrusted JS, < 2.7.4) and CVE-2023-39663 (ReDoS, ≤ 2.7.9). There are **0** repo advisories for MathJax-src — [GitHub Advisory DB (npm mathjax)](https://api.github.com/advisories?ecosystem=npm&affects=mathjax)
- GitHub renders Markdown math "using the highly popular MathJax library" — [GitHub Blog: Math support in Markdown](https://github.blog/news-insights/product-news/math-support-in-markdown/)

### Inferences
- **CSP for MathJax 4 with defaults.** Under `script-src 'self'` with no `'unsafe-eval'`, MathJax 4 can load from same-origin files. Its default accessibility path still needs `worker-src blob:` (otherwise `worker-src` falls back to `script-src`, and `blob:` is not `'self'`). The worker then `importScripts` same-origin SRE files, which `'self'` allows. CHTML's dynamically created `<style>` will hit `style-src`, though `insertRule` through the CSSOM is generally not blocked. Disabling `enableSpeech`/`enableBraille`/`enableExplorer`, or using `mml-*`/SVG output without the a11y extensions, would avoid the blob worker. That would be a configuration exercise, not verified here.
- **Payload.** MathJax is about 4–7× the payload of KaTeX/Temml and is designed to scan and typeset the page DOM itself. That fits poorly with "render a string in our own worker, then sanitise in Rust". Its main advantages are the strongest built-in accessibility (speech, Braille, explorer), line-breaking, and supply-chain provenance.

### Gaps
- I found no official MathJax 4 page that explicitly documents CSP requirements; my search found only v2/v3-era reports. The CSP behaviour above is derived from the shipped code, not from documentation.
- I did not test whether MathJax 4's `liteAdaptor`/`mathjax` core can run `tex2mml` inside a worker in the browser. Its Node docs show `liteAdaptor` usage, which suggests it is possible.

## Practical vendoring recipe and how other projects ship math offline

### Takeaway
For KaTeX, ship `katex.min.js`, plus `katex.min.css` and `fonts/*.woff2` if HTML output is used. The CSS's relative `url(fonts/…)` works unchanged when the two are siblings, and Docusaurus explicitly documents this ("only .woff2 font types should be enough"). For Temml, ship `temml.min.js` plus one `Temml-*.css` plus `Temml.woff2` in the same folder. mdBook and GitHub both use MathJax: mdBook still hard-codes a cdnjs MathJax **2.7.1** URL, so it is not CSP-friendly or offline.

### Cited Findings
- Docusaurus enables math with `remark-math@6` and `rehype-katex@7`; KaTeX runs at build time. "In case you prefer to self-host the katex.min.css (along with required KaTeX fonts) … copy katex.min.css and fonts directory (only `.woff2` font types should be enough) to your site's static directory … replace the stylesheet's href … to your local path (say, `/katex/katex.min.css`)" — [Docusaurus: Math Equations](https://docusaurus.io/docs/markdown-features/math-equations)
- mdBook's `mathjax-support = true` injects `<script async src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.1/MathJax.js?config=TeX-AMS-MML_HTMLorMML">` into its template — [mdBook index.hbs](https://raw.githubusercontent.com/rust-lang/mdBook/master/crates/mdbook-html/front-end/templates/index.hbs); [mdBook: MathJax support](https://rust-lang.github.io/mdBook/format/mathjax.html)
- GitHub uses MathJax — [GitHub Blog](https://github.blog/news-insights/product-news/math-support-in-markdown/)
- KaTeX's CSS uses relative font URLs, `url(fonts/KaTeX_*.woff2)` with woff and ttf fallbacks in the same `src` — measured from [katex.min.css](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.css). KaTeX 0.16.25 added a `katex-swap.css` variant with `font-display: swap` — [KaTeX CHANGELOG](https://raw.githubusercontent.com/KaTeX/KaTeX/main/CHANGELOG.md)
- Temml: "The css file and font file must be in the same folder" — [Temml Administration](https://temml.org/docs/en/administration)
- MathJax self-hosting: copy `node_modules/mathjax` and the font package to the server, then set `output.fontPath` — [MathJax: Hosting](https://docs.mathjax.org/en/latest/web/hosting.html)
- Integrity anchors available: KaTeX and Temml publish **no** npm provenance, while MathJax 4.1.3 does (SLSA v1) — [npm: katex](https://registry.npmjs.org/katex), [npm: temml](https://registry.npmjs.org/temml), [npm: mathjax](https://registry.npmjs.org/mathjax)

### Inferences
- **Concrete recipe for the leptos-daisyui-rs worker pattern** (mirroring `assets/pele/`):
  - Vendor `katex.min.js` 0.19.0 (272,868 B) or `temml.min.js` 0.14.0 (167,881 B) into `assets/katex/` or `assets/temml/`.
  - Pin its SHA-256 in a unit test.
  - In the worker, `importScripts('/katex/katex.min.js')`, call `renderToString(src, {output:'mathml', trust:false, maxSize:…, maxExpand:1000, throwOnError:false, strict:'ignore'})`, and post the result back.
  - In Rust, re-parse and allow-list MathML Core.
  - With MathML output you can drop the CSS and fonts entirely. With KaTeX HTML output, also vendor `katex.min.css` and the 20 woff2 files (259,792 B), keep `fonts/` next to the CSS, delete the woff and ttf `src` entries if desired, and make sure `font-src 'self'` and `style-src 'self'` are in the CSP.
  - A `<script>`-level SRI attribute does not apply to `importScripts`; the hash pin in tests plus same-origin serving is the equivalent control.
- **Version pinning.** Vendoring a copy also decouples you from KaTeX's 2026 churn (0.17, 0.18 and 0.19 in five months).

### Gaps
- Zola: its shortcodes docs URL returned 404, and I did not verify how Zola themes ship KaTeX.
- rustdoc's `--html-in-header` pattern (for example the `katex-header` crate used on docs.rs) and Obsidian's bundled MathJax were not verified in this session, so no claims are made about them.

## Sanitising the output: safe as-is with trust:false, or allow-list (DOMPurify MathML profile)? Known XSS in output

### Takeaway
Both projects say their output "should be safe" with `trust:false`, but KaTeX itself recommends sanitising, and **every KaTeX XSS-class advisory has been in the trust and URL paths**: protocol normalisation (CVE-2024-28246), `\includegraphics` filename escaping (CVE-2024-28245), `\htmlData` attribute names (CVE-2025-23207), and prototype pollution flipping `trust` on (CVE-2026-103923). The advisories themselves list "sanitize HTML output" as the mitigation. A post-render allow-list is therefore justified defence-in-depth, and MathML-only output makes it small.

### Cited Findings
- "Any HTML generated by KaTeX should be safe … it is always a good idea to sanitize the HTML, though you will need a rather generous whitelist (including some of SVG and MathML)" — [KaTeX Security](https://katex.org/docs/security)
- GHSA-cg87 workaround: "Sanitize HTML output from KaTeX" — [GHSA-cg87-wmx4-v546](https://github.com/advisories/GHSA-cg87-wmx4-v546)
- GHSA-238p: "Exploitation requires the consuming application to insert KaTeX output into a web page without a separate sanitizer. (KaTeX does not execute scripts merely by rendering an expression.)" — [GHSA-238p-pmpm-9mq7](https://github.com/advisories/GHSA-238p-pmpm-9mq7)
- KaTeX 0.18.0 notes that allow-lists "targeting KaTeX's internal classes must update their selectors" — [KaTeX CHANGELOG](https://raw.githubusercontent.com/KaTeX/KaTeX/main/CHANGELOG.md)
- Measured with `trust:false` on both KaTeX 0.19.0 and Temml 0.14.0:
  - `\href{javascript:…}`, `\htmlStyle`, `\htmlData` and `\includegraphics` produced no `href`, `<img>` or `data-*` attributes; they rendered as coloured error text.
  - With KaTeX `trust:true`, `href="javascript:alert(1)"` **was** emitted.
  - Sources: [katex.min.js](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.min.js), [temml.min.js](https://cdn.jsdelivr.net/npm/temml@0.14.0/dist/temml.min.js)
- KaTeX's HTML output includes SVG (`createElementNS` for `svg` and `path`, used for `\sqrt`, tall delimiters and similar) — measured from [katex.js 0.19.0](https://cdn.jsdelivr.net/npm/katex@0.19.0/dist/katex.js)
- `dompurify` is at 3.4.17 (2026-10-12), licensed `(MPL-2.0 OR Apache-2.0)`, with 17 releases between April and October 2026 — [npm registry: dompurify](https://registry.npmjs.org/dompurify)

### Inferences
- **Recommended stance.** Keep `trust:false`, or a `trust` function that allows nothing. Set `maxSize` to a finite value (the default is Infinity) and keep `maxExpand` at 1000. Run the render in the worker, which isolates hangs. Then allow-list the result in Rust, as is already done for pele's SVG.
- **Allow-list size.** For `output:'mathml'` (KaTeX) or Temml, the allow-list is MathML Core elements plus a short attribute list: `class`, `display`, `mathvariant`, `mathcolor`, `mathbackground`, `width`, `height`, `lspace`, `voffset`, `rowspacing`, `columnalign`, `columnspacing`, `fence`, `stretchy`, `scriptlevel`, `displaystyle`, `encoding` on `<annotation>`, and optionally a constrained `style`. Drop `href` entirely.
- **HTML-mode cost.** KaTeX HTML mode would require allowing `span`, SVG `svg`/`path`, many classes (version-sensitive after 0.18.0) and many `style` properties. That is a much larger and more brittle surface.
- **DOMPurify** has a MathML profile (`USE_PROFILES: {mathMl: true}`), but adding a JS sanitiser is unnecessary given the existing Rust sanitiser. Its frequent 2026 release cadence also reflects how hard mXSS-safe sanitisation is in JS.

### Gaps
- I did not find any published XSS in KaTeX or Temml output under `trust:false` beyond the advisories listed. I did not audit Temml's `\href`/`\url` handling with `trust` enabled.
- I did not fetch DOMPurify's MathML-profile docs this session; the profile name is from prior knowledge and should be verified.
