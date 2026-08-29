# Keyed ResultList Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a source-compatible keyed ResultList path that preserves business identity across asynchronous replacements and returns the exact current typed payload.

**Architecture:** Keep the existing `ResultList` public signature as a legacy adapter over one private generic listbox core. Add `ResultListItem<T>` and `KeyedResultList<T>` for stable-key consumers; the core owns keyed selection, live item lookup, validation, listbox semantics, and rendering while an explicit replacement policy preserves legacy reset behavior.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, daisyUI 5, PixelProof browser harness, Beads.

**Spec:** `doc/plans/2026-08-29-keyed-result-list-design.md`

## Global Constraints

- Preserve the existing `ResultList`, `ResultRow`, `on_select(ResultRow)`, and `on_selection_change(Option<usize>)` source behavior.
- `ResultListItem<T>` payloads use `T: Clone + Send + Sync + 'static` with the standard synchronized Leptos `Signal` storage.
- Stable keys are non-empty after trimming and unique within the current list; invalid lists fail visibly and emit no activation.
- Activation resolves the selected key against the latest supplied items at event time.
- Do not add to or run the 32-test reactivity inventory; use one separate, selectively run `result_list_smoke` target.
- Use `apply_patch` for edits, scoped rustfmt, focused native checks while implementing, and defer the selected real-browser run until the repository is committed and pushed as required by the session landing policy.

## Pause checkpoint (2026-08-29)

Tasks 1 and 2 are complete and committed on `main`:

- `d90ca43` adds the stable typed item model, key validation, reconciliation,
  current-payload lookup, and collision-free option IDs.
- `7ac1640` moves the legacy component onto the private generic listbox core
  with the explicit `ResetFirst` compatibility policy.
- `$env:CARGO_BUILD_JOBS='2'; cargo test --lib components::result_list
  --no-default-features` passes all 28 focused tests with no warnings.

Resume at Task 3. `KeyedResultList<T>`, the showcase fixture, focused browser
contract, consumer guide, and Bead closure have not yet been implemented.

---

### Task 1: Stable result identity model and pure state transitions

**Files:**
- Modify: `src/components/result_list/types.rs`
- Modify: `src/components/result_list/tests.rs`

**Interfaces:**
- Consumes: existing `ResultRow`, `move_selection`, `select_first`, and `select_last`.
- Produces: `ResultListItem<T>`, `ResultListKeyError`, `validate_result_list_items`, `reconcile_result_key`, `move_result_key`, `current_result_item`, and `keyed_option_dom_id`.

- [x] **Step 1: Add failing model and reconciliation tests**

Append focused tests with concrete duplicate-looking rows and distinct payloads:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CasePayload {
    case_number: &'static str,
    generation: u8,
}

fn keyed(key: &str, title: &str, case_number: &'static str, generation: u8)
    -> ResultListItem<CasePayload>
{
    ResultListItem::new(
        key,
        ResultRow::new(title),
        CasePayload { case_number, generation },
    )
}

#[test]
fn keyed_items_separate_duplicate_display_from_payload_identity() {
    let first = keyed("case-a", "Alex Morgan", "A-100", 1);
    let second = keyed("case-b", "Alex Morgan", "B-200", 1);
    assert_eq!(first.row, second.row);
    assert_ne!(first.key, second.key);
    assert_ne!(first.payload, second.payload);
}

#[test]
fn validation_rejects_blank_and_duplicate_keys() {
    let blank = vec![keyed("   ", "Blank", "A-100", 1)];
    assert_eq!(
        validate_result_list_items(&blank),
        Err(ResultListKeyError::EmptyKey { index: 0 })
    );

    let duplicate = vec![
        keyed("case-a", "Alex Morgan", "A-100", 1),
        keyed("case-a", "Alex Morgan", "B-200", 1),
    ];
    assert_eq!(
        validate_result_list_items(&duplicate),
        Err(ResultListKeyError::DuplicateKey {
            key: "case-a".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        })
    );
}

#[test]
fn replacement_preserves_key_and_uses_latest_payload() {
    let replacement = vec![
        keyed("case-b", "Alejandro Morgan", "B-200", 2),
        keyed("case-a", "Alex Morgan", "A-100", 1),
    ];
    assert_eq!(
        reconcile_result_key(Some("case-b"), &replacement),
        Some("case-b".to_owned())
    );
    assert_eq!(
        current_result_item(&replacement, "case-b").unwrap().payload.generation,
        2
    );
}

#[test]
fn removal_falls_back_to_first_and_keyboard_uses_current_order() {
    let rows = vec![
        keyed("case-c", "Third", "C-300", 1),
        keyed("case-a", "First", "A-100", 1),
    ];
    assert_eq!(reconcile_result_key(Some("case-b"), &rows), Some("case-c".into()));
    assert_eq!(move_result_key(Some("case-c"), 1, &rows), Some("case-a".into()));
    assert_eq!(move_result_key(Some("case-a"), 1, &rows), Some("case-a".into()));
}

#[test]
fn option_dom_ids_are_collision_free_for_arbitrary_key_bytes() {
    assert_ne!(keyed_option_dom_id(7, "a b"), keyed_option_dom_id(7, "a-b"));
    assert_ne!(keyed_option_dom_id(7, "é"), keyed_option_dom_id(7, "e"));
    assert!(keyed_option_dom_id(7, "a b").starts_with("ld-result-list-7-option-"));
}
```

- [x] **Step 2: Run the focused test and verify the red state**

Run:

```powershell
$env:CARGO_BUILD_JOBS='2'; cargo test --lib components::result_list --no-default-features
```

Expected: compilation fails because `ResultListItem` and the keyed helper functions do not exist.

- [x] **Step 3: Implement the public model and validation**

Add these definitions to `types.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResultListItem<T> {
    pub key: String,
    pub row: ResultRow,
    pub payload: T,
}

impl<T> ResultListItem<T> {
    pub fn new(key: impl Into<String>, row: ResultRow, payload: T) -> Self {
        Self { key: key.into(), row, payload }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultListKeyError {
    EmptyKey { index: usize },
    DuplicateKey {
        key: String,
        first_index: usize,
        duplicate_index: usize,
    },
}
```

Implement `Display` for `ResultListKeyError` with index- and key-specific diagnostics. Implement the pure helpers with these exact signatures:

```rust
pub fn validate_result_list_items<T>(
    items: &[ResultListItem<T>],
) -> Result<(), ResultListKeyError>;

pub fn reconcile_result_key<T>(
    current: Option<&str>,
    items: &[ResultListItem<T>],
) -> Option<String>;

pub fn move_result_key<T>(
    current: Option<&str>,
    delta: i32,
    items: &[ResultListItem<T>],
) -> Option<String>;

pub fn current_result_item<T: Clone>(
    items: &[ResultListItem<T>],
    key: &str,
) -> Option<ResultListItem<T>>;

pub(crate) fn keyed_option_dom_id(instance: u64, key: &str) -> String;
```

`reconcile_result_key` retains `current` if present, otherwise clones the first key. `move_result_key` finds the current key's latest position, delegates clamping to `move_selection`, and returns the key at the resulting current-order index. `keyed_option_dom_id` encodes every UTF-8 byte as two lowercase hexadecimal digits after the instance prefix, so it needs neither CSS escaping nor a collision-prone hash.

- [x] **Step 4: Format and rerun the focused native tests**

Run:

```powershell
rustfmt --edition 2024 src/components/result_list/types.rs src/components/result_list/tests.rs
$env:CARGO_BUILD_JOBS='2'; cargo test --lib components::result_list --no-default-features
```

Expected: every ResultList unit test passes.

- [x] **Step 5: Commit the identity model**

```powershell
git add -- src/components/result_list/types.rs src/components/result_list/tests.rs
git commit -m "feat(result-list): add stable typed result identity (ldui-r1z)"
```

### Task 2: Private keyed listbox core and legacy adapter

**Files:**
- Create: `src/components/result_list/core.rs`
- Modify: `src/components/result_list/component.rs`
- Modify: `src/components/result_list/mod.rs`
- Modify: `src/components/result_list/tests.rs`

**Interfaces:**
- Consumes: all Task 1 result-item and identity helpers.
- Produces: private `ResultListCore<T>` and `ResultReplacementPolicy::{ResetFirst, PreserveKey}`; keeps public `ResultList` behavior unchanged.

- [x] **Step 1: Add a source-contract test before the refactor**

Add a test that locks the public wrapper and private policy names into the source:

```rust
#[test]
fn legacy_result_list_remains_an_adapter_with_reset_first_policy() {
    let source = include_str!("component.rs");
    assert!(source.contains("pub fn ResultList("));
    assert!(source.contains("ResultReplacementPolicy::ResetFirst"));
    assert!(source.contains("Callback<ResultRow>"));
    assert!(source.contains("Callback<Option<usize>>"));
}
```

- [x] **Step 2: Run the test and verify it fails on the missing policy**

Run the Task 1 focused command. Expected: the new source-contract assertion fails because the wrapper does not yet use `ResetFirst`.

- [x] **Step 3: Create the generic core state and replacement effect**

In `core.rs`, define:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResultReplacementPolicy {
    ResetFirst,
    PreserveKey,
}

#[component]
pub(super) fn ResultListCore<T>(
    items: Signal<Vec<ResultListItem<T>>>,
    empty_message: Signal<String>,
    replacement_policy: ResultReplacementPolicy,
    on_select: Option<Callback<ResultListItem<T>>>,
    on_selection_change: Option<Callback<Option<String>>>,
    class: &'static str,
    node_ref: NodeRef<Div>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
```

Move the instance sequence, listbox markup, hover handling, scroll-into-view, and Arrow/Home/End/Enter handling from `component.rs` into this core. Store `selected` and `hover` as `Option<String>`. The item-replacement effect must:

```rust
let validation = validate_result_list_items(&latest);
let next = if validation.is_err() {
    None
} else {
    match replacement_policy {
        ResultReplacementPolicy::ResetFirst => latest.first().map(|item| item.key.clone()),
        ResultReplacementPolicy::PreserveKey => {
            reconcile_result_key(selected.get_untracked().as_deref(), &latest)
        }
    }
};
let changed = selected.get_untracked() != next;
selected.set(next.clone());
hover.set(None);
if changed || replacement_policy == ResultReplacementPolicy::ResetFirst {
    if let Some(callback) = on_selection_change {
        callback.run(next);
    }
}
```

Enter and click must call `current_result_item(&items.get_untracked(), &key)` immediately before invoking `on_select`. The `<For>` iterates current keys, keys each child by the stable key, and each child reads title/secondary/payload through a live signal that looks up that key in the latest `items`. Render `data-result-key=key` on every option. Invalid keys render a single `role="alert"` row with `data-result-list-key-error`, no `role="option"` children, and no active descendant.

- [x] **Step 4: Convert existing ResultList into a compatibility adapter**

In `component.rs`, map each current display row into a core item whose payload carries its legacy index and row:

```rust
let core_items = Signal::derive(move || {
    items.get().into_iter().enumerate().map(|(index, row)| {
        let (_, hash) = result_row_key(index, &row);
        ResultListItem::new(
            format!("legacy-{index}-{hash:016x}"),
            row.clone(),
            (index, row),
        )
    }).collect()
});
```

Adapt core activation to `Callback<ResultRow>` by returning `item.payload.1`. Adapt core key selection back to the latest legacy index by looking up that key in `core_items`. Invoke the core with `ResultReplacementPolicy::ResetFirst`. Preserve the existing public props and rustdoc example verbatim except for implementation-only imports.

Declare `mod core;` in `mod.rs`; keep the core private.

- [x] **Step 5: Run legacy and keyed pure tests**

Run the focused ResultList command. Expected: all existing legacy navigation/key tests plus the new source contract pass.

- [x] **Step 6: Commit the shared core refactor**

```powershell
git add -- src/components/result_list/core.rs src/components/result_list/component.rs src/components/result_list/mod.rs src/components/result_list/tests.rs
git commit -m "refactor(result-list): share keyed listbox core (ldui-r1z)"
```

### Task 3: Public KeyedResultList and showcase replacement fixture

**Files:**
- Create: `src/components/result_list/keyed_component.rs`
- Modify: `src/components/result_list/mod.rs`
- Modify: `demo/src/demos/result_list.rs`

**Interfaces:**
- Consumes: `ResultListCore<T>`, `ResultReplacementPolicy::PreserveKey`, and `ResultListItem<T>`.
- Produces: public `KeyedResultList<T>` and a deterministic browser fixture at `/result-list`.

- [ ] **Step 1: Add the public generic wrapper**

Implement this wrapper in `keyed_component.rs`:

```rust
#[component]
pub fn KeyedResultList<T>(
    #[prop(optional, into)] items: Signal<Vec<ResultListItem<T>>>,
    #[prop(optional, into, default = "No results found.".to_string().into())]
    empty_message: Signal<String>,
    #[prop(optional)] on_select: Option<Callback<ResultListItem<T>>>,
    #[prop(optional)] on_selection_change: Option<Callback<Option<String>>>,
    #[prop(optional, into)] class: &'static str,
    #[prop(optional)] node_ref: NodeRef<Div>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    view! {
        <ResultListCore
            items=items
            empty_message=empty_message
            replacement_policy=ResultReplacementPolicy::PreserveKey
            on_select=on_select
            on_selection_change=on_selection_change
            class=class
            node_ref=node_ref
        />
    }
}
```

If Leptos optional props reject `Option<Callback<_>>` forwarding, make the core callback props required `Option` values (without `#[prop(optional)]`) while keeping the public wrapper props optional. Export `keyed_component::*` from `mod.rs`.

- [ ] **Step 2: Build the duplicate-looking showcase fixture**

In `demo/src/demos/result_list.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CasePayload {
    case_number: String,
    generation: u8,
}

fn case_result(key: &str, display: &str, case_number: &str, generation: u8)
    -> ResultListItem<CasePayload>
{
    ResultListItem::new(
        key,
        ResultRow {
            title: display.to_owned(),
            subtitle: "Open matter".to_owned(),
            snippet: String::new(),
        },
        CasePayload {
            case_number: case_number.to_owned(),
            generation,
        },
    )
}
```

Initialize `case-a/A-100/v1`, `case-b/B-200/v1`, and `case-c/C-300/v1`, with the first two sharing the visible title `Alex Morgan`. Render `KeyedResultList` as `id="keyed-result-list"`; expose selected key and activation as `data-testid="keyed-result-selected"` and `data-testid="keyed-result-activated"` using `key|case_number|generation` text.

Add deterministic buttons with these exact test ids and operations:

- `keyed-result-reorder`: reverse the current vector.
- `keyed-result-relabel`: change only `case-b.row.title` to `Alejandro Morgan`.
- `keyed-result-replace-payload`: change only `case-b.payload.generation` to `2`.
- `keyed-result-remove-selected`: remove the key currently reported by the selection callback.
- `keyed-result-duplicate`: append another item with key `case-a`.
- `keyed-result-restore`: restore the original three valid items.

- [ ] **Step 3: Compile the public wrapper and showcase**

Run:

```powershell
rustfmt --edition 2024 src/components/result_list/keyed_component.rs src/components/result_list/mod.rs demo/src/demos/result_list.rs
$env:CARGO_BUILD_JOBS='2'; cargo test --lib components::result_list --no-default-features
$env:CARGO_BUILD_JOBS='2'; cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown
```

Expected: native tests and the WASM showcase compile pass.

- [ ] **Step 4: Commit the public path and fixture**

```powershell
git add -- src/components/result_list/keyed_component.rs src/components/result_list/mod.rs demo/src/demos/result_list.rs
git commit -m "feat(result-list): expose keyed typed payloads (ldui-r1z)"
```

### Task 4: Focused browser contract and consumer documentation

**Files:**
- Create: `tests/result_list_smoke.rs`
- Create: `doc/components/result_list.md`
- Modify: `src/components/result_list/mod.rs`

**Interfaces:**
- Consumes: the Task 3 fixture markers and public keyed API.
- Produces: one selectively run browser proof and complete consumer guidance.

- [ ] **Step 1: Add the focused ignored browser journey**

Create `tests/result_list_smoke.rs` using `mod common`, `harness_at("/result-list")`, `begin_browser_error_capture`, and a local `eval_json` helper. One test named `keyed_result_list_preserves_identity_and_activates_current_payload` must perform this sequence:

```rust
let initial = eval_json(&harness, r#"(() => {
    const list = document.querySelector('#keyed-result-list');
    const options = Array.from(list.querySelectorAll('[role="option"]'));
    return {
        keys: options.map(option => option.dataset.resultKey),
        titles: options.map(option => option.querySelector('span').textContent.trim()),
        active: list.getAttribute('aria-activedescendant'),
        selected: options.find(option => option.getAttribute('aria-selected') === 'true')?.dataset.resultKey,
    };
})()"#).await;
assert_eq!(initial["keys"], json!(["case-a", "case-b", "case-c"]));
assert_eq!(initial["titles"], json!(["Alex Morgan", "Alex Morgan", "Taylor Rivera"]));
assert_eq!(initial["selected"], json!("case-a"));
```

Then click the `case-b` option and assert `case-b|B-200|1`. Store a DOM probe on that option, run reorder and relabel, and assert the same key is selected, its title updates, the node retains the probe, and `aria-activedescendant` still equals its id. Replace the payload, dispatch Enter on the focused listbox, and assert `case-b|B-200|2`. Remove the selected key and assert fallback to the first current key. Trigger the duplicate fixture and assert one `role=alert`, zero options, and no active descendant; restore and assert three options. Finish with `assert_no_browser_errors`.

- [ ] **Step 2: Compile the dedicated browser target**

Run:

```powershell
$env:CARGO_BUILD_JOBS='2'; cargo test --test result_list_smoke --no-run
```

Expected: one ignored browser test compiles. Do not run it before the repository is committed and pushed.

- [ ] **Step 3: Document adoption and ownership boundaries**

Create `doc/components/result_list.md` with:

- A choice table: `ResultList` for display-row compatibility, `KeyedResultList<T>` for business activation.
- A complete `ResultListItem::new("case-123", display, CasePayload { ... })` example.
- The `Clone + Send + Sync + 'static` payload bound.
- Key preservation and removal fallback semantics.
- Empty/duplicate-key fail-closed behavior and runtime markers.
- Keyboard/listbox behavior and reactive localization.
- Explicit caller ownership of fetching, debounce, stale-response suppression, authorization, routing, and effects.
- The existing daisyUI/Tailwind source-scan instructions from the component rustdoc.

Update `src/components/result_list/mod.rs` module docs to link both public variants and the new guide path.

- [ ] **Step 4: Perform the browser-test negative control**

After the final code has been committed and pushed and the dedicated demo server is running, use `apply_patch` to temporarily change the core Enter/click path from latest-key lookup to a row captured when the option view was created. Run only:

```powershell
cargo test --test result_list_smoke keyed_result_list_preserves_identity_and_activates_current_payload -- --ignored --nocapture
```

Expected: failure at the `case-b|B-200|2` assertion. Revert that one temporary mutation with `apply_patch`, rerun the same command, and expect a pass. Do not use `git checkout --` in the dirty worktree.

- [ ] **Step 5: Commit browser coverage and docs**

```powershell
git add -- tests/result_list_smoke.rs doc/components/result_list.md src/components/result_list/mod.rs
git commit -m "test(result-list): prove keyed replacement safety (ldui-r1z)"
```

### Task 5: Focused verification, Bead closure, and queue refresh

**Files:**
- Modify: `.beads/issues.jsonl` through `bd close`

**Interfaces:**
- Consumes: all Task 1-4 deliverables.
- Produces: a closed `ldui-r1z` and a fresh ready-work snapshot.

- [ ] **Step 1: Run final focused non-browser checks**

```powershell
git diff --check
$env:CARGO_BUILD_JOBS='2'; cargo test --lib components::result_list --no-default-features
$env:CARGO_BUILD_JOBS='2'; cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown
$env:CARGO_BUILD_JOBS='2'; cargo test --test result_list_smoke --no-run
```

Expected: every command exits zero. The real-browser execution remains the selected post-push check described in Task 4.

- [ ] **Step 2: Close the Bead with exact evidence**

```powershell
bd close ldui-r1z --reason "Added source-compatible KeyedResultList<T> with stable keys, current typed payload activation, identity-preserving replacement, fail-closed key validation, showcase/browser fixture, and consumer docs. Focused native tests, showcase WASM, and result_list_smoke compile pass." --json
```

- [ ] **Step 3: Refresh every queue view immediately**

```powershell
bd ready --json
bd list --status open --json
bd list --status in_progress --json
bd list --status blocked --json
```

Expected: `ldui-r1z` is closed, its dependent `ldui-i95p` becomes ready if no other dependency remains, and no issue is silently left in progress.
