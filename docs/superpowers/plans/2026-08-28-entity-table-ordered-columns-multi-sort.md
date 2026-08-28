# EntityTable Ordered Columns and Multi-Sort Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend controlled `EntityTable` preferences with deterministic column order and multi-column sorting, including migration-safe serialization and accessible browser operations.

**Architecture:** Keep `EntityTablePreferences.sort` typed as `EntitySort` and preserve the historical public `System`, `Ascending`, and `Descending` variants; add `Multiple` for ordered clauses and custom serde that emits the canonical clause array while accepting the legacy enum payload. Add `column_order` as a normalized preference, keep controlled ownership persistence-neutral, and apply both preferences through pure model helpers before the renderer. Plain header activation remains a single-sort cycle; Shift+activation adds, changes, or removes a clause without disturbing the other priorities. Column-order controls live in a keyed list inside the chooser, and the shared table resize math now supports focusable ARIA separators with arrow/Home/End operation.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, serde/serde_json, daisyUI 5, `pixelproof-web`, `ldui-audit`, cargo xtask.

**Spec:** `C:\dev\4iiz-Office\Architecture-Construction-Guidelines.md` sections 6.3, 7.1, and Phase 0B; Bead `ldui-wdo`.

## Global Constraints

- Controlled ownership emits one normalized full replacement per UI operation and performs no browser I/O.
- `EntityTablePreferencePersistence::LegacyLocalStorage` remains an explicitly named compatibility path and reads legacy single-sort JSON.
- Empty sort clauses mean server/system row order; ties preserve incoming dataset order.
- Column order contains each declared column at most once; unknown and duplicate IDs are removed, and omitted columns are appended in declaration order so required columns cannot disappear.
- Only the primary sorted header owns `aria-sort`; secondary direction and priority are exposed through accessible labels, semantic data attributes, and the demo model oracle.
- Use existing `Button`, `Menu`, shared pagination/resize/chooser mechanics, daisyUI 5 classes, and `merge_classes!`; add no page-local Office implementation.
- New native and browser assertions must be observed failing for the intended missing behavior before production implementation, then pass after implementation.
- The final candidate must pass `cargo xtask verify-full`; use scoped xtask commands rather than workspace-wide fmt/clippy.
- The Bead is one landing unit, so implementation, tests, docs, and this plan land in one scoped Conventional Commit.

---

### Task 1: Define the canonical preference schema and legacy migration

**Files:**
- Modify: `src/components/entity_table/types.rs`
- Modify: `src/components/entity_table/storage.rs`
- Modify: `src/components/entity_table/tests.rs`
- Modify: `src/components/entity_table/mod.rs`

**Interfaces:**
- Produces: `EntitySortDirection::{Ascending, Descending}`.
- Produces: `EntitySortColumn::{ascending, descending}` with public `column` and `direction` fields.
- Produces: `EntitySort::{System, Ascending, Descending, Multiple}` plus `ascending`, `descending`, `multiple`, `clauses`, `is_system`, `primary`, `clause_for`, and `priority_for`, while keeping `EntityTablePreferences.sort: EntitySort`.
- Produces: `EntityTablePreferences.column_order: Vec<String>` with `#[serde(default)]` migration behavior.
- Consumes later: model sorting, header semantics, and demo/debug serialization.

- [x] **Step 1: Write failing schema and migration tests**

Add tests that independently assert the canonical wire value and legacy migration:

```rust
#[test]
fn entity_sort_serializes_clauses_and_migrates_legacy_single_sort() {
    let canonical = serde_json::json!([
        {"column":"rank","direction":"ascending"},
        {"column":"client","direction":"descending"}
    ]);
    let sort: EntitySort = serde_json::from_value(canonical.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&sort).unwrap(),
        canonical
    );
    assert_eq!(
        serde_json::from_str::<EntitySort>(r#"{"Descending":{"column":"rank"}}"#).unwrap(),
        EntitySort::descending("rank")
    );
}

#[test]
fn legacy_preferences_without_column_order_decode_with_system_order() {
    let payload = r#"{"schema_version":1,"page_size":25,"sort":"System","hidden_columns":[],"column_widths":{}}"#;
    let decoded = decode_preferences(payload, 1, &columns());
    assert_eq!(decoded.sort, EntitySort::System);
    assert_eq!(
        serde_json::to_value(decoded).unwrap()["column_order"],
        serde_json::json!(["client", "rank", "office", "actions"])
    );
}
```

- [x] **Step 2: Run the focused unit test and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode entity_sort_serializes_clauses_and_migrates_legacy_single_sort -- --exact
```

Expected: the test compiles, then fails because an ordered sort array cannot deserialize and normalized preferences do not serialize `column_order` yet.

- [x] **Step 3: Implement the canonical types and serde compatibility**

Extend the single-value enum with a multi-clause variant while preserving the historical variants and constructor surface:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitySortDirection { Ascending, Descending }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySortColumn {
    pub column: String,
    pub direction: EntitySortDirection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum EntitySort {
    #[default]
    System,
    Ascending { column: String },
    Descending { column: String },
    Multiple { clauses: Vec<EntitySortColumn> },
}
```

Implement custom `Deserialize` using an untagged canonical-list/legacy-enum representation. Map legacy `System` to an empty list and legacy `Ascending`/`Descending` to one clause. Add `#[serde(default)] pub column_order: Vec<String>` to `EntityTablePreferences`.

- [x] **Step 4: Run focused schema tests and verify GREEN**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode entity_sort -- --nocapture
```

Expected: all schema/migration tests pass with no warnings.

---

### Task 2: Normalize and execute ordered columns plus lexicographic multi-sort

**Files:**
- Modify: `src/components/entity_table/model.rs`
- Modify: `src/components/entity_table/tests.rs`
- Modify: `src/components/entity_table/mod.rs`

**Interfaces:**
- Consumes: canonical `EntitySort` clauses and `EntityTablePreferences.column_order` from Task 1.
- Produces: `next_sort_additive`, `ordered_columns`, `move_column`, and `EntityColumnMove::{Earlier, Later}`.
- Produces: `sorted_indices` lexicographic comparison with once-per-row key extraction per active clause.
- Produces: deterministic normalization that removes unknown/duplicate sort and order IDs, appends missing columns, preserves required columns, and clamps existing fields.

- [x] **Step 1: Write failing pure-model tests**

Add literal, behavior-focused cases:

```rust
#[test]
fn multi_sort_uses_clause_priority_and_preserves_system_ties() {
    let rows = vec![
        Row { id: "r1", name: "Zulu", rank: 1 },
        Row { id: "r2", name: "Alpha", rank: 1 },
        Row { id: "r3", name: "Bravo", rank: 2 },
    ];
    let sort = EntitySort::multiple([
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::descending("client"),
    ]);
    assert_eq!(sorted_indices(&rows, &columns(), &sort), [0, 1, 2]);
}

#[test]
fn normalization_canonicalizes_sort_and_column_order() {
    let mut supplied = EntityTablePreferences::new(1);
    supplied.sort = EntitySort::multiple([
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::descending("rank"),
        EntitySortColumn::ascending("missing"),
    ]);
    supplied.column_order = vec!["office".into(), "office".into(), "missing".into()];
    supplied.hidden_columns.insert("client".into());
    let normalized = normalize_preferences(&supplied, 1, &columns());
    assert_eq!(normalized.sort.clauses(), [EntitySortColumn::ascending("rank")]);
    assert_eq!(normalized.column_order, ["office", "client", "rank", "actions"]);
    assert!(!normalized.hidden_columns.contains("client"));
}

#[test]
fn additive_sort_and_column_move_are_reversible() {
    let sort = next_sort_additive(&EntitySort::ascending("rank"), "client", true);
    assert_eq!(sort.clauses(), [
        EntitySortColumn::ascending("rank"),
        EntitySortColumn::ascending("client"),
    ]);
    let mut preferences = EntityTablePreferences::new(1);
    assert!(move_column(&mut preferences, &columns(), "office", EntityColumnMove::Earlier));
    assert_eq!(preferences.column_order, ["client", "office", "rank", "actions"]);
}
```

- [x] **Step 2: Run model tests and verify RED**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode multi_sort -- --nocapture
cargo test -p leptos-daisyui-rs --lib --features test-mode normalization_canonicalizes_sort_and_column_order -- --exact
```

Expected: failures identify missing lexicographic sorting, normalization, additive cycling, and column movement.

- [x] **Step 3: Implement minimal model behavior**

Build active comparators in clause order, extract text keys exactly once per row per active text clause, and return the first non-equal comparison after applying direction. Keep the incoming index order for full ties. Normalize clauses first-wins by column ID and normalize column order first-wins plus declaration-order append. Make `next_sort` keep its current replace/cycle contract and add `next_sort_additive` for Shift+activation. Implement adjacent swaps through `move_column` and use `ordered_columns` for rendering.

- [x] **Step 4: Run all EntityTable unit tests and verify GREEN**

Run:

```powershell
cargo test -p leptos-daisyui-rs --lib --features test-mode entity_table -- --nocapture
```

Expected: all EntityTable tests pass; key-extraction call counts equal `rows × active text clauses`.

---

### Task 3: Render ordered columns and expose accessible multi-sort/reorder operations

**Files:**
- Modify: `src/components/entity_table/component.rs`
- Modify: `src/components/entity_table/types.rs`
- Modify: `demo/src/demos/client_snapshot_list.rs`
- Modify: `tests/entity_table_smoke.rs`

**Interfaces:**
- Consumes: `ordered_columns`, `move_column`, `next_sort`, and `next_sort_additive` from Task 2.
- Produces DOM: `data-entity-sort-priority`, `data-entity-sort-direction`, `data-entity-sort-summary`, `data-entity-column-order`, and `data-entity-column-move`.
- Produces behavior: plain click/Enter replaces the sort; Shift+click/Shift+Enter updates the multi-sort; chooser buttons move every column earlier/later and preserve focus through keyed rendering.
- Produces demo oracle: `window.__APP_DEBUG__.state().state["entity_table.preferences"]` matches the rendered order/sort.

- [x] **Step 1: Extend the browser journey with failing DOM/model/a11y assertions**

Before component changes, extend `client_snapshot_list_contract_works_end_to_end` to assert:

```rust
// Shift+activate a second header and verify primary-only aria-sort plus both priorities.
assert_eq!(multi["priorities"], json!([1, 2]));
assert_eq!(multi["ariaSort"], json!(["ascending", null]));
assert_eq!(multi["modelSort"], json!([
    {"column":"client","direction":"ascending"},
    {"column":"status","direction":"ascending"}
]));

// Move Status earlier using the real chooser button and cross-check DOM/model order.
assert_eq!(reordered["headers"], json!(["Status", "Client", "Case type", "Received", "Actions"]));
assert_eq!(reordered["modelOrder"], json!(["status", "client", "case_type", "received", "actions"]));
assert_eq!(reordered["activeLabel"], json!("Move Status earlier, position 1 of 5"));
```

Also assert `localStorage.getItem("ldui-entity-table:client-snapshot-demo")` remains `null` in the controlled demo and that no browser/axe errors are introduced.

- [x] **Step 2: Run the browser lane and verify RED**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --browser
```

Expected: the existing journey reaches the new assertions and fails because multi-sort priority attributes, reorder buttons, and controlled model state are absent.

- [x] **Step 3: Implement ordered rendering and accessible controls**

Use normalized order for headers, wide cells, and the default compact row. In each sortable header:

- set `aria-sort` only when the column is the primary clause;
- expose clause priority/direction as semantic data attributes;
- show `▲1`, `▼1`, `▲2`, etc. while keeping inactive `↕`;
- include current priority and the next plain/Shift operation in `aria-label`;
- branch on `MouseEvent::shift_key()` to call additive vs. replacement sorting.

Add an `aria-live="polite"` sort summary. Inside the chooser, render a keyed ordered list of every column with `Button` controls labelled `Move {header} earlier/later, position {n} of {total}`. Disable the boundary action and keep the keyed node stable so focus follows a moved column.

Switch the demo to accepted controlled ownership, write accepted preferences to the existing debug-state bridge, and remove its `storage_key` prop. Keep native legacy-storage tests as the compatibility proof.

- [x] **Step 4: Run the browser lane and verify GREEN**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --browser
```

Expected: DOM and model sort/order agree after each action; Shift+keyboard behavior, focus retention, primary-only `aria-sort`, compact/wide ordering, audits, and browser error capture pass.

- [x] **Step 5: Demonstrate targeted break-and-revert controls**

Temporarily make additive activation replace all clauses, run the focused browser assertion and verify it fails on the second priority, then revert. Temporarily make `move_column` return without swapping, run its focused native test and verify it fails on literal order, then revert. Re-run both focused lanes green and confirm `git diff` contains none of the injected faults.

---

### Task 3A: Close accessibility and downstream-review gaps

**Files:**
- Modify: `src/components/data_table/{header.rs,resize.rs,mod.rs}`
- Modify: `src/components/entity_table/{component.rs,types.rs,tests.rs}`
- Modify: `tests/{common/mod.rs,entity_table_smoke.rs}`
- Modify: `doc/components/entity_table.md`

- [x] **Step 1: Prove the reported gaps with failing tests**

The browser journey failed first on boundary reorder focus, then on real CDP
Shift+Enter. The new vendored axe run exposed the chooser's invalid list
structure. Shared resize-math tests failed before the keyboard helper existed.

- [x] **Step 2: Implement shared keyboard resize and complete semantics**

Make both table separators focusable and expose min/max/current value semantics.
Use Left/Right for 16-pixel steps and Home/End for bounds; keep controlled
EntityTable updates persistence-neutral. Add visible focus styling.

- [x] **Step 3: Fix boundary focus, explicit action labels, Shift+keyboard, storage proof, and chooser structure**

Fallback to the enabled opposite reorder control at a boundary, include current
sort direction/priority plus exact plain/Shift next actions, handle modified
keyboard activation exactly once, seed storage before remount, and render the
mixed chooser content in a semantic `div` container around its inner menu/list.

- [x] **Step 4: Preserve legacy source variants and document the required consumer migration**

Retain `EntitySort::{System, Ascending, Descending}`, add `Multiple`, and prove
legacy construction/patterns in native and rustdoc tests. Document that existing
preference struct literals need `column_order` and exhaustive matches need a
`Multiple` arm when the framework revision is vendored.

- [x] **Step 5: Demonstrate and revert a targeted keyboard-resize mutation**

- [x] **Step 6: Obtain a clean follow-up code review**

---

### Task 4: Document the public contract and run the candidate gates

**Files:**
- Modify: `doc/components/entity_table.md`
- Modify: `doc/ci-cd.md` only if the existing pattern-lane description needs new coverage named
- Modify: `docs/superpowers/plans/2026-08-28-entity-table-ordered-columns-multi-sort.md` (check completed steps)

**Interfaces:**
- Documents canonical JSON, legacy migration, column-order normalization, Shift+activation, primary `aria-sort`, reorder controls, controlled persistence, and reset behavior.

- [x] **Step 1: Update EntityTable documentation**

Add examples using:

```rust
preferences.sort = EntitySort::multiple([
    EntitySortColumn::ascending("status"),
    EntitySortColumn::descending("received"),
]);
preferences.column_order = vec![
    "status".into(), "client".into(), "received".into(), "actions".into(),
];
```

State that normal activation creates/replaces one sort, Shift+activation extends the ordered sort clauses, duplicate/unknown IDs are removed first-wins, omitted columns append in declaration order, reset restores declaration order, and controlled mode never touches localStorage.

- [x] **Step 2: Run focused native verification**

Run:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
```

Expected: PASS.

- [x] **Step 3: Run the required final gate**

Run:

```powershell
cargo xtask verify-full
```

Expected: every native, browser, accessibility, style/layout, and release-Trunk step passes with its real exit code.

- [x] **Step 4: Reconcile and land the live queue**

After the long gate, re-run `bd ready --json`, plus open/in-progress/blocked lists. If still complete, close and re-read `ldui-wdo`, then:

```powershell
git diff --check
git diff --name-only HEAD..main -- <every touched path>
git add <owned files>
git commit -m "feat(entity-table): add ordered columns and multi-sort (ldui-wdo)"
git pull --rebase
bd dolt push
git push
git status -sb
```

Verify the pushed remote SHA equals local `HEAD`, the tree is clean, and a final fresh Beads inventory is empty.
