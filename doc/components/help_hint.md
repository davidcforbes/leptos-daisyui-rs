# HelpHint

`HelpHint` supplies nonessential explanatory help through a shared `Button`
and `Tooltip` composition. Its disclosure state supports mouse hover, keyboard
focus, click/tap pinning, and Escape dismissal. A compact trigger has the same
accessible name as the visible-label variant.

Keep essential instructions, validation errors, and required action information
always visible. Use `Tooltip` alone for a short action label that does not need
this disclosure lifecycle. HelpHint content is explanatory text, not an
interactive menu or dialog.

## Basic use

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::HelpHint;

view! {
    <HelpHint
        id="billing-cycle-help".to_owned()
        label="About billing cycles".to_owned()
        text="Each completed cycle appears in the history list.".to_owned()
    />
}
```

The trigger's `id` must be nonempty and unique in the mounted document. Its
description has the stable ID returned by `help_hint_description_id(&id)`
(`billing-cycle-help-description` above). Supply distinct IDs when several
hints appear on a page; changing reactive text does not change their identity.

## Compact trigger with a subject

Optional children appear beside the help trigger and retain their own native
focus and activation behavior. The help trigger remains available even when
the adjacent subject is disabled. Associate a subject with the same description
explicitly when that relationship is appropriate:

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, HelpHint, help_hint_description_id};

let id = "history-export-help".to_owned();
let description_id = help_hint_description_id(&id);
view! {
    <HelpHint
        id=id
        label="About history exports".to_owned()
        text="Exports contain the records you are authorized to view.".to_owned()
        compact=true
    >
        <Button attr:aria-describedby=description_id disabled=true>
            "Export history"
        </Button>
    </HelpHint>
}
```

The compact trigger shows `?`; the full label remains its accessible name.
The subject owns any command handler. Activating the help trigger never invokes
that handler or submits the containing form.

## API

| Prop | Type | Contract |
|---|---|---|
| `id` | `String` | Required caller-owned, page-unique stable trigger ID. |
| `label` | `Signal<String>` | Required accessible name; visible unless compact. Accepts an `Into` value. |
| `text` | `Signal<String>` | Required supplementary description; rendered as escaped DOM text. Accepts an `Into` value. |
| `compact` | `bool` | Defaults to `false`; selects the compact icon-style trigger. |
| `children` | `Option<Children>` | Optional adjacent subject; does not become the help trigger. |

`help_hint_description_id(&str) -> String` lets callers reference the stable
description without duplicating the component's naming convention.

## Interaction contract

| Input | Result |
|---|---|
| Pointer enters the composition or keyboard focus enters it | Reveals transient help. |
| Pointer leaves | Clears hover; help stays visible if focused or pinned. |
| Click or tap | Pins; the first tap still pins when focus has already revealed help. |
| Second click/tap or activation | Unpins and dismisses, even while focus/hover remains. |
| Enter or Space on the focused trigger | Uses ordinary button activation to pin or toggle. |
| Focus leaves the entire composition | Clears focus and a keyboard-created pin; a pointer/touch pin remains. |
| Focus moves between subject and help trigger | Remains within the composition, not a blur/re-entry. |
| Escape while a hint is visible | Dismisses the most recently active visible hint, without moving focus. |

Escape also works when hover revealed help while focus stayed elsewhere. A
consumed Escape does not reach unrelated bubbling overlay handlers. Hidden
hints do not intercept it. When several hints are visible, successive Escapes
can dismiss them independently; unmounting one does not disable the others.

Dismissal remains effective while existing hover/focus remains. A new deliberate
entry or activation can reveal help again. Visibility is explicitly controlled,
so Tooltip's CSS hover/focus rules cannot silently reopen dismissed help.

The trigger exposes `aria-describedby` and truthful `aria-expanded`. The real
description node has `role="tooltip"`; the shared Button supplies keyboard
operation and focus-visible styling. Global Escape listeners and active-hint
registration are removed on component cleanup.

## Maintenance checks

- Keep dismissal state authoritative over Tooltip hover/focus CSS. Suppress
  both `::before` and `::after` locally: a real description node does not remove
  the shared tooltip's otherwise empty pseudo-element chrome.
- Preserve a continuous pointer path from the trigger into the description.
  Check the actual boundary and text bounds near the pane edge; visibility
  alone does not catch a dead hover gap or a clipped, centered tooltip.
- Test multiple pinned hints in reverse registration order. One Escape must
  dismiss only the latest visible hint, and hiding or unmounting it must leave
  the previous visible hint dismissible. Preserve immediate propagation stop
  for a consumed Escape and remove the exact registered listener on cleanup.
- Move keyboard focus between subject and trigger after dismissal; an internal
  transition must not masquerade as leaving and re-entering the composition.
  Assert the subject's action count remains unchanged during help interaction,
  then prove that direct subject activation still works.
- In the browser harness, scroll the fixture into view before coordinate input.
  CDP `touchEnd` needs an explicit empty `touchPoints` array; a serializer that
  omits it produces a transport error, not evidence of a component defect. Keep
  fixture/harness failures distinct from intentional product negative controls.

## Verification and consumer boundary

Run the focused release browser lane from the repository root:

```text
cargo xtask test-help-hint
```

Pure state tests run with:

```text
cargo test -p leptos-daisyui-rs --lib --features test-mode help_hint
```

The browser lane uses real pointer, keyboard and trusted touch input, separate
semantic and computed-visibility assertions, subject-action checks, multiple
instances, cleanup, browser error capture and accessibility checks. It is also
registered in `cargo xtask verify-full`. The Tooltip showcase includes both
trigger variants and an adjacent subject.

Consumers own the explanatory wording, domain policy and subject actions.
Migrating a consumer's local helper, accepting its visual baselines, and
deploying that application are separate consumer work. No ETL behavior or copy
is embedded in this library component.
