# Quick Start Guide

Get up and running with leptos-daisyui-rs in minutes.

## Installation

This internal fork is not published to crates.io. Check it out beside the
consuming repository and add it as a path dependency:

```toml
[dependencies]
leptos-daisyui-rs = { path = "../leptos-daisyui-rs" }
```

## Setup Tailwind CSS

Create or update your `input.css`:

```css
@import "tailwindcss";
@import "../leptos-daisyui-rs/styles/tokens.css";
@plugin "daisyui";
@source "../src/**/*.rs";
@source "../leptos-daisyui-rs/src/**/*.rs";
@source inline("btn btn-primary btn-secondary btn-accent btn-ghost");
```

Paths are relative to `input.css`; adjust them if the web crate is nested. The
token import is required for opinionated components. It supplies, among other
semantic values, the DataTable/EntityTable dark-blue header, light-blue aligned
filter row, and faint cell grid. Scanning the library source ensures Tailwind
emits the classes the Rust components use.

## Basic Usage

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div class="p-8">
            <h1 class="text-4xl font-bold mb-4">"Hello daisyUI!"</h1>
            <Button color=ButtonColor::Primary>"Click Me"</Button>
        </div>
    }
}
```

## With Theming

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::theme::ThemeProvider;
use leptos_daisyui_rs::components::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <ThemeProvider load_from_storage=true>
            <div class="min-h-screen p-8">
                <h1 class="text-4xl font-bold">"Themed App"</h1>
                <BaseThemeSelector />
            </div>
        </ThemeProvider>
    }
}
```

## Next Steps

- [Theming System Guide](./THEMING.md) - Comprehensive theming documentation
- [Component Documentation](./components/) - Individual component guides
- [Demo Application](../demo/) - Interactive examples
