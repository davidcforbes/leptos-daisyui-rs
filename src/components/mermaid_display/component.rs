use super::style::MermaidTheme;
use crate::components::SvgDisplay;
use crate::pele::{self, PeleRenderError, PeleSvg, escape_html};
use leptos::{html::Figure, prelude::*};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

/// Where a [`MermaidDisplay`] is in its render, exposed on the figure as
/// `data-mermaid-state` for hosts and browser tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MermaidRenderState {
    /// No source (blank or whitespace): nothing is drawn.
    #[default]
    Empty,
    /// The diagram is being drawn in pele's worker; the source is shown in a
    /// muted `pre`.
    Pending,
    /// The diagram is drawn (`svg.pele` inside the figure).
    Rendered,
    /// pele rejected the source, the worker could not start, or the diagram
    /// overran its time limit; the `mermaid-error` box carries the message
    /// and its `data-pele-error` code.
    Error,
}

impl MermaidRenderState {
    /// Value of the figure's `data-mermaid-state` attribute.
    pub fn as_str(&self) -> &'static str {
        match self {
            MermaidRenderState::Empty => "empty",
            MermaidRenderState::Pending => "pending",
            MermaidRenderState::Rendered => "rendered",
            MermaidRenderState::Error => "error",
        }
    }
}

/// Decode the four HTML entities a host may have left in diagram source
/// (`&lt;`, `&gt;`, `&amp;`, `&quot;`), in the order the component always
/// has, so `&amp;lt;` becomes `&lt;` rather than `<`.
pub(crate) fn decode_entities(src: &str) -> String {
    src.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
}

/// The neutral pending state: the diagram source, escaped, in a muted `pre`.
pub(crate) fn pending_html(src: &str) -> String {
    format!(
        "<pre class=\"mermaid-pending text-sm text-base-content/75 whitespace-pre-wrap\" aria-busy=\"true\">{}</pre>",
        escape_html(src)
    )
}

/// Map a render outcome to the figure's content, state, and the count of
/// elements and attributes the sanitiser removed (`None` without an SVG).
pub(crate) fn outcome_html(
    result: Result<PeleSvg, PeleRenderError>,
) -> (String, MermaidRenderState, Option<usize>) {
    match result {
        Ok(drawn) => (
            drawn.svg,
            MermaidRenderState::Rendered,
            Some(drawn.stripped),
        ),
        Err(err) => (err.to_html(), MermaidRenderState::Error, None),
    }
}

/// # Mermaid Diagram Display Component
///
/// A reactive Leptos component that renders Mermaid diagram source as inline
/// SVG with [pele](https://github.com/obsidianmd/pele) (pinned in
/// [`crate::pele::PELE_VERSION`]), in the browser. Composes `SvgDisplay`.
///
/// pele draws all 33 Mermaid 12 diagram types and reads frontmatter and
/// `%%{init}%%` directives itself. HTML entities (`&lt;`, `&gt;`, `&amp;`,
/// `&quot;`) in `source` are decoded first.
///
/// ## Rendering is asynchronous, off the UI thread
///
/// pele runs in a same-origin Web Worker that the host serves from this
/// crate's `assets/pele/` (see [`crate::pele`];
/// [`crate::pele::set_pele_asset_base`] moves it). While a diagram is
/// drawn the figure shows the source in a muted `pre`
/// (`data-mermaid-state="pending"`); then the sanitised SVG (`"rendered"`,
/// with `data-pele-stripped` counting what the sanitiser removed). A
/// diagram pele rejects, a worker that cannot start, or a diagram that
/// overruns [`crate::pele::PELE_RENDER_TIMEOUT_MS`] shows the message,
/// escaped, in the `mermaid-error` box (`"error"`); a later source change
/// tries again.
///
/// ## Colours follow the daisyUI theme
///
/// pele takes every colour from `--pele-*` CSS variables, which
/// `styles/tokens.css` maps onto the daisyUI 5 theme variables, so a
/// `data-theme` switch recolours a drawn diagram with no re-render. The
/// `theme` prop is kept for API compatibility and no longer recolours: it is
/// exposed as `data-mermaid-theme` on the figure for host CSS that wants it.
///
/// ### Add to `input.css`
/// ```css
/// @import "../leptos-daisyui-rs/styles/tokens.css"; /* the .pele colour mapping */
/// @source inline("mermaid-pending mermaid-error text-sm text-base-content/75 whitespace-pre-wrap");
/// ```
///
/// ## Node References
/// - `node_ref` - References the `<figure>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn MermaidDisplay(
    /// Mermaid diagram source text
    #[prop(into)]
    source: Signal<String>,

    /// Kept for API compatibility. Colours now come from the daisyUI theme
    /// via `--pele-*` CSS variables; this value is only reflected as the
    /// figure's `data-mermaid-theme` attribute.
    #[prop(optional, into)]
    theme: Signal<MermaidTheme>,

    /// Maximum width in pixels (CSS max-width)
    #[prop(optional, into)]
    max_width: Signal<Option<f32>>,

    /// Maximum height in pixels (CSS max-height)
    #[prop(optional, into)]
    max_height: Signal<Option<f32>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the figure element
    #[prop(optional)]
    node_ref: NodeRef<Figure>,
) -> impl IntoView {
    let content = RwSignal::new(String::new());
    let state = RwSignal::new(MermaidRenderState::Empty);
    let stripped = RwSignal::new(None::<usize>);
    // Last request wins: a load that finishes after a newer source (or after
    // unmount, when the stored value is gone) is dropped.
    let generation = StoredValue::new(0u64);

    Effect::new(move |_| {
        let src = decode_entities(&source.get());
        let current = generation.get_value().wrapping_add(1);
        generation.set_value(current);

        stripped.set(None);
        if src.trim().is_empty() {
            content.set(String::new());
            state.set(MermaidRenderState::Empty);
            return;
        }

        // pele measures labels in the figure's font (see pele_options_for).
        let container = node_ref
            .get_untracked()
            .map(|figure| figure.unchecked_into::<web_sys::Element>());

        content.set(pending_html(&src));
        state.set(MermaidRenderState::Pending);
        spawn_local(async move {
            let result = pele::render_when_ready(src, container).await;
            if generation.try_get_value() != Some(current) {
                return;
            }
            let (html, next, removed) = outcome_html(result);
            let _ = content.try_set(html);
            let _ = state.try_set(next);
            let _ = stripped.try_set(removed);
        });
    });

    view! {
        <SvgDisplay
            content=Signal::derive(move || content.get())
            alt="Mermaid diagram"
            max_width=max_width
            max_height=max_height
            class=class
            node_ref=node_ref
            attr:data-mermaid-state=move || state.get().as_str()
            attr:data-mermaid-theme=move || theme.get().as_str()
            attr:data-pele-stripped=move || stripped.get().map(|n| n.to_string())
        />
    }
}
