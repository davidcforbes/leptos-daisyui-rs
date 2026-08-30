# EntityTable

`EntityTable<T>` is the opinionated table for a complete, typed client
snapshot. The caller downloads the whole selected dataset, supplies stable row
keys and typed column functions, and the component owns transient paging while
sorting, resizing, column visibility, and column order stay local. When the snapshot needs
filtering, the caller supplies the locally filtered snapshot and changes
`page_reset_key` to return paging to the first page.

## Choose the table from data ownership

| Data ownership | Component | Observable mode | Rule |
|---|---|---|---|
| Complete typed snapshot is already in the browser | `EntityTable<T>` | `client-snapshot` | Preferred for new contracted snapshot pages. |
| Server owns filtering, sorting, paging, and total count | `ServerDataTable` | `server-query` | Pass only the current slice and round-trip every query change. |
| Existing client table uses dynamic `HashMap` rows or DataTable-only features | `components::DataTable` | `compatibility-client` | Compatibility path; do not choose it for a new contracted snapshot by habit. |
| Existing simple table needs automatic link/badge columns or bulk selection | `widgets::DataTable` | n/a | Retained legacy widget with a different row model. |

Do not pass a server page to `EntityTable` and let it sort or filter that slice.
That silently changes a server query into page-local behavior. Do not download a
complete dataset merely to satisfy `EntityTable` when the server must own the
query. The component roots expose `data-table-data-mode` so browser audits can
detect an ownership mismatch on the running page.

## Shared mechanics, separate data models

`EntityTable` and the DataTable family intentionally keep different row and
column types, but they do not carry separate behavior for common mechanics:

| Mechanic | Shared owner |
|---|---|
| Page count, clamping, bounds, numbered window, and row-range caption | `components::data_table::pagination` |
| Resize minimum, maximum, and drag-delta bounds | `components::data_table::resize` |
| Show/hide transition, required-column guard, and last-visible guard | `components::data_table::chooser` |

The renderers remain separate because `EntityColumn<T>` resolves typed rows,
whereas `Column` resolves dynamic `TableRow` maps. Sharing those renderers would
erase the compile-time distinction the snapshot component exists to provide.

## Core inputs

| Prop | Purpose |
|---|---|
| `data: Signal<Rc<Vec<T>>, LocalStorage>` | Rows rendered by the current local view; this may be a controlled filtered projection. |
| `columns: EntityColumns<T>` | Static `Vec` compatibility or local reactive typed declarations; stable IDs own preference identity. |
| `row_key: EntityRowKey<T>` | Stable identity used for keyed DOM rows and activation. |
| `dataset_identity: Signal<String>` | Identifies the downloaded dataset; a change resets only the current page. |
| `page_reset_key` | Optional identity for local view-state changes that should reset only paging. |
| `viewport_fit` | Optional `EntityTableViewportFit`; derives visible row capacity from a definite parent or CSS height without changing the saved page-size preference. |
| `compact_row: EntityCompactRow<T>` | Default, static, or reactive single-cell renderer used at compact breakpoints without duplicating rows. |
| `column_filters: EntityColumnFilters` | Controlled one-to-one filter controls aligned beneath stable desktop columns. |
| `source_data` | Optional authoritative source membership used only for safe post-removal focus recovery. |
| `focus_scope` | Optional opaque dataset/access generation; recovery never crosses a change. |
| `preference_ownership` | Controlled or uncontrolled preference policy. |
| `storage_key` | Legacy local-storage compatibility prop; mutually exclusive with `preference_ownership`. |
| `page_size_control_id` | Optional stable caller-owned DOM ID for the rows-per-page select. |
| `toolbar_actions` | Optional caller-rendered table utilities placed after page size and immediately before the framework-owned column chooser. |
| `on_display_projection` | Optional callback receiving one atomic read-only snapshot of ordered visible columns plus sorted/filtered rows and current-page bounds. |
| `projection_action_columns` | `Exclude` by default; set `EntityTableActionColumnPolicy::Include` only when action-copy intentionally belongs in the projection. |
| `column_chooser_trigger` | Reactive `Text` (default) or compact framework-owned `Icon` presentation; the localized accessible name is unchanged. |

Page number, free-text search, selected dataset, row data, and snapshot revision
are transient state and do not belong in `EntityTablePreferences`.

For the canonical page, pass a state-minted
`SnapshotLocalRowProjection<T>` through `SnapshotTablePage::local_rows`; the
page supplies its filtered rows as `data` and the complete displayed snapshot
as `source_data`. Standalone tables should preserve the same distinction and
give each page-size control a collision-safe `page_size_control_id`.

`toolbar_actions` is presentation-only composition. The table owns the
wrapping toolbar and chooser placement; the caller owns action labels and
behavior (for example Export CSV or Refresh). Omitting the slot produces no
wrapper or markup change. Supplying it cannot mutate table preferences or
dataset identity unless the caller explicitly wires such behavior into its own
action.

`EntityColumnChooserTrigger::Icon` renders a compact gear button next to those
actions. It is presentation-only: both variants remain native buttons with the
same localized `aria-label`, menu relationship, expanded state, Enter/Space
activation, Escape dismissal, and focus restoration. The icon is
`aria-hidden`, and its forced-colors border/text use system colors. Consumers
cannot replace the chooser control or remove its semantics.

## Reactive column semantics

Existing `columns=vec![...]` calls remain source-compatible. A localized or
otherwise runtime-defined table can instead pass
`Signal<Vec<EntityColumn<T>>, LocalStorage>`. Each replacement advances an
internal semantic generation: headers, chooser copy, default compact labels,
sort/accessibility names, and comparator or sort-key behavior all switch to
the new declarations. The sorted-index cache includes that generation, so an
unchanged row `Rc` and sort value can never reuse indices from an obsolete
comparator.

Preferences normalize by stable column ID after replacement. Surviving order,
visibility, widths, sort clauses, and page size remain intact; removed IDs and
newly non-sortable sort clauses disappear, and new IDs append in system order.
A label-only locale update therefore updates mounted header nodes without
resetting consumer state.

`EntityTableTexts` is a live signal and covers every framework-owned visible
or accessible string, including column-order actions, resize names/value text,
sort state/action/summary copy, region name, pagination, and empty state.

## Plain-text overflow policies

`EntityColumn::text` wraps normally by default. Use `.ellipsis()` for a
single clipped line or `.line_clamp(lines)` for a positive, bounded number of
lines. Passing zero to `line_clamp` is rejected immediately. The same policy
is applied to the ordinary wide cell and the framework's default compact row,
so a breakpoint does not silently change which value is shown.

The original text callback remains canonical: its complete value stays in the
DOM for assistive technology, sorting, and future display projections. Clipped
cells also expose that same complete value as a native `title`; they do not add
a second screen-reader-only copy that would be spoken twice. A long unbroken
value therefore cannot widen its declared track, but it is never shortened in
the data model.

`render_with` remains the rich-content escape hatch and takes precedence over
the plain-text overflow policy. A custom renderer owns its own clipping and
accessible composition; the table does not wrap it in an additional overflow
element. Prefer the typed policy whenever the cell is otherwise ordinary text.

## Alignment and numeric presentation

Formatting stays caller-owned: the canonical text callback decides currency,
percentage, duration, sign, optional, and localized display strings. Add
`.align_start()`, `.align_center()`, or `.align_end()` when the framework should
align the wide header, wide value, and compact value consistently. Add
`.tabular_numbers()` to inherit tabular-width numeral glyphs without changing
the text or its sort key.

The compatibility `Auto` default preserves current layout: wide content starts
at the inline start while values in the default compact label/value row end at
the inline end. An explicit alignment uses that edge in both layouts. Compact
labels always remain at the start so columns scan as label/value pairs.

Alignment and tabular numerals live on the header/cell presentation wrappers.
They therefore survive resize, reorder, visibility, sorting, scrolling, and
forced-colors mode and still apply as inherited presentation around a
`render_with` result. A rich renderer may deliberately override either style
inside its own markup. The canonical full text remains the accessibility,
sorting, and export value; these builders never introduce number formatters.

## Semantic badge and icon cells

Use `.badge_with(...)` or `.icon_with(...)` for ordinary status treatment that
does not justify a custom view. The canonical text callback remains the only
value source. A badge displays that text once inside an LDUI small soft badge;
the mapper chooses its semantic `BadgeColor` and may replace the default
`BadgeStyle::Soft`. An icon is decorative and the canonical text is emitted
once as screen-reader copy, so sorting, accessibility, and future export all
agree even though the visible cell is glyph-only.

Both mappers return `Option`. `None` deliberately falls back to the ordinary
plain-text renderer for unknown or unmapped domain states. An empty canonical
value renders an empty marked cell instead of an unlabeled badge/icon or an
invented, non-localized placeholder. Reactive row or column replacement
updates visible badge text and icon accessibility copy. Wide and compact rows
use the same renderer, and forced-colors hooks route borders and foregrounds
through system colors.

`render_with` always wins regardless of builder call order. Alignment and
tabular-number metadata still live on the surrounding cell, but the rich
renderer owns all inner markup. This keeps existing custom cells compatible
while making the common badge/icon path auditable and consistent.

## Primary and secondary text presentation

Use `.primary_secondary(primary, secondary)` for an opinionated two-line
cell -- a primary line with an optional muted caption beneath it -- in place
of one plain canonical-text line. Like `.badge_with` and `.icon_with`, this
only changes visual presentation: the column's canonical `text` callback
(from `EntityColumn::new`/`text`) remains the sole accessible name, sort
input (unless overridden with `.sortable_by_key`/`.sortable_by`), and future
export value.

The canonical value stays complete and is never spoken twice: the primary
and secondary lines render inside an `aria-hidden` wrapper, and a single
`sr-only` span beside it carries the complete canonical text once for
assistive technology, title-on-hover truncation, and any future display
projection. Write the canonical `text` callback as a complete value on its
own -- for example folding role/status into one string -- even though the
primary/secondary split only shows part of it visually.

```rust,no_run
use leptos_daisyui_rs::components::EntityColumn;
# struct Row { name: String, role: Option<String> }
let column = EntityColumn::text("contact", "Contact", |row: &Row| {
    match row.role.as_deref().map(str::trim).filter(|role| !role.is_empty()) {
        Some(role) => format!("{} (Role: {role})", row.name),
        None => row.name.clone(),
    }
})
.primary_secondary(
    |row: &Row| row.name.clone(),
    |row: &Row| {
        row.role
            .as_deref()
            .map(str::trim)
            .filter(|role| !role.is_empty())
            .map(|role| format!("Role: {role}"))
    },
);
```

`secondary` returns `Option<String>`; `None`, or an empty or
whitespace-only value, renders no secondary line at all -- no stray spacing
or punctuation left behind for an absent second line. Both lines follow the
column's existing `EntityTextOverflow` policy (wrap/ellipsis/line-clamp),
and sorting is untouched by this call. A `render_with` renderer always wins
over `primary_secondary`, exactly as it wins over badge/icon.

## Display and export projection

Bind `on_display_projection` to caller-owned state when a toolbar action must
export exactly what the table is displaying. Each
`EntityTableDisplayProjection` atomically contains ordered visible column
descriptors, stable row keys, canonical full cell text, every locally filtered
row in current sort order, and the current-page half-open bounds. Select rows
explicitly with `rows(EntityTableProjectionScope::CurrentPage)` or
`rows(EntityTableProjectionScope::AllFiltered)`.

The projection follows the same data signal, typed sort, effective page size,
page, hidden-column set, user column order, reactive labels, row-key callback,
and canonical `EntityColumn::text` callbacks as rendering. Compact layout does
not create a second projection. Clipping, badges, icons, and `render_with`
markup never replace canonical export text. Action columns are excluded by
default and require `EntityTableActionColumnPolicy::Include` to opt in.

The callback is an observation seam, not an export engine: LDUI does not add a
dataset identity, initiate a download, select CSV, or decide authorization and
domain export policy. Store the latest snapshot in caller state and read it
inside the colocated `toolbar_actions` callback.

## Hybrid aligned filters

Prefer `EntityColumnFilter::text(...)` and `EntityColumnFilter::select(...)`
for ordinary controlled filters. Both require a document-unique base control
ID, a reactive localized label, the accepted value signal, and a replacement
callback. `text` also accepts a reactive placeholder. `select` accepts a
reactive localized all-option label and reactive
`EntityColumnFilterOption` value/label pairs, so locale replacement never
changes the stable submitted value.

The supplied value is the only source of truth. A proposal that the caller
rejects is immediately replaced in the DOM by the accepted value; external
replacement, dataset reset, and clear all flow through that same signal and
callback. Active metadata is derived from the accepted value and cannot drift
from the rendered control. The header uses the supplied base ID and the
responsive copy uses its deterministic `-responsive` suffix, avoiding duplicate
IDs when presentation changes.

Use `EntityColumnFilter::new("stable_column_id", renderer)` as the compatible
escape hatch for unusual controls. Add `.with_responsive(label, active,
on_clear)` when a custom filter must remain usable in compact layouts. Custom
markup, IDs, active state, and clear semantics remain caller-owned.

At desktop width, `EntityColumnFilters` renders one light-blue second-header
cell for every visible column and places the control only in its target cell.
Below `lg`, the same renderer instance moves to a labelled responsive panel;
the hidden desktop row contains no duplicate controls or IDs. An active filter
disables its column's hide item and announces that reason. If controlled or
legacy preferences nevertheless arrive with that active column hidden, its
single control appears in the responsive panel at desktop width as well, with
the caller-owned clear intent. Inactive hidden filters return without reset
when their column is restored.

Reorder and visibility use the same ordered descriptor list as the header and
body, so tracks cannot drift. Filter cells stop pointer, keyboard, and
pointer-down propagation; selecting a value cannot sort, resize, or activate a
row.

Keep global search and controls that do not map to one column in the utility
`FilterBar`. The complete pattern has exactly one Reset and one Save as Default
there; it does not duplicate column controls above the table. See
[`client-snapshot-list.md`](../patterns/client-snapshot-list.md).

## Viewport-fit paging

Fixed 25/50/100 paging remains the default. Opt in with
`EntityTableViewportFit::fill_parent()` when the table's parent has a definite
height, or `EntityTableViewportFit::max_height("...")` when the table should own
that CSS height budget. The table measures its real header (including the
filter band) and first body row, then derives a presentation-only row capacity.
The rows-per-page select continues to show and persist the caller's configured
fallback; measurement never writes a synthetic preference.

Resize observation and reactive table inputs trigger a coalesced remeasurement,
so browser-height, localized header/filter copy, column visibility, and row-height
changes settle without a reload. If fewer than the policy's minimum usable rows
fit, the configured page size is rendered and the table region becomes the one
internal vertical scroller. The toolbar and pager stay outside that region, and
page clamping uses the effective capacity so a resize cannot leave a stale page
index. `with_min_rows(...)` changes the fallback threshold without changing the
configured page size.

## Visual hierarchy and shell stability

`EntityTable` emits the canonical `#004578` header with white content, the
aligned `#e5f1fb` filter band with dark content, and a collapsed faint `#e0e0e0`
grid across every table cell. Zebra striping is off unless requested. Its fixed
layout and declared `colgroup` keep column tracks stable; sorting updates rows,
markers, and accessibility state in place without replacing the table/header
nodes or moving the shell.

These are generated semantic utilities, not demo-only CSS. Every consuming
Tailwind build must import `leptos-daisyui-rs/styles/tokens.css` and scan
`leptos-daisyui-rs/src/**/*.rs`, with paths resolved from its own `input.css`.
See the [DataTable consumer CSS setup](./data_table.md#consumer-inputcss).

## Row-action focus recovery

Wrap repeatable action controls in `EntityRowAction action_id="..."`. When a
focused source row is actually removed within the same `focus_scope`, the
table uses its real filtered/sorted/paged order and visible position to focus
the same enabled, visible action on the row that moved into that position. A
last-row removal clamps to the preceding visible row.

If filtering or paging merely hides a row that remains in `source_data`, if
the matching action is absent/disabled/hidden, or if no neighbor remains,
focus goes to the named programmatically focusable table region. Dataset or
access-generation changes clear recovery without cross-focusing. A declined
or failed action that leaves the row present retains native focus, and focus
already moved by the user is never stolen. Consumers must not reproduce this
with DOM queries or source-order guesses.

## Controlled single-row selection

`selection` wires one proposal-first, single-row selection to `EntityTable`,
keyed by the table's mandatory `row_key`. It mirrors `ServerTableSelection`'s
shape (`ldui-4lp`): the caller supplies the accepted key through
`selected_key`, and a plain click or keyboard Enter/Space on a row emits one
proposed replacement key without changing what is rendered. `EntityTable`
never optimistically paints a proposed key -- a rejected or delayed proposal
leaves `aria-selected` and the selected-row background aligned with
whatever key the caller currently supplies. Ctrl/Meta/Shift gestures
neither propose nor activate; selection is deliberately single-select, not
a range or toggle.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::EntityTableSelection;

let selected_key = RwSignal::new(Option::<String>::None);
let selection = EntityTableSelection::controlled(
    selected_key.into(),
    Callback::new(move |proposed: Option<String>| selected_key.set(proposed)),
);
// Pass `selection` through EntityTable's `selection` prop.
```

`selection` is independent of `on_row_activate`; both may be supplied
together, and a plain click or Enter/Space fires both in that case.

`aria-selected` is emitted only when `selection` is configured at all --
never based on general row interactivity. An `on_row_activate`-only table
(no `selection`) renders no `aria-selected` attribute and no selected
background on any row, exactly as it did before this prop existed: it has
no selection concept to report, and stamping `aria-selected="false"` on
every row would wrongly claim it does. A table with `selection` configured
emits `aria-selected` on every row, `"true"` or `"false"`.

Matching is exact stable-key equality against `row_key`, with no positional
fallback. A selected key with no matching row on the current page --
because sorting, filtering, paging, a dataset swap, or removal moved or
deleted it -- simply selects nothing until the caller supplies a key that is
visible again; `EntityTable` never falls back to selecting whatever renders
in the same position. `row_emphasis` below uses the same fail-safe shape.

## Typed summary-row emphasis

`row_emphasis` classifies each row into `EntityRowEmphasis` -- `Standard`,
`Summary`, `Muted`, or `Attention` -- a narrow, framework-owned enum, never
an unrestricted class-string hook. `EntityTable` owns every token, stroke
width, and forced-colors rule a variant applies, identically across the
wide and compact presentations that share one `<tr>`; the caller supplies
only the classification predicate, so no per-column renderer needs to
change.

```rust,no_run
use std::rc::Rc;
use leptos_daisyui_rs::components::EntityRowEmphasis;
# struct Row { status: String }
let row_emphasis = Rc::new(|row: &Row| match row.status.as_str() {
    "Total" => EntityRowEmphasis::Summary,
    "Archived" => EntityRowEmphasis::Muted,
    "Overdue" => EntityRowEmphasis::Attention,
    _ => EntityRowEmphasis::Standard,
});
// Pass `row_emphasis` through EntityTable's `row_emphasis` prop.
```

Every variant is presentation-only and text/border only -- `Summary` adds
bold weight plus a top rule, `Muted` holds text at the framework's
AA-audited `text-base-content/75` (never a lower, axe-failing opacity), and
`Attention` adds warning-toned bold text. **No variant sets
`background-color`.** That is deliberate: `selection` paints `bg-base-200`
on a selected `<tr>` independently, and `zebra` paints alternating row
backgrounds via `table-zebra` on the ancestor `<table>`. Because emphasis
never touches background, all three compose freely instead of racing for
the same CSS property -- a selected `Summary` row keeps its selected
background alongside its bold text and top rule, and a zebra-striped
`Muted` row keeps its stripe alongside its reduced-contrast text.

Classification is a pure function of the row's own content, computed fresh
for whatever row is currently rendered at a position -- so it automatically
follows a row across sorting, filtering, and paging rather than pinning a
look to a position. `Standard` (the default when a table has no
`row_emphasis` classifier at all, or when the row being classified is
absent -- the same fail-safe as selection above) renders identically to a
table that predates this prop: no extra class, no
`data-entity-row-emphasis` attribute on any row.

## Preference ownership

### Controlled: recommended for governed persistence

Controlled mode makes the consumer the only source of truth. Every table UI
operation emits one normalized, complete `EntityTablePreferences` replacement.
The component performs no browser-storage I/O and keeps rendering the supplied
signal until the consumer accepts a replacement.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferences,
};

let preferences = RwSignal::new(EntityTablePreferences::new(3));
let ownership = EntityTablePreferenceOwnership::controlled(
    preferences.into(),
    Callback::new(move |replacement| {
        // Validate/save through the page's governed persistence path, then
        // accept the returned value. This in-memory assignment illustrates
        // the acceptance step.
        preferences.set(replacement);
    }),
);

// Pass `ownership` through the EntityTable `preference_ownership` prop and
// keep `preference_version=3` aligned with the DTO schema.
```

Normalization is pure and deterministic. A schema-version mismatch resets to
defaults. Ordered columns keep the first occurrence of each known id and append
missing declarations in system order. Sort clauses likewise keep the first
occurrence of each sortable known id while removing duplicates, unknown ids,
and non-sortable columns. Required ids are removed from `hidden_columns`, and
widths use the shared DataTable bounds.

## Ordered columns and multi-column sort

`EntityTablePreferences::column_order` is the complete ordered list of stable
column ids. The table renders wide headers, wide cells, and the default compact
row in that same order. The chooser exposes an ordered list with named “move
earlier” and “move later” buttons. Boundary buttons are disabled, and focus is
restored to the corresponding control after a move.

`EntityTablePreferences::sort` is an `EntitySort`. The historical public
`System`, `Ascending`, and `Descending` variants remain available for
single-column source code; `Multiple` carries two or more ordered clauses:

```rust,no_run
use leptos_daisyui_rs::components::{EntitySort, EntitySortColumn};

let sort = EntitySort::multiple([
    EntitySortColumn::ascending("status"),
    EntitySortColumn::descending("client"),
]);
```

Clauses are compared lexicographically, with stable dataset order breaking full
ties. Text keys are extracted once per row for each active text clause rather
than allocated inside the comparison loop.

For values whose displayed text is not their true order, attach a typed key:

```rust,no_run
use leptos_daisyui_rs::components::{EntityColumn, EntityNullOrder};

let count = EntityColumn::text("count", "Count", |row: &Row| row.count.to_string())
    .sortable_by_key(|row| row.count);
let date = EntityColumn::text("date", "Date", |row: &Row| row.date_label.clone())
    .sortable_by_key(|row| row.date); // any date/time type implementing Ord
let owner = EntityColumn::text("owner", "Owner", |row: &Row| row.owner_label())
    .sortable_by_optional_key(EntityNullOrder::Last, |row| row.owner_sort_key());
```

`sortable_by_key` accepts signed or unsigned integers, strings, date/time
types, tuples, and domain newtypes implementing `Ord`. Each key is extracted
once per row, equal keys retain source order, and ordered multi-sort uses the
same prepared keys. `sortable_by_optional_key` requires `First` or `Last`;
that null placement is absolute in both ascending and descending value order.
The existing normalized text fallback and `sortable_by` two-row comparator
remain available for ordinary text and domain-specific comparisons.

A plain header activation keeps the compatibility single-sort cycle: ascending,
descending, then system order. Shift+activation edits one clause without
discarding the others: absent becomes ascending, ascending becomes descending,
and descending is removed. Markers and accessible labels expose every clause's
direction and one-based priority. Only the primary header carries `aria-sort`,
as required for a table with one primary ordering; a polite live summary
announces the complete clause sequence.

Resizable headers expose focusable vertical separators with `aria-valuemin`,
`aria-valuemax`, and live `aria-valuenow`/`aria-valuetext` semantics. Left and
Right resize by 16 pixels, while Home and End select the allowed minimum and
maximum. Keyboard and pointer changes use the same shared bounds; in controlled
mode each completed keyboard action emits one normalized preference
replacement and performs no browser storage I/O. The shared `DataTable` header
uses the same keyboard resize math and separator semantics.

### Interaction and accessibility release evidence

A green screenshot, style audit, or layout audit is Layer A evidence; it does
not prove that an interactive table is keyboard-operable or exposes accurate
semantics. Before approving a table change, enumerate every supported operation
and attach browser evidence for each one:

| Operation | Required browser evidence |
|---|---|
| Sort | Real Tab plus Enter/Space and Shift-modified activation; one activation per key; accurate current/next-action labels; primary-only `aria-sort`; priorities, live summary, rendered rows, and controlled model agree. |
| Resize | A visibly focusable, uniquely named vertical `separator`; ordered `aria-valuemin <= aria-valuenow <= aria-valuemax` plus value text; real Left/Right/Home/End input; no scroll or sort activation; rendered width and controlled model agree. |
| Reorder | Named earlier/later controls include position; boundary state is disabled; focus follows the moved column or its enabled opposite at a boundary; DOM and model order agree. |
| Visibility | Keyboard-operable chooser state agrees with rendered columns; required-column and last-visible guards remain enforced. |
| Paging | Current page and boundary-disabled states are exposed; keyboard button activation changes the row range and rendered rows exactly once. |
| Compact rows | Compact content follows the same normalized column order and preserves names/actions without duplicate hidden focus targets. |

The shared minimum-width helper caps every public `min_width`, including
`u32::MAX`, at the global maximum. That keeps the ARIA range ordered and every
direct `f64::clamp` range valid. Prove the extreme input in a pure regression
test and prove focus plus key operation in the real DOM; either test alone is
insufficient. An axe run complements these checks but cannot prove keyboard
operation. Keep an inject/catch/revert negative control for a key behavior or
focusability assertion so a green journey demonstrates detection, not merely
execution.

### Uncontrolled without persistence

This is the default when neither ownership nor `storage_key` is supplied. The
component owns an in-memory signal for its lifetime and never reads or writes
`localStorage`.

```rust,no_run
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
};

let ownership = EntityTablePreferenceOwnership::uncontrolled(
    EntityTablePreferencePersistence::Disabled,
);
```

### Legacy local storage

Existing callers can keep the historical prop unchanged:

```rust,ignore
<EntityTable
    // required snapshot props omitted
    storage_key="no-hires"
    preference_version=2
/>
```

It resolves to uncontrolled `LegacyLocalStorage` with the existing
`ldui-entity-table:<storage_key>` key. Supplying both `storage_key` and
`preference_ownership` fails closed instead of silently allowing a controlled
table to perform browser I/O.

Legacy serialized single-sort values (`System`, `Ascending`, and `Descending`)
remain readable. The next write uses the canonical sort-clause array, and a
legacy payload without `column_order` normalizes to the declared system order.

This schema addition is an intentional source migration for consumers that use
an `EntityTablePreferences` struct literal or exhaustively match `EntitySort`.
Add `column_order: Vec::new()` to legacy literals, and handle
`EntitySort::Multiple { clauses }` (or consume `sort.clauses()`) in exhaustive
matches. Vendor the framework and those consumer changes atomically.

## Migration path

For an existing `EntityTable` using `storage_key`, preserve that prop until the
consumer has a governed persistence endpoint. Then load the saved preference
DTO into a signal, pass controlled ownership, handle each full replacement in
the page, and remove `storage_key`. Keep `preference_version` stable unless the
preference schema actually changes.

Existing `components::DataTable` call sites remain compatible. Migrate a call
site to `EntityTable<T>` only when it represents a complete snapshot and its
required DataTable-only features have typed equivalents. Existing server-owned
pages migrate directly to `ServerDataTable`; they must not pass one fetched page
through either client table.

A page-local raw `EntityTable` kept only to reach `page_reset_key`,
`viewport_fit`, `toolbar_actions`, `on_display_projection`/
`projection_action_columns`, or `column_chooser_trigger` -- because
`SnapshotTablePage` did not yet expose them -- should migrate back onto
`SnapshotTablePage` and `SnapshotEntityTableConfig`'s typed builders for the
same names (`ldui-myhh` / `ldui-5ano`). See
[`doc/patterns/client-snapshot-list.md`](../patterns/client-snapshot-list.md#behavior-only-entitytable-passthroughs)
for the full builder table and a worked example.

## Verification

The focused inner and browser lanes are:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
```

The browser lane checks real keyboard reorder with boundary-safe focus
restoration, real Enter/Shift+Enter and Shift-click multi-sort, real keyboard
resize with controlled-model readback, paging, chooser behavior, controlled
preference mount behavior without browser storage reads or writes, compact
rendering, row/action activation, stable page-size/dataset control identities,
a vendored axe-core audit with the chooser open, style oracles, and the
`client-snapshot` ownership marker. Native tests retain explicit coverage of
the legacy local-storage compatibility path and generation-bound row
projections.
