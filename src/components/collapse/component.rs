use super::style::{CollapseForceModifier, CollapseModifier};
use crate::merge_classes;
use leptos::{
    html::{Div, Input},
    prelude::*,
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_COLLAPSE_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique id for one `Collapse` toggle (`ld-collapse-0`, ...),
/// minted only when the caller supplies no `id`. Monotonic counter, not
/// randomness: stable within a page's lifetime, which is all
/// `aria-labelledby` needs.
pub(super) fn next_collapse_id() -> String {
    format!(
        "ld-collapse-{}",
        NEXT_COLLAPSE_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Resolved identity of a `Collapse`'s internal toggle input (ldui-3k00).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapseIdentity {
    /// DOM `id` of the toggle input: the caller's `id`, else a minted one.
    pub id: String,
    /// Form `name` of the toggle input: the caller's `name`, else the id.
    pub name: String,
    /// Id carried by this collapse's `CollapseTitle`, which names the toggle
    /// via `aria-labelledby` unless an explicit `aria_label` is supplied.
    pub title_id: String,
}

/// Resolve the toggle's `id`/`name`/title id from the caller's optional
/// props, minting an id with `mint` only when none was supplied. Mirrors
/// `Checkbox`: an explicit `id` wins, and becomes the `name` when no `name`
/// is given (a form key is the server's vocabulary, so an explicit `name`
/// is passed through verbatim).
pub fn resolve_collapse_identity(
    id: Option<String>,
    name: Option<String>,
    mint: impl FnOnce() -> String,
) -> CollapseIdentity {
    let id = id.unwrap_or_else(mint);
    let name = name.unwrap_or_else(|| id.clone());
    let title_id = format!("{id}-title");
    CollapseIdentity { id, name, title_id }
}

/// The accessible-name attributes a `Collapse` toggle carries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CollapseInputNaming {
    /// `aria-label`, when the caller supplied one.
    pub aria_label: Option<String>,
    /// `aria-labelledby`, pointing at the `CollapseTitle`, otherwise.
    pub labelled_by: Option<String>,
}

/// Choose how the toggle is named. An explicit `aria_label` names it
/// directly and suppresses the `aria-labelledby` reference; otherwise the
/// visible `CollapseTitle` names it by id. Exactly one of the two is ever
/// emitted, because `aria-labelledby` wins over `aria-label` in the
/// accessible-name computation and would silently discard an explicit label
/// if both were present.
pub fn collapse_input_naming(aria_label: Option<String>, title_id: &str) -> CollapseInputNaming {
    match aria_label {
        Some(label) => CollapseInputNaming {
            aria_label: Some(label),
            labelled_by: None,
        },
        None => CollapseInputNaming {
            aria_label: None,
            labelled_by: Some(title_id.to_string()),
        },
    }
}

/// Provided by [`Collapse`] to its children. [`CollapseTitle`] consumes it
/// to carry the id that the toggle's `aria-labelledby` points at, so the
/// visible title is the toggle's accessible name without the consumer
/// wiring anything.
#[derive(Clone, Debug)]
pub struct CollapseContext {
    /// Id the title element must carry.
    pub title_id: String,
}

/// A collapsible container that can expand and contract to show or hide content.
///
/// The `Collapse` component uses tabindex-based interaction, allowing users to click
/// on the collapse to toggle its state. For more control, use `CollapseCheckbox`.
///
/// ## Toggle identity and accessible name (ldui-3k00)
///
/// In checkbox mode (the default, `focus_open=false`) the toggle is a real
/// `<input type="checkbox">` stretched over the title. It always carries an
/// `id` (yours, or a minted `ld-collapse-N`), a `name` (yours, or the id),
/// and an accessible name: `aria-labelledby` pointing at this collapse's
/// [`CollapseTitle`] by default, or `aria-label` when you pass `aria_label`
/// (which then suppresses the `aria-labelledby`). A consumer therefore never
/// ships an unidentified, unnamed form control, and a form audit can tell
/// one collapse's toggle from another's.
///
/// ```rust,ignore
/// <Collapse id="filters" name="show_filters">
///     <CollapseTitle>"Filters"</CollapseTitle>   // names the toggle
///     <CollapseContent>...</CollapseContent>
/// </Collapse>
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("collapse collapse-title collapse-content collapse-arrow collapse-plus collapse-open collapse-close");
/// ```
///
/// ## Node References
/// - `outer_node_ref` - References the outer `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
/// - `inner_node_ref` - References the inner `<input>` element ([HTMLInputElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement))
#[component]
pub fn Collapse(
    /// Whether to focus open or not
    #[prop(optional, into)]
    focus_open: Signal<bool>,

    /// Reactive signal controlling whether the collapse is checked for open
    #[prop(optional, into)]
    checked: Signal<bool>,

    /// Visual style and behavior modifier for the collapse
    #[prop(optional, into)]
    modifier: Signal<CollapseModifier>,

    /// Reactive signal controlling whether the collapse is open/close
    #[prop(optional, into)]
    force: Signal<CollapseForceModifier>,

    /// Stable DOM `id` for the toggle input. Read once when the component is
    /// created; when omitted, a process-unique `ld-collapse-N` is minted.
    /// Becomes the `name` when no `name` is given, and seeds the title's id
    /// (`<id>-title`).
    #[prop(optional, into)]
    id: MaybeProp<String>,

    /// Form `name` for the toggle input, passed through verbatim. Read once
    /// when the component is created.
    #[prop(optional, into)]
    name: MaybeProp<String>,

    /// Explicit accessible name for the toggle. Reactive. When supplied it
    /// replaces the default `aria-labelledby` reference to the
    /// [`CollapseTitle`]; reach for it only when the title is not the right
    /// name for the control (an icon-only title, or a title that reads
    /// wrongly out of context).
    #[prop(optional, into)]
    aria_label: MaybeProp<String>,

    /// Additional CSS classes to apply to the collapse container
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer container `<div>` element
    #[prop(optional)]
    outer_node_ref: NodeRef<Div>,

    /// Node reference for the internal radio `<input>` element
    /// If focus_open is false, mount.
    #[prop(optional)]
    inner_node_ref: NodeRef<Input>,

    /// Child elements, typically CollapseTitle and CollapseContent
    children: Children,
) -> impl IntoView {
    // Identity is structural: read once, like `Checkbox`'s `id`/`name`.
    let identity =
        resolve_collapse_identity(id.get_untracked(), name.get_untracked(), next_collapse_id);
    let title_id = identity.title_id.clone();
    provide_context(CollapseContext {
        title_id: title_id.clone(),
    });

    view! {
        <div
            node_ref=outer_node_ref
            tabindex=move || { if focus_open.get() { Some("0") } else { None } }
            class=move || {
                merge_classes!(
                    "collapse",
                    modifier.get().as_str(),
                    force.get().as_str(),
                    class
                )
            }
        >
            {move || {
                if focus_open.get() {
                    ().into_any()
                } else {
                    let input_id = identity.id.clone();
                    let input_name = identity.name.clone();
                    let label_title_id = title_id.clone();
                    let labelled_by_title_id = title_id.clone();
                    view! {
                        <input
                            node_ref=inner_node_ref
                            type="checkbox"
                            checked=checked
                            id=input_id
                            name=input_name
                            aria-label=move || {
                                collapse_input_naming(aria_label.get(), &label_title_id).aria_label
                            }
                            aria-labelledby=move || {
                                collapse_input_naming(aria_label.get(), &labelled_by_title_id)
                                    .labelled_by
                            }
                        />
                    }
                        .into_any()
                }
            }}

            {children()}
        </div>
    }
}

/// The clickable title section of a collapse component.
///
/// This component renders the header/title area that users click to toggle
/// the collapse state. Inside a [`Collapse`] it carries the id the toggle's
/// `aria-labelledby` points at (see [`CollapseContext`]), so the visible
/// title is the toggle's accessible name.
///
/// ## Node References
/// - `node_ref` - References the top `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn CollapseTitle(
    /// Additional CSS classes to apply to the title element
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the title `<div>` element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Title content (text, icons, etc.)
    children: Children,
) -> impl IntoView {
    let title_id = use_context::<CollapseContext>().map(|context| context.title_id);
    view! {
        <div node_ref=node_ref id=title_id class=move || merge_classes!("collapse-title", class)>
            {children()}
        </div>
    }
}

/// The collapsible content section of a collapse component.
///
/// This component renders the content that is shown/hidden when the collapse
/// is toggled.
///
/// ## Node References
/// - `node_ref` - References the top `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn CollapseContent(
    /// Additional CSS classes to apply to the content element
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the content `<div>` element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Content to show/hide when collapse is toggled
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("collapse-content", class)>
            {children()}
        </div>
    }
}
