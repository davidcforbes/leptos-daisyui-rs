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

@media (prefers-reduced-motion: reduce) {
    .ld-eased,
    .ld-elevated {
        transition: none;
    }
    .ld-pressable:active:not(:disabled):not([aria-disabled='true']),
    .ld-elevated:hover {
        transform: none;
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
