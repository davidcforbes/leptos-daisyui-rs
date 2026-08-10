/// Preview line component for dependency creation drag operation
///
/// Shows a temporary line from source task to cursor position while dragging
/// to create a new dependency.
use crate::charts::paint::stroke_attrs_with;
use leptos::prelude::*;

/// The preview line's non-colour declarations. They share the element's single
/// `style` attribute with a routed colour — see `charts::paint::merge_style`.
/// Dropping `pointer-events: none` would let the preview swallow the drop.
const PREVIEW_STYLE: &str = "opacity: 0.7; pointer-events: none";

/// Component that renders the preview line during dependency creation
#[component]
pub fn DependencyPreview(
    /// Whether the preview is currently active
    #[prop(into)]
    is_active: Signal<bool>,

    /// X position where the drag started (source task)
    #[prop(into)]
    start_x: Signal<f64>,

    /// Y position where the drag started (source task)
    #[prop(into)]
    start_y: Signal<f64>,

    /// Current X position of the cursor
    #[prop(into)]
    current_x: Signal<f64>,

    /// Current Y position of the cursor
    #[prop(into)]
    current_y: Signal<f64>,

    /// Whether there's a valid drop target under the cursor
    #[prop(optional, into, default=Signal::derive(|| false))]
    has_valid_target: Signal<bool>,
) -> impl IntoView {
    let path_d = Signal::derive(move || {
        if !is_active.get() {
            return String::new();
        }

        let sx = start_x.get();
        let sy = start_y.get();
        let cx = current_x.get();
        let cy = current_y.get();

        // Simple straight line for preview (could be curved like dependency_link.rs)
        format!("M {} {} L {} {}", sx, sy, cx, cy)
    });

    // daisyUI 5 renamed the semantic colours: `--su`/`--wa` (and their
    // `--fallback-*` companions) no longer exist, so the whole `oklch()` that
    // used to be emitted here failed to parse and `stroke` fell back to its
    // initial `none` — the preview line was invisible (ldui-xxc).
    let stroke_color = Signal::derive(move || {
        if has_valid_target.get() {
            "var(--color-success)".to_string() // Over a valid drop target
        } else {
            "var(--color-warning)".to_string() // Otherwise
        }
    });
    let preview_stroke = move || stroke_attrs_with(PREVIEW_STYLE, stroke_color.get()).0;
    let preview_style = move || stroke_attrs_with(PREVIEW_STYLE, stroke_color.get()).1;

    view! {
        <Show when=move || is_active.get()>
            <g class="dependency-preview">
                <path
                    d=move || path_d.get()
                    class="dependency-preview-line"
                    stroke=preview_stroke
                    stroke-width="2"
                    stroke-dasharray="5,5"
                    fill="none"
                    marker-end="url(#preview-arrowhead)"
                    style=preview_style
                />
            </g>
        </Show>
    }
}

/// Arrow marker for the dependency preview line
/// Should be included once in the SVG defs section
#[component]
pub fn DependencyPreviewMarker() -> impl IntoView {
    view! {
        <defs>
            <marker
                id="preview-arrowhead"
                markerWidth="10"
                markerHeight="10"
                refX="9"
                refY="3"
                orient="auto"
                markerUnits="strokeWidth"
            >
                <path
                    d="M0,0 L0,6 L9,3 z"
                    fill="currentColor"
                    style="opacity: 0.7;"
                />
            </marker>
        </defs>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dependency_preview_path() {
        // This test is primarily for documentation - actual rendering tests
        // would need Leptos test utilities

        // Path calculation is straightforward: M start_x start_y L current_x current_y
        let start_x = 100.0;
        let start_y = 50.0;
        let current_x = 200.0;
        let current_y = 150.0;

        let expected = format!("M {} {} L {} {}", start_x, start_y, current_x, current_y);
        assert_eq!(expected, "M 100 50 L 200 150");
    }
}
