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
| `viewport_fit` | Optional `EntityTableViewportFit`; adds an explicit `Auto` rows-per-page choice that derives visible row capacity from a definite parent or CSS height, without changing the saved page-size preference. |
| `pagination: EntityTablePagination` | `Paged` (default) or `ConstrainedScroll`, which renders every row and suppresses the footer for a table bounded by its container (`ldui-tmoz`). |
| `draft_row: EntityDraftRow<T>` | Optional inline draft-row and per-row editing (`ldui-ff2f`). Absent, the table renders no `+`, has no edit mode, and emits no extra DOM. |
| `compact_row: EntityCompactRow<T>` | Default, static, or reactive single-cell renderer used at compact breakpoints without duplicating rows. |
| `column_filters: EntityColumnFilters` | Controlled one-to-one filter controls aligned beneath stable desktop columns. |
| `source_data` | Optional authoritative source membership used only for safe post-removal focus recovery. |
| `focus_scope` | Optional opaque dataset/access generation; recovery never crosses a change. |
| `preference_ownership` | Controlled or uncontrolled preference policy. |
| `storage_key` | Legacy local-storage compatibility prop; mutually exclusive with `preference_ownership`. |
| `page_size_control_id` | Optional stable caller-owned DOM ID for the rows-per-page select, which renders in the footer row (see below). |
| `toolbar_actions` | Optional caller-rendered table utilities placed before the framework-owned column chooser, in the top toolbar. |
| `on_page_size_resolved` | Optional callback receiving the resolved `EntityPageSize` whenever it changes, including after a `viewport_fit` resize. |
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

## Rows per page: one resolved value (ldui-5p06)

A viewport-fitted table used to render a fitted row count while its
rows-per-page control and pager described a different size -- an Office Setup
comparison showed the control reading `25` over a five-row body advertising
four pages. Auto-fit is now an explicit *choice*, and there is exactly one
resolved page size per render.

**The type.** `EntityPageSize` pairs a mode with the row count actually
rendered. Its fields are private and both constructors clamp the count, so a
row count with no mode, a mode with no row count, and a zero row count are all
unrepresentable rather than merely avoided. It is produced only by
`resolve_entity_page_size(intent, auto_available, configured_rows,
measured_rows)`, the single place the stored intent and the transient
measurement are combined:

| Inputs | Result |
|---|---|
| No `viewport_fit` policy | `Fixed(configured)` -- a stored `Auto` cannot label a table that never measures. |
| `Auto`, a measurement exists | `Auto(measured)`. |
| `Auto`, before the first measurement | `Auto(configured)` -- what the first paint genuinely renders. |
| `Fixed` | `Fixed(configured)`, whatever was measured. Choosing `25` renders up to 25 rows and the region scrolls. |

The rendered body, the `{start}-{end} of {total}` summary, the rows-per-page
control's selected value and label, and the pager's page count all read that
one value. The table exposes it as `data-entity-effective-page-size` (rows)
and `data-entity-page-size-mode` (`auto` / `fixed`).

**The control.** With `viewport_fit`, the select's options are `Auto`, 25, 50,
100 and `Auto` is the default. The `Auto` option's *value* is `auto` -- stable
across resizes, so a resize never moves the user's selection -- while its
*label* carries the fitted count from
`EntityTableTexts::rows_per_page_auto` (default `"Auto ({rows})"`, so
`Auto (5)`). Localize that key like any other; it is never hardcoded English.
Without `viewport_fit` the option list and behavior are exactly as before.

**Controlled state.** `EntityTablePreferences` gained
`page_size_mode: EntityPageSizeIntent` (`Auto` | `Fixed`, `#[serde(default)]`
so pre-existing payloads keep today's auto-fit behavior). A satellite persists
**only** that intent and the numeric `page_size` -- both explicit user
choices, both emitted together in one normalized replacement by the ordinary
preference-change callback. It must **not** persist the measured row count:
that is transient presentation state belonging to one viewport at one moment.
When a consumer needs the effective size (for a caption, an export footer, or
telemetry), read it from `on_page_size_resolved`; no consumer should measure
the DOM or keep duplicate pagination state.

```rust
<EntityTable
    // ...
    viewport_fit=EntityTableViewportFit::fill_parent()
    preference_ownership=EntityTablePreferenceOwnership::controlled(
        preferences.into(),
        Callback::new(move |replacement| {
            // Persist the explicit choice: `page_size` + `page_size_mode`.
            preferences.set(replacement);
        }),
    )
    on_page_size_resolved=Callback::new(move |page_size: EntityPageSize| {
        // Transient. Display it; never store it.
        effective_rows.set(page_size.rows());
    })
/>
```

## Toolbar and footer placement (ldui-z0n1)

`EntityTable` follows the established desktop table grammar: toolbar actions
and the column chooser render **above** the table; pagination metadata
renders **below** it. The rows-per-page control lives in the footer, not the
top toolbar -- the footer's DOM order is Rows per page, then the
`{start}-{end} of {total}` row-range text, then Previous/page-number/Next.
The top toolbar (`toolbar_actions` and the column chooser) never contains the
page-size control. This is purely a placement move: the page-size select's
`id`/`name` derivation (`page_size_control_id`, or the process-unique
default from `next_entity_page_size_id` when omitted, ldui-kl55), the
`label[for]` association, the controlled/uncontrolled preference callback,
and localized `EntityTableTexts::rows_per_page` copy are all unchanged.

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

### `.numeric()` and `.identifier()`

`EntityColumn::kind` (`EntityColumnKind`, default `Text`) names two common
presentations so callers stop hand-writing the same Tailwind utilities at
every call site -- mirroring `DataTable::Column::numeric`/`identifier`
(`ldui-lrig`) after the same duplication was measured in `EntityTable`'s own
`tabular_numbers: bool` plumbing (`ldui-no94`):

- `.numeric()` -- tabular (monospaced) figures plus right alignment. Equivalent
  to `.tabular_numbers().align_end()`. It does **not** imply a numeric sort
  key: `EntityColumn` is typed over its row and already has an exact way to
  say "sort this numerically" -- `.sortable_by_key(...)` -- unlike
  `DataTable::Column`, which is untyped (`HashMap<String, String>` rows) and
  has to re-parse the displayed text at sort time via `SortAs::Number`.
  Re-deriving a numeric comparator from `.numeric()`'s own presentational
  text would be strictly less correct than a caller's typed extractor, and
  making it conditional on builder call order would reintroduce the exact
  kind of silent disagreement this bead removed. Pair `.numeric()` with
  `.sortable_by_key(...)` explicitly.
- `.identifier()` -- the theme's declared monospace face (`font-mono`) for
  ids, codes, hashes, and SKUs. Does not change alignment or sorting.
- `.tabular_numbers()` -- the lower-level primitive `.numeric()` is built
  from: tabular figures only, alignment untouched. `EntityColumn` has no raw
  CSS class escape hatch (see below), so this remains the only way to express
  a centered or left-aligned column that still wants tabular figures (a
  centered date column, for example).

As with any builder, later calls win: `.numeric().align_center()` keeps the
tabular figures but overrides the implied right alignment.

`EntityColumn` never exposes a `with_class`-style raw CSS override -- every
visual decision here is a narrow, framework-owned enum (alignment, row
emphasis, and now this kind), by design. So there is no
`effective_class`/wholesale-replace pairing to document the way
`DataTable::Column::with_class` documents against `Column::numeric`/
`identifier`: a kind's contributed class is always an additive token combined
with the already-independent `alignment` field, and a caller "overrides" it
the same way any builder call overrides an earlier one -- by calling it later.

An `.identifier()` column's `font-mono` currently trips the style audit's
typography-family check regardless of who applied it -- `StyleProfile` records
one dominant family per page (`ldui-kq9w`, a known upstream gap, not a reason
to avoid `.identifier()`).

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

Prefer `EntityColumnFilter::text(...)`, `EntityColumnFilter::select(...)` and
`EntityColumnFilter::date(...)` for ordinary controlled filters — a consumer
should never hand-roll a private replacement for one of these. All three
require a document-unique base control
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

### Controlled date filter (ldui-lx5t)

`EntityColumnFilter::date(column_id, control_id, label, value, invalid_hint,
on_change)` is the framework-owned date control. It carries the same contract
as `text` and `select` — document-unique base ID, `-responsive` suffix for the
compact copy, reactive localized label, caller-owned value, proposal-only
`on_change` — and adds three things that a text field spelling `YYYY-MM-DD`
cannot give you.

**It is a native `date` input.** The platform picker, its keyboard operation
and its locale-aware *presentation* come for free, while the value stays the
machine `YYYY-MM-DD` text a URL query, a saved view and a server all already
speak. Add `@source inline("input input-xs input-error");` to your
`input.css`.

**Its proposal is typed.** `on_change` receives an `EntityDateFilterProposal`
carrying the complete resulting `raw` text, that text already interpreted as
an `EntityDateBound`, an `EntityDateFilterCause` (`Edited` or `Cleared`), and
the `column_id` / `control_id` scope stamp — the same shape
`EntityTableSelectionProposal` uses, so a caller wiring several date filters
through one callback routes on identity rather than on call order. `control_id`
is always the caller's own base ID, never the placement-suffixed DOM ID.

**An unreadable value is announced, not swallowed.** A native picker cannot
produce a bad value, but a restored URL query, saved view or migrated
preference can. The browser blanks a value a `date` input cannot parse, so
without an explicit error state an unreadable constraint would look exactly
like *no* constraint while still hiding every row. When `value` is neither
empty nor a real calendar day the control adds `aria-invalid`, the daisyUI
`input-error` treatment, a `data-entity-filter-invalid` hook, and
`invalid_hint` as its accessible description. It also stays **active** while
unreadable, so the responsive panel keeps offering the clear action that
recovers from it.

Every control carries `data-entity-filter-control="<column_id>"`,
`data-entity-filter-placement="header" | "responsive"` and
`data-entity-filter-kind="text" | "select" | "date"`. Locate a filter by those,
never by position.

#### What it compares: `EntityDateFilter`

The filter row is a control surface; the table filters nothing itself. Apply
the constraint with `EntityDateFilter`, the framework-owned predicate, against
an `Option<EntityDate>` your own row accessor produces:

```rust,ignore
let cutoff = EntityDateFilter::parse_on_or_before(&cutoff_text.get());
let visible: Vec<Matter> = matters
    .iter()
    .filter(|matter| cutoff.matches(matter.arrived_on))
    .cloned()
    .collect();
```

`EntityDate` is a timezone-free civil date. Collapsing a timestamp to a
calendar day is *your* job, because "arrived on or before 4 August" is a claim
about the calendar the user is reading, not about a point on the UTC timeline.
Never filter on rendered cell text: that is display copy, and its meaning
changes with the locale, the column renderer or a format callback.

Both range ends are **inclusive** — the bound variant is named
`EntityDateBound::Inclusive` so the question cannot be answered by experiment.
`EntityDateFilter::status()` names each outcome, and `matches` follows it:

| State | `status()` | `matches` |
| --- | --- | --- |
| Both ends empty | `Unconstrained` | everything, **including rows with no date** — the user has not filtered |
| One end bounded (half-open) | `Constrained` | compares that end only; a row with **no date does not pass** |
| Both bounded, start after end | `Impossible` | nothing — deliberately, and reportably |
| Either end unreadable | `Invalid` | nothing; `invalid_input()` returns the offending text |

`constrains()` is the active-filter signal: `Impossible` and `Invalid` both
count as active, because they are excluding everything and the user must be
able to see and clear that.

The date surface is ANDed with free-text search and with `filterable()` column
filters, exactly like every other filter: each can only remove rows. Under row
grouping it is an ordinary child-row filter, so a group whose children it
removes loses its heading with them (see
[Controlled accessible row groups](#controlled-accessible-row-groups-ldui-iyfa)).

A two-ended range is expressed by holding two values and calling
`EntityDateFilter::parse_bounds(start, end)`; `EntityColumnFilter::date`
itself renders one control, which is the single-cutoff shape the aligned filter
row has room for.

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

The stable `colgroup` and its forced content `min-width` are desktop-only
geometry (ldui-ibjk). Below the `lg` breakpoint the table switches to its
compact single-cell row renderer, and the `<colgroup>`/`min-width` are omitted
entirely rather than merely hidden — a hidden `<td>` (`lg:hidden`) does not
stop its `<col>` track from claiming width, so the desktop column widths would
otherwise force the compact card region wider than its viewport and require
horizontal scrolling even though nothing is off to the side. Compact mode
therefore fits its containing block; desktop keeps its exact prior geometry
and horizontal scrolling unchanged.

These are generated semantic utilities, not demo-only CSS. Every consuming
Tailwind build must import `leptos-daisyui-rs/styles/tokens.css` and scan
`leptos-daisyui-rs/src/**/*.rs`, with paths resolved from its own `input.css`.
See the [DataTable consumer CSS setup](./data_table.md#consumer-inputcss).

## Constrained-scroll tables (ldui-tmoz)

A small table inside a section card — Work Types, Case Types, a settings list —
is bounded by the card, not by a page size. It scrolls. In that shape the
rows-per-page select is furniture the user cannot act on, and a footer reading
`Showing 1-8 of 17` beside no control is worse than no footer at all.

```rust,no_run
view! {
    <EntityTable
        data=work_types
        columns=columns
        row_key=row_key
        dataset_identity="work-types"
        viewport_fit=EntityTableViewportFit::fill_parent()
        pagination=EntityTablePagination::ConstrainedScroll
    />
}
```

`ConstrainedScroll` resolves the page size to the row count, so there is
exactly **one page holding every row**. The footer is suppressed entirely —
no rows-per-page control, no pager, no row-range summary — because with one
page none of it would say anything true.

### One mode, not three flags

No select, no pager, all rows rendered are not independent choices a consumer
should have to combine correctly; they are one shape, so they are one value.
Combining them by hand is how you end up with a pager that pages nothing.

### It renders every row

There is no virtualization. `ConstrainedScroll` is for **bounded** collections
— a settings table, not an unbounded result set. A table that can grow without
limit should stay `Paged` and let the user page it.

> **Decision record.** The alternative considered was a narrower
> `show_page_size_control=false` flag that hid only the select and left the
> pager. It was rejected because the consumer's tables scroll rather than page,
> so the pager would remain inert furniture. The mode is additive and opt-in,
> so reversing this costs a deprecation of one enum variant and nothing else —
> `Paged` is the default and every existing table is unaffected.

## Inline row creation and editing (ldui-ff2f)

Opt in with `draft_row`, mark the columns that accept input, and designate
exactly one existing action column with `.inline_edit_host()`. Omitting the prop
leaves the table exactly as it was: no `+`, no Edit action, no edit mode, and no
extra DOM. `.allow_row_edit(true)` adds the existing-row entry point; leave it
false when the table should create rows but never update them.

```rust,no_run
let columns = vec![
    EntityColumn::text("name", "Name", |r: &WorkType| r.name.clone())
        .editable(EntityCellEditor::text(
            |r: &WorkType| r.name.clone(),
            |r: &mut WorkType, value| r.name = value,
        )),
    // No `.editable(...)`: stays read-only even inside the live row.
    EntityColumn::text("created", "Created", |r: &WorkType| r.created.clone()),
    EntityColumn::action("actions", "Actions", |_r: &WorkType| String::new())
        .render_with(render_work_type_actions)
        .inline_edit_host(),
];

view! {
    <EntityTable
        data=rows
        columns=columns
        row_key=row_key
        dataset_identity="work-types"
        draft_row=EntityDraftRow::new(WorkType::blank, on_commit)
            .with_texts(draft_texts)
            .allow_row_edit(true)
    />
}
```

While the table is idle, the marked host keeps the consumer-rendered actions
and appends the framework Edit control. For the one live row, the host becomes
the framework-owned Save/Cancel surface; other consumer actions in that cell
are unavailable until the session ends. The draft row uses that same host for
Save/Cancel, so the table never appends a synthetic action column.

### One mode, exclusive by construction

While a row is live, **the accepted table is frozen and inert**: every other row
has `aria-disabled="true"`, all non-live descendants leave the tab order, and
sort/filter/page/column controls that could move or hide the live row are
locked. Only the local working-row overlay remains active. That is not
decoration: it is the invariant the whole feature rests on. A second entry
point firing while a row is live is *refused*
(`EntityEditDisposition::IgnoredBusy`), so two simultaneously editable rows are
unrepresentable rather than merely discouraged.

Inert rows leave the tab order, which deliberately differs from the
`aria-disabled` treatment of the empty-table header checkbox. That control
stays tabbable so a keyboard user hears *why* it is inert — right for one
control, wrong for N rows, since seventeen tabbable disabled rows would make
Tab-to-Save walk the whole table first.

### The consumer owns persistence

Save hands over the edited row and a `resolve` handle, then **waits**:

```rust,no_run
let on_commit = Callback::new(move |commit: EntityDraftCommit<WorkType>| {
    spawn_local(async move {
        match save_work_type(commit.row).await {
            Ok(()) => commit.resolve.run(EntityEditOutcome::Accepted),
            Err(e) => commit.resolve.run(EntityEditOutcome::Rejected(e.to_string())),
        }
    });
});
```

Until `resolve` runs the table stays in `Committing`: Save is disabled (no
double submit) and the row cannot change underneath the write. `Rejected`
returns to editing **with the user's input intact** and the message available;
typing then clears it, because it described a value the user has since changed.

`Accepted` ends the session and discards the working overlay. The saved draft or
updated row re-enters through your normal data flow rather than being injected
by the table.

### Refreshes wait until editing is complete

The component never publishes refreshed input while `Drafting` or `Committing`.
It keeps the accepted rows, columns, dataset identity, and revision frozen as
one coherent snapshot and stores only the latest arriving input envelope as
pending. The table therefore cannot change underneath either a draft or an
existing-row edit; repeated refreshes coalesce instead of replaying a backlog.

Cancel or Escape discards the working row, publishes the latest pending
snapshot atomically (if present), and only then re-enables the table. An
`Accepted` commit follows the same release order. A `Rejected` commit returns to
editing with the user's input intact and leaves the pending refresh queued,
because the edit is not complete. With no pending refresh, cancel or acceptance
simply returns to the frozen accepted snapshot.

### Keyboard and responsive behavior

The toolbar `+` and a row's Edit control both focus the first visible enabled
editor. At either wide or compact width, Tab follows declared column order and
then reaches Save and Cancel; the desktop controls hidden at compact width are
never focus targets. During `Drafting`, Escape discards the overlay and restores
focus to `+`, the surviving row Edit control, or the named table region. During
`Committing`, the controls stay locked until the consumer resolves the write.

The compact live row is a single cell spanning the visible column count. It
uses the same reducer-backed editors and actions as the wide row, adds a visible
label for every editor, and keeps the consumer's compact summary while idle.

### Observability

`data-entity-edit-phase` on the table root reports `idle` / `drafting` /
`committing`, and is absent entirely on a table that did not opt in. Prefer it
to inferring state from which controls happen to be disabled. The draft row is
`data-entity-draft-row`; all live editors carry
`data-entity-edit-input="<column-id>"`, with the draft-only compatibility marker
`data-entity-draft-input`. Row actions expose `data-entity-row-edit-state`, and
the toolbar entry point remains `data-entity-draft-add`.

Proof: `cargo xtask test-entity-draft-row`, whose fixture mounts an opted-in
and a plain table on one document so every claim carries a negative control.

### Not yet supported

- **Grouped tables**: the draft row renders in the ungrouped `<tbody>` only.
  Which group a new row belongs to is a question the framework should not
  answer for you.

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

### Typed focus requests for external mutations (ldui-o0iw)

Recovery above begins with a `focusin` inside the table's own region, so it
serves only a mutation the table observed. An editor panel *beside* the table
whose Delete button removes the selected row is destroyed along with the row it
deleted: focus falls to `<body>` and there is no record to recover from. The
`focus_request` prop is how that page says "focus this row" without querying
DOM this crate owns.

```rust,ignore
let request = RwSignal::new(Option::<EntityFocusRequest>::None);

// After the central API accepted the delete and re-read:
request.set(Some(EntityFocusRequest::row(next_id, scope, successor_key)));
// ...or, to land on a named action inside that row:
request.set(Some(EntityFocusRequest::row_action(next_id, scope, key, "open")));

view! {
    <EntityTable
        // ...
        focus_scope=scope
        focus_request=request
        on_focus_request_resolved=Callback::new(move |resolved: EntityFocusRequestResolution| {
            log::debug!("focus request {} -> {}", resolved.request_id, resolved.outcome.as_str());
        })
    />
}
```

The contract:

- **Resolved against the presentation, never source order.** The request names
  a stable row key and is answered against the rows the table is painting right
  now -- after filtering, sorting, paging, grouping and collapse.
- **Documented fallback, never a positional guess.** A row that is filtered
  away, paged away, removed, collapsed or not focusable (a display-only table
  has no focusable rows), or a named action that is absent or disabled, focuses
  the named table region and reports
  `EntityFocusRequestOutcome::TableRegion`. It never focuses "whatever row now
  sits where that one used to".
- **Stale scopes are rejected.** A `scope` the table has since left reports
  `StaleScope` and moves nothing.
- **One id, one application.** A signal that keeps reporting an honored request
  cannot take focus back from the user later; bump `EntityFocusRequest::id` for
  each new request.
- **Focus the user moved is not stolen.** If focus rests on another meaningful
  target when the replacement paints, the request reports `Declined`. `<body>`
  is not a meaningful target -- it is where focus lands when the element that
  had it was destroyed, which is the case this exists to repair.
- **It survives the element being recreated.** The row is re-queried by stable
  key on the next animation frame, and once more on the frame after that,
  before falling back; no element reference is held across the replacement.
- **A request states that a mutation was accepted.** Issue nothing for a failed
  or declined mutation: there is nothing to move focus to, and the editor that
  still owns focus should keep it.

Internal row-action recovery is untouched by all of this: the two paths read
none of each other's state.

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

## Controlled checkbox multi-selection (ldui-nz6d)

`multi_selection` is the bulk-action counterpart to `selection`. Supplying it
renders a leading checkbox column plus a header checkbox, keyed by the same
mandatory `row_key`.

```rust,no_run
use std::collections::BTreeSet;
use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    EntityTableMultiSelection, EntityTableSelectionProposal,
};

let accepted = RwSignal::new(BTreeSet::<String>::new());
let multi_selection = EntityTableMultiSelection::controlled(
    accepted.into(),
    Callback::new(move |proposal: EntityTableSelectionProposal| {
        // One atomic event carrying the COMPLETE resulting set.
        accepted.set(proposal.keys);
    }),
);
// Pass `multi_selection` through EntityTable's `multi_selection` prop.
```

### The callback is atomic

Every gesture -- one row checkbox, or a header checkbox covering a hundred
rows -- emits exactly ONE `EntityTableSelectionProposal`. Its `keys` field is
the complete proposed set, not a delta and not a patch: apply it wholesale or
decline it wholesale. There is never a stream of per-row events for the caller
to reassemble. `cause` says which gesture produced it
(`EntityTableSelectionCause::Row` or `::DisplayedPage`, the latter carrying
the exact keys it covered), and `scope` stamps the dataset identity the
proposal was computed against.

Accepted truth stays caller-owned. Both checkboxes re-assert the accepted
state onto the element the browser just toggled *before* emitting, so a
declined or delayed proposal leaves no optimistic divergence to reconcile.

### What the header checkbox governs

**Exactly the rows currently displayed, and nothing else.** `EntityTable`
holds the complete dataset, so it *could* offer a genuine "select every row"
affordance; it deliberately does not, and the type says so.
`EntityTableDisplayedPageSelection` is computed over
`EntityTableDisplayedPage` -- the stable keys the table is painting right now,
after filtering, sorting and paging have all been applied. That population is
the body's own `page_row_keys`, itself derived from the one resolved
`EntityPageSize` every other part of the render reads (`ldui-5p06`); the
header does not recompute a second page window, so it cannot come to mean a
different set of rows than the body shows.

| Header state | Meaning | Rendered as |
| --- | --- | --- |
| `NoRows` | The table is displaying nothing | unchecked, `aria-disabled` |
| `None` | No displayed row is selected | unchecked |
| `Partial` | Some but not all displayed rows are selected | `indeterminate` |
| `All` | Every displayed row is selected | checked |

`indeterminate` therefore has one precise meaning: *some but not all of the
rows currently displayed are selected*. Accepted keys on another page never
tint the header checkbox and can never turn it checked -- that would tell a
user the rows in front of them are all selected when they are not. Their count
is announced separately, in a `role="status"` live region, through
`EntityTableSelectionTexts::selection_summary`.

`indeterminate` is a **DOM property with no HTML attribute at all**. Writing
`indeterminate="true"` in markup does nothing. `EntityTable` binds it with
`prop:indeterminate` and re-writes it with `set_indeterminate` inside the
change handler, because the browser clears the property the moment the user
clicks the box.

### Off-page keys and aliasing

Selection is keyed by stable business identity, never by row position. Every
proposal is a pure set operation over named keys that carries all other
accepted keys through untouched, so off-page keys survive paging, filtering
and sorting *by construction* rather than by a preservation step someone could
forget. Removing a row, replacing the dataset, or re-sorting can only stop a
key from being rendered -- there is no index anywhere for a different entity to
slide into. `EntityTableSelectionProposal::scope` (the table's
`dataset_identity` by default, overridable with `with_scope`) lets a caller
refuse a proposal minted against a previous dataset rather than have keys
silently relabelled.

### Accessibility

Each row checkbox is named after its own row -- the leading visible cell's
text by default, the stable key when that is blank, or whatever
`with_row_label` resolves from the key -- never the bare word "checkbox". The
header checkbox is named from `EntityTableSelectionTexts` and every default
string says *this page*, so no rendered copy can be read as a claim about rows
the user is not looking at. State is carried by the native checkbox glyph and
by `aria-selected` on the row, never by colour alone. The checkbox cell stops
click and keydown propagation, so ticking a box never also fires
`on_row_activate`; multi-selection does not make the row itself a click
target, because its gesture already lives in a native, keyboard-operable
control.

### Incompatible configuration is refused, not resolved

`selection` and `multi_selection` are mutually exclusive. Supplying both
**panics at construction** with
`EntityTable configuration cannot combine selection with multi_selection`,
the same way `preference_ownership` plus `storage_key` already fails closed.
Silently honouring one would make a bulk-assignment workflow act on a single
row, or a single-row workflow act on a set. Omitting `multi_selection`
entirely renders exactly the markup a table that predates it rendered: no
leading track, no leading cells, no live region.

## Controlled accessible row groups (ldui-iyfa)

`row_grouping` partitions the rendered rows into accessible sections without
splitting the table. It exists because a dataset with repeated child facts --
Office Coordinator Activity's 459 activity rows, where every row repeats its
coordinator's name and the workflow reads Task / Goal / Actual beneath one
coordinator heading -- previously had only two bad options: repeat the group
identity in every row, or fork one `EntityTable` per group and duplicate the
column header and filter row with it.

```rust,ignore
use leptos_daisyui_rs::components::{
    EntityRowGroup, EntityRowGrouping, EntityTable,
};
use std::rc::Rc;

let groups = Signal::derive_local(move || {
    coordinators
        .get()
        .into_iter()
        .map(|coordinator| {
            EntityRowGroup::new(coordinator.id, coordinator.display_name)
                .with_meta(coordinator.cadence)
        })
        .collect::<Vec<_>>()
});

view! {
    <EntityTable
        data=rows
        columns=activity_columns()
        row_key=Rc::new(|row: &ActivityRow| row.id.clone())
        dataset_identity=dataset
        row_grouping=EntityRowGrouping::controlled(
            Rc::new(|row: &ActivityRow| row.coordinator_id.clone()),
            groups,
        )
    />
}
```

### The key is the identity; the label is copy

`EntityRowGroup::new(key, label)` separates the two on purpose. The key drives
the partition, the section rank, collapse state, and the exported group
identity. The label drives the rendered heading and the exported group column,
nothing else. Two groups may carry the *same* label and stay entirely
distinct, and relabelling a group (a locale change, a renamed coordinator)
repartitions nothing, reorders nothing, and cannot move a collapse flag onto a
different group.

Groups the caller never declared are not dropped. They rank after every
declared group, in first-appearance order, so a dataset that grows a new group
key still shows every record rather than silently hiding rows.

### One global header, one filter row

Grouping renders exactly one `<thead>` and one controlled filter row for the
whole table, regardless of how many sections it paints. Consumers need no DOM
post-processing and no local table fork.

### Sorting and filtering, stated explicitly

- **Filters apply to child rows.** `EntityTable` filters nothing itself; the
  caller filters `data` as before. A group whose rows are all filtered away
  has no rows left, so its heading disappears with them. There is no separate
  "hide empty groups" switch to forget.
- **Row sorting happens within groups.** Grouping applies a *stable* partition
  by group rank on top of the table's own sort permutation, so the sort order
  inside every section is exactly the order the sort produced.
- **Group order is caller-controlled.** The default is the declared order of
  `groups`. `EntityRowGrouping::with_order` selects an explicit group sort --
  `EntityGroupOrder::LabelAscending` / `LabelDescending` -- which replaces the
  rank only. Ties fall back to declared order, so the result is total rather
  than dependent on sort implementation.

### Pagination never inflates counts

Group headings are presentation rows. They are not records, so they never
enter the `Showing x-y of z` summary, the page count, or the display
projection's row list. Paging remains strictly over data rows.

A heading is only ever derived from a row that is on the current page, which
makes an *orphan heading* -- an expanded group's heading stranded as the last
visible row with its children on the next page -- unrepresentable rather than
merely avoided. When a group's rows straddle a page boundary, the next page
opens with a continuation heading (`"{group} (continued)"`).

### A group that fits is kept whole (ldui-5in5)

Pagination is group-aware. **A group whose complete row count fits within one
page capacity is never split merely to fill the remainder of the previous
page**: the page ends early, leaving those slots empty, and the group starts
whole on the next page. Three seventeen-row coordinators at a capacity of
eighteen therefore render one coordinator per page, instead of seventeen rows
plus the next coordinator's heading and one orphaned row.

**A group larger than the whole capacity cannot be kept whole by any packing**,
so it degrades honestly to the previous fill-first behavior: it fills the
current page's remainder and resumes on the next under the existing
continuation heading. Deferring it to a fresh page would still split it, would
still need the continuation heading, and would waste a page of rows for
nothing.

Two invariants keep the rule total. Every page holds **at least one row** (the
early break only fires when the page already holds one), so no empty page can
be emitted and paging always advances. And the pages **partition** the
displayed rows exactly -- contiguous, disjoint, complete -- so counts stay
truthful: the `Showing x-y of z` summary is read off the resulting page
window, never multiplied out of `page * capacity`, which is simply the wrong
number once a page can stop short.

The plan is computed from the displayed order, so filtering, sorting, collapse
and a dataset swap all recompute the group boundaries *before* paging. It is
the single source of page boundaries -- the body, the pager, the footer range,
the displayed-page selection population, focus recovery and the export
projection all read it -- exactly as `ldui-5p06` made the page *size* single.
An ungrouped table is unaffected: it delegates to the same
`page_count`/`page_bounds`/`row_range` arithmetic it always used.

Nothing about this asks the consumer to pick a magic page size. Capacity is
still whatever `viewport_fit` measures or the rows-per-page control selects.

### Collapse is optional, controlled, and a filter

The default exposes every filtered row. `EntityRowGrouping::collapsible`
binds a caller-owned `Signal<BTreeSet<String>>` of collapsed keys plus a
`Callback<EntityGroupCollapseProposal>`. Like `multi_selection`, every gesture
emits ONE proposal carrying the COMPLETE resulting key set -- never a delta --
stamped with the scope it was computed against, and nothing changes until the
caller's own signal does.

Collapsing removes the group's rows from the displayed model outright: they
leave paging, the row-range summary, the displayed-page selection population,
and `on_display_projection` together. That is what keeps every count truthful,
and it is why collapsed children leave the accessibility tree instead of being
painted and hidden. A collapsed group keeps its heading and its honest row
count, so nothing disappears without a trace.

### Accessibility

- **One `<tbody>` per rendered section**, which is already `role="rowgroup"`,
  so the section boundary is the structural fact that these rows belong
  together. It carries `aria-labelledby` pointing at its heading.
- **The heading is a `<th scope="colgroup">`** spanning every column. HTML's
  own header-association algorithm applies a `colgroup`-scoped header to the
  remaining cells in those columns, so a screen reader in table navigation
  attributes each child cell to the heading **automatically** -- no per-row
  attribute, and no group label repeated in a data cell. That repetition is
  the defect being fixed, so re-introducing it as an ARIA crutch would defeat
  the point.
- **The heading row is never focusable and never `aria-selected`.** It is
  presentation, not a record, and carries no row key for selection or focus
  recovery to latch onto.
- **Collapse lives on a control, not the row.** When `collapsible` is bound,
  the heading holds one ordinary `<button>` with `aria-expanded` and
  `aria-controls` naming its `<tbody>` -- a single normal tab stop, no trap,
  no roving state of its own. Its accessible name contains the visible group
  label, so it satisfies label-in-name.
- **The heading spans the current column count**, including the leading
  multi-selection control cell when one is rendered. The empty-state row and
  every heading read one shared derivation, because two independent colspan
  computations is how a full-width row comes to be short by one and desync the
  declared `<colgroup>` tracks (`ldui-ibjk`).

### Grouping and multi-selection

A group heading does **not** participate in the displayed-page selection
population that `multi_selection`'s header checkbox governs (`ldui-nz6d`): it
has no row key, so it cannot. The header checkbox continues to mean exactly
"the rows currently displayed", now grouped.

**There is deliberately no per-group select-all.** A group spans pages, and a
collapsed group displays nothing at all, so a per-group checkbox would have to
name keys the table is not painting -- reintroducing precisely the "checked
means something you cannot verify" defect `ldui-nz6d` refused for the header
checkbox. Selecting a whole group is done the same way selecting everything
is: widen the page size until the group is displayed, which makes the
widening an explicit, visible act.

Collapsing a group does not clear its selected keys. They stay accepted and
are reported by the live region as off-page, exactly like keys on another
page.

### Export carries the group identity

The visual table stops repeating the group in every row, so the display
projection puts it back. On a grouped table `on_display_projection` prepends a
synthetic `ENTITY_GROUP_COLUMN_ID` column carrying the group **label** (what a
person reads in a CSV), and every `EntityTableDisplayRow` carries
`group_key: Some(..)` with the stable **identity** (what a re-import joins
on). Rows are in the same grouped order the body paints. An ungrouped table's
projection is unchanged: no group column, and `group_key` is `None`.

### Localization

Group copy lives on its own `EntityGroupTexts`, supplied via
`EntityRowGrouping::with_texts`, for the same reason `EntityTableSelectionTexts`
is separate: copy that only exists when the feature is configured must not
widen the always-required `EntityTableTexts` and break every consumer's
literal. It carries `column_header`, `row_count` (`{count}`), `continued`
(`{group}`), `collapse` (`{group}`), and `expand` (`{group}`).

### Stable IDs

Each section mints a stable `<tbody id>` and a `<th id>` derived from it, so
`aria-labelledby` and `aria-controls` resolve without the caller supplying
ids or post-processing the DOM. Group identity is additionally exposed for
tests and styling as `data-entity-group`, `data-entity-group-header`,
`data-entity-group-continued`, `data-entity-group-collapsed`,
`data-entity-group-toggle`, `data-entity-group-meta`, and
`data-entity-group-actions`.

## Provider-empty is not filtered-empty (ldui-g4nw)

`EntityTableTexts` carries two empty-state sentences, because they are two
different facts and the table already knows which one is true:

| field | shown when | default |
| --- | --- | --- |
| `no_rows` | the authoritative source dataset holds no rows | `"No rows"` |
| `no_matching_rows` | source rows exist, the current projection selected none | `"No rows match the current filters"` |

Source membership is `source_data` when supplied, otherwise the rendered `data`
snapshot -- the same fallback focus recovery uses, and the reason a table with
no separate source still classifies a zero-row render as provider-empty. Local
filtering, searching, a bounded date filter and collapsed groups all reach the
*filtered* state; collapse counts because it removes rows from the displayed
model outright.

The rendered cell carries `data-entity-empty-state="provider" | "filtered"`, so
a test or a consumer can assert on the state rather than on localizable copy.

A caller that overrides only `no_rows` -- every caller written before this
existed -- is unchanged: their domain sentence ("No contribution credits are
present in this snapshot.") still owns the provider-empty case and stops being
asserted over a merely over-narrow filter, which is the bug. Override
`no_matching_rows` too when the filtered case wants domain copy of its own.

## Deterministic control identity (ldui-izkq)

An accessible *name* is not a DOM *identity*. The generated select-all and
per-row selection checkboxes now carry an `id` **and** a `name` alongside their
localized `aria-label`, so a consuming page can reference them from a
`label[for]`, an `aria-controls`, a form submission or a deterministic
automation selector without reaching into markup this crate owns. `name` is not
optional decoration: it is what makes the input a real form control.

The scheme is the one `DataTable` established in `ldui-j6sh`, restated for
`EntityTable` rather than imported across a module boundary:

- One **table control prefix** per mounted table. `control_id` supplies it;
  omitting it mints a process-unique `ldui-entity-table-N`, so two tables on one
  page never collide even with no configuration. Supply your own when you want a
  prefix stable across builds -- a mount-order counter is not. A supplied value
  is trimmed and escaped into `[A-Za-z0-9_-]`, because an `id` may not contain
  whitespace and a `.` or `#` breaks every selector built from it.
- The select-all checkbox is `{prefix}-select-all`; a row's checkbox is
  `{prefix}-select-row-{encoded row key}`.
- The token is **escape-encoded, not slugified**: every byte outside
  `[A-Za-z0-9]` (including `_`) becomes `_` plus two hex digits. A slug is not
  injective -- `a b`, `a-b` and `a_b` all collapse to `a-b` -- so three distinct
  rows would share one id. The encoding is decodable, hence injective, and
  contains no `-`, which keeps the `-`-joined id segments unambiguous.
- **Row identity is the stable row key, never the position.** An index-derived
  id re-points at a different row the moment the table sorts, filters, pages,
  groups or collapses, and an id that silently aliases to another row is worse
  than no id.

`page_size_control_id` predates this prop and still wins outright for the
rows-per-page select. Supplying only `control_id` names that select too
(`{prefix}-page-size`); supplying neither leaves its own minted
`ldui-entity-page-size-N` exactly as it was.

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

## Interactive-row hover (ldui-jdzr)

Any row that would receive a click/keyboard handler -- `on_row_activate` is
supplied, `selection` is supplied, or both -- gets a framework-owned
light-blue hover background, reusing the same `interactive` predicate that
already drives `tabindex`/`cursor-pointer`/the focus ring, never a second
notion of "interactive". A table with neither carries no hover class on any
row. The color is this crate's existing table-hierarchy token
(`--color-table-filter`, `ui_tokens::color::table::FILTER`) rather than a new
hardcoded hex -- the same light blue already used for the column-filter row
and dropdown, so a hovered row and the filter chrome read as one visual
language.

**Precedence: hover < selected.** The hover utility class is present in the
row's class list only while the row is *not* selected -- not merely
out-ranked by a later declaration. A selected row's `bg-base-200` is an
unconditional class, not itself a `:hover` rule; if both classes were always
present, the `:hover` pseudo-class selector would win the specificity fight
over a plain class selector regardless of source order, and hovering a
selected row would visually read as unselected. Dropping the hover class
outright when a row is selected keeps the selected treatment dominant no
matter how Tailwind or daisyUI order their generated rules.

Emphasis (`row_emphasis`, above) never sets a `background-color`, so
Summary/Muted/Attention rows pick up the hover background exactly the way
they already compose with selection and `zebra` -- text/border emphasis on
top of whichever background (none, selected, zebra-striped, or hovered) is
already painted underneath. The wide and compact presentations share one
`<tr>`, so the hover class is applied once at the row level and covers both
layouts' cells without per-cell styling; neither `<td>` sets its own
background, so the row's hover paints through.

Under `forced-colors` (Windows High Contrast), the hover state uses the
system `Highlight`/`HighlightText` color pair instead of the light-blue
token, mirroring how a native control signals a hover/focus target in that
mode.

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
