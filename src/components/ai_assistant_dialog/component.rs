use crate::components::button::{Button, ButtonShape, ButtonSize, ButtonStyle};
use crate::components::filter_sidebar::SidebarSide;
use crate::components::icon::IconSize;
use crate::components::modal::{Modal, ModalCloseCause, ModalCloseProposal};
use leptos::{html::Div, prelude::*};
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-instance sequence for the title `id` the dialog is labelled by, so two
/// mounted dialogs never share one.
static AI_ASSISTANT_DIALOG_SEQ: AtomicU64 = AtomicU64::new(0);

/// The `<dialog>`'s placement: vertically centred and pushed to the
/// viewport's END (right), where a side rail used to sit. Taken from
/// 4iiz-Office's `ASSISTANT_DIALOG_CLASS` (op-zm1ow.1), the first floating
/// assistant, so a host migrating to this component keeps its geometry.
pub const AI_ASSISTANT_DIALOG_CLASS: &str = "modal-middle justify-items-end";

/// The box at its NORMAL size: the side rail's feel -- 28rem wide (never
/// under 24rem, never wider than the viewport minus a gutter), 85vh tall,
/// a column whose body scrolls while the header stays, a small gap to the
/// right edge from `sm`. Office's `ASSISTANT_DIALOG_BOX_CLASS`, unchanged.
pub const AI_ASSISTANT_DIALOG_NORMAL_CLASS: &str =
    "h-[85vh] max-h-[85vh] w-[28rem] min-w-[24rem] max-w-[calc(100vw-2rem)] sm:mr-4";

/// The box MAXIMIZED (ldui-kq3w): 90% of the window in both directions,
/// centred. `max-w-[90vw]` because daisyUI's `.modal-box` caps itself at
/// 32rem and a `w-` alone would be clamped back to it; `justify-self-center`
/// overrides the dialog's `justify-items-end` for this one box.
pub const AI_ASSISTANT_DIALOG_MAXIMIZED_CLASS: &str =
    "h-[90vh] max-h-[90vh] w-[90vw] min-w-[24rem] max-w-[90vw] justify-self-center";

/// Classes shared by both sizes. Width and height transition so the toggle
/// grows the box rather than jumping.
const AI_ASSISTANT_DIALOG_BOX_BASE: &str = "modal-box flex flex-col gap-2 overflow-hidden p-3 \
     transition-[width,max-width,height,max-height] duration-[250ms] ease-out";

/// Every string the dialog renders itself. The title arrives separately
/// because it is the host's (an engine or use-case name), not chrome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiAssistantDialogTexts {
    /// Accessible name and tooltip of the `<<` / `>>` toggle. ONE label:
    /// the button reports its state through `aria-pressed`, and a label that
    /// flipped between "Expand" and "Restore" would contradict it.
    pub maximize_label: String,
    /// Accessible name of the close button.
    pub close_label: String,
}

impl Default for AiAssistantDialogTexts {
    fn default() -> Self {
        Self {
            maximize_label: "Expand the assistant".to_string(),
            close_label: "Close the assistant".to_string(),
        }
    }
}

impl AiAssistantDialogTexts {
    /// The Spanish table, so a bilingual host need not translate chrome it
    /// did not write.
    pub fn spanish() -> Self {
        Self {
            maximize_label: "Ampliar el asistente".to_string(),
            close_label: "Cerrar el asistente".to_string(),
        }
    }
}

/// The box's classes for one state.
pub fn ai_assistant_dialog_box_class(maximized: bool) -> String {
    format!(
        "{AI_ASSISTANT_DIALOG_BOX_BASE} {}",
        if maximized {
            AI_ASSISTANT_DIALOG_MAXIMIZED_CLASS
        } else {
            AI_ASSISTANT_DIALOG_NORMAL_CLASS
        }
    )
}

/// The toggle's glyph. The dialog docks against the RIGHT edge, so at its
/// normal size it grows leftwards (`<<`) and maximized it shrinks back
/// (`>>`) -- the same rule, and the same mapping, as a right-docked
/// `FilterSidebar`'s maximize toggle.
pub fn ai_assistant_dialog_maximize_glyph(maximized: bool) -> &'static str {
    SidebarSide::Right.maximize_glyph(maximized)
}

/// # AiAssistantDialog
///
/// The STANDARD AI assistant frame (owner, 2026-11-13: *"We will be using the
/// floating dialog in all applications as the standard going forward"*): a
/// floating modal docked to the right of the viewport, with a title row, a
/// host-owned slot for panel controls (a settings gear, New session), a
/// `<<` / `>>` toggle that widens the dialog to 90% of the window and back,
/// and a close button. The body -- typically an `AiChatWorkspace` with
/// `layout=AiChatWorkspaceLayout::Rail` -- is the host's.
///
/// Promoted from 4iiz-Office's `AssistantDialog` (op-zm1ow.1), whose
/// geometry it keeps, so every app frames its assistant the same way.
///
/// ## Open and close
///
/// CONTROLLED, through `Modal`'s controlled mode: the host owns `open`;
/// Escape, the backdrop and the close button each PROPOSE a close through
/// `on_close_request`, and the host accepts by setting `open` to `false`.
/// The body stays mounted while closed, so a conversation, its scroll and a
/// half-typed question survive a close and reopen. Focus returns to the
/// launcher through the native `<dialog>`'s own restoration.
///
/// ## Maximize
///
/// `maximized` is the host's `RwSignal<bool>` when it wants to persist or
/// read the size; omitted, the dialog keeps its own. The size survives a
/// close: reopening shows the assistant the way the person left it.
///
/// ## DOM hooks
///
/// `data-ai-assistant-dialog` on the box (`"open"`/`"closed"`),
/// `data-ai-assistant-dialog-maximized` (`"true"`/`"false"`),
/// `data-ai-assistant-dialog-maximize` on the toggle,
/// `data-ai-assistant-dialog-close` on the close button and
/// `data-ai-assistant-dialog-body` on the scrolling body.
///
/// ## CSS
/// ```css
/// @source inline("modal-middle justify-items-end justify-self-center sm:mr-4");
/// @source inline("h-[85vh] max-h-[85vh] w-[28rem] min-w-[24rem] max-w-[calc(100vw-2rem)]");
/// @source inline("h-[90vh] max-h-[90vh] w-[90vw] max-w-[90vw]");
/// @source inline("transition-[width,max-width,height,max-height] duration-[250ms] ease-out");
/// ```
///
/// ```rust,ignore
/// let open = RwSignal::new(false);
/// view! {
///     <AiAssistantDialog
///         open=open
///         on_close_request=Callback::new(move |_| open.set(false))
///         title="Assistant"
///         header_actions=Box::new(|| view! { <SettingsGear /> }.into_any())
///     >
///         <AiChatWorkspace backend=backend layout=AiChatWorkspaceLayout::Rail />
///     </AiAssistantDialog>
/// }
/// ```
///
/// ## Node References
/// - `node_ref` - References the box `<div>`
///   ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement)).
#[component]
pub fn AiAssistantDialog(
    /// Whether the dialog is open. Owned by the host.
    #[prop(into)]
    open: Signal<bool>,
    /// A close was proposed (Escape, the backdrop, or the close button).
    /// Accept it by setting `open` to `false`.
    #[prop(into)]
    on_close_request: Callback<ModalCloseProposal>,
    /// The title row's heading, and the dialog's accessible name.
    #[prop(into)]
    title: Signal<String>,
    /// The host's size signal. Omit it and the dialog keeps its own.
    #[prop(optional)]
    maximized: Option<RwSignal<bool>>,
    /// Panel-scoped controls rendered between the title and the `<<` / `>>`
    /// toggle (a settings gear, a New session button). `None` renders no
    /// wrapper at all.
    #[prop(optional)]
    header_actions: Option<Children>,
    /// The chrome's own strings.
    #[prop(optional, into, default = Signal::stored(AiAssistantDialogTexts::default()))]
    texts: Signal<AiAssistantDialogTexts>,
    /// Whether clicking outside the box proposes a close. Defaults to `true`.
    #[prop(optional, into, default = Signal::stored(true))]
    backdrop: Signal<bool>,
    /// The box's DOM `id`, for a launcher's `aria-controls`.
    #[prop(optional, into)]
    panel_id: Option<String>,
    /// Reference to the box `<div>`.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
    /// The assistant itself.
    children: Children,
) -> impl IntoView {
    let maximized = maximized.unwrap_or_else(|| RwSignal::new(false));
    let title_id = format!(
        "ai-assistant-dialog-title-{}",
        AI_ASSISTANT_DIALOG_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let labelled_by = title_id.clone();
    let header_actions = header_actions.map(|actions| {
        view! {
            <div class="flex shrink-0 items-center gap-1" data-ai-assistant-dialog-actions="">
                {actions()}
            </div>
        }
    });

    view! {
        <Modal
            open=open
            backdrop=backdrop
            on_close_request=on_close_request
            labelled_by=labelled_by
            class=AI_ASSISTANT_DIALOG_CLASS
        >
            <div
                node_ref=node_ref
                id=panel_id
                class=move || ai_assistant_dialog_box_class(maximized.get())
                data-ai-assistant-dialog=move || if open.get() { "open" } else { "closed" }
                data-ai-assistant-dialog-maximized=move || maximized.get().to_string()
            >
                <header class="flex shrink-0 items-center gap-1">
                    <h2 id=title_id class="min-w-0 flex-1 truncate text-base font-semibold">
                        {move || title.get()}
                    </h2>
                    {header_actions}
                    <Button
                        shape=ButtonShape::Square
                        style=ButtonStyle::Ghost
                        size=ButtonSize::Xs
                        class="font-mono text-[12px] font-bold"
                        on_click=Callback::new(move |_| maximized.update(|m| *m = !*m))
                        attr:aria-label=move || texts.with(|t| t.maximize_label.clone())
                        attr:title=move || texts.with(|t| t.maximize_label.clone())
                        attr:aria-pressed=move || maximized.get().to_string()
                        attr:data-ai-assistant-dialog-maximize="true"
                    >
                        <span aria-hidden="true">
                            {move || ai_assistant_dialog_maximize_glyph(maximized.get())}
                        </span>
                    </Button>
                    <Button
                        shape=ButtonShape::Square
                        style=ButtonStyle::Ghost
                        size=ButtonSize::Xs
                        // Proposed as `DialogForm`, the cause daisyUI's own
                        // in-content close button carries, so a host's
                        // existing proposal handling needs no new arm.
                        on_click=Callback::new(move |_| {
                            on_close_request.run(ModalCloseProposal::new(ModalCloseCause::DialogForm));
                        })
                        attr:aria-label=move || texts.with(|t| t.close_label.clone())
                        attr:data-ai-assistant-dialog-close="true"
                    >
                        // Inline, not `Icon`: the sprite is host-owned and the
                        // showcase's has no `close` symbol, so a sprite X drew
                        // an empty button (found in the browser, ldui-kq3w).
                        <svg
                            class=IconSize::XSmall.as_str()
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                            focusable="false"
                        >
                            <path d="M18 6 6 18" />
                            <path d="m6 6 12 12" />
                        </svg>
                    </Button>
                </header>
                <div
                    class="flex min-h-0 flex-1 flex-col overflow-y-auto"
                    data-ai-assistant-dialog-body="true"
                >
                    {children()}
                </div>
            </div>
        </Modal>
    }
}
