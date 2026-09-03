# EntityTable inline draft-row editing — design (ldui-ff2f)

> **Implementation status, 2026-09-03.** The **draft-row foundation is built
> and on main** (`a7ade64`): the exclusive reducer (§2), per-column editors
> (§3.2), the commit/resolve payload (§3.3), the toolbar `+`, initial draft-row
> rendering, and row-level inert metadata (§4). Consumer documentation lives
> in
> [`doc/components/entity_table.md`](../components/entity_table.md#inline-draft-row-editing-ldui-ff2f).
>
> **Approved completion, not yet built:** one existing action column becomes
> the framework's inline-edit host (§3.4/§4b), existing rows use the same
> editable-cell renderer as new drafts, every descendant of a non-live row is
> genuinely inert, table controls lock during a session, and compact rows get
> the same complete interaction. The accepted table snapshot also freezes for
> the whole session and coalesces incoming refreshes until editing finishes
> (§6). The browser proof is expanded accordingly; the earlier draft-only proof
> did not cover all of those claims.

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
        +-- '+' / row Edit pressed -------------------------+
        |                                                   v
   [ Idle ] <--- Escape / commit resolved --- [ Drafting ] --+
        ^                                          |
        |                                          | Save pressed
        +------- consumer resolves outcome --- [ Committing ]
```

- **Idle** — today's table, byte-identical. No opt-in config ⇒ this is the
  only reachable state.
- **Drafting** — one new or existing working row is live; all other rows are
  inert; its first editable cell has focus.
- **Committing** — Save has fired the consumer's intent; the working row is
  frozen (not removed) until the consumer resolves it. This state prevents
  the classic double-submit and the "row vanished before the server answered"
  bug.

`Drafting` and `Committing` both hold the table's accepted backing snapshot
fixed. Returning to `Idle` first publishes the newest pending refresh, if one
arrived, then re-enables the table. A rejection returns to `Drafting`, so it
does not release the pending refresh or replace the user's input.

`Committing` is an approved addition to the owner's interaction. It exists because the consumer
owns persistence and persistence is asynchronous: without it, Save must either
optimistically drop the draft (losing the user's input if the write fails) or
block the UI.

## 3. API

### 3.1 Opt-in, absent by default

```rust
pub struct EntityDraftRow<T> {
    /// Builds the blank row the table inserts. The consumer owns the type,
    /// so the framework never invents a `T`.
    new_row: Rc<dyn Fn() -> T>,
    /// Fired on Save with the edited draft. The consumer persists; the
    /// component never writes.
    on_commit: Callback<EntityDraftCommit<T>>,
    /// Reactive copy for framework-owned add/edit/save/cancel controls.
    texts: Signal<EntityDraftTexts>,
    /// Adds Edit to existing rows; false retains create-only behavior.
    allow_row_edit: bool,
}
```

Wired as one optional prop, so every existing table is untouched:

```rust
<EntityTable
    // ...existing props unchanged...
    draft_row=EntityDraftRow::new(|| WorkType::blank(), on_commit)
        .allow_row_edit(true)
/>
```

**Absent ⇒ no toolbar `+`, no edit mode, no new DOM.** Same discipline as
`filter_actions` on `SnapshotTablePage` (ldui-nj3q).

### 3.2 Per-column editability

Most columns are derived and must stay read-only, so editability is declared
per column:

```rust
EntityColumn::text("name", "Name", |r: &WorkType| r.name.clone())
    .editable(EntityCellEditor::text(
        |row: &WorkType| row.name.clone(),
        |row: &mut WorkType, v| row.name = v,
    ))
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
today. The same check covers the action-host marker in §3.4.

### 3.3 The commit payload

```rust
pub struct EntityDraftCommit<T> {
    /// The edited row. Consumer validates and persists.
    pub row: T,
    /// New draft, or existing row with its stable key.
    pub target: EntityEditTarget,
    /// Resolve the outcome. Until called, the table stays in Committing.
    pub resolve: Callback<EntityEditOutcome>,
}

pub enum EntityEditOutcome {
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

### 3.4 One existing action column hosts the framework controls

The action column remains an `EntityColumn<T>` and retains its existing rich
consumer renderer. The consumer marks exactly one action column as the inline
edit host:

```rust
EntityColumn::action("actions", "Actions", |row: &WorkType| {
    row.available_actions_label()
})
.render_with(render_work_type_actions)
.inline_edit_host()
```

`inline_edit_host()` also makes the column required. Hiding the only Save
control through a stored column preference would otherwise create an
unfinishable edit session. Supplying `draft_row` without exactly one marked
action column is a construction error with the offending configuration named;
the table never guesses which of several action columns owns Edit/Save.

This is deliberately a marker on the existing column rather than a second
row-action API. Existing action renderers can depend on the complete typed row,
render icons/tooltips, and carry domain-specific disabled states. Re-expressing
them as label-plus-key declarations would lose that behavior and would still
need to solve header, colgroup, filtering, compact-row, visibility, and export
semantics for a parallel synthetic column.

## 4. Behaviour the spec pins down

| Spec item | Implementation |
|---|---|
| `+` inserts a blank row | Rendered in the existing `toolbar_actions` region, beside Export. Framework-owned button, so every table's `+` looks and reads the same. |
| Cursor jumps to first column | Focus moves to the **first editable** cell's control after mount. Not simply "first column" — a leading derived/action column has no editor to focus. |
| All other rows disabled | `aria-disabled="true"`, the native `inert` attribute, and removal of the row itself from the tab order — see the divergence note below. `inert` is what also disables nested consumer-rendered buttons and links. |
| Tab / click to next field | Native tab order within the draft row. Tab past the last field lands on the row's Save button, so the whole flow is reachable without a pointer. |
| Save invokes the action | Fires `on_commit`; enters Committing. |
| Escape exits, no blank row | Returns to Idle, drops the draft, restores focus to the `+` that opened it. No phantom row, and focus never lands on `<body>`. |
| Refresh while editing | Captures the newest incoming snapshot as pending without changing the frozen table. The pending snapshot becomes visible only after Save is accepted or the edit is discarded. |

### Where Save lives — settled

Owner clarification, 2026-09-03:

> The row has a column of action buttons (Retire, Edit/Save), so those must be
> selectable.

So **Save is the row's own action button, not a toolbar button.** The marked
action column already exists (`EntityColumn::action().inline_edit_host()`), and
the bead's "while a row is being edited its action button reads 'Save' instead
of 'Edit'" describes exactly this: one framework control in that cell,
relabeled by state.

That settles a question §3 had left implicit and it simplifies the toolbar —
`+` is the *only* framework-owned toolbar addition. The active row stays
interactive; its consumer actions are replaced by Save/Cancel for the duration
of the session. Every other row is inert as a unit, including arbitrary nested
controls in its consumer-rendered action cell.

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

## 4b. Editing an existing row — answered

Owner, 2026-09-03:

> If the user presses the Edit button on any particular row, **the table goes
> disabled**, the selected row becomes editable and the Edit button becomes
> the Save button to commit the updates and invoke the save function, and
> return the table to readonly mode.

So the two paths are **one mode with two entry points**, exactly as hoped:

| | Entry | Editable row | Commit control | Exit |
|---|---|---|---|---|
| Create | toolbar `+` | a new blank row | that row's action button | Save ⇒ Idle, Escape ⇒ Idle, row dropped |
| Update | a row's Edit button | that existing row | the same button, relabelled | Save ⇒ Idle, Escape ⇒ Idle, edits discarded |

The state machine in §2 is unchanged — `Drafting` simply gains a second way in
and carries which row is live. Nothing else in this design moves, which is a
good sign the exclusive-mode constraint was the right backbone.

The existing row keeps its keyed `<tr>` in the frozen accepted snapshot. While
it is live, every editable data column renders from the reducer's cloned
working row rather than from `data`; derived columns remain read-only. Its
marked action cell renders Save/Cancel. On Escape, the working clone is
discarded and the pending snapshot is published; focus returns to that row's
Edit control if its key still exists, otherwise to the named table region. On
acceptance, the row returns through the consumer's normal data flow; on
rejection, the working clone and focusable editors remain in place.

The same structure is rendered at compact widths. A live compact row uses one
cell spanning the declared column count, with visible labels paired to its
editors and Save/Cancel after the final field. An idle compact row retains the
consumer's compact rendering and exposes the framework Edit control. The
desktop-only `hidden lg:table-cell` path is therefore never the sole way to
finish or cancel a session.

**"The table goes disabled" resolves two further questions by implication,**
and I am recording the inference explicitly rather than pretending it was
stated:

- **Sort / filter / paging are locked** while a row is live (was question 2).
  They are part of the table, and the alternative re-sorts or pages away the
  row the user is typing into.
- **Other rows' action buttons go inert** (was question 5). This is not merely
  consistent — it is *required*: if a second row's Edit button stayed live,
  pressing it would open a second editable row and break the single-live-row
  invariant the whole design rests on. Retire on another row goes inert with
  it, which the plain reading of "the table goes disabled" also supports.

The same lock covers table-owned controls that can move or reshape the live
row: sorting, column filters, paging/page size, column visibility/reorder,
selection, and resize handles. Consumer toolbar actions sit inside the same
inert toolbar region. Event handlers retain reducer-side busy guards as a
second line of defense; DOM state is not the only enforcement boundary.

## 5. Decisions complete

| Question | Decision |
|---|---|
| New and existing rows? | One exclusive mode with two entry points (§4b). |
| Async persistence? | Keep `Committing`; only the consumer's resolve handle ends or rejects the write. |
| Which cell owns Edit/Save? | Exactly one existing action column marked `inline_edit_host()` (§3.4). |
| Other rows' actions during a session? | Inert with the rest of the row; native `inert` covers nested controls. |
| Table chrome during a session? | Locked wherever an operation can move, hide, reorder, or mutate the live row. |
| Data refresh during a session? | Queue only the newest coherent snapshot; publish it when the session returns to `Idle`. |
| Column compatibility? | Adding the marker field follows the already-verified `EntityColumn` builder-only usage across the local consumers. |

## 6. What a refresh does mid-edit

The table does not admit a refreshed dataset while an edit is active. The
accepted snapshot is frozen and inert; the single local working-row overlay is
the only active part of the table.

A producer may still complete a request and update the input signals. The
component captures those inputs as a **pending coherent snapshot** — rendered
`data`, authoritative `source_data`, dataset identity, page-reset identity, and
focus scope together — but does not publish it into the displayed table during
`Drafting` or `Committing`. More arrivals replace the pending value, so the
queue is latest-wins rather than a backlog of obsolete intermediate snapshots.

The release rules are explicit:

- **Cancel / Escape:** discard the working row, publish the newest pending
  snapshot atomically, then re-enable the table and restore focus against that
  refreshed result (`+` for a new draft; the row's Edit control when it still
  exists; otherwise the named table region).
- **Accepted commit:** end the working session, publish the newest pending
  snapshot atomically, then re-enable. If the consumer's saved result arrives
  later, it is an ordinary idle refresh.
- **Rejected commit:** return to `Drafting` with the working row and accepted
  backing snapshot unchanged. The pending refresh stays pending because the
  edit is not complete.
- **No pending refresh:** return to the same accepted snapshot and re-enable.

This removes the earlier "pin a row that disappeared underneath the editor"
case entirely: the backing table cannot change underneath the overlay. If the
newest pending snapshot removed the edited row, that removal becomes visible
only after the user has saved or discarded the overlay.

## 7. Verification plan

Following this repo's rule that a suite registered in no lane runs nowhere
(`ldui-a8an`), and that browser coverage which only compiles proves nothing:

- **Native:** reducer transitions plus pure configuration/column-shape helpers:
  exactly one required edit host, draft and existing targets share the commit
  path, a session freezes the accepted input envelope, pending refreshes
  coalesce latest-wins, rejection keeps them pending, and cancel/accept releases
  exactly one coherent snapshot.
- **Browser / Layer B+D1** (`test-entity-draft-row`, registered as its own
  xtask step *and* in `full_steps()`): use real pointer and keyboard input for
  `+`, Edit, typing, Tab, Save, Escape, rejection, and acceptance. Assert DOM
  phase plus the consumer-observed commit target/row, no browser errors, and no
  prohibited callback or table-control activation while locked. Drive two data
  refreshes during a session: neither may change the rendered backing rows, and
  only the second may appear after cancel or accepted resolution.
- **Structure:** header, colgroup, filter, body, empty row, draft row, and group
  spans agree on one column count; there is no synthetic unmatched `<td>`.
  Existing-row editing changes cell content without replacing its keyed `<tr>`.
- **Accessibility / Layer C:** first-editor focus, sequential Tab to Save,
  Escape focus restoration, `inert` descendants absent from tab order, named
  controls, ordered focus in compact mode, and an axe-core run with editing
  open.
- **Responsive / Layer A+B:** force wide and compact widths. Run the existing
  style/layout audits for table hierarchy and zero overlap; inspect the live
  fixture rather than minting a new screenshot baseline from the same change.
- **Negative controls:** the fixture keeps an otherwise-identical table without
  `draft_row`; deliberately break one interaction/model assertion, observe the
  focused lane fail, revert it, and observe green.
- Verify registration by the **step count changing**, never by the step
  passing.

## 8. Chosen approach and implementation order

Three action-ownership approaches were considered:

1. **Chosen — mark one existing action column as the host.** It preserves rich
   consumer renderers and every existing column semantic while giving the
   framework the stateful cell it needs.
2. **Rejected — a second declarative `EntityRowActionSpec` API.** It duplicates
   the existing action-column vocabulary, cannot express current rich actions,
   and creates a parallel column that must be threaded through every geometry
   and projection path.
3. **Rejected — expose the reducer/controller to a consumer renderer.** That
   leaks table-local state, makes exclusivity depend on each consumer, and
   prevents the framework from guaranteeing focus and locked descendants.

The remaining implementation order is:

1. Write failing native tests for action-host validation and the accepted /
   pending snapshot gate, then add the minimal column/config helpers.
2. Expand the browser fixture and write failing wide-layout journeys for
   existing-row Edit⇄Save, real focus/Tab/Escape, locked controls, consumer
   actions, target-aware commits, and column geometry.
3. Unify draft/existing live-row rendering inside the marked action column and
   apply real inertness plus control locks.
4. Add the compact-width journey and compact live-row form, then run the
   style/layout/axe assertions and break-and-revert proof.
5. Update the consumer guide, run the focused lane, then run
   `cargo xtask verify-full` on the final candidate tree.
