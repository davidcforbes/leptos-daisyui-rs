# LineChart

`leptos_daisyui_rs::charts::LineChart` renders three surfaces from one prop
set: the legacy numeric XY chart (`Vec<(f64, f64)>`), and the categorical
multi-series chart with its legend, hover/focus card, keyboard activation and
hidden accessible table. An active categorical card also paints a vertical
guide through the plot so points from every series line up visually.

This page documents the **secondary value axis** (`ldui-j0mt`) and the shared
active-category guide (`ldui-ukac`). Everything else is documented on the
component itself.

## Active-category guide

Every interactive categorical chart gets the guide without another prop. It
uses the same active category as the hover/focus card and the same
`category_x` projection as the series markers, so the two surfaces cannot
disagree. Pointer movement selects the nearest category and the guide jumps to
that coordinate; it never tracks a continuously variable pointer position.

Responsive tick thinning affects only which category labels are printed. It
does not remove data increments, so a category whose label is hidden at a
narrow width can still own the card and guide. Pointer leave, blur, Escape,
data reconciliation, or `LineInteractionMode::Disabled` hide the guide under
the existing card-state rules. Keyboard focus shows it as the same visual aid.
The SVG line is decorative (`aria-hidden`) and cannot intercept pointer input.

## Why a second axis

A reporting page that combines counts with a duration — Office's Conversations
Reporting combines three conversation counts with an average first-response
time — cannot read both against one scale. Counts in the hundreds and a
duration in tens of seconds share a domain of `0..N`, and the duration
flatlines along the bottom. The alternative a consumer reaches for is a private
dual-axis SVG, which then owns none of this component's interaction,
reconciliation or accessibility work.

## Assigning an axis

Assignment is per series and typed:

```rust
use leptos_daisyui_rs::charts::{LineAxisOptions, LineSeries, LinePoint, LineValueAxis};

let counts = LineSeries::new("opened", "Opened", "var(--color-primary)", points);
let duration = LineSeries::new("first-response", "Average first response", "var(--color-accent)", points)
    .on_secondary_axis();                       // or .with_axis(LineValueAxis::Secondary)
```

`LineSeries::axis` is `LineValueAxis::Primary` by default, and
`LineSeries::new` sets it explicitly, so **a series that never mentions an axis
behaves exactly as it did before this feature existed** — same domain, same
geometry, same legend text, same table columns, same tooltip text.

> **Source-level note.** `LineSeries` has public fields, so a caller that
> builds one with a *struct literal* rather than `LineSeries::new` must add
> `axis: LineValueAxis::Primary` (or `LineValueAxis::default()`). That is the
> only migration this change requires.

## Axis options

Naming and formatting for each axis is supplied at the chart level:

```rust
view! {
    <LineChart
        data=data
        primary_axis=LineAxisOptions::default().with_label("Conversations")
        secondary_axis=LineAxisOptions::default()
            .with_label("First response")
            .with_unit(" s")
            .with_decimals(1)
    />
}
```

- `label` — the axis title, drawn rotated outside its tick column and used to
  attribute a series in the legend, the accessible names and the hidden table.
- `unit` — appended **verbatim**, so the caller owns the separator (`"%"` and
  `" s"` are both correct for their locale).
- `decimals` — precision. Unset keeps the prior rendering: shortest round-trip
  text for a value, one decimal for a tick.

## One source for a unit

A chart states a number in four places: the tick scale, the hover card, the
hidden table, and the typed `LineChartActivation` payload. Formatting them
separately is how a unit ends up on the ticks but not in the table.

All four call `charts::line_chart::format`'s `value_text` / `tick_text` with
the axis options the series was normalized against, so a unit or a precision is
declared once and reaches all of them. A point's own `display_value` still
wins wherever it is set — that contract is unchanged.

## What is drawn, and when

| Condition | Result |
|---|---|
| No series on the secondary axis | No right axis line, no right ticks, no right gutter, no axis attribution anywhere. The chart is byte-for-byte what it was. |
| A secondary series with only missing values | Same as above — the axis is keyed on a **finite** domain, not on the assignment. |
| A secondary series with finite values | Right axis line and five right ticks inside `<g data-line-chart-y-axis="secondary">`, an optional rotated title, and axis attribution on the legend, the hidden table and the hover card. |
| Every series on the secondary axis | The right axis renders; the left tick scale is omitted (nothing is measured against it). |

Both axes read against the **same five gridline fractions**, so a left tick and
a right tick at the same height are directly comparable.

### Degenerate domains

An axis whose values are all equal (`min == max`) would hand the projection a
zero-height domain and a division by zero. `normalize::domain_for` expands such
a domain symmetrically by `max(|value| * 0.05, 1.0)` **per axis**, explicitly,
rather than letting the projection's `unwrap_or(0.5)` fallback silently flatten
every point of that axis onto the plot's mid-line. Expanding one axis never
disturbs the other.

## Attribution in the non-visual surfaces

The hidden table is the chart's non-visual truth, and a bare column of numbers
is ambiguous once two scales exist. With two axes:

- each series column is headed by `"<series name> (<axis name>)"` — the same
  caption the legend shows, built once;
- each cell carries its axis' unit through the shared formatter;
- `<th>` and `<td>` carry `data-axis="primary"` / `data-axis="secondary"`.

The axis name is the axis' `label` when set, else the corresponding field of
`LineChartTexts` (`primary_axis` / `secondary_axis`). No English is hardcoded
at the fallback — override `texts` to localize it, along with
`category_header` and `no_value`.

With one axis, none of those attributes or captions are emitted at all.

## Stable selectors

Locate axis elements by identity, never by document position:

| Selector | Meaning |
|---|---|
| `[data-line-chart-axes="dual"]` | on the chart root, only when two axes render |
| `[data-line-chart-y-axis="secondary"]` | the right axis group (line plus ticks) |
| `text[data-axis="secondary"]` | a right-hand tick label |
| `[data-line-chart-axis-label="primary"\|"secondary"]` | a rotated axis title |
| `[data-line-chart-legend] [data-axis]` | a legend entry's axis |
| `[data-line-chart-table] th[data-axis]` | a table column's axis |
| `[data-testid="line-chart-tooltip"] [data-axis]` | a card row's axis |
| `[data-line-chart-category-guide]` | the active category's plot-height vertical guide |

## Paint routing

Every new tick label, axis title, axis line and category guide routes its colour through
`charts::paint` (`paint_attrs` / `stroke_attrs`), like every other SVG paint in
the crate. For the `currentColor` these elements use, the routers return the
presentation attribute unchanged, so the DOM is identical — the routing is
what makes a future themed axis colour safe rather than a silent
`fill: black` / `stroke: none`. `tests/svg_paint_routing.rs` enforces it.
