use leptos::prelude::*;
use ui_tokens::elevation::{Shadow, LEVEL_16, LEVEL_2, LEVEL_4, LEVEL_64, LEVEL_8};
use ui_tokens::motion::{Easing, DURATION_FAST_MS, DURATION_NORMAL_MS, DURATION_SLOW_MS};

/// Build the CSS custom-property block exposing every `ui-tokens` design
/// token as a `--ld-*` variable on `:root`.
///
/// Values derive directly from `ui_tokens` constants, so a change upstream
/// flows through here without any hand-copy.
pub fn ui_tokens_css() -> String {
    let mut css = String::with_capacity(1024);
    css.push_str(":root {\n");

    css.push_str(&format!("  --ld-duration-fast: {}ms;\n", DURATION_FAST_MS));
    css.push_str(&format!(
        "  --ld-duration-normal: {}ms;\n",
        DURATION_NORMAL_MS
    ));
    css.push_str(&format!("  --ld-duration-slow: {}ms;\n", DURATION_SLOW_MS));

    for (name, easing) in [
        ("linear", Easing::Linear),
        ("standard", Easing::Standard),
        ("decelerate", Easing::Decelerate),
        ("accelerate", Easing::Accelerate),
    ] {
        let (x1, y1, x2, y2) = easing.bezier();
        css.push_str(&format!(
            "  --ld-ease-{}: cubic-bezier({}, {}, {}, {});\n",
            name, x1, y1, x2, y2
        ));
    }

    for (name, shadow) in [
        ("2", LEVEL_2),
        ("4", LEVEL_4),
        ("8", LEVEL_8),
        ("16", LEVEL_16),
        ("64", LEVEL_64),
    ] {
        css.push_str(&format!(
            "  --ld-elevation-{}: {};\n",
            name,
            shadow_to_box_shadow(shadow)
        ));
    }

    css.push_str("}\n");
    css
}

/// Format a [`Shadow`] as a single CSS `box-shadow` value.
fn shadow_to_box_shadow(s: Shadow) -> String {
    format!(
        "{}px {}px {}px rgba(0, 0, 0, {:.2})",
        s.offset_x, s.offset_y, s.blur, s.opacity
    )
}

/// Component that mounts the `--ld-*` token block once at the root of an
/// app. Render it near the top of your component tree (e.g. inside the
/// router shell or directly under the [`Router`]).
///
/// The resulting `<style>` element exposes the design tokens as CSS custom
/// properties on `:root`, so any rule downstream can reference
/// `var(--ld-duration-fast)`, `var(--ld-ease-standard)`,
/// `var(--ld-elevation-4)`, etc.
#[component]
pub fn UiTokensPreamble() -> impl IntoView {
    view! { <style>{ui_tokens_css()}</style> }
}
