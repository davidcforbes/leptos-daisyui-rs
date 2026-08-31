# KPI strip

`KpiStrip` is the Layer-2 pattern named in `Future-Architecture.md` for a
responsive row of consistently sized stat cards. It exists because daisyUI's
low-level `Stats`/`Stat` pair renders a *joined* strip -- a shared background
and internal dividers -- so eight metrics composed as `Stat` children inside
a `Stats` container read as one table row instead of eight independent
cards. `KpiStrip` builds ordinary bordered, shadowed boxes in a responsive
CSS grid instead. `Stats`/`Stat` are unchanged and remain independently
usable -- reach for them directly when daisyUI's own joined presentation is
actually what's wanted.

## When to reach for this vs `Stats`/`Stat`

| Need | Use |
|---|---|
| A row of visually independent stat cards (dashboard summary, workspace header) | `KpiStrip` + `KpiItem` |
| daisyUI's own joined stats strip (a single card with internal dividers) | `Stats` + `Stat` directly |
| One featured metric, or an unusual layout `KpiStrip`'s grid doesn't fit | `KpiCard` directly |

## The typed item model

`KpiStrip` takes `items: Signal<Vec<KpiItem>>`. `KpiItem` is plain owned
data, not a `Signal`-bearing struct -- the whole list is itself reactive, the
same posture as `ActiveFilterChip`/`DatasetOption` elsewhere in this crate.
Rebuilding the list (a data refresh, a locale change) is how a `KpiStrip`
updates; there is no per-field signal to thread through.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::StatDeltaTrend;
use leptos_daisyui_rs::patterns::{KpiItem, KpiStatus, KpiStrip, KpiTrend};

let items = Signal::derive(|| {
    vec![
        KpiItem::new("open", "Open matters", "128")
            .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
        KpiItem::new("overdue", "Overdue tasks", "6")
            .status(KpiStatus::Warning)
            .help("Tasks past their due date, across every assignee."),
        KpiItem::new("revenue", "Revenue booked", "$18,400")
            .status(KpiStatus::Success),
        KpiItem::new("sync", "Last sync", "").unavailable(),
    ]
});

view! { <KpiStrip items=items /> }
```

Builder methods on `KpiItem`:

- `KpiItem::new(id, label, value)` -- an available card.
- `.unavailable()` -- clears the value; the card renders a muted placeholder
  (never a fabricated `0` or empty string) and the framework-owned
  `unavailable` text from `KpiStripTexts`.
- `.description(text)` -- optional supporting copy; renders nothing when
  empty.
- `.status(KpiStatus)` -- optional semantic emphasis (`Info`/`Success`/
  `Warning`/`Error`). Drives the value text color *and* a top accent
  stripe together, so status is never color-only; `Neutral` (the default)
  renders no stripe.
- `.trend(KpiTrend)` -- optional up/down/steady indicator, reusing
  `StatDeltaTrend` so `KpiStrip` and `StatDelta` agree on what "positive"
  means for a given metric. `KpiTrend::new(value, direction)` plus an
  optional `.label(text)`.
- `.help(text)` -- optional help text. Renders a small "?" affordance with a
  hover tooltip, and is also wired to the card's `aria-describedby` so it
  reaches assistive tech without requiring a hover.

## Grid and `compact`

`KpiStrip` owns the responsive grid: two columns at the narrowest width,
growing through three and four columns to a full eight-column row at `xl`.
Fewer than eight items simply leave the remaining explicit-column tracks
empty -- CSS Grid does not stretch existing cards to fill them, so card size
stays equal regardless of count. The strip never scrolls horizontally.

`compact=true` tightens card padding and gap (`p-4`/`gap-4` down to
`p-3`/`gap-3`, both still on the canonical spacing scale) and steps the
value text down one rung on the `.ld-text-*` ramp (`ld-text-display` to
`ld-text-title`), for dense contexts such as a sidebar summary or an
embedded card.

## Accessibility

Each `KpiCard` is `role="group"` with a computed `aria-label` combining the
label, the value (or the unavailable fallback), and the trend direction as a
word ("trending up"/"trending down"/"steady") rather than only a glyph --
so a screen reader announces one coherent phrase per card instead of reading
unrelated child text nodes. Help text is exposed via `aria-describedby`
independent of hover.

## Localization

Caller-supplied text (label/value/description/help) localizes by rebuilding
the `items` list for the active locale, like any other reactive prop in this
crate. The pattern's own generated copy -- the unavailable-value fallback and
the trend words folded into the accessible name -- is a separate,
overridable `texts: Signal<KpiStripTexts>` prop, forwarded from `KpiStrip` to
every card:

```rust,no_run
use leptos_daisyui_rs::patterns::KpiStripTexts;

let texts = KpiStripTexts {
    unavailable: "Indisponible".to_string(),
    trend_up: "en hausse".to_string(),
    trend_down: "en baisse".to_string(),
    trend_steady: "stable".to_string(),
};
```

## Section heading and period selection stay caller-owned

`KpiStrip` renders only the cards. A section heading above it (title,
period-selector, refresh action) is ordinary composition -- typically
`SectionHeading` from this same module -- not a `KpiStrip` prop:

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{KpiStrip, SectionHeading};
# use leptos_daisyui_rs::patterns::KpiItem;
# fn items() -> Vec<KpiItem> { vec![] }

view! {
    <section>
        <SectionHeading title="Work stats" />
        <KpiStrip items=Signal::derive(items) class="mt-4" />
    </section>
}
```

## Add to `input.css`

```css
@source inline("grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-8 gap-3 gap-4");
@source inline("rounded-box border border-base-300 bg-base-100 shadow-sm h-full min-w-0 overflow-hidden");
@source inline("forced-colors:border-[CanvasText]");
@source inline("h-(--border-width-accent) w-full");
@source inline("bg-info bg-success bg-warning bg-error");
@source inline("flex flex-col items-center gap-1 gap-2 p-3 p-4 min-w-0 shrink-0 truncate");
@source inline("font-semibold uppercase tracking-wide tabular-nums break-words italic");
@source inline("text-base-content/75 text-base-content/40 text-base-content/60 text-info text-success text-warning text-error");
@source inline("tooltip tooltip-top inline-flex h-4 w-4 items-center justify-center rounded-full border sr-only");
```

The `.ld-text-*` classes are **not** listed above and must not be added via
`@source inline`: they are not Tailwind utilities (`@source` scanning cannot
generate them), they are plain rules generated into `styles/tokens.css` by
`cargo xtask gen-tokens` (ldui-h7tw). Import that file once, as shown under
[CSS Configuration](../../CLAUDE.md#css-configuration), and the ramp resolves
with no further action -- unlike most of this crate's `--ld-*` custom
properties (durations, easings, elevation, `.ld-eased`/`.ld-focus-ring`/etc.),
which still require mounting `UiTokensPreamble`/`UiAnimationsPreamble` at
runtime. See `src/tokens/preamble.rs` for which family is which.

## Reference

Demo: `/components/kpi_strip`. Source: `src/patterns/kpi_strip.rs`.
