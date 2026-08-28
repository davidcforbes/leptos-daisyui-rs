# Snapshot Table Opinionated Foundation Design

**Status:** Approved by the user on 2026-08-28.

**Scope:** Beads `ldui-w1e`, `ldui-ifj`, and `ldui-ifj.1` through
`ldui-ifj.4`.

**Source requirements:** `Future-Architecture.md` sections 7.3, 8.3, 8.4,
10, and 14; the Office No-Hires satellite-page pilot; the Phase 0B keyboard
and accessibility feedback attached to `ldui-w1e`.

## Outcome

The framework will expose one complete, opinionated path for client-snapshot
table pages. It will own snapshot transition semantics, page-state and action
feedback presentation, filter/default controls, table mechanics, focus
recovery, localization hooks, and the fixed vertical composition. Consumers
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
- row-action focus recovery requires the table's filtered, sorted, and paged
  order, which a consumer cannot safely reconstruct;
- several accessible names are hard-coded in English;
- shared `DataTable` sorting is pointer-only even though resizing, reordering,
  visibility, paging, and compact rows are keyboard-operable;
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

The framework will add a pure dataset-transition model built from three
pieces:

- `DatasetRequest` carries a monotonically increasing request token and the
  requested stable dataset identity.
- `SnapshotData<R, V, M>` carries the displayed dataset identity, rows in an
  `Rc<Vec<R>>`, snapshot revision, authoritative row count, and optional typed
  metadata. Replacing this value swaps those fields atomically.
- `DatasetPresentation<R, V, E, M>` represents `NeverLoaded`,
  `InitialLoading`, `InitialError`, `Displaying`, `Replacing`, and
  `RetainedError`.

`Replacing` and `RetainedError` contain both the complete displayed snapshot
and the requested destination. The table identity and retained label always
come from the displayed snapshot. The selector's requested value and pending
status come from the active request. A failure without displayed data becomes
`InitialError`; a failure with displayed data becomes `RetainedError`.

The type exposes read-only helpers for displayed identity/data, requested
identity, busy state, retained-data state, and error state. Callers derive the
`DatasetSelector`, result/KPI inputs, and `EntityTable::dataset_identity` from
the same signal rather than maintaining aliases.

### Reducer rules

Pure transition methods will start a request, accept a response, and record a
failure. A response is applied only when both its token and dataset identity
match the active request. Older, duplicated, or mismatched responses return a
non-applied disposition and cannot change the displayed snapshot. A successful
match replaces the entire `SnapshotData` value once. A retry creates a new
token while preserving the displayed snapshot.

The model does not fetch data or generate request tokens from wall-clock time.
The consumer owns transport and supplies monotonically increasing tokens,
making races deterministic in native tests.

## Orthogonal Runtime Presentation

`PresentationState` remains the compile-time story and test catalog in page
contracts. It will not become the page's mutable runtime state.

Runtime presentation is
`SnapshotTablePresentation<R, V, E, M, K = String>`, a typed struct with
orthogonal fields:

- `dataset: DatasetPresentation<R, V, E, M>`;
- `content: SnapshotContentState`, which is ready, empty dataset, or no local-
  filter results;
- `access: SnapshotAccessState`, which is allowed, expired session, or
  forbidden;
- `action: ActionFeedbackState<K>`, which is idle or one keyed action outcome.

This permits valid combinations such as retained rows plus a row-action
conflict, or retained rows plus a preference-save failure. Expired and
forbidden access are replacement states and suppress actionable data content.
Initial loading/error, empty/no-results, and access replacement take
precedence in one documented rendering function; retained transition and
action notices remain above mounted content. Preference-save feedback is a
separate controlled `FilterBar` axis so it cannot displace page data or render
twice.

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

`ActionFeedbackState<K>` will be keyed by stable row/action identity and cover:

- pending;
- success;
- recoverable conflict;
- stale-row reconciliation;
- partial success;
- retryable failure;
- terminal failure.

The renderer receives reactive texts plus optional Retry and Dismiss
callbacks. Pending state disables only the corresponding action through
consumer-controlled state; it does not freeze the table. Feedback announces
changes without stealing focus. The component reports intent only and treats
callback invocation as neither transport success nor completion proof.

## `SnapshotTablePage`

`SnapshotTablePage` is a typed composition root, not a generated page. It
accepts a stable page-contract ID, the runtime presentation signal, and slots
for:

1. one `PageHeader` without a nested dataset control;
2. one distinct `DatasetSelector` slot;
3. optional full-width `KpiStrip` content;
4. one complete `FilterBar`;
5. framework-rendered state/action feedback;
6. one `EntityTable` content slot.

It owns full-width sizing, vertical rhythm, slot order, retained-content
mounting, and observable `data-*` markers used by audits. It does not create
columns, filters, rows, routes, requests, capabilities, or preference
payloads. Documentation will show the one supported signal flow: selector and
table derive their identities from `DatasetPresentation`; the table derives
rows from its displayed snapshot; the page derives panels from the complete
runtime presentation.

`PageHeader` keeps its existing `dataset` child for source compatibility, but
the canonical `SnapshotTablePage` path does not use it. The separate second
slot is the architecture's observable distinction between replacing the
source dataset and changing local filters.

## Controlled Filter and Default Preferences

### FilterBar contract

`FilterBar` becomes the complete local-view meta-component. One instance owns
the responsive horizontal control row and summary/action row containing:

- search and typed filter-control slots;
- active removable chips;
- localized active-filter summary;
- localized result count;
- one Reset action;
- dirty/clean state;
- one explicit Save as Default action;
- pending, saved, conflict, and failure feedback.

The old layout-only call shape remains available during migration, but the
reference page and documentation use the complete shape. `ActiveFilterChips`
remains exported for lower-level composition; the complete `FilterBar` embeds
it and does not render a second clear/reset action.

### Persistence-neutral payload

The save callback receives `SnapshotViewDefaults<F>`, whose public fields are
local filter defaults and `EntityTablePreferences`. It has no dataset,
rows/revision, current page, session, or action fields. The constructor uses
the existing validated local `FilterSchema`, which already rejects the
dataset selector as a local filter. This gives the framework payload no place
to serialize a dataset identity.

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
working, while localized consumers can pass a local reactive signal. The
component converts the static variant to one internal stored value and reads
the reactive variant on demand. Column IDs remain `&'static str` and are the
sole identity used for preference normalization.

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
  rows.

The compact renderer also becomes reactive, or the default compact renderer
uses the latest columns, so narrow layouts cannot retain stale locale copy.

## EntityTable Row-Action Focus Recovery

The framework will add an explicit `EntityRowAction` marker/wrapper with a
stable action ID. `EntityTable` records the focused row key, action ID, and
position when focus enters a marked action. Consumers do not query the DOM or
guess source order.

If that row disappears from the supplied dataset, the table recomputes the
actual filtered, sorted, and paged visible order and focuses the same action
on the row now occupying the removed row's visible position. If that was the
last row, it chooses the preceding visible row. If the page collapses, it uses
the clamped resulting page. If no matching action remains, focus falls back to
the named `EntityTable` region, which is programmatically focusable with
`tabindex="-1"`, rather than the document body. Wide and compact layouts share
the same row/action identity contract.

If an action is declined or fails and the row remains, the table does nothing
and native focus stays on the initiating action. The same disappearance logic
handles another user's event without a consumer-specific branch. Framework
code may inspect its own table subtree to resolve a marked focus target;
consumer DOM queries are forbidden.

## DataTable and ServerDataTable Keyboard Sorting

Sortable shared-table headers will contain a native Button-equivalent sort
control inside the `th`. The `th` retains canonical `aria-sort`; the control's
accessible name is the localized column header. Native click activation gives
pointer, Enter, and Space one callback path and prevents hand-written key
handlers from double firing. Non-sortable headers render no control and no tab
stop.

The resize separator remains a sibling of the sort control, keeps separator
range semantics, and stops its pointer/click events from sorting. The existing
keyboard resize contract from `bc0d92e` remains unchanged. Browser coverage
must exercise both client `DataTable` and `ServerDataTable` because they share
the renderer but own different query state.

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

No new visual-defect family was discovered in the review, so no new audit rule
is proposed. The existing rulebook warning about valid default variants still
requires manual comparison of the final reference page with its approved
intent.

## Verification Strategy

This is a web/WASM surface and uses the repository's PixelProof A/B/C/D
methodology.

### Native and contract tests

Pure tests cover every dataset transition and stale-response disposition,
panel precedence, save-state reducer, default-payload serialization, locale
replacement, column normalization, actual-order focus targeting, page
collapse, and Field ID allocation.

### Layer A: visual

The reference SnapshotTablePage receives reviewed desktop and narrow named
states for displaying, replacing, retained error, no results, action conflict,
and preference failure. Component-region SSIM and the existing style/layout
audits enforce wrapping, no overlap, no clipping, declared typography, shape,
shadow, and component drift.

### Layer B: structure, state, and model

Browser tests compare the pure model/debug oracle with DOM-observable
displayed/requested identities, retained row counts, panel kinds, filter
summary, preference state, current columns, sort/page state, and focused row
action. Dataset races prove stale responses leave all displayed fields
unchanged.

### Layer C: accessibility

Tests use real keyboard input for DataTable and ServerDataTable sort, table
resize/reorder/visibility/paging/compact behavior, FilterBar actions, and row
focus recovery. They assert names, roles, `aria-sort`, separator values,
label/control associations, live feedback, visible focus, and zero new
critical/serious axe or browser accessibility errors.

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

- `DataTable` and `ServerDataTable` signatures remain source compatible; only
  sortable-header markup and behavior change.
- `EntityTable` static columns, legacy storage key, and uncontrolled preference
  behavior remain available. Reactive columns and focus markers are additive.
- `ListPage`, `AsyncDataSection`, and standalone `ActiveFilterChips` remain
  exported. Documentation identifies them as lower-level compatibility paths.
- `PresentationState` and page-contract serialization do not change meaning.
- The reference client-snapshot demo migrates first and becomes the executable
  usage guide. Consumer vendoring occurs only after the clean framework
  revision passes full gates.

## Acceptance

The design is complete when the exported framework supplies the adopted
Layer 2 and Layer 3 path without consumer-local mechanics; all six Beads meet
their stated acceptance criteria; documentation shows one unambiguous
controlled composition; the focused A/B/C/D negative controls and full clean
gate pass; all issue state, commits, and remotes are synchronized.
