use leptos::prelude::*;
use ui_tokens::elevation::{LEVEL_2, LEVEL_4, LEVEL_8, LEVEL_16, LEVEL_64, Shadow};
use ui_tokens::motion::{DURATION_FAST_MS, DURATION_NORMAL_MS, DURATION_SLOW_MS, Easing};
use ui_tokens::radius;
use ui_tokens::spacing::{
    SPACE_HUGE, SPACE_L, SPACE_M, SPACE_S, SPACE_XL, SPACE_XS, SPACE_XXL, SPACE_XXS, SPACE_XXXL,
};
use ui_tokens::stroke;
use ui_tokens::typography as ty;

/// The spacing scale as `(css-suffix, dips)`, ascending.
///
/// Ordering matters: [`crate::tokens::spacing_scale_px`] and the Tailwind
/// theme generator both iterate this, so the CSS variables and the Tailwind
/// keys stay in lockstep by construction rather than by hand-copy.
pub const SPACE_STEPS: [(&str, f32); 9] = [
    ("xxs", SPACE_XXS),
    ("xs", SPACE_XS),
    ("s", SPACE_S),
    ("m", SPACE_M),
    ("l", SPACE_L),
    ("xl", SPACE_XL),
    ("xxl", SPACE_XXL),
    ("xxxl", SPACE_XXXL),
    ("huge", SPACE_HUGE),
];

/// The type ramp as `(css-suffix, font-size, line-height)`, largest first.
///
/// The line height is taken from `ui_tokens`' `LINE_*` ramp, never from a
/// font metric — that is the whole point of pinning the rhythm.
pub const TYPE_STEPS: [(&str, f32, f32); 6] = [
    ("display", ty::SIZE_DISPLAY, ty::LINE_DISPLAY),
    ("title", ty::SIZE_TITLE, ty::LINE_TITLE),
    ("subtitle", ty::SIZE_SUBTITLE, ty::LINE_SUBTITLE),
    ("body", ty::SIZE_BODY, ty::LINE_BODY),
    ("caption", ty::SIZE_CAPTION, ty::LINE_CAPTION),
    ("small", ty::SIZE_SMALL, ty::LINE_SMALL),
];

/// The stroke family as `(css-suffix, dips)`, ascending.
pub const STROKE_STEPS: [(&str, f32); 4] = [
    ("hairline", stroke::HAIRLINE),
    ("thin", stroke::THIN),
    ("accent", stroke::ACCENT),
    ("emphasis", stroke::EMPHASIS),
];

/// The canonical spacing scale in pixels, ascending.
///
/// Exposed so a consumer (or a spacing checker) can ask what is on the scale
/// without re-deriving the list.
pub fn spacing_scale_px() -> [f32; 9] {
    SPACE_STEPS.map(|(_, dips)| dips)
}

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

    // ---- spacing scale -------------------------------------------------
    // The first spacing custom properties on the web side. Until these
    // existed, Tailwind's defaults merely *happened* to agree with the
    // desktop scale; now there is one source of truth.
    for (name, dips) in SPACE_STEPS {
        css.push_str(&format!("  --ld-space-{}: {}px;\n", name, dips));
    }

    // ---- stroke family -------------------------------------------------
    // Borders and dividers, deliberately separate from spacing so a 1px
    // hairline stops being reported as an off-grid gap.
    for (name, dips) in STROKE_STEPS {
        css.push_str(&format!("  --ld-stroke-{}: {}px;\n", name, dips));
    }

    // ---- corner radii --------------------------------------------------
    for (name, dips) in [
        ("control", radius::CONTROL),
        ("card", radius::CARD),
        ("badge", radius::BADGE),
    ] {
        css.push_str(&format!("  --ld-radius-{}: {}px;\n", name, dips));
    }
    css.push_str("  --ld-radius-pill: 9999px;\n");

    // ---- type ramp + grid-aligned line heights -------------------------
    for (name, size, line) in TYPE_STEPS {
        css.push_str(&format!("  --ld-text-{}: {}px;\n", name, size));
        css.push_str(&format!("  --ld-line-{}: {}px;\n", name, line));
    }

    css.push_str("}\n");

    // Type-ramp classes. Every step pins BOTH size and line height, so a
    // stack of N lines occupies exactly N x line-height instead of N x
    // whatever the font's natural metrics happen to be. Inheriting the
    // browser or Tailwind default here is what makes the two faces drift.
    for (name, _, _) in TYPE_STEPS {
        css.push_str(&format!(
            "\n.ld-text-{name} {{\n  font-size: var(--ld-text-{name});\n  line-height: var(--ld-line-{name});\n}}\n"
        ));
    }

    // ldui-k4fn: the framework's STATIC card-elevation policy, emitted here
    // as well as into the generated `styles/tokens.css`, exactly like the
    // type ramp above. The generated stylesheet is the primary path — a
    // consumer that imports it gets this rule with nothing mounted — and
    // this copy only keeps a runtime-only consumer working. Both derive from
    // `ui_tokens::elevation`, so they cannot drift.
    //
    // Distinct from `ld-elevated` (see `super::animations`) on purpose:
    // that class is an interaction affordance (rests at LEVEL_4, lifts to
    // LEVEL_8 with a transform on hover, transitions both), which is wrong
    // for a read-only card. This rule paints one shadow and nothing else.
    // `--ld-card-shadow` is deliberately left undefined so the fallback
    // paints and a product theme can substitute its approved card shadow by
    // setting that one property on `:root`.
    // ldui-xr7i: this rule composes with Tailwind v4's ring/inset variables
    // exactly as its own `shadow-*` utilities do. A bare `box-shadow:` here
    // wins the WHOLE property over a layered `ring-*`/`focus-visible:ring-*`
    // utility on the same element and silently discards the ring -- which is
    // how SelectableSummaryCard lost its selection ring the day it adopted
    // this class. Transparent fallbacks keep it valid where no ring utility
    // ever registered the variables.
    css.push_str(
        "\n.ld-card-depth {\n  --tw-shadow: var(--ld-card-shadow, var(--ld-elevation-4));\n  box-shadow: var(--tw-inset-shadow, 0 0 #0000), var(--tw-inset-ring-shadow, 0 0 #0000), var(--tw-ring-offset-shadow, 0 0 #0000), var(--tw-ring-shadow, 0 0 #0000), var(--tw-shadow);\n}\n",
    );

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
