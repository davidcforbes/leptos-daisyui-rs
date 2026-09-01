# Section heading

`SectionHeading` (`src/patterns/section_heading.rs`) is the opinionated
eyebrow/title/description composition for content beneath a
[`PageHeader`](../../src/patterns/page_header.rs): a small kicker label, the
section's heading (`H2` by default, one level below `PageHeader`'s `<h1>`),
optional supporting copy, an optional status slot, and optional action
controls -- all on one responsive line that wraps at compact widths instead
of squeezing the title.

## `status` vs `actions`: two different contracts

`actions` is for interactive controls -- buttons, menus, anything a user can
activate. `status` is for noninteractive presentation -- a badge, a freshness
note, a maturity label. Nothing about `SectionHeading` makes `status` a
button or a menu item; it is exactly the `Children` the caller hands it,
rendered without any interactive role, tabindex, or click handler added.
Composing status content through `actions` would get the right geometry but
would misrepresent noninteractive text as a control -- that mismatch is what
`ldui-17rz` exists to close, not the geometry itself.

## Where `status` renders: `SectionHeadingStatusPlacement`

By default (`SectionHeadingStatusPlacement::Inline`) `status` renders in the
same flex row as the heading text -- right for a compact badge that belongs
immediately beside the title, e.g. a sync badge next to "Sync status".

`SectionHeadingStatusPlacement::Trailing` renders the same `status` slot as a
separate sibling, after the title/description group and before `actions`,
aligned to the trailing (far) edge at desktop widths and wrapping onto its
own row at compact widths. This is for an established, full-width section
header whose noninteractive maturity/freshness copy belongs at the far right
rather than crowding the title -- the exact shape of the Office Scorecard's
Commitments, Results produced, and Track Record headers (4iiz-Office
`op-dlfua.7.34`), which place text like `"Provisional -- pending measure
review"` against the right edge.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{SectionHeading, SectionHeadingStatusPlacement};

#[component]
fn ScorecardCommitments() -> impl IntoView {
    view! {
        <SectionHeading
            title="Commitments"
            status_placement=SectionHeadingStatusPlacement::Trailing
            status=Box::new(|| view! {
                <span class="ld-text-caption text-base-content/75">
                    "Provisional -- pending measure review"
                </span>
            }.into_any())
        />
    }
}
```

### Why an enum, not a `bool`

A `bool` prop (e.g. `status_trailing: bool`) would name the *mechanism* --
"is the status trailing or not" -- rather than the *intent*. It also leaves
no room for a third placement (e.g. a below-title full-width band) without
either overloading the same flag with a different meaning or adding a second
independent `bool` that some combination of both would leave ambiguous.
`SectionHeadingStatusPlacement` is exhaustively matched in the component
body, so adding a case later is a compile error everywhere it isn't handled,
not a silently-ignored flag.

### Backward compatibility

`status_placement` defaults to `Inline`. The `Inline` match arm forwards
`status` through unchanged and leaves the trailing slot empty, so the
title-row markup is byte-for-byte what it was before this prop existed --
no existing caller's rendering changes. `tests/section_heading_smoke.rs`
(`cargo xtask test-section-heading`) proves this on the live DOM against a
pre-existing `status` fixture that never passes `status_placement`.

### DOM order, duplication, and non-interactivity

Regardless of placement, DOM order is always title/description group, then
(when `Trailing`) status, then actions -- never the reverse, and status is
rendered exactly once (there is no code path that renders the `status`
slot's content into more than one place at a time). The trailing status
wrapper carries its own `data-section-heading-status` attribute, distinct
from `data-section-heading-actions`, so a test (or a consumer's own
tooling) can assert on one without matching the other. Neither wrapper adds
a `role`, a `tabindex`, or a click handler of its own -- whatever
interactivity exists is exactly what the caller's own `status`/`actions`
content brought with it, so a noninteractive `status` slot stays
noninteractive to assistive technology no matter which placement is chosen.

### Wrapping

Both the trailing status wrapper and the `actions` wrapper use
`sm:shrink-0`, and the title/description group uses `flex-1 min-w-0`. Because
the title group's `flex-1` claims the row's free space first, the trailing
status (and actions, when both are present) sit packed at the far edge
without needing `justify-between` to do any work -- and at a compact
viewport the parent's `flex-col` layout (row layout only applies from the
`sm:` breakpoint up) stacks every child onto its own line automatically, so a
long status never overlaps or compresses a long title. See the `desktop`
and `compact` cases in `tests/section_heading_smoke.rs` for the pinned
proof, including a long-title-plus-long-status fixture
(`section-heading-trailing-long-title` on the `/components/section_heading`
showcase page).

## Composition catalog

The showcase page (`demo/src/demos/section_heading.rs`) covers:

- Plain (no status, no actions)
- Inline status (default placement)
- Actions only
- Long copy with inline status and actions together
- Trailing status only
- Trailing status with actions
- Trailing status with a long title
- Localized (reactive) copy
