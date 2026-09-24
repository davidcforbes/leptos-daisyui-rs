# leptos-daisyui-rs: how to build with this design system

This system's components are **Rust/Leptos components that render daisyUI 5
markup**. There are no React components to import: build designs from plain
HTML elements carrying the **same classes the real components emit**. The
stylesheet (`styles.css`) is the real compiled CSS of the showcase: daisyUI 5,
Tailwind v4 utilities, the shared `ui-tokens`, and the crate's own `ld-*`
classes. Engineers turn a design into code by using the Leptos component
named on each reference page.

**Loading**: link `styles.css` and nothing else. `window.LeptosDaisyui` is
empty, so skip the generic component-loading and `components/` instructions
that follow this guide; this system ships none.

## Where the truth lives

- `guidelines/guides/index.md`: every component, by group. Each page (e.g.
  `guidelines/guides/action/button.md`) holds the **real rendered markup** of
  each showcase example plus its screenshot. Copy element structure and
  classes from there; do not invent a variant the page does not show.
- `_ds_bundle.css` (imported by `styles.css`): the compiled stylesheet. The
  build is **purged**: only classes the crate or its showcase actually use
  exist. A Tailwind utility that appears in no reference page may not render.
  Prefer classes you can find in `guidelines/`.

## Setup

- Put the theme on the root: `<html data-theme="light">`. Enabled themes:
  `light` (default), `dark`, `cupcake`, `cyberpunk`, `retro`, `forest`,
  `aqua`, `dracula`. Colours come from the theme, never hard-coded hex.
- Page background `bg-base-100`, raised surfaces `bg-base-200`, borders
  `border-base-300`, text `text-base-content`.

## Styling idiom

- **daisyUI 5 component classes**: `btn btn-primary btn-sm`, `card card-body
  card-title`, `badge badge-outline`, `alert alert-info`, `input`, `select`,
  `textarea`, `checkbox`, `toggle`, `tabs tab tab-active`, `menu`, `modal
  modal-box`, `table`, `stats stat`, `tooltip`, `dropdown`, `navbar`,
  `drawer`, `join`, `loading loading-spinner`, `skeleton`.
- **daisyUI 5, not 4**: `form-control`, `label-text` and `label-text-alt` no
  longer exist. Stack a label and its input with `fieldset` + `label`, or
  `flex flex-col gap-2`.
- **Semantic colours**: `primary`, `secondary`, `accent`, `neutral`,
  `info`, `success`, `warning`, `error`, `base-100/200/300`,
  `base-content`, as `bg-*`, `text-*`, `border-*` and component modifiers.
- **Spacing is a fixed scale**: Tailwind steps `1 2 3 4 6 8 12 16 24` (4-96
  px) for `p-*`, `m-*`, `gap-*`. A container's padding never exceeds the gap
  between it and its neighbours.
- **Type ramp**: `ld-text-display`, `ld-text-title`, `ld-text-subtitle`,
  `ld-text-body`, `ld-text-caption`, `ld-text-small`.
- **Muted text** is `text-base-content/75` or darker; `/60` and `/70` fail
  contrast on the light theme.
- **Depth and motion**: `ld-card-depth` for card shadow, `ld-focus-ring` for
  keyboard focus, `ld-eased` for colour transitions. Tokens are CSS custom
  properties (`var(--ld-duration-normal)`, `var(--ld-ease-standard)`).
- **Copy the whole class list a component emits**, not just its daisyUI
  class: the real `Button` renders `btn ld-eased ld-pressable ld-focus-ring`
  plus a colour and a size (`btn-md`). The reference pages show each list.
- A disabled control always says why: pair it with visible or
  `aria-describedby` text.

## Example

```html
<div class="card bg-base-100 ld-card-depth">
  <div class="card-body gap-4">
    <h2 class="card-title ld-text-title">Open requests</h2>
    <p class="text-base-content/75">12 waiting for a reply</p>
    <div class="card-actions justify-end">
      <button class="btn ld-eased ld-pressable ld-focus-ring btn-ghost btn-md" type="button">Dismiss</button>
      <button class="btn ld-eased ld-pressable ld-focus-ring btn-primary btn-md" type="button">Review</button>
    </div>
  </div>
</div>
```
