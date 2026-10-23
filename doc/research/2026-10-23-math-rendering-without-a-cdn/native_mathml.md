# Native browser MathML (MathML Core) as the rendering target for LaTeX math (as of 2026-10-23)

Research date: 2026-10-23. Browser versions current on wpt.fyi that day: Chrome 156.0.8078.47, Firefox 158.0.1, Safari 27.0. Browser-compat data is from `@mdn/browser-compat-data` 8.1.7 (timestamp 2026-10-22). Anything older is labelled with its date.

## 1. Browser support status in 2026: what works, what is missing, Interop, WPT

### Takeaway
All three engines render MathML Core's element set in October 2026. Chromium tracks MathML Core most closely: about 97.7% of WPT `/mathml/` subtests pass. Firefox and Safari are at about 83% and 86%. MathML has never been an Interop focus area: it was not selected in 2024, 2025 or 2026. The remaining gaps are in the CSS-level math features. Safari only got `math-depth` and `font-size: math` in 26.4. Neither Firefox nor Safari supports `display: math`. Chrome deliberately implements none of the non-Core MathML 3 extras (`menclose`, `mfenced`, the `mtable` attributes, `mlabeledtr`).

### Cited Findings
**Element support (BCD 8.1.7, 2026-10-22):**
- Chrome/Edge/Chrome Android support every MathML Core element from version 109: `math`, `mi`, `mn`, `mo`, `ms`, `mtext`, `mspace`, `mrow`, `mfrac`, `msqrt`, `mroot`, `mstyle`, `merror`, `mpadded`, `mphantom`, `msub`, `msup`, `msubsup`, `munder`, `mover`, `munderover`, `mmultiscripts`, `mprescripts`, `mtable`, `mtr`, `mtd`, `maction`, `semantics`, `annotation` and `annotation-xml`. Firefox has supported them since version 1 to 6. Safari has supported them since 5 to 9. — [MDN BCD data.json (unpkg @mdn/browser-compat-data 8.1.7)](https://unpkg.com/@mdn/browser-compat-data/data.json); element pages under [MDN MathML element reference](https://developer.mozilla.org/en-US/docs/Web/MathML/Reference/Element)
- `menclose` (not in MathML Core): Chrome **not supported**, Firefox 1, Safari 7. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `mfenced` (deprecated, not in Core): Chrome **not supported**, Firefox 1, Safari 6. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `mlabeledtr` does not appear in BCD at all. It is not part of MathML Core, so no engine is tracked for it. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `maction` in Chrome follows MathML Core: only the first child is rendered and the others get `display: none`. The `actiontype` and `selection` attributes are not acted on. — [BCD note on maction](https://unpkg.com/@mdn/browser-compat-data/data.json); same rule in the [MathML Core UA stylesheet](https://w3c.github.io/mathml-core/#user-agent-stylesheet) (`maction > :not(:first-child) { display: none; }`)
- The MathML `<a>` element (new in MathML Core) shipped in **Firefox 152** only. Chrome and Safari do not support it. Firefox **removed** the legacy `href` global attribute on arbitrary MathML elements in 152 (it was supported from 7 to 151). **Safari still supports `href` on any MathML element (since 10).** Chrome never supported it. — [BCD mathml.elements.a / mathml.global_attributes.href](https://unpkg.com/@mdn/browser-compat-data/data.json); [MathML Core: the a element](https://w3c.github.io/mathml-core/#the-a-element)

**Attribute gaps relevant to LaTeX output:**
- `mtable` attributes `align`, `columnalign`, `columnlines`, `columnspacing`, `frame`, `framespacing`, `rowalign`, `rowlines`, `rowspacing`, `width`: **Chrome supports none of them**. They are not Core. Firefox supports all of them. Safari supports `columnalign`, `columnlines`, `frame`, `rowalign` and `rowlines` (from 5.1), but not the spacing attributes. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- Chrome 109 supports the `mpadded` Core attributes (`width`, `height`, `depth`, `lspace`, `voffset`). Chrome does not support pseudo-units, relative values, named spaces or scale factors, and Safari lacks most of them too. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- The `mo` Core attributes (`form`, `largeop`, `lspace`, `rspace`, `maxsize`, `minsize`, `movablelimits`, `stretchy`, `symmetric`) work in Chrome 109, Firefox 1 and Safari 5 to 10. `mo@accent` is not Core and Chrome does not support it. In Core, accents go on `mover`/`munder`. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `displaystyle` works in Chrome 109, Firefox 83 and Safari 14. Safari implements it through `math-style` from 14.1. `scriptlevel` works in Chrome 109 and Firefox 70, and is **not supported in Safari**. Firefox does not implement it through `math-depth`. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)

**CSS math features:**
- `math-style`: Chrome 109, Firefox 117, Safari 14.1, iOS 14.5. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `math-depth`: Chrome 109, Firefox 117, **Safari 26.4** (2026). — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `math-shift`: Chrome 109, **Firefox 146**, **Safari 26.2**. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `font-size: math`: Chrome 109, Firefox 117, Safari 26.4. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `display: math` / `block math`: Chrome 109. **Not supported in Firefox or Safari.** — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- `text-transform: math-auto`: Chrome 109, Firefox 117. **Not supported in Safari.** — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)

**Rendering problems with real LaTeX output (Temml author's table; Temml 0.13/0.14 era, 2025-2026):**
- `\left( x \right)`: poor in WebKit, which misaligns short parentheses. Radical degree: poor in Chromium and WebKit, which place it too far left with Latin Modern (Cambria Math looks better). Extensible arrows: poor in Chromium and WebKit, with notes placed too high; some arrows do not stretch in Cambria Math or Noto Sans Math, while Latin Modern is okay. Radical height: "meh" in Firefox and WebKit, which sometimes pick radicals that are too tall. Size-4 radicals: "meh" in Chromium, where the vinculum does not match the surd in very tall radicals. Line-breaking: **automatic linebreaking of inline math works in Chromium and Firefox, not in WebKit**. Flattened circumflex accent: poor in all three. — [Temml docs: Browser Issues](https://temml.org/docs/en/administration)

**RTL math (2025):**
- July 2025: Igalia merged MathML Core spec changes for RTL operator mirroring, and the Chromium implementation landed behind an experimental flag in Canary. WebKit's operator mirroring is "quite limited", and Firefox's implementation has issues. — [Frédéric Wang, www-math, 2025-07-22](https://lists.w3.org/Archives/Public/www-math/2025Jul/0010.html)

**Interop:**
- The Interop 2026 focus areas are: container style queries, anchor positioning, `attr()`, `contrast-color()`, CSS zoom, custom highlights, dialogs/popovers, fetch uploads/ranges, IndexedDB, JSPI for Wasm, media pseudo-classes, Navigation API, scoped custom element registries, scroll-driven animations, scroll snap, `shape()`, view transitions, web compat, WebRTC and WebTransport. The investigation efforts are accessibility testing, JPEG XL, mobile testing and WebVTT. **There is no MathML.** — [web-platform-tests/interop 2026 README](https://github.com/web-platform-tests/interop/blob/main/2026/README.md)
- No Interop 2025 focus area or investigation mentions MathML. — [interop 2025 README](https://github.com/web-platform-tests/interop/blob/main/2025/README.md)
- (Dec 2024) Igalia's 2024 MathML work was "largely inspired by addressing feedback ... and public submissions to Interop 2024 which were not chosen". By the end of 2024, MathML passed many tests for CSS `width`/`height`/`margin`/`border` in all engines. 251 new MathML subtests were added to WPT in 2024, and two MathML issues were submitted to Interop 2025. — [Igalia, "MathML 2024", 2024-12-17](https://igalia.com/2024/12/17/MathML-2024.html)
- (2023) www-math discussed proposing MathML for Interop. An earlier proposal on CSS-with-MathML had not been accepted. — [www-math 2023-09](https://lists.w3.org/Archives/Public/www-math/2023Sep/0020.html)

**WPT pass rates (computed 2026-10-23 from the wpt.fyi search API, query `/mathml/`, latest stable master runs):**
- 623 test files. Subtests passed: **Chrome 156: 4088/4186 (97.7%)**, **Firefox 158: 3447/4148 (83.1%)**, **Safari 27.0: 3547/4145 (85.6%)**. — [wpt.fyi /mathml](https://wpt.fyi/results/mathml?label=master&label=stable) (computed via `https://wpt.fyi/api/search?run_ids=5348835735109632,5199781143248896,6066188923961344&q=/mathml/`)

### Inferences
- For KaTeX/Temml/Rust-generated **Core** markup (`mrow`/`mfrac`/`msqrt`/`msub`/`mtable`/`mo stretchy`), all three engines produce usable output in 2026. Quality differences come from glyph assembly and spacing, not from missing elements.
- **Matrices and aligned environments are the weak point.** KaTeX's MathML-only output leans on `mtable@columnalign`/`rowspacing`/`columnspacing`, which Chrome ignores. Temml and math-core instead emit CSS classes or inline styles (plus a "mathfixes" CSS) for alignment. A Rust converter has to do the same: style `mtd` with CSS (`text-align`, `padding`) rather than MathML 3 attributes.
- `\cancel`/`\boxed`/`\enclose` produce `menclose` in some converters, and that renders unstyled in Chromium. A converter has to lower these to `mrow` plus CSS borders or backgrounds. Temml does this, but this session did not confirm its exact output.
- `\tag`/equation numbering via `mlabeledtr` will not work. Use an extra `mtd` column or a CSS-positioned label instead.
- `scriptlevel` is ignored in Safari, so converters should use `displaystyle`/`math-style` and rely on the UA's automatic `math-depth` (Safari 26.4+). On Safari before 26.4, script-size reduction depended on WebKit's legacy internal logic, not `math-depth`.

### Gaps
- I found no Igalia 2025 year-end MathML report. The July 2025 www-math thread is the newest Igalia status found, so I could not confirm 2026 Igalia/WebKit changes beyond BCD version numbers.
- I did not confirm whether Chromium's RTL operator mirroring shipped unflagged by October 2026.
- The WPT numbers cover the whole `/mathml/` directory, which includes some non-Core and legacy tests. They are not a Core-only score, and wpt.fyi has no official "Interop MathML" score.
- Stretchy-operator quality and `mpadded` correctness per engine beyond the Temml table were not measured. No 2026 visual-regression comparison was found.

## 2. Fonts: is an OpenType MATH font needed, OS fallbacks, sizes and licences, CSS

### Takeaway
Yes. Good native MathML rendering depends on an OpenType font with a `MATH` table. The spec itself says the fallbacks are not guaranteed to look good. With no font specified, the UA stylesheet uses `font-family: math`, which resolves to Cambria Math on Windows, STIX Two Math on macOS 13+ and iOS (Safari 16+), and probably Noto Sans Math on Android. Self-hosting a complete math font costs about 95 KB (Fira Math) to 690 KB (NewCM Math) as woff2. Latin Modern Math (388 KB, GUST licence) gives the most LaTeX-like look.

### Cited Findings
- MathML Core: "It is strongly encouraged to render MathML with a math font with the proper OpenType features. There is no guarantee that the fallback options provided will provide good enough rendering." Every layout constant has a fallback, for example `scriptPercentScaleDown` falls back to 0.71 and `axisHeight` to half of `OS/2.sxHeight`. — [MathML Core §5 OpenType MATH table](https://w3c.github.io/mathml-core/#opentype-math-table)
- MathML Core UA stylesheet: `math { font-family: math; font-size: inherit; font-style: normal; font-weight: normal; direction: ltr; line-height: normal; ... display: inline math; math-style: compact; math-depth: 0 }`, plus `math[display="block" i] { display: block math; math-style: normal; }`. "Because good mathematical rendering requires use of mathematical fonts, the user agent stylesheet should set the font-family to the math value on the `<math>` element instead of inheriting it." — [MathML Core §2.1.1 / Appendix A](https://w3c.github.io/mathml-core/#user-agent-stylesheet)
- MDN recommends installing both **Latin Modern Math** (Computer Modern style) and **STIX Two Math** (broad Unicode coverage). "Cambria Math is installed by default on Windows and should ensure relatively good MathML rendering." "If you are using macOS Ventura (version 13) or higher, then STIX Two Math is already pre-installed." A deprecated STIX 1 has been preinstalled since OS X Lion. On Linux, Debian/Ubuntu `fonts-stix` installed STIX 1.x, not STIX Two Math, up to Debian 13 and Ubuntu 24. — [MDN: Fonts for MathML](https://developer.mozilla.org/en-US/docs/Web/MathML/Guides/Fonts)
- MDN Authoring advises shipping math fonts as web fonts and points to Frédéric Wang's MathFonts stylesheets (Latin Modern, STIX, XITS for RTL). — [MDN: Authoring MathML](https://developer.mozilla.org/en-US/docs/Web/MathML/Guides/Authoring)
- Temml's guidance (Temml 0.14.0 era): **Latin Modern gives the best quality**, but needs an extra small `Temml.woff2` for `\mathscr` and for correct prime alignment in Chrome/Edge. **Temml-Local.css** uses system fonts only: Cambria Math (Windows), STIX Two (iOS and macOS as of Safari 16) and Noto Sans Math ("I think comes pre-installed in Android"). The first two need `Temml.woff2`. "Sadly, this option has rendering issues. Chrome and WebKit both botch extensible arrows." Asana and Libertinus share some of Cambria's problems. Temml's Noto Sans CSS loads the **ttf** because the woff2 has many rendering issues. — [Temml docs: Fonts](https://temml.org/docs/en/administration)
- Temml.woff2 is **9,404 bytes**. Temml-Latin-Modern.css is 9,288 bytes, Temml-Local.css 8,992 bytes, and temml.min.js 167,881 bytes (Temml 0.14.0 on unpkg). Temml-Latin-Modern.css declares `@font-face { font-family: Latin Modern Math; src: url('./latinmodernmath.woff2') }` and `math { font-family: "Latin Modern Math", math; }`. — [unpkg temml/dist](https://unpkg.com/temml/dist/Temml-Latin-Modern.css)
- **woff2 sizes, measured 2026-10-23 from fred-wang.github.io/MathFonts:** Latin Modern Math **388,212 B**; STIX Two Math Regular **558,644 B**; Fira Math Regular **95,524 B**; NewCM Math Book **686,232 B**; Asana Math 249,712 B; XITS Math 295,364 B; Garamond Math 413,900 B. Libertinus Math and Noto Sans Math returned 404 at the paths tried, so I have no measurement. — [MathFonts](https://fred-wang.github.io/MathFonts/)
- "The Latin Modern fonts are released under the GUST font license, which is legally equivalent to the LaTeX Project Public License." The MathFonts CSS uses `src: local('Latin Modern Math'), local('LatinModernMath-Regular'), url('latinmodern-math.woff2')`. — [MathFonts LatinModern/mathfonts.css](https://fred-wang.github.io/MathFonts/LatinModern/mathfonts.css)
- KaTeX 0.19.0 option `output`: `html` | `mathml` | `htmlAndMathml` (the default). — [KaTeX options](https://katex.org/docs/options) / [docs/options.md](https://raw.githubusercontent.com/KaTeX/KaTeX/main/docs/options.md)

### Inferences
- Minimal CSS for self-hosting under `script-src 'self'`, assuming fonts are also same-origin (`font-src 'self'`):
  `@font-face { font-family: "Latin Modern Math"; src: local("Latin Modern Math"), url("/fonts/latinmodern-math.woff2") format("woff2"); font-display: swap; }`
  `math { font-family: "Latin Modern Math", "STIX Two Math", "Cambria Math", math; }`
  The `local()` first entry avoids the download on machines that already have the font.
- Without a shipped math font, output is acceptable but uneven. Windows (Cambria Math) and Apple (STIX Two) will look reasonable for common formulas. Extensible arrows and radical degrees misrender in Chromium and WebKit, and Android/Linux are the least predictable. For a component library, shipping Latin Modern Math (about 380 KB, cacheable, loaded only when a `<math>` is present) is the safe default. Fira Math (about 93 KB) is the smallest full option and is sans-serif, so it may suit a daisyUI UI better than Computer Modern. That is a design judgement, not something measured here.
- Because `math` resets `font-family` instead of inheriting, the app's body font never reaches formulas unless the CSS sets one. `mtext` uses the math font too. Temml's CSS adds a text-font rule for that, as line 337 of Temml-Latin-Modern.css shows.

### Gaps
- Licences for STIX Two Math, Fira Math, NewCM Math and Libertinus Math were not verified in this session. From prior knowledge, STIX Two, Fira Math and Libertinus are SIL OFL 1.1 and NewCM is GUST, but the repos' LICENSE files need checking.
- I could not confirm which math font Android ships, or what Chrome on Android resolves `font-family: math` to. Temml's author hedges with "I think".
- I found no current published screenshots of KaTeX/Temml MathML-only output rendered with no math font installed.

## 3. Visual fidelity compared with KaTeX HTML+CSS output

### Takeaway
I found no 2026 side-by-side study of native MathML against KaTeX's HTML+CSS output. The best available evidence is Temml's own MathML comparison page and its browser-issues table. Both say native MathML is now readable in all four major browsers, with specific weak spots: radicals, extensible arrows, accents, short delimiters in WebKit, and Chrome ignoring `mtable` attributes. KaTeX's HTML mode avoids these because it positions its own bundled fonts with CSS.

### Cited Findings
- Temml's comparison page covers the MathML output of Temml 0.13.02, MathJax 4.0, KaTeX 0.16.27 and TeXZilla 1.0.2.0, "now readable in Chrome, Edge, Firefox, and Safari". — [Temml: comparison](https://temml.org/docs/en/comparison)
- The per-engine defects are listed in the Temml table in §1 (radicals, extensible arrows, `\left(`, line-breaking, circumflex). — [Temml docs: Browser Issues](https://temml.org/docs/en/administration)
- The math-core (Rust LaTeX→MathML Core) README says Temml "produces much higher quality MathML output than KaTeX" and "is much more willing to work around browser bugs ... with specific CSS hacks for each browser". It says math-core does so to a lesser degree: "Even if you don't include the CSS from `mathfixes.css`, math-core's output will look mostly good." It also notes pulldown-latex cannot strip redundant `mrow` grouping, and as of June 2025 did not distinguish `\mathcal` from `\mathscr`. — [tmke8/math-core README](https://github.com/tmke8/math-core)

### Inferences
- The fidelity gap is mostly **converter quality plus font choice**, not the browsers alone. KaTeX's `output: "mathml"` was designed as an accessibility shadow for its HTML rendering, not as the primary visual output. Temml and math-core target visual MathML and include browser-specific CSS fixes. For option (b), Temml is the better self-hosted JS choice. For option (a), the Rust converter needs its own "mathfixes" stylesheet.
- To match KaTeX's look closely, use Latin Modern Math. KaTeX's own fonts are Computer Modern derivatives, but they lack a MATH table, so they cannot be reused for native MathML.

### Gaps
- I found no screenshot-based or pixel-diff comparison of native MathML against KaTeX HTML, and no 2026 independent review. The visual-fidelity claims here come from tool authors, who have an interest in the outcome.
- Matrix and `aligned` spacing fidelity in Chrome was not measured directly.

## 4. Accessibility: screen readers, axe, alttext/annotations, copy-paste

### Takeaway
Screen-reader support for MathML on Windows improved a lot in 2026: **NVDA 2026.1 ships MathCAT built in**, so speech, navigation and Nemeth/UEB braille need no add-on, and Vispero (JAWS) also incorporates MathCAT. Native MathML is the best input for these tools, better than an image or an aria-label. I found no 2026 source for VoiceOver, TalkBack or Narrator quality, and no axe-core rule aimed at MathML.

### Cited Findings
- NVDA 2026.1: "This release includes support for reading math content with MathCAT, which is now built-in to NVDA." — [NVDA What's New](https://www.nvaccess.org/files/nvda/documentation/changes.html) (the latest listed release is 2026.2)
- The MathCAT status page (updated 3/27/26) says MathCAT converts MathML to speech, braille (Nemeth, UEB, CMU, Vietnamese, German/Austrian, LaTeX, ASCIIMath) and navigation. Speech covers English, German, Spanish, Finnish, Indonesian, Norwegian, Russian, Swedish, Vietnamese and Chinese (Traditional). "As of NVDA 2026.1, MathCAT is built into NVDA and no download is needed." "A number of other assistive technologies have incorporated MathCAT into their products. Notable among these is Vispero/J[AWS]..." (the text was truncated in my fetch). MathCAT can use MathML "intent" to disambiguate notation. — [MathCAT (DAISY)](https://daisy.github.io/MathCAT/)
- (Legacy, MathJax 2.6 to 3.1 docs, about 2016 to 2020) VoiceOver in Safari read hidden MathML but sometimes dropped parts. In Chrome on macOS it detected but misread MathML. MathPlayer was the engine for JAWS and NVDA at that time. — [MathJax 3.1 legacy accessibility](https://docs.mathjax.org/en/v3.1/basic/legacy-accessibility.html); [MathJax v2.7 accessibility](https://docs.mathjax.org/en/v2.7/misc/accessibility-features.html)
- A search of the axe-core 4.14 rule-descriptions list (`develop` branch) found no rule targeting `<math>` or MathML. The closest is `svg-img-alt`, which applies to `<svg>` with img roles. — [axe-core rule descriptions](https://raw.githubusercontent.com/dequelabs/axe-core/develop/doc/rule-descriptions.md)
- MathML Core semantics: `<semantics>` holds the presentation as its first child, followed by `<annotation encoding="application/x-tex">` or `annotation-xml`. The UA stylesheet hides every child except the first (`semantics > :not(:first-child) { display: none; }`). — [MathML Core §3.8](https://w3c.github.io/mathml-core/#semantics-and-presentation)

### Inferences
- In CI, axe will not flag broken math structure. If the math sits in containers with `aria-hidden` or odd roles, axe's generic rules apply, but nothing checks that the formula is speakable. A custom lane assertion is needed, for example that every `<math>` has element children and a TeX annotation, or that `display` is set on block math.
- Do **not** put `aria-label` on `<math>`. It would replace MathCAT's structured speech and navigation with a flat string. Keeping the TeX in `<annotation encoding="application/x-tex">` preserves the source for tools and copy-paste without affecting rendering. `alttext` is a MathML 3 attribute and is not in MathML Core or the Sanitizer default allow-list, so it adds little.
- Colour contrast checks in the repo's `common::axe` helpers should still work on MathML text, because it is ordinary text with computed `color`.

### Gaps
- I found no 2026 primary source on **JAWS** version details, **VoiceOver** (macOS/iOS 26), **TalkBack** or **Narrator** MathML support. I also found nothing on Chrome versus Safari MathML accessibility-tree exposure in 2026.
- I found no source on copy-paste behaviour of native MathML (whether a selection copies the visible glyphs, the annotation, or MathML markup) in 2026 browsers.
- I could not confirm whether axe has any `math`-role handling in its `aria-*` rules.

## 5. Theming: does MathML inherit `color` for dark mode?

### Takeaway
Yes. The MathML Core UA stylesheet resets fonts, direction and spacing on `<math>` but **not `color`**, so formulas inherit the daisyUI `--color-base-content` text colour and switch with the theme. Fraction bars, radicals and stretchy glyphs are painted in the text colour. The exceptions are `<merror>`, which hard-codes a light-yellow background and red border, and any `mathcolor`/`mathbackground` attributes, which act as presentational hints that override inheritance.

### Cited Findings
- The `math` UA rule lists `direction, text-indent, letter-spacing, line-height, word-spacing, font-family, font-size, font-style, font-weight, display, math-shift, math-style, math-depth`. **`color` is not reset.** — [MathML Core Appendix A](https://w3c.github.io/mathml-core/#user-agent-stylesheet)
- The UA stylesheet sets `merror { border: 1px solid red; background-color: lightYellow; }`. — [MathML Core Appendix A](https://w3c.github.io/mathml-core/#user-agent-stylesheet)
- `mathcolor`/`mathbackground` "are expected to treat these attributes as a presentational hint setting the element's color and background-color". Core recommends CSS instead. `mathbackground` is marked deprecated in BCD. — [MathML Core §2.1.5](https://w3c.github.io/mathml-core/#legacy-mathml-style-attributes); [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)
- The default `mtd` padding is `0.5ex 0.4em` with `text-align: center`. — [MathML Core Appendix A](https://w3c.github.io/mathml-core/#user-agent-stylesheet)

### Inferences
- Add `merror { background: transparent; border-color: var(--color-error); color: var(--color-error); }` or similar. Otherwise parse errors render as light-yellow boxes in dark themes, and the repo's contrast gate may fail them.
- Strip or ignore `mathcolor`/`mathbackground` (or LaTeX `\color{}`) so a converter cannot inject a hard-coded black that disappears in dark mode. Alternatively, map `\color` names to theme tokens.

### Gaps
- I did not verify whether every engine paints stretchy-operator glyph assemblies and `menclose` borders with `currentColor`. They are expected to, as text glyphs and CSS borders, but this was not tested here.

## 6. Security and sanitising MathML

### Takeaway
The MathML risks are (1) **parser namespace confusion and mutation XSS**: `mtext`/`mi`/`mo`/`mn`/`ms` are text integration points, `annotation-xml encoding="text/html"` is an HTML integration point, and `mglyph`/`malignmark` behave specially; (2) **link attributes**: Safari still honours `href` on *any* MathML element, and MathML `<a>` exists in Firefox 152+; (3) **`maction`**, whose actions are legacy and not Core. The WHATWG HTML Sanitizer's built-in safe default allows exactly the Core presentation elements plus `semantics`. It **drops `annotation`, `annotation-xml`, `maction`, `menclose`, `mglyph` and `malignmark`**, and blocks `javascript:` in MathML `href`. `Element.setHTML()` shipped in Chrome/Edge 146 and Firefox 148, and is not in Safari.

### Cited Findings
**Sanitizer API status (2026):**
- The Sanitizer API spec "has been moved to WHATWG HTML". The WICG repo now redirects issues there. — [WICG sanitizer-api index.bs](https://github.com/WICG/sanitizer-api/blob/main/index.bs); [WHATWG HTML: HTML sanitization](https://html.spec.whatwg.org/#html-sanitization)
- `Element.setHTML` and `Sanitizer` ship in **Chrome 146, Edge 146, Chrome Android 146 and Firefox 148**. Safari and iOS: **not supported**. `setHTMLUnsafe` ships in Chrome 124, Firefox 123 and Safari 26. — [BCD 8.1.7](https://unpkg.com/@mdn/browser-compat-data/data.json)
- Firefox 148 (released 2026-02-24) "supports the Sanitizer API as well as Trusted Types". Developers can pass a custom configuration. — [Mozilla Hacks, Feb 2026](https://hacks.mozilla.org/2026/02/goodbye-innerhtml-hello-sethtml-stronger-xss-protection-in-firefox-148/)
- **MathML elements in the built-in safe default configuration**, with allowed attributes:
  - `a` (`href`, `hreflang`, `type`)
  - `math`, `merror`, `mfrac`, `mi`, `mmultiscripts`, `mn`
  - `mo` (`fence`, `form`, `largeop`, `lspace`, `maxsize`, `minsize`, `movablelimits`, `rspace`, `separator`, `stretchy`, `symmetric`)
  - `mover` (`accent`)
  - `mpadded` (`depth`, `height`, `lspace`, `voffset`, `width`)
  - `mphantom`, `mprescripts`, `mroot`, `mrow`, `ms`
  - `mspace` (`depth`, `height`, `width`)
  - `msqrt`, `mstyle`, `msub`, `msubsup`, `msup`, `mtable`
  - `mtd` (`columnspan`, `rowspan`)
  - `mtext`, `mtr`
  - `munder` (`accentunder`)
  - `munderover` (`accent`, `accentunder`)
  - `semantics`

  Global attributes allowed include `dir`, `lang`, `title`, `display`, `displaystyle`, `mathbackground`, `mathcolor`, `mathsize` and `scriptlevel`. `comments` is false, `dataAttributes` false and `javascriptURLs` false. **`annotation`, `annotation-xml`, `maction`, `menclose`, `mglyph` and `malignmark` are not listed.** — [WHATWG HTML source, built-in safe default configuration](https://html.spec.whatwg.org/#built-in-safe-default-configuration)
- Spec rule: "If elementNamespace is the MathML namespace, localName is "href", namespace is null or the XLink namespace, and value contains a javascript: URL, then return false." — [WHATWG HTML sanitizer algorithms](https://html.spec.whatwg.org/#html-sanitization)

**Link attributes:**
- `href` on any MathML element: Safari since 10 (still supported), Firefox 7 to 151 (removed in 152), Chrome never. The MathML `<a>` element: Firefox 152 only. — [BCD](https://unpkg.com/@mdn/browser-compat-data/data.json)

**mXSS history involving MathML:**
- (2020) Michał Bentkowski's DOMPurify 2.0.17 bypass used `<math>` with `<style>` to confuse the HTML parser's namespace handling. — [Securitum: Mutation XSS via namespace confusion – DOMPurify < 2.0.17 bypass](https://research.securitum.com/mutation-xss-via-mathml-mutation-dompurify-2-0-17-bypass/); summarised by [PortSwigger](https://portswigger.net/research/bypassing-dompurify-again-with-mutation-xss)
- (7 Oct 2020) Gareth Heyes bypassed the patched version with `<math><mtext><table><mglyph><style><!--</style><img title="--&gt;&lt;img src=1 onerror=alert(1)&gt;">`. It worked in Chrome, and a CDATA variant worked in Firefox. DOMPurify's patch had looked for mutation in text nodes but not in comments. — [PortSwigger Research](https://portswigger.net/research/bypassing-dompurify-again-with-mutation-xss)
- (Oct 2024) CVE-2024-47875: a nesting-based mXSS in DOMPurify, fixed in 2.5.0 and 3.1.3, rated critical by NVD. — [GitLab advisory DB / CVE listings](https://db.gcve.eu/vuln/CVE-2024-47875) (secondary aggregator)
- (Feb 2025) CVE-2025-26791: DOMPurify before 3.2.4, where an incorrect template-literal regex with `SAFE_FOR_TEMPLATES` led to mXSS. — [GitLab advisory](https://advisories.gitlab.com/pkg/npm/dompurify/CVE-2025-26791/)
- (2026-03-25) GHSA-h8r8-wccr-v5f2 "DOMPurify mXSS via Re-Contextualization", affecting 3.3.1. Sanitized output re-inserted with `innerHTML` inside `script`, `xmp`, `iframe`, `noembed`, `noframes` or `noscript` wrappers mutates into executable markup, e.g. `<img src=x alt="</script><img src=x onerror=alert(1)>">`. — [GHSA-h8r8-wccr-v5f2](https://github.com/cure53/DOMPurify/security/advisories/GHSA-h8r8-wccr-v5f2)
- 2026 DOMPurify advisories (about 25 between March and October 2026) are mostly about `IN_PLACE` mode, hooks, `ADD_TAGS`/`CUSTOM_ELEMENT_HANDLING` config pollution, `SAFE_FOR_TEMPLATES`, and a high-severity `selectedcontent` re-clone XSS (CVE-2026-47423, 2026-05-19). None is titled as MathML-specific. The latest release is 3.4.17 (2026-10-12), which "hardened several smaller edge cases (namespaces, literal-text elements, isValidAttribute())". — [DOMPurify security advisories](https://github.com/cure53/DOMPurify/security/advisories); [DOMPurify releases](https://github.com/cure53/DOMPurify/releases)
- HTML parsing rules: the MathML `mi`, `mo`, `mn`, `ms` and `mtext` are **MathML text integration points**. `annotation-xml` with `encoding` of `text/html` or `application/xhtml+xml` is an **HTML integration point**, where the parser switches back to HTML. That switch is the mechanism behind namespace-confusion mXSS. — [WHATWG HTML parsing: tree construction](https://html.spec.whatwg.org/multipage/parsing.html#html-integration-point)
- MathML Core allows HTML inside `mtext` (its example shows `<mtext><input ...></mtext>`) and SVG/HTML inside `annotation-xml`. — [MathML Core §2.2.1, §3.8](https://w3c.github.io/mathml-core/#html-and-svg)
- KaTeX's `trust` option defaults to `false`. In that state `\href`, `\url`, `\includegraphics`, `\htmlClass` and similar commands are rendered in `errorColor` instead of emitted. A handler can allow specific commands or protocols. — [KaTeX options](https://katex.org/docs/options)

### Inferences
- **Recommended minimal allow-list for LaTeX output.**
  - Elements: `math`, `semantics`, `annotation` (text content only), `mrow`, `mi`, `mn`, `mo`, `mtext`, `mspace`, `ms`, `mfrac`, `msqrt`, `mroot`, `msub`, `msup`, `msubsup`, `munder`, `mover`, `munderover`, `mmultiscripts`, `mprescripts`, `mtable`, `mtr`, `mtd`, `mpadded`, `mphantom`, `mstyle`, `merror`. This is the WHATWG default minus `a`, plus `annotation`.
  - Attributes: `display` (on `math` only), `displaystyle`, `scriptlevel`, `dir`, `class` (if the converter emits classes), and a tightly validated `style`, or no `style` if the fixes stylesheet uses classes.
  - Per-element attributes: the `mo` and `mpadded`/`mspace` lists above, `accent`/`accentunder`, `columnspan`/`rowspan`, and `encoding` on `annotation` restricted to the value `application/x-tex`.
  - **Always drop:** `href`/`xlink:href` on every MathML element (because of Safari), the `<a>` element, `maction`, `annotation-xml`, `mglyph`, `malignmark`, `menclose` (or handle it explicitly), any HTML or SVG child inside `mtext`, every `on*` attribute, `mathcolor`/`mathbackground` (for theming), and anything not in the MathML namespace below `<math>`.
- **Sanitizing the DOM rather than a string, and inserting nodes with no re-parse, removes the mXSS class.** For option (a), the Rust converter can build nodes with `document.createElementNS(MATHML_NS, ...)` and `textContent` and never pass strings through `innerHTML`. No parse means no mutation. It should still assert that the output passes the allow-list, as defence in depth. If strings are used, sanitize and then insert the *sanitized DOM*. Never re-serialize and re-parse, and never wrap the result in rawtext elements, which is the GHSA-h8r8 lesson.
- `setHTML()` is not a complete answer in October 2026: Safari has none, and the default config strips the TeX `<annotation>`. A custom `Sanitizer` config would have to add `annotation`, and the Safari fallback would still need the app's own sanitiser. The repo's existing allow-list sanitiser stays necessary.
- `annotation` content is plain text under the HTML parser. It is not an integration point, unlike `annotation-xml`, so allowing it with text-only children and a fixed `encoding` value is low-risk.

### Gaps
- I could not retrieve the full text of the Securitum article (the fetch returned only the site chrome), so the exact Bentkowski payload is quoted second-hand via PortSwigger's summary.
- The CVE-2024-47875 details came from aggregators (gcve/circl), and the GitHub advisory text was not read directly. The GitHub API rate limit blocked retrieving pre-2026 advisory descriptions.
- I did not test whether Chrome 146 and Firefox 148 `setHTML()` actually keep the MathML elements from the spec table in practice.
- I found no specific 2025-2026 MathML-namespace DOMPurify bypass. The 2026 advisories are not MathML-titled, but a namespace-hardening release note exists (3.4.17).
