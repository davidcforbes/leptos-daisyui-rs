# Rust crates for rendering LaTeX math in the browser from wasm32 with no JavaScript engine (as of 2026-10-23)

Scope: pure-Rust crates that can replace `window.katex.render` inside a Leptos 0.8 CSR wasm32-unknown-unknown app, emitting MathML (Core) or SVG, with output then passed through a Rust allow-list sanitiser.

**About the "measured" rows.** Some findings below come from measurements I ran myself on 2026-10-23, and they are labelled **[measured]**. The setup was one `cdylib` per crate, built with `--target wasm32-unknown-unknown`, `opt-level="z"`, `lto=true`, `codegen-units=1`, `panic="abort"` and `strip=true`, with no wasm-bindgen and no wasm-opt, on rustc/cargo 1.98.1. Each module was loaded in Node (default Rust wasm stack, 1 MiB) and given the same 32-case corpus, which includes hostile inputs. The harness, the generator and the raw results are in a scratch directory (the scripts and raw results are in `measurements/` beside these notes) (`gen.py`, `run.mjs`, `run2.mjs`, `results.tsv`, `results2.txt`). These are first-party measurements with no external URL; cite them as "local measurement".

## Which crates exist, and how alive is each? (version, date, activity, maintainers, downloads, licence)

### Takeaway
There are five credible pure-Rust candidates, all actively maintained in 2026:
- **math-core**: LaTeX to MathML Core.
- **pulldown-latex**: LaTeX to MathML Core.
- **katex-rs**: a native Rust port of KaTeX that emits KaTeX's HTML+MathML.
- **tex2math**: LaTeX to MathML.
- **RaTeX**: KaTeX-compatible layout to SVG.

`latex2mathml` (abandoned in 2020), ReX (abandoned in 2020) and the old `katex` crate (which wraps the JS engine) are not viable.

### Cited Findings
- **math-core** (tmke8/math-core):
  - Releases: 0.9.0 on 2026-10-12, then 0.8.2 (2026-09-01), 0.8.1 (2026-08-30), 0.8.0 (2026-08-27), 0.7.0 (2026-07-03) and 0.6.4 (2026-06-12). The crate was first published on 2025-06-16.
  - Downloads: 24,490 total, 21,291 recent. Licence: MIT. Repo has 45 stars and 47 open issues; last push 2026-10-20.
  - Activity: 100+ commits since 2026-01-01.
  - Contributors: tmke8 (1,458 commits), Jules-Bertholet (63), notriddle (29). Effectively a single maintainer with a few regular contributors.
  - It also ships a CLI (`math-core-cli`), a PyPI package and an npm package. MSRV is 1.96.
  - Sources: [crates.io](https://crates.io/crates/math-core); [GitHub](https://github.com/tmke8/math-core); [releases](https://github.com/tmke8/math-core/releases); [contributors](https://github.com/tmke8/math-core/graphs/contributors)
- **pulldown-latex** (carloskiki/pulldown-latex):
  - Releases: 0.8.0 on 2026-07-28. Before that, 0.7.1 was on 2024-11-21, a 20-month gap.
  - Downloads: 93,454 total, 57,456 recent. Licence: MIT. Repo has 73 stars and 30 open issues; last push 2026-08-21.
  - Activity: 67 commits since 2026-01-01.
  - Contributors: carloskiki (232), jonhoo (59), gjtorikian (41). 15 contributors in total.
  - Commits landed after 0.8.0 and are unreleased (a non-ASCII `\char` panic fix on 2026-08-19, merged 2026-08-21).
  - Sources: [crates.io](https://crates.io/crates/pulldown-latex); [GitHub](https://github.com/carloskiki/pulldown-latex); [commits](https://github.com/carloskiki/pulldown-latex/commits/main)
- **katex-rs** (the crate is `katex-rs`, the lib is `katex`; repo katex-rs/katex-rs):
  - What it is: a Rust re-implementation of KaTeX, not a binding. It tracks KaTeX 0.18.5 at upstream commit 49904aa.
  - Releases: 0.3.0 on 2026-09-05. The crate was created on 2025-09-23.
  - Downloads: 79,729 total, 59,981 recent. Licence: MIT. Repo has 56 stars and 2 open issues; last push 2026-10-23.
  - Activity: 55 commits since 2025.
  - Maintainer: effectively one person (sepcnt, 44 commits), plus bots.
  - Sources: [crates.io](https://crates.io/crates/katex-rs); [GitHub README](https://github.com/katex-rs/katex-rs)
- **tex2math** (twn39/tex2math):
  - Releases: 2.4.1 on 2026-08-26. The crate was created on 2026-03-30.
  - Downloads: 237 total. Licence: **LGPL-3.0-only**. Repo has 0 stars and one contributor.
  - Activity: 100+ commits since 2025.
  - Sources: [crates.io](https://crates.io/crates/tex2math); [GitHub](https://github.com/twn39/tex2math)
- **RaTeX** (erweixin/RaTeX):
  - Packaging: a crate family (`ratex-parser`, `ratex-layout`, `ratex-svg`, `ratex-wasm`, …), all at 0.1.15 dated 2026-10-16. The first crates were published on 2026-03-12.
  - Downloads: `ratex-layout` 139,033 total; `ratex-svg` 64,026; `ratex-wasm` 374.
  - Licence: MIT. The LICENSE file reads "MIT License"; GitHub's API reports NOASSERTION.
  - Repo: 1,546 stars, 17 contributors, 100+ commits since 2025; last push 2026-10-19.
  - Sources: [crates.io ratex-svg](https://crates.io/crates/ratex-svg); [GitHub](https://github.com/erweixin/RaTeX)
- **latex2mathml** (osanshouo/latex2mathml):
  - Releases: 0.2.3 on 2020-04-27, with no commits since 2020-04-21.
  - Downloads: 121,759 total, 35,895 recent. Licence: MIT. One contributor.
  - math-core started as a fork of this crate.
  - Sources: [crates.io](https://crates.io/crates/latex2mathml); [GitHub](https://github.com/osanshouo/latex2mathml); [math-core README acknowledgments](https://github.com/tmke8/math-core)
- **katex** (the old crate, xu-cheng/katex-rs):
  - Releases: 0.4.6 on 2023-02-05; last push 2024-05-03.
  - Downloads: 1,067,855 total, 255,550 recent, mostly through mdbook-katex. Licence: MIT OR Apache-2.0.
  - What it is: bindings that run the real `katex.min.js` (vendored through `include_str!`) inside a JS engine.
  - Sources: [crates.io](https://crates.io/crates/katex); [lib.rs source](https://github.com/xu-cheng/katex-rs/blob/master/src/lib.rs)
- **mathemascii** (nfejzic): AsciiMath, not LaTeX. 0.4.0 on 2023-12-27, no commits since 2024-01. 44,351 total downloads. Licence: Apache-2.0 on crates.io; the repo has no licence file detected. Sources: [crates.io](https://crates.io/crates/mathemascii); [GitHub](https://github.com/nfejzic/mathemascii)
- **ReX** (ReTeX/ReX):
  - Activity: last push 2020-07-15; 137 stars. GitHub detects no licence.
  - Not on crates.io under that name: the crates.io crate `rex` is an unrelated functional-programming language (4.0.0, 2026-08-24).
  - Sources: [GitHub](https://github.com/ReTeX/ReX); [crates.io rex](https://crates.io/crates/rex)
- **mitex** (mitex-rs/mitex): a LaTeX to **Typst** converter, not a renderer.
  - Releases: crates.io `mitex` 0.2.4 on 2024-06-17. GitHub releases go up to 0.2.7 (2026-04-07); the later releases are of the Typst package and are not on crates.io.
  - Repo: 638 stars, 16 contributors. Licence: Apache-2.0.
  - Sources: [crates.io](https://crates.io/crates/mitex); [GitHub](https://github.com/mitex-rs/mitex)
- **typst / typst-svg**: 0.15.1 on 2026-07-17, Apache-2.0, more than 3.4M downloads. The browser compiler package `@myriaddreamin/typst-ts-web-compiler` 0.7.0 unpacks to **28.4 MB**. Sources: [crates.io typst](https://crates.io/crates/typst); [npm typst-ts-web-compiler](https://www.npmjs.com/package/@myriaddreamin/typst-ts-web-compiler)
- **mathml-rs** (0.1.2, 2021) and **mathml** (0.4.4, 2020) are MathML *parsers*; neither converts LaTeX. Sources: [crates.io mathml-rs](https://crates.io/crates/mathml-rs); [crates.io mathml](https://crates.io/crates/mathml)
- **tex2mathml**, **latex_to_mathml**, **texmath** and **temml** do not exist on crates.io (HTTP 404 on 2026-10-23). Source: [crates.io API](https://crates.io/api/v1/crates/temml)
- **Newer or minor candidates** found by searching crates.io:
  - `latex-rust` 3.0.1: "Pure-Rust LaTeX math renderer … emit SVG, PNG, or egui shapes". Created 2026-09-02; 4,796 downloads; 2 stars; MSRV **1.99**; MIT OR Apache-2.0.
  - `mathtex` 0.2.0: a TeX engine translated by C2Rust, with an **unsafe** core and a dumped "format file". 0 stars; 134 downloads.
  - `mathra` 1.2.0: LaTeX to SVG, created 2026-10-20; 155 downloads.
  - `katex-parser` 0.1.1: AGPL-3.0; only a Unicode backend.
  - `formulary` 0.3.0: MathML Core *layout*, not LaTeX input.
  - Sources: [crates.io latex-rust](https://crates.io/crates/latex-rust); [mathtex README](https://github.com/gabriel-nsiqueira/mathtex); [crates.io mathra](https://crates.io/crates/mathra); [crates.io katex-parser](https://crates.io/crates/katex-parser)

### Inferences
- The two most mature MathML Core converters are math-core and pulldown-latex. math-core has the faster release cadence in 2026 (six releases between June and October). pulldown-latex has more contributors but releases irregularly.
- katex-rs and RaTeX both carry KaTeX's semantics into Rust, which makes them the closest drop-in visual match to the current KaTeX output.
- Bus factor is about one for math-core, katex-rs and tex2math.
- tex2math's LGPL-3.0-only licence is a compliance consideration when the crate is statically linked into a single wasm binary. Whether that is acceptable is a legal question this research does not answer.

### Gaps
- I did not use crates.io download time series beyond total/recent, and I did not gather GitHub traffic.
- I did not check the reverse dependencies of katex-rs.

## What does each crate output, and does it target MathML Core?

### Takeaway
- math-core and pulldown-latex explicitly target MathML Core.
- tex2math emits a "MathML Core-friendly" mode by default, but uses `mstyle`/`mathcolor`/`columnlines` and similar, which are outside Core.
- katex-rs emits KaTeX's HTML span tree plus hidden MathML, with inline styles and SVG.
- RaTeX emits SVG that, by default, relies on KaTeX webfonts.
- latex2mathml emits legacy full MathML.

### Cited Findings
- **math-core**:
  - Converts "LaTeX math to MathML Core"; its goals include to "avoid CSS hacks … (and definitely don't use JavaScript in any way)" and to "keep the compiled WebAssembly code small".
  - It requires `css/mathmlfixes.css` on the page. That stylesheet resets `<mtd>` padding, polyfills `<menclose>` in Chromium (used by `\cancel`/`\sout`), and adds accent margins.
  - It recommends patched math fonts (New Computer Modern, Libertinus or Noto Sans Math) from math-core-fonts. A subset of New Computer Modern is about 450 kB as WOFF2, or about 200 kB without `ssty`.
  - Source: [math-core README](https://github.com/tmke8/math-core)
- **pulldown-latex**: provides "a MathML renderer" that "closely follows the MathML Core specification". It needs a stylesheet plus fonts; the docs show them loaded from jsDelivr, or downloaded from the releases page. Source: [docs.rs pulldown_latex](https://docs.rs/pulldown-latex/latest/pulldown_latex/)
- **katex-rs**: "converts LaTeX math into HTML and MathML". `render_to_string` produces "KaTeX-compatible HTML + MathML markup". Source: [katex-rs README](https://github.com/katex-rs/katex-rs)
- **RaTeX SVG**:
  - By default, `ratex-svg` turns each glyph into an SVG `<text>` element using KaTeX font-family names, and the host page must load the KaTeX stylesheets.
  - The `standalone` and `embed-fonts` features emit `<path>` glyphs instead.
  - In the browser, `ratex-wasm` emits DisplayList JSON for a Canvas 2D or `<ratex-formula>` web component.
  - Sources: [docs.rs ratex_svg](https://docs.rs/ratex-svg/latest/ratex_svg/); [RaTeX README](https://github.com/erweixin/RaTeX)
- **tex2math**: the default is `mathml_core: true` ("Core-friendly"); `--no-mathml-core` gives legacy `<menclose>`; it also has an experimental MathML 4 `intent` mode. Source: [tex2math README](https://github.com/twn39/tex2math)
- **Vocabulary each crate actually emitted [measured]**, over a 20-formula corpus:
  - **math-core**: tags `math, mfrac, mi, mmultiscripts, mn, mo, mover, mphantom, mprescripts, mroot, mrow, mspace, msub, msubsup, msup, mtable, mtd, mtext, mtr, munder, munderover`. Attributes `accent, accentunder, display, displaystyle, form, linethickness, lspace, mathvariant, maxsize, minsize, rspace, scriptlevel, stretchy, style, width`.
  - **pulldown-latex**: the same family of tags, without `mmultiscripts`/`mphantom`. Its attributes add `class, movablelimits, symmetric`.
  - **tex2math**: adds `mstyle`, `none`, `mathcolor`, `mathbackground`, `columnalign`, `columnlines`, `rowlines`, `columnspacing` and `xmlns`.
  - **katex-rs**: adds `span, svg, path, line, semantics, annotation, menclose, mpadded`, plus `aria-hidden, class, style, d, viewBox, preserveAspectRatio` and others.
  - **RaTeX**: `svg, path, rect, text`, with `font-family`, `fill`, `d` and others.
- **Inline `style` attributes [measured]**:
  - math-core writes `style` for `\color` (`style="color:#FF0000;"`) and for alignment cells. Examples: `aligned` gives `style="text-align: right;justify-items: end;padding-right: 0;"`; `align` gives `style="width: 100%"`; an `array` with `|` gives `style="border-right: 0.05em solid currentcolor;…"`.
  - pulldown-latex writes `style="color: rgb(255 0 0)"`.
  - tex2math uses `mathcolor="red"` instead of `style`.

### Inferences
- math-core and pulldown-latex have the smallest output vocabularies, which makes them the easiest to allow-list.
- Whichever MathML crate is chosen, the sanitiser must either allow a constrained `style` attribute (colour, text-align, justify-items, padding, border, width) or translate those values to `mathcolor` and similar. Otherwise `\color`, `aligned`/`align` and `array` rules will silently lose formatting.
- Inline `style` attributes inserted by `innerHTML` fall under the page's CSP `style-src` (`style-src-attr`). If the consumer's CSP lacks `'unsafe-inline'` for styles, those declarations will be dropped. This should be checked against the actual policy.
- katex-rs removes the JS dependency but not the KaTeX CSS and font dependency. It also emits far more markup, roughly 10 to 15 times the bytes of math-core [measured]: the quadratic formula is 3,848 B against 268 B.

### Gaps
- I did not visually compare rendering of each crate's output in Chrome, Firefox and Safari. math-core's README points to Temml's [MathML browser status page](https://temml.org/docs/en/mathml-status) for known browser bugs.

## LaTeX coverage and fidelity

### Takeaway
- math-core publishes a coverage table against KaTeX: in its own comparison page, 52 of about 1,046 KaTeX commands produce math-core's unknown-command error.
- pulldown-latex claims about 95% of KaTeX.
- katex-rs and RaTeX claim KaTeX parity, and RaTeX also supports mhchem.
- latex2mathml lacks `aligned`, `cases` and `array`.
- In my corpus, math-core handled every mainstream construct.

### Cited Findings
- **math-core's comparison table** ([comparison.html](https://tmke8.github.io/math-core/comparison.html)) is adapted from Temml's table and compares against KaTeX 0.17.0. Counting the cells that carry `<merror class="math-core-unknown-cmd">` gives 52 of 1,046 rows. The list is:
  - `\above, \allowbreak, \ce, \centerdot, \char, \colorbox, \cr, \dotsb, \dotsc, \dotsi, \dotsm, \dotso`
  - `\edef, \fcolorbox, \futurelet, \hbox, \intop, \llap, \long, \mathclap, \mathellipsis, \mathllap`
  - `\mathnormal, \mathrlap, \mathsfit, \nobreak, \noexpand, \nshortmid, \nshortparallel, \overlinesegment`
  - `\pmb, \pu, \rlap, \rule, \sdot, \shortmid, \shortparallel, \smallfrown, \smallsetminus, \smallsmile`
  - `\smash, \substack, \textmd, \textnormal, \textup, \thickapprox, \thicksim, \underlinesegment, \varpropto, \vcenter, \verb, \xdef`
  - Tracking issues for missing environments and commands: [#154](https://github.com/tmke8/math-core/issues/154) and [#155](https://github.com/tmke8/math-core/issues/155).
  - Nested math mode inside `\text` is not implemented: [#431](https://github.com/tmke8/math-core/issues/431).
  - `\mathit{012}` (italic digits) will not be supported.
- **math-core macros**:
  - `LatexToMathML::new(config, macros: Vec<(String, String)>)` takes predefined macros, with arguments allowed (`\mathbb{#1}`).
  - `global_group` controls whether `\newcommand` persists across snippets; the default is local, "matches the default behavior of KaTeX".
  - Source: [docs.rs LatexToMathML](https://docs.rs/math-core/latest/math_core/struct.LatexToMathML.html); [MathCoreConfig](https://docs.rs/math-core/latest/math_core/struct.MathCoreConfig.html)
- **pulldown-latex**:
  - Claims "95% of what KaTeX and the likes support is properly working and minimally tested", and says it is "not recommended for large scale production use as more robust testing is required".
  - Known gaps: `\sideset`, raise/lower boxes, deprecated `\over`/`\atop`, `\edef`, `\csname`, `\hfil`.
  - Unsupported KaTeX options include the macros preamble, `ThrowOnError`, `maxSize` and `trust`.
  - math-core's README (June 2025) says pulldown-latex then did not distinguish `\mathcal` from `\mathscr`.
  - Sources: [pulldown-latex README](https://github.com/carloskiki/pulldown-latex); [math-core README](https://github.com/tmke8/math-core)
- **katex-rs**: "Spec-driven test suite. Rust tests mirror the upstream KaTeX spec cases". The project status claims "Pixel perfect rendering with KaTeX across all platforms", backed by an automated screenshot regression harness. Source: [katex-rs README](https://github.com/katex-rs/katex-rs)
- **RaTeX**:
  - "Aligned with KaTeX's math syntax"; publishes a [support table](https://erweixin.github.io/RaTeX/demo/support-table.html) against KaTeX.
  - Supports mhchem `\ce`/`\pu` and bussproofs.
  - `\htmlClass`/`\htmlId`/`\htmlData` drop their attribute argument.
  - Source: [RaTeX README](https://github.com/erweixin/RaTeX)
- **latex2mathml**: supports `matrix`/`pmatrix`/`bmatrix`/`vmatrix` and "experimental" `align`. It does not support `\\` or `&` outside those environments. Source: [latex2mathml README](https://github.com/osanshouo/latex2mathml)
- **My corpus [measured]**, covering `\frac/\sqrt`, `aligned`, `align`, `pmatrix`, `cases`, `array` with `\hline`, accents, `\operatorname`, `\mathbb/\mathcal/\mathfrak/\mathscr/\mathbf`, `\text`, `\color`/`\textcolor`, `\sum/\int/\lim`, `\binom/\overset/\underbrace`, `\boxed/\cancel` and `\ce`:
  - **math-core**: every mainstream case succeeded. The only failures were `\ce` (unsupported) and my `\newcommand{\R}` test, which errored with "This command is already defined" because math-core predefines `\R`.
  - **pulldown-latex 0.8.0**: failed the font line, the `\boxed`/`\cancel` line and `\newcommand{\R}{…}`, each with "unknown primitive command found" or "expected an argument".
  - **latex2mathml**: failed `aligned`, `cases` and `array` ("unknown environment").
  - **katex-rs**: parsed everything mainstream, but needs mhchem for `\ce`.
  - **RaTeX**: rendered `\ce{H2O}`.
  - **tex2math**: "succeeded" on unknown commands by default, rendering `\foo` as a literal `<mi>\foo</mi>`.

### Inferences
- For Markdown chat or notes content, math-core's gaps (mostly `\dots*` variants, `\substack`, `\smash`, `\rlap`, `\colorbox`, mhchem) are minor. `\substack` and `\colorbox` are the most likely to appear in real content.
- pulldown-latex's failures on common font and `\boxed` commands are a fidelity risk. Which exact command failed in the combined line was not isolated.

### Gaps
- I did not find a published per-command pass rate for pulldown-latex, tex2math or katex-rs.
- I did not compute RaTeX's support-table score; the page is rendered by JS.
- I did not isolate which single command failed in pulldown-latex's font and `\boxed`/`\cancel` lines.

## Error handling: does each crate return a Result with a message?

### Takeaway
- math-core returns `Result<ConvertResult, Box<LatexError>>` with a byte range and plain or HTML formatters. That fits a "show source plus error" fallback directly.
- katex-rs and RaTeX return a `ParseError` with KaTeX-style messages.
- pulldown-latex reports errors as `Err` items in its event iterator, and renders them inline in red by default.
- tex2math defaults to silently rendering unknown commands as identifiers.
- latex2mathml embeds `[PARSE ERROR: …]` text for unknown commands.

### Cited Findings
- **math-core**:
  - `convert_with_local_state(&self, latex, display) -> Result<ConvertResult, Box<LatexError>>`.
  - `LatexError(Range<usize>)` offers `to_html(latex, display, css_class)`, `error_message()`, `to_message()` and `label()`, and implements `std::error::Error`.
  - `ignore_unknown_commands: bool` renders unknown commands as red text instead of returning an error.
  - `LatexToMathML::new` returns `Err(MacroParseError)` for bad predefined macros.
  - Sources: [LatexError docs](https://docs.rs/math-core/latest/math_core/struct.LatexError.html); [MathCoreConfig docs](https://docs.rs/math-core/latest/math_core/struct.MathCoreConfig.html)
- **math-core messages [measured]**: `Unknown command "\foo".`, `Expected closing token "}", but reached end of input.`, `Expected "\end{matrix}", but found "\end{pmatrix}".`, `Too many expansions of custom commands. A command may be expanding to itself.`
- **pulldown-latex**: `Parser` yields `Result<Event, ParserError>`. `RenderConfig.error_color` (default `(178,34,34)`) colours "errors and invalid LaTeX commands" inline, and KaTeX's `ThrowOnError` is listed as unsupported. Messages [measured]: `parsing error: unknown primitive command found` and `unbalanced group found, expected it to be closed with }`. Sources: [docs.rs RenderConfig](https://docs.rs/pulldown-latex/latest/pulldown_latex/config/struct.RenderConfig.html); [README](https://github.com/carloskiki/pulldown-latex)
- **katex-rs**: `render_to_string(&ctx, latex, &settings) -> Result<String, katex::ParseError>`. Messages [measured] match KaTeX.js, for example `KaTeX parse error: Undefined control sequence: \foo at position 1`. Source: [katex-rs README](https://github.com/katex-rs/katex-rs)
- **RaTeX**: `ratex_parser::parse` returns `ParseResult` with `ParseError`, for example [measured] `ParseError at position 0: Undefined control sequence: \foo`. Source: [docs.rs ratex_parser](https://docs.rs/ratex-parser/latest/ratex_parser/)
- **tex2math**:
  - Has a typed error (`(NestingLimit)` with a byte offset).
  - The default `UnknownCommandPolicy` renders unknown commands as identifiers; `UnknownCommandPolicy::Error` renders `<merror>` instead.
  - It does "tolerant recovery" for a missing `}` or an unclosed environment, emitting `<merror>`.
  - Source: [tex2math README](https://github.com/twn39/tex2math)
- **latex2mathml**: `latex_to_mathml(s, DisplayStyle) -> Result<String, LatexError>` errors on unknown environments and a missing `}`, but unknown *commands* return `Ok` with `<mtext>[PARSE ERROR: Undefined("Command(\"color\")")]</mtext>` [measured].

### Inferences
- math-core's `Result` plus `error_message()` maps one-to-one onto the existing pele-style "show source + error" fallback.
- pulldown-latex can be made strict by scanning the iterator for `Err` before rendering (this is what my harness did), but that costs two parses.

### Gaps
- I did not test whether any crate panics, rather than returning `Err`, on a broad fuzz corpus beyond the cases listed under security below.

## wasm32-unknown-unknown compatibility, dependency footprint and added size

### Takeaway
- **[measured]** Every pure-Rust candidate compiled for wasm32-unknown-unknown with no wasm-bindgen.
- Size added over an 18 KB baseline:
  - MathML converters (math-core, pulldown-latex, tex2math): about +200 KB raw, about +65 to 75 KB brotli.
  - katex-rs: about +795 KB raw, about +211 KB brotli.
  - RaTeX parser, layout and SVG without fonts: about +2.26 MB raw, about +495 KB brotli.
- The old `katex` crate cannot work without a JS engine.

### Cited Findings
- **Sizes [measured]**. The baseline is a trivial cdylib using `String`/`format!`; all figures are before wasm-opt.

  | Module | Raw bytes | gzip -9 | brotli -q11 | Added over baseline (raw / brotli) |
  |---|---|---|---|---|
  | baseline | 18,401 | 7,688 | 6,750 | — |
  | latex2mathml 0.2.3 | 113,630 | 39,948 | 32,927 | +95 KB / +26 KB |
  | math-core 0.9.0 (`default-features=false`, no_std+alloc) | 220,776 | 86,142 | 72,618 | +202 KB / +66 KB |
  | tex2math 2.4.1 | 222,264 | 87,163 | 73,634 | +204 KB / +67 KB |
  | pulldown-latex 0.8.0 | 224,359 | 81,369 | 68,324 | +206 KB / +62 KB |
  | katex-rs 0.3.0 (`default-features=false`) | 812,789 | 267,929 | 217,900 | +794 KB / +211 KB |
  | RaTeX parser+layout+svg 0.1.15 (no fonts) | 2,278,612 | 697,937 | 502,032 | +2.26 MB / +495 KB |

- **Published npm packages**, as a cross-check (these include JS glue and typings):
  - math-core@0.9.0: 329,795 B unpacked, 7 files.
  - katex-rs@0.1.1: 885,196 B.
  - ratex-wasm@0.1.15: 3,925,969 B, 87 files.
  - KaTeX JS katex@0.19.0: 4,043,696 B, 225 files including fonts.
  - temml@0.14.0: 2,165,141 B.
  - Sources: [npm math-core](https://www.npmjs.com/package/math-core); [npm katex-rs](https://www.npmjs.com/package/katex-rs); [npm ratex-wasm](https://www.npmjs.com/package/ratex-wasm); [npm katex](https://www.npmjs.com/package/katex)
- **math-core's own wasm work**: PR [#1220](https://github.com/tmke8/math-core/pull/1220) ("Optimize the npm wasm binary with wasm-opt", 2026-09-10) and PR [#1256](https://github.com/tmke8/math-core/pull/1256) ("Re-enable phf's ptrhash feature for WASM", 2026-10-13). The repo has a `wasm_ci.yml` workflow. Source: [math-core repo](https://github.com/tmke8/math-core)
- **math-core dependencies**: `math-core-renderer-internal`, `phf` (macros), `strum`/`strum_macros`, `static_assertions`, `rustc-hash`, `hashbrown`, `kstring` and `memchr`, plus optional `serde` and `ariadne`.
  - The `std` feature is default; with `default-features = false` "the crate [is] `no_std`; the `alloc` crate is still required". `ariadne` requires std.
  - None of these use threads or `std::time`.
  - Sources: [math-core Cargo.toml](https://github.com/tmke8/math-core/blob/main/crates/math-core/Cargo.toml); [docs.rs features](https://docs.rs/math-core/latest/math_core/)
- **katex-rs dependencies**: `bon, phf, rapidhash, strum, strum_macros, thiserror`. wasm-bindgen, js-sys, web-sys and console_error_panic_hook come in only through the opt-in `wasm` feature (for its npm build). Source: [crates.io katex-rs 0.3.0](https://crates.io/crates/katex-rs)
- **ratex-parser dependencies**: `fancy-regex`, `regex`, `regex-lite`, `serde`, `serde_json`, `unicode-normalization`, `thiserror`, plus the ratex crates. Source: [docs.rs ratex_parser](https://docs.rs/ratex-parser/latest/ratex_parser/)
- **pulldown-latex**: MSRV 1.74.1. Its crate tarball is 799 KB, the largest of the MathML crates, because it ships fonts and CSS in the package. Source: [crates.io](https://crates.io/crates/pulldown-latex)
- **The old `katex` crate (xu-cheng) on wasm32**:
  - Backends: `quick-js` (the default) and `duktape` are C engines.
  - `wasm-js` uses `js_sys::eval(code)` to evaluate the bundled `katex.min.js` string in the host page's JS engine.
  - So on wasm32 it still runs KaTeX's JavaScript, and through `eval`, which is blocked by a `script-src 'self'` CSP that lacks `'unsafe-eval'`.
  - Sources: [katex README](https://github.com/xu-cheng/katex-rs); [wasm_js.rs](https://github.com/xu-cheng/katex-rs/blob/master/src/js_engine/wasm_js.rs); [lib.rs](https://github.com/xu-cheng/katex-rs/blob/master/src/lib.rs)
- **latex-rust**: MSRV is 1.99 (stated in its README), above the 1.98.1 toolchain used here, so I did not build it. Source: [latex-rust README](https://github.com/jscarr64/latex-rust)
- **mathtex**: runs on wasm32-unknown-unknown but needs "a packaged format file" dumped by a wasm32 build, and its engine core is "unsafe Rust" translated by C2Rust from TeX, eTeX and XeTeX. Source: [mathtex README](https://github.com/gabriel-nsiqueira/mathtex)
- **Typst as a library in the browser**: the typst.ts web compiler npm package is 28.4 MB unpacked. You would also need mitex to translate LaTeX to Typst, plus fonts. Sources: [npm typst-ts-web-compiler](https://www.npmjs.com/package/@myriaddreamin/typst-ts-web-compiler); [mitex](https://github.com/mitex-rs/mitex)

### Inferences
- In this measurement, wasm-opt and the existing Leptos binary's already-linked `alloc`/`core` code were not counted. The real increment inside the Leptos wasm should therefore be somewhat below these "added" numbers, perhaps around 60 to 70 KB brotli for the MathML crates. This is an estimate, not a measurement.
- The MathML crates also need math fonts and CSS. math-core: a font subset of about 200 to 450 kB WOFF2, which is optional because a system math font works, plus mathmlfixes.css.
- katex-rs and RaTeX need the KaTeX fonts and CSS (or embedded glyphs), served same-origin.
- typst is out of proportion for rendering single formulas.

### Gaps
- I ran no wasm-opt pass, because wasm-opt is not installed.
- I did not measure the increment inside an actual Leptos binary.
- I did not measure math-core with `std` enabled.

## Security: href/javascript:, raw HTML passthrough, macro bombs and deep nesting

### Takeaway
- None of the MathML crates emitted `href`, `javascript:` URLs, `id` attributes or raw HTML in my tests.
  - math-core rejects `\href`, `\url`, `\htmlId` and `\includegraphics` as unknown commands.
  - All of them escape `\text{<script>}`.
- Macro loops and bombs are bounded in math-core (1,000 expansions), katex-rs, RaTeX and pulldown-latex.
- **Deep nesting is the real hazard.** math-core, latex2mathml and katex-rs overflow the wasm stack and trap. tex2math (limit 64) and RaTeX return an error. pulldown-latex survives but goes quadratic.
- pulldown-latex and latex2mathml emit unescaped `<` and `&` in text nodes.

### Cited Findings
- **Hostile input results [measured]**:
  - `\href{javascript:alert(1)}{click}`, `\url{javascript:…}`, `\htmlId`/`\htmlClass`/`\htmlStyle` and `\includegraphics{http://evil/x.png}`:
    - math-core returns `Err(Unknown command)` for all of them.
    - pulldown-latex returns `Err` ("unknown primitive command").
    - tex2math and latex2mathml render the command name as text.
    - katex-rs (default `trust`) produced no `href` attribute; the `javascript:` string appeared only inside the `<annotation encoding="application/x-tex">` source text.
    - RaTeX drew the text as SVG `<text>`.
  - `\text{<script>alert(1)</script>}` was escaped to `&lt;script&gt;` by math-core, pulldown-latex, tex2math and katex-rs.
- **Unescaped characters [measured]**: for the input `a<b \& c`:
  - math-core emits `<mo>&lt;</mo>…<mo>&amp;</mo>`.
  - tex2math escapes correctly.
  - **pulldown-latex emits `<mo><</mo>…<mi>&</mi>`**, and latex2mathml emits `<mo><</mo>…<mi mathvariant="normal">&</mi>`. That is not well-formed XML.
- **Macro limits**:
  - math-core: `MaxExpansions`, "default is 1000, which is the same limit that KaTeX uses for its maxExpand setting". Source: [docs.rs MaxExpansions](https://docs.rs/math-core/latest/math_core/struct.MaxExpansions.html)
  - [measured] A self-recursive `\newcommand{\a}{\a\a}` and a 10^6-expansion `\def` bomb both returned `Err("Too many expansions…")` in about 2 to 6 ms in math-core. katex-rs and RaTeX gave "Too many expansions". pulldown-latex gave "macro expansion depth limit exceeded".
- **Deep nesting [measured]** (wasm, default 1 MiB stack). The trap is `RuntimeError: memory access out of bounds`, which is a stack overflow.

  | Crate | Maximum safe nesting depth before trapping |
  |---|---|
  | math-core 0.9.0 | `{…}` 849; `\frac{…}{y}` 451; `x^{…}` 797 |
  | latex2mathml | `{…}` 1,363; `\frac` 719; `^{…}` 1,148 |
  | katex-rs 0.3.0 | `{…}` 498; `\frac` 203; `^{…}` 224 |
  | tex2math | `Err(NestingLimit): Maximum nesting depth (64) exceeded` (configurable `--max-depth`) |
  | RaTeX | `Err(ParseError: Recursion limit exceeded)` |
  | pulldown-latex 0.8.0 | no trap up to 20,000, but time grows quadratically: `\sqrt` nesting of 500 took 23 ms, 1,000 took 81 ms, 2,000 took 280 ms, 4,000 took 1,094 ms and 10,000 took 10.7 s |

  - Searching the math-core repo found no `max_depth`/`recursion`/`stacker` code; a `fuzz/fuzz_targets/vs_katex.rs` target exists. Source: [math-core fuzz target](https://github.com/tmke8/math-core/tree/main/fuzz)
- **pulldown-latex issue history**:
  - Open panic issues on malformed input: [#57](https://github.com/carloskiki/pulldown-latex/issues/57) (lex.rs:387) and [#58](https://github.com/carloskiki/pulldown-latex/issues/58) (parser.rs:500), both opened 2026-05-31 and still open.
  - Fixed: [#43](https://github.com/carloskiki/pulldown-latex/issues/43) (stack overflow through recursive macro expansion) and [#44](https://github.com/carloskiki/pulldown-latex/issues/44) (arithmetic overflow).
  - Fuzzing was added in [#46](https://github.com/carloskiki/pulldown-latex/pull/46) (2026-03).
  - "Bound the depth of unbraced control sequence arguments" landed in [#73](https://github.com/carloskiki/pulldown-latex/pull/73) on 2026-08-22, after the 0.8.0 release.
  - The open PR [#74](https://github.com/carloskiki/pulldown-latex/pull/74) replaces recursion with a loop.
- **RaTeX hardening**:
  - [#143](https://github.com/erweixin/RaTeX/issues/143) (an "uncatchable stack overflow" in `layout_accent`) and [#144](https://github.com/erweixin/RaTeX/issues/144) (a recursion guard of 512 that overflowed on stacks under 8 MiB) were both fixed.
  - [#146](https://github.com/erweixin/RaTeX/pull/146) adds a stack-safety and input-depth policy.
  - [#171](https://github.com/erweixin/RaTeX/pull/171) (2026-10-16) "enforce[s] resource limits across parsing, layout, and rendering". `ratex_layout` exposes `MAX_LAYOUT_OUTPUT_ITEMS` and `validate_layout`.
  - Source: [docs.rs ratex_layout](https://docs.rs/ratex-layout/latest/ratex_layout/)

### Inferences
- For math-core, the practical mitigation is a cheap pre-scan that rejects input whose `{`, `\left`, `\begin` or `^`/`_` nesting exceeds about 100 to 200 before calling the converter. A larger wasm stack (`-C link-arg=-zstack-size=…`) is an alternative. Inside a Leptos app the stack already holds frames, so the real headroom is below the measured 451 to 849.
- A wasm trap from Rust leaves the instance in an undefined state. In a CSR app this effectively kills the UI, so the pre-scan matters more than it would in native code. This is an inference from wasm semantics; I did not test it in Leptos.
- Even with a well-escaping crate, the planned Rust allow-list sanitiser should parse the output as HTML, not XML, and re-serialise it. That re-serialising step neutralises the unescaped `<` that pulldown-latex and latex2mathml produce.

### Gaps
- I found no published security audit, RUSTSEC advisory search result or `cargo-fuzz` corpus result for math-core or katex-rs.
- I did not test inputs with very wide (not deep) structures, such as 100k-cell matrices.

## Who uses each crate?

### Takeaway
- math-core is the converter in rustdoc's LaTeX-math implementation PR (RFC 3958 was merged), and it has a handful of documentation-tool dependents.
- pulldown-latex is already used by a Leptos chat crate.
- The old `katex` crate's adoption (mdbook-katex) is server- and native-side only.
- The Markdown parsers only *delimit* math; they do not render it.

### Cited Findings
- **rustdoc**:
  - RFC [rust-lang/rfcs#3958](https://github.com/rust-lang/rfcs/pull/3958), "Rustdoc LaTeX math", was merged on 2026-05-01. Tracking issue: [rust-lang/rust#162365](https://github.com/rust-lang/rust/issues/162365), which is open.
  - The implementation PR [rust-lang/rust#156032](https://github.com/rust-lang/rust/pull/156032) by notriddle (still open) adds `math-core = "0.6.1"` to `src/librustdoc/Cargo.toml`, together with `mathml.css` and a modified Noto Sans Math WOFF2.
- **math-core reverse dependencies** (7): meread, umd, mdquiz, rinx_renderer, math-core-cli, little-docs, adocers-html. Source: [crates.io reverse deps](https://crates.io/crates/math-core/reverse_dependencies)
- **pulldown-latex reverse dependencies** (11), including `leptos-rich-chat` 0.2.2 ("Leptos chat components that render Markdown, LaTeX math", 2026-10-18), iced_math, waterui-math, panache-formatter and markdown-it-latex. Sources: [crates.io reverse deps](https://crates.io/crates/pulldown-latex/reverse_dependencies); [leptos-rich-chat](https://crates.io/crates/leptos-rich-chat)
- **latex2mathml reverse dependencies** (22), including ftml (Wikidot), tpnote-lib and org-rust-exporter. Source: [crates.io](https://crates.io/crates/latex2mathml/reverse_dependencies)
- **katex (old crate) reverse dependencies** (24): `mdbook-katex` 0.10.0 depends on `katex ^0.4.6`, plus pandoc-katex, aiken-project and others. Source: [crates.io mdbook-katex](https://crates.io/crates/mdbook-katex)
- **mitex reverse dependencies** (10), including tylax and egui_commonmark_backend_extended. Source: [crates.io](https://crates.io/crates/mitex/reverse_dependencies)
- **zola**: PR [getzola/zola#2791](https://github.com/getzola/zola/pull/2791), "add support for math rendering with typst and katex", is open (opened 2025-02-07).
- **pulldown-cmark**: `Options::ENABLE_MATH` emits `Event::InlineMath` / `Event::DisplayMath` that "conventionally contain TeX formulas", and does no rendering. The latest version is 0.13.5 (2026-10-21). Source: [docs.rs Options](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html)
- **comrak** 0.57.0 (2026-10-21): `math_dollars`, `math_latex` and `math_code` emit `<span data-math-style="inline|display">` or `<code data-math-style=…>` wrappers, with no rendering. Source: [docs.rs comrak Extension](https://docs.rs/comrak/latest/comrak/options/struct.Extension.html)

### Inferences
- rustdoc's choice of math-core, with a rustdoc maintainer contributing upstream (notriddle, 29 commits), is the strongest institutional signal among the candidates. Because the PR has not merged yet, it is an endorsement in progress, not a shipped dependency.
- Whichever converter is chosen, a pulldown-cmark or comrak front end would only supply the delimited TeX strings. The converter call replaces `katex.render` one for one.

### Gaps
- I did not verify Lemmy's or Zulip's current math approach.
- I did not check whether mdBook core has adopted a MathML converter; I found only mdbook-katex, which uses the JS-engine `katex` crate.
- I did not find any blog post comparing the Rust LaTeX-to-MathML converters beyond math-core's own "Alternatives" section, dated June 2025, which says none of the alternatives then rendered `\vec` and `\hat` correctly. Source: [math-core README](https://github.com/tmke8/math-core)
