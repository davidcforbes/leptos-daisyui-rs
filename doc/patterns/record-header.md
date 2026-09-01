# Record header

`RecordHeader` (`src/patterns/record_header.rs`) is the opinionated *record
identity row* for a record-detail page: an avatar, the record's primary
title, compact metadata (some of it navigable), a single primary status, any
secondary classification badges, and an ordered set of glyph quick actions --
all on one responsive line.

It exists because record-detail consumers (Office Account, No-Hire Detail)
each rebuilt an incompatible identity row on top of the generic
[`PageHeader`](../../src/patterns/page_header.rs), drifting on avatar size,
metadata typography, link semantics, status vocabulary, and -- most
damagingly -- on whether a glyph-only action carried an accessible name at
all. See `ldui-9d0q`.

It is a **typed composition pattern, not a record-page generator.** It
fetches nothing, owns no tab state, and runs no domain logic.

## Where it sits

Three layers, three owners:

| Layer | Owner | Responsibility |
|---|---|---|
| `PageHeader` | the page | back navigation, the page's single `<h1>`, page-level actions |
| **`RecordHeader`** | **this pattern** | the record's identity row: avatar, `<h2>` title, metadata, status, badges, quick actions |
| `TabSet` + `TabPanel` | the page | the record's sections and their selected key |

`RecordHeader` deliberately starts at `HeadingLevel::H2` -- the same enum
[`SectionHeading`](../../src/patterns/section_heading.rs) uses, and the same
type-ramp mapping, so the two cannot drift. It never renders an `<h1>`;
a source-level test pins that.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Tab, TabPanel, TabSet, Tabs};
use leptos_daisyui_rs::patterns::{
    PageHeader, RecordAvatar, RecordBadge, RecordHeader, RecordMetaItem, RecordQuickAction,
    RecordStatus, RecordStatusTone,
};

#[component]
fn AccountDetail() -> impl IntoView {
    // Domain state stays here. RecordHeader never duplicates it.
    let (selected_tab, set_selected_tab) = signal("overview".to_string());
    let on_action = Callback::new(|id: String| leptos::logging::log!("action {id}"));

    view! {
        <PageHeader title="Accounts" subtitle="Every account in this office." />

        <RecordHeader
            id="account-identity"
            title="Northwind Logistics"
            avatar=Some(RecordAvatar::new("Northwind Logistics").initials("NW"))
            metadata=vec![
                RecordMetaItem::new("owner", "Owner", "Maria Gonzalez").icon("user"),
                RecordMetaItem::new("matter", "Matter", "MAT-1023")
                    .link("/matters/1023")
                    .icon("file-text"),
                RecordMetaItem::new("portal", "Portal", "ACC-2201")
                    .link("https://portal.example.com/acc-2201")
                    .external(),
            ]
            status=Some(RecordStatus::new("Active").tone(RecordStatusTone::Success))
            badges=vec![RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info)]
            actions=vec![
                RecordQuickAction::new("call", "phone", "Call account"),
                RecordQuickAction::new("email", "mail", "Email account"),
                RecordQuickAction::new("archive", "trash", "Archive account")
                    .disabled("Locked while a compliance review is open"),
            ]
            on_action=on_action
        />

        <TabSet
            id="account-tabs"
            label="Account sections"
            selected_key=selected_tab
            on_select=Callback::new(move |key| set_selected_tab.set(key))
        >
            <Tabs>
                <Tab tab_key="overview">"Overview"</Tab>
                <Tab tab_key="matters">"Matters"</Tab>
            </Tabs>
            <TabPanel tab_key="overview"><p>"Overview"</p></TabPanel>
            <TabPanel tab_key="matters"><p>"Matters"</p></TabPanel>
        </TabSet>
    }
}
```

The tab key lives in the page, not in the header, and the header does not
receive it. That is what "without duplicating domain state" means here: the
only state `RecordHeader` reads is what it renders.

## Typed inputs, not opaque slots

Every region is a typed value list rather than a `Children` slot:
`RecordAvatar`, `Vec<RecordMetaItem>`, `Option<RecordStatus>`,
`Vec<RecordBadge>`, `Vec<RecordQuickAction>`. That is a deliberate departure
from `SectionHeading`/`PageHeader`, which take slots.

A slot can render anything, so it cannot be checked. The defects this bead
was filed for -- a glyph-only button with no accessible name, a status
communicated only by a colour swatch, a disabled control with no reason --
are all *inside* the slot content, invisible to the framework. Typed inputs
make them structurally impossible: an action cannot be constructed without a
`label`, a status cannot be constructed without a visible word, and
`RecordQuickAction::disabled` takes the reason as its only argument.

Every builder is plain owned data. Rebuild the list to update it -- for a
data refresh or a locale change -- exactly as with
[`KpiItem`](./kpi-strip.md).

## Status is never conveyed by colour alone

Three redundant channels, in order of reliability:

1. **Text.** `RecordStatus::label` is always rendered visibly, and it is
   required. A status with no word is not constructible.
2. **Shape.** Each of the four semantic tones (`Info`, `Success`, `Warning`,
   `Error`) carries a distinct Lucide glyph. `Neutral` deliberately carries
   none: the sprite's honest neutral glyph is blank, and an invisible glyph
   is worse than no glyph. A unit test asserts the four glyphs are distinct
   *and* that each resolves in the shipped sprite -- an unknown name
   degrades silently to `blank`, which would quietly reduce tone to colour
   alone, the exact defect the mapping exists to prevent.
3. **Colour.** The daisyUI badge colour. Remove it entirely -- greyscale,
   forced-colors, colour-blind viewing -- and nothing is lost.

The same `RecordStatusTone` drives secondary badges and action feedback, so
one record never speaks three status vocabularies. Feedback copy uses
`text-base-content/75` for the neutral tone, never an `opacity-*` utility,
which the style audit rejects for contrast.

A screen reader hears the role too: the badge carries a visually hidden
`"Status: Active"` prefix built from `RecordHeaderTexts::status_label`, so
the badge announces what it is rather than a bare adjective.

## Glyph-only actions

A glyph carries no name. `RecordQuickAction::accessible_name` is therefore
the **only** name the control has, and it is used for three things at once:

- the control's `aria-label`,
- the tip of the `Tooltip` wrapping it, so a sighted mouse or keyboard user
  gets the same string on hover/focus,
- and, in tests, the single string both channels are asserted against -- they
  cannot drift, because they are literally the same value.

The name always leads with the action's label and appends the state
qualifier: `"Archive account (Locked while a compliance review is open)"`,
`"Email account (in progress)"`, `"Open in portal, opens in a new tab"`.
Every qualifier word comes from `RecordHeaderTexts`, so it localizes.

The focus ring is daisyUI/LDUI's own (`ld-focus-ring`, applied by `Button`),
so a keyboard user sees where they are.

### Disabled means `aria-disabled`, never native `disabled`

A natively disabled button is removed from the tab order, so its tooltip can
never be reached by keyboard -- the reason becomes invisible to exactly the
users who most need it. `RecordQuickActionState::Disabled(reason)` therefore
renders `aria-disabled="true"`, keeps the control focusable, and swallows
activation in the handler. It also avoids daisyUI's `btn-disabled` class,
which sets `pointer-events: none` and would kill the tooltip for mouse users
too. A source-level test pins both.

### Pending and keyed feedback

`RecordQuickActionState::Pending` swaps the glyph for a spinner, reports
`aria-busy`, and refuses further activation -- the control keeps its position
in the row, so the row does not reflow mid-action.

`RecordActionFeedback` is *keyed*: it hangs off the action's own `id` and is
rendered into a single `role="status" aria-live="polite"` region below the
row, tagged `data-record-action-feedback="<id>"`. That is how a glyph-only
control reports what it did. The message text carries the outcome; the tone
only colours it.

Domain callbacks stay consumer-owned throughout. A link action renders an
anchor to its `href`; every other action reports its `id` through
`on_action`. A non-`Ready` link degrades to a button, because an anchor
cannot express "disabled" accessibly and one left navigable would contradict
its own accessible name.

## Presentation states

`RecordHeaderState` changes the row *within itself*. The surrounding page,
its tabs, and its panels stay mounted -- a refresh failure must not blank a
page the user is reading.

| State | Identity | Actions | Notes |
|---|---|---|---|
| `Ready` | shown | shown | the default |
| `Loading` | skeletons | withheld | `aria-busy="true"`; the heading is a visually hidden `RecordHeaderTexts::loading`, so the region's accessible name is truthful rather than a stale record name |
| `Retained` | shown | shown | real but possibly stale data; a muted notice says so, and everything stays interactive |
| `Unavailable` | replaced | withheld | the caller's title is **not** rendered -- an identity that failed to load must never be presented as loaded |

`Unavailable` renders `RecordHeaderTexts::unavailable` as the heading. That
text is reactive, so parameterize it to name the record that failed
(`"Account ACC-2201 could not be loaded."`) rather than reaching for the
stale title.

## Layout and truncation

One row on `lg` and above (`lg:flex-row lg:items-center lg:justify-between`),
stacking into a predictable column below it. Two rules hold the row together:

- The identity cluster is `min-w-0 flex-1` with a `truncate` heading, so a
  long name shortens with an ellipsis rather than pushing anything. The full
  string stays available through the heading's `title` attribute.
- The status/actions edge is `shrink-0`, so it never gives up width to long
  identity text and never overlaps it.

All spacing is on the canonical scale (`gap-1`, `gap-2`, `gap-3`, `gap-x-4`)
and the row has no padding of its own, so internal spacing can never exceed
the gap separating it from its neighbours. Type comes from the `ld-text-*`
ramp only.

## Consumer setup

The utilities the pattern emits are listed in an `@source inline(...)` block
on the component's own doc comment -- copy it into your `input.css`. The
`ld-text-*` steps are deliberately **not** listed: they are authored rules in
`styles/tokens.css`, not Tailwind utilities, so `@source inline` cannot
generate them (ldui-h7tw, ldui-fg2h). Import that stylesheet instead.

## Demo

`/components/record_header` (`demo/src/demos/record_header.rs`) carries
Account-style and No-Hire-style metadata with and without avatar, links and
badges, 1/2/3/4 glyph actions, every action state, every presentation state,
a long-identity truncation story, and the full
`PageHeader` / `RecordHeader` / controlled `TabSet` composition.
