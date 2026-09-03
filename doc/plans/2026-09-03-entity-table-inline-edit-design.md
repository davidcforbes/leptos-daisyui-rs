# EntityTable inline draft-row editing — design proposal (ldui-ff2f)

**Status:** proposal, awaiting framework-owner review. Nothing implemented.
**Filed from:** 4iiz-Office Setup page (Work Types), owner request 2026-09-03.
**Related:** `ldui-tmoz` (hiding the rows-per-page control on the same page).

---

## 1. The interaction, as specified

The owner's spec, verbatim in shape, is the authority for this design:

> The EntityTable inserts a blank row in the table and the cursor jumps to the
> first column of the blank row (**all other table rows are disabled**). The
> user can enter values and then press Tab or click into the next field in the
> blank row. When finished the Save button can be pressed to invoke that
> action/event. The Escape key exits edit mode and returns the table to its
> normal state without the blank row.

One consequence dominates every decision below, and it is worth stating
plainly because it makes the feature *much* smaller than the bead implied:

> **Edit mode is exclusive and table-wide, not per-row and concurrent.**

Disabling every other row means there is never more than one draft in flight,
so there is no set of open editors to reconcile, no per-row editing state to
key by row identity, and no question about what a refresh does to *other*
rows' editors. The table has one extra state, not N.

## 2. State model

```
        +-- '+' pressed ------------------------------------+
        |                                                   v
   [ Idle ] <--- Escape / commit resolved --- [ Drafting ] --+
        ^                                          |
        |                                          | Save pressed
        +------- consumer resolves outcome --- [ Committing ]
```

- **Idle** — today's table, byte-identical. No opt-in config ⇒ this is the
  only reachable state.
- **Drafting** — one blank row is present; all other rows are inert; the
  draft's first editable cell has focus.
- **Committing** — Save has fired the consumer's intent; the draft is frozen
  (not removed) until the consumer resolves it. This state is what prevents
  the classic double-submit and the "row vanished before the server answered"
  bug.

`Committing` is my addition, not in the spec. It exists because the consumer
owns persistence and persistence is asynchronous: without it, Save must either
optimistically drop the draft (losing the user's input if the write fails) or
block the UI. I recommend keeping it and would like that confirmed.

## 3. Proposed API

### 3.1 Opt-in, absent by default

```rust
pub struct EntityDraftRow<T> {
    /// Builds the blank row the table inserts. The consumer owns the type,
    /// so the framework never invents a `T`.
    new_row: Rc<dyn Fn() -> T>,
    /// Fired on Save with the edited draft. The consumer persists; the
    /// component never writes.
    on_commit: Callback<EntityDraftCommit<T>>,
    /// Optional: fired on Escape, for a consumer that wants to know.
    on_cancel: Option<Callback<()>>,
}
```

Wired as one optional prop, so every existing table is untouched:

```rust
<EntityTable
    // ...existing props unchanged...
    draft_row=EntityDraftRow::new(|| WorkType::blank(), on_commit)
/>
```

**Absent ⇒ no toolbar `+`, no edit mode, no new DOM.** Same discipline as
`filter_actions` on `SnapshotTablePage` (ldui-nj3q).

### 3.2 Per-column editability

Most columns are derived and must stay read-only, so editability is declared
per column:

```rust
EntityColumn::text("name", "Name", |r: &WorkType| r.name.clone())
    .editable(EntityCellEditor::Text {
        set: Rc::new(|row: &mut WorkType, v: String| row.name = v),
    })
```

Columns without `.editable(...)` render their normal read-only cell **even in
the draft row**. That is deliberate: a derived column (a computed total, a
badge) has nothing meaningful to show for a blank row, and inventing an
editor for it would be wrong.

**Compatibility — checked, not assumed.** `EntityColumn<T>` has `pub` fields,
so adding one breaks any consumer constructing it with a struct literal. I
grepped the 4iiz-Office surfaces for `EntityColumn {` and found **zero** —
every call site uses the builders (`EntityColumn::text(...).required()`).
Adding a field is therefore source-compatible for the consumers that exist
today. Open question 4 in §5 is now just a confirmation, not a risk.

### 3.3 The commit payload

```rust
pub struct EntityDraftCommit<T> {
    /// The edited row. Consumer validates and persists.
    pub row: T,
    /// Resolve the outcome. Until called, the table stays in Committing.
    pub resolve: Callback<EntityDraftOutcome>,
}

pub enum EntityDraftOutcome {
    /// Persisted. Table returns to Idle and drops the draft; the row
    /// re-enters through the consumer's normal data flow.
    Accepted,
    /// Rejected. Table returns to Drafting with the user's input intact,
    /// and surfaces the message through the existing action-feedback
    /// vocabulary rather than inventing a second error channel.
    Rejected(String),
}
```

The `resolve` handle is what keeps persistence entirely on the consumer's side
while still letting the component own the state machine. It mirrors the
handle-based discipline already in `SnapshotTableState` (a minted handle whose
completion can be ignored if stale).

## 4. Behaviour the spec pins down

| Spec item | Implementation |
|---|---|
| `+` inserts a blank row | Rendered in the existing `toolbar_actions` region, beside Export. Framework-owned button, so every table's `+` looks and reads the same. |
| Cursor jumps to first column | Focus moves to the **first editable** cell's control after mount. Not simply "first column" — a leading derived/action column has no editor to focus. |
| All other rows disabled | `aria-disabled="true"` **plus removal from the tab order** — see the divergence note below. |
| Tab / click to next field | Native tab order within the draft row. Tab past the last field lands on Save, which makes the whole flow reachable without a pointer. |
| Save invokes the action | Fires `on_commit`; enters Committing. |
| Escape exits, no blank row | Returns to Idle, drops the draft, restores focus to the `+` that opened it. No phantom row, and focus never lands on `<body>`. |

**Divergence from the existing `aria-disabled` precedent — deliberate.**
`component.rs:1266` uses `aria-disabled` on the empty-table header checkbox
and **deliberately keeps it in the tab order**, so a keyboard user reaches it
and hears *why* it is inert ("No rows are displayed"). That reasoning is right
for **one** control and wrong for **N rows**: leaving 17 disabled rows tabbable
would make the spec's "Tab to the next field, then Save" flow require tabbing
through the entire table first.

So disabled rows here take `aria-disabled="true"` **and** leave the tab order.
The "tell the user why" obligation the precedent was protecting is met once,
at the region level, rather than N times per row — the table announces that it
is in edit mode, instead of every row announcing that it is not.

I am flagging this rather than quietly diverging: the two rules look identical
and are not, and a future reader comparing them deserves to know which case
they are in.

## 5. Open questions for review

These are the ones where I do not want to guess:

1. **Does the same mode cover editing *existing* rows?** The bead title says
   "and per-row Edit/Save editing", but your interaction spec describes only
   the draft-row path. The exclusive-mode design extends to it symmetrically
   (click Edit ⇒ that row becomes the editable one, all others disabled,
   button reads Save) at modest extra cost. **Recommend: yes, same mode** —
   but confirm, because it roughly doubles the test surface.
2. **Are sort / filter / paging locked during Drafting?** If they are not, a
   sort can move the draft row out from under the user, or a page change can
   scroll it away entirely. **Recommend: lock them** (same `aria-disabled`
   treatment) — the alternative is a genuinely confusing UI.
3. **Confirm the `Committing` state** (§2) rather than optimistic drop.
4. **Confirm the `EntityColumn` field addition** is acceptable (§3.2).

## 6. What a refresh does mid-edit

The bead calls this out and it is the subtlest part. Rule:

> **A data refresh landing during Drafting or Committing never removes the
> draft row and never discards its input.**

The draft is not part of `data`; it is table-local state layered over it. So a
new `data` signal replaces the *backing rows* and the draft survives on top,
because it was never in that collection to begin with. This falls out of the
design rather than needing a special case — which is a good sign the state is
in the right place.

The one real case to handle: a refresh that removes rows can change which page
the draft sits on. Since paging is locked during Drafting (open question 2),
this cannot bite while a draft is open.

## 7. Verification plan

Following this repo's rule that a suite registered in no lane runs nowhere
(`ldui-a8an`), and that browser coverage which only compiles proves nothing:

- **Native:** the state machine as a pure reducer — Idle→Drafting→Committing→
  Idle, Escape from each state, Rejected returning to Drafting with input
  intact, stale resolve ignored.
- **Browser** (`test-entity-draft-row`, registered as its own xtask step *and*
  in `full_steps()`): `+` inserts and focuses the first editable cell; other
  rows carry `aria-disabled` and are out of the tab order; Tab walks the draft
  and reaches Save; Escape removes the row and restores focus to `+`; a
  refresh mid-draft leaves the draft intact.
- **Negative control per assertion**, and a demo fixture mounting both an
  opted-in and a plain table on one document — the pattern used for
  `ldui-nj3q`, so the "renders unchanged when absent" claim is proven on the
  same run rather than asserted.
- Verify registration by the **step count changing**, never by the step
  passing.

## 8. Recommendation

Approve §2–§4 and answer the four questions in §5, and this is a well-bounded
build. The exclusive-mode constraint from your spec is what makes it
tractable: it removes the concurrency that would otherwise dominate the design.
