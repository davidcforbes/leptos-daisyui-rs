# Snapshot Table Opinionated Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the framework-owned client-snapshot table path, including keyboard-operable shared-table sorting, atomic dataset presentation, complete page/filter feedback, reactive EntityTable metadata, deterministic row-action focus recovery, and a rendered Field ID contract.

**Architecture:** Add pure reducers and typed wrappers before rendering components. Keep transport, authorization, routes, persistence, and domain data in consumers. Migrate the client-snapshot showcase to the complete controlled composition, then prove each behavior through native tests and real Chrome/WASM oracles. Existing lower-level APIs remain compatible.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, serde/serde_json, daisyUI 5, `pixelproof-web`, `ldui-audit`, cargo xtask, Chrome.

**Spec:** `docs/superpowers/specs/2026-08-28-snapshot-table-opinionated-foundation-design.md`; Beads `ldui-w1e`, `ldui-ifj`, and `ldui-ifj.1` through `ldui-ifj.4`.

## Global Constraints

- Follow red-green-refactor: add one focused native or browser oracle, run it and observe the expected failure, implement the smallest behavior, then rerun it green.
- Demonstrate every new browser oracle with a targeted break-and-revert negative control before closing its Bead.
- Use framework `Button`, `Input`, `Select`, alert, pagination, and token APIs; do not add raw page-local controls or arbitrary styling.
- Keep the new path persistence-neutral and free of network, database, route, session, and local-storage side effects.
- Preserve source compatibility for existing `DataTable`, `ServerDataTable`, `EntityTable`, `FilterBar`, `ActiveFilterChips`, `DatasetSelector`, `ListPage`, and `AsyncDataSection` call sites.
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

**Contract:** A sortable `th` contains one native framework `Button`; the `th` retains `aria-sort`, the button is named by the localized column heading, and pointer/Enter/Space all reach the existing sort callback once. A non-sortable header has no button or tab stop. The resize separator remains a sibling and cannot trigger sorting.

- [ ] **Step 1: Add the failing browser oracle**

Add `data_table_sort_is_keyboard_operable_for_client_and_server_tables` to `tests/reactivity_smoke.rs`. Give the client fixture `id="keyboard-sort-table"`, and publish the server query through `debug_state["server_datatable.query"]`. For each table, focus its first sort button, send Enter and Space with `Key::Enter` and `Key::Space`, and assert the debug oracle advances exactly once per key. Also assert the non-sortable header has no sort control (its independent resize separator remains focusable) and run the vendored axe critical/serious check plus browser-error capture.

- [ ] **Step 2: Run the browser suite and verify RED**

Run:

```powershell
cargo xtask test-reactivity
```

Expected: the new test cannot find a sort button in either shared table; existing pointer sorting remains green.

- [ ] **Step 3: Render the framework Button inside sortable headers**

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

- [ ] **Step 4: Verify GREEN and the negative control**

Rerun `cargo xtask test-reactivity`. Then temporarily remove the button from the client fixture, confirm the new test fails at its focus/name assertion, revert that fault, and rerun green. Run `cargo xtask test-style` to prove the button uses the canonical primitive and no drift ceiling changes.

- [ ] **Step 5: Document, commit, push, and close**

Document the keyboard contract and shared client/server coverage. Run `cargo xtask verify`, inspect `bd ready --json` plus open/in-progress/blocked snapshots, close `ldui-w1e`, and commit with `fix(data-table): make shared sorting keyboard accessible (ldui-w1e)`. Push and verify the remote branch points at the new commit.

---

### Task 2: Add the atomic dataset presentation reducer (`ldui-ifj.1`)

**Files:**
- Add: `src/patterns/snapshot_table.rs`
- Add: `src/patterns/snapshot_table/tests.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**

```rust
pub struct DatasetRequest<V> { pub token: u64, pub dataset: V }
pub struct SnapshotData<R, V, M> {
    pub dataset: V,
    pub rows: Rc<Vec<R>>,
    pub revision: String,
    pub total_rows: usize,
    pub metadata: Option<M>,
}
pub enum DatasetPresentation<R, V, E, M> {
    NeverLoaded,
    InitialLoading { request: DatasetRequest<V> },
    InitialError { request: DatasetRequest<V>, error: E },
    Displaying { snapshot: SnapshotData<R, V, M> },
    Replacing { displayed: SnapshotData<R, V, M>, request: DatasetRequest<V> },
    RetainedError { displayed: SnapshotData<R, V, M>, request: DatasetRequest<V>, error: E },
}
pub enum DatasetResponseDisposition { Applied, IgnoredStale, IgnoredMismatchedDataset }
```

- [ ] **Step 1: Write failing pure transition tests**

Cover initial load, initial failure/retry, displayed-to-replacing, retained failure/retry, successful atomic replacement, duplicate responses, older tokens, and a matching token carrying the wrong dataset identity. Assert ignored responses preserve dataset, rows, revision, count, and metadata byte-for-byte.

- [ ] **Step 2: Run the focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table::tests -- --nocapture
```

Expected: compilation fails because the dataset types and transition methods do not exist.

- [ ] **Step 3: Implement the minimal pure model**

Add `start_request`, `accept_response`, `record_failure`, and read-only helpers for displayed snapshot/identity, requested identity, busy, retained data, and error. Apply a response only when token and dataset both equal the active request. Swap the complete `SnapshotData` in one enum replacement.

- [ ] **Step 4: Verify GREEN**

Run the focused command again, followed by `cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table -- --nocapture`.

---

### Task 3: Add orthogonal runtime state, PageStatePanel, ActionFeedback, and SnapshotTablePage (`ldui-ifj.1`)

**Files:**
- Modify: `src/patterns/snapshot_table.rs`
- Modify: `src/patterns/snapshot_table/tests.rs`
- Add: `src/patterns/page_state_panel.rs`
- Add: `src/patterns/action_feedback.rs`
- Add: `src/patterns/snapshot_table_page.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`
- Add: `tests/snapshot_table_smoke.rs`

**Interfaces:** Add `SnapshotContentState::{Ready, EmptyDataset, NoFilterResults}`, `SnapshotAccessState::{Allowed, Expired, Forbidden}`, keyed `ActionFeedbackState<K>`, and `SnapshotTablePresentation<R,V,E,M,K> { dataset, content, access, action }`. Add complete reactive `PageStatePanelTexts` and `ActionFeedbackTexts` structs.

- [ ] **Step 1: Write failing precedence and renderer tests**

Native cases must prove: expired/forbidden suppress mounted content; initial loading/error replace content; empty/no-results replace content; replacing and retained-error keep content mounted; action feedback coexists with retained rows; and preference feedback is not represented by `ActionFeedbackState`.

Add a browser fixture with literal IDs `snapshot-page`, `snapshot-page-dataset`, `snapshot-page-filters`, `snapshot-page-feedback`, and `snapshot-page-table`. Assert the DOM order is header, distinct selector, KPI, filters, feedback, then table, and assert retained transitions leave the same table node mounted.

- [ ] **Step 2: Run focused native/browser checks and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_table -- --nocapture
cargo test -p leptos-daisyui-rs --test snapshot_table_smoke --features browser-tests -- --ignored --nocapture
```

Expected: missing state types and component markers prevent the new assertions from compiling or locating the required regions.

- [ ] **Step 3: Implement typed render selection and components**

Implement one pure render-decision function and make `SnapshotTablePage` consume it. Use framework alerts, skeletons, and Buttons. Feedback announcements use status/alert semantics without calling focus. The page owns only sizing, vertical rhythm, slot order, retained mounting, and stable `data-*` markers.

- [ ] **Step 4: Verify GREEN and demonstrate the DOM oracle**

Run both focused commands. Temporarily swap the filter and selector slot markers, confirm the browser ordering assertion fails, revert, and rerun green. Capture browser errors and WASM panics in the smoke test.

---

### Task 4: Complete controlled, localized filters and explicit defaults (`ldui-ifj.2`)

**Files:**
- Modify: `src/patterns/filter_bar.rs`
- Modify: `src/patterns/dataset_selector.rs`
- Modify: `src/patterns/active_filter_chips.rs`
- Add: `src/patterns/filter_bar/tests.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/lib.rs`
- Modify: `tests/snapshot_table_smoke.rs`

**Interfaces:**

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotViewDefaults<F> {
    pub filters: F,
    pub table: EntityTablePreferences,
}
```

Add reactive complete `FilterBarTexts`, `DatasetSelectorTexts`, and `ActiveFilterTexts`. Represent the optional typed save binding behind a `SnapshotDefaultSave` wrapper so existing layout-only `FilterBar` calls do not gain an unconstrained generic parameter.

- [ ] **Step 1: Write failing payload and interaction tests**

Native tests serialize `SnapshotViewDefaults` and assert its only top-level keys are `filters` and `table`. Construct it through a validated `FilterSchema` and assert the dataset selector cannot appear among local filter fields.

Browser tests exercise pointer, Enter, and Space on Reset and Save. Assert exactly one save callback per explicit enabled Save, while filter edits, Reset, dataset selection, locale replacement, and feedback rendering emit zero saves. Assert dirty/clean/pending/disabled reasons, active chip summaries, localized result counts, and live pending/saved/conflict/failure feedback.

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode snapshot_view_defaults -- --nocapture
cargo test -p leptos-daisyui-rs --test snapshot_table_smoke --features browser-tests -- --ignored --nocapture
```

Expected: the payload type and complete FilterBar actions/texts are absent, and the browser cannot locate the named Save control.

- [ ] **Step 3: Implement the complete controlled contracts**

Keep all values/signals consumer-owned. Embed `ActiveFilterChips` once within complete `FilterBar`; render one Reset and one explicit Save as Default. Save invokes the supplied callback with the current filters and table preferences only. No state transition except activation may call it. Every framework-owned visible or accessible string comes from a reactive text signal.

- [ ] **Step 4: Verify GREEN and negative controls**

Run the native and browser commands. Temporarily call save from Reset, confirm the callback-count assertion fails, revert, and rerun green. Replace the locale signal during the same mounted page and assert labels change without altering dataset, page, sort, widths, order, visibility, filters, or dirty state.

---

### Task 5: Migrate and visually prove the client-snapshot reference page (`ldui-ifj.1`, `ldui-ifj.2`)

**Files:**
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`
- Modify: `tests/snapshot_table_smoke.rs`
- Add or update: `tests/visual/baselines/client-snapshot-list*.png`
- Modify: `doc/patterns/client-snapshot-list.md`
- Modify: `doc/visual-quality/visual-test-plan.md`

- [ ] **Step 1: Add failing page-level race and state oracles**

Expose a serializable demo debug model containing displayed dataset, requested dataset, revision, row count, panel kind, filter summary, save state, current columns, sort, page, and focused row/action. Add browser cases for displaying, replacing, stale response ignored, retained error, no local results, action conflict, and preference failure. In every case compare the DOM to the debug model rather than accepting DOM-only evidence.

- [ ] **Step 2: Run the page-scoped commands and verify RED**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
```

Expected: the legacy composition lacks the complete debug fields and state markers required by the new oracles.

- [ ] **Step 3: Migrate the demo to the canonical signal flow**

Use one `SnapshotTablePresentation` signal. Derive selector requested/displayed values, KPI data, table rows, table dataset identity, panels, and feedback from that signal. Compose the fixed `SnapshotTablePage` slots and complete `FilterBar`; do not retain alias signals for dataset labels or rows.

- [ ] **Step 4: Add reviewed desktop and narrow visual states**

Capture named component-region baselines for displaying, replacing, retained error, no results, action conflict, and preference failure. Review each image for wrapping, overlap, clipping, focus, typography, shapes, shadows, and default-variant intent. Update the visual test plan with its responsive matrix and accepted baseline rationale.

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

### Task 6: Make EntityTable columns and compact rendering reactive (`ldui-ifj.3`)

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

Implement `From<Vec<EntityColumn<T>>>` and `From<Signal<Vec<EntityColumn<T>>, LocalStorage>>`; mirror those conversions for compact rows. Accept both through `#[prop(into)]` so static call sites remain source compatible.

- [ ] **Step 1: Write failing native normalization tests**

Prove a changed declaration removes unknown sort/width/order/visibility IDs, appends newly declared IDs, preserves page size and surviving preferences, and uses the newest comparator/key callbacks. Prove a label-only change leaves page and all preference values unchanged.

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

Read the active column vector at each reactive render/model boundary. Normalize controlled preferences against current stable IDs without emitting spurious preference callbacks for a label-only change. Make the default compact renderer read current columns and make an explicit compact renderer use the reactive wrapper.

- [ ] **Step 5: Verify GREEN and the locale negative control**

Run both focused commands. Temporarily retain the initial chooser labels, confirm the mounted locale test fails, revert, and rerun green.

---

### Task 7: Add deterministic row-action focus recovery (`ldui-ifj.3`)

**Files:**
- Modify: `src/components/entity_table/types.rs`
- Modify: `src/components/entity_table/model.rs`
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/components/entity_table/tests.rs`
- Modify: `src/components/entity_table/mod.rs`
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`

**Contract:** Add `EntityRowAction` with a stable action ID and framework-owned `data-entity-row-action` marker. Record row key, action ID, and visible position when focus enters a marked action. When that row disappears, focus the same action at the same clamped position in the actual filtered/sorted/paged visible order, otherwise focus the preceding row, otherwise the named table region. If the row remains after an action failure, do not move focus.

- [ ] **Step 1: Write failing pure focus-target tests**

Add a pure selector that consumes the prior focused row/action/position plus the newly visible row keys. Cover a middle-row deletion, last-row deletion, sorted order, locally filtered order, page collapse, no matching action, an unchanged row, and an external deletion. Assert the returned target is either `RowAction { row_key, action_id }`, `TableRegion`, or `NoChange`.

- [ ] **Step 2: Run the focused unit tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode focus_target -- --nocapture
```

Expected: the focus target model and stable action marker do not exist.

- [ ] **Step 3: Implement the marker, tracking, and post-render recovery**

Make the EntityTable region programmatically focusable with `tabindex="-1"` and a stable region node reference. Capture focus only from marked actions inside the table. After the rendered visible rows change, compute the target from the same sorted/filtered/paged indices used by rendering, then resolve the matching marker inside the table subtree. Never query from consumer code and never fall back to `document.body`.

- [ ] **Step 4: Add failing browser focus cases**

In both wide and compact modes, focus a Delete action, remove the row, and assert `document.activeElement` identifies the expected row/action. Repeat after sorting, filtering, and moving to the last row of a page that collapses. Reject one action without removing its row and assert focus remains on the initiating element. Simulate an external row removal and assert the same recovery path.

- [ ] **Step 5: Verify browser GREEN and the negative control**

Run:

```powershell
cargo test -p leptos-daisyui-rs --test entity_table_smoke --features browser-tests -- --ignored --nocapture
```

Temporarily choose source-row order instead of rendered visible order, confirm the sorted deletion case fails, revert, and rerun green. Capture browser errors, WASM panics, and axe critical/serious results.

- [ ] **Step 6: Close and save the EntityTable child**

Run the complete EntityTable unit/browser scope, update its rustdoc and pattern documentation, close `ldui-ifj.3`, commit with a scoped subject, push, and verify remote equality.

---

### Task 8: Prove unique Field associations in a real WASM form (`ldui-ifj.4`)

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

### Task 9: Finish documentation, broad verification, epic closure, and landing (`ldui-ifj`)

**Files:**
- Modify: `doc/patterns/client-snapshot-list.md`
- Modify: `doc/visual-quality/visual-test-plan.md`
- Modify: `doc/ci-cd.md` only if a new focused command is added
- Modify: `README.md`
- Modify: `AGENTS.md` only for durable repo-specific execution knowledge discovered during implementation
- Modify: repo-local skill instructions only if they exist and the implementation changed their durable contract

- [ ] **Step 1: Document the one canonical composition**

Show complete typed construction of `DatasetPresentation`, `SnapshotTablePresentation`, reactive columns, complete `FilterBar`, `SnapshotViewDefaults`, `EntityRowAction`, and `SnapshotTablePage`. State which legacy lower-level APIs remain available, that defaults exclude dataset identity, and that D2 transport verification belongs in consumers.

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
