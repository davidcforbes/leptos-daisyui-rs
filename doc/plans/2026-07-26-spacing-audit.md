# Spacing & vertical-rhythm audit — web (leptos-daisyui-rs)

**Issue:** `ldui-lua` (epic `ldui-mai`)
**Date:** 2026-07-26
**Counterpart:** `C:\dev\Rust-DeskApp\docs\plans\2026-07-26-spacing-system-audit.md`

Baseline established before any change. Method and reproduction at the end.

## Headline

The web side is **much cleaner than desktop, and the ticket's suspicion was right**:
Tailwind's 4px-based default scale has been doing the work for us. But the two
faces are *not* sharing a system — they are independently converging on similar
numbers, and where they disagree, they disagree silently.

| Measure | Desktop (audited) | Web (this audit) |
|---|---|---|
| Distinct dimension values | 114 | **23** |
| Off the 4px grid | 21% | **3.2%** (14 of 443) |
| Off the canonical scale (4,8,12,16,24,32,48,64,96) | — | **12.9%** (57 of 443) |
| Component files consuming the shared tokens | 5 of ~105 | **0 of 424** |

Scope: 424 `.rs` files across 112 component directories; 443 measurable spacing
occurrences in live code (doc-comment examples excluded — see Method).

## F1 — Token adoption in components is zero, not low

`rg 'ui_tokens' src/components/` returns exactly one hit, and it is a doc comment
in `card/component.rs:41` referring the reader to `UiTokensPreamble`. **No component
file consumes `ui_tokens` at all.** The desktop finding (5 of ~105) overstates the
web position; the correct number here is 0.

## F2 — The token bridge exists but carries no spacing

`src/tokens/preamble.rs` emits `--ld-*` custom properties on `:root` from
`ui_tokens`, and it is genuinely wired to the crate — a change upstream flows
through without hand-copy. But it emits **only motion and elevation**:

- `--ld-duration-{fast,normal,slow}`
- `--ld-ease-{linear,standard,decelerate,accelerate}`
- `--ld-elevation-{2,4,8,16,64}`

There is **no spacing scale, no line-height ramp, and no typography** in the
emitted CSS. This corrects an assumption in `ldui-d14`, which says to emit the
`LINE_*` ramp *"alongside the existing spacing scale"* — there is no existing
spacing scale on the web side to sit alongside. `ldui-d14` has to add the first
`--ld-space-*` block as well as `--ld-line-*`, or `ldui-1mx` has nothing to map
Tailwind's theme onto.

## F3 — Histogram: 23 distinct values, 96.8% on the 4px grid

| px | Tailwind | uses | 4-grid | canonical |
|---:|---:|---:|:--:|:--:|
| 2 | 0.5 | 9 | **no** | – |
| 4 | 1 | 70 | yes | yes |
| 6 | 1.5 | 3 | **no** | – |
| 8 | 2 | 106 | yes | yes |
| 12 | 3 | 18 | yes | yes |
| 14 | 3.5 | 2 | **no** | – |
| 16 | 4 | 76 | yes | yes |
| 20 | 5 | 18 | yes | – |
| 24 | 6 | 42 | yes | yes |
| 32 | 8 | 25 | yes | yes |
| 40 | 10 | 11 | yes | – |
| 48 | 12 | 31 | yes | yes |
| 64 | 16 | 14 | yes | yes |
| 80 | 20 | 1 | yes | – |
| 96 | 24 | 4 | yes | yes |
| 160, 208, 224, 240, 256, 288, 320, 384 | – | 13 | yes | – |

Plus 7 hairlines (`w-px`/`h-px`/`gap-px`, 1px dividers — legitimate, not spacing)
and 137 non-numeric (`full`, `auto`, `screen`, `fit`, fractions).

**Off-4-grid — the whole list, 14 occurrences in 8 components:**

| px | util | location |
|---:|---|---|
| 6 | `w-1.5` | `data_table/component.rs:139`, `data_table/header.rs:130`, `data_table/server_component.rs:63` |
| 2 | `space-y-0.5` | `login_screen/component.rs:282` |
| 2 | `gap-0.5` | `metric_row/component.rs:38`, `result_list/component.rs:56,189,221` |
| 6 | `h-1.5` | `segmented_bar/component.rs:48,77` |
| 2 | `h-0.5`, `-bottom-0.5` | `toolbar/component.rs:62,291` |
| 6 | `gap-1.5` | `tree/component.rs:280,415` |
| 14, 2 | `w-3.5`/`h-3.5`, `w-0.5`, `mt-0.5` | `vertical_steps/component.rs:56,59,114,119,133` |

Nearly all are sub-4px decorative accents — indicator dots, connector rails,
underline bars, progress-track thickness. **Recommendation: sanction 2px as a
decorative sub-unit rather than migrating these.** The desktop scale already
carries an off-grid 2px (`BADGE_PADDING_V`), so this would be an honest
codification of existing practice on both faces. That leaves only the 6px and
14px uses (10 occurrences, 5 components) as genuine violations to fix.

## F4 — Padding is already perfect; size is where the fragmentation lives

| family | uses | on canonical scale |
|---|---:|---|
| padding | 109 | **100%** |
| margin | 66 | 97% |
| space | 27 | 96% |
| gap | 106 | 92% |
| inset | 23 | 83% |
| **size (w/h/min/max)** | **295** | **75%** |

Rule 3 ("tokens, never raw values") is close to satisfied for *padding* already.
The migration effort in `ldui-1mx` should be aimed almost entirely at the `size`
family — that is where 75 of the 90 off-canonical uses sit.

## F5 — Same role, different number

**Height** — 83 uses, 13 distinct values:

| px | components |
|---:|---|
| 2 | toolbar *(off-grid)* |
| 4 | loading_bar |
| 6 | segmented_bar *(off-grid)* |
| 8 | day_scheduler, loading_bar, week_view |
| 12 | capacity_bar, loading_bar, tree |
| 14 | vertical_steps *(off-grid)* |
| 16 | gantt, icon, input, kanban, loading_bar, tree |
| 20 | base_theme_selector, gantt, icon, kanban *(off-canonical)* |
| 24 | ai_chat, base_theme_selector, icon, icon_tile, nav_rail, theme_export_import, theme_share |
| 32 | icon, icon_tile, preset_themes_gallery |
| 40 | icon_tile *(off-canonical)* |
| 48 | color_customizer, color_picker, icon, icon_tile, nav_rail, upload_file |
| 64 | icon_tile |

`icon` and `icon_tile` alone span 16/20/24/32/40/48/64 — that is a size ramp, not
a disagreement, and it should become a named ramp rather than seven raw utilities.
The real disagreement is **20px vs 24px for the same small-control role**
(base_theme_selector, gantt, icon, kanban all use both).

**Container padding** — 44 uses, 6 values, all canonical: 4 / 8 / 12 / 16 / 24 / 32.
No violations. The 8px and 16px steps carry most of the weight.

**Gap** — 80 uses: 2 *(off-grid)*, 4, 6 *(off-grid)*, 8, 12, 16.

**Stack rhythm (`space-y-*`)** — 26 uses: 2 *(off-grid)*, 8, 12, 16, 24.

## F6 — The two faces disagree on named layout dimensions

This is the finding that matters most for the epic, because these are the values
`ui_tokens` already names and the web side re-derives by hand.

| Role | `ui_tokens` | web | status |
|---|---:|---|---|
| `TITLE_BAR_HEIGHT` / `HEADER_HEIGHT` | 48 | 48, widely used | agree |
| `NAV_RAIL_WIDTH` | 64 | `w-16` = 64 | agree |
| `NAV_ITEM_SIZE` | 48 | `h-12`/`w-12` = 48 | agree |
| `NAV_ICON_SIZE` | 24 | `h-6` = 24 | agree |
| `NAV_ACCENT_WIDTH` | **3** | `w-1` = **4** | **diverge** |
| `RIGHT_PANEL_WIDTH` | **280** | `w-48` = 192 or `w-64` = 256 | **diverge** |
| `TABLE_ROW_HEIGHT` | **40** | not expressed | **absent** |
| `TABLE_HEADER_HEIGHT` | **36** | not expressed | **absent** |
| `CARD_ROW_HEIGHT` | **120** | not expressed | **absent** |
| `CARD_PADDING` / `CARD_GAP` | 16 | daisyUI `card-body` default | absent |
| `STATUS_BAR_HEIGHT` | **28** | not expressed | **absent** |
| `BADGE_PADDING_H` / `_V` | 8 / 2 | daisyUI `badge` default | absent |

The values 28, 36, 120 and 280 appear **nowhere** in the web source. `card`
contains no spacing utilities at all — it inherits daisyUI's `card-body` padding,
which is not on our scale and not under our control. There is no `status_bar`
component; the equivalent is `AppShellStatusBar`
(`app_shell/component.rs:644`), which sets `px-4 gap-2` but **no height** — it is
content-sized, so `STATUS_BAR_HEIGHT = 28` has no web counterpart to diverge from
yet.

**Feed back upstream:** `NAV_ACCENT_WIDTH = 3.0` and `BADGE_PADDING_V = 2.0` are
themselves off the 4px grid *inside the shared token crate*. `ui-tokens`'
`scale_is_strictly_ascending` / `scale_lives_on_the_4px_grid` tests only cover
`SCALE`, not the named layout dimensions, so nothing catches this. Worth a ticket
against Rust-DeskApp.

## Addendum — what changed after the audit

Recorded here because several of the audit's own recommendations did not
survive contact with the implementation.

**The 2px recommendation was withdrawn.** The audit above suggests sanctioning
2px as a decorative sub-unit. On inspecting the 14 *live* off-grid uses (the
count above included doc-comment `@source` lines) they split cleanly into
strokes and genuine gaps rather than forming one decorative class, so the
better answer was to keep the scale pure and give strokes their own family.
Off-grid is now 3 of 443 (0.7%), and all three are stroke-role: the toolbar's
2px active underline and the vertical-steps 2px connector rail. See `ldui-mai.2`.

**F2 understated the problem, in a way that matters for `ldui-1mx`.** The
`ui-tokens` crate had no `SPACE_XXXL`/`SPACE_HUGE` and no stroke family at all —
`ldui-1mx` is written as though they already landed upstream. They did not, and
Rust-DeskApp had no open ticket that would deliver them. They were added
(Rust-DeskApp `532f6ea`), which is what unblocked the rest.

**Two findings that only appear once you compile.** Neither is visible from
reading the Rust source, which is worth remembering next time:

- Emitting the tokens' raw DIPs as `px` into the Tailwind theme is a WCAG 1.4.4
  regression — it pins every font size and gap against the user's browser
  font-size preference. Tailwind ships rem sizes and *unitless* line-height
  ratios for exactly this reason. The generator now converts.
- Adding named `--spacing-*` keys silently redefined `max-w-xs` from 20rem to
  0.5rem — a 40x shrink across four components — because Tailwind resolves
  `max-w-*` against `--spacing-*` before `--container-*`. Caught by diffing the
  compiled stylesheet, not by any test that existed at the time. There is now a
  unit test.

**A daisyUI 4 leftover, unrelated to spacing but blocking `ldui-6qb`.**
`.form-control`, `.label-text` and `.label-text-alt` were removed in daisyUI 5
and appear zero times in `daisyui.css`, yet the repo uses them 206 times (81 in
`src/`, 125 in `demo/src/`). They are inert. `.form-control` supplied the
`display:flex; flex-direction:column` that made a label stack above its input,
so `<label class="form-control w-full">` now falls back to `display:inline` and
`w-full` does nothing. `ldui-6qb`'s form-control rule cannot be checked until
this is fixed — filed as `ldui-mai.3`.

Also worth knowing: daisyUI's own `.fieldset` ships `gap: calc(0.25rem * 1.5)` =
**6px**, off our canonical scale and not under our control. This is the general
shape of the problem the rendered-DOM checker faces, and why its grid check is a
ratchet rather than a zero.

## What the rendered-DOM sweep found

The source audit above measures what we *wrote*. The detector (`ldui-dg2`,
`tests/layout_audit_smoke.rs`) measures what actually *renders*. Final state
across the six audited pages:

| page | scanned | overlap | off-grid | internal>external |
|---|---:|---:|---:|---:|
| button | 71 | 0 | 0 | 0 |
| alert | 39 | 0 | 0 | 0 |
| card | 76 | 0 | 0 | 0 |
| tab | 52 | 0 | 0 | 1 |
| data-table | 1771 | 0 | 2 | 0 |
| kanban | 136 | 0 | 34 | 2 |

**Zero overlaps, and zero off-grid gaps that we own.** Every remaining `grid`
hit is daisyUI's own declared 6px gap (`calc(0.25rem * 1.5)`) inside `.btn`,
`.label` and `.fieldset` — the same 6px that turns up in `.fieldset` above.
It is off our canonical scale and not ours to change without overriding
daisyUI's component internals.

Three false-positive classes had to be removed before those numbers meant
anything, and the third is the cautionary one:

1. daisyUI's `join` collapses shared borders, so joined buttons overlap by
   exactly 1px. That was **100%** of the 24 overlaps first reported.
2. Siblings deliberately flush (`card-body` above `card-actions`) are sections
   of one surface, not competing groups — the proximity rule has nothing to
   disambiguate at a zero gap.
3. **Measuring gaps inside a wrapping flex container is meaningless.** The
   distance from a short item to the next row is the row-gap *plus* the height
   difference to its tallest neighbour. This produced 19 phantom "28px gaps".
   Before the `Section` change it had read 0 — but only because 8 + 16 = 24
   happened to land on the scale. The check was silently wrong from the start
   and arithmetic luck was hiding it. Flex and grid containers are now checked
   by reading their *declared* row/column-gap instead.

That third one is the argument for the negative control that now ships with
the suite: a detector reporting zero is indistinguishable from a detector that
works, until you make it fail on purpose.

## What this means for the rest of the epic

1. **`ldui-d14` grows.** It must add `--ld-space-*` (the first spacing custom
   properties on the web side) in addition to `--ld-line-*`. Its premise that a
   spacing scale already exists is wrong.
2. **`ldui-1mx` shrinks and re-aims.** Padding/margin/gap/space are already
   ~95–100% canonical; there is little to migrate. Point it at the `size` family
   (75% canonical, 295 uses) and at replacing `icon`/`icon_tile`'s seven raw
   sizes with one named ramp.
3. **A new concern, not currently ticketed:** the *named* dimensions diverge
   between faces (F6) while the *scale* agrees. Reconciling 280 vs 256, 40 vs
   nothing, 3 vs 4 is a distinct piece of work from migrating utilities onto a
   scale, and it is the part that actually makes desktop and web look like one
   product.
4. **Sanction 2px** as a decorative sub-unit before `ldui-dg2` builds the
   detector, or the checker will flag 9 legitimate accent rules as violations on
   day one.

## Method

The extractor was a throwaway Python script (session scratchpad,
`audit_spacing.py`). It is deliberately **not** committed: per this repo's
two-layer CI rule (`doc/ci-cd.md`), detector *logic* belongs in the Rust `xtask`
crate, and `ldui-dg2` is the ticket that should port it there rather than
inheriting a stray script. The algorithm below is the spec for that port.

It walks `src/components/**/*.rs`, extracts every
Tailwind spacing/sizing utility appearing inside a string literal, and converts
it to pixels (Tailwind `n` → `n × 4px`; `[13px]` → 13; `px` → 1px hairline).

Two corrections applied during the run, both material:

- **Raw px/rem regex allowed whitespace**, so `bg-base-200 px-4` was read as the
  raw value "200 px". Tightened to reject a space and a following `-`. This cut
  false raw-value hits from 22 to 15.
- **Doc comments were counted.** `///` lines quote class strings both as usage
  examples and as `@source inline(...)` Tailwind safelist hints, mirroring the
  live classes. That inflated counts by 21% (839 → 443 measurable). Doc hits are
  tagged and excluded from every number above.

Genuine raw px/rem in live code — 15 occurrences, 6 files (tests and the
`*_customizer` dev tools, which generate CSS by design, excluded):

```
ai_chat/component.rs:84,516      max-h-[320px]
ai_chat/component.rs:84,615      text-[10px]
data_table/component.rs:93,244   calc(100vh - 260px)
data_table/component.rs:770      padding: 12px 0
data_table/server_component.rs:130,232
gantt/component.rs:392,535       height: calc(100vh - 200px)
gantt/component.rs:477           width: 4px
kanban/column.rs:137             max-height: 500px; min-height: 200px
kanban/types.rs:461              600px
```

The `calc(100vh - Npx)` viewport offsets (260/200) are the interesting ones: they
hardcode an assumed chrome height, which is exactly the `HEADER_HEIGHT` +
`STATUS_BAR_HEIGHT` sum the tokens already name.
