# AppShell

`AppShell` composes a full-height application frame around caller-owned
navigation and content. Its optional `top_bar` and `status_bar` regions stay
pinned while the body consumes the remaining height with `min-h-0`; an
`AppShellContent` inside that body is the independently scrolling main region.

Callers that omit both regions retain the original single-row class and direct
child DOM. When either region is present, the root switches to a contained
column layout and marks its structure with `data-app-shell-root`,
`data-app-shell-body`, `data-app-shell-top-bar`, and
`data-app-shell-status-bar`.

## Application top bar

`AppShellTopBar` provides responsive `start`, `center`, and `end` slots. It
owns the banner landmark, spacing, and compact wrapping only. Brand values,
search transport, locale state, account data, navigation, and close behavior
remain caller-owned.

```rust,no_run
view! {
    <AppShell
        top_bar=Box::new(|| view! {
            <AppShellTopBar
                label=Signal::stored("Application controls".to_owned())
                start=Box::new(|| view! { <strong>"Acme"</strong> }.into_any())
                center=Box::new(|| view! {
                    <input type="search" aria-label="Search" />
                }.into_any())
                end=Box::new(|| view! {
                    <button type="button">"Account"</button>
                }.into_any())
            />
        }.into_any())
    >
        <AppShellContent>"Page content"</AppShellContent>
    </AppShell>
}
```

At compact widths the center slot takes a full wrapped row. DOM and keyboard
order remain start, center, end, then body even when CSS changes visual row
placement. Give the top bar a localized `label`; consumers cannot replace its
native `header`/`role="banner"` semantics through a slot.

## Consumer CSS

Consumers should scan the library Rust source as described in the
[DataTable consumer CSS guidance](./data_table.md#consumer-inputcss). The top
bar's responsive classes are emitted by the component source; no demo-only
stylesheet is required.
