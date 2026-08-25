use crate::merge_classes;
use leptos::{ev, html::Button as HTMLButton, prelude::*};

/// # Pressable Component
///
/// An intentionally **unstyled** action primitive: a real `<button>` that
/// carries the library's behavioral contract without any `.btn` geometry.
///
/// ## Why "unstyled" is a designed state, not an omission
///
/// Some actions are semantically buttons but visually something else — a menu
/// item inside a `dropdown-content` list, an icon-only toggler, a clickable
/// table cell, a card that selects on press, a link-styled navigation action.
/// Forcing `.btn` onto those breaks their layout; hand-rolling a raw
/// `<button>` loses the shared contract (and reads as drift to the
/// `ldui-audit` `button-without-btn` heuristic, which cannot tell a designed
/// unstyled action from an accident). `Pressable` expresses the intent in the
/// type system instead of in ad-hoc markup or exemption comments.
///
/// ## The contract it does provide
///
/// - `type="button"` — never an implicit form submit.
/// - Native `disabled` semantics (a reactive [`Signal<bool>`]).
/// - The library's focus/press affordances **only**: `ld-focus-ring`
///   (focus-visible ring) and `ld-pressable` (subtle press scale), eased by
///   `ld-eased` — i.e. exactly [`Button`](crate::components::Button)'s
///   behavioral classes minus the `.btn` geometry/color.
/// - `data-pressable="true"` — the auditable marker. The `ldui-audit`
///   component-drift sweep recognizes it and does not flag the element as a
///   raw button, so audit counts drop honestly rather than by exemption.
///
/// All visual classes (geometry, color, layout) are the **caller's**: pass
/// them through `class` (reactive — accepts `&'static str`, `String`, a
/// `Signal<String>`, or `Signal::derive(...)`). Extra attributes
/// (`attr:aria-label`, `attr:title`, ...) and event listeners (`on:click`)
/// spread onto the `<button>` as with any Leptos 0.7+ component.
///
/// ## When NOT to use it
///
/// - A visually button-shaped action: use [`Button`](crate::components::Button).
/// - Structural controls with their own pinned contracts (modal backdrops,
///   the worker-picker trigger pattern): keep the raw element and its
///   documented exemption.
///
/// ## Node References
/// - `node_ref` - References the `<button>` element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn Pressable(
    /// Whether the action is disabled (native `disabled` attribute).
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Node reference for the button element
    #[prop(optional)]
    node_ref: NodeRef<HTMLButton>,

    /// Optional click callback, mirroring `Button`'s callback composition.
    /// An `on:click` listener spread onto the component works equally; both
    /// may be present and both will run.
    #[prop(optional)]
    on_click: Option<Callback<ev::MouseEvent>>,

    /// The caller's visual classes — reactive, so selected/accent states can
    /// change at runtime (`class=Signal::derive(move || ...)`).
    #[prop(optional, into)]
    class: Signal<String>,

    /// Button content (text, icons, or other elements)
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            data-pressable="true"
            disabled=disabled
            node_ref=node_ref
            class=move || merge_classes!("ld-eased ld-pressable ld-focus-ring", class.get())
            on:click=move |ev| {
                if let Some(callback) = on_click {
                    callback.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
