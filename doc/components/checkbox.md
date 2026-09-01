# Checkbox

A daisyUI checkbox that can stay natively uncontrolled (the default) or take
part in controlled application state through one atomic change proposal
(`ldui-fqan`).

## Ownership: the one thing to decide first

`Checkbox` follows the contract the rest of this library converged on
(`EntityTableMultiSelection`, `ServerTableMultiSelection`, `ModalCloseProposal`):

- The **caller owns accepted truth** as a `Signal`.
- The component emits **one atomic typed proposal** per user gesture.
- The component **never diverges optimistically** from accepted state.

A checkbox needs one extra step the other three do not. The browser flips
`input.checked` *natively*, before any handler runs, so "decline to write" is
not enough — the write already happened. The change handler therefore
**re-asserts the accepted value onto the element before it proposes anything**,
which is what makes a declined proposal a visual no-op.

| You want | Use |
|---|---|
| A checkbox the browser owns (forms, static markup, no app state) | nothing — or `default_checked` for the initial value |
| A checkbox whose value is application state (a filter, a setting, a row selection) | `binding=CheckboxBinding::controlled(...)` |
| A tri-state "some of these are selected" | `.with_indeterminate(...)` on the binding |

Supplying `binding` **and** `default_checked` is refused, not resolved — see
[Refused configurations](#refused-configurations).

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `color` | `Signal<CheckboxColor>` | `Default` | daisyUI colour variant |
| `size` | `Signal<CheckboxSize>` | `Md` | daisyUI size variant |
| `disabled` | `Signal<bool>` | `false` | Native `disabled`; emits no proposals |
| `id` | `MaybeProp<String>` | minted / none | Stable DOM id. Wins over a surrounding `Field`'s id and over the mint |
| `name` | `MaybeProp<String>` | derived from `id` | Form key, passed through verbatim |
| `label` | `MaybeProp<String>` | none | Visible reactive label text; switches the root to a wrapping `<label>` |
| `aria_label` | `MaybeProp<String>` | none | Accessible name when there is no visible text. Mutually exclusive with `label` |
| `default_checked` | `MaybeProp<bool>` | none | Uncontrolled initial value. Mutually exclusive with `binding` |
| `binding` | `Option<CheckboxBinding>` | none | Controlled ownership (accepted signal + proposal callback) |
| `class` | `&'static str` | `""` | Extra classes, merged onto the `<input>` |
| `node_ref` | `NodeRef<Input>` | — | Reference to the `<input>` element |

### Types

- `CheckboxBinding` — `controlled(checked, on_change)`, plus
  `with_indeterminate(signal)`.
- `CheckboxChangeProposal` — `{ checked: bool, from: CheckboxState }`. `checked`
  is the **complete proposed value**, not a delta: apply or decline it wholesale.
- `CheckboxState` — `Unchecked` / `Checked` / `Mixed`, with `is_checked()`,
  `is_indeterminate()`, `aria_checked()`, `toggles_to()`, `as_str()`.

## Uncontrolled (the default, unchanged)

Existing call sites keep working exactly as before — same classes, same DOM, no
`id`, no `name`, no `aria-*`, no change handler.

```rust
view! {
    <Checkbox />
    <Checkbox color=CheckboxColor::Primary size=CheckboxSize::Sm />
    // Initial value, then the browser owns it:
    <Checkbox default_checked=true />
}
```

## Controlled

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn PastDueFilter() -> impl IntoView {
    // Accepted truth lives here, in the caller.
    let past_due_only = RwSignal::new(false);

    view! {
        <Checkbox
            id="past-due-only"
            label="Past due only"
            binding=CheckboxBinding::controlled(
                past_due_only.into(),
                Callback::new(move |proposal: CheckboxChangeProposal| {
                    past_due_only.set(proposal.checked);
                }),
            )
        />
    }
}
```

### Declining or delaying a proposal

Nothing is applied until *your* signal changes, so a validation gate, a
confirmation step or an in-flight request is expressed by simply not applying
the proposal yet:

```rust
Callback::new(move |proposal: CheckboxChangeProposal| {
    if saving.get_untracked() {
        return;                       // ignored: the DOM stays on accepted truth
    }
    spawn_local(async move {
        if persist(proposal.checked).await.is_ok() {
            past_due_only.set(proposal.checked);
        }
    });
})
```

An externally restored value (a reset button, a saved view, a route change)
reaches the DOM for the same reason: the rendered state follows **only** the
accepted signal, with no internal mirror to fall out of step.

### Inside a `FilterBar` or a column filter

Nothing special is required — a controlled `Checkbox` sits beside a controlled
`Select`/`Input` and reports to the same owner:

```rust
let filters = RwSignal::new(Filters::default());

view! {
    <div class="flex flex-wrap items-center gap-4">
        <Select /* ... */ />
        <Checkbox
            id="filters-past-due"
            label=Signal::derive(move || t("filters.past_due_only"))
            binding=CheckboxBinding::controlled(
                Signal::derive(move || filters.get().past_due_only),
                Callback::new(move |p: CheckboxChangeProposal| {
                    filters.update(|f| f.past_due_only = p.checked);
                }),
            )
        />
    </div>
}
```

## Indeterminate

There is **no `indeterminate` content attribute**. Markup that says
`indeterminate="true"` sets nothing at all; it is a DOM property only. Worse,
the browser *clears* the flag while handling a click, so a component that writes
it once on render silently degrades to a plain checkbox after the first
interaction (`ldui-nz6d`).

`Checkbox` handles both halves: it writes the property on render *and*
re-asserts it in the change handler, from the same `CheckboxState` the render
path reads, so the two cannot drift.

```rust
let all_selected = RwSignal::new(false);
let some_selected = RwSignal::new(true);

view! {
    <Checkbox
        id="select-all-offices"
        label="Select all offices"
        binding=CheckboxBinding::controlled(
            all_selected.into(),
            Callback::new(move |p: CheckboxChangeProposal| {
                all_selected.set(p.checked);
                some_selected.set(false);
            }),
        ).with_indeterminate(some_selected.into())
    />
}
```

- Mixed **wins over** checked: two accepted signals that disagree describe a
  partial selection, and drawing a full tick would claim more than you said.
- Mixed is announced as `aria-checked="mixed"`. The attribute is emitted *only*
  for mixed — a native checkbox already computes true/false correctly, and
  restating it would give assistive technology a second copy to contradict.
- A gesture from mixed proposes `true` ("select all of it"), matching every
  native tri-state control. `proposal.from` tells you it came from mixed, so
  "the user cleared a partial selection" is distinguishable from "the user
  unticked a full one".

## Identity: `id` and `name`

Following the scheme `ldui-j6sh` established for the table controls:

1. A caller-supplied `id` wins (normalized so it is a valid HTML id and a safe
   CSS selector: `[A-Za-z0-9_-]` survives, anything else is escaped as `_` plus
   two hex digits).
2. Otherwise the id a surrounding [`Field`](#field-integration) minted.
3. Otherwise — **only when the component needs an id of its own**, i.e. when
   `label` is supplied and its `<label for=…>` needs a target — a process-unique
   minted `ldui-checkbox-N`.
4. Otherwise no `id` attribute at all, which is what keeps existing callers
   byte-identical.

`name` matters separately from `id`: `name` is what makes the element a real
form control. A supplied `name` wins and is passed through **verbatim** (a form
key is the server's vocabulary — `filters[past_due]` — not an HTML id, and
normalizing it would silently rename the submitted field). Otherwise a supplied
`id` becomes the `name`.

A **minted** id never becomes a `name`. The mint depends on mount order, so
using it as a form key would change what the form submits whenever the page's
component order changed — an unstable `name` is worse than no `name`, because
the breakage is silent and lands on the server.

## Labelling and localization

`label` renders visible text beside the box; `aria_label` names a checkbox that
has no visible text (a selection column, a compact toolbar). Both are reactive
signals owned by the caller, so switching locale replaces the name **in place**
without re-mounting the input — focus survives.

> **Note:** `label`'s *presence* is structural — read once when the component is
> created, like `Input`'s `leading_icon` — and when it is present the
> component's root element is the wrapping `<label>`, so spread attributes land
> there rather than on the input. Use the typed props in that configuration, and
> select the input as `[data-testid="…"] input` from tests.

daisyUI 5 removed `.form-control`, `.label-text` and `.label-text-alt`; they do
nothing and a gate test fails if they reappear. The wrapper is a plain
`fieldset`/`label` + flex layout.

## `Field` integration

`Checkbox` consumes `FieldContext` exactly as `Input`, `Select` and `Textarea`
do, so wrapping one in a `Field` yields a fully associated control: the Field's
`label[for]` points at the checkbox's `id`, the help line is referenced via
`aria-describedby`, and the error line via `aria-errormessage` plus
`aria-invalid="true"`. An explicit `id` prop still wins over the Field's minted
one.

Put **one** control in a `Field` — a `Field` wrapping both a `Checkbox` and an
`Input` would hand both the same id.

## Refused configurations

Two configurations are ambiguous, and both are **refused rather than resolved**:

| Configuration | Why it is refused |
|---|---|
| `binding` + `default_checked` | Two sources of truth for one boolean — exactly the failure this contract exists to remove |
| `label` + `aria_label` | Different visible and accessible names is a WCAG 2.5.3 (Label in Name) failure and breaks speech control |

Either one renders a visible `role="alert"` panel carrying the reason (plus a
`data-checkbox-config-error` hook) and **no input at all**, so nothing
ambiguously-owned can be read back or submitted.

This follows `ServerDataTable`'s fail-closed panel rather than `EntityTable`'s
panic: a checkbox is a leaf control that may be rendered hundreds of times in a
list, and a panic in a CSR wasm app takes the whole page down with it.

## Accessibility

- Real label association: a wrapping `<label for=…>` for `label`, `aria-label`
  otherwise, and the `Field` association above when wrapped.
- Native keyboard operation: Space toggles, and it goes through the same
  one-proposal-per-gesture path as a click.
- `ld-focus-ring` gives a visible focus indicator; `ld-eased` respects
  `prefers-reduced-motion`. Both come from the framework, not the caller.
- Mixed state is conveyed by the native `indeterminate` property *and*
  `aria-checked="mixed"` — never by colour alone.
- `disabled` keeps native semantics and emits nothing.

## Style variants

### `CheckboxSize`
`Xs`, `Sm`, `Md` (default), `Lg`, `Xl`

### `CheckboxColor`
`Default`, `Primary`, `Secondary`, `Accent`, `Neutral`, `Success`, `Warning`,
`Info`, `Error`

## Add to `input.css`

```css
@source inline("checkbox checkbox-primary checkbox-secondary checkbox-accent checkbox-neutral checkbox-success checkbox-warning checkbox-info checkbox-error");
@source inline("checkbox-xs checkbox-sm checkbox-md checkbox-lg checkbox-xl");
@source inline("flex items-center gap-2 cursor-pointer cursor-not-allowed text-base-content/75 text-error");
```

## Coverage

- Native: `src/components/checkbox/state.rs` (state machine, proposals,
  ownership refusal, id/name derivation, and source scans proving the view is
  wired to all of it) and `src/components/checkbox/tests.rs` (backward
  compatibility of the uncontrolled branch, daisyUI 5 markup).
- Browser: the `ldui-fqan` block in `tests/reactivity_smoke.rs` — bare-checkbox
  DOM equality, `default_checked`, id/name/label association, external reset,
  accepted and declined proposals, indeterminate surviving a declined click,
  mixed transitions, disabled silence, EN↔ES label replacement on the same node,
  Space operation, and the refused configuration.
