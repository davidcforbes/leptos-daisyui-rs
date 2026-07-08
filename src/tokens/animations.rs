use leptos::prelude::*;

/// CSS for state-transition primitives and named keyframes shared across
/// `leptos-daisyui-rs` components. Every duration / easing / elevation
/// reference is a `var(--ld-*)` lookup, so the values flow from
/// [`UiTokensPreamble`](super::UiTokensPreamble), which in turn flows
/// from the `ui-tokens` crate.
///
/// Mount [`UiAnimationsPreamble`] once at the root of an app, alongside
/// `UiTokensPreamble`, to make the following classes available globally:
///
/// - `ld-eased` — smooth transitions on opacity, color, transform, shadow.
/// - `ld-pressable` — adds a subtle `scale(0.97)` on `:active`.
/// - `ld-elevated` — resting LEVEL_4 shadow that lifts to LEVEL_8 + a 1px
///   translate on hover.
///
/// Plus named entrance animations triggered by daisyUI's open-state classes:
///
/// - `<dialog class="modal">` scales up and the backdrop fades on `[open]`.
/// - `.toast > *` slides in from the corresponding edge.
/// - `.dropdown-open > .dropdown-content` slides down (also on `:hover`).
/// - `.drawer-open .drawer-side` slides in from the start or end edge.
///
/// Plus a component-driven flow animation:
///
/// - `.ld-vstep-rail > .ld-vstep-flow-dash` — a small dash sliding down a
///   [`VerticalSteps`](crate::components::VerticalSteps) rail segment.
/// - `.ld-aichat-msg-in` — a message bubble's entrance fade-in-and-rise,
///   applied on mount by [`AiChat`](crate::components::AiChat).
pub fn ui_animations_css() -> &'static str {
    r#"
.ld-eased {
    transition:
        background-color var(--ld-duration-fast) var(--ld-ease-standard),
        color var(--ld-duration-fast) var(--ld-ease-standard),
        border-color var(--ld-duration-fast) var(--ld-ease-standard),
        opacity var(--ld-duration-fast) var(--ld-ease-standard),
        transform var(--ld-duration-fast) var(--ld-ease-standard),
        box-shadow var(--ld-duration-normal) var(--ld-ease-standard);
}

.ld-pressable:active:not(:disabled):not([aria-disabled='true']) {
    transform: scale(0.97);
}

.ld-elevated {
    box-shadow: var(--ld-elevation-4);
    transition:
        box-shadow var(--ld-duration-normal) var(--ld-ease-standard),
        transform var(--ld-duration-normal) var(--ld-ease-standard);
}

.ld-elevated:hover {
    box-shadow: var(--ld-elevation-8);
    transform: translateY(-1px);
}

.ld-ripple-host {
    position: relative;
    overflow: hidden;
}
.ld-ripple-element {
    position: absolute;
    width: 0;
    height: 0;
    border-radius: 50%;
    background-color: currentColor;
    opacity: 0.25;
    pointer-events: none;
    transform: translate(-50%, -50%);
    animation: ld-ripple var(--ld-duration-normal) var(--ld-ease-standard) forwards;
}
@keyframes ld-ripple {
    to {
        width: 400px;
        height: 400px;
        opacity: 0;
    }
}

@keyframes ld-dialog-in {
    from { transform: scale(0.92); opacity: 0; }
    to   { transform: scale(1);    opacity: 1; }
}
@keyframes ld-backdrop-in {
    from { opacity: 0; }
    to   { opacity: 1; }
}
dialog.modal[open] > .modal-box {
    animation: ld-dialog-in var(--ld-duration-normal) var(--ld-ease-decelerate);
}
dialog.modal[open]::backdrop {
    animation: ld-backdrop-in var(--ld-duration-normal) var(--ld-ease-standard) forwards;
}

@keyframes ld-toast-in-top {
    from { transform: translateY(-1rem); opacity: 0; }
    to   { transform: translateY(0);     opacity: 1; }
}
@keyframes ld-toast-in-bottom {
    from { transform: translateY(1rem);  opacity: 0; }
    to   { transform: translateY(0);     opacity: 1; }
}
.toast.toast-top > *,
.toast.toast-middle > * {
    animation: ld-toast-in-top var(--ld-duration-normal) var(--ld-ease-decelerate);
}
.toast.toast-bottom > * {
    animation: ld-toast-in-bottom var(--ld-duration-normal) var(--ld-ease-decelerate);
}

@keyframes ld-dropdown-in {
    from { transform: translateY(-0.25rem) scale(0.96); opacity: 0; }
    to   { transform: translateY(0)        scale(1);    opacity: 1; }
}
.dropdown-open > .dropdown-content,
.dropdown.dropdown-hover:hover > .dropdown-content {
    animation: ld-dropdown-in var(--ld-duration-fast) var(--ld-ease-decelerate);
}

@keyframes ld-drawer-in-start {
    from { transform: translateX(-100%); }
    to   { transform: translateX(0); }
}
@keyframes ld-drawer-in-end {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
}
@keyframes ld-drawer-overlay-in {
    from { opacity: 0; }
    to   { opacity: 1; }
}
.drawer-open:not(.drawer-end) .drawer-side {
    animation: ld-drawer-in-start var(--ld-duration-normal) var(--ld-ease-decelerate);
}
.drawer-open.drawer-end .drawer-side {
    animation: ld-drawer-in-end var(--ld-duration-normal) var(--ld-ease-decelerate);
}
.drawer-open .drawer-overlay {
    animation: ld-drawer-overlay-in var(--ld-duration-normal) var(--ld-ease-standard);
}

@keyframes ld-message-in {
    from { opacity: 0; transform: translateY(0.35rem); }
    to   { opacity: 1; transform: translateY(0); }
}
.ld-aichat-msg-in {
    animation: ld-message-in var(--ld-duration-fast) var(--ld-ease-decelerate);
}

.ld-focus-ring:focus-visible {
    outline: 2px solid var(--p, currentColor);
    outline-offset: 2px;
    animation: ld-focus-ring-in var(--ld-duration-fast) var(--ld-ease-decelerate);
}
@keyframes ld-focus-ring-in {
    from { outline-color: transparent; }
    to   { outline-color: var(--p, currentColor); }
}

/* VerticalSteps: a small dash sliding down a "lit" rail segment, giving a
   sense of forward progress through a chain of dependent checks. Ported from
   d2d-ui's `VerticalSteps::draw`, which computed the dash's y-offset each
   frame from a 1400ms phase; here it is a plain looping CSS animation. */
.ld-vstep-rail {
    position: relative;
    overflow: hidden;
}
.ld-vstep-flow-dash {
    position: absolute;
    left: 50%;
    top: -0.75rem;
    width: 2px;
    height: 0.75rem;
    border-radius: 9999px;
    background-color: var(--color-accent-content, currentColor);
    transform: translateX(-50%);
    animation: ld-vstep-flow 1.4s linear infinite;
}
@keyframes ld-vstep-flow {
    0% { top: -0.75rem; opacity: 0; }
    10% { opacity: 1; }
    90% { opacity: 1; }
    100% { top: 100%; opacity: 0; }
}

@media (prefers-reduced-motion: reduce) {
    .ld-eased,
    .ld-elevated {
        transition: none;
    }
    .ld-pressable:active:not(:disabled):not([aria-disabled='true']),
    .ld-elevated:hover {
        transform: none;
    }
    dialog.modal[open] > .modal-box,
    dialog.modal[open]::backdrop,
    .toast > *,
    .dropdown-open > .dropdown-content,
    .dropdown.dropdown-hover:hover > .dropdown-content,
    .drawer-open .drawer-side,
    .drawer-open .drawer-overlay,
    .ld-ripple-element,
    .ld-focus-ring:focus-visible,
    .ld-vstep-flow-dash,
    .ld-aichat-msg-in {
        animation: none;
    }
    .ld-vstep-flow-dash {
        display: none;
    }
}
"#
}

/// Component that mounts the shared animation primitives once. Render it
/// near the root of your app alongside [`UiTokensPreamble`](super::UiTokensPreamble).
#[component]
pub fn UiAnimationsPreamble() -> impl IntoView {
    view! { <style>{ui_animations_css()}</style> }
}
