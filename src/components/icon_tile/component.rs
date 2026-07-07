use super::style::{IconTileColor, IconTileSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// A tinted rounded-square tile framing a centered icon glyph.
///
/// The recurring "leading icon" wrapper: a subtly-tinted square that frames
/// a small icon (an SVG, emoji, or `Icon`) with
/// independent background and foreground colors. Ported from d2d-ui's
/// owner-drawn `IconTile` control -- the Direct2D fill/glyph drawing is
/// replaced here by a `<div>` styled with Tailwind/daisyUI utility classes,
/// and the children slot takes the place of the fixed glyph string.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{IconTile, IconTileColor, IconTileSize};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <IconTile bg=IconTileColor::Error fg=IconTileColor::Error size=IconTileSize::Lg>
///             <span>"!"</span>
///         </IconTile>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("inline-flex items-center justify-center shrink-0 rounded-lg rounded-full");
/// @source inline("bg-neutral/10 bg-primary/10 bg-secondary/10 bg-accent/10 bg-info/10 bg-success/10 bg-warning/10 bg-error/10");
/// @source inline("text-neutral text-primary text-secondary text-accent text-info text-success text-warning text-error");
/// @source inline("w-6 h-6 w-8 h-8 w-10 h-10 w-12 h-12 w-16 h-16");
/// @source inline("text-xs text-sm text-base text-lg text-2xl");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn IconTile(
    /// Tile background tint color
    #[prop(optional, into)]
    bg: Signal<IconTileColor>,

    /// Icon glyph foreground color
    #[prop(optional, into)]
    fg: Signal<IconTileColor>,

    /// Size of the tile (also scales the icon glyph)
    #[prop(optional, into)]
    size: Signal<IconTileSize>,

    /// Render as a circle (`rounded-full`) instead of the default rounded
    /// square (`rounded-lg`). Mirrors d2d-ui's `with_corner_radius(size / 2.0)`.
    #[prop(optional, into)]
    circle: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Centered icon content (e.g. an inline SVG, emoji, or `Icon`)
    children: Children,
) -> impl IntoView {
    let radius_class = move || if circle.get() { "rounded-full" } else { "rounded-lg" };

    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "inline-flex items-center justify-center shrink-0",
                    bg.get().as_bg_class(),
                    fg.get().as_fg_class(),
                    size.get().as_str(),
                    radius_class,
                    class
                )
            }
        >
            {children()}
        </div>
    }
}
