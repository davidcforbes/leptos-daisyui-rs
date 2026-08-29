# Interactive Multi-Line LineChart Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use
> `superpowers:executing-plans` to execute this plan, and use
> `superpowers:test-driven-development` plus
> `superpowers:verification-before-completion` for each implementation slice.
> Track progress in beads under `ldui-9tr`; the steps are numbered because this
> repository prohibits Markdown task lists.

**Goal:** Upgrade `LineChart` with a source-compatible categorical multi-series
mode that renders patterned SVG lines and markers, exposes a category-wide
hover/focus card, and emits keyboard-accessible typed activation intent.

**Architecture:** Keep the existing XY renderer as the compatibility branch.
Add an owned public data model and an explicit static/reactive source adapter,
then normalize categorical data into focused pure geometry and interaction
modules. Rust/Leptos compiled to `wasm32-unknown-unknown` owns SVG/HTML DOM,
measurement, hit testing, tooltip state, and events. Test-only CDP expressions
and vendored axe-core remain in the Rust browser harness and never enter the
application bundle.

**Tech stack:** Rust 2024, Leptos 0.8 CSR/WASM, `web-sys`/`js-sys`, SVG and HTML,
daisyUI 5/Tailwind CSS, native Rust unit tests, PixelProof CDP/SSIM, axe-core
4.13.0, cargo-make, and the existing `cargo xtask` gates.

**Specification:**
[`docs/superpowers/specs/2026-08-14-interactive-multi-line-chart-design.md`](../specs/2026-08-14-interactive-multi-line-chart-design.md)

**Tracking:** `ldui-9tr` is the implementation epic. Tasks `ldui-9tr.1` through
`ldui-9tr.8` correspond one-for-one with the numbered tasks below.

## Global Constraints

1. Run commands from `C:\dev\leptos-daisyui-rs` in PowerShell. Before each
   commit, inspect `git status --short` and stage only the paths named by that
   task. The current worktree contains unrelated user changes and line-ending
   noise; never stash, reset, restore, or reformat those files wholesale.
2. Before asserting that a file or feature is absent, compare the working base
   with `main`:

   ```powershell
   git rev-parse --short HEAD
   git rev-parse --short main
   git log --oneline HEAD..main
   git diff --name-only HEAD..main -- src/charts demo/src/demos/charts.rs tests
   ```

3. Every behavior starts with a failing focused test. After it passes, inject
   one deliberate defect, observe the intended oracle fail, revert that defect,
   and record the command and failure in the matching bead with
   `bd update <id> --append-notes '...' --json`.
4. Preserve unchanged legacy view-macro calls such as
   `<LineChart data=weekly_series() ... />`. Keep legacy XY visual behavior,
   `minimal`, `x_labels`, `tick_anchor`, axes, labels, and paint routing intact.
5. Route all dynamic SVG `fill` and `stroke` values through
   `src/charts/paint.rs`; a `var(--color-*)` token must never be written directly
   to an SVG presentation attribute.
6. Use only on-ramp 11, 12, and 14 pixel typography in the new categorical
   renderer. Use the repository's declared radius/shadow/button vocabulary;
   do not absorb a new visual-quality finding into a ceiling without source
   evidence.
7. Do not add `leptos-chartistry`, Plotters, D3, Chart.js, ECharts, or another
   chart runtime. Do not write application JavaScript. `web-sys` and `js-sys`
   calls authored in Rust are the intended WASM/browser boundary.
8. Do not edit `C:\dev\4iiz-office`. Document its migration shape only; that
   repository owns drilldown requests, filtering, completion, and errors.

## Chartistry Source-Shape Comparison

The comparison uses the current
[Chartistry source tree](https://github.com/feral-dot-io/leptos-chartistry/tree/master/leptos-chartistry/src)
as architectural research, not copied code or a compatibility target.

| Concern | Chartistry shape | Decision here |
|---|---|---|
| Root orchestration | `chart.rs` measures the node, builds state/projection, and composes SVG plus an HTML overlay | **Adopt:** keep `line_chart.rs` as the owner of measurement, signals, composition, and legacy dispatch. |
| Series | `series/` uses typed extractor closures and separate line/bar descriptors | **Adapt:** use owned `LineSeries` and `LinePoint` descriptors because callers already possess categorical labels, display strings, stable keys, and styles. |
| Projection | `projection.rs` isolates data-range-to-SVG conversion | **Adopt:** make `Projection` and plot bounds pure, deterministic values with native tests. |
| Layout | `layout/` offers generic edge and inner plugins | **Reject for v1:** focused axes, legend, markers, and hit overlay are sufficient; a plugin framework would exceed this component's approved scope. |
| State and guides | `state.rs` tracks mouse positions and guide hover | **Adapt:** track category key, category index, preferred series, focus, and dismissal, with one reducer shared by pointer and keyboard paths. |
| Tooltip | `overlay/` renders an HTML tooltip above SVG | **Adopt and extend:** render one measured HTML card, but aggregate every finite series value and support focus, clamping, and `aria-describedby`. |
| Tick abstraction | Chartistry's typed ticks are designed around numeric/time axes | **Reject for categorical identity:** category keys and host display labels remain separate owned strings; no public tick trait is introduced. |

## File Map

| Path | Responsibility |
|---|---|
| `src/charts/line_chart.rs` | Component props, source dispatch, ResizeObserver lifecycle, categorical composition, and the preserved legacy XY renderer |
| `src/charts/line_chart/types.rs` | Public data, styling, source, activation, legend, and interaction types |
| `src/charts/line_chart/normalize.rs` | Compatibility/categorical normalization, diagnostics, formatted values, and finite-value policy |
| `src/charts/line_chart/geometry.rs` | Plot bounds, domain, projection, paths, ticks, hit testing, markers, and tooltip placement |
| `src/charts/line_chart/interaction.rs` | Pure reducer, reconciliation, activation payload construction, modifier mapping, and SVG focus helper |
| `src/charts/line_chart/tooltip.rs` | Tooltip view model and HTML card rendering |
| `src/charts/mod.rs` | Intentional public re-exports |
| `demo/src/demos/charts.rs` | Existing compatibility examples plus deterministic interactive categorical fixture |
| `tests/common/mod.rs` | Real CDP pointer-position helper and browser error/panic capture |
| `tests/reactivity_smoke.rs` | DOM, state, keyboard, activation, reconciliation, D1, and axe oracles |
| `tests/visual_smoke.rs` | Component-region SSIM states and containment assertions |
| `tests/style_audit_smoke.rs` | Measured charts-page typography/shape/depth/grid/internal ratchet |
| `tests/layout_audit_smoke.rs` | Measured charts-page overlap/grid/internal ratchet |
| `tests/vendor/axe-core/` | Test-only pinned `axe.min.js`, MPL-2.0 license, version and hash record |
| `tests/visual/baselines/charts/interactive-line-chart/` | Reviewed component-region PNG baselines |

## Task 1: Define the Public Model and Compatibility Source

**Bead:** `ldui-9tr.1`

**Files:** Create `src/charts/line_chart/types.rs`; modify
`src/charts/line_chart.rs`, `src/charts/mod.rs`, and the unit-test module under
`types.rs`.

1. Claim the task and add failing tests for legacy/static/reactive conversion,
   marker defaults, and builder output:

   ```powershell
   bd update ldui-9tr.1 --claim --json
   cargo test --lib charts::line_chart::types
   ```

   The first run must fail because the model and module do not exist. Pin this
   public shape in the tests:

   ```rust
   #[test]
   fn legacy_vec_becomes_static_xy_data() {
       let source = LineChartDataSource::from(vec![(0.0, 2.0), (1.0, 4.0)]);
       assert_eq!(
           source.get(),
           LineChartData::XY(vec![(0.0, 2.0), (1.0, 4.0)])
       );
   }

   ```

2. Implement and rustdoc the exact public types approved by the specification:

   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub enum LineChartData {
       XY(Vec<(f64, f64)>),
       Categorical {
           categories: Vec<LineCategory>,
           series: Vec<LineSeries>,
       },
   }

   #[derive(Clone, Debug, PartialEq)]
   pub struct LineCategory {
       pub key: String,
       pub label: String,
   }

   #[derive(Clone, Debug, PartialEq)]
   pub struct LineSeries {
       pub id: String,
       pub name: String,
       pub points: Vec<LinePoint>,
       pub color: String,
       pub pattern: LinePattern,
       pub marker: MarkerStyle,
       pub show_data_labels: bool,
   }

   #[derive(Clone, Debug, PartialEq)]
   pub struct LinePoint {
       pub value: Option<f64>,
       pub display_value: Option<String>,
       pub data_label: Option<String>,
       pub marker_color: Option<String>,
   }
   ```

   Add `LinePattern::{Solid, Dashed, Dotted, Custom(Vec<f64>)}`,
   `MarkerShape::{None, Circle, Square, Diamond}`, `MarkerStyle`,
   `LineLegendMode::{Auto, Always, Never}`,
   `LineInteractionMode::{Auto, Enabled, Disabled}`, and the activation types
   from the specification. Derive `Clone`, `Debug`, and `PartialEq` everywhere;
   use `Copy`, `Eq`, and `Default` on fieldless enums where valid. Define marker
   `size` as SVG-viewBox radius units, with a finite default of `4.0`.

3. Add concise constructors so a consumer does not need verbose struct literals:

   ```rust
   impl LinePoint {
       pub fn new(value: f64) -> Self;
       pub fn missing() -> Self;
       pub fn with_display_value(self, value: impl Into<String>) -> Self;
       pub fn with_data_label(self, label: impl Into<String>) -> Self;
   }

   impl LineSeries {
       pub fn new(
           id: impl Into<String>,
           name: impl Into<String>,
           color: impl Into<String>,
           points: Vec<LinePoint>,
       ) -> Self;
   }

   impl LineChartData {
       pub fn categorical(
           categories: Vec<LineCategory>,
           series: Vec<LineSeries>,
       ) -> Self;
   }
   ```

4. Implement the conversion boundary without relying on chained `Into`:

   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub struct LineChartDataSource(LineChartDataSourceInner);

   #[derive(Clone, Debug, PartialEq)]
   enum LineChartDataSourceInner {
       Static(LineChartData),
       Reactive(Signal<LineChartData>),
   }

   impl LineChartDataSource {
       pub(crate) fn get(&self) -> LineChartData {
           match &self.0 {
               LineChartDataSourceInner::Static(data) => data.clone(),
               LineChartDataSourceInner::Reactive(data) => data.get(),
           }
       }
   }
   ```

   Implement `From<Vec<(f64, f64)>>`, `From<LineChartData>`,
   `From<Signal<LineChartData>>`, and `From<RwSignal<LineChartData>>`. Keep the
   inner enum private so source transport is not another consumer API.

5. Change the component prop to `#[prop(into)] data: LineChartDataSource` and
   add these props with documented defaults:

   ```rust
   #[prop(default = LineLegendMode::Auto)] legend_mode: LineLegendMode,
   #[prop(default = LineInteractionMode::Auto)] interaction_mode: LineInteractionMode,
   #[prop(default = "Line chart".to_string())] accessible_label: String,
   #[prop(optional)] description: Option<String>,
   #[prop(default = true)] show_data_table: bool,
   #[prop(optional)] on_point_activate: Option<Callback<LineChartActivation>>,
   ```

   Move the current body into a private `render_xy` helper with the same props
   and preserve `pub(super) fn tick_anchor`. Dispatch through a `Memo`/reactive
   closure so a `Signal` can switch variants and replace data without remounting
   the consuming component. Do not add a wrapper or new categorical styles to
   the legacy SVG branch.

6. Re-export all intentional public types from `src/charts/mod.rs`, then prove
   the unchanged demo expressions compile:

   ```powershell
   cargo test --lib charts::line_chart
   cargo check -p leptos-daisyui-showcase
   cargo test --doc
   ```

7. Perform the negative control by temporarily removing
   `From<Vec<(f64, f64)>>`, observe `cargo check -p leptos-daisyui-showcase`
   fail at the existing two `LineChart` calls, restore it, and rerun green.
   Format only the touched Rust files, stage exact paths, inspect the staged
   diff, commit, close the bead, and push its Dolt state:

   ```powershell
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/types.rs src/charts/mod.rs
   git add src/charts/line_chart.rs src/charts/line_chart/types.rs src/charts/mod.rs
   git diff --cached --check
   git commit -m "feat(charts): add multi-line data model (ldui-9tr.1)"
   bd close ldui-9tr.1 --reason "Public model and compatibility source implemented and verified" --json
   bd dolt push
   ```

## Task 2: Build Normalization, Projection, and Geometry

**Bead:** `ldui-9tr.2`

**Files:** Create `src/charts/line_chart/normalize.rs` and
`src/charts/line_chart/geometry.rs`; modify `src/charts/line_chart.rs` only to
declare the modules.

1. Claim the task. Write failing pure tests before implementations for all
   approved edge policies: short-series padding, extra-point truncation,
   `None`/NaN/infinity gaps, duplicate identifiers, all-missing data, singleton
   domains, width/height clamping, edge ticks, segmented paths, dash fallback,
   nearest category/series, and all four tooltip edges.

   ```powershell
   bd update ldui-9tr.2 --claim --json
   cargo test --lib charts::line_chart::normalize
   cargo test --lib charts::line_chart::geometry
   ```

2. Normalize once at the source boundary into private values that never require
   defensive indexing during render:

   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub(super) struct NormalizedChart {
       pub categories: Vec<NormalizedCategory>,
       pub series: Vec<NormalizedSeries>,
       pub domain: Option<Domain>,
   }

   #[derive(Clone, Debug, PartialEq)]
   pub(super) struct NormalizedPoint {
       pub value: Option<f64>,
       pub display_value: Option<String>,
       pub data_label: Option<String>,
       pub marker_color: Option<String>,
   }

   pub(super) fn normalize_categorical(
       categories: &[LineCategory],
       series: &[LineSeries],
   ) -> NormalizedChart;
   ```

   Use category count as authoritative. Pad missing positions with
   `NormalizedPoint { value: None, ... }`, ignore surplus points, and convert
   every non-finite value to `None`. Preserve supplied keys/ids in callbacks,
   but append input indices to internal DOM identity. In debug builds, emit one
   `web_sys::console::warn_1` per duplicate key/id set; never panic.

3. Make projection data-only and usable by native tests:

   ```rust
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub(super) struct PlotBounds {
       pub left: f64,
       pub top: f64,
       pub right: f64,
       pub bottom: f64,
   }

   #[derive(Clone, Copy, Debug, PartialEq)]
   pub(super) struct Projection {
       pub bounds: PlotBounds,
       pub category_count: usize,
       pub domain: Domain,
   }

   impl Projection {
       pub fn category_x(&self, index: usize) -> f64;
       pub fn value_y(&self, value: f64) -> f64;
       pub fn category_at_x(&self, x: f64) -> Option<usize>;
   }
   ```

   A singleton y value expands by `max(abs(value) * 0.05, 1.0)` on both sides.
   Clamp `width` and `height` to at least one viewBox unit. Plot padding must
   include axis text, maximum valid marker radius, and data-label clearance.

4. Implement the remaining pure geometry contracts:

   ```rust
   pub(super) fn path_segments(
       series: &NormalizedSeries,
       projection: Projection,
   ) -> Vec<String>;

   pub(super) fn visible_tick_indices(
       category_count: usize,
       css_width: f64,
       minimum_gap_px: f64,
   ) -> Vec<usize>;

   pub(super) fn nearest_series_at(
       chart: &NormalizedChart,
       projection: Projection,
       category_index: usize,
       svg_y: f64,
   ) -> Option<usize>;

   pub(super) fn place_tooltip(
       anchor: Point,
       tooltip: Size,
       container: Size,
       gap: f64,
   ) -> TooltipPlacement;
   ```

   `path_segments` emits separate `M ... L ...` paths across gaps and never
   bridges a missing point. Tick thinning always retains the first and last
   categories. Tooltip placement tries upper-right, upper-left, lower-right,
   then lower-left, and finally clamps both axes into the chart wrapper.

5. Add serialization helpers for markers and patterns. A custom dash array is
   valid only when non-empty and every segment is finite and positive; invalid
   input returns no `stroke-dasharray`, identical to solid. Invalid marker
   size/stroke width resolves to documented defaults. Keep all numeric strings
   finite and bounded before they enter SVG.

6. Run the focused tests and the full library suite. Break `path_segments` by
   joining across one missing fixture, observe the gap test fail, restore, and
   record the evidence.

   ```powershell
   cargo test --lib charts::line_chart
   cargo test --lib
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/normalize.rs src/charts/line_chart/geometry.rs
   git add src/charts/line_chart.rs src/charts/line_chart/normalize.rs src/charts/line_chart/geometry.rs
   git diff --cached --check
   git commit -m "feat(charts): add categorical geometry (ldui-9tr.2)"
   bd close ldui-9tr.2 --reason "Normalization, projection, geometry, and edge policies verified" --json
   bd dolt push
   ```

## Task 3: Implement the Interaction Reducer and Activation Contract

**Bead:** `ldui-9tr.3`

**Files:** Create `src/charts/line_chart/interaction.rs`; modify
`src/charts/line_chart.rs` only to declare the module.

1. Claim the task and write failing native tests for hover/focus precedence,
   pointer leave, focus blur, arrows, Home, End, Escape, missing-category skips,
   preferred series, modifier copying, value ordering, and reconciliation by
   stable category key.

   ```powershell
   bd update ldui-9tr.3 --claim --json
   cargo test --lib charts::line_chart::interaction
   ```

2. Implement a pure reducer. Keep the roving focus target separate from the
   active tooltip so removing data can close the card while retaining a valid
   tab stop:

   ```rust
   #[derive(Clone, Debug, Default, PartialEq)]
   pub(super) struct InteractionState {
       pub hovered: Option<ActivePoint>,
       pub focused: Option<ActivePoint>,
       pub roving_category_key: Option<String>,
       pub dismissed_category_key: Option<String>,
   }

   #[derive(Clone, Debug, PartialEq)]
   pub(super) enum InteractionAction {
       PointerMoved(ActivePoint),
       PointerLeft,
       Focused(ActivePoint),
       Blurred,
       MoveFocus(NavigationKey),
       Dismiss,
       ReconcileData,
   }

   pub(super) fn reduce(
       state: &InteractionState,
       action: InteractionAction,
       previous: &NormalizedChart,
       next: &NormalizedChart,
   ) -> InteractionState;
   ```

   While the pointer is inside the plot, hover wins; after pointer leave, focus
   resumes. Escape hides the card for the current category until pointer or
   focus moves. ArrowLeft/ArrowRight move to the previous/next category with at
   least one finite value; Home/End jump to the first/last such category.

3. Implement one payload builder for pointer, Enter, and Space:

   ```rust
   pub(super) fn activation_for(
       chart: &NormalizedChart,
       active: ActivePoint,
       source: LineChartActivationSource,
       modifiers: LineChartModifiers,
   ) -> Option<LineChartActivation>;
   ```

   Preserve series order in `values`. Pointer input supplies the geometrically
   nearest series. Keyboard input uses its selected/preferred series, then the
   first finite series. A category with no finite values returns `None`.

4. Reconcile replacements by key. If the key remains, update index and values
   without firing activation. If it disappears, clear hover/focus/dismissed
   state and move the roving target to the nearest valid category by the old
   index. Duplicate keys select the first match, as specified.

5. Add a Rust-authored SVG focus helper using browser bindings rather than
   handwritten JavaScript:

   ```rust
   #[cfg(target_arch = "wasm32")]
   pub(super) fn focus_svg_element(id: &str) {
       let Some(element) = document().get_element_by_id(id) else { return };
       let Ok(value) = js_sys::Reflect::get(
           element.as_ref(),
           &wasm_bindgen::JsValue::from_str("focus"),
       ) else { return };
       let Ok(focus) = value.dyn_into::<js_sys::Function>() else { return };
       let _ = focus.call0(element.as_ref());
   }
   ```

   Provide a non-WASM no-op only if native compilation needs it; reducer tests
   must never require a browser.

6. Break value ordering in `activation_for`, observe the focused test fail,
   restore it, run the library suite, and commit only this slice:

   ```powershell
   cargo test --lib charts::line_chart::interaction
   cargo test --lib
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/interaction.rs
   git add src/charts/line_chart.rs src/charts/line_chart/interaction.rs
   git diff --cached --check
   git commit -m "feat(charts): add line chart interaction model (ldui-9tr.3)"
   bd close ldui-9tr.3 --reason "Reducer, reconciliation, and activation contract verified" --json
   bd dolt push
   ```

## Task 4: Render the Categorical SVG and Deterministic Demo

**Bead:** `ldui-9tr.4`

**Files:** Modify `src/charts/line_chart.rs`, `src/charts/paint.rs` only if a
composable paint helper is genuinely missing, `demo/src/demos/charts.rs`,
`tests/reactivity_smoke.rs`, and `src/charts/line_chart/geometry.rs` for
renderer-discovered pure fixes.

1. Claim the task. Add a failing browser test named
`categorical_line_chart_exposes_static_render_contract` to
`tests/reactivity_smoke.rs`. It must assert these selectors and values on the
real `/components/charts` page:

   | Selector/attribute | Expected fixture evidence |
   |---|---|
   | `[data-testid="interactive-line-chart"]` | exactly one categorical wrapper |
   | `[data-line-chart-plot]` | one SVG plot |
   | `[data-series-id]` paths | three distinct series ids |
   | `[data-series-id="rolling-average"]` | a non-empty `stroke-dasharray` |
   | `[data-series-id="actual"]` | no dash array |
   | `[data-category-index]` markers | circles, squares, and a missing-point gap |
   | `[data-line-chart-legend]` | three named entries with pattern swatches |
   | `[data-line-chart-table]` | caption, fourteen body rows, three series columns |
   | `[data-line-chart-empty]` | absent for the populated fixture |

   ```powershell
   bd update ldui-9tr.4 --claim --json
   cargo xtask test-reactivity
   ```

2. Put the new example first inside the existing `LineChart` section so it is
   immediately visible, while leaving both old `weekly_series()` expressions
   unchanged as compile/visual compatibility fixtures. Add a deterministic
   fourteen-category model with stable keys `week-01` through `week-14`:

   ```rust
   fn interactive_line_data() -> LineChartData {
       LineChartData::categorical(
           (1..=14)
               .map(|week| LineCategory {
                   key: format!("week-{week:02}"),
                   label: format!("W{week:02}"),
               })
               .collect(),
           vec![actual_series(), rolling_average_series(), target_series()],
       )
   }
   ```

   Use a primary solid circle series, a secondary dashed square series, and an
   accent dotted diamond series. Include one interior missing actual point,
   one short series, host-provided display strings, and selected data labels.
   Use only `var(--color-primary|secondary|accent)` paint values.

3. Render categorical output under a relative HTML group wrapper:

   ```text
   div[data-testid=interactive-line-chart][role=group]
     div[data-line-chart-legend]
     div[data-line-chart-stage]
       svg[data-line-chart-plot][role=img]
         title + desc
         axes/grid/ticks
         path segments per series
         marker shapes and optional labels
         category focus targets
         transparent pointer overlay
       div[data-testid=line-chart-tooltip][role=tooltip]
     table[data-line-chart-table].sr-only
   ```

   Use stable, instance-prefixed ids from an `AtomicU64`, matching the
   repository's `RosterGrid` approach. Add `data-series-id`,
   `data-category-index`, `data-category-key`, `data-active-category`, and
   `data-preferred-series` as test/debug selectors; do not encode state only in
   CSS classes.

4. Draw each contiguous segment as an SVG `<path>` so gaps remain visible.
   Route stroke through `stroke_attrs`; route marker fill and stroke through
   `paint_attrs`, `stroke_attrs`, and `merge_style`. Render circle, square, and
   diamond shapes from one marker-view helper. Apply pattern through
   `stroke-dasharray`, never by changing color alone.

5. Render a responsive legend when `legend_mode` resolves enabled. Its swatch
   must show both the stroke pattern and marker shape, and its text uses the
   series name. Render optional data labels with edge-aware anchor and on-ramp
   typography. Use `visible_tick_indices` based on measured CSS width; before
   measurement, use viewBox width as a deterministic initial value.

6. Render the hidden table by default even before interaction is wired. Caption
   uses `accessible_label`; header columns are series names; category rows use
   labels and formatted values; missing cells say `No value`. When categories,
   series, or finite values are absent, render a named `No chart data` state and
   suppress legend, focus targets, overlay, tooltip, and callbacks.

7. Run the static browser contract and audit the generated SVG strings for
   `NaN`/`inf`. Break the dashed fixture by forcing its pattern to solid,
   observe the dash assertion fail, restore, then commit:

   ```powershell
   cargo test --lib charts::line_chart
   cargo xtask test-reactivity
   cargo xtask check-demo
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/geometry.rs demo/src/demos/charts.rs tests/reactivity_smoke.rs
   git add src/charts/line_chart.rs src/charts/line_chart/geometry.rs demo/src/demos/charts.rs tests/reactivity_smoke.rs
   git diff --cached --check
   git commit -m "feat(charts): render categorical multi-line chart (ldui-9tr.4)"
   bd close ldui-9tr.4 --reason "Categorical SVG, legend, hidden table, and demo contract verified" --json
   bd dolt push
   ```

## Task 5: Wire Pointer, Keyboard, Tooltip, and Reactive Behavior

**Bead:** `ldui-9tr.5`

**Files:** Create `src/charts/line_chart/tooltip.rs`; modify
`src/charts/line_chart.rs`, `src/charts/line_chart/interaction.rs`,
`demo/src/demos/charts.rs`, `tests/common/mod.rs`, and
`tests/reactivity_smoke.rs`.

1. Claim the task. In `tests/common/mod.rs`, add two small reusable test seams:

   ```rust
   pub async fn move_pointer_to_svg_fraction(
       h: &Harness,
       selector: &str,
       x_fraction: f64,
       y_fraction: f64,
   );

   pub async fn click_svg_fraction(
       h: &Harness,
       selector: &str,
       x_fraction: f64,
       y_fraction: f64,
   );

   pub async fn begin_browser_error_capture(h: &Harness);
   pub async fn assert_no_browser_errors(h: &Harness, context: &str);
   ```

   The pointer helpers read the plot bounding box and send real CDP
   `MouseMoved` or `MousePressed`/`MouseReleased` input events. Error capture
   installs one buffered
   `console.error`, `window.error`, and `unhandledrejection` observer, clears
   the buffer before a journey, and reads it afterward. Keep each custom
   evaluation expression below 7 KB; do not replace real pointer/keyboard
   input with synthetic DOM events.

2. Add failing real-browser journeys before wiring behavior:

   1. Hover category 8 and assert one visible card, category label, all finite
      series rows in input order, preferred series, and matching root state.
   2. Hover the first and last categories and assert the tooltip bounding box
      stays within `[data-line-chart-stage]`.
   3. Tab into the chart, assert exactly one category target has `tabindex=0`,
      then exercise ArrowRight, End, Home, and Escape with `pixelproof_web::Key`.
   4. Click a marker-position with `click_svg_fraction`, press Enter, and press
      Space; after each action,
      assert exactly one new `chart.activation` payload with category key,
      preferred series, all finite values, source, and false modifiers.
   5. Reorder data while a card is active and assert reconciliation by key;
      remove that key and assert the card closes, focus moves to the nearest
      valid category, and no activation fires.
   6. Assert `assert_no_browser_errors` after each complete journey.

   ```powershell
   bd update ldui-9tr.5 --claim --json
   cargo xtask test-reactivity
   ```

3. Attach one `ResizeObserver` to the chart stage. Follow `DataTable`'s
   `Closure` plus `send_wrapper::SendWrapper` cleanup pattern: update a CSS-size
   signal only when width/height actually change, call `disconnect()` during
   cleanup, and never observe the SVG element whose viewBox output depends on
   the same signal. This prevents a measurement feedback loop.

4. Add one transparent plot overlay with `pointer-events: all`. On
   `pointermove`, convert `client_x/client_y` through the overlay's
   `get_bounding_client_rect()` into viewBox coordinates, use
   `Projection::category_at_x`, then `nearest_series_at`. On `pointerleave`,
   dispatch `PointerLeft`. On click, build the typed payload and invoke the
   callback once. Do not attach independent click handlers to visible markers.

5. Add one focus target per category with finite data. It has a generous
   invisible focus box, a visible focus-ring child, deterministic id, accessible
   name, and roving `tabindex`; it has `pointer-events: none` so the overlay owns
   pointer hit testing. `focus`, `blur`, and `keydown` dispatch reducer actions.
   Prevent default only for ArrowLeft/Right, Home/End, Escape, Enter, and Space.
   After navigation, schedule `focus_svg_element` for the new target id.

6. Build the tooltip view model independently from its DOM:

   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub(super) struct TooltipModel {
       pub id: String,
       pub category_label: String,
       pub rows: Vec<TooltipRow>,
       pub preferred_series_id: Option<String>,
       pub anchor: Point,
   }
   ```

   Render it as an absolutely positioned HTML card with `role="tooltip"`, no
   focusable elements, pattern/marker swatches, host display strings, and a
   stable id. Measure the card after render through a `NodeRef<html::Div>` and
   `request_animation_frame`, call `place_tooltip`, then update transform.
   Hide with `visibility: hidden` until measured so an unclamped first frame is
   never painted.

7. Make activation semantics conditional. With `on_point_activate`, targets
   expose `role="button"` and Enter/Space invoke the callback. Without it, use
   descriptive `role="group"`; hover/focus card and navigation still work, but
   click/Enter/Space are inert and do not claim button behavior.

8. In the demo, store categorical data in `RwSignal<LineChartData>`. Provide
   deterministic `Reorder data`, `Remove active week`, and `Restore data`
   controls. Convert each activation into an explicit `serde_json::json!`
   object and write it with
   `debug_state::set("chart.activation", payload)`. Also write
   `chart.activation_count` so duplicate callbacks cannot pass by overwriting
   the same key. Add a `Show gaps` control that replaces the model with a
   deterministic multi-gap variant for the visual state in Task 7. The public
   activation types need not derive `Serialize`.

9. Break click handling by invoking the callback twice, observe the activation
   count test fail, restore, then run the focused native and browser gates:

   ```powershell
   cargo test --lib charts::line_chart
   cargo xtask test-reactivity
   cargo xtask check-demo
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/interaction.rs src/charts/line_chart/tooltip.rs demo/src/demos/charts.rs tests/common/mod.rs tests/reactivity_smoke.rs
   git add src/charts/line_chart.rs src/charts/line_chart/interaction.rs src/charts/line_chart/tooltip.rs demo/src/demos/charts.rs tests/common/mod.rs tests/reactivity_smoke.rs
   git diff --cached --check
   git commit -m "feat(charts): add interactive line chart behavior (ldui-9tr.5)"
   bd close ldui-9tr.5 --reason "Pointer, keyboard, tooltip, activation, D1, and reconciliation journeys verified" --json
   bd dolt push
   ```

## Task 6: Prove the Accessibility Contract with axe

**Bead:** `ldui-9tr.6`

**Files:** Modify `src/charts/line_chart.rs`,
`src/charts/line_chart/tooltip.rs`, `tests/reactivity_smoke.rs`; create
`tests/vendor/axe-core/axe.min.js`, `tests/vendor/axe-core/LICENSE`, and
`tests/vendor/axe-core/README.md`.

1. Claim the task and add failing DOM assertions for the exact semantic graph:

   1. Named outer `role="group"`.
   2. SVG `role="img"` with unique `<title>` and `<desc>` references.
   3. Exactly one roving tab stop among finite categories.
   4. Target names containing category plus every available series value.
   5. `role="button"` only when activation is configured.
   6. Active target `aria-describedby` points to the one visible tooltip.
   7. Hidden table caption, headers, fourteen rows, and `No value` cells.
   8. Keyboard focus changes a non-color cue such as ring width/marker size.

   ```powershell
   bd update ldui-9tr.6 --claim --json
   cargo xtask test-reactivity
   ```

2. Vendor the official npm package rather than a CDN URL. Use a temporary
   directory under `.review`, verify `package/package.json` says `4.13.0`, copy
   `package/axe.min.js` and its MPL-2.0 license, and record package version,
   source URL, upgrade command, and the emitted SHA-256 in the README:

   ```powershell
   New-Item -ItemType Directory -Force .review\axe-core | Out-Null
   npm pack axe-core@4.13.0 --pack-destination .review\axe-core
   tar -xf .review\axe-core\axe-core-4.13.0.tgz -C .review\axe-core
   Get-Content .review\axe-core\package\package.json | ConvertFrom-Json | Select-Object name,version,license
   Get-FileHash .review\axe-core\package\axe.min.js -Algorithm SHA256
   New-Item -ItemType Directory -Force tests\vendor\axe-core | Out-Null
   Copy-Item .review\axe-core\package\axe.min.js tests\vendor\axe-core\axe.min.js
   Copy-Item .review\axe-core\package\LICENSE tests\vendor\axe-core\LICENSE
   ```

   Copying this third-party test asset is a mechanical vendor operation. Add
   the README with `apply_patch`, using the version and hash just read back. Do
   not load axe from the network at test time. Keep axe out of `demo/`, Trunk,
   and non-dev dependencies.

3. Add an opt-in browser-gated test to `tests/reactivity_smoke.rs`. It runs via
   `cargo xtask test-reactivity` and `verify-full`, not ordinary `verify`:

   ```rust
   let axe = pixelproof_web::a11y::Axe::from_path(
       "tests/vendor/axe-core/axe.min.js",
   )
   .expect("load vendored axe-core");
   let report = axe.run(h.page()).await.expect("run axe-core");
   report
       .assert_no_blocking("interactive-line-chart")
       .unwrap_or_else(|e| panic!("{e}; {}", report.summary()));
   ```

   The test gate is zero new Serious/Critical violations. Print every blocking
   rule id and selector on failure. Do not disable a rule to make the initial
   result green; fix the component semantics or document a verified
   PixelProof/axe defect in a new bead.

4. Confirm tab entry/traversal/exit with real CDP input: Tab reaches one chart
   target, arrows stay within the composite, Enter/Space activate only the
   interactive chart, Escape dismisses, and the next Tab leaves the chart.
   Confirm the callback-less categorical fixture has descriptive groups and no
   false button roles.

5. Break the outer accessible label, observe either the semantic or axe test
   fail, restore, and run the complete reactivity suite. Stage the vendored
   license/readme with the component/test changes:

   ```powershell
   cargo xtask test-reactivity
   cargo test --doc
   rustfmt --edition 2024 src/charts/line_chart.rs src/charts/line_chart/tooltip.rs tests/reactivity_smoke.rs
   git add src/charts/line_chart.rs src/charts/line_chart/tooltip.rs tests/reactivity_smoke.rs tests/vendor/axe-core/axe.min.js tests/vendor/axe-core/LICENSE tests/vendor/axe-core/README.md
   git diff --cached --check
   git commit -m "test(charts): enforce line chart accessibility (ldui-9tr.6)"
   bd close ldui-9tr.6 --reason "Semantic, keyboard, hidden-table, and axe contracts verified" --json
   bd dolt push
   ```

## Task 7: Add PixelProof Visual and Audit Regression Gates

**Bead:** `ldui-9tr.7`

**Files:** Modify `tests/common/mod.rs`, `tests/visual_smoke.rs`,
`tests/style_audit_smoke.rs`, and `tests/layout_audit_smoke.rs`; create reviewed
PNG files under
`tests/visual/baselines/charts/interactive-line-chart/`.

1. Claim the task. Extend `state` with an explicit-width form while preserving
   existing filenames:

   ```rust
   pub fn state_at(name: &str, width: u32) -> String {
       format!("{name}.w{width}")
   }

   pub fn state(name: &str) -> String {
       state_at(name, VIEWPORT.width)
   }
   ```

2. Add five ignored PixelProof tests using
   `Harness::capture_and_compare_region` and selector
   `[data-testid="interactive-line-chart"]`:

   | State | Required setup and oracle |
   |---|---|
   | `default.w1280` | no active point; three patterned lines, legend, labels, and markers visible |
   | `hovered.w1280` | real pointer at category 8; card lists all values and remains inside wrapper |
   | `focused.w1280` | real Tab/arrow input; visible focus cue and the same card contract |
   | `missing-data.w1280` | click `Show gaps`; the replacement fixture has multiple interior gaps and paths never bridge them |
   | `narrow.w768` | `ViewportSize::TABLET`; ticks thin, legend wraps, edge label and tooltip remain inside |

   After every setup, call `assert_region_within` for tooltip/labels that have
   DOM boxes and assert the root/SVG contain no hard overlap. Use the component
   region so a small chart regression cannot be diluted by the full showcase
   page.

3. Run compare mode first and observe missing-baseline failures. Then set
   `VISUAL_TEST_MODE=capture`, capture exactly these states, and inspect each PNG
   at original detail with the available image-viewing tool before accepting
   it. Never approve a baseline from a failing DOM, D1, accessibility, or
   containment test.

   ```powershell
   cargo make test-visual
   $env:VISUAL_TEST_MODE='capture'
   cargo make test-visual
   Remove-Item Env:VISUAL_TEST_MODE
   cargo make test-visual
   ```

4. Run the charts page through the existing style and layout sweeps. Keep the
   current ceilings when counts do not rise. If the new chart changes a count,
   inspect every new selector, fix accidental off-ramp typography, shape,
   shadow, overlap, grid, or internal-spacing output, then set the constant to
   the measured irreducible count with a source comment. A ceiling may not gain
   spare headroom. Shape, depth, component drift, internal spacing, and overlap
   should remain zero for the new renderer.

   ```powershell
   cargo xtask test-style
   cargo xtask test-layout
   ```

5. Prove the visual oracle by temporarily changing the actual series dash or
   marker radius, observe the component-region SSIM comparison fail below the
   repository's `0.98` threshold, restore it, and observe green. Prove the
   containment oracle by temporarily moving the right tooltip beyond the stage
   and observing `assert_region_within` fail. Record both failures in
   `ldui-9tr.7`.

6. Stage only the test sources and reviewed chart baselines. Inspect every PNG
   path and staged source diff before committing:

   ```powershell
   rustfmt --edition 2024 tests/common/mod.rs tests/visual_smoke.rs tests/style_audit_smoke.rs tests/layout_audit_smoke.rs
   git add tests/common/mod.rs tests/visual_smoke.rs tests/style_audit_smoke.rs tests/layout_audit_smoke.rs tests/visual/baselines/charts/interactive-line-chart
   git diff --cached --check
   git diff --cached --stat
   git commit -m "test(charts): baseline interactive line chart (ldui-9tr.7)"
   bd close ldui-9tr.7 --reason "Reviewed component baselines, containment, and audit ratchets verified" --json
   bd dolt push
   ```

## Task 8: Verify, Document, and Prepare the Consumer Handoff

**Bead:** `ldui-9tr.8`

**Files:** Modify public rustdoc in `src/charts/line_chart.rs` and
`src/charts/line_chart/types.rs`, plus the Consumer Migration section of the
approved specification. Modify implementation/test files only for defects found
by the final gates.

1. Claim the task. Re-read the specification and check every acceptance item
   against a named test. Add missing assertions before changing documentation.
   Confirm all prior child beads are closed:

   ```powershell
   bd update ldui-9tr.8 --claim --json
   bd list --parent ldui-9tr --json
   ```

2. Add one compiling rustdoc example for legacy XY and one for categorical
   activation. The categorical example should use the public builders, a
   reactive source, and a typed callback, with no raw event or JavaScript:

   ```rust
   let data = RwSignal::new(LineChartData::categorical(categories, series));
   let activated_key = RwSignal::new(None::<String>);
   let on_point_activate = Callback::new(move |intent: LineChartActivation| {
       // The host maps this key to its own route/filter/request.
       activated_key.set(Some(intent.category_key));
   });

   view! {
       <LineChart
           data=data
           accessible_label="Weekly matters".to_string()
           on_point_activate=on_point_activate
       />
   };
   ```

   Use a compilable output mechanism already available to doctests rather than
   adding `log` solely for this example; the comment above expresses the
   consumer-owned side effect.

3. Expand the specification's Consumer Migration section with an exact
   `WeekPointDto -> LineChartData` mapping and activation callback shape for
   `4iiz-office`. Explicitly state that its request/filter completion and error
   journeys remain a separate consumer issue. Do not edit or commit anything in
   that sibling repository.

4. Prove the application bundle remains WASM/DOM-owned. Test-only CDP
   evaluation strings for assertions and error capture are allowed, but they
   are not application runtime code. Verify no new chart dependency appears in
   manifests or lockfile, no `.js` file exists under `src/` or `demo/` for this
   feature, and the only new `.js` source file is the test-only axe vendor
   asset:

   ```powershell
   rg -n 'leptos-chartistry|plotters|chart\.js|echarts|\bd3\b' Cargo.toml Cargo.lock demo/Cargo.toml
   rg --files src demo -g '*.js'
   git diff --name-only 6a54cad..HEAD -- '*.js'
   ```

   The first two commands are expected to return no feature-related matches;
   the final command should name only
   `tests/vendor/axe-core/axe.min.js` when the glob is evaluated across the
   repository.

5. Run the complete gate set with direct exit statuses. Do not pipe a build or
   test into `tail`, `tee`, or another command and then trust `$LASTEXITCODE`:

   ```powershell
   cargo fmt --all -- --check
   cargo test
   cargo test --doc
   cargo xtask test-reactivity
   cargo xtask test-style
   cargo xtask test-layout
   cargo make test-visual
   cargo xtask verify-full
   ```

6. Inspect source and generated DOM one final time for `NaN`, infinity, duplicate
   ids, direct `var()` presentation attributes, more than one roving tab stop,
   tooltip overflow, callback duplication, and hidden-table mismatch. Run
   `git diff --check` and an unresolved-token scan over the specification,
   plan, and public docs.

7. Stage only final documentation/fixes, commit, close the task and epic, sync
   beads, rebase only if the worktree is clean enough to do so without touching
   user changes, and push. Verify the remote branch by reading it back:

   ```powershell
   git add src/charts/line_chart.rs src/charts/line_chart/types.rs docs/superpowers/specs/2026-08-14-interactive-multi-line-chart-design.md
   git diff --cached --check
   git commit -m "docs(charts): finish interactive line chart handoff (ldui-9tr.8)"
   bd close ldui-9tr.8 --reason "Full gates and consumer handoff verified" --json
   bd close ldui-9tr --reason "Interactive multi-line LineChart implemented and verified" --json
   bd dolt push
   git fetch fork
   git push
   git status --short --branch
   git ls-remote fork refs/heads/feature/visual-quality-audit
   ```

   If `git pull --rebase` refuses because unrelated user changes remain, do not
   stash or reset them. Fetch, prove the remote is not ahead with
   `git rev-list --left-right --count HEAD...fork/feature/visual-quality-audit`,
   then push. Completion requires the remote hash to equal local `HEAD`.

## Final Acceptance Map

| Specification outcome | Primary proof |
|---|---|
| Unchanged legacy callers and visual mode | Task 1 demo compile plus existing XY examples and visual checks |
| Solid/dashed/dotted lines, markers, labels, legend, gaps | Task 2 unit geometry, Task 4 DOM contract, Task 7 region baselines |
| Category-wide hover/focus card and edge clamping | Task 2 placement tests, Task 5 CDP journeys, Task 7 containment and SSIM |
| Click/Enter/Space typed drilldown intent | Task 3 payload tests and Task 5 activation-count oracle |
| Reactive data reconciliation | Task 3 reducer tests and Task 5 reorder/remove journeys |
| Keyboard and non-visual access | Task 5 roving focus, Task 6 semantic/axe/hidden-table tests |
| No browser panic or hidden error | Task 5 D1 capture after every journey |
| No component-owned network/database side effect | API inspection; D2 remains the consuming application's responsibility |
| WASM-owned SVG/HTML runtime with no JS chart dependency | Task 8 manifest/file audit and Trunk/browser verification |
| Visual-system compliance | Task 7 style/layout ratchets and reviewed component-region baselines |
