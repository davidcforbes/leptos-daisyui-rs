# PersonPicker

An opinionated person roster for the right edge of a page: search, a collapse
rail, a single-select list, a selected badge and a footer. Design and
rationale: [`doc/plans/2026-09-21-person-picker-design.md`](../plans/2026-09-21-person-picker-design.md).

```rust
let selected = RwSignal::new(None::<String>);
let collapsed = RwSignal::new(false);

view! {
    <PersonPicker
        roster=roster                       // Signal<Vec<PickerPerson>>
        selected=selected
        on_select=Callback::new(move |next| selected.set(next))
        collapsed=collapsed
        on_toggle_collapsed=Callback::new(move |()| collapsed.update(|c| *c = !*c))
        title=Signal::stored("Client Coordinators".to_owned())
        texts=texts                         // Signal<PersonPickerTexts>; reactive
    />
}
```

## One person

```rust
PickerPerson {
    id: "w-1042".to_owned(),                          // stable key
    name: "Ana Lopez".to_owned(),
    secondary: "Denver".to_owned(),                   // office, role
    initials: "AL".to_owned(),                        // supplied, not derived
    presence: Some(PersonPresence::Available),
    activity: Some("Replying to #4127".to_owned()),   // what they are DOING
}
```

- **`presence`** is *whether they are available*: `Available`, `Busy`, `Away`,
  `Offline`, or `Unknown`. It renders as a dot **and** a word, because a dot
  alone is colour-only information.
- **`Unknown` is not `Offline`.** `Unknown` means there is no presence record at
  all — the person has not signed in today. `Offline` means a session that
  ended. `Unknown` renders **no dot** and reads "Not signed in", so a leader is
  never told that someone who simply hasn't opened the app yet has gone
  offline. Map "no heartbeat row" to `Unknown`, never to `Offline`.
- **`activity`** is *what they are doing*, a separate optional caption. Never
  encode an activity ("Replying", "Viewing") as a presence state.
- There is **no way to supply custom card markup**, deliberately. A per-page
  card override is what produced the extra Select button this pattern replaced.

## Contract

- **The card is the control.** Click selects; clicking the selected person
  again deselects; clicking another moves the selection.
- **Keyboard:** the roster is one tab stop; Up/Down/Home/End move focus
  **without** selecting — selection drives assignment, so reading the list must
  not reassign; Space/Enter toggle.
- **Search** filters by name or secondary line and never clears the selection.
  Its × clears the search.
- **The selected badge** names the pick only if it is on the roster, and its ×
  clears the selection. The collapsed rail's count follows the same rule.
- **Fixed 360px**; names, secondary lines and activity captions wrap, never
  scroll sideways.
- `selected` is controlled: `on_select` proposes `Some(id)` or `None`; nothing
  changes until the page accepts it.
- `texts` is a `Signal`, so a language that arrives after mount propagates.

## Why a listbox and not a radiogroup

`SelectableSummaryGroup` is a radiogroup because its cards never have "no
selection". Radio semantics forbid deselect-by-click, which this roster
requires. Do not harmonise the two.

## Inside `SnapshotTablePage`

Pass it as `side_panel`. From `lg` the column takes the panel's own 360px (and
shrinks with it when collapsed to the rail); below `lg` it stacks under the
table.

## Migrating 4iiz-Office's `CoordinatorPanel`

1. Map each `CoordinatorRosterMember` to `PickerPerson` (`id = worker_ref`,
   `secondary = office`, `initials` as today).
2. Map presence from `PresenceAvailabilityDto`: `Available → Available`,
   `Away → Away`, `Offline → Offline`, **`Unknown → Unknown`**. Office supplies
   no `Busy` — its shell's Availability control stores nothing.
3. Map the focus dimension (`Viewing / Replying`) to `activity`, never to
   `presence`.
4. Replace the `picked: RwSignal<Option<String>>` write with `selected=picked`
   and `on_select=Callback::new(move |next| picked.set(next))`.
5. Delete `member_view` and every page's custom card — this is where the extra
   Select button came from.
6. Keep role gating (`in_class`) in Office: it is business logic.
7. Remove the `search` prop the panel took; search is internal now.
