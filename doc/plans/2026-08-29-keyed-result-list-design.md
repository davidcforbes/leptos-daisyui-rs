# Keyed ResultList with typed payloads

Status: approved design for `ldui-r1z`

## Context

`ResultList` currently accepts display-only `ResultRow` values, tracks the
highlight by page index, and returns a cloned display row from `on_select`.
That contract cannot safely activate a business entity when two results have
the same visible copy or when an asynchronous search replacement reorders the
rows. Office Account and No-Hires therefore bypass the component to retain a
case number or other typed transport identity.

The existing `ResultList` API must remain source-compatible. This change adds
one keyed, generic path and does not make the legacy component generic.

## Public API

Add a generic item type:

```rust
pub struct ResultListItem<T> {
    pub key: String,
    pub row: ResultRow,
    pub payload: T,
}
```

`ResultListItem::new(key, row, payload)` is the normal constructor. `key` is a
stable business identity, `row` is a localized display projection, and
`payload` is caller-owned activation data. The type derives the traits its
generic payload permits.

Add `KeyedResultList<T>` with the same empty-state, class, and node-reference
props as `ResultList`, plus:

- `items: Signal<Vec<ResultListItem<T>>>`
- `on_select: Option<Callback<ResultListItem<T>>>`
- `on_selection_change: Option<Callback<Option<String>>>`

`T` must be `Clone + Send + Sync + 'static`: event callbacks return a snapshot
of the current item, and the standard Leptos `Signal` uses synchronized
storage. Returning the complete item gives the consumer its stable key and
exact typed payload without reconstructing either from display text or an
index.

The existing `ResultList`, `ResultRow`, `on_select(ResultRow)`, and
`on_selection_change(Option<usize>)` contracts remain available unchanged.

## Identity and replacement behavior

The keyed path stores highlight and hover state by stable key. On replacement:

1. Keep the highlighted key when it still exists, regardless of reorder,
   insertion, label changes, or payload replacement.
2. If it disappeared, highlight the first current item.
3. If the list is empty or invalid, highlight nothing.

Keyboard movement maps the accepted key into the latest item order before
applying the existing clamped Arrow/Home/End rules. Enter and pointer
activation look up the key in the latest `items` signal at event time. A
closure captured from an older result set therefore cannot return an old
payload.

The keyed `<For>` identity is the stable key. Title, subtitle, snippet, and
payload are read through a live lookup for that key so localization or an
asynchronous payload refresh updates the mounted option without transferring
selection to another entity. Option DOM ids use the list instance plus a
collision-free hexadecimal encoding of the key bytes; `aria-activedescendant`
therefore follows identity instead of position.

The legacy wrapper retains its current reset-to-first-on-replacement behavior
and index callback. A private shared renderer/navigation core receives an
explicit replacement policy so adding keyed preservation cannot silently
change legacy behavior.

## Invalid input and fail-closed behavior

Every keyed item must have a non-empty, non-whitespace key, and keys must be
unique within the current list. Invalid input renders one visible `role=alert`
state with a diagnostic data attribute, emits no options, clears highlight and
hover, and never calls `on_select`. The component does not guess which
duplicate is authoritative.

An event whose key disappeared between dispatch and handling is a no-op. The
caller remains responsible for search execution, stale-response suppression,
authorization, navigation, and mutation effects.

## Accessibility and interaction

The listbox keeps DOM focus and exposes the highlighted option through
`aria-activedescendant`. Each option retains `role=option` and
`aria-selected`. ArrowUp, ArrowDown, Home, End, Enter, hover preview, click
activation, variable-height wrapping, scroll-into-view, empty copy, and
runtime localization remain framework-owned.

Selection-change callbacks fire only when the effective key changes. A pure
relabel or payload refresh for the selected key does not create a false
selection transition.

## Testing

Implementation follows test-driven development:

- Pure native tests cover validation, collision-free DOM ids, key
  reconciliation, clamped navigation after reorder, removal fallback, and
  current-payload lookup.
- Existing legacy tests remain unchanged and prove source behavior.
- The showcase gains one keyed fixture with duplicate-looking rows and
  controls for reorder, relabel, insertion, removal, and payload replacement.
- A focused ignored `result_list_smoke` browser journey verifies pointer and
  keyboard activation, `aria-activedescendant`, retained key identity, current
  payload delivery, removal fallback, and invalid duplicate keys. It is
  separate from the 32-test reactivity inventory and is run selectively.
- The browser assertion receives an inject/catch/revert negative control before
  completion to prove it detects index-based or stale-payload behavior.

## Non-goals

- No async fetching, debounce, routing, authorization, or dialog composition.
- No multi-selection or externally controlled selection in this change.
- No encoding of business identity into visible title, subtitle, or snippet.
- No removal of the legacy `ResultList` convenience API.
