# BarChart

`leptos_daisyui_rs::charts::BarChart` renders two surfaces from one prop set
and one geometry: the preserved positional chart (`Vec<(String, f64)>` plus the
parallel `bar_colors` list), and a typed categorical chart with stable keys,
caller-owned status, reactive copy, an accessible data table and optional
keyboard/pointer activation.

This page documents the signed/diverging work (`ldui-y2ed`). Everything else is
documented on the component itself.

## The defect this replaces

Three things were wrong at once, and a consumer building a signed decomposition
hit all three:

1. **The domain took the maximum and clamped it at zero.** For all-positive
   data that is a correct `0..max` axis. For anything else it is not a domain:
   a negative value divided to a negative fraction and became a negative
   `width`/`height` on a `<rect>` — invalid SVG geometry, drawn silently. An
   all-negative series was worse: `max(...).max(0.0)` collapsed to `0.0`, the
   guard substituted a range of `1.0`, and every bar was drawn at minus its raw
   value in view-box units.
2. **Colour was a second positional vector.** `bar_colors[i]` described
   `data[i]` only by convention, so sorting the rows without sorting the
   colours repainted every bar with a neighbour's judgement — silently, with no
   error and no length mismatch to catch it.
3. **There was no accessible surface at all**: no name, no description, no
   semantic table, no stable identity, no keyboard, and a hard-coded English
   `"No data"` in the empty branch.

The consumer's workaround for (1) — pre-absoluting the values — erases
direction, which for a most-dragging-first ranking is the entire message.

## The typed item

```rust
use leptos_daisyui_rs::charts::{BarChartItem, BarStatus};

let items = vec![
    BarChartItem::new("north", "North", -12.5)
        .with_display_value("-12.5 vs baseline")
        .with_status(BarStatus::Unfavorable),
    BarChartItem::new("central", "Central", 0.0),
    BarChartItem::new("west", "West", 9.5).with_status(BarStatus::Favorable),
    BarChartItem::missing("riverside", "Riverside"),
];
```

Key, label, signed value, formatted display text, judgement and colour are one
value. There is no second array to keep in step, so **no sort, filter or
truncation the caller performs can misalign them** — which is the property the
positional list structurally cannot have.

`bar_colors` is still honoured, and still mismatch-safe exactly as before, but
only for a **neutral** typed item with no colour of its own. Precedence is:
item colour, then status colour, then the positional list, then the chart-wide
`color`.

### Status is the caller's

`BarStatus::Neutral` is the default and paints with the chart-wide colour. The
framework never infers a judgement, because "up" is good for a throughput
measure and bad for a limit, and only the caller knows which this is. An
activity measure therefore passes no status at all and looks exactly as an
unjudged chart always did.

A judged bar also gets an **end cap** on its outward end — solid for favorable,
dashed for unfavorable — so the distinction is not carried by hue alone. Hue is
unavailable in forced-colors mode and to a reader with a colour vision
deficiency; a dash pattern survives both. The judgement is additionally stated
in words in the hidden table and in each bar's accessible name.

## The signed domain and the zero line

`signed_domain` spans the finite values **and always includes zero**:

```text
min = min(finite values).min(0.0)
max = max(finite values).max(0.0)
if max - min <= 0  =>  0..1        // every finite value was exactly zero
```

Every bar is then the *interval* between the zero line and its value, so its
start is whichever comes first and its length is their distance. A negative
value produces a bar on the other side of zero, never a negative dimension.

| Data | Domain | Zero line | Result |
|---|---|---|---|
| All positive | `0..max` | Bottom (vertical) / left (horizontal) | **Byte-identical to the pre-existing chart**, including the baseline it already drew |
| All negative | `min..0` | Top / right | Bars hang from the zero line; lengths positive |
| Mixed signs | `min..max` | Inside the plot at `-min / (max - min)` | Bars diverge both ways |
| A single `0.0`, or all zeroes | `0..1` | Bottom / left | Zero-length bars, the same `1.0` fallback range the original code used — no division by zero, no NaN |
| All equal and nonzero (e.g. `7,7,7`) | `0..7` | Bottom / left | Full-length bars, which is what the chart already did |
| No finite value at all | none | — | The empty branch, with supplied copy |

Because a bar's length is `|value| / (max - min)`, **equal magnitudes of
opposite sign always have equal geometry**, in any domain — not only a
symmetric one. `src/charts/bar_chart/geometry.rs` asserts this as a property
rather than on a fixture.

A NaN or infinite value is normalized to *missing*, not to zero. A missing item
draws no bar (a rect on the baseline would assert the office was exactly on
target), keeps its label and its table row, is stated with the supplied
`no_value` copy, and can never be focused or activated.

## Layout

```rust
use leptos_daisyui_rs::charts::BarChartLayout;
// BarChartLayout::Auto (default) — follows the legacy `horizontal` prop
// BarChartLayout::Vertical / Horizontal
// BarChartLayout::DivergingHorizontal
```

`Auto` is the default and resolves against `horizontal`, so no existing
caller's orientation moves. `DivergingHorizontal` is horizontal bars with the
zero rule **always** drawn — a caller reaches for it precisely because
direction is the message, and a filtering that happens to leave only positive
values must not silently drop the reference every bar is read against.

A vertical chart draws its rule at the zero position always (for all-positive
data that is the baseline it already drew, unmoved). A plain `Horizontal` chart
draws one only when the data actually reaches below zero, so a legacy
horizontal chart gains nothing it did not have.

Value labels sit at each bar's **outward** end — above a positive column and
below a negative one; right of a positive bar and left of a negative one. For
all-positive data that is where they already were. A chart whose domain reaches
below zero reserves extra gutter for the labels that now appear on the far
side; an all-positive chart's plot rectangle is untouched.

## Accessibility

| Surface | Contract |
|---|---|
| Root `<div>` | `role="group"`, reactive `aria-label`, `data-bar-chart-layout`, `data-active-category` |
| `<svg>` | `role="group"` when interactive, `role="img"` when not, `aria-labelledby` a `<title>`/`<desc>` pair |
| Focus targets | One `<rect>` per activatable bar spanning its whole category slot, `role="button"` **only** when `on_bar_activate` is wired, else `role="group"` |
| Tab stops | Exactly one — a roving `tabindex` keyed by identity |
| Keys | Arrow (either axis), Home, End, Escape; Enter/Space activate |
| Table | `sr-only` `<table>` with category, value and status columns, enabled by default |

`role="img"` on the interactive SVG would make every focusable descendant
presentational — an axe blocker (`nested-interactive`), and the reason
`LineChart` moved off it in `ldui-9tr.6`. The legacy positional surface renders
as a bare `<svg>` with no roles, no targets and no table, so it gains no tab
stops at all.

## Activation

```rust
pub struct BarChartActivation {
    pub category_key: String,        // stable identity
    pub category_label: String,
    pub value: f64,                  // always finite
    pub display_value: String,
    pub status: BarStatus,
    pub source: BarChartActivationSource,
    pub modifiers: BarChartModifiers,
}
```

There is deliberately **no index field**. An index re-points at a different
office the moment the caller sorts, filters or replaces the data, so a host
that stored one and acted on it later would act on the wrong row — the identity
rule `ldui-nz6d`/`ldui-px06` established for tables.

Focus is reconciled by key on every data change: a bar keeps focus through a
reorder, and a removed bar hands focus to whatever now occupies its old
position (clamped to the end), without firing any activation.

## One formatter

The drawn value label, the focus target's accessible name, the hidden table
cell and the activation payload all resolve through
`bar_chart::format::displayed_value` with the same `BarValueFormat`, so a unit
or a precision declared once reaches all four. An item's own `display_value`
wins wherever it is set.

`BarValueFormat::default()` means one decimal and no unit, which is the
`format!("{value:.1}")` the chart already emitted.

## Copy

Every string the chart produces itself lives in `BarChartTexts`, supplied as a
`Signal` so a locale change re-renders the words without touching keys, values,
order, focus or selection. The defaults reproduce what the chart already
emitted, including the previously hard-coded `"No data"`.

| Field | Default |
|---|---|
| `empty` | `No data` |
| `category_header` / `value_header` / `status_header` | `Category` / `Value` / `Status` |
| `no_value` | `No value` |
| `status_neutral` / `status_favorable` / `status_unfavorable` | `Neutral` / `Favorable` / `Unfavorable` |

## Migration

None is required. `data` became `#[prop(into)] BarChartDataSource`, and
`Vec<(String, f64)>` converts into it, so every existing call site — including
`bar_colors` and `horizontal` — compiles and renders unchanged. The one
behavioural change to the positional surface is that negative values now draw
correctly instead of producing invalid geometry.

Adopt the typed surface by passing a `Vec<BarChartItem>` (or a
`Signal<BarChartData>`) instead.

## Colour routing

Every paint in this component — bars, status caps, the zero rule, the category
and value labels — goes through `charts::paint`. A `var()` in an SVG
*presentation attribute* is not specified to substitute and would degrade
silently to `fill: black` / `stroke: none`; `tests/svg_paint_routing.rs` scans
all of `src/` and is a gate step.
