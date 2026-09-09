# Server EntityTable Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Export a canonical `ServerEntityTable` offset-paging facade that makes the complete opinionated server table experience the shortest supported path while keeping all query and data truth server-owned.

**Architecture:** Add a focused wrapper beside `ServerDataTable`, backed by the existing controlled offset query state machine and renderer. Extend the shared server column-tools bridge so resize widths participate in the same EntityTable preference payload as visibility and order. Prove the facade with a History-shaped simulated server whose accepted state is independently observable.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, daisyUI 5, `pixelproof-web`/Chromium release-WASM browser tests, Cargo xtask.

**Spec:** `docs/superpowers/specs/2026-09-09-server-entity-table-design.md`

## Global Constraints

- `EntityTable<T>` remains complete-client-snapshot only; never add a server mode to it.
- `ServerDataTable` remains the lower-level compatibility and cursor-paging primitive.
- `ServerEntityTable` supports controlled numbered offset paging only.
- Rows, query, total, filter vocabularies, endpoint capabilities, transport acceptance, failures, authorization, and navigation remain consumer-owned.
- Every domain column in the canonical facade declares an exact or text filter; missing filters fail visibly.
- Gestures emit full query or preference replacements and never optimistically mutate accepted state.
- Auto sizing proposes server capacity from a definite fill-parent viewport; fixed sizing survives resize.
- Use framework primitives and daisyUI 5 classes; preserve zero-slack style/layout ceilings.
- Run xtask commands from `C:/dev/leptos-daisyui-rs`; do not use workspace-wide fmt/clippy shortcuts.
- Preserve and review the concurrent `ldui-ova3` native-select patch before final integration.

---

### Task 1: Make server resize widths preference-owned

**Files:**
- Modify: `src/components/data_table/server_column_tools.rs`
- Modify: `src/components/data_table/server_component.rs`
- Test: `src/components/data_table/server_column_tools.rs`
- Test: `tests/server_table_column_tools_smoke.rs`

**Interfaces:**
- Consumes: `EntityTablePreferences::column_widths`, `ServerColumnToolsState`, and the header's existing `RwSignal<HashMap<&'static str, f64>>`.
- Produces: pure conversions between validated preference widths and rendered `f64` widths, plus one normalized replacement proposal per accepted resize.

- [ ] **Step 1: Write failing native tests for width hydration and replacement**

Add tests that build two columns, hydrate `{"module": 184}` from `EntityTablePreferences`, and assert the runtime width map is exactly `HashMap::from([("module", 184.0)])`. Add a replacement test that supplies runtime widths for `module` and an unknown ID, then asserts the emitted normalized preferences retain only `module: 196` and preserve page size, page-size mode, sort, order, and hidden columns.

- [ ] **Step 2: Run the focused native tests and observe RED**

Run: `cargo test -p leptos-daisyui-rs --lib components::data_table::server_column_tools::tests`

Expected: compilation fails because the planned width conversion/replacement helpers do not exist.

- [ ] **Step 3: Implement pure width helpers and state proposal plumbing**

Add helpers with these signatures:

```rust
pub(crate) fn server_runtime_widths(
    preferences: &EntityTablePreferences,
    columns: &[Column],
) -> HashMap<&'static str, f64>;

pub(crate) fn server_preferences_with_widths(
    preferences: &EntityTablePreferences,
    widths: &HashMap<&'static str, f64>,
    columns: &[Column],
) -> EntityTablePreferences;
```

Clamp and round through the existing resize bounds, ignore unknown/non-resizable columns, and preserve every unrelated preference field. Extend `ServerColumnToolsState` with a method that emits the complete normalized replacement through controlled ownership or updates uncontrolled ownership. In `ServerDataTable`, hydrate the header width signal from accepted preferences and use an effect to propose only genuinely changed width maps, avoiding an initialization callback and controlled feedback loops.

- [ ] **Step 4: Run native tests and observe GREEN**

Run: `cargo test -p leptos-daisyui-rs --lib components::data_table::server_column_tools::tests`

Expected: all server column-tools tests pass.

- [ ] **Step 5: Extend the existing browser test with width model readback**

In the simulated server-table fixture, publish the accepted `EntityTablePreferences::column_widths`. Exercise the named separator with Right, Home, and End, and assert `aria-valuemin <= aria-valuenow <= aria-valuemax`, the rendered column track changes, exactly one replacement is proposed per committed keyboard resize, and a deliberately declined replacement restores the accepted rendered width.

- [ ] **Step 6: Run the focused server-table browser lane**

Run: `cargo xtask test-server-table-column-tools`

Expected: the updated browser test passes against the release host with no browser errors.

- [ ] **Step 7: Commit the width ownership increment**

```text
feat(data-table): persist server column widths (ldui-9ke9)
```

### Task 2: Add the canonical facade and fail-closed validation

**Files:**
- Create: `src/components/data_table/server_entity_table.rs`
- Modify: `src/components/data_table/mod.rs`
- Test: `src/components/data_table/server_entity_table.rs`

**Interfaces:**
- Consumes: `ServerDataTable`, `TableQuery`, `ServerQueryCapabilities`, `ServerTablePageSizePreference`, `ServerTableColumnTools`, `EntityTablePreferenceOwnership`, filter-option types, and existing renderer/selection callback types.
- Produces: public `ServerEntityTable` and `validate_server_entity_columns(columns: &[Column]) -> Result<(), ServerEntityTableConfigurationError>`.

- [ ] **Step 1: Write failing validator and source-topology tests**

Write tests for:

```rust
assert_eq!(
    validate_server_entity_columns(&[Column::new("run", "Run")]),
    Err(ServerEntityTableConfigurationError::MissingFilter { column_id: "run" })
);
assert!(validate_server_entity_columns(&[
    Column::new("run", "Run").filterable_text().action(),
    Column::new("verdict", "Verdict").filterable(),
]).is_ok());
```

Add an `include_str!` topology assertion that the facade calls `ServerDataTable` and contains no `row_matches_*`, `compare_cells`, `page_bounds`, or local row slicing.

- [ ] **Step 2: Run the new native test and observe RED**

Run: `cargo test -p leptos-daisyui-rs --lib components::data_table::server_entity_table::tests`

Expected: compilation fails because the module, error type, validator, and component do not exist.

- [ ] **Step 3: Implement the validator and public facade**

Create a non-exhaustive configuration error enum with `MissingFilter { column_id: &'static str }`, `Display`, and stable alert copy. Implement a Leptos component whose required props are:

```rust
rows: Signal<Vec<TableRow>>,
columns: Signal<Vec<Column>>,
query: Signal<TableQuery>,
total_count: Signal<i64>,
on_query_change: Callback<TableQuery>,
loading: Signal<bool>,
control_id: String,
preference_ownership: EntityTablePreferenceOwnership,
preference_version: u16,
page_size_preference: RwSignal<ServerTablePageSizePreference>,
query_capabilities: ServerQueryCapabilities,
viewport_fit: Signal<bool>,
```

Derive current page/page size from `query`, construct the controlled offset ownership, and pass a no-op compatibility page callback because `ServerDataTable` already emits the full controlled proposal. Always construct icon-triggered `ServerTableColumnTools`; forward authoritative filter options and bounded existing render/row-action/text/selection props. Render validation failure as `role="alert"` with `data-server-entity-table-config-error` and the missing column ID. Mark the successful root with `data-server-entity-table="true"` without replacing the underlying `data-table-data-mode="server-query"` marker.

- [ ] **Step 4: Run facade native tests and observe GREEN**

Run: `cargo test -p leptos-daisyui-rs --lib components::data_table::server_entity_table::tests`

Expected: validator and topology tests pass.

- [ ] **Step 5: Run library and showcase type checks**

Run: `cargo check -p leptos-daisyui-rs --all-targets` and `cargo check -p leptos-daisyui-showcase`

Expected: both commands exit zero without new warnings.

- [ ] **Step 6: Commit the facade increment**

```text
feat(data-table): add canonical ServerEntityTable (ldui-9ke9)
```

### Task 3: Build the History-shaped accepted-server fixture

**Files:**
- Modify: `demo/src/demos/data_table.rs`
- Modify: `tests/server_table_column_tools_smoke.rs`

**Interfaces:**
- Consumes: public `ServerEntityTable` from Task 2.
- Produces: `#server-entity-history` fixture plus debug markers for accepted query, row IDs, total, proposal count, preference payload, request state, and failure state.

- [ ] **Step 1: Replace one fixture call with the wished-for facade and observe compile RED**

Add a History-shaped dataset of at least 36 rows with values deliberately arranged so each column has a match outside page one. Build a pure simulated-server function that applies `TableQuery` filters/sort/page to the complete fixture population, while passing only the accepted current page into `ServerEntityTable`. Publish separate proposed and accepted query markers. Compile before completing the host acknowledgment flow.

Run: `cargo check -p leptos-daisyui-showcase`

Expected: compilation fails at the incomplete accepted-response fixture, demonstrating that the new facade is the surface under test.

- [ ] **Step 2: Complete deterministic accepted/rejected response handling**

Each proposal increments a request token. Normal responses atomically replace accepted query, rows, and total after a short deterministic delay. A fixture-only Fail next request control retains accepted query/rows/total and publishes `retained-failure`; a stale token is ignored. Exact options are derived from the complete simulated population, never the displayed page.

- [ ] **Step 3: Compile the completed fixture and observe GREEN**

Run: `cargo check -p leptos-daisyui-showcase`

Expected: showcase compiles.

- [ ] **Step 4: Write failing browser tests for server-population truth**

Add tests that first record page-one IDs, then use real filter controls for Run, Module, Mode, Started, Duration, Verdict, Captured, Rejected, Trigger, and Build. For each filter, assert the accepted query contains the correct column/value and at least one resulting ID was absent from the original page. Assert the DOM IDs equal the separately published accepted page IDs and no client-only operation occurred before acknowledgment.

- [ ] **Step 5: Run the focused browser lane and observe RED**

Run: `cargo xtask test-server-table-column-tools`

Expected: the new History tests fail until the fixture exposes and acknowledges every requested transition.

- [ ] **Step 6: Finish the minimum fixture behavior and observe GREEN**

Implement only the missing deterministic acknowledgment behavior identified by the RED run, then rerun `cargo xtask test-server-table-column-tools` and require all tests to pass.

- [ ] **Step 7: Commit the accepted-server fixture increment**

```text
test(data-table): prove server EntityTable population truth (ldui-9ke9)
```

### Task 4: Prove the complete opinionated interaction contract

**Files:**
- Modify: `tests/server_table_column_tools_smoke.rs`
- Modify: `demo/src/demos/data_table.rs`
- Create: `doc/verification/server-entity-table-2026-09-09.md`

**Interfaces:**
- Consumes: History fixture markers from Task 3 and width preference readback from Task 1.
- Produces: release-browser evidence for standard controls, accepted-state locking, and retained failure.

- [ ] **Step 1: Add browser tests for gear, footer, sizing, sorting, and failure**

Use pointer input for gear open/hide/restore/reorder and compare DOM order to accepted preferences. Use keyboard Left/Right/Home/End on a named separator and compare geometry to width preferences. Use focus, Space, Home/Arrow, Enter on the page-size Select; prove fixed intent survives desktop resize and Auto proposes/accepts a different capacity. Sort and navigate to page two, then fail the next filter request and assert the accepted query, row IDs, total, range, page-size label, column order, widths, and proposal-independent preference state remain unchanged.

- [ ] **Step 2: Run browser tests and observe the first RED assertion**

Run: `cargo xtask test-server-table-column-tools`

Expected: at least one newly asserted interaction is absent or incorrectly exposed before the fixture/component is completed.

- [ ] **Step 3: Implement only the missing interaction wiring**

Adjust facade forwarding or fixture acknowledgment so every gesture reaches an existing `ServerDataTable` mechanism and every accepted value remains host-owned. Do not add local sort/filter/page logic to the facade.

- [ ] **Step 4: Run browser tests and observe GREEN**

Run: `cargo xtask test-server-table-column-tools`

Expected: all existing and new tests pass with no browser errors.

- [ ] **Step 5: Execute and record negative controls**

Temporarily change the simulator to filter only its current page and observe the outside-page assertion fail; restore it. Temporarily bypass `validate_server_entity_columns` and observe the missing-filter alert test fail; restore it. Temporarily apply a failed proposal optimistically and observe the accepted-state snapshot fail; restore it. Temporarily suppress width preference publication and observe the model-readback assertion fail; restore it. Record exact test names and restored results in the verification document.

- [ ] **Step 6: Run style, layout, and accessibility evidence**

Run: `cargo xtask test-style`, `cargo xtask test-layout`, and `cargo xtask test-server-table-column-tools` from the repository root. Keep all existing ceilings unchanged unless an actual engine measurement change is separately demonstrated.

- [ ] **Step 7: Commit the complete interaction evidence**

```text
test(data-table): qualify canonical server EntityTable (ldui-9ke9)
```

### Task 5: Document adoption and reconcile the native-select fix

**Files:**
- Modify: `doc/components/data_table.md`
- Modify: `doc/components/entity_table.md`
- Modify: `README.md`
- Review/modify: `src/components/entity_table/component.rs`
- Review/modify: `tests/entity_table_smoke.rs`
- Review/modify: `doc/visual-quality/README.md`
- Review/modify: `doc/visual-quality/native-select-label-arrow-overlap.md`

**Interfaces:**
- Consumes: final facade API and verified `ldui-ova3` patch.
- Produces: one supported History adoption example and accurate table-selection guidance.

- [ ] **Step 1: Verify the native-select patch independently**

Run `cargo xtask test-client-snapshot`. Confirm the real keyboard selection, fixed/Auto readback, one/two/three-digit label clearance, injected 80px failure, restored width, stylesheet freshness, and browser-error checks all execute. If the run collides with another Trunk process, preserve the log, wait for that owner to finish, and rerun without deleting shared build artifacts.

- [ ] **Step 2: Review the patch against the issue**

Confirm the 112px control remains within its footer at 1440x900, does not change IDs or values, and does not introduce a mobile requirement. Keep or revise the rulebook entry based on the observed browser evidence. Close `ldui-ova3` only after the focused test and final gate pass.

- [ ] **Step 3: Write the canonical selection and History adoption docs**

Document `ServerEntityTable` as preferred for controlled offset-paged populations, `ServerDataTable` as cursor/lower-level compatibility, and `EntityTable<T>` as complete client snapshot. Include a compiling History-shaped example and an explicit list of consumer-owned row mapping, query translation, filter vocabularies, request correlation, navigation, authorization, persistence, and failure copy.

- [ ] **Step 4: Run doc/source guards**

Run: `cargo xtask verify`

Expected: formatting, clippy, builds, native tests, and source guards pass.

- [ ] **Step 5: Commit documentation and reconciled select work**

```text
fix(entity-table): preserve native Auto label clearance (ldui-ova3)
docs(data-table): document ServerEntityTable adoption (ldui-9ke9)
```

### Task 6: Review, verify, land, and clean repository topology

**Files:**
- Review: every path in `git diff fork/main...HEAD --name-only`
- Update: `doc/verification/server-entity-table-2026-09-09.md`
- Update: Beads `ldui-9ke9` and `ldui-ova3`

**Interfaces:**
- Consumes: all prior task commits.
- Produces: one reviewed, pushed `main` with no extra local branches or worktrees.

- [ ] **Step 1: Review the complete diff and stale-base risk**

Run `git rev-parse --short HEAD`, `git rev-parse --short main`, `git diff --check`, and `git diff --name-only HEAD..main -- <every touched path>`. Review the facade for local data operations, permissive ownership defaults, missing extension forwarding, unsafe preference feedback, and mismatch between docs and code.

- [ ] **Step 2: Run the final required gate**

Run: `cargo xtask verify-full`

Record the real process exit and actual xtask summary. Immediately afterward rerun `bd ready --json`, open, in-progress, and blocked inventories.

- [ ] **Step 3: Close completed issues and push issue state**

Close `ldui-9ke9` and `ldui-ova3` with exact verification evidence, re-read both records, then run `bd dolt push` and wait for `Push complete.`

- [ ] **Step 4: Rebase and push main**

Run `git pull --rebase`, resolve only reviewed changes, rerun affected gates if the candidate changes, then run `git push`. Read back `HEAD` and `fork/main` and require identical hashes.

- [ ] **Step 5: Remove verified stray local topology**

Run `git worktree list --porcelain` and `git branch -vv`. Remove only additional registered worktrees after proving every commit is reachable from pushed `main`; delete only additional local branches after the same reachability proof. Prune remote-tracking references with `git fetch --prune`. Do not delete upstream `origin/*` branches, which are not local branches and belong to the standalone source remote.

- [ ] **Step 6: Final readback**

Require one registered worktree, one local branch (`main`), `main...fork/main` with no ahead/behind count, no in-progress completed issue, and no tracked or staged changes. Retained `.tmp/` evidence may remain untracked until its ownership is established; do not delete it merely to make status visually empty.
