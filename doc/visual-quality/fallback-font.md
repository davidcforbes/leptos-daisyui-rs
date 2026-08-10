# Silent font fallback

**Status:** automated (typography, since 2026-08-09) — both a failed load and a
missing `@font-face` are caught, **but only if the profile pins a real family
name; a CSS generic covers nothing.** See the caveat under Automation.
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
`pixelproof-style-audit`) checks `StyleProfile.font_family` once per page and
pushes a `family::TYPOGRAPHY` violation when the family is not painting.
`font_family` comes from `ldui_audit::from_ui_tokens`, so the declared name is
never hand-typed at the call site. Caught by `cargo xtask test-style`
(`tests/style_audit_smoke.rs`).

**Both failure modes are now covered, and they report differently** — since
`PixelProof-zbr` (engine commit `9b5020a`, 2026-08-09). The sweep no longer
asks `document.fonts.check`; it **measures** whether the declared family
actually paints, comparing canvas text metrics against two generics:

| broken how | detected? | detail it reports |
|---|---|---|
| `@font-face` present, font file 404s | **yes** | `has an @font-face rule but never loaded (src failed?)` |
| `@font-face` block deleted entirely | **yes** | `does not resolve — no @font-face rule and no system font by that name` |

Two failure modes, two fixes, so two distinct details. Both are pinned by
browser negative controls in the engine's own suite
(`typography_catches_a_declared_family_with_no_font_face_rule`), and a real
system font that resolves without any `@font-face` stays green
(`typography_accepts_a_resolving_system_font`).

> **The row above was `no` until 2026-08-09.** It was established as a genuine
> blind spot by break-and-revert against office-perf-web (`ldui-9rf.12`) —
> deleting the `@font-face` block left the suite green, because
> `document.fonts.check` answers "are all *matching* faces loaded?" and returns
> `true` vacuously when none match. That finding is what produced the engine
> fix. Kept here because the reasoning is what makes the current coverage
> trustworthy, not because the gap remains.

### ⚠ The caveat that decides whether any of this applies to you

**A CSS generic family is exempt from the resolution probe, so pinning one
covers nothing.** `ui-sans-serif`, `system-ui`, `sans-serif` and the other
generic keywords always resolve by definition, and the probe cannot say
anything useful about them (`PixelProof-1mf`).

This matters more than it sounds: **Tailwind's default font stack begins with
`ui-sans-serif`**, and the documented way to pin a profile —
`body_font_family()`, taking the first family from the computed body stack —
therefore yields a generic for any app that has not set its own font. Such an
app gets no font-fallback coverage at all, and the audit will not tell it so.
This repo's own demo is exactly that case.

**If your app ships a web font, pin that family by name rather than whatever
`body_font_family()` returns.** Otherwise this entry's automation is inert for
you and you are relying entirely on the static check below.

A **static** check that the `@font-face` `src`, the font file, and the build's
asset-copy directive all agree is still worth having — it fails at build time
rather than needing a browser, and it catches the copy-step disagreement that
produced op-edag.1 before anything renders. office-perf-web's
`font_pipeline.rs` is that check. Keep it; the in-page sweep is now a second
line of defence rather than the only one.
