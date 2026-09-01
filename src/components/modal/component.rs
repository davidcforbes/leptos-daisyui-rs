use crate::merge_classes;
use leptos::{
    html::{Dialog, Div, Form},
    prelude::*,
};
use wasm_bindgen::JsCast;

/// What the user did to ask a controlled [`Modal`] to close (`ldui-e0fw`).
///
/// A bare unit callback would tell the owner *that* a close was requested
/// but not *how*, and the two differ for the caller: Escape is a dismissal
/// that should usually discard scoped feedback, a backdrop activation is a
/// pointer dismissal of the same weight, and an in-content
/// `<form method="dialog">` submit is a deliberate confirm-style action
/// whose form values the caller may still want to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalCloseCause {
    /// The user pressed Escape. This is the platform's own "close request",
    /// which arrives as the dialog's cancelable `cancel` event.
    Escape,
    /// The user activated [`ModalBackdrop`] — the `method="dialog"` form
    /// covering the area outside the modal box.
    Backdrop,
    /// A `<form method="dialog">` inside the modal *content* was submitted,
    /// e.g. daisyUI's documented close button. This closes a native dialog
    /// just as silently as Escape does, so it is proposed too rather than
    /// left as a second undetected drift path.
    DialogForm,
}

impl ModalCloseCause {
    /// Stable slug for tests, telemetry, and consumer state machines.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Escape => "escape",
            Self::Backdrop => "backdrop",
            Self::DialogForm => "dialog-form",
        }
    }
}

/// One user-proposed close of a controlled [`Modal`].
///
/// Nothing is applied when this is emitted. The dialog is still open, and it
/// stays open until the caller's own `open` signal says otherwise — so a
/// proposal the caller ignores or rejects leaves zero drift between the
/// accepted state and the DOM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModalCloseProposal {
    /// The gesture that produced this proposal.
    pub cause: ModalCloseCause,
}

impl ModalCloseProposal {
    /// Builds a proposal for `cause`.
    pub fn new(cause: ModalCloseCause) -> Self {
        Self { cause }
    }
}

/// Localized copy owned by the modal chrome rather than by the caller's
/// children.
///
/// Only [`ModalBackdrop`] renders framework-owned visible text — daisyUI's
/// documented backdrop markup is a `method="dialog"` form wrapping a button
/// whose label is also that button's accessible name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalTexts {
    /// Label and accessible name of the backdrop's close control. Defaults
    /// to daisyUI's own lowercase `close`.
    pub backdrop_close: String,
}

impl Default for ModalTexts {
    fn default() -> Self {
        Self {
            backdrop_close: "close".to_owned(),
        }
    }
}

/// Whether a `submit` bubbling out of a [`Modal`] is a dialog-closing
/// submission, and which gesture it was.
///
/// Returns `None` for any other form — a real `method="post"` search or
/// login form inside the modal must submit normally and must never be
/// mistaken for a close.
pub fn modal_submit_close_cause(form_method: &str, is_backdrop: bool) -> Option<ModalCloseCause> {
    if !form_method.eq_ignore_ascii_case("dialog") {
        return None;
    }
    Some(if is_backdrop {
        ModalCloseCause::Backdrop
    } else {
        ModalCloseCause::DialogForm
    })
}

/// Whether a fired `close` event means the DOM has drifted away from the
/// caller's accepted state, and the dialog must be restored.
///
/// In controlled mode every user close is vetoed and re-proposed, so the
/// only `close` that should ever reach a controlled dialog is the one the
/// component itself performs after the caller set `open` to `false`. A
/// `close` observed while the accepted target is still `true` is therefore
/// exactly the defect this component exists to prevent, and is repaired
/// rather than reported.
pub fn modal_close_is_state_drift(controlled: bool, open_target: bool) -> bool {
    controlled && open_target
}

/// Value of the dialog's `data-modal-close-mode` marker. Consumers and
/// browser tests read it to assert which contract a given dialog is under
/// without inferring it from behaviour.
pub fn modal_close_mode_attr(controlled: bool) -> &'static str {
    if controlled {
        "controlled"
    } else {
        "uncontrolled"
    }
}

/// The `aria-label` a [`Modal`] should carry: `None` when a visible heading
/// names the dialog via `labelled_by` (an `aria-label` would override it),
/// else the caller's `label`, else the legacy `"Modal"` fallback — an
/// unnamed dialog is an axe violation, so the generic name is still better
/// than none for callers that haven't adopted the naming props yet.
pub fn modal_aria_label(label: Option<String>, has_labelled_by: bool) -> Option<String> {
    if has_labelled_by {
        None
    } else {
        Some(label.unwrap_or_else(|| "Modal".to_string()))
    }
}

/// # Modal Component
///
/// A reactive Leptos wrapper for daisyUI's modal component that provides
/// overlay dialogs using native HTML dialog elements with proper state management.
///
/// Name the dialog: pass `labelled_by` (the id of the visible heading inside
/// [`ModalBox`]) or `label` (a translated accessible name). Without either,
/// every dialog is announced with the same generic word.
///
/// ## Controlled close (`ldui-e0fw`)
///
/// A native `<dialog>` can close itself. Escape fires a cancelable `cancel`
/// event and then closes; a `method="dialog"` form submit — which is what
/// [`ModalBackdrop`] is — closes with no `cancel` at all. Both leave a
/// caller's `open` signal reading `true` over a dialog that is shut, after
/// which a `true`-to-`true` change cannot reopen it and scoped feedback is
/// never cleared.
///
/// Supplying `on_close_request` switches the dialog into **controlled**
/// mode, where the caller's signal is the only thing that ever closes it:
///
/// - Escape is vetoed (`cancel` is `preventDefault`ed) and re-emitted as
///   [`ModalCloseCause::Escape`].
/// - A backdrop or in-content `method="dialog"` submit is vetoed the same
///   way and re-emitted as [`ModalCloseCause::Backdrop`] or
///   [`ModalCloseCause::DialogForm`]. Forms with any other method submit
///   untouched.
/// - Accepting a proposal means setting `open` to `false`; the component
///   then calls the dialog's own `close()` exactly once, which is what
///   preserves the platform's focus return to the trigger.
/// - Ignoring or rejecting a proposal leaves the dialog open and the
///   accepted state untouched — nothing optimistic was written.
/// - A programmatic `open` change to `false` emits no proposal; proposals
///   only ever originate from a user gesture.
///
/// Without `on_close_request` nothing changes: Escape and the backdrop close
/// natively and existing `on:close` call sites behave exactly as before.
///
/// ```rust,ignore
/// let (open, set_open) = signal(false);
/// view! {
///     <Modal
///         open=open
///         backdrop=true
///         labelled_by="reassign-title"
///         on_close_request=Callback::new(move |proposal: ModalCloseProposal| {
///             set_feedback.set(None);
///             set_open.set(false);
///             log_dismissal(proposal.cause.as_str());
///         })
///     >
///         <ModalBox>
///             <h3 id="reassign-title">"Reassign matter"</h3>
///         </ModalBox>
///     </Modal>
/// }
/// ```
///
/// ### Focus return
///
/// Trigger-focus restoration is the platform's, not this component's. A
/// modal opened with `show_modal()` records the previously focused element
/// and restores focus to it when `close()` runs. This component's job is to
/// make sure every close really does go through `close()` — never through a
/// removed or hidden dialog — which is precisely what controlled mode
/// guarantees. Owning focus here would mean fighting that machinery and
/// would break the common case where the trigger has been re-rendered; a
/// caller that wants focus somewhere else moves it from
/// `on_close_request`.
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

    /// Controlled-close proposal sink. Supplying it makes `open` the only
    /// thing that can close this dialog; see the component docs above.
    #[prop(optional, into)]
    on_close_request: Option<Callback<ModalCloseProposal>>,

    /// Localized copy for the modal chrome — currently only the backdrop's
    /// close control. A `Signal` so it can follow a runtime locale switch.
    #[prop(into, default = Signal::stored(ModalTexts::default()))]
    texts: Signal<ModalTexts>,

    /// Accessible name for the dialog (`aria-label`) — pass the dialog's
    /// (translated) purpose, e.g. `"Reassign matter"`. Prefer `labelled_by`
    /// when the dialog has a visible heading; without either, the legacy
    /// generic `"Modal"` is used so the dialog is at least not nameless.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Id of the element that names the dialog (`aria-labelledby`) —
    /// typically the visible `<h3>` inside [`ModalBox`]. Takes precedence
    /// over `label`, so assistive technology hears exactly what sighted
    /// users read, in whatever language the page is rendering.
    #[prop(optional, into)]
    labelled_by: MaybeProp<String>,

    /// Id of the element that describes the dialog (`aria-describedby`) —
    /// e.g. the summary paragraph under the heading.
    #[prop(optional, into)]
    described_by: MaybeProp<String>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the dialog element
    #[prop(optional)]
    node_ref: NodeRef<Dialog>,

    /// Modal content
    children: Children,
) -> impl IntoView {
    let controlled = on_close_request.is_some();

    Effect::new(move || {
        let Some(node) = node_ref.get() else { return };

        if open.get() {
            let _ = node.show_modal();
        } else {
            node.close();
        }
    });

    let propose = move |cause: ModalCloseCause| {
        if let Some(callback) = on_close_request {
            callback.run(ModalCloseProposal::new(cause));
        }
    };

    // Escape reaches a dialog as its own cancelable `cancel` event. A
    // `keydown` `preventDefault` does NOT suppress the native close; only
    // this does.
    let handle_cancel = move |event: web_sys::Event| {
        if !controlled {
            return;
        }
        event.prevent_default();
        propose(ModalCloseCause::Escape);
    };

    // A `method="dialog"` submit closes the dialog with no `cancel` event
    // whatsoever, which is why the backdrop was invisible to every
    // `cancel`-based workaround. `submit` bubbles, so one listener on the
    // dialog covers `ModalBackdrop` and any in-content dialog form.
    let handle_submit = move |event: web_sys::SubmitEvent| {
        if !controlled {
            return;
        }
        let Some(form) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlFormElement>().ok())
        else {
            return;
        };
        let is_backdrop = form.class_list().contains("modal-backdrop");
        let Some(cause) = modal_submit_close_cause(&form.method(), is_backdrop) else {
            return;
        };
        event.prevent_default();
        propose(cause);
    };

    // Repair, not report: a `close` seen while the accepted target is still
    // `true` is state drift, and re-showing keeps the DOM equal to what the
    // caller actually asked for. The close the component itself performs
    // after an accepted proposal never lands here, because by then the
    // target is `false`.
    let handle_close = move |_: web_sys::Event| {
        if !modal_close_is_state_drift(controlled, open.get_untracked()) {
            return;
        }
        if let Some(node) = node_ref.get_untracked() {
            let _ = node.show_modal();
        }
    };

    view! {
        <dialog
            aria-label=move || modal_aria_label(label.get(), labelled_by.get().is_some())
            aria-labelledby=move || labelled_by.get()
            aria-describedby=move || described_by.get()
            data-modal-close-mode=modal_close_mode_attr(controlled)
            node_ref=node_ref
            class=move || merge_classes!("modal", class)
            class:modal-open=open
            on:cancel=handle_cancel
            on:submit=handle_submit
            on:close=handle_close
        >
            {children()}
            {move || {
                if backdrop.get() {
                    view! { <ModalBackdrop texts=texts /> }.into_any()
                } else {
                    ().into_any()
                }
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
/// This is daisyUI's documented backdrop: a `method="dialog"` form covering
/// the area outside the modal box, so activating it submits the form and the
/// browser closes the dialog. That close fires `close` but **never**
/// `cancel`, which is why a backdrop dismissal used to be invisible to a
/// controlling owner. Inside a [`Modal`] with `on_close_request` the submit
/// is vetoed and re-emitted as [`ModalCloseCause::Backdrop`] instead.
///
/// The form carries `data-modal-backdrop="true"` as a stable hook, and the
/// `modal-backdrop` class is what distinguishes its submit from an
/// in-content dialog form's.
///
/// ## Node References
/// - `node_ref` - References the top form element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement))
#[component]
pub fn ModalBackdrop(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Localized copy for the close control this backdrop renders.
    #[prop(into, default = Signal::stored(ModalTexts::default()))]
    texts: Signal<ModalTexts>,

    /// Reference to the form element
    #[prop(optional)]
    node_ref: NodeRef<Form>,
) -> impl IntoView {
    view! {
        <form
            node_ref=node_ref
            method="dialog"
            data-modal-backdrop="true"
            class=move || merge_classes!("modal-backdrop", class)
        >
            <button>{move || texts.get().backdrop_close}</button>
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
