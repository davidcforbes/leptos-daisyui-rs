use crate::merge_classes;
use leptos::{
    html::{Dialog, Div, Form},
    prelude::*,
};

/// # Modal Component
///
/// A reactive Leptos wrapper for daisyUI's modal component that provides
/// overlay dialogs using native HTML dialog elements with proper state management.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("modal modal-backdrop modal-box modal-action modal-toggle modal-open modal-top modal-middle modal-bottom");
/// ```
///
/// ## Node References
/// - `node_ref` - References the dialog element ([HTMLDialogElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement))
#[component]
pub fn Modal(
    /// Signal controlling modal open state
    #[prop(optional, into)]
    open: Signal<bool>,

    /// Whether to include backdrop for click-to-close
    #[prop(optional, into)]
    backdrop: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the dialog element
    #[prop(optional)]
    node_ref: NodeRef<Dialog>,

    /// Modal content
    children: Children,
) -> impl IntoView {
    Effect::new(move || {
        let Some(node) = node_ref.get() else { return };

        if open.get() {
            let _ = node.show_modal();
        } else {
            node.close();
        }
    });

    view! {
        <dialog
            aria_modal=move || open.get()
            aria-label="Modal"
            node_ref=node_ref
            class=move || merge_classes!("modal", class)
            class:modal-open=open
        >
            {children()}
            {move || {
                if backdrop.get() { view! { <ModalBackdrop /> }.into_any() } else { ().into_any() }
            }}
        </dialog>
    }
}

/// Content container for modal dialogs.
///
/// Provides styled container for modal content with proper spacing, background,
/// and responsive design. Should be used inside a Modal component.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ModalBox(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Modal content
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("modal-box", class)>
            {children()}
        </div>
    }
}

/// Action button container for modal dialogs.
///
/// Provides a styled container for action buttons, typically placed at the bottom
/// of a modal. Handles proper spacing and alignment for button groups.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ModalAction(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,
    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
    /// Action buttons
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("modal-action", class)>
            {children()}
        </div>
    }
}

/// # ModalBackdrop component
///
/// For modal click-to-close functionality.
///
/// ## Node References
/// - `node_ref` - References the top form element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement))
#[component]
pub fn ModalBackdrop(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the form element
    #[prop(optional)]
    node_ref: NodeRef<Form>,
) -> impl IntoView {
    view! {
        <form
            node_ref=node_ref
            method="dialog"
            class=move || merge_classes!("modal-backdrop", class)
        >
            <button>close</button>
        </form>
    }
}

/// # ModalInfoRow component
///
/// A horizontal `label: value` row for the find-and-restore dialog
/// recipe (title → info → search → status → body → actions). Use one or
/// more inside a [`ModalBox`] above the search row to surface read-only
/// metadata like a source path, snapshot id, or counts.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ModalInfoRow(
    /// Optional bold label rendered before the children.
    #[prop(optional, into)]
    label: &'static str,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Value content
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!(
                "flex items-center gap-2 py-2 border-b border-base-200",
                class
            )
        >
            {(!label.is_empty()).then(|| view! { <span class="font-semibold">{label}</span> })}
            {children()}
        </div>
    }
}

/// # ModalSearchRow component
///
/// A flex row for the search input in find-and-restore dialogs. Children
/// typically include an `<Input>` and an optional action button. The
/// wrapper supplies consistent spacing and a daisyUI-themed divider
/// below.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ModalSearchRow(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Row content
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!(
                "flex items-center gap-2 py-2 border-b border-base-200",
                class
            )
        >
            {children()}
        </div>
    }
}

/// # ModalStatusRow component
///
/// A subdued row for inline status text — match counts, last-refreshed
/// timestamps, error banners — that sits between the search row and the
/// dialog body in the find-and-restore recipe.
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ModalStatusRow(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Row content
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!(
                "flex items-center gap-2 py-2 text-sm text-base-content/70",
                class
            )
        >
            {children()}
        </div>
    }
}
