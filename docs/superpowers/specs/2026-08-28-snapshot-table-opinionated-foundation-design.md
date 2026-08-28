# Snapshot Table Opinionated Foundation Design

**Status:** Approved by the user on 2026-08-28; amended by the owner's
inline-filter, visual-hierarchy, and sort-geometry decisions later that day.

**Scope:** Beads `ldui-w1e`, `ldui-gbs`, `ldui-ifj`, and `ldui-ifj.1`
through `ldui-ifj.5`.

**Source requirements:** `Future-Architecture.md` sections 7.3, 8.3, 8.4,
10, and 14; the Office No-Hires satellite-page pilot; the Phase 0B keyboard
and accessibility feedback attached to `ldui-w1e`; and the approved
`4iiz-etl` column-aligned filter-row usage.

## Outcome

The framework will expose one complete, opinionated path for client-snapshot
table pages. It will own snapshot transition semantics, page-state and action
feedback presentation, filter/default controls, table mechanics, focus
recovery, localization hooks, stable column geometry, the opinionated visual
hierarchy, and the fixed vertical composition. Consumers
will continue to own domain rows, filters, authorization, transport,
persistence, routes, and explicit Rust page composition.

This is an additive framework completion, not a page generator. Existing
`ListPage`, `AsyncDataSection`, `DataTable`, and uncontrolled `EntityTable`
call sites remain supported while consumers migrate to the stronger
contracts.

## Why This Work Belongs in the Framework

The No-Hires pilot proved that the current primitives are individually useful
but leave several architectural invariants to each page:

- a dataset switch can display rows from one office while labeling them as
  another;
- loading, retained error, session, mutation, and preference states can be
  collapsed into one mutually exclusive enum;
- the current `FilterBar` only arranges controls, leaving active filters,
  result count, Reset, Save as Default, and save feedback to page code;
- detached one-to-one dropdowns repeat column labels and weaken the visual
  relationship between a filter and the data it constrains;
- row-action focus recovery requires the table's filtered, sorted, and paged
  order, which a consumer cannot safely reconstruct;
- several accessible names are hard-coded in English;
- shared `DataTable` sorting is pointer-only even though resizing, reordering,
  visibility, paging, and compact rows are keyboard-operable;
- sorting can rebuild the header and let a newly paged body subset recompute
  content-driven column widths, moving the table outline and controls;
- native Field ID tests do not prove the rendered WASM page contract.

These are repeated mechanics and accessibility obligations, not domain
policy. Centralizing them makes the adopted architecture true in code and
prevents Office and Inventory from growing incompatible local copies.

## Approaches Considered

### Keep the framework primitive-only

Consumers would continue composing `ListPage`, `DatasetSelector`,
`FilterBar`, `ActiveFilterChips`, `AsyncDataSection`, and `EntityTable`
directly. This has the smallest framework diff, but it repeats the exact
transition and focus bugs observed by the pilot and does not satisfy the
adopted Layer 2/Layer 3 catalog.

### Build a generic page schema or generator

A manifest or macro could describe every slot and generate the page. This
would enforce structure, but it would create a second UI language, obscure
ordinary Rust control flow, and violate `Future-Architecture.md` section 8.6.

### Add typed state plus bounded meta-components

The selected approach adds pure typed models and narrowly opinionated
components. The models make invalid transition combinations hard to express;
the components own layout, accessibility, and presentation; typed children
and callbacks leave domain behavior explicit. This satisfies the architecture
without inferring routes, requests, permissions, or business fields.

## Architectural Boundaries

The implementation follows the existing five layers.

- Layer 0 continues to own tokens, focus rules, and audit ceilings.
- Layer 1 primitives remain the only implementation path for buttons, inputs,
  selects, alerts, and pagination controls.
- Layer 2 gains complete `DatasetSelector`, `FilterBar`, `PageStatePanel`, and
  `ActionFeedback` contracts and extends `EntityTable` behavior.
- Layer 3 gains `SnapshotTablePage`, a fixed typed composition root.
- Layer 4 consumers supply domain values and callbacks through those APIs.

No new component performs network, database, local-storage, session, route,
or authorization work. The existing explicitly named
`LegacyLocalStorage` table mode remains a compatibility exception; the new
controlled path does not call it.

## Atomic Dataset Presentation

### State shape

The framework will add a pure `SnapshotTableState<R, V, E, M, K>` controller.
Its fields and internal phase enum are private. Public code cannot assemble
orthogonal dataset, content, access, or action values directly. The controller
owns:

- the next checked request sequence;
- the active opaque `SnapshotRequestHandle<V>`, if any;
- the complete displayed `SnapshotData<R, V, M>`, if any;
- the current access replacement state;
- keyed concurrent action feedback; and
- an opaque dataset/access generation used by table focus and local-result
  summaries.

`SnapshotData` has private fields and a validated constructor for the
displayed dataset identity, `Rc<Vec<R>>`, snapshot revision, authoritative row
count, and optional typed metadata. Replacing it swaps those values once.
`SnapshotRequestHandle` also has private fields; it cannot be created,
decreased, or reused by a caller.

The controller exposes a copyable/read-only `SnapshotTableView` whose phase is
one of never loaded, initial loading, initial error, displaying, replacing, or
retained error. Replacing and retained-error views expose both the complete
displayed snapshot and requested destination. The table identity and retained
label always come from the displayed binding; selector request presentation
comes from the active handle.

Local content state is derived through `LocalResultSummary`, which can be
minted only from the current displayed binding and carries its opaque
generation/revision. A stale summary is rejected and cannot label a newly
displayed snapshot as empty or no-results. Empty-dataset versus no-local-
results is derived from authoritative and filtered counts rather than a
caller-constructed enum.

### Reducer rules

Pure transition methods start a request, accept a response, record a failure,
replace access, and update one keyed action. `start_request(dataset)` mints and
returns the next opaque handle; it never accepts a caller token. Completion or
failure applies only when the supplied handle is still active and its dataset
matches. Older, duplicated, consumed, or mismatched handles return a
non-applied disposition and cannot change the displayed snapshot. A successful
match replaces the complete snapshot once and consumes the handle. A retry
mints a new handle while preserving the displayed snapshot. Checked sequence
exhaustion is an explicit error rather than wraparound.

Expired or forbidden access increments the access generation, consumes any
request, clears actionable bindings, and suppresses displayed content. Returning
to allowed access does not resurrect the suppressed snapshot implicitly. The
model does not fetch data or use wall-clock time; consumers still own transport
and pass the framework-issued handle back with completion.

## Validated Runtime Presentation

`PresentationState` remains the compile-time story and test catalog in page
contracts. It will not become the page's mutable runtime state.

Runtime presentation is the read-only view derived by
`SnapshotTableState::view(current_local_result)`. There is no public struct
whose independent fields can express `NeverLoaded + Ready`, non-empty rows as
`EmptyDataset`, stale filtered counts against a newer revision, or allowed
actions during an access replacement. A single pure render-decision function
applies precedence:

1. expired/forbidden access replaces the page content;
2. never-loaded, initial-loading, and initial-error replace table content;
3. empty-dataset or no-local-results replace the table body only when their
   summary matches the displayed generation;
4. replacing and retained-error notices remain above the mounted table; and
5. keyed action and preference feedback coexist with retained rows.

Legitimate axes remain independent inside the controller, but only reducer
operations can change them. Preference-save feedback remains a separate
controlled `FilterBar` model so it cannot displace page data or render twice.

## `PageStatePanel`

`PageStatePanel` will own consistent presentation for:

- initial loading;
- initial error with optional Retry;
- never loaded;
- empty dataset;
- no local-filter results;
- expired session;
- forbidden access;
- replacing/refreshing retained data;
- retained load/refresh failure with optional Retry.

It receives a reactive complete text struct and typed optional callbacks. It
uses the framework's Button and alert/skeleton primitives, appropriate
`status` or `alert` semantics, and `aria-busy`. Replacement panels hide the
table; retained panels never unmount it. `AsyncDataSection` remains available
for compatibility and is documented as the lower-level legacy composition.

## `ActionFeedback`

`ActionFeedbackModel<K>` is a private-field keyed collection. Distinct keys may
be pending concurrently; updating one key never disables, replaces, or
dismisses another. Each entry covers:

- pending;
- success;
- recoverable conflict;
- stale-row reconciliation;
- partial success;
- retryable failure;
- terminal failure.

The model also stores one monotonically sequenced latest announcement. The
renderer may show all relevant keyed entries, but only the latest transition
is sent through the polite/assertive live region, preventing concurrent
updates from producing competing announcements. Retry and Dismiss callbacks
carry the key; retryable terminal entries may be retried, completed outcomes
may be dismissed, and pending entries cannot be dismissed into a false idle
state. Pending disables only the corresponding action through
consumer-controlled state; it does not freeze the table. Feedback never steals
focus. Callback invocation reports intent and is neither transport success nor
completion proof.

## `SnapshotTablePage`

`SnapshotTablePage` is a typed composition root, not a generated page. It
accepts a stable page-contract ID, the private-field state signal, and:

1. one `PageHeader` without a nested dataset control;
2. one `SnapshotDatasetSelectorConfig<V>`;
3. optional full-width `KpiStrip` content;
4. one complete controlled `SnapshotFilterConfig`;
5. framework-rendered state/action feedback;
6. one `SnapshotEntityTableConfig<R>`.

The two critical configs deliberately have no selected/requested dataset,
rows, revision, total, or table-dataset fields. The page renders
`DatasetSelector` and `EntityTable` itself and injects those bindings from the
same `SnapshotTableView`. A consumer may still use lower-level raw components,
but the canonical page API cannot supply unrelated selector/table identity
signals. Debug/audit markers repeat the opaque generation on the page,
selector, and table so internal wiring regressions are independently
detectable in a real browser.

The page owns full-width sizing, vertical rhythm, slot order, retained-content
mounting, and observable `data-*` markers. It does not invent columns, filter
values, routes, requests, capabilities, or preference payloads; those remain
typed config/callback inputs. Documentation shows one supported signal flow,
with no alias signal for dataset labels or rows.

`PageHeader` keeps its existing `dataset` child for source compatibility, but
the canonical `SnapshotTablePage` path does not use it. The separate second
slot is the architecture's observable distinction between replacing the
source dataset and changing local filters.

## Controlled Filter and Default Preferences

### FilterBar contract

`FilterBar` becomes the complete local-view utility meta-component. One
controlled filter model spans this utility row and the table's aligned filter
row. The utility row contains:

- global search and typed non-column/domain filter slots;
- active removable chips;
- localized active-filter summary;
- localized result count;
- one Reset action;
- dirty/clean state;
- one explicit Save as Default action;
- pending, saved, conflict, and failure feedback.

`EntityColumnFilters` maps zero or one framework-primitive control to a stable
column ID and renders those controls in a second `thead` row directly beneath
the column headers. Every visible column still receives a filter `th`, keeping
header, filter, and body tracks aligned through reorder/visibility changes.
Controls stop pointer/keyboard propagation so interacting with a filter cannot
sort, resize, or activate a row. A field that maps one-to-one to a column is
not duplicated in the utility row. At narrow widths the header, filter, and
body remain one horizontally scrolling unit; an optional drawer renderer may
mirror the same controlled model but cannot own separate values.

The old layout-only `FilterBar` call shape and `ActiveFilterChips` remain
available for lower-level composition. The reference page uses the complete
hybrid shape and renders one Reset/clear action total.

### Persistence-neutral payload

The save callback receives non-generic `SnapshotViewDefaults`, whose private
fields are `LocalFilterDefaults` and `EntityTablePreferences` with read-only
accessors. `LocalFilterDefaults` is a framework-owned, schema-ordered map of
serialized values. It can be created only through
`FilterSchema::project_defaults`; undeclared keys and the schema's dataset
selector key are rejected. Arbitrary consumer structs are never serialized as
the payload, so an `office_id` member cannot bypass the schema through a
generic type parameter. A negative fixture attempts exactly that projection.

The payload has no dataset, rows/revision, current page, free-text search,
session, or action fields. Its serializer emits only validated local filter
keys plus table preferences.

No callback fires when controls change, Reset is pressed, locale changes, a
dataset changes, or a save result is rendered. Exactly one callback fires from
an enabled explicit Save as Default activation. The consumer owns the request,
completion barrier, version/revision update, conflicts, and errors.

### Reactive text

Complete reactive text structs will cover every visible and accessible string:

- `FilterBarTexts`: region label, active-filter count templates, result-count
  templates, Reset, Save as Default, dirty/clean disabled explanations, and
  save-state feedback;
- `DatasetSelectorTexts`: loading, displayed dataset, requested destination,
  retained-error, and Retry copy;
- `ActiveFilterTexts`: zero/one/many summaries, remove-chip accessible-name
  template, and clear/reset compatibility copy;
- `PageStatePanelTexts` and `ActionFeedbackTexts`: every framework-owned state
  label and action.

The structs are signals, so changing locale updates existing DOM without
recreating page state or table preferences. Domain option labels and chip
labels remain caller-supplied localized values.

## EntityTable Reactive Columns

`EntityTable` will accept an `#[prop(into)] EntityColumns<T>` value. The public
wrapper has exactly two variants: `Static(Vec<EntityColumn<T>>)` and
`Reactive(Signal<Vec<EntityColumn<T>>, LocalStorage>)`, with `From`
implementations for both inputs. Existing `columns=vec![...]` call sites keep
working, while localized consumers can pass a local reactive signal. Column
IDs remain `&'static str` and are the sole public identity used for preference
normalization.

Internally the wrapper publishes an opaque semantic generation. Every
reactive vector replacement increments it, even when only presentation text
changed, because comparator and sort-key callbacks cannot be compared safely.
`SortedIndexCache` keys reuse by row `Rc`, normalized sort, and semantic
generation. This may perform an extra sort after a label-only locale update,
but can never reuse indices produced by obsolete behavior. Removed or newly
non-sortable column IDs are removed from active sort clauses during the same
normalization pass.

Compact rendering uses the analogous `EntityCompactRow<T>` static/reactive
wrapper. Omitting it continues to select the default compact renderer, which
always reads the current columns.

When reactive columns change:

- headers, compact-row copy, chooser labels, and accessible names update;
- visibility, order, widths, sort, and page-size preferences are normalized
  against stable IDs and otherwise preserved;
- removed/unknown IDs are discarded and newly declared IDs append in system
  order;
- a locale-only change does not reset current page or table preferences;
- sorting uses the latest comparator/key callbacks without mutating source
  rows or reusing the previous semantic generation's cache.

The compact renderer also becomes reactive, or the default compact renderer
uses the latest columns, so narrow layouts cannot retain stale locale copy.

## EntityTable Row-Action Focus Recovery

The framework will add an explicit `EntityRowAction` marker/wrapper with a
stable action ID. `EntityTable` records the current dataset/access generation,
focused row key, action ID, and visible position when focus enters a marked
action. Consumers do not query the DOM or guess source order.

If that row is removed from the supplied source dataset within the same
generation, the table recomputes the actual filtered, sorted, and paged visible
order and targets the same action on the row now occupying the removed row's
visible position. If that was the last row, it chooses the preceding visible
row. If the page collapses, it uses the clamped resulting page. The target is
used only when the exact action is rendered, enabled, visible, and focusable;
otherwise focus falls back to the named `EntityTable` region, which is
programmatically focusable with `tabindex="-1"`, rather than the document body.
Wide and compact layouts share the same row/action identity contract.

A dataset replacement or expired/forbidden access changes the generation,
clears the record, and suppresses neighbor recovery: focus never jumps into an
unrelated dataset or newly authorized surface. If filtering, paging, or a
refresh hides the focused row while it still exists in the same source
dataset, the table may focus its own region but never a neighboring row.
Recovery also verifies that the marked action still owns focus (or was removed
while owning it) so a user who already moved to a filter or pager is not
interrupted.

If an action is declined or fails and the row remains, the table does nothing
and native focus stays on the initiating action. The same disappearance logic
handles another user's event without a consumer-specific branch. Framework
code may inspect its own table subtree to resolve a marked focus target;
consumer DOM queries are forbidden.

## DataTable and ServerDataTable Keyboard Sorting

Sortable shared-table headers will contain a native Button-equivalent sort
control inside the `th`. The `th` retains canonical `aria-sort`; the focused
control directly exposes localized column name, current state, and next plain
action. `EntityTable` additionally exposes multi-sort priority and the next
additive action. Native click activation gives pointer, Enter, and Space one
callback path and prevents hand-written key handlers from double firing.
Non-sortable headers render no control and no tab stop.

The resize separator remains a sibling of the sort control, keeps separator
range semantics, and stops its pointer/click events from sorting. The existing
keyboard resize contract from `bc0d92e` remains unchanged. Browser coverage
must exercise both client `DataTable` and `ServerDataTable` because they share
the renderer but own different query state.

## Opinionated Table Visual and Geometry Contract

Layer 0 gains generated semantic tokens sourced from `ui_tokens`:

- table header: `STATUS_BLUE_FG` (`#004578`) with white content;
- table filter: `STATUS_BLUE_BG` (`#E5F1FB`) with `TEXT_PRIMARY`
  (`#1A1A1A`) content; and
- table grid: `CONTROL_BORDER` (`#E0E0E0`).

The canonical `EntityTable`/snapshot path uses a dark-blue column-header band,
light-blue aligned filter band, collapsed faint row and column borders, and no
zebra striping by default. Zebra remains an explicit opt-in. These are semantic
framework utilities generated from shared tokens, never consumer color
literals. Contrast, forced-colors behavior, light/dark theme rendering, and
focus visibility are verified in the reference page.

Sorting must preserve the table shell geometrically. The table uses a stable
`colgroup`/fixed track model derived from column definitions and controlled
width preferences, never the current page's cell contents. Sort indicators
always occupy a fixed reserved slot, including the unsorted state. Header and
filter cells render through keyed column nodes; sort changes update only
`aria-sort`, the reserved indicator, announcement, and keyed body order.
Pointer, Enter, and Space sorting must preserve within a documented subpixel
tolerance:

- outer viewport and table bounds;
- every header/filter track x-position and width;
- row/column grid-line positions;
- horizontal scroll origin; and
- unaffected header/filter DOM identity.

Column resize, visibility, reorder, viewport resize, or locale-driven column
declaration changes are legitimate geometry transitions and are tested
separately. A compatibility `DataTable` may retain content-driven layout unless
its stable-geometry option is selected; canonical snapshot and server-query
page configs always select stable geometry.

## Field ID Browser Contract

A real WASM fixture will render at least six mixed Field-wrapped Input and
Select controls in one form. It will assert unique control IDs, one-to-one
`label[for]` targets, message associations, and no Chrome duplicate-ID issue.

The current monotonic allocator is retained if that fixture proves it correct;
the Office observation came from a stale vendored framework revision and is
not by itself evidence that current `main` still duplicates IDs. The browser
oracle will be demonstrated by temporarily forcing a duplicate and observing
the intended failure before reverting. If current code fails the fixture, the
implementation plan will replace the allocator with a deterministic
owner/page-scoped strategy without requiring consumer call-site changes.

## Accessibility and Visual Rules

All new interactive elements use framework primitives and daisyUI 5 semantic
classes. Focus indicators, names, roles, state, disabled reasons, and live
announcements are part of the public contract. No page-local raw button or
arbitrary token is introduced. The existing `button-without-btn` drift rule
and visual-quality ceilings remain zero-slack.

The review discovered a new sort-induced geometry defect family. Browser
geometry assertions and reviewed before/after captures are therefore required
in addition to the existing layout/style ceilings. The rulebook warning about
valid default variants still requires manual comparison of the final reference
page with its approved intent.

## Verification Strategy

This is a web/WASM surface and uses the repository's PixelProof A/B/C/D
methodology.

### Native and contract tests

Pure tests cover every dataset transition and stale-response disposition,
handle consumption/reuse, panel precedence, concurrent keyed action updates,
save-state reducer, schema-projected default serialization, locale replacement,
column semantic-generation invalidation, actual-order focus targeting,
dataset/access focus boundaries, page collapse, and Field ID allocation.

### Layer A: visual

The reference SnapshotTablePage receives reviewed desktop and narrow named
states for displaying, replacing, retained error, no results, action conflict,
preference failure, and inline filters. Component-region SSIM and the existing
style/layout audits enforce wrapping, no overlap, no clipping, declared
typography, semantic blue bands, faint grid, shape, shadow, and component
drift.

### Layer B: structure, state, and model

Browser tests compare the pure model/debug oracle with DOM-observable
displayed/requested identities, retained row counts, panel kinds, filter
summary, preference state, current columns, sort/page state, and focused row
action. Dataset races prove stale responses leave all displayed fields
unchanged. Dataset/table/selector generation markers must agree.

### Layer C: accessibility

Tests use real keyboard input for DataTable and ServerDataTable sort, table
resize/reorder/visibility/paging/compact behavior, FilterBar actions, and row
focus recovery. They assert names, roles, `aria-sort`, separator values,
label/control associations, live feedback, visible focus, and zero new
critical/serious axe or browser accessibility errors.

Before and after every pointer/Enter/Space sort, the browser records table,
header, filter, and column bounding boxes, scroll origin, and stable header
node identities. It asserts only keyed body order, sort state, and announcement
change. The negative control temporarily removes the fixed track model or
reserved indicator slot and must fail this geometry oracle.

### Layer D: behavior and side effects

D1 tests assert exactly one callback per pointer/Enter/Space activation and
capture browser console errors and WASM panics. Filter/default tests prove that
only explicit Save emits a payload and that Reset, dataset changes, locale
changes, and feedback rendering emit none.

D2 database/network proof is not applicable inside this persistence-neutral
framework. The components forbid those side effects. Consumer repositories
must test their save, snapshot, mutation, and session transports against their
own completion barriers.

Every new oracle requires break-and-revert evidence: introduce one targeted
fault, observe the specific test fail, revert the fault, and observe green.
Final verification is `cargo xtask verify-full` from a clean revision, after
focused `test-reactivity`, `test-style`, `test-layout`, and visual checks.

## Compatibility and Migration

- `DataTable` and `ServerDataTable` signatures remain source compatible;
  keyboard sort text and stable-geometry selection are additive. Canonical
  server-query config selects stable geometry.
- `EntityTable` static columns, legacy storage key, and uncontrolled preference
  behavior remain available. Reactive columns, aligned filter controls,
  semantic table styling, stable tracks, and focus markers are additive; zebra
  remains opt-in.
- `ListPage`, `AsyncDataSection`, and standalone `ActiveFilterChips` remain
  exported. Documentation identifies them as lower-level compatibility paths.
- `PresentationState` and page-contract serialization do not change meaning.
- The reference client-snapshot demo migrates first and becomes the executable
  usage guide. Consumer vendoring occurs only after the clean framework
  revision passes full gates.

## Acceptance

The design is complete when the exported framework supplies the adopted
Layer 2 and Layer 3 path without consumer-local mechanics; every scoped Bead
meets its stated acceptance criteria; documentation shows one unambiguous
controlled composition; the focused A/B/C/D negative controls and full clean
gate pass; all issue state, commits, and remotes are synchronized.
