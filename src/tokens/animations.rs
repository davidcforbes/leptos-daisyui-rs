use leptos::prelude::*;

pub use ldui_design::tokens::ui_animations_css;

/// Component that mounts the shared animation primitives once. Render it
/// near the root of your app alongside [`UiTokensPreamble`](super::UiTokensPreamble).
///
/// The stylesheet is [`ui_animations_css`], which lists every class it defines.
#[component]
pub fn UiAnimationsPreamble() -> impl IntoView {
    view! { <style>{ui_animations_css()}</style> }
}
