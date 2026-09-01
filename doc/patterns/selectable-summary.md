# Selectable Summary (`ldui-l5cw`)

`SelectableSummaryGroup` + `SelectableSummaryCard` — an opinionated,
single-selection group of compact count cards. It is the widget a diagnostic
page (Office Data Quality and its siblings) needs above its detail table: a
dense grid of "check → count" cards where exactly one is chosen, and the
chosen one drives what the rest of the page shows.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    SelectableSummaryGroup, SelectableSummaryItem, SelectableSummaryStatus,
};

let (selected, set_selected) = signal(Some("duplicates".to_string()));
let items = Signal::derive(|| vec![
    SelectableSummaryItem::new("duplicates", "Duplicate records", 12)
        .status(SelectableSummaryStatus::Warning)
        .description("Same identifier on more than one row"),
    SelectableSummaryItem::new("orphans", "Orphaned rows", 0)
        .status(SelectableSummaryStatus::Clean),
    SelectableSummaryItem::unmeasured("freshness", "Feed freshness"),
]);

view! {
    <SelectableSummaryGroup
        label="Data quality checks"
        items=items
        selected=selected
        on_select=Callback::new(move |id| set_selected.set(Some(id)))
    />
}
```

Showcase: `/components/selectable_summary`.

## What it owns, and what it refuses to own

Owns: the accessible group name, `role="radiogroup"`/`role="radio"`
semantics, the roving tab stop, the full arrow-key contract, the
container-query grid, equal card geometry, the selected treatment (including
forced-colors), the status channel, and the unmeasured-vs-zero distinction.

Refuses: fetching, selection state, headings, toolbars, detail panels, and
domain vocabulary. It is a selection pattern, **not a page generator**, and
it names no check.

## Why `role="radiogroup"`, not `aria-pressed`

Single selection among buttons has two legitimate encodings with *different*
keyboard contracts, so the choice has to be made once and implemented
completely — half a radiogroup is worse than a correct set of toggle
buttons.

| | `aria-pressed` toggles | `role="radiogroup"` (chosen) |
|---|---|---|
| Tab stops for 14 cards | 14 | **1** |
| "Exactly one of these" expressed | no | yes |
| Arrow keys | not expected | move **and** select |
| Announced as | 14 unrelated toggles, all off | a named group of 14 radios, one checked |

Fourteen cards decides it. Fourteen tab stops in front of the table the page
exists to show is a real cost to a keyboard user, and fourteen independent
toggles is a lie about the widget.

### The keyboard contract, in full

- <kbd>Tab</kbd> enters the group at **one** stop — the selected card, or
  the first selectable card when nothing is selected. The next <kbd>Tab</kbd>
  leaves the group entirely.
- <kbd>ArrowRight</kbd>/<kbd>ArrowDown</kbd> → next selectable card, focused
  **and** selected. <kbd>ArrowLeft</kbd>/<kbd>ArrowUp</kbd> → previous. Both
  wrap. Both axes move, because a wrapped grid has no single reading
  direction and the APG contract is defined over option *order*.
- <kbd>Home</kbd> → first selectable card, <kbd>End</kbd> → last.
- <kbd>Space</kbd>/<kbd>Enter</kbd> select the focused card (native
  `<button>` activation — the pattern does not intercept them, which would
  double-fire).
- Disabled cards are skipped by every rule above, never focused-and-refused.
- Any other key keeps its default: `preventDefault` runs only *after* a step
  resolves.

## Controlled, never optimistic

`selected` is read and never written; `on_select` is a *proposal*. Arrow keys
move focus immediately and emit the proposal, so a caller that declines it
leaves `aria-checked` where it was while focus has moved. That is the honest
controlled behaviour — accept the proposal to keep the two together.

`on_select` also fires when the already-selected card is activated again, so
a consumer can treat a repeat press as a refresh; ignore it if not wanted.

## Unmeasured is not zero

A card reading `0` when it means "we could not measure this" is a lie, and it
is the easiest defect to ship. The two constructors make it unspellable:

| | constructor | rendered count | accessible name |
|---|---|---|---|
| measured zero | `SelectableSummaryItem::new(id, label, 0)` | `0` (tabular) | `"Orphaned rows: 0, clean"` |
| no measurement | `SelectableSummaryItem::unmeasured(id, label)` | `"Not measured"` (italic, muted) | `"Feed freshness: Not measured"` |

`count` is `Option<u64>`, so there is no `0` to accidentally mean "unknown".
`unmeasured` defaults the status to `Unavailable`. The presentational
`count_text` override (for locale-grouped digits) is ignored entirely when
there is no count, so it can never invent a value.

`Unavailable` also *is* the spoken status word, so the name says it once, not
twice.

## Status is never colour-only

Three channels, in order of durability:

1. **Count and label** — always present.
2. **Glyph shape** — `circle-check` / `triangle-alert` / `circle-alert` /
   `help-circle`, one per non-neutral status, each pinned by a test to
   resolve in the shipped sprite (an unknown Lucide name degrades silently to
   `blank`, which would reduce status to colour alone).
3. **Colour** — the left accent edge and the count's text colour. Removable.

Plus a spoken status word folded into the card's accessible name.

## The left accent edge matches `KpiCard`

Same convention, deliberately: a **left** edge, always laid out (so a status
card and a neutral card share one text alignment), `Neutral` painting the
house `bg-status-blue` as the *default* rather than nothing, and
`forced-colors:bg-[CanvasText]` so the edge survives forced-colors mode. Two
card families in one library disagreeing about where the accent lives is the
drift this pattern exists to prevent.

## Selection survives forced-colors mode

Selected cards get `ring-2 ring-primary` (a shape that is simply *absent*
when unselected, not a hue swap) plus `border-primary`. Forced-colors mode
drops box-shadows, so the selected card additionally claims the system
`Highlight` border colour while an unselected one claims `CanvasText` — two
distinct system colours. Border **width** is identical in both states, so
selecting a card never reflows the grid. Disabled adds `border-dashed` —
again a shape.

## Geometry

- `@container` + `@sm`/`@lg`/`@3xl`/`@5xl`, **never** `sm:`/`md:`/`xl:`. The
  column count follows the *group's* width; fourteen cards in a constrained
  column is exactly the situation where viewport breakpoints render
  unreadable cards (`ldui-tnyq`). 2 → 3 → 4 → 5 → 7 columns, so fourteen
  cards land as two even rows at the widest step.
- `gap-3` between cards, `p-3` inside one: internal ≤ external, or the cards
  read as a single group.
- The label clamps to two lines and *reserves* both (`min-h-8` = two
  `ld-text-small` line boxes), so every card's count starts at the same
  vertical offset regardless of label length — same numbers as `KpiCard`, so
  the two families line up when they share a page.
- Muted copy is `text-base-content/75`, never `opacity-*` (the style audit
  fails `opacity-60`/`50` text for contrast).

## Stable hooks for tests

Locate by attribute, never by position:

| attribute | on |
|---|---|
| `data-selectable-summary-group="true"` | the radiogroup |
| `data-selectable-summary-card="<id>"` | each card button |
| `data-selectable-summary-status="<status>"` | each card button |
| `data-selectable-summary-measured="true\|false"` | each card button |
| `data-selectable-summary-count="true"` | the count span |

## `input.css`

See the `@source inline(...)` blocks on `SelectableSummaryCard` and
`SelectableSummaryGroup` in the rustdoc. The `ld-text-*` steps are **not**
listed there: they are authored rules emitted into `styles/tokens.css`, not
Tailwind utilities, so `@source inline(...)` cannot generate them
(`ldui-fg2h`). Import that stylesheet instead.
