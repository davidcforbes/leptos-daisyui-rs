# Silent font fallback

**Status:** partially automated (typography, since 2026-08-08) — catches a
declared font that fails to load, does NOT catch a missing `@font-face`
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
is the fast tell. `getComputedStyle(el).fontFamily` is NOT a tell: it reports
the declared stack, so it reads `"Manrope", ...` byte-for-byte identically on
a healthy page and on one rendering Segoe UI.

`document.fonts.check('16px "Manrope"')` is the better console probe, but
know its blind spot before you trust it: it answers "are all *matching*
`FontFace` objects loaded?", and when the `@font-face` block is absent
entirely there are no matching faces, so it returns **`true` vacuously** —
the one case you most wanted it to catch. Confirm the face exists first:
`[...document.fonts].map(f => [f.family, f.status])`. An empty list on a page
that declares a web font is itself the defect.

## Automation

`ldui_audit::audit_page` (via the engine's generic sweep,
`pixelproof-style-audit`) runs that `document.fonts.check` against
`StyleProfile.font_family` once per page and pushes a `family::TYPOGRAPHY`
violation with detail `declared family "<name>" is not loaded — text is
silently falling back` when it fails. `font_family` comes from
`ldui_audit::from_ui_tokens`, so the declared name is never hand-typed at the
call site. Caught by `cargo xtask test-style` (`tests/style_audit_smoke.rs`).

**What this actually covers — established by break-and-revert, not assumed**
(2026-08-09, against office-perf-web; see `ldui-9rf.12`):

| broken how | detected? |
|---|---|
| `@font-face` present, font file 404s | **yes** — all four audited routes failed |
| `@font-face` block deleted entirely | **no** — suite stayed green |

The second row is the vacuous-`check` blind spot described above, and it is
the exact shape op-edag.1 took. Nothing observable in the rendered DOM
distinguishes that page from a healthy one — computed styles are identical —
so no in-page sweep can close it. It needs a **static** check that the
`@font-face` `src`, the font file, and the build's asset-copy directive all
agree; office-perf-web's `font_pipeline.rs` is that check, and an app without
one is uncovered for this row no matter how green its style audit is.

Filed against the engine as **`PixelProof-zbr`** (P1): the sweep should assert
the `@font-face` EXISTS before asking whether it loaded, so the two failure
modes report as two distinct violations with two distinct fixes.
