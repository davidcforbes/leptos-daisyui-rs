# math-core stylesheet (compiled in; nothing to serve)

`MarkdownView` and `MarkdownGraphicEditor` turn `$…$` and `$$…$$` into
native MathML with the Rust crate [math-core](https://github.com/tmke8/math-core)
(MIT), compiled into this library's wasm (`src/markdown/math/`, ldui-r0co).
There is no JavaScript, no CDN and no worker, and a host serves nothing.

| File | What it is |
|---|---|
| `mathmlfixes.css` | math-core's browser fixes for its MathML output: `mtd` padding, a Chromium `menclose` polyfill (`\cancel`, `\boxed`, `\phase`), accent spacing. Compiled in with `include_str!` and installed once as `<style id="ldui-mathmlfixes">` the first time a formula renders. |
| `LICENSE-math-core` | math-core's MIT licence |
| `gen_arity.py` | generates `src/markdown/math/arity.rs` (which commands take arguments, and how many) from the pinned version's `src/commands.rs`; the depth guard simulates nesting with it. Re-run on every math-core upgrade: `python -X utf8 assets/math-core/gen_arity.py <cargo registry>/math-core-<ver>/src/commands.rs src/markdown/math/arity.rs`, then `cargo fmt` |

## Provenance of `mathmlfixes.css` (2026-10-23)

- **Source:** `https://github.com/tmke8/math-core`, tag `v0.9.0`
  (`fc95cb704a8159467b69f1ebc93460de7c195ce1`), file `css/mathmlfixes.css`,
  git blob `1837369ce8ad5c7664cbb37c501c411332e7e5dd` (identical to the
  vendored file's `git hash-object`). The tag matches the
  `math-core = "=0.9.0"` pin in `Cargo.toml`; upgrade both together.
- **SHA-256** `15128a36f105da7de7b382e5d7fd26dc1f9bc814a29d13f639d4681ff8eda679`
  (2,395 bytes), pinned in `src/markdown/math/mod.rs`
  (`MATHML_FIXES_SHA256`) and checked by the gate's `test-lib` step.
  `.gitattributes` marks the file `-text`, so a CRLF checkout cannot change
  its bytes.

## Fonts

No font is vendored. The library's stylesheet asks for
`"Latin Modern Math", "STIX Two Math", "Cambria Math", math`, so Windows
draws with its built-in Cambria Math and macOS/iOS with STIX Two Math. A
host that wants identical rendering everywhere can self-host an OpenType
MATH font (Latin Modern Math, GUST licence, about 388 KB woff2) with an
`@font-face` named `"Latin Modern Math"` in its own CSS; nothing in this
crate has to change.
