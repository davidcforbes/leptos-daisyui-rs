# Heatmap

`leptos_daisyui_rs::charts::Heatmap` renders two surfaces from one prop set and
one geometry: the preserved **positional** grid (`row_labels`, `col_labels` and
a `Vec<HeatmapCell>` addressed by array index), and a **typed** matrix with
stable row and column keys, an accessible name and description, an equivalent
row-by-column data table, reactive copy, and optional keyboard/pointer
activation.

This page documents the accessibility/i18n work (`ldui-8d94`). The colour ramp,
the `max_cell_h` cap and the slanted headers are documented on the component
itself.

## The defect this replaces

The component was a picture with no non-visual equivalent, and its one
interaction hook was pointer-only and positional:

1. **No role, no name, no description, no data table.** The SVG announced
   nothing. Every visible number was painted as SVG `text` with no row, no
   column and no measure attached to it, so a screen-reader user heard a stream
   of bare percentages.
2. **A hard-coded English `"No data"`** in the empty branch, unreachable by any
   locale.
3. **`on_cell_click` added transparent rects with a pointer cursor and
   `on:click` only** — no focusability, no role, no accessible name, no
   Enter/Space, no focus recovery. A pointer-only SVG is not an interactive
   control.
4. **The callback emitted `(row_index, col_index)`.** Sorting the offices
   worst-first, or hiding a KPI column, re-points both numbers at a different
   cell — silently, with no error and no length mismatch to catch it.

The consumer's workaround was to hand-roll a second HTML table beside the
chart. Two parallel projections of one dataset drift, which is why the fix
belongs here rather than in a consumer overlay.

## A cell's identity is a PAIR of keys

```rust
use leptos_daisyui_rs::charts::{HeatmapCategory, HeatmapMatrix, HeatmapValue};

let matrix = HeatmapMatrix::new(
    vec![HeatmapCategory::new("office-north", "North")],
    vec![
        HeatmapCategory::new("closed", "Matters closed"),
        HeatmapCategory::new("sla", "SLA met"),
    ],
    vec![
        HeatmapValue::new("office-north", "closed", 0.6)
            .with_display_value("+12%")
            .with_accessible_value("12 percent above the 12-week baseline"),
        HeatmapValue::missing("office-north", "sla"),
    ],
);
```

`HeatmapCategory` bundles a stable key with its localized label, so no sort of
either axis can separate them. `HeatmapValue` names the row and the column it
belongs to, so **the values never have to be touched when an axis moves** — the
demo's "sort offices worst first" control rewrites only `rows`.

Two consequences are deliberate:

- A value naming a key the axis does not carry is **dropped**, not placed at a
  guessed index. Hiding a column is therefore a one-line change: filter
  `columns` and leave `values` alone.
- The rendered grid is **dense**. Every `(row, column)` combination exists as a
  cell, populated or missing, which is what lets the data table state a
  complete matrix and lets an arrow key move a predictable distance.

`display_value` is the short text drawn in the tile; `accessible_value` is the
complete localized text a screen reader hears instead. One resolver
(`accessible_value` → `display_value` → `texts.missing_value`) feeds the table
cell, the accessible name and the activation payload, so they cannot disagree.

## The keyboard model, in two axes

The ARIA grid pattern, because a matrix is what a reader has already met:

| Key | Moves to |
|---|---|
| Left / Right | previous / next **column**, same row |
| Up / Down | previous / next **row**, same column |
| Home / End | first / last column **of the current row** |
| Ctrl+Home / Ctrl+End | first / last cell of the whole **grid** |
| Escape | drops the highlight, **without** moving the tab stop |
| Enter / Space | activates, when a callback is wired |

Every move clamps rather than wrapping: jumping from the last column back to the
first hides the edge of the matrix from exactly the reader who cannot see where
it is. The whole grid is **one tab stop** (roving `tabindex`), so Tab enters and
leaves it rather than walking 36 cells.

Focus is held as a pair of keys and reconciled across data changes: the two axes
reconcile independently, so sorting the rows leaves a reader on the same office
*and* the same KPI, and removing a column moves them to whatever now occupies
its position while keeping their row.

## The data table is a matrix, not a list

```html
<table class="sr-only">
  <caption>Office by KPI deviation from baseline</caption>
  <thead>
    <tr><th scope="col">Office</th><th scope="col">Matters closed</th>…</tr>
  </thead>
  <tbody>
    <tr><th scope="row">North</th><td>+12%, Favorable</td>…</tr>
  </tbody>
</table>
```

Row labels are `th[scope="row"]` and column labels `th[scope="col"]`, so a
screen reader announces **both** headers when the reader lands on a cell: a
value is located by "North, SLA met", not by counting position in a flat stream.
The corner cell names the row axis (`texts.row_header`). Every coordinate has a
cell, so a gap is heard as the localized missing copy *at its own position*
rather than being skipped.

`show_data_table` defaults to `true` and can only be turned off explicitly.

## Judgement is never colour alone

Under `HeatScale::Judgement` the sign of the intensity picks the hue. That hue
is invisible under forced colours, to a reader with a colour-vision deficiency,
and to a screen reader, so the typed surface adds two more carriers of the same
verdict:

- a **sense rule** drawn inside the cell — solid for favorable, dashed (`3 2`)
  for unfavorable, absent for no verdict — the same solid/dashed convention as
  `BarChart`'s status caps;
- the verdict **in words**, appended to the cell's reading in the data table and
  in its accessible name (`"+12%, Favorable"`).

An exactly-zero deviation is a real measurement with no verdict, and paints
fully transparent under either hue, so it gets neither. `HeatScale::Magnitude`
has no sign to read and therefore expresses no verdict at all — which is the
reason the judgement axis exists.

## The activation payload carries no index

```rust
pub struct HeatmapActivation {
    pub row_key: String,
    pub row_label: String,
    pub column_key: String,
    pub column_label: String,
    pub intensity: Option<f64>,
    pub display_value: String,
    pub sense: HeatmapSense,
    pub source: HeatmapActivationSource,
    pub modifiers: HeatmapModifiers,
}
```

`intensity` is an `Option` because **every grid position is activatable**,
including one with no measurement: a heatmap cell is a coordinate — this office
by that KPI — and a reader drilling into it is asking about the coordinate, not
about a number that may not exist. A gap therefore activates honestly with
`intensity: None` and the localized missing copy, rather than a fabricated zero.
(This is the one place the heatmap diverges from `BarChart`, where a missing bar
is not activatable at all: a bar *is* its value, a heatmap cell is a position.)

## Interaction is opt-in

`HeatmapInteractionMode::Auto` (the default) makes a typed grid interactive
**exactly when an activation callback is wired**. This is stricter than
`BarChart`'s equivalent default on purpose: a matrix has rows-times-columns
cells rather than a handful, its complete non-visual truth is already in the
data table, and a tab stop into a purely descriptive grid buys a keyboard reader
nothing while costing them a stop to escape from. `Enabled` forces navigable
cells with no button role; `Disabled` forbids them.

Roles follow the rule `LineChart` established (`ldui-9tr.6`): an interactive SVG
is `role="group"`, **never** `role="img"` — `img` makes every descendant
presentational, which contradicts the focusable cells inside and blocks axe. A
target is `role="button"` only when a callback is actually wired.

## Backward compatibility

Every existing caller keeps working, unchanged:

- `row_labels`, `col_labels` and `cells` are still accepted (now `optional`, so
  a named-argument call site compiles identically) and still render the original
  element tree: the same rects, the same labels, no wrapper, no roles, no tab
  stops, no data table.
- `on_cell_click` still fires with `(row, col)` and still overlays **every** grid
  position, including empty ones.
- The empty branch now reads `texts.no_data`, whose default is the string it
  hard-coded — `"No data"` — so nothing moves until a caller overrides it.

**The migration path is incremental.** `on_cell_click` also fires on the typed
surface, with the activated cell's current indices, alongside `on_cell_activate`.
A consumer can therefore adopt `HeatmapMatrix` first and rewrite its handler
second. The indices are positions in the *current* render and re-point the
moment either axis moves, which is exactly why `HeatmapActivation` carries keys
— treat the overlap as a bridge, not a destination.

## Copy

Every user-visible string the component produces is a field of `HeatmapTexts`,
supplied as a `Signal` so a locale change re-renders the words without touching
keys, intensities, order, focus or the identity an activation reports:

| Field | Default | Where it appears |
|---|---|---|
| `no_data` | `"No data"` | the empty grid |
| `data_table_caption` | `"Heatmap data"` | the table `<caption>` |
| `row_header` | `"Row"` | table corner cell; the row half of a cell's name |
| `column_header` | `"Column"` | the column half of a cell's name |
| `value_header` | `"Value"` | the value half of a cell's name |
| `missing_value` | `"No value"` | any coordinate with no measurement |
| `sense_favorable` / `sense_unfavorable` / `sense_neutral` | `"Favorable"` / `"Unfavorable"` / `"Neutral"` | the verdict, in words |

A cell's accessible name states both axis names because an SVG target has no
table structure to borrow them from — `"Office: North, KPI: Matters closed,
Deviation: 12 percent above the 12-week baseline, Favorable"`. Without them the
cell announces three unlabelled fragments.

## Testing

- Native, in `src/charts/heatmap/`: the colour ramp and the sense derivation
  (`scale.rs`), the grid frame (`geometry.rs`), the dense normalization and the
  one value resolver (`normalize.rs`), the two-axis reducer including every
  reorder and removal journey (`interaction.rs`), the public types
  (`types.rs`), and the role/name/activation decisions (`tests.rs`).
- Browser, in `tests/heatmap_matrix_smoke.rs`: the rendered matrix, the live
  focus journeys in both axes, pointer and keyboard activation, the EN → ES → EN
  round trip, the localized empty state, and the reporting-only posture. **That
  lane is not yet registered in `xtask`** — see the file's module docs.
