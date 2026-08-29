# Snapshot Table Opinionated Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the framework-owned client-snapshot table path, including keyboard-operable shared-table sorting, atomic validated dataset presentation, the preferred aligned filter row and semantic visual hierarchy, sort-stable geometry, complete page/filter feedback, reactive EntityTable metadata, deterministic row-action focus recovery, and a rendered Field ID contract.

**Architecture:** Add pure reducers and typed wrappers before rendering components. Keep transport, authorization, routes, persistence, and domain data in consumers. Migrate the client-snapshot showcase to the complete controlled composition, then prove each behavior through native tests and real Chrome/WASM oracles. Existing lower-level APIs remain compatible.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, serde/serde_json, daisyUI 5, `pixelproof-web`, `ldui-audit`, cargo xtask, Chrome.

**Spec:** `docs/superpowers/specs/2026-08-28-snapshot-table-opinionated-foundation-design.md`; Beads `ldui-w1e`, `ldui-gbs`, `ldui-ifj`, and `ldui-ifj.1` through `ldui-ifj.5`.

## Global Constraints

- Follow red-green-refactor: add one focused native or browser oracle, run it and observe the expected failure, implement the smallest behavior, then rerun it green.
- Demonstrate every new browser oracle with a targeted break-and-revert negative control before closing its Bead.
- Use framework `Button`, `Input`, `Select`, alert, pagination, and token APIs; do not add raw page-local controls or arbitrary styling.
- Keep the new path persistence-neutral and free of network, database, route, session, and local-storage side effects.
- Preserve source compatibility for existing `DataTable`, `ServerDataTable`, `EntityTable`, `FilterBar`, `ActiveFilterChips`, `DatasetSelector`, `ListPage`, and `AsyncDataSection` call sites.
- Render one-to-one filters only in the aligned table filter row. Keep only global/non-column controls, state summary, Reset, and Save as Default in the utility `FilterBar`; both regions share one controlled model.
- Use generated semantic table tokens and stable fixed column tracks. Sorting may change body order and sort state only, never shell geometry.
- Use focused xtask gates while a Bead is active. Run `cargo xtask verify-full` only on the final candidate tree, then reread every Beads queue before landing.
- Layer D2 is explicitly inapplicable here: the framework reports callbacks but performs no transport or durable write. Consumer repositories own completion-barrier proof.

---

### Task 1: Make shared DataTable sorting keyboard accessible (`ldui-w1e`)

**Files:**
- Modify: `src/components/data_table/header.rs`
- Modify: `demo/src/demos/data_table.rs`
- Modify: `tests/reactivity_smoke.rs`
- Modify: `tests/style_smoke.rs`
- Modify: `src/components/data_table/README.md`

**Contract:** A sortable `th` contains one native framework `Button`; the `th` retains `aria-sort`, the button directly names the localized column, current sort state, and next action, and pointer/Enter/Space all reach the existing sort callback once. A non-sortable header has no button or tab stop. The resize separator remains a sibling and cannot trigger sorting.

- [x] **Step 1: Add the failing browser oracle**

Add `data_table_sort_is_keyboard_operable_for_client_and_server_tables` to `tests/reactivity_smoke.rs`. Give the client fixture `id="keyboard-sort-table"`, and publish the server query through `debug_state["server_datatable.query"]`. For each table, focus its first sort button, send Enter and Space with `Key::Enter` and `Key::Space`, and assert the debug oracle advances exactly once per key. Also assert the non-sortable header has no sort control (its independent resize separator remains focusable) and run the vendored axe critical/serious check plus browser-error capture.

- [x] **Step 2: Run the browser suite and verify RED**

Run:

```powershell
cargo xtask test-reactivity
```

Expected: the new test cannot find a sort button in either shared table; existing pointer sorting remains green.

- [x] **Step 3: Render the framework Button inside sortable headers**

Replace the sortable `th` click handler with this ownership shape:

```rust
<th aria-sort=aria_sort style=width_style>
    <Button
        style=ButtonStyle::Ghost
        size=ButtonSize::Sm
        class="min-h-0 h-auto"
        on_click=Callback::new(move |_| on_sort.run(column_id.clone()))
    >
        {column.header.clone()}
        {sort_indicator}
    </Button>
    {resize_separator}
</th>
```

Keep the exact existing separator range/key behavior and propagation guards. Render plain text for non-sortable columns.

- [x] **Step 4: Verify GREEN and the negative control**

Rerun `cargo xtask test-reactivity`. Then temporarily remove the button from the client fixture, confirm the new test fails at its focus/name assertion, revert that fault, and rerun green. Run `cargo xtask test-style` to prove the button uses the canonical primitive and no drift ceiling changes.

- [x] **Step 5: Document, commit, push, and close**

Completed in pushed commit `b470ab9`. At that revision, focused reactivity
passed 27/27, style 7/7, `cargo xtask verify` 14/14, and the then-current
`cargo xtask verify-full` passed 18/18 before `ldui-w1e` closed. The gate later
expanded to 19 steps and the independently selectable reactivity lane to 32
checks; this line preserves the historical result rather than describing the
current gate.

---

### Task 2: Freeze reviewed invariants and owner visual decisions (`ldui-ifj.5`)

**Files:**
- Modify: `Future-Architecture.md`
- Modify: `docs/superpowers/specs/2026-08-28-snapshot-table-opinionated-foundation-design.md`
- Modify: `docs/superpowers/plans/2026-08-28-snapshot-table-opinionated-foundation.md`
- Update: Beads `ldui-ifj`, `ldui-ifj.1`, `ldui-ifj.2`, `ldui-ifj.3`, `ldui-ifj.5`, and `ldui-gbs`

**Contract:** Before implementation, freeze private validated state, framework-issued request handles, typed identity-critical page configs, keyed concurrent actions, schema-projected defaults, semantic column generations, dataset/access-scoped focus, aligned column filters, semantic blue bands/faint grid, and sort-stable geometry.

- [x] **Step 1: Revise the governing documents**

Replace every permissive or contradictory interface in the approved spec and plan. Record the owner's 4iiz-etl filter-row decision and table palette in `Future-Architecture.md`; do not leave the earlier detached one-to-one filter wording as a competing path.

- [x] **Step 2: Update affected Bead acceptance criteria**

Make each child independently enforce the relevant invariants. `ldui-ifj.1` owns private state/request handles/typed critical configs/concurrent actions; `ldui-ifj.2` owns the hybrid filter placement and schema-projected payload; `ldui-ifj.3` owns semantic-generation cache invalidation and focus scope; `ldui-gbs` owns geometry and tokenized visual hierarchy. Re-read each issue after writing it.

- [x] **Step 3: Audit for stale design language**

Search for obsolete caller tokens, public orthogonal state fields, arbitrary `SnapshotViewDefaults<F>`, opaque selector/table slots, one-action state, detached one-to-one controls, content-driven canonical layouts, and unscoped focus recovery. The search result must contain no normative stale path.

- [x] **Step 4: Save and close the design review**

Run `git diff --check`, commit the coherent document/Bead checkpoint, push it, verify the remote, and close `ldui-ifj.5` only after all eight review findings and the two owner decisions are represented in both the spec and implementation steps.

---

### Task 3: Add the atomic dataset presentation reducer (`ldui-ifj.1`)

**Files:**
- Add: `src/patterns/snapshot_table.rs`
- Add: `src/patterns/snapshot_table/tests.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**

```rust
pub struct SnapshotRequestHandle<V> { /* private sequence + dataset */ }
pub struct SnapshotActionHandle<K> { /* private generation + sequence + key */ }
pub struct SnapshotData<R, V, M> { /* private atomic snapshot fields */ }
pub struct SnapshotTableState<R, V, E, M, K> { /* private reducer state */ }
pub struct SnapshotTableView<'a, R, V, E, M, K> { /* read-only derived view */ }
pub enum SnapshotResponseDisposition {
    Applied,
    IgnoredConsumed,
    IgnoredStale,
    IgnoredMismatchedDataset,
}
```

- [x] **Step 1: Write failing pure transition tests**

Cover initial load, initial failure/retry, displayed-to-replacing, retained failure/retry, successful atomic replacement, duplicate/consumed handles, older handles, a matching sequence carrying the wrong dataset identity, checked sequence exhaustion, and access replacement. Assert ignored responses preserve dataset, rows, revision, count, metadata, actions, and generation byte-for-byte. Prove no public constructor can mint or decrease a handle.

- [x] **Step 2: Run the focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table::tests -- --nocapture
```

Expected: compilation fails because the dataset types and transition methods do not exist.

- [x] **Step 3: Implement the minimal pure model**

Add `start_request`, `complete`, `fail`, `replace_access`, generation-bound action transitions, and read-only view helpers. `start_request` mints the handle internally. Apply a response only when its still-active opaque handle and dataset match. Swap the complete `SnapshotData` in one reducer operation and consume the handle. Mint `LocalResultSummary` only from the current displayed binding and reject a summary from an older generation/revision.

- [x] **Step 4: Verify GREEN**

Run the focused command again, followed by `cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table -- --nocapture`.

---

### Task 4: Add validated runtime state, PageStatePanel, ActionFeedback, and SnapshotTablePage (`ldui-ifj.1`)

**Files:**
- Modify: `src/patterns/snapshot_table.rs`
- Modify: `src/patterns/snapshot_table/tests.rs`
- Add: `src/patterns/page_state_panel.rs`
- Add: `src/patterns/action_feedback.rs`
- Add: `src/patterns/snapshot_table_page.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`
- Add: `tests/snapshot_table_smoke.rs`

**Interfaces:** Keep phase/content/access private inside `SnapshotTableState`. Add a read-only `SnapshotTableView`, generation-bound `LocalResultSummary`, keyed concurrent `ActionFeedbackModel<K>`, and complete reactive `PageStatePanelTexts` and `ActionFeedbackTexts`. Add `SnapshotDatasetSelectorConfig<V>` and `SnapshotEntityTableConfig<R>` whose public fields cannot supply selected/displayed identity, rows, revision, or generation.

- [x] **Step 1: Write failing precedence and renderer tests**

Native cases must prove: contradictory phase/content/access combinations have no public constructor; a stale `LocalResultSummary` is rejected; expired/forbidden suppress mounted content; initial loading/error replace content; empty/no-results come only from the matching displayed generation; replacing and retained-error keep content mounted; two distinct action keys may remain pending concurrently; updating/dismissing one preserves the other; only the latest transition is announced; and preference feedback is not represented by the action model.

Add a browser fixture with literal IDs `snapshot-page`, `snapshot-page-dataset`, `snapshot-page-filters`, `snapshot-page-feedback`, and `snapshot-page-table`. Assert the DOM order is header, distinct selector, KPI, filters, feedback, then table; assert page/selector/table generation markers agree; and assert retained transitions leave the same table node mounted.

- [x] **Step 2: Run focused native/browser checks and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table -- --nocapture
cargo xtask test-client-snapshot
```

Expected: missing state types and component markers prevent the new assertions from compiling or locating the required regions.

- [x] **Step 3: Implement typed render selection and components**

Implement one pure render-decision function and make `SnapshotTablePage` consume it. The page itself renders the selector and table from configs that omit all identity-critical fields. Use framework alerts, skeletons, and Buttons. Concurrent keyed feedback uses one latest live announcement without calling focus. The page owns only typed binding, sizing, vertical rhythm, slot order, retained mounting, and stable `data-*` markers.

- [x] **Step 4: Verify GREEN and demonstrate the DOM oracle**

Run both focused commands. Temporarily swap the filter and selector slot markers, confirm the browser ordering assertion fails, revert, and rerun green. Capture browser errors and WASM panics in the smoke test.

---

### Task 5: Complete controlled, localized filters and explicit defaults (`ldui-ifj.2`)

**Files:**
- Modify: `src/patterns/filter_bar.rs`
- Modify: `src/patterns/dataset_selector.rs`
- Modify: `src/patterns/active_filter_chips.rs`
- Add: `src/patterns/filter_bar/tests.rs`
- Modify: `src/patterns/contracts.rs`
- Modify: `src/components/entity_table/types.rs`
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`
- Modify: `tests/snapshot_table_smoke.rs`

**Interfaces:**

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalFilterDefaults { /* private schema-ordered values */ }
pub struct SnapshotViewDefaults { /* private filters + table */ }
pub struct EntityColumnFilter { pub column_id: &'static str, /* renderer */ }
```

Add reactive complete `FilterBarTexts`, `DatasetSelectorTexts`, and `ActiveFilterTexts`. Represent the optional save binding behind `SnapshotDefaultSave`; its payload can be created only by `FilterSchema::project_defaults`. Add static/reactive `EntityColumnFilters`, keyed by stable column ID, for the second `thead` row. Existing layout-only `FilterBar` calls retain their current API.

- [ ] **Step 1: Write failing payload and interaction tests**

Native tests serialize `SnapshotViewDefaults` and assert its only top-level keys are `filters` and `table`. Project a schema-ordered map through a validated `FilterSchema`; reject undeclared keys and the exact dataset selector. Add a negative consumer fixture with an `office_id` dataset member and prove it cannot be passed/serialized as defaults.

Browser tests exercise pointer, Enter, and Space on Reset and Save. Assert exactly one save callback per explicit enabled Save, while filter edits, Reset, dataset selection, locale replacement, and feedback rendering emit zero saves. Assert one-to-one filters render only beneath their columns, utility-only filters render only above, no clear/reset action is duplicated, filter controls cannot trigger sorting, and reorder/hide operations keep controls on the correct stable column ID. Assert dirty/clean/pending/disabled reasons, active chip summaries, localized result counts, and live pending/saved/conflict/failure feedback.

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_view_defaults -- --nocapture
cargo test -p leptos-daisyui-rs --test snapshot_table_smoke --features browser-tests -- --ignored --nocapture
```

Expected: the payload type and complete FilterBar actions/texts are absent, and the browser cannot locate the named Save control.

- [ ] **Step 3: Implement the complete controlled contracts**

Keep all values/signals consumer-owned. Embed `ActiveFilterChips` once within the utility `FilterBar`; render one Reset and one explicit Save as Default. Render `EntityColumnFilters` as a second header row with one `th` per visible column and event isolation around each control. Save invokes the supplied callback with schema-projected local defaults and table preferences only. No state transition except activation may call it. Every framework-owned visible or accessible string comes from a reactive text signal.

- [ ] **Step 4: Verify GREEN and negative controls**

Run the native and browser commands. Temporarily call save from Reset, confirm the callback-count assertion fails, revert, and rerun green. Replace the locale signal during the same mounted page and assert labels change without altering dataset, page, sort, widths, order, visibility, filters, or dirty state.

---

### Task 6: Make opinionated table visuals and geometry stable (`ldui-gbs`)

**Files:**
- Modify: `xtask/src/gen_tokens.rs`
- Regenerate: `styles/tokens.css`
- Modify: `src/components/data_table/header.rs`
- Modify: `src/components/data_table/filter.rs`
- Modify: `src/components/data_table/component.rs`
- Modify: `src/components/data_table/server_component.rs`
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/components/entity_table/types.rs`
- Modify: `tests/reactivity_smoke.rs`
- Modify: `tests/entity_table_smoke.rs`
- Modify: `tests/style_audit_smoke.rs`
- Modify: `tests/layout_audit_smoke.rs`

**Contract:** Canonical snapshot and server-query tables use generated semantic dark-blue/white header, light-blue/dark filter band, faint collapsed row/column grid, opt-in zebra, a stable `colgroup`, and fixed reserved sort-indicator slots. Sorting updates body order/state without replacing header/filter nodes or moving any shell track.

- [ ] **Step 1: Write failing semantic-token and browser geometry tests**

Pin generated table-header/filter/grid tokens to the existing shared `ui_tokens` palette. In desktop and narrow horizontally scrolled fixtures, capture the viewport/table/header/filter bounding boxes, every column x/width, grid-line positions, `scrollLeft`, and header/filter node identities. Sort via pointer, Enter, and Space with a fixture whose first page has radically different content widths after sorting. Assert the complete shell remains within `0.5px`, nodes remain identical, and only body row order plus sort state/announcement change. Repeat for localized longer headings and a filtered/paged table.

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode table_geometry -- --nocapture
cargo xtask test-reactivity
cargo test -p leptos-daisyui-rs --test entity_table_smoke --features browser-tests -- --ignored --nocapture
```

Expected: content-driven layout and non-keyed header maps move/recreate shell geometry; semantic table tokens are absent.

- [ ] **Step 3: Implement stable tracks and preserved header/filter nodes**

Generate `--color-table-header`, `--color-table-header-content`, `--color-table-filter`, `--color-table-filter-content`, and `--color-table-grid`. Add stable geometry helpers that render a `colgroup` from column definitions plus controlled widths. Use fixed table layout in canonical configs. Convert header/filter maps to keyed `For` nodes so sort-state changes update reactive attributes/indicator content in place. Always render a fixed-width unsorted/sorted indicator slot. Keep low-level compatibility layout selectable and source compatible.

- [ ] **Step 4: Apply the opinionated visual hierarchy**

Use semantic generated classes on the canonical table only: dark-blue header/white content, light-blue filter/dark content, faint collapsed borders on header/filter/body cells, and zebra disabled by default. Verify focus rings and resize handles remain visible against both bands and in forced colors.

- [ ] **Step 5: Prove the negative control and visual result**

Temporarily remove the `colgroup` or reserved indicator width, confirm the bounding-box oracle fails with the moved columns, revert, and rerun green. Run `cargo xtask test-style`, `cargo xtask test-layout`, and the page visual lane; review desktop and narrow captures rather than approving by metric alone.

- [ ] **Step 6: Save the geometry checkpoint**

Update table docs, run `cargo xtask verify`, close `ldui-gbs`, commit with `fix(entity-table): keep sort geometry stable (ldui-gbs)`, push, and verify remote equality.

---

### Task 7: Migrate and visually prove the client-snapshot reference page (`ldui-ifj.1`, `ldui-ifj.2`)

**Files:**
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`
- Modify: `tests/snapshot_table_smoke.rs`
- Add or update: `tests/visual/baselines/client-snapshot-list*.png`
- Modify: `doc/patterns/client-snapshot-list.md`
- Modify: `doc/visual-quality/visual-test-plan.md`

- [ ] **Step 1: Add failing page-level race and state oracles**

Expose a serializable demo debug model containing displayed dataset, requested dataset, opaque generation label, revision, row count, panel kind, filter summary/placements, save state, current columns/semantic generation, sort, page, and focused row/action scope. Add browser cases for displaying, replacing, stale response ignored, retained error, no local results, concurrent action conflict, and preference failure. In every case compare the DOM to the debug model rather than accepting DOM-only evidence.

- [ ] **Step 2: Run the page-scoped commands and verify RED**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
```

Expected: the legacy composition lacks the complete debug fields and state markers required by the new oracles.

- [ ] **Step 3: Migrate the demo to the canonical signal flow**

Use one private-field `SnapshotTableState` signal. Supply `SnapshotDatasetSelectorConfig`, `SnapshotFilterConfig`, and `SnapshotEntityTableConfig`; let the page inject requested/displayed values, rows, dataset identity, generation, panels, and feedback. Place column-mapped controls in the aligned table filter row and only global/domain controls in the utility `FilterBar`. Do not retain alias signals for dataset labels or rows.

- [ ] **Step 4: Add reviewed desktop and narrow visual states**

Capture named component-region baselines for displaying, replacing, retained error, no results, action conflict, preference failure, and a fully populated filter row. Review each image for wrapping, overlap, clipping, focus, semantic header/filter colors, faint grid, stable tracks, typography, shapes, shadows, and default-variant intent. Update the visual test plan with its responsive matrix and accepted baseline rationale.

- [ ] **Step 5: Verify all A/B/C/D1 layers and close child Beads**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
cargo xtask test-style
cargo xtask test-layout
cargo make test-visual
```

For the stale-response oracle, temporarily accept any token, observe the dataset/revision/row-count assertion fail, revert, and rerun green. Close `ldui-ifj.1` and `ldui-ifj.2` only after focused gates pass; commit and push their implementation with scoped Conventional Commit subjects.

---

### Task 8: Make EntityTable columns and compact rendering reactive (`ldui-ifj.3`)

**Files:**
- Modify: `src/components/entity_table/types.rs`
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/components/entity_table/model.rs`
- Modify: `src/components/entity_table/tests.rs`
- Modify: `src/components/entity_table/mod.rs`
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`

**Interfaces:**

```rust
pub enum EntityColumns<T: 'static> {
    Static(Vec<EntityColumn<T>>),
    Reactive(Signal<Vec<EntityColumn<T>>, LocalStorage>),
}

pub enum EntityCompactRow<T: 'static> {
    Static(EntityRowRenderer<T>),
    Reactive(Signal<EntityRowRenderer<T>, LocalStorage>),
}
```

Implement `From<Vec<EntityColumn<T>>>` and `From<Signal<Vec<EntityColumn<T>>, LocalStorage>>`; mirror those conversions for compact rows. Accept both through `#[prop(into)]` so static call sites remain source compatible. Internally expose an opaque semantic generation that advances on every reactive vector replacement and participates in sort-cache invalidation.

- [ ] **Step 1: Write failing native normalization tests**

Prove a changed declaration removes unknown/non-sortable sort clauses plus unknown width/order/visibility IDs, appends newly declared IDs, preserves page size and surviving preferences, increments semantic generation, and uses the newest comparator/key callbacks with unchanged row `Rc` and sort. Prove a label-only change leaves page and all preference values unchanged while safely invalidating the sort cache.

- [ ] **Step 2: Add a failing mounted locale test**

Mount reactive columns and compact-row copy, change locale, and assert wide headers, chooser labels, sort names, compact labels, and accessible names update in place while row/page/sort/order/width/visibility debug state remains equal.

- [ ] **Step 3: Run focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode entity_table -- --nocapture
cargo test -p leptos-daisyui-rs --test entity_table_smoke --features browser-tests -- --ignored --nocapture
```

Expected: `Vec` is captured in stored values and rendered labels/comparators do not react to the signal replacement.

- [ ] **Step 4: Implement reactive wrappers and normalization**

Read the active column vector and semantic generation at each reactive render/model boundary. Include the generation in `SortedIndexCache` reuse. Normalize controlled preferences against current stable/sortable IDs without emitting spurious preference callbacks for a label-only change. Make the default compact renderer read current columns and make an explicit compact renderer use the reactive wrapper.

- [ ] **Step 5: Verify GREEN and the locale negative control**

Run both focused commands. Temporarily omit semantic generation from the cache key while replacing a comparator with unchanged rows/sort, confirm the ordered-index assertion fails, revert, and rerun green. Also temporarily retain initial chooser labels, confirm the mounted locale test fails, revert, and rerun green.

---

### Task 9: Add deterministic row-action focus recovery (`ldui-ifj.3`)

**Files:**
- Modify: `src/components/entity_table/types.rs`
- Modify: `src/components/entity_table/model.rs`
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/components/entity_table/tests.rs`
- Modify: `src/components/entity_table/mod.rs`
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`

**Contract:** Add `EntityRowAction` with a stable action ID and framework-owned `data-entity-row-action` marker. Record dataset/access generation, row key, action ID, and visible position when focus enters a marked action. Recover to a neighbor only for source-row removal within the same generation and only when the same action is rendered, enabled, visible, and focusable. Filtering/paging hide falls back to the named table region; dataset/access change clears recovery; a user who already moved focus is never interrupted.

- [ ] **Step 1: Write failing pure focus-target tests**

Add a pure selector that consumes the prior focus record, prior/current source keys, current visible keys, and current generation. Cover a middle-row deletion, last-row deletion, sorted order, locally filtered order, page collapse, disabled/hidden/missing matching action, an unchanged row, a row hidden by filter/page while still in source, external deletion, dataset replacement, expired/forbidden access, and already-moved focus. Assert the returned target is either `RowAction { row_key, action_id }`, `TableRegion`, `Clear`, or `NoChange`.

- [ ] **Step 2: Run the focused unit tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode focus_target -- --nocapture
```

Expected: the focus target model and stable action marker do not exist.

- [ ] **Step 3: Implement the marker, tracking, and post-render recovery**

Make the EntityTable region programmatically focusable with `tabindex="-1"` and a stable region node reference. Capture focus only from marked actions inside the table together with the page-supplied opaque focus scope. After source/visible rows change, compute the target from the same sorted/filtered/paged indices used by rendering, verify generation and removal-vs-hide semantics, then resolve and eligibility-check the marker inside the table subtree. Never query from consumer code, cross a generation, steal focus after it moved elsewhere, or fall back to `document.body`.

- [ ] **Step 4: Add failing browser focus cases**

In both wide and compact modes, focus a Delete action, remove the row, and assert `document.activeElement` identifies the expected eligible row/action. Repeat after sorting and a last-row page collapse. Hide a focused row by filtering/paging without deleting it and assert table-region fallback, not neighbor focus. Disable or hide the candidate action and assert table-region fallback. Replace the dataset and access state and assert no cross-generation neighbor focus. Reject one action without removing its row and assert focus remains on the initiating element. Simulate an external same-generation row removal and assert the same recovery path.

- [ ] **Step 5: Verify browser GREEN and the negative control**

Run:

```powershell
cargo test -p leptos-daisyui-rs --test entity_table_smoke --features browser-tests -- --ignored --nocapture
```

Temporarily choose source-row order instead of rendered visible order, confirm the sorted deletion case fails, revert, and rerun green. Capture browser errors, WASM panics, and axe critical/serious results.

- [ ] **Step 6: Close and save the EntityTable child**

Run the complete EntityTable unit/browser scope, update its rustdoc and pattern documentation, close `ldui-ifj.3`, commit with a scoped subject, push, and verify remote equality.

---

### Task 10: Prove unique Field associations in a real WASM form (`ldui-ifj.4`)

**Files:**
- Modify: `demo/src/demos/field.rs`
- Modify: `tests/reactivity_smoke.rs`
- Modify only if the fixture fails: `src/components/field/component.rs`
- Modify only if the fixture fails: `src/components/field/tests.rs`
- Modify: `src/components/field/README.md`

- [ ] **Step 1: Add the six-control fixture and failing browser assertions**

Render at least three Field-wrapped Inputs and three Field-wrapped Selects in one form, mixing help, error, required, and disabled states. Assert all six control IDs are non-empty and unique; each label `for` resolves to exactly one control; each `aria-describedby` token resolves to exactly one help/error node; and the document contains no duplicate IDs. Run the vendored axe critical/serious check and browser-error capture.

- [ ] **Step 2: Run the focused browser suite**

Run:

```powershell
cargo xtask test-reactivity
```

Expected result determines implementation: if current monotonic allocation is correct, the new regression test is immediately green and no allocator rewrite is justified; if it fails, capture the exact duplicate/association before changing production code.

- [ ] **Step 3: Demonstrate the oracle with a duplicate-ID fault**

Temporarily force the second fixture control to use the first control's ID. Confirm the duplicate-ID or label-target assertion fails with both IDs in its diagnostic. Revert the fixture fault and rerun green.

- [ ] **Step 4: Fix only if current production allocation failed**

If Step 2 exposed a production defect, add a native failing allocator test and implement a deterministic owner/page-scoped allocator without changing consumer call sites. If Step 2 was green, retain the current monotonic allocator and document that browser evidence supersedes the stale vendored Office observation.

- [ ] **Step 5: Close and save the Field child**

Run the focused Field native tests plus `cargo xtask test-reactivity`, close `ldui-ifj.4`, commit with `test(field): prove unique rendered associations (ldui-ifj.4)`, push, and verify the remote branch contains it.

---

### Task 11: Finish documentation, broad verification, epic closure, and landing (`ldui-ifj`)

**Files:**
- Modify: `doc/patterns/client-snapshot-list.md`
- Modify: `doc/visual-quality/visual-test-plan.md`
- Modify: `doc/ci-cd.md` only if a new focused command is added
- Modify: `README.md`
- Modify: `AGENTS.md` only for durable repo-specific execution knowledge discovered during implementation
- Modify: repo-local skill instructions only if they exist and the implementation changed their durable contract

- [ ] **Step 1: Document the one canonical composition**

Show complete typed construction of private-field `SnapshotTableState`, framework-issued `SnapshotRequestHandle`, generation-bound views, typed selector/table configs, keyed concurrent actions, reactive columns, the utility-plus-aligned-filter composition, schema-projected `SnapshotViewDefaults`, stable geometry, `EntityRowAction`, and `SnapshotTablePage`. State which legacy lower-level APIs remain available, that defaults exclude dataset identity, and that D2 transport verification belongs in consumers.

- [ ] **Step 2: Run focused gates from the final candidate tree**

Run:

```powershell
cargo xtask test-reactivity
cargo xtask test-style
cargo xtask test-layout
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
cargo make test-visual
```

Read each command's own exit code and inspect its report/artifacts rather than trusting wrapper output.

- [ ] **Step 3: Run the clean broad gate**

Announce its coverage, then run:

```powershell
cargo xtask verify-full
```

Expected: all native tests, audits, browser suites, visual checks, and release Trunk build pass. If any command changes generated files, review and commit only intentional output, then rerun the affected gate.

- [ ] **Step 4: Reconcile the live Beads queue**

Immediately after the long gate, rerun `bd ready --json` and the open, in-progress, and blocked queries. File discovered work with `discovered-from:ldui-ifj`; do not close the epic while any child or newly discovered required issue remains. Close `ldui-ifj` only when every acceptance criterion is proven.

- [ ] **Step 5: Land and independently verify persistence**

Review `git diff --check`, status, commit history, and `HEAD..main` overlap for every touched file. Commit any final documentation/metadata, then run:

```powershell
git pull --rebase
bd dolt push
git push
git status
```

Re-read Bead state after its write, fetch the remote branch, and assert local `HEAD` equals the remote tracking ref. The landing is complete only when the worktree is clean, status says up to date, and no open/in-progress/blocked Bead remains in this approved scope.
