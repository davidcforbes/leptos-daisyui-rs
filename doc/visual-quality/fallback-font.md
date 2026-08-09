# Silent font fallback

**Status:** automated (typography, since 2026-08-08)
**Seen in:** office-perf-web (op-edag.1)

## What it looks like

The page loads, nothing errors, and it looks *almost* right — slightly wider
letterforms, a slightly different x-height, numerals that don't quite line
up with the design mock. Manrope was declared everywhere in code and CSS, but
the browser is quietly rendering Segoe UI (or whatever the platform default
is) instead. Because nothing crashes and no console warning fires by default,
this is the defect class nobody files: it reads as "the designer's eye", not
a bug.

## Root cause

Three independent things have to agree for a web font to actually load:
the `@font-face` declaration, the font file being present at the path it
names, and the build's copy step actually putting that file where the
declaration expects it. In op-edag.1 the Trunk asset-copy directive and the
`@font-face` `src` disagreed after a `demo/` restructure — the file existed,
just not where the browser looked for it — so every element inherited the
generic fallback the `font-family` stack names second (or the UA default if
none was named).

## How to check (manual)

In DevTools: Network tab, filter `font`, reload — a missing/404'd font file
is the fast tell. Or in the console: `document.fonts.check('16px "Manrope"')`
returns `false` when the family never loaded, regardless of what
`getComputedStyle(el).fontFamily` reports (that string reflects what was
*requested*, not what's *rendering*).

## Automation

`ldui_audit::audit_page` (via the engine's generic sweep,
`pixelproof-style-audit`) runs exactly that `document.fonts.check` against
`StyleProfile.font_family` once per page and pushes a `family::TYPOGRAPHY`
violation with detail `declared family "<name>" is not loaded — text is
silently falling back` when it fails. `font_family` comes from
`ldui_audit::from_ui_tokens`, so the declared name is never hand-typed at the
call site. Caught by `cargo xtask test-style` (`tests/style_audit_smoke.rs`).
