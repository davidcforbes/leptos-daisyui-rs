# ResultList / KeyedResultList

A flat, ranked, keyboard-navigable results picker (search results, a
type-ahead's match list, a case lookup) ported from d2d-ui's
`controls::result_list::ResultList`. Two public components share one private
listbox core (`src/components/result_list/core.rs`):

| Component | Row identity | Activation payload | Use when |
|---|---|---|---|
| `ResultList` | Positional (index into `items`) | The displayed `ResultRow` itself | Rows have no identity beyond their display text — a plain ranked list where the row *is* the payload. |
| `KeyedResultList<T>` | Caller-assigned stable `key: String` | A typed `T`, independent of the display row | Two rows can display the same text, results arrive asynchronously and may reorder/replace between renders, or the activation value is a typed id/record the display text does not fully determine. |

Both support the same interaction contract: `ArrowUp`/`ArrowDown` move one
row (clamped at the ends, no wraparound), `Home`/`End` jump to the
first/last row, `Enter` activates the highlighted row, hovering previews a
row, clicking both selects and activates it, the highlighted row auto-scrolls
into view (`Element.scrollIntoView({ block: "nearest" })`), and the listbox
follows the WAI-ARIA `listbox`/`option` pattern with `aria-activedescendant`.

## Why keyed identity exists

`ResultList` keys its internal `<For>` list by a content hash of each row
(`result_row_key`), so replacing `items` on every keystroke re-renders any row
whose *content* changed at a given index — but it still has no notion of
identity independent of that content. Two visually distinct rows with the
same title (`"Alex Morgan"` from two different intake dates, say) are
indistinguishable to a consumer whose `on_select: Callback<ResultRow>` only
ever receives the row that's on screen. If the list is a wrapper deriving a
database id or case number from `row.title`, a duplicate title makes that
derivation ambiguous or wrong.

`KeyedResultList<T>` closes that gap by requiring every
[`ResultListItem<T>`](../../src/components/result_list/types.rs) to carry:

- `key: String` — the caller's stable business identity, used for selection,
  keyboard movement, DOM reconciliation (`<For>`'s key), and
  `aria-activedescendant` wiring. Must be non-blank and unique within the
  current `items`; see `validate_result_list_items`.
- `row: ResultRow` — the same display-only title/subtitle/snippet shape
  `ResultList` uses.
- `payload: T` — the value returned to the consumer on activation, entirely
  independent of `row`.

Selection and activation are always resolved by looking up the *current*
`key` against the *current* `items` (`current_result_item`), never by
capturing a value when a row's view was first created. That is what makes the
following safe:

- **Reordering** `items` (a fresh search rescoring results) keeps the
  selected key highlighted at its new position.
- **Duplicate-looking rows** (`case-a`/`case-b`, both "Alex Morgan") each
  activate their own distinct payload — clicking one never returns the
  other's.
- **Relabeling** the selected row (same key, new display text) keeps the
  selection; the newly rendered title is what's on screen, but the key (and
  therefore the returned payload) is unchanged.
- **Inserting** a new top result does not shift an existing selection by
  index, because nothing is tracked by index.
- **Removing** the selected key falls back to reconciliation
  (`reconcile_result_key`): the first remaining result is selected, or `None`
  for an empty list — never a stale payload for a key that no longer exists.

## Core inputs

| Prop | Purpose |
|---|---|
| `items: Signal<Vec<ResultListItem<T>>>` | Required. Keyed, typed results to display, top to bottom. |
| `empty_message: Signal<String>` | Message shown in place of the list when `items` is empty. Defaults to `"No results found."`. |
| `on_select: Option<Callback<ResultListItem<T>>>` | Fired on activation (`Enter` or click) with the exact current `ResultListItem` for the activated key. |
| `on_selection_change: Option<Callback<Option<String>>>` | Fired whenever the highlighted **key** changes: keyboard nav, click, or the reconciliation that runs after `items` is replaced. |
| `class` | Additional CSS classes for the listbox container. |
| `node_ref: NodeRef<Div>` | References the listbox container div. |

`ResultList`'s props are the same shape, minus the identity split:
`items: Signal<Vec<ResultRow>>`, `on_select: Option<Callback<ResultRow>>`,
`on_selection_change: Option<Callback<Option<usize>>>` (an index, since rows
have no independent key).

When `items` carries a blank key or a duplicate key, the listbox renders an
inline error banner (`role="alert"`, `data-result-list-key-error`) instead of
guessing which row a duplicate key means — see
`validate_result_list_items`/`ResultListKeyError`.

## Migration from `ResultList`

`ResultList` remains fully source-compatible — it is now a thin adapter over
the same private core, synthesizing a `ResultListItem<(usize, ResultRow)>`
per row with a content-hashed key (`legacy-{index}-{hash}`) and a
`ResultReplacementPolicy::ResetFirst` policy (selection always resets to the
first row after a replacement, matching its pre-existing behavior). No
existing `ResultList` caller needs to change.

To move a call site to `KeyedResultList<T>`:

1. Decide what `T` is — usually whatever your search/lookup returns before
   you reduced it to a `ResultRow` for display (a full record, a database id,
   a case number).
2. Decide what the stable `key` is — usually that same id, stringified. It
   must be unique per current result and must not be derived from `row`
   (that reintroduces the duplicate-title ambiguity `KeyedResultList` exists
   to avoid).
3. Build `Vec<ResultListItem<T>>` with `ResultListItem::new(key, row,
   payload)` instead of `Vec<ResultRow>`.
4. Change `on_select: Callback<ResultRow>` to
   `Callback<ResultListItem<T>>`, and read `.payload` instead of the row
   itself.
5. Change `on_selection_change: Callback<Option<usize>>` to
   `Callback<Option<String>>` if you track the highlighted result outside the
   component; the value is now a key, not an index.

```rust,ignore
use leptos::prelude::*;
use leptos_daisyui_rs::*;

#[derive(Clone)]
struct CaseRef { case_number: &'static str }

let items = vec![
    ResultListItem::new(
        "case-a",
        ResultRow::new("Alex Morgan"),
        CaseRef { case_number: "A-100" },
    ),
    ResultListItem::new(
        "case-b",
        ResultRow::new("Alex Morgan"),
        CaseRef { case_number: "B-200" },
    ),
];

view! {
    <KeyedResultList
        items=Signal::derive(move || items.clone())
        on_select=Callback::new(|item: ResultListItem<CaseRef>| {
            leptos::logging::log!("activated {}", item.payload.case_number);
        })
    />
}
```

## Add to `input.css`

`KeyedResultList` renders the same markup/classes as `ResultList` — no
additional `@source inline(...)` entries are needed beyond what `ResultList`
already documents:

```css
@source inline("flex flex-col gap-2 max-h-80 overflow-y-auto rounded-box border border-base-300 bg-base-100");
@source inline("outline-none focus:ring-2 focus:ring-primary/50");
@source inline("px-3 py-2 cursor-pointer");
@source inline("bg-primary/10 text-primary bg-base-200");
@source inline("font-semibold text-sm truncate");
@source inline("text-xs opacity-60 whitespace-normal break-words");
@source inline("p-4 text-sm text-center opacity-60");
```

## Demo and browser coverage

The `/components/result-list` showcase page's "Keyed & Typed Results" section
exercises a three-row fixture where two rows intentionally share the display
title "Alex Morgan", with buttons to reorder, remove, insert, relabel, and
clear the result set — reproducing every replacement shape the acceptance
contract calls out. `tests/keyed_result_list_smoke.rs`
(`cargo xtask test-keyed-result-list`) drives that fixture in a real browser
and asserts duplicate-label activation, reorder/insert/remove/relabel
reconciliation, keyboard navigation with `aria-activedescendant`, and the
empty state. Native unit coverage for the key-identity model lives in
`src/components/result_list/tests.rs`
(`cargo test --lib components::result_list --no-default-features`).
