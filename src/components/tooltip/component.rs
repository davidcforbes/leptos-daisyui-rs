use super::style::{TooltipColor, TooltipEdge, TooltipPosition, tooltip_position_for_edge};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Tooltip Component
///
/// A reactive Leptos wrapper for daisyUI's tooltip component that displays contextual
/// messages on hover. Supports positioning, colors, and both simple text and custom content.
///
/// ## Edge awareness (Office op-du33s)
///
/// daisyUI positions the bubble with a static class and performs no
/// collision detection, so a trigger pressed against its container's edge
/// opens a `tooltip-top` bubble straight off that edge -- it is horizontally
/// centered on a trigger that has no room on one side. A trigger's owner
/// knows where it sits; pass [`TooltipEdge`] and the component resolves the
/// placement through [`tooltip_position_for_edge`], emitting exactly ONE
/// position class so nothing depends on stylesheet ordering:
///
/// ```rust,ignore
/// // last card in a strip -- the bubble grows inward, to the left
/// <Tooltip tip=help edge=TooltipEdge::Right>{trigger}</Tooltip>
/// ```
///
/// The resolved side is readable as `data-tooltip-placement` and the
/// declared edge as `data-tooltip-edge`, so a DOM oracle never has to parse
/// the class attribute.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("tooltip tooltip-top tooltip-bottom tooltip-left tooltip-right tooltip-neutral tooltip-primary tooltip-secondary tooltip-accent tooltip-info tooltip-success tooltip-warning tooltip-error tooltip-open");
/// ```
///
/// ## Node References
/// - `node_ref` - References the tooltip container `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
///
/// ## Spread attributes
///
/// Call-site `attr:`, `class:`, `style:`, and `on:` spread attributes land on
/// the tooltip container `<div>`, which remains this component's single root.
#[component]
pub fn Tooltip(
    /// Tooltip text content (simple string tooltip)
    #[prop(optional, into)]
    tip: Signal<String>,

    /// Preferred position of the tooltip relative to the trigger element.
    /// `edge` may flip it inward; see [`tooltip_position_for_edge`].
    #[prop(optional, into)]
    position: Signal<TooltipPosition>,

    /// Which edge of its container this trigger is pressed against.
    /// Defaults to [`TooltipEdge::Interior`], which never flips anything --
    /// every existing call site renders exactly as it did before this prop.
    #[prop(optional, into)]
    edge: Signal<TooltipEdge>,

    /// Color variant of the tooltip
    #[prop(optional, into)]
    color: Signal<TooltipColor>,

    /// Force tooltip to always be visible
    #[prop(optional, into)]
    open: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the tooltip container element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Trigger element and optional custom tooltip content
    children: Children,
) -> impl IntoView {
    let placement = move || tooltip_position_for_edge(&position.get(), edge.get());

    view! {
        <div
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "tooltip",
                    placement().as_str(),
                    color.get().as_str(),
                    class
                )
            }
            class:tooltip-open=open
            data-tip=move || tip.get()
            data-tooltip-placement=move || placement().as_placement()
            data-tooltip-edge=move || edge.get().as_str()
        >
            {children()}
        </div>
    }
}
