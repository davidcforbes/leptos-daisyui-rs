# Confirmable search-picker dialog

`ConfirmableSearchPickerDialog<T>` (`src/patterns/search_picker_dialog.rs`,
bead `ldui-iq0o`) is the review-before-mutation form of the search picker:
**search → select → explicitly confirm**. Selecting a result is reversible and
has no side effect; the caller's mutation runs only from the Confirm control.

## Which one do I want?

|                        | `SearchPickerDialog`                        | `ConfirmableSearchPickerDialog`                        |
| ---------------------- | ------------------------------------------- | ------------------------------------------------------ |
| Activating a row       | *is* the terminal action                    | only moves the caller's selected key                    |
| Caller callback        | `on_select`                                 | `on_confirm`                                            |
| Steps to act           | one                                         | two                                                     |
| `Enter` on a result    | activates                                   | nothing — confirming is a separate control              |
| Reach for it when      | navigating, opening, filtering (reversible) | assign, reassign, link-record, restore, choose-owner    |

The split is **at the type level, not behind a flag**. The confirmable
component passes no `on_select` to its `KeyedResultList` at all and exposes
`on_confirm` instead, so there is no configuration of either component in
which activating a result can reach a mutation callback. Nothing about
`SearchPickerDialog` changed: its props, its `on_select` terminal semantics,
and its browser coverage are all untouched, and its native build test is an
exhaustive props literal that fails to compile if a prop is ever added.

## Everything is controlled

`open`, `query`, `status`, `items`, `selected_key`, and `pending` are all
caller-owned signals. Every gesture emits a typed proposal and nothing is ever
applied optimistically:

| Gesture                             | Emits                                            |
| ----------------------------------- | ------------------------------------------------ |
| Click a row / Arrow / Home / End     | `on_selection_change(KeyedResultListSelectionProposal)` |
| Escape / backdrop / Cancel           | `on_close(SearchPickerDismissCause)`             |
| Confirm                              | `on_confirm(ResultListItem<T>)`                  |
| Retry on a state panel               | `on_retry(())`                                   |

Selection ownership is `KeyedResultList`'s own controlled model
(`KeyedResultListSelection::controlled`, bead `ldui-bf8c`), not a
re-implementation. That is what gives the next section for free.

## A selection survives the search narrowing past it

Search narrows `items`, so the selected row routinely leaves the visible set.
Two mechanisms keep the selection alive across that:

1. `KeyedResultList` renders **no false highlight** for a controlled key with
   no matching row, and never proposes clearing it. The highlight returns the
   instant a matching row does.
2. The pattern keeps the **last typed item the accepted key resolved to**, so
   the summary keeps naming the selection and Confirm keeps resolving it.

Resolution precedence (`resolve_search_picker_selection`) is *fresh item
first, retained item only as an identity fallback for the same key*:

- the key matches a row in the current `items` → that item (a relabelled or
  repayloaded row therefore updates the summary);
- else the retained item, **only if its key equals the accepted key**;
- else `None` — an unknown key resolves to nothing rather than to the
  previously selected item. Confirmation then fails closed.

## Confirmation fails closed

`search_picker_confirm_block` is pure and payload-free, and both the disabled
presentation and the click handler call it, so they cannot disagree:

| Block                  | `data-confirm-state`            | When                                              |
| ---------------------- | ------------------------------- | ------------------------------------------------- |
| `NoSelection`          | `blocked-no-selection`          | no key selected                                   |
| `Pending`              | `pending`                       | the caller reports a confirmation in flight       |
| `UnresolvedSelection`  | `blocked-unresolved-selection`  | a key that resolves to no typed item              |
| *(none)*               | `ready`                         | confirmation may proceed                          |

Precedence is `NoSelection` → `Pending` → `UnresolvedSelection`: a caller
reporting `pending` with no key is reporting a state this pattern cannot have
produced, and naming the missing selection is more useful than naming the
flight.

The handler recomputes resolution **untracked at activation time**, not from
the last render, so nothing that happened between paint and click — a landed
response, a cleared key, a confirmation that started — can let a write
through. A synthesized click on the visually blocked control performs no
mutation.

## Confirm is `aria-disabled`, never natively `disabled`

A natively disabled button leaves the accessibility tree and the tab order,
taking the explanation of *why* it is unavailable with it — exactly the users
who need that reason lose access to it. Worse for a pending control: a button
that natively disables itself under the user's own focus dumps focus to
`<body>` mid-interaction. So the control stays focusable, reports
`aria-disabled="true"`, and points `aria-describedby` at the element carrying
the blocking reason (`texts.confirm_blocked_no_selection`,
`texts.confirm_blocked_unresolved`, or `texts.confirm_pending`). Same ruling
and reasoning as `RecordHeader`'s quick actions (`ldui-9d0q`).

## Dismissing with a pending selection

`Escape`, the backdrop, and Cancel each emit `on_close` with the cause that
produced them (`SearchPickerDismissCause::{Escape, Backdrop, DialogForm,
Cancel}`) and **nothing else**. Two deliberate consequences:

- **The selection is not discarded.** The pattern never proposes clearing
  `selected_key` on dismissal, so reopening the dialog restores the selection,
  summary and all. A caller that *wants* dismissal to discard clears its own
  key from `on_close` — an explicit choice rather than a silent loss.
- **Dismissal is never blocked**, including while a confirmation is in flight.
  A pattern that refuses to close traps the user. Because `open` is
  caller-owned, a caller with an uncancellable write in flight can simply
  ignore the proposal; that veto belongs to the caller, not the component.

Every close still routes through `Modal`'s controlled-close contract
(`ldui-e0fw`/`ldui-rolc`), so the dialog's own `close()` runs and the platform
restores focus to the trigger.

## While a confirmation is in flight, and when it fails

`pending=true` blocks confirmation, swaps the Confirm label for
`texts.confirm_pending`, and marks the control `aria-busy="true"`. The dialog
**does not close itself on confirm** — it has no write to observe and cannot
know whether one succeeded. The caller closes on success. On failure it clears
`pending` and supplies `confirm_error`, which renders as a `role="alert"`
inside the dialog; because the dialog stayed open with its selection intact,
the user's context survives the failure and Confirm can simply be pressed
again.

## Focus

- **On open**: the search field, deferred a frame so it lands after
  `show_modal()` puts the dialog in the top layer.
- **On selection**: unchanged. Arrow/Home/End are forwarded from the focused
  search field into the listbox as a native `keydown`, so `KeyedResultList`'s
  own handler runs and focus never leaves the search field. `Enter` is
  deliberately **not** forwarded — the list has no `on_select` to run, and
  `Enter` must not be a one-keystroke path to a mutation.
- **On confirm**: unchanged. The control is `aria-disabled` rather than
  natively disabled precisely so it keeps focus while the write is in flight.
- **On close**: the platform returns focus to the trigger via `close()`.

## Ids and names

Every id and name is derived from the caller-supplied `control_id`, so two
simultaneous instances with distinct contract ids cannot collide:

| Derived                | Used for                              |
| ---------------------- | ------------------------------------- |
| `{control_id}-title`   | `aria-labelledby`                     |
| `{control_id}-description` | `aria-describedby` (only when `description` is `Some`) |
| `{control_id}-selection`   | the selected-result summary region |
| `{control_id}-confirm-hint` | the confirm control's `aria-describedby` target |
| `{control_id}-search`  | the search input's `name`             |
| `{control_id}-cancel` / `{control_id}-confirm` | the footer controls' `name`s |

The search input's `id` and its `<label for>` are minted by `Field`, which
already guarantees per-instance uniqueness and the label association.

## Copy

All pattern-owned user-visible copy lives in
`ConfirmableSearchPickerDialogTexts` (`search_label`, `search_placeholder`,
`selected_label`, `selected_none`, `cancel`, `confirm`, `confirm_pending`,
`confirm_blocked_no_selection`, `confirm_blocked_unresolved`) and is a
reactive `Signal`, so a locale change relabels the dialog live without
touching stable result keys or the selected identity. Loading/error/empty/
retry copy is `PageStatePanelTexts`, reused rather than duplicated.
`error_detail` and `confirm_error` are caller-supplied *content*, not
pattern-owned copy.

## Typed example

```rust,ignore
use leptos::prelude::*;
use leptos_daisyui_rs::components::{KeyedResultListSelectionProposal, ResultListItem, ResultRow};
use leptos_daisyui_rs::patterns::{ConfirmableSearchPickerDialog, SearchPickerStatus};

#[derive(Clone)]
struct Worker {
    worker_id: String,
}

#[component]
fn AssignOwner() -> impl IntoView {
    let open = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let status = RwSignal::new(SearchPickerStatus::Idle);
    let items = RwSignal::new(Vec::<ResultListItem<Worker>>::new());
    let selected_key = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);
    let confirm_error = RwSignal::new(None::<String>);

    view! {
        <ConfirmableSearchPickerDialog
            open=open
            control_id="standing-order-owner"
            title="Assign owner"
            description="Choose a worker, then confirm the assignment."
            query=query
            status=status
            items=items
            selected_key=selected_key
            pending=pending
            confirm_error=confirm_error
            on_query_change=Callback::new(move |text| query.set(text))
            on_selection_change=Callback::new(move |proposal: KeyedResultListSelectionProposal| {
                // Accepted truth stays caller-owned: apply, or decline.
                selected_key.set(proposal.key);
            })
            on_confirm=Callback::new(move |item: ResultListItem<Worker>| {
                pending.set(true);
                confirm_error.set(None);
                // ... start the write; on success set `open` false,
                // on failure clear `pending` and set `confirm_error`.
            })
            // Dismissal never clears `selected_key`: reopening restores it.
            on_close=Callback::new(move |_| open.set(false))
        />
    }
}
```

## Testing hooks

Located by stable data attributes, never by document position:

| Attribute                                        | Element                                    |
| ------------------------------------------------ | ------------------------------------------ |
| `data-confirmable-search-picker-dialog="true"`    | the dialog's modal box (also `data-control-id`) |
| `data-confirmable-search-picker-search="true"`    | the search input                           |
| `data-confirmable-search-picker-results="true"`   | the result-list wrapper                    |
| `data-confirmable-search-picker-summary="true"`   | the summary (also `data-selection-state` of `none`/`resolved`/`unresolved` and `data-selected-key`) |
| `data-confirmable-search-picker-confirm-hint="true"` | the confirm control's reason              |
| `data-confirmable-search-picker-error="true"`     | the `role="alert"` confirmation failure    |
| `data-confirmable-search-picker-cancel="true"`    | Cancel                                     |
| `data-confirmable-search-picker-confirm="true"`   | Confirm (also `data-confirm-state`)        |

Native coverage lives in `src/patterns/search_picker_dialog.rs`'s test module
(pure selection/confirmation resolution, block precedence, markers, copy).
Browser coverage lives in `tests/search_picker_dialog_smoke.rs`, lane
`cargo xtask test-search-picker-dialog`, over the
`/components/search_picker_dialog` showcase fixtures `confirm-x`/`confirm-y`.
