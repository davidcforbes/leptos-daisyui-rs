# 2026-08-29 Open-beads wave — clear all open ldui beads

Spec authority: each task's bead (`bd show <id>`) — description, design, and
acceptance criteria are transcribed below and are binding. Controller manages
bead state (`bd start`/`bd close`); implementers only code, test, commit.

## Global Constraints

- Work directly in `C:\dev\leptos-daisyui-rs` on `main` (established repo
  practice; sibling path-deps and demo node_modules make worktrees costly).
- **Run `cargo xtask` only from the repo root.** Never `cargo fmt --all` or
  `cargo clippy --workspace` (they reach sibling repos / hit feature
  unification). Use per-package:
  `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask -p ldui-audit`
  and `cargo clippy -p <crate> --all-targets --features test-mode -- -D warnings`
  for the library crate.
- Library tests: `cargo test -p leptos-daisyui-rs --features test-mode` (or the
  focused `cargo test --lib <module> --no-default-features` pattern where a
  module's tests are native-only).
- **Doc comments: every inline backtick code span stays on ONE `///` line**
  (a wrapped span ICEs clippy 1.95 and silently disables linting).
- daisyUI 5 only: `.form-control`, `.label-text`, `.label-text-alt` are dead
  classes and gated against. Use `fieldset`/`label` or `flex flex-col gap-2`.
- Spacing values must be on the canonical scale (Tailwind 1,2,3,4,6,8,12,16,24);
  sub-4px = stroke tokens; sizes use the enums.
- SVG paints route through `src/charts/paint.rs` (gate-enforced).
- New/changed daisyUI classes used dynamically need `@source inline(...)`
  coverage notes in the component doc comment and demo `input.css` if the demo
  showcases them.
- Components follow the module pattern (`component.rs` / `style.rs` / `mod.rs`),
  `#[prop(optional, into)]` reactive props, `merge_classes!` for user classes,
  single root element for spread attrs.
- Source compatibility: existing callers must compile unchanged unless a task
  says otherwise.
- Browser suites (`test-reactivity`, `test-layout`, `test-style`,
  `verify-full`) are run by the CONTROLLER at batch boundaries — implementers
  write browser/wasm fixtures where acceptance criteria demand them and ensure
  they COMPILE (`cargo check` the test target / showcase wasm), but do not run
  the 8-minute suites per task.
- Commit per task with a conventional message referencing the bead id, ending
  with the Claude co-author trailer.
- If a bead's acceptance criteria demand something genuinely impossible in this
  codebase, report DONE_WITH_CONCERNS naming it — do not silently skip.

## Task 1 — ldui-r1z (resume at its Task 3): public KeyedResultList

In progress; Tasks 1–2 of its internal plan landed in d90ca43 and 7ac1640
(stable typed item model, validation/reconciliation helpers, private generic
listbox core; legacy ResultList source-compatible). Focused
`cargo test --lib components::result_list --no-default-features` passes 28 tests.

Remaining work: public `KeyedResultList` component exposing the keyed/typed
path; showcase demo page section + browser fixture; consumer guide doc
(`doc/components/result_list.md` update or sibling); final verification.

Bead acceptance criteria (binding): caller provides stable key, localized
display projection, and typed payload per result; selection/activation return
the exact current keyed payload without reconstructing from display text or
index. Replacing results with reordered, duplicate-looking, relabeled,
inserted, or removed entries cannot activate a stale payload or transfer
selection to another identity. Arrow/Home/End/Enter and pointer behavior,
aria-activedescendant IDs, scrolling, empty state, and external result
replacement remain coherent. Existing ResultRow callers remain
source-compatible. Focused native/browser fixtures reproduce duplicate labels
and asynchronous result replacement.

## Task 2 — ldui-89rp (bug): DataTable auto_page_size overflow with variable-height rows

`src/components/data_table/component.rs` (~853) + `auto_page.rs`.
auto_page_size measures only the FIRST tbody row; with variable-height rows a
short first row derives an overflowing count → scrollbar + pagination together.

Fix (bead-specified, in preference order):
1. Measure the MAX offset_height across currently rendered tbody rows in the
   existing ResizeObserver callback (average is not enough).
2. Belt-and-braces: after applying a derived count, if wrapper.scroll_height >
   wrapper.offset_height by more than a row's tolerance, decrement the count
   once on the next frame (bounded, no loop).
3. Keep FALLBACK_ROW_HEIGHT for the nothing-rendered-yet first paint.

Tests: `rows_per_page_for_height` already unit-tested — add a variable-height
case (feed the max); add a wasm/DOM test with one tall row asserting no scroll
overflow at the derived count.

## Task 3 — ldui-kl55 (bug): EntityTable page-size control default identity

EntityTable passes optional `page_size_control_id` straight to Select; when
omitted the framework-owned rows-per-page control has a label but no id/name,
and multiple tables can't be repaired with one shared hard-coded value.

Design: generate a deterministic per-instance id and name when no caller
override is supplied; preserve the override; multiple EntityTables on one page
never emit duplicate identities.

Acceptance (binding): an EntityTable without page_size_control_id renders a
page-size select with non-empty id and name; two or more tables mounted
together receive unique values; caller-supplied identity remains stable and
honored; labels remain correctly associated; focused browser coverage reports
no missing-id-or-name issue for the framework control.

## Task 4 — ldui-g66e (bug): FilterSidebar search reactive accessible name

FilterSidebar's optional search input has only a placeholder — no label, no
aria-label. Add complete reactive search copy including a dedicated accessible
label, with a safe documented fallback for callers not supplying it. Keep
search value and filtering caller-owned; one stable accessible name; no added
visible layout; name not coupled to current input value.

Acceptance (binding): when search is present, its input has a nonempty
accessible name independent of placeholder and value. Label reacts to locale
changes without replacing the search signal, caret, focus, or typed text.
Search omission emits no hidden label. Multiple FilterSidebars stay
independently named. Existing callers source-compatible under a documented
fallback. Native render tests, axe, and a real-browser fixture cover empty,
typed, localized, collapsed, expanded, and right-side variants; the
input-outside-field audit no longer reports this internal control.

## Task 5 — ldui-gp34: gen-tokens sources table-hierarchy colors from ui-tokens

Sibling Rust-DeskApp master (b99962b, confirmed present on
`../Rust-DeskApp` master checkout) exposes `ui_tokens::color::table`
{ HEADER 0x004578, HEADER_CONTENT 0xFFFFFF, FILTER 0xE5F1FB, FILTER_CONTENT
0x1A1A1A, GRID 0xE0E0E0 } plus `color::table::dark`. These are exactly the
five hand-written `--color-table-*` literals in `styles/tokens.css`'s @theme
block. Wire `cargo xtask gen-tokens` to emit them from ui-tokens instead of
literals. No visual change expected — the regenerated tokens.css should be
byte-identical for these values. `tokens-fresh` gate and
`check-sibling-tokens` must stay green. Commit the regenerated tokens.css if
formatting shifts.

## Task 6 — ldui-9vs: Button typed native form semantics

Add `ButtonType { Button (default), Submit, Reset }`; render the corresponding
native `type` attribute; keep existing API/styling; form action/method stay
caller-owned; document precedence vs `attr:type`.

Acceptance (binding): caller can render Button as type button/submit/reset;
default remains button; submit activates the containing form exactly once by
click and keyboard activation; reset follows native behavior; disabled and
loading submit buttons cannot submit; variants, node refs, ripple, focus ring,
and arbitrary data/aria attributes still work; nested/form-associated edge
behavior documented; native and browser fixtures cover all three variants and
verify the emitted type attribute.

## Task 7 — ldui-z16: InputType temporal variants

Extend InputType with Date, Time, Month, Week, DateTimeLocal. Value parsing,
domain validation, timezone policy, min/max/step, formatting stay
caller-owned. Text remains default. Document interaction with `attr:type`
(prefer one authoritative path). Range/File stay out of scope.

Acceptance (binding): each new variant emits the exact valid HTML type token;
existing variants and default remain source-compatible; temporal values
round-trip through controlled value and on_input without LDUI normalization;
Field association, disabled, readonly, required, focus ring, validation
metadata intact; explicit conflicting attr:type policy documented and tested;
focused native render tests and real-browser fixtures exercise at least date,
time, month, datetime-local.

## Task 8 — ldui-3br: FilterBar optional search slot

Make FilterBar's search Children slot optional, preserving the search-first
default for existing callers. When absent, omit the search wrapper entirely;
chips, result count, Reset, Save as Default, actions, feedback use the row
width. Filter state and persistence stay caller-controlled.

Acceptance (binding): FilterBar renders with or without search; search-backed
callers render compatibly; no-search configuration emits no empty/placeholder
search region, keeps coherent region name and keyboard order, lays out
children/chips/count/actions/Reset/Save-as-Default/feedback without wasted
width at wide and compact breakpoints; omission does not change filter schema,
dataset identity, reset, persistence, or localization; focused render and
browser fixtures cover actions-only, column-filters-only summary, and ordinary
search configurations.

## Task 9 — ldui-baz4: ActionFeedback per-action message detail

Extend the keyed action outcome with optional caller-supplied plain-text
primary/detail content; ActionFeedbackState stays authoritative for semantic
color, pending behavior, retry, dismiss, live-region policy. Framework default
text remains fallback. Content ties to the keyed transition sequence so an
older completion cannot replace a newer message.

Acceptance (binding): each keyed pending/completed outcome can carry optional
attempt-specific text without HTML injection; missing content uses the
localized default; concurrent keys retain independent content while the latest
transition produces one coherent live announcement; retry, dismissal,
replacement by newer attempt, stale/duplicate completion, generation reset,
and locale replacement cannot display detail from another key or attempt;
existing state-only callers source-compatible; native and browser fixtures
cover conflict reason, partial-success counts, retryable failure detail,
concurrent actions, stale completion, and screen-reader announcement text.

## Task 10 — ldui-sh3: EntityTable controlled single-row selection

Optional controlled single-selection binding, separate from activation and
row actions. `EntityTableSelection` value carries the selected stable row key
and emits replacement proposals. EntityTable owns selected-row styling and
aria; callers keep domain state. Default non-selectable rendering stays
source-compatible. (ServerDataTable equivalent landed as ldui-4lp — mirror its
proposal-first API shape where sensible.)

Acceptance (binding): caller binds an optional selected stable row key and
receives one selection proposal per pointer or documented keyboard selection;
matching wide and compact rows get coherent selected styling and aria-selected
without changing the canonical row key; row-action controls do not select or
activate; external replacement, filtering, sorting, paging, row removal,
dataset replacement, disabled rows, and a selected key absent from the visible
page have documented fail-safe behavior; focus and selection remain distinct;
existing on_row_activate callers source-compatible; native and browser
fixtures cover master-detail selection and selected-row removal.

## Task 11 — ldui-97v: EntityColumn primary-secondary text presentation

Caller supplies canonical text plus primary and optional secondary callbacks;
EntityTable owns typography, spacing, wrapping, compact rendering, overflow
integration, accessible composition. No Office-specific labels.

Acceptance (binding): primary + optional secondary without render_with;
intentional wide and compact layouts; empty secondary leaves no
spacing/punctuation; long/unbroken values obey column overflow policy;
accessible/export value complete and not spoken twice; sorting stays tied to
canonical or typed sort key; reactive column replacement updates both lines;
existing custom renderers compatible; browser fixtures cover resizing, hiding,
compact width, forced colors.

## Task 12 — ldui-mqb: EntityTable typed summary-row emphasis

Narrow framework-owned semantic enum returned by a row callback (Standard,
Summary, Muted, Attention) — not a class-string hook. EntityTable owns tokens
and responsive styling. Presentation-only: no change to identity, ordering,
activation, selection, mutation.

Acceptance (binding): caller classifies a row without duplicating per-cell
renderers; summary styling coherent across wide/compact, survives sorting,
filtering, paging, hiding, resizing, zebra, hover, focus; legible in forced
colors; classification never changes row keys, accessible names, action
eligibility, sort values, or source data; default renders identically to
current EntityTable; focused native and browser fixtures cover a total row
moving under sort and a compact summary row.

## Task 13 — ldui-2bt3: ServerDataTable viewport-fit query sizing

Opt-in server viewport-fit policy reusing EntityTable/DataTable measurement
rules but emitting a controlled page-size query proposal instead of slicing
locally. Measurement is presentation state, not persisted preference. Offset
mode resets to page one; cursor mode requests First (tokens minted for another
size are invalid). Fixed-slice / page-size-disabled capabilities reject the
policy visibly. Displayed slice retained while the caller fetches.

Acceptance (binding): controlled offset or cursor ServerDataTable opts into a
definite-height viewport-fit policy and receives a page-size proposal matching
measured row capacity; resize, density, header/filter-row height, localized
wrapping, and column changes recompute without oscillation; cursor proposals
reset to First and never reuse a token minted for another size; offset
proposals reset to page one; rejected/failed proposals retain accepted rows
and size; stale measurements cannot overwrite newer state; page-size-disabled
fails closed; fixed page-size behavior source-compatible; native reducer tests
and real-browser negative controls cover short and tall windows, horizontal
scrollbars, retained failure, rapid resize.

## Task 14 — ldui-22gv: reusable opaque cursor history reducer

Transport-neutral `ServerCursorHistory` state/reducer composing with
ServerCursorPagination: derives accepted current query and ServerCursorPage
metadata, translates a typed navigation proposal into an opaque request
handle, commits or fails that handle after the caller-owned fetch. Must not
fetch, parse cursors, own rows, invent totals, or impose an API schema.
Query-shape replacement starts a new history generation.

Acceptance (binding): an API returning only a next cursor gets truthful
Previous/Next without page-local Vec stacks; First/Next/Previous proposals
produce opaque request handles; only the latest matching success commits;
failure retains accepted slice and history; stale/duplicate completions cannot
move navigation; search/sort/filter/page-size/dataset/access reset starts a
coherent first-slice generation; helper derives ServerCursorQuery and
ServerCursorPage values accepted by the existing component; APIs returning
both cursors can skip it; focused reducer tests cover forward, backward,
reset, retained failure, out-of-order completion.

## Task 15 — ldui-lwu: SectionHeading composition

Small SectionHeading pattern: reactive eyebrow, title, optional description,
optional status/action composition. LDUI owns semantic spacing, typography,
wrapping, responsive alignment; callers own copy/status/actions/content.
Explicit heading-level or semantic association.

Acceptance (binding): reactive localized eyebrow/title/description and
optional status/action content without page-local layout classes; valid
heading hierarchy with stable optional id for aria-labelledby; status/actions
wrap without squeezing text; readable compact and forced-colors; empty
optional regions leave no spacing; PageHeader semantics not duplicated or
weakened; showcase plus focused render/browser fixtures cover plain, status,
action, long-copy, localized variants.

## Task 16 — ldui-i95p: SearchPickerDialog pattern (after Task 1)

Built from Modal, Field/Input, PageStatePanel-compatible states, and the
keyed ResultList from Task 1. Typed composition and callbacks; caller owns
query execution, debounce, result payloads, authorization, activation. LDUI
owns dialog semantics, focus trap/return, labelled search markup,
loading/error/empty presentation, keyboard result navigation, responsive
layout.

Acceptance (binding): controlled labelled search dialog with controlled query
and async result state returns the exact stable typed payload on activation;
opening focuses search; Escape/Cancel closes and restores focus;
Arrow/Home/End/Enter operate the list; result replacement,
duplicate-looking rows, loading, empty, retained error, and stale responses
cannot activate an old payload; compact and wide layouts avoid horizontal
escape; localized copy fully reactive; Modal and ResultList stay usable
independently; native and real-browser fixtures cover focus return, typed
identity, async replacement, empty/error/retry, and two dialog instances.
