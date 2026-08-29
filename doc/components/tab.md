# Tabs and TabSet

Use `TabSet` for switchable application content. It owns the WAI-ARIA tabset
contract while the caller remains the sole owner of the selected stable key,
panel content, routing, and localized copy. `Tabs` continues to own daisyUI's
visual list variants and sizes.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Tab, TabPanel, TabSet, Tabs, TabVariant};

let selected = RwSignal::new("overview".to_owned());

view! {
    <TabSet
        id="account-tabs"
        label="Account sections"
        selected_key=selected
        on_select=Callback::new(move |key| selected.set(key))
    >
        <Tabs variant=TabVariant::Border>
            <Tab tab_key="overview">"Overview"</Tab>
            <Tab tab_key="history">"History"</Tab>
            <Tab tab_key="billing" disabled=true>"Billing"</Tab>
        </Tabs>
        <TabPanel tab_key="overview">"Overview content"</TabPanel>
        <TabPanel tab_key="history">"History content"</TabPanel>
        <TabPanel tab_key="billing">"Billing content"</TabPanel>
    </TabSet>
}
```

## Controlled contract

`TabSet::id` and every `tab_key` are stable identity, not visible copy. The
framework hex-encodes each key into collision-safe tab and panel IDs and emits
the matching `aria-controls` and `aria-labelledby` pair. Duplicate keys fail
immediately. Labels and the tab-list accessible name are reactive and may be
localized without changing identity.

`selected_key` is controlled. Pointer activation and Enter or Space call
`on_select` with one proposed key; the old panel remains selected if the caller
rejects that proposal. Arrow keys move focus without selecting:

- Horizontal lists use Left and Right Arrow; vertical lists use Up and Down.
- Home and End reach the first and last enabled tabs.
- Directional movement wraps and skips disabled tabs.
- Exactly one enabled tab has `tabindex="0"`; Tab then leaves the composite
  for the selected panel instead of visiting every tab.

If the selected tab is removed or becomes disabled, the first enabled tab is
shown immediately and proposed through `on_select`. Focus moves there only
when the removed tab actually held focus, so unrelated user focus is not
stolen. If no enabled tab remains, no panel is selected and there is no tab
stop.

Horizontal controlled lists scroll internally at narrow widths and do not
widen the page. Focus outlines use the framework primary color and the system
Highlight color in forced-colors mode.

## Compatibility layout

Existing `Tabs` plus `Tab` calls outside `TabSet` remain the source-compatible
layout-only path. Their `active`, `disabled`, variants, sizes, placement,
classes, node references, and caller attributes are unchanged. That path does
not invent selection or panel semantics; use it only for static presentation
or while migrating. A `tab_key` is accepted only inside `TabSet`, and every
`Tab` inside `TabSet` must have one.

`TabRadio` remains the separate native-radio/form-integration variant and is
not part of the controlled tab-panel contract.
