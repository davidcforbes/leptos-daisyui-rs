# PersonPicker — design

**Status:** approved 2026-09-21. **Owner:** leptos-daisyui-rs.
**First consumer:** 4iiz-Office `CoordinatorPanel` (`crates/office-perf-satellite-ui`).

## Why

4iiz-Office's `CoordinatorPanel` is a right-hand roster panel — search, a list of
person cards, one pick — that its hub subpages and Conversations all mount. It is
about to be needed on 20+ pages. As shipped it has four defects the owner named:

1. **Horizontal scroll.** The roster is `overflow-y-auto`, which forces
   `overflow-x` to `auto` as well, and names/offices use `truncate`. Separately,
   the panel hard-codes 360px while `SnapshotTablePage::side_panel` (82ee866)
   hard-codes its column to 320px, so the panel overflows its host.
2. **No deselect.** The card's click only ever does `picked.set(Some(..))`.
3. **Selection is barely visible.** It is daisyUI's `btn-active` on a
   `btn-ghost` — a faint tint.
4. **An extra Select button** on some pages, from a page's `member_view`
   override bypassing the shared card.

Twenty pages each fixing those differently is the drift this library exists to
prevent. So this is a library pattern, not an Office patch.

## Decisions

| Question | Decision |
|---|---|
| What is picked | **Always people**, with optional **presence**. The card shape is fixed; pages cannot vary it. |
| Selection semantics | **Listbox** (`role="listbox"` / `role="option"`, `aria-selected`). See below. |
| State ownership | **Controlled.** The page owns `selected`; the component proposes via `on_select`. Nothing changes optimistically. |
| What the component owns | Search box, collapse rail, roster, selected summary, footer slot. |
| Width | **Fixed 360px** expanded — what Office's hub pages use today. |
| Search text | **Internal** view state. Add a controlled prop only when a page needs it. |

### Why a listbox, and why this is not `SelectableSummaryGroup`

`patterns/selectable_summary.rs` deliberately uses `role="radiogroup"`, and that
decision stays correct *for it*: its fourteen check cards never have "no
selection". Radio semantics **forbid** deselect-by-click — clicking the checked
radio does nothing and there is no route back to none — and this roster's owner
requires a second click to deselect.

`aria-pressed` toggle buttons would allow deselect, but every card becomes its
own tab stop and a screen reader is never told only one can be picked. A
single-select **listbox** allows an empty selection, costs one tab stop, and
announces "selected". Do not "harmonise" the two patterns: they encode different
contracts on purpose.

## Units

`src/patterns/person_picker/`:

| File | Owns |
|---|---|
| `model.rs` | `PickerPerson`, `PersonPresence`, `next_selection`, `matches_search`, `nameable_selection` |
| `listbox.rs` | `PersonListbox` — listbox semantics, keyboard, roving tab stop. Nothing visual beyond the card. |
| `panel.rs` | `PersonPicker` — search, collapse rail, roster, summary, footer |
| `texts.rs` | `PersonPickerTexts`, English default and `::es()` |
| `tests.rs` | native tests |

The listbox is its own unit because its keyboard contract is the part most likely
to be subtly wrong, and isolating it lets it be tested without the panel.

### Data

```rust
pub struct PickerPerson {
    pub id: String,                       // stable key (Office: worker_ref)
    pub name: String,
    pub secondary: String,                // office, role
    pub initials: String,                 // passed in; not derived
    pub presence: Option<PersonPresence>,
    pub activity: Option<String>,         // what they are DOING (added 2026-09-21)
}

pub enum PersonPresence { Available, Busy, Away, Offline, Unknown }
```

`Unknown` was added after Office verified the assumption below against
its wire enum: it means *no presence record at all* (has not signed in
today), which is not `Offline` (a session that ended). It renders **no
dot** and the word "Not signed in", so it can never read as offline.

Initials are supplied, not derived: correct initials across every culture's names
is hard, and the consumer already has them.

### Panel API

```rust
#[component]
pub fn PersonPicker(
    roster: Signal<Vec<PickerPerson>>,
    selected: Signal<Option<String>>,
    on_select: Callback<Option<String>>,      // Some(id) = select, None = clear
    collapsed: Signal<bool>,                  // page-owned so it can be remembered
    on_toggle_collapsed: Callback<()>,
    title: Signal<String>,
    #[prop(optional)] texts: Signal<PersonPickerTexts>,  // reactive: no language freeze
    #[prop(optional)] footer: Option<Children>,
) -> impl IntoView
```

`texts` is a `Signal` so a language that arrives after mount propagates — the
defect Office shipped with `filter_text` (op-e6dsi).

### Pure rules (native-tested)

- `next_selection(current, clicked)` — `None` if `clicked` is `current`, else
  `Some(clicked)`.
- `matches_search(person, query)` — case-insensitive substring on name and
  secondary; empty query matches all.
- `nameable_selection(roster, selected)` — the selected person **only if** their
  id is in the roster; a stale id yields `None`. (Office op-4c18j: a stale
  `worker_ref` otherwise leaves the panel naming a coordinator nothing on screen
  identifies.)

## Behaviour

**Mouse.** Click selects; click on the selected person proposes `None`; click on
another person proposes them. Every click also moves the roving tab stop to that
card.

**Keyboard** (listbox):
- The roster is **one tab stop**. Tab lands on the selected person if visible,
  else the first person.
- ↑/↓ move focus; Home/End jump to the ends.
- **Space/Enter toggle** the focused person.
- **Arrowing never changes the selection.** Selection has consequences — the
  toolbar's Assign acts on it — so reading the list must not reassign.

**Invariant — exactly one reachable card.** While the visible roster is non-empty,
exactly one option has `tabindex="0"`. Enforced on first render, after search
filters out the focused card, and after the roster changes. This repo shipped the
opposite twice (`Heatmap`, `BarChart`: the reconcile effect returned early on its
first run, so no element carried `tabindex="0"`, invisible to 3000+ native tests).

**Search** filters on `matches_search` and **never clears the selection**. A
non-empty search box shows a **×** that clears the search.

**Selected summary** — a badge naming `nameable_selection`, with a **×** that
proposes `None`. Absent when nothing nameable is selected. Deliberately **not** a
live region: the option's own `aria-selected` change is what a screen reader
announces, and a live summary would speak every change twice.

**Collapse rail** — the collapsed state shows a count of 1 only for a *nameable*
selection, per the same rule.

**Empty states** — an empty roster and "no matches" render a short message, and
no `role="listbox"` with zero options.

## Visual

- **360px fixed** expanded (`w-[360px] min-w-[360px] max-w-[360px]`).
- **Wrapping, not truncation.** Name and secondary use `whitespace-normal
  break-words`, plus `overflow-wrap:anywhere` so an unbroken string (an email)
  breaks instead of overflowing.
- **No horizontal scroll.** The roster is `overflow-y-auto overflow-x-hidden`
  explicitly, and every flex child that holds text carries `min-w-0`.
- **Selected:** `bg-primary text-primary-content` plus a check icon.
  **Unselected:** `bg-base-100` with a `border-base-300` border and a
  `hover:bg-base-200` tint.
- **Forced colors** (Windows high contrast strips backgrounds): the selected card
  also gets a `forced-colors:` border, and the check icon carries the state.
- **Presence:** a dot on the avatar **and** the status word in the secondary line
  ("Denver · Busy"). A dot alone is colour-only information (WCAG 1.4.1).
- **Muted text uses `/75` or darker.** `/60` fails AA at 14px on this theme
  (3.37:1; 3c35912). Contrast is measured by axe in both states, never assumed.
- Spacing on the canonical scale; the avatar uses `IconTileSize`, not literal
  `w-*`/`h-*`.

## Integration

- **`SnapshotTablePage::side_panel` drops its fixed `w-80`.** A slot should not
  dictate the width of what goes in it; the panel owns its 360px. Mobile keeps
  `w-full`.
- **Office migration** (Office's repo, by Office): `CoordinatorPanel` becomes a
  thin wrapper mapping `CoordinatorRosterMember` → `PickerPerson`, the
  `member_view` override is removed, and role-based gating (`in_class`) stays in
  Office as business logic. This repo ships migration notes; it does not edit
  Office.
- **Presence, verified 2026-09-21 against Office's `PresenceAvailabilityDto`**
  (`Available / Away / Offline / Unknown`): `Unknown` was missing and has
  been added (above). `Busy` stays -- Office's UI offers it as a user choice
  and is confirming what it stores. Office's *focus* dimension (`Viewing /
  Replying`) is what someone is DOING, not whether they are available, and
  must never become an availability state. **Decided 2026-09-21:** it is a
  separate optional `activity` caption -- plain text, never markup, so the
  card's shape stays fixed -- shown under the secondary line.
- **`Busy` is not supplied by Office:** its shell's Availability control
  stores nothing (a local signal no request or column reads). Office's
  complete set is `Available / Away / Offline / Unknown`. `Busy` remains in
  the enum for other consumers.

## Testing

**Native** (`tests.rs`): `next_selection` all four cases; `matches_search`
including case, secondary-line match and empty query; `nameable_selection` with a
stale id; English and Spanish texts differ.

**Browser** — a new fixture page and a new suite **registered in its own xtask
lane** (a suite registered in no lane runs nowhere; verify by the test count
moving). At 1440 wide:

1. One tab stop; Tab lands on the selected person.
2. ↑/↓ move focus **without** changing `aria-selected`.
3. Space toggles; click toggles; clicking another person moves the selection.
4. Exactly one `tabindex="0"` option after a search filters out the focused card.
5. Search × clears the search; badge × clears the selection.
6. **No horizontal overflow** with a deliberately long name and an unbroken
   email: `scrollWidth <= clientWidth` on the roster and on each card.
7. Expanded width is 360px.
8. axe `color-contrast` passes at rest **and** with a selection.
9. No browser errors — the suite's own capture; FilterBar's ResizeObserver loop
   was caught exactly this way (82ee866).

A showcase demo page is added for the pattern.

## Out of scope

Multi-select; type-ahead in the listbox (the search box covers it); a
page-controlled search prop; custom card layouts (no `member_view` equivalent —
that override is the source of the drift this replaces).
