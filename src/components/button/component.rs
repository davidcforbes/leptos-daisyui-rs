use super::style::{ButtonColor, ButtonShape, ButtonSize, ButtonStyle, ButtonType};
use crate::merge_classes;
use crate::utils::{RippleOverlay, use_ripple};
use leptos::{
    ev,
    html::{A, Button as HTMLButton},
    prelude::*,
};

/// Resolves the native `disabled` attribute for the underlying `<button>`.
///
/// `loading` alone does not disable a plain `type="button"` — it only shows
/// the spinner class — but for a [`ButtonType::Submit`] button that gap is a
/// real bug: a form would submit again on every click while a prior submit
/// is still in flight. Rather than special-case that on `ButtonType`, every
/// loading button is disabled at the DOM level for the duration of
/// `loading`, in addition to the explicit `disabled` prop — this also
/// stops `on_click`/ripple from firing mid-loading on ordinary buttons,
/// which is the same "already working" contract users expect from a
/// disabled control. Native `<button disabled>` semantics mean the browser
/// itself refuses to dispatch `click`, so no extra guard is needed in the
/// `on:click` handler below.
pub(crate) fn resolve_native_disabled(disabled: bool, loading: bool) -> bool {
    disabled || loading
}

/// # Button Component
///
/// A reactive wrapper for daisyUI's button component with comprehensive styling options
/// including colors, sizes, shapes, and interactive states.
///
/// ## Native form semantics (`button_type`, ldui-9vs)
///
/// `button_type` sets the native `type` attribute and defaults to
/// [`ButtonType::Button`], so every existing caller keeps rendering
/// `type="button"` unchanged. Set it to [`ButtonType::Submit`] for a real
/// native form submission (no JS shim — Office satellites use this for
/// POST-launch forms) or [`ButtonType::Reset`] for native form reset.
/// `Button` does not own or infer the containing `<form>`'s `action`/
/// `method`/`target` — those stay entirely caller-owned on the `<form>`
/// element itself.
///
/// ```rust,ignore
/// // Native POST launch form: the form owns action/method, the button just
/// // triggers a real submit, no on_click / on:submit JS required.
/// view! {
///     <form action="/office/launch" method="post" target="_blank">
///         <input type="hidden" name="doc_id" value=doc_id />
///         <Button button_type=ButtonType::Submit color=ButtonColor::Primary>
///             "Open in Office"
///         </Button>
///     </form>
/// }
/// ```
///
/// Because `type="submit"`/`type="reset"` is native HTML button behavior,
/// keyboard activation works with no extra wiring: a focused submit/reset
/// button activates on Enter (keydown) and Space (keyup), exactly like a
/// mouse click, per the browser's own default `<button>` behavior — nothing
/// in this component intercepts or duplicates that, so it fires the
/// containing form's submit/reset exactly once per activation, never twice.
///
/// ### Disabled and loading buttons cannot submit
///
/// `loading=true` disables the underlying `<button>` at the DOM level for
/// as long as it is `true`, in addition to the explicit `disabled` prop —
/// see [`resolve_native_disabled`]. A native `disabled` button dispatches no
/// `click` at all (browser-enforced), so neither a disabled nor a loading
/// [`ButtonType::Submit`]/[`ButtonType::Reset`] button can submit/reset its
/// form, by mouse or keyboard. This is a small, deliberate behavior change
/// from before ldui-9vs: `loading` used to be purely a CSS spinner and left
/// the button fully interactive.
///
/// ### Nested/form-associated edge cases
///
/// - **Multiple submit buttons in one `<form>`**: each keeps its own
///   `name`/`value` pair (pass them as spread attrs, e.g.
///   `name="action" value="save"`); the browser includes only the
///   *activated* button's pair in the submission, standard native `<form>`
///   behavior — `Button` adds no extra logic here.
/// - **A `Reset` button never restores reactive (`prop:value`-bound) fields
///   to their signal's current value** — native reset restores each field to
///   its `defaultValue`/`defaultChecked` (the *HTML attribute* at mount,
///   e.g. a literal `value="…"` on the input), not to whatever a Leptos
///   signal currently holds via `prop:value`. An input driven purely by
///   `prop:value` with no `value` attribute defaults to empty, so it resets
///   to empty, not to the signal's value — keep an uncontrolled `value`
///   attribute (or re-sync the signal from a `on:reset` handler) if a
///   different default is wanted.
/// - **A `Submit`/`Reset` button outside any `<form>`** (and not associated
///   via the HTML `form="id"` attribute) is inert for that purpose — clicking
///   it does nothing beyond firing `on_click`, per native HTML; this is not
///   a `Button`-specific behavior.
///
/// ## Precedence vs a spread `attr:type`
///
/// `Button` always emits its own `type` attribute (driven by `button_type`).
/// Spreading a duplicate — `<Button attr:r#type="submit" .../>` (`type` is a
/// Rust keyword, so the spread needs the raw-identifier escape) — is
/// redundant and its result should not be relied upon, but concretely: on
/// this crate's CSR-only rendering path, spread attributes are applied
/// *after* the component's own view is built, so `set_attribute` runs last
/// and **the spread `attr:r#type` wins**, silently overriding
/// `button_type` — verified directly by the demo's
/// `#button-type-precedence-probe` fixture and
/// `spread_attr_type_overrides_the_button_type_prop` in
/// `tests/reactivity_smoke.rs` (ldui-9vs), same mechanism already documented
/// on [`Progress`](crate::components::progress::Progress)'s `attr:max`. Use
/// `button_type`, not a spread `type`.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("btn btn-neutral btn-primary btn-secondary btn-accent btn-info btn-success btn-warning btn-error btn-outline btn-dash btn-soft btn-ghost btn-link btn-xs btn-sm btn-md btn-lg btn-xl btn-wide btn-block btn-square btn-circle btn-active btn-disabled loading");
/// ```
///
/// ## Node References
/// - `node_ref` - References the `<button>` element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn Button(
    /// Shows loading spinner when true. Also disables the button at the DOM
    /// level for as long as it is true — see [`resolve_native_disabled`] and
    /// the component doc's "Disabled and loading buttons cannot submit"
    /// section.
    #[prop(optional, into)]
    loading: Signal<bool>,

    /// Native `type` attribute: [`ButtonType::Button`] (default, no form
    /// action), [`ButtonType::Submit`], or [`ButtonType::Reset`]. See the
    /// component doc's "Native form semantics" section.
    #[prop(optional, into)]
    button_type: Signal<ButtonType>,

    /// Button color variant
    #[prop(optional, into)]
    color: Signal<ButtonColor>,

    /// Button visual style
    #[prop(optional, into)]
    style: Signal<ButtonStyle>,

    /// Button size variant
    #[prop(optional, into)]
    size: Signal<ButtonSize>,

    /// Button shape/layout modifier
    #[prop(optional, into)]
    shape: Signal<ButtonShape>,

    /// Whether the button appears in active state
    #[prop(optional, into)]
    active: Signal<bool>,

    /// Whether the button is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Enables the click ripple effect. Defaults to off.
    #[prop(optional, into)]
    ripple: Signal<bool>,

    /// Node reference for the button element
    #[prop(optional)]
    node_ref: NodeRef<HTMLButton>,

    /// Optional click callback. This composes with the built-in ripple handler
    /// so consumers do not have to fall back to a raw `<button>` for actions.
    #[prop(optional)]
    on_click: Option<Callback<ev::MouseEvent>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Button content (text, icons, or other elements)
    children: Children,
) -> impl IntoView {
    let ripple_handle = use_ripple();
    view! {
        <button
            type=move || button_type.get().as_str()
            disabled=move || resolve_native_disabled(disabled.get(), loading.get())
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "btn ld-eased ld-pressable ld-focus-ring",
                    color.get().as_str(),
                    style.get().as_str(),
                    size.get().as_str(),
                    shape.get().as_str(),
                    class
                )
            }

            class:btn-active=active
            class:btn-disabled=disabled
            class:loading=loading
            class:ld-ripple-host=ripple
            on:click=move |ev| {
                if ripple.get_untracked() {
                    ripple_handle.trigger.run(ev.clone());
                }
                if let Some(callback) = on_click {
                    callback.run(ev);
                }
            }
        >
            {children()}
            {move || ripple.get().then(|| view! { <RippleOverlay handle=ripple_handle /> })}
        </button>
    }
}

/// # Link Button Component
///
/// An anchor element styled as a daisyUI button for navigation actions.
/// Provides the same styling options as Button but renders as a link.
///
/// ## Node References
/// - `node_ref` - References the `<a>` element ([HTMLAnchorElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement))
#[component]
pub fn LinkButton(
    /// URL to navigate to when clicked. Accepts static strings, owned `String`s
    /// (e.g. a per-row route like `format!("/matter/{case_no}")`), or reactive
    /// signals. Rendered verbatim as the `<a href>`: do not interpolate
    /// untrusted input (a `javascript:` scheme would execute), same contract
    /// as `MenuItem`/`BreadcrumbItem` hrefs.
    #[prop(optional, into)]
    href: MaybeProp<String>,

    /// Button color variant (same as Button component)
    #[prop(optional, into)]
    color: Signal<ButtonColor>,

    /// Button visual style (same as Button component)
    #[prop(optional, into)]
    style: Signal<ButtonStyle>,

    /// Button size variant (same as Button component)
    #[prop(optional, into)]
    size: Signal<ButtonSize>,

    /// Button shape/layout modifier (same as Button component)
    #[prop(optional, into)]
    shape: Signal<ButtonShape>,

    /// Enables the click ripple effect. Defaults to off.
    #[prop(optional, into)]
    ripple: Signal<bool>,

    /// Node reference for the anchor element
    #[prop(optional)]
    node_ref: NodeRef<A>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Link content (text, icons, or other elements)
    children: Children,
) -> impl IntoView {
    let ripple_handle = use_ripple();
    view! {
        <a
            href=move || href.get()
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "btn ld-eased ld-pressable ld-focus-ring",
                    color.get().as_str(),
                    style.get().as_str(),
                    size.get().as_str(),
                    shape.get().as_str(),
                    class
                )
            }
            class:ld-ripple-host=ripple
            on:click=move |ev| {
                if ripple.get_untracked() {
                    ripple_handle.trigger.run(ev);
                }
            }
        >

            {children()}
            {move || ripple.get().then(|| view! { <RippleOverlay handle=ripple_handle /> })}
        </a>
    }
}
