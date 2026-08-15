# Interactive Multi-Line Chart - Design

**Date:** 2026-08-14

**Status:** Approved architecture; written specification awaiting review

**Issue:** `ldui-j6k`

**Scope:** `leptos-daisyui-rs` chart component and demo; consumer migration is
specified but remains a separate change in `C:\dev\4iiz-office`

## Context

The shared `LineChart` currently accepts one `Vec<(f64, f64)>` series and
renders a static SVG polyline with optional dots. It has no legend, tooltip,
focus model, or activation callback. The 4Ease office dashboard therefore
implements its own two-line trend chart. That chart already draws a solid
actual series, a dashed rolling-average series, markers, labels, and a legend,
but it is not interactive or keyboard-operable and its final labels clip at
the right edge.

A read-only CDP inspection of the Power BI reference report on 2026-08-14
established the target behavior: multiple patterned lines, node markers, a
category-wide hover card listing every series value, a generous invisible hit
area, and accessible point descriptions. Power BI also separates selection
from report-specific filtering. The shared component should follow that
separation: it emits an activation intent; its consumer decides whether to
filter, navigate, or load detail data.

## Goals

- Render two or more categorical series with independent colors, solid/dashed/
  dotted/custom line patterns, markers, and optional data labels.
- Show one clamped hover/focus card for the active category, containing every
  available series value at that category.
- Make point activation work with pointer click, Enter, and Space, and expose a
  typed payload suitable for consumer-owned drilldown.
- Provide useful names, keyboard navigation, focus indication, and a tabular
  alternative without requiring a mouse.
- Preserve current `LineChart` call sites and their single-series visual output.
- Keep rendering dependency-free: Leptos, SVG, CSS, and existing `web-sys`
  features only.

## Non-goals

- Fetching data, changing routes, applying filters, or rendering drilldown
  content inside the chart.
- Zoom, pan, brushing, secondary axes, area fills, smooth/step interpolation,
  or arbitrary consumer-provided tooltip views in v1.
- Extending the stacked-bar `ChartSeries`; changing that public struct would
  break existing struct literals and mix unrelated chart semantics.
- Reproducing every Power BI interaction. The report is a behavioral and
  visual reference, not an API contract.

## Decision and Alternatives

Build a focused SVG engine inside the existing charts module. It retains the
library's lightweight model, integrates naturally with Leptos signals and
callbacks, and lets geometry, hit testing, and accessibility be tested without
a JavaScript chart runtime.

Two alternatives were rejected. Importing a canvas/JavaScript chart library
would add WASM interop and bundle weight while making DOM and accessibility
oracles weaker. Keeping a chart implementation in each consumer would avoid a
new shared API but perpetuate duplicated geometry, accessibility, clipping,
and interaction defects.

## Public Model and Compatibility

`LineChart` changes its `data` prop to `#[prop(into)] data: LineChartData`.
`From<Vec<(f64, f64)>> for LineChartData` preserves existing Leptos view-macro
call sites. The generated `LineChartProps` type is implementation detail;
callers constructing it directly may need `.into()`.

```rust
pub enum LineChartData {
    XY(Vec<(f64, f64)>),
    Categorical {
        categories: Vec<LineCategory>,
        series: Vec<LineSeries>,
    },
}

pub struct LineCategory {
    pub key: String,
    pub label: String,
}

pub struct LineSeries {
    pub id: String,
    pub name: String,
    pub points: Vec<LinePoint>,
    pub color: String,
    pub pattern: LinePattern,
    pub marker: MarkerStyle,
    pub show_data_labels: bool,
}

pub struct LinePoint {
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub data_label: Option<String>,
    pub marker_color: Option<String>,
}

pub enum LinePattern {
    Solid,
    Dashed,
    Dotted,
    Custom(Vec<f64>),
}

pub enum MarkerShape {
    None,
    Circle,
    Square,
    Diamond,
}

pub struct MarkerStyle {
    pub shape: MarkerShape,
    pub size: f64,
    pub fill: Option<String>,
    pub stroke_width: f64,
}
```

All public types are owned, cloneable, debuggable, and `PartialEq` so they work
cleanly with Leptos props and signals. Builders/defaults should keep ordinary
call sites short. `LinePattern::Custom` accepts only finite positive segments;
an empty or invalid pattern resolves to `Solid` rather than producing invalid
SVG.

Legacy `XY` mode retains numeric x scaling, `color`, `show_dots`, `x_labels`,
axis labels, tick behavior, and `minimal` behavior. Categorical series own
their stroke and marker styling. New `LineLegendMode::{Auto, Always, Never}`
and `LineInteractionMode::{Auto, Enabled, Disabled}` props control optional
behavior. `Auto` shows a legend for two or more categorical series and enables
categorical interaction; it leaves legacy `XY` output unchanged.

## Normalization and Geometry

The component first normalizes either public data variant into internal
categories and series. This boundary isolates compatibility code from the
renderer and provides pure functions for unit testing.

- Categorical x positions are evenly spaced in input order. Category `key` is
  stable identity; `label` is presentation text.
- A short series is padded with missing points. Extra points are ignored. This
  avoids a render panic while making category count authoritative.
- `None`, NaN, and infinite values are missing. Missing values break a path
  into separate segments; the renderer never bridges a gap.
- The y-domain includes every finite visible value. A single-valued domain is
  expanded symmetrically so division by zero is impossible. No usable values
  produce an explicit empty chart state.
- Plot padding is calculated from axes, labels, marker radius, and data-label
  bounds. The first and last category labels use edge-aware anchoring, and the
  tooltip is clamped to the chart container, fixing the dashboard's current
  right-edge clipping.
- Tick density is selected from available plot width and a minimum label gap,
  rather than the current fixed maximum of five. The first and last categories
  remain eligible even when intermediate labels are thinned.

Each line is an SVG `<path>` so missing-value segments and dash patterns are
represented correctly. Visible strokes remain visually sized by the series
style. Interaction uses a separate transparent plot overlay and computed
nearest-point logic, avoiding tiny pointer targets and avoiding duplicate wide
paths in the accessibility tree.

## Interaction and State Flow

The component owns only ephemeral presentation state:

```text
pointer/focus coordinates
        -> active category + preferred series
        -> marker emphasis + tooltip rows
        -> click / Enter / Space
        -> LineChartActivation callback
        -> consumer filter, route, or detail request
```

Pointer coordinates are converted from the element's client rectangle into
SVG view-box coordinates. The nearest category containing at least one finite
point is chosen by x distance. Among that category's finite points, the nearest
y position chooses the preferred series; ties use input series order. State
updates only when the category or preferred series changes, limiting
pointer-move reactivity.

The chart has one tab stop at a time. On entry, the first category with a
finite point becomes active. Left/Right (and Home/End) move between categories;
Up/Down cycle available series at the active category. Moving focus also shows
the same category-wide card used by hover. Enter and Space activate. Escape
dismisses the card until the pointer re-enters or focus moves. Pointer leave
hides the card unless keyboard focus remains inside the chart.

## Tooltip Card

The card is an HTML overlay positioned relative to the chart container, not
an SVG `foreignObject`. This permits normal daisyUI typography and layout,
predictable wrapping, and stable assistive-technology semantics. It contains:

1. the active category label;
2. one row per series with a value at that category;
3. a patterned/color swatch, series name, and host-supplied display value.

The card prefers the upper-right of the active point, flips when it would
overflow, and is finally clamped within the component. It has `role="tooltip"`
and a stable id referenced by the active control's `aria-describedby`. It is
not focusable and contains no actions.

`display_value` is used by the tooltip and accessible text; `data_label` is the
optional text drawn beside a marker. Both let the host preserve
business-specific formatting such as `911`, `72.4%`, or currency. When
`display_value` is absent, the exact fallback is Rust's non-localized
`f64::to_string()`. The component never infers currency, percent, units, or
locale from a raw number.

## Activation Contract

```rust
pub struct LineChartActivation {
    pub category_index: usize,
    pub category_key: String,
    pub category_label: String,
    pub preferred_series_id: Option<String>,
    pub values: Vec<LineChartActivationValue>,
    pub source: LineChartActivationSource,
    pub modifiers: LineChartModifiers,
}

pub struct LineChartActivationValue {
    pub series_id: String,
    pub series_name: String,
    pub value: f64,
    pub display_value: String,
}

pub enum LineChartActivationSource {
    Pointer,
    Keyboard,
}

pub struct LineChartModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
```

`on_point_activate: Option<Callback<LineChartActivation>>` receives owned,
typed intent data rather than a raw browser event. `modifiers` carries
Shift/Ctrl/Alt/Meta booleans for consumers that support additive selection.
Pointer activation reports the geometrically nearest series. Keyboard
activation reports the selected series, falling back to the first finite
series in input order. `values` always contains all finite values at the
category in series order.

This follows the repository's `DataTable` pattern: mouse and keyboard use one
typed callback path. It also makes the behavior portable and testable. The
chart does not interpret modifiers and does not consider callback completion
proof that a drilldown succeeded; the consumer owns that side effect and its
loading/error UI.

## Accessibility

The chart container is a named group with a concise `aria-label`; the SVG has
`<title>` and `<desc>` for its visual summary. Each category interaction target
has `role="button"`, roving `tabindex`, a visible focus indicator, and an
accessible name containing the category and all available series values.

The component also renders a visually hidden data table by default for
categorical data. Its caption uses the chart label, columns are series, and
rows are categories. This is the durable non-visual representation recommended
for complex images; consumers may disable it only when they provide an
equivalent adjacent table. Color and line pattern are redundant series cues,
and focused/hovered markers change size or outline rather than color alone.

The keyboard contract and tooltip behavior follow the WAI guidance that
tooltip content is exposed on hover/focus and dismissed with Escape. Automated
axe checks supplement, but do not replace, keyboard journey tests.

## Failure and Edge Behavior

- Zero categories, zero series, or no finite values renders the axes/container
  plus a named "No chart data" state; no callback fires.
- A category with no values remains addressable in the hidden table but is
  skipped by pointer activation and keyboard navigation.
- Duplicate category keys or series ids do not panic. Development builds emit
  a diagnostic; internal rendering identity includes the input index, callback
  values preserve the supplied identifiers, and reconciliation selects the
  first matching category key.
- Non-finite dimensions, marker sizes, or stroke widths resolve to documented
  safe defaults. Width/height are clamped above zero.
- Removing or replacing data while a category is active reconciles state by
  category key; if the key is gone, the card closes and focus moves to the
  nearest valid category without firing activation.
- Callback code is outside the renderer. A consumer error cannot corrupt chart
  state and must surface through the consumer's own error/trace channel.

## Component Boundaries and File Shape

`src/charts/line_chart.rs` should become an orchestration component rather than
absorbing every concern. Implementation planning should split focused modules
under `src/charts/line_chart/` for public types, normalization/domain math,
SVG geometry, interaction state, and tooltip rendering. `src/charts/mod.rs`
re-exports the intentional public types. Shared paint resolution remains in
`src/charts/paint.rs`.

The demo gains a deterministic multi-line example with at least fourteen
categories, a solid actual series, a dashed average series, markers, missing
data, host-formatted tooltip values, and an activation log visible to the test
harness. The old single-series example remains as the compatibility fixture.

## Consumer Migration

After the library feature lands, `4iiz-office` can replace both current paths:

- `office-perf-web/src/screens/dashboard.rs::TrendView`, the hand-written
  solid/dashed chart;
- `office-perf-web/src/screens/workspace/manager/by_week.rs`, the existing
  shared single-series `LineChart` consumer.

`WeekPointDto` already provides the category and actual/average values, but
the consumer must supply display strings for both series. If the average needs
business-specific formatting, add an `avg12_fmt` field or format it in the
view-model; do not push that rule into the generic chart. The activation
callback maps `category_key` to the report's week filter or route and owns
loading, empty, and error states. Consumer migration and drilldown side-effect
tests are separate follow-up work, not part of the library implementation.

## Verification Strategy

This web/WASM component uses the repository's PixelProof A/B/C/D methodology
and existing browser orchestration.

### Unit and contract tests

- normalization of legacy and categorical data;
- y-domain expansion, categorical positions, tick thinning, and edge anchors;
- missing/non-finite values and segmented paths;
- dash and marker serialization with invalid-style fallbacks;
- pointer-to-view-box conversion and nearest category/series selection;
- tooltip flip/clamp placement;
- activation payload ordering and modifier/source mapping;
- state reconciliation after reactive data replacement.

### Layer A - visual

Add reviewed component-region baselines for default, hovered, keyboard-focus,
missing-data, and narrow-width states. Compare with SSIM at the repository
default threshold; never byte-compare. Rightmost labels, markers, and tooltip
must remain inside the component bounds. Run `cargo make test-visual` and
review every changed baseline before capture.

### Layer B - structure, state, and model

Extend `tests/reactivity_smoke.rs` to assert path/marker/legend counts, dash
attributes, active-category state, tooltip row values, and edge geometry. The
pure interaction-state reducer is the model oracle; DOM attributes and tooltip
content are the rendered oracle. After hover, keyboard movement, and reactive
data replacement, assert the two agree. Run `cargo xtask test-reactivity`.

### Layer C - accessibility

Assert zero new critical/serious axe violations, one roving tab stop, useful
accessible names, the hidden-table relationship, visible focus, arrow/Home/End
navigation, Enter/Space parity, and Escape dismissal. A mouse-only passing path
is insufficient.

### Layer D - behavior and side effects

The demo callback records activation payloads in a deterministic test surface.
Browser tests assert exactly one payload for click, Enter, and Space; verify
category, preferred series, all values, source, and modifiers, while also
failing on console errors or panics (D1). D2 is not applicable to the library:
the component contract forbids database or network writes. A consumer that
uses the callback for drilldown must test its request/filter effect, completion
barrier, and error path in that consumer repo.

### Visual-quality and completion gates

Run from the repository root:

```powershell
cargo test
cargo xtask test-reactivity
cargo xtask test-style
cargo xtask test-layout
cargo make test-visual
cargo xtask verify-full
```

Style and layout ceilings may only ratchet down; a new chart demo page receives
measured initial ceilings rather than borrowing another page's. Confirm the
stylesheet freshness marker before trusting browser results. Every new oracle
must pass a break-and-revert demonstration: inject the defect, observe the
specific failure, revert, and observe green. Baselines come only from the
human-reviewed final rendering.

## Acceptance Criteria

1. Existing `Vec<(f64, f64)>` `LineChart` examples compile and retain their
   current output without source changes.
2. The deterministic demo renders solid and dashed series, distinct markers,
   a legend, data labels, gaps, and unclipped edge content.
3. Hover or focus shows one category-wide card with all finite series values;
   placement remains inside the chart at every edge and narrow viewport.
4. Pointer click, Enter, and Space emit equivalent typed activation payloads;
   no route, request, or filter is performed by the component.
5. Keyboard users can enter, traverse, activate, dismiss, and leave the chart;
   the hidden table exposes the complete categorical dataset.
6. Unit and A/B/C/D tests cover the behavior above, each new oracle has recorded
   break-and-revert evidence, and all applicable repository gates pass.

## Reference Material

- [Power BI line-chart formatting](https://learn.microsoft.com/en-gb/power-bi/visuals/power-bi-line-chart)
- [Power BI report tooltips](https://learn.microsoft.com/en-us/power-bi/create-reports/desktop-tooltips)
- [Power BI filtering and highlighting](https://learn.microsoft.com/en-us/power-bi/create-reports/power-bi-reports-filters-and-highlighting)
- [WAI-ARIA tooltip pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/)
- [W3C complex-image alternatives](https://www.w3.org/WAI/tutorials/images/complex/)
- [W3C Pointer Events](https://www.w3.org/TR/pointerevents/)
