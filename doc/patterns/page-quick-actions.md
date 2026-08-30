# Page quick actions

`PageQuickActions` (`src/patterns/page_quick_actions.rs`) is a small,
opinionated composition for a [`PageHeader`](../../src/patterns/page_header.rs)'s
`actions` slot: a wrapping row of icon-and-label command controls with
accessible group naming, consistent gaps, and stable icon/text alignment. It
exists because `PageHeader` only ever supplied a fixed, non-wrapping flex
row for `actions` -- a base page with seven cross-surface actions overflowed
it, and every consumer had to hand-compose icon, visible label,
tooltip/accessibility, and button/link/form styling from scratch. See
`ldui-ynmd.2`.

## What it owns, what stays caller-owned

`PageQuickActions` owns exactly three things:

- an accessible group name (`role="group"` + `aria-label`)
- a gap on the canonical spacing scale (`gap-2`, 8px) between actions
- left-to-right wrapping (`flex-wrap`), so a full row of actions moves onto
  further rows at compact widths instead of overflowing the page
  horizontally

It does **not** render buttons. Activation, routes, HTTP method/target, and
domain authorization stay entirely on whatever
[`Button`](../../src/components/button/component.rs)/
[`LinkButton`](../../src/components/button/component.rs) (or caller-owned
`<form>`) each action wraps. `PageQuickActionContent` is the companion
icon-plus-label content helper -- it goes *inside* an action element, never
around it, and owns icon size, icon/text gap, alignment, and (opt-in)
responsive label collapse.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, ButtonColor, ButtonSize, ButtonStyle, ButtonType, LinkButton};
use leptos_daisyui_rs::patterns::{PageHeader, PageQuickActionContent, PageQuickActions};

#[component]
fn Example() -> impl IntoView {
    view! {
        <PageHeader
            title="Active matters"
            subtitle="Everything currently open across the firm."
            actions=Box::new(|| view! {
                <PageQuickActions label="Case actions">
                    <Button style=ButtonStyle::Outline size=ButtonSize::Sm color=ButtonColor::Primary>
                        <PageQuickActionContent icon="plus" label="New matter" />
                    </Button>
                    <LinkButton href="/reports" style=ButtonStyle::Outline size=ButtonSize::Sm>
                        <PageQuickActionContent icon="file-text" label="Reports" />
                    </LinkButton>
                    // A native POST-launch action: the surrounding <form>
                    // owns action/method/target, ButtonType::Submit owns
                    // nothing beyond the native `type="submit"` attribute.
                    <form action="/office/launch" method="post" target="_blank">
                        <input type="hidden" name="doc_id" value="42" />
                        <Button button_type=ButtonType::Submit style=ButtonStyle::Outline size=ButtonSize::Sm>
                            <PageQuickActionContent icon="external-link" label="Open in Office" />
                        </Button>
                    </form>
                </PageQuickActions>
            }.into_any())
        />
    }
}
```

## Visual convention: outline, small

`PageQuickActions` cannot reach into opaque children to force a size or
style, so it documents rather than enforces a convention: use
`ButtonStyle::Outline` at `ButtonSize::Sm` for quick actions. That gives a
consistent secondary hierarchy beside a header's title, distinct from a
page's one primary call-to-action, across every consumer that follows the
convention. A composition that deliberately wants a primary quick action can
still set `ButtonColor::Primary`/`Secondary` while keeping the outline style
and small size, as in the example above.

## Icon-only collapse: `PageQuickActionLabelVisibility`

`PageQuickActionContent`'s `label_visibility` prop defaults to
[`PageQuickActionLabelVisibility::Always`] (label always visible). Setting it
to `CollapseBelowSm` hides the label *visually* below Tailwind's `sm`
breakpoint via `sr-only sm:not-sr-only sm:inline` -- **never** `hidden`, so
the label stays in the accessibility tree at every width and the
surrounding Button/LinkButton's accessible name never changes. Wrap the
surrounding action element in [`Tooltip`](../../src/components/tooltip/component.rs)
with the same text so a sighted mouse/keyboard user still sees the label
once it collapses:

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, ButtonSize, ButtonStyle, Tooltip};
use leptos_daisyui_rs::patterns::{PageQuickActionContent, PageQuickActionLabelVisibility, PageQuickActions};

#[component]
fn Example() -> impl IntoView {
    view! {
        <PageQuickActions label="Case actions">
            <Tooltip tip="Export report">
                <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                    <PageQuickActionContent
                        icon="upload"
                        label="Export report"
                        label_visibility=PageQuickActionLabelVisibility::CollapseBelowSm
                    />
                </Button>
            </Tooltip>
        </PageQuickActions>
    }
}
```

## `PageHeader`'s wrapping actions host and typed divider

`PageHeader`'s `actions` slot host renders `flex flex-wrap items-center
gap-2` in both `PageHeaderNavigationLayout` branches (it previously omitted
`flex-wrap`, which is the fixed-row bug this pattern exists to fix) --
`PageQuickActions` composes on top of that as a second, independently
wrapping layer, but plain children dropped directly into `actions` now wrap
too.

`PageHeader` also accepts a typed `divider` prop
(`PageHeaderDivider::Shown` default / `Hidden`) so a base-page composition
that supplies its own separation (or wants none) can omit the historical
`border-b border-base-300 pb-4` rule:

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{PageHeader, PageHeaderDivider};

#[component]
fn Example() -> impl IntoView {
    view! {
        <PageHeader
            title="Coordinator workbench"
            divider=PageHeaderDivider::Hidden
        />
    }
}
```

`divider` defaults to `Shown`, so every existing `PageHeader` caller (none of
which pass it) renders exactly the border it already has -- source-compatible
by construction.

## Reference fixture

`/components/page_quick_actions` in the showcase demo
(`demo/src/demos/page_quick_actions.rs`) exercises:

- a base page (no back button) with seven actions at wide width
- the same seven actions with `divider=PageHeaderDivider::Hidden`
- a long localized title/subtitle beside the seven actions (reactive
  toggle)
- a narrow, fixed-width container that forces the seven actions to wrap
  onto more than one row without horizontal overflow
- two actions using `PageQuickActionLabelVisibility::CollapseBelowSm` inside
  `Tooltip`, at a compact width

`tests/page_quick_actions_smoke.rs` drives that fixture in a real browser
(`cargo xtask test-page-quick-actions`) and asserts: seven distinct
non-empty accessible names inside one named group; the divider marker
reflects the `Shown`/`Hidden` prop; the compact fixture never overflows its
own container and spans more than one row; the collapsed actions keep their
`sr-only` accessible label and a matching `Tooltip` `data-tip`; and the
localized title updates reactively without disturbing the actions row.
