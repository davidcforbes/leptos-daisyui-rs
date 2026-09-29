use leptos::prelude::*;

pub use ldui_design::tokens::{
    SPACE_STEPS, STROKE_STEPS, TYPE_STEPS, spacing_scale_px, ui_tokens_css,
};

/// Component that mounts the `--ld-*` token block once at the root of an
/// app. Render it near the top of your component tree (e.g. inside the
/// router shell or directly under the `Router`).
///
/// The resulting `<style>` element exposes the design tokens as CSS custom
/// properties on `:root`, so any rule downstream can reference
/// `var(--ld-duration-fast)`, `var(--ld-ease-standard)`,
/// `var(--ld-elevation-4)`, etc.
#[component]
pub fn UiTokensPreamble() -> impl IntoView {
    view! { <style>{ui_tokens_css()}</style> }
}
