//! Composable typed search-picker dialog: a labelled search box plus a
//! keyboard-navigable, typed result list inside a focus-trapped modal.
//!
//! Built entirely from existing pieces rather than reinventing any of them:
//! [`Modal`](crate::components::Modal) owns the dialog element, focus trap,
//! and accessible naming; [`Field`](crate::components::Field) +
//! [`Input`](crate::components::Input) own the labelled search markup;
//! [`KeyedResultList`](crate::components::KeyedResultList) owns keyboard
//! result navigation and the stable-key activation guarantee (reordering,
//! duplicate-looking rows, relabeling, insertion, and removal all resolve
//! against the *current* items, never a captured stale value -- see
//! `doc/components/result_list.md`); [`PageStatePanel`](super::PageStatePanel)
//! owns loading/error/empty presentation. This module owns only the
//! composition: which of those pieces is mounted for a given
//! [`SearchPickerStatus`], focusing the search box on open, and forwarding
//! `ArrowUp`/`ArrowDown`/`Home`/`End`/`Enter` from the search box to the list
//! so a caller can navigate results without leaving the search field.
//!
//! The caller owns query execution, debounce policy, result payloads,
//! authorization, and activation side effects -- `query`, `status`, and
//! `items` are all controlled signals; this component never fetches
//! anything itself. Because activation is always resolved by
//! [`KeyedResultList`] against the *current* `items` for the *current*
//! selected key, a stale async response that never reaches `items` can never
//! be activated, and neither can a superseded key after `items` is replaced.
//!
//! # Two activation models, two components
//!
//! [`SearchPickerDialog`] treats activating a result as the *terminal*
//! action -- right for navigating, opening, or filtering, where the effect
//! is reversible. [`ConfirmableSearchPickerDialog`] (`ldui-iq0o`) splits
//! that in two: activating a result only moves the caller's selected key,
//! and a separate explicit Confirm control runs the mutation. Reach for it
//! whenever confirming has a side effect -- assign, reassign, link-record,
//! restore, choose-owner.
//!
//! The split is at the type level, not behind a flag: the confirmable
//! component passes no `on_select` to its [`KeyedResultList`] at all and
//! exposes `on_confirm` instead, so no configuration of either component
//! can turn result activation into a write. See
//! `doc/patterns/confirmable-search-picker-dialog.md`.

use super::{PageStatePanel, PageStatePanelKind, PageStatePanelTexts};
use crate::components::{
    Button, ButtonColor, ButtonStyle, Field, Input, InputType, KeyedResultList,
    KeyedResultListSelection, KeyedResultListSelectionProposal, Modal, ModalAction, ModalBox,
    ModalCloseCause, ModalCloseProposal, ModalSearchRow, ResultListItem,
};
use leptos::{
    html::{Dialog, Div, Input as HtmlInput},
    prelude::*,
};
use std::sync::atomic::{AtomicU64, Ordering};

static SEARCH_PICKER_DIALOG_SEQ: AtomicU64 = AtomicU64::new(0);

/// A process-unique id base for one [`SearchPickerDialog`] instance, so two
/// dialogs open in the same document never collide on the heading id that
/// backs `aria-labelledby`.
fn next_search_picker_dialog_id() -> String {
    format!(
        "ld-search-picker-dialog-{}",
        SEARCH_PICKER_DIALOG_SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

/// Caller-reported lifecycle of the current search query, controlling which
/// of the search box/[`PageStatePanel`]/[`KeyedResultList`] combination is
/// mounted. Mirrors the `SnapshotTablePhase`/`PageStatePanelKind` shape used
/// by the client-snapshot patterns so search dialogs present loading, error,
/// and empty states the same way the rest of the framework does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchPickerStatus {
    /// No search has run yet (e.g. the query is still empty).
    Idle,
    /// The first request for the current query is in flight.
    Loading,
    /// The first request for the current query failed.
    Error,
    /// A request completed successfully; `items` reflects its results
    /// (possibly empty).
    Ready,
    /// A replacement request is in flight while the previous results remain
    /// visible.
    Refreshing,
    /// A replacement request failed while the previous results remain
    /// visible.
    RefreshError,
}

/// Pure render decision for one [`SearchPickerStatus`] plus whether `items`
/// is currently non-empty: which [`PageStatePanelKind`] (if any) to show,
/// and whether [`KeyedResultList`] stays mounted underneath it. Mirrors
/// `SnapshotRenderDecision`'s replacement/retained split -- a panel either
/// *replaces* the list (no usable rows exist) or is shown *above* retained
/// rows (a refresh in flight or failed while older rows remain usable).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchPickerRenderDecision {
    panel: Option<PageStatePanelKind>,
    list_mounted: bool,
}

impl SearchPickerRenderDecision {
    /// The state panel to show, if any.
    pub const fn panel(self) -> Option<PageStatePanelKind> {
        self.panel
    }

    /// Whether [`KeyedResultList`] should be mounted (with current or
    /// retained `items`).
    pub const fn list_mounted(self) -> bool {
        self.list_mounted
    }
}

/// Computes the [`SearchPickerRenderDecision`] for a status and whether
/// `items` is currently non-empty. Pure and DOM-free so the precedence is
/// unit-testable without mounting anything.
pub fn search_picker_render_decision(
    status: SearchPickerStatus,
    has_items: bool,
) -> SearchPickerRenderDecision {
    match status {
        SearchPickerStatus::Idle => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::NeverLoaded),
            list_mounted: false,
        },
        SearchPickerStatus::Loading => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::InitialLoading),
            list_mounted: false,
        },
        SearchPickerStatus::Error => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::InitialError),
            list_mounted: false,
        },
        SearchPickerStatus::Ready if !has_items => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::EmptyDataset),
            list_mounted: false,
        },
        SearchPickerStatus::Ready => SearchPickerRenderDecision {
            panel: None,
            list_mounted: true,
        },
        SearchPickerStatus::Refreshing => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::Replacing),
            list_mounted: true,
        },
        SearchPickerStatus::RefreshError => SearchPickerRenderDecision {
            panel: Some(PageStatePanelKind::RetainedError),
            list_mounted: true,
        },
    }
}

/// Complete localizable copy owned by [`SearchPickerDialog`] itself (state
/// panel copy is a separate [`PageStatePanelTexts`], reused rather than
/// duplicated).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchPickerDialogTexts {
    /// Visible label for the search field.
    pub search_label: String,
    /// Placeholder shown in the empty search field.
    pub search_placeholder: String,
    /// Cancel button label.
    pub cancel: String,
}

impl Default for SearchPickerDialogTexts {
    fn default() -> Self {
        Self {
            search_label: "Search".to_owned(),
            search_placeholder: "Type to search…".to_owned(),
            cancel: "Cancel".to_owned(),
        }
    }
}

/// The keys [`SearchPickerDialog`] forwards from the focused search field to
/// the result list, matching exactly the set [`KeyedResultList`] itself
/// operates on.
fn is_forwarded_navigation_key(key: &str) -> bool {
    matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End" | "Enter")
}

/// # SearchPickerDialog
///
/// A controlled, labelled search dialog: a search field plus a typed,
/// keyboard-navigable result list, composed from [`Modal`], [`Field`] +
/// [`Input`], [`KeyedResultList`], and [`PageStatePanel`]. The caller
/// controls `query`, `status`, and `items`; this component controls dialog
/// semantics (focus trap, opening focuses the search field; Escape, a
/// backdrop click, or Cancel all close and return focus to the trigger via
/// the dialog's own native `close()`, through [`Modal`]'s controlled-close
/// contract -- see [`Modal`]), state presentation, and forwarding
/// `ArrowUp`/`ArrowDown`/`Home`/`End`/`Enter` from the search field to the
/// list so results can be navigated without leaving the field.
///
/// Activation (`Enter`, or clicking a row) always resolves against the
/// *current* `items` for the *current* selected key -- see
/// `doc/components/result_list.md` for why this makes result replacement,
/// duplicate-looking rows, and stale async responses safe by construction.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{ResultListItem, ResultRow};
/// use leptos_daisyui_rs::patterns::{SearchPickerDialog, SearchPickerStatus};
///
/// #[derive(Clone)]
/// struct CaseRef { case_number: &'static str }
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let (open, set_open) = signal(false);
///     let (query, set_query) = signal(String::new());
///     let (status, set_status) = signal(SearchPickerStatus::Idle);
///     let items = RwSignal::new(Vec::<ResultListItem<CaseRef>>::new());
///
///     view! {
///         <SearchPickerDialog
///             open=open
///             title="Find a case"
///             query=query
///             status=status
///             items=items
///             on_query_change=Callback::new(move |q| set_query.set(q))
///             on_select=Callback::new(move |item: ResultListItem<CaseRef>| {
///                 leptos::logging::log!("activated {}", item.payload.case_number);
///                 set_open.set(false);
///             })
///             on_close=Callback::new(move |_| set_open.set(false))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// Composes [`Modal`], [`Field`], [`Input`], [`KeyedResultList`], and
/// [`PageStatePanel`] verbatim -- add each of their documented classes; no
/// additional classes are introduced beyond `modal-box`/`modal-action`
/// sizing utilities already covered by `Modal`'s own guidance.
///
/// ## Node References
/// - `node_ref` - References the dialog element ([HTMLDialogElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement))
#[component]
pub fn SearchPickerDialog<T>(
    /// Controls whether the dialog is open.
    #[prop(into)]
    open: Signal<bool>,

    /// Visible dialog title, read by `aria-labelledby`.
    #[prop(into)]
    title: Signal<String>,

    /// Controlled search query text.
    #[prop(into)]
    query: Signal<String>,

    /// Caller-reported lifecycle of the current query, selecting which
    /// state presentation (if any) is shown.
    #[prop(into)]
    status: Signal<SearchPickerStatus>,

    /// Current (or retained, while refreshing) typed, keyed results.
    #[prop(into)]
    items: Signal<Vec<ResultListItem<T>>>,

    /// Fired with the post-filter query text on every edit to the search
    /// field.
    #[prop(optional)]
    on_query_change: Option<Callback<String>>,

    /// Fired when a result is activated (`Enter` or click) with the current
    /// [`ResultListItem`] for the activated key -- see [`KeyedResultList`].
    #[prop(optional)]
    on_select: Option<Callback<ResultListItem<T>>>,

    /// Fired when the dialog should close: `Escape`, a backdrop click, or
    /// the Cancel button. The caller owns `open`; this only requests that it
    /// become `false`.
    #[prop(optional)]
    on_close: Option<Callback<()>>,

    /// Fired when the retry action is activated on an error state
    /// ([`SearchPickerStatus::Error`] or [`SearchPickerStatus::RefreshError`]).
    #[prop(optional)]
    on_retry: Option<Callback<()>>,

    /// Reactive complete localized copy for the dialog chrome.
    #[prop(optional, into, default = Signal::stored(SearchPickerDialogTexts::default()))]
    texts: Signal<SearchPickerDialogTexts>,

    /// Reactive complete localized copy for the loading/error/empty state
    /// panel, reused verbatim from [`PageStatePanel`].
    #[prop(optional, into, default = Signal::stored(PageStatePanelTexts::default()))]
    state_texts: Signal<PageStatePanelTexts>,

    /// Optional caller-provided error detail, forwarded to
    /// [`PageStatePanel`].
    #[prop(optional, into)]
    error_detail: Signal<Option<String>>,

    /// Additional CSS classes for the dialog element.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the dialog element.
    #[prop(optional)]
    node_ref: NodeRef<Dialog>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let heading_id = next_search_picker_dialog_id();
    let title_heading_id = heading_id.clone();
    let search_ref = NodeRef::<HtmlInput>::new();
    let list_ref = NodeRef::<Div>::new();

    // Opening the dialog focuses the search field. Deferred to the next
    // animation frame -- like `focus_tab` in the Tab component -- so it
    // runs after Modal's own effect calls `show_modal()`: calling `.focus()`
    // on a descendant before the dialog is the document's top layer is a
    // silent no-op.
    Effect::new(move |_| {
        if open.get() {
            request_animation_frame(move || {
                if let Some(element) = search_ref.get_untracked() {
                    let _ = element.focus();
                }
            });
        }
    });

    let request_close = move || {
        if let Some(callback) = on_close {
            callback.run(());
        }
    };

    let decision = Signal::derive(move || {
        search_picker_render_decision(status.get(), !items.with(Vec::is_empty))
    });

    // Escape fires a cancelable `cancel` and a backdrop click submits a
    // `method="dialog"` form, which fires no `cancel` at all -- see
    // `Modal`'s controlled-close contract (`ldui-e0fw`). Both, plus any
    // in-content `method="dialog"` submit, are vetoed by `Modal` itself
    // (`on_close_request` switches it into controlled mode) and re-emitted
    // here as one typed `ModalCloseProposal` per gesture. Routing every
    // cause through the same `request_close` callback that already backs
    // Cancel keeps the caller's controlled `open` signal -- the single
    // source of truth `Modal` reads -- as the only thing that ever actually
    // closes the dialog, so `open` can never desync from the DOM the way a
    // hand-rolled `on:cancel` veto (which only ever saw Escape, and never a
    // backdrop click) allowed.
    let handle_close_request = move |_proposal: ModalCloseProposal| {
        request_close();
    };

    // Forwards Arrow/Home/End/Enter from the focused search field to the
    // listbox by dispatching an equivalent native `keydown` on it, so the
    // exact same handler `KeyedResultList` already uses for its own
    // roving-focus keyboard contract runs -- no navigation logic is
    // duplicated here.
    let handle_search_keydown = move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        if !is_forwarded_navigation_key(&key) {
            return;
        }
        let Some(list) = list_ref.get_untracked() else {
            return;
        };
        event.prevent_default();
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(&key);
        init.set_bubbles(true);
        init.set_cancelable(true);
        if let Ok(forwarded) =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        {
            let _ = list.dispatch_event(&forwarded);
        }
    };

    view! {
        <Modal
            open=open
            backdrop=true
            labelled_by=heading_id
            node_ref=node_ref
            class=class
            on_close_request=Callback::new(handle_close_request)
        >
            <ModalBox attr:data-search-picker-dialog="true" class="w-full max-w-2xl">
                <h3 id=title_heading_id class="text-lg font-bold">
                    {move || title.get()}
                </h3>

                <ModalSearchRow>
                    <Field
                        label=Signal::derive(move || Some(texts.with(|t| t.search_label.clone())))
                        class="w-full"
                    >
                        <Input
                            input_type=InputType::Search
                            value=query
                            placeholder=Signal::derive(move || {
                                texts.with(|t| t.search_placeholder.clone())
                            })
                            nostrip:on_input=on_query_change
                            on_keydown=Callback::new(handle_search_keydown)
                            node_ref=search_ref
                            class="w-full"
                        />
                    </Field>
                </ModalSearchRow>

                {move || {
                    decision
                        .get()
                        .panel()
                        .map(|kind| {
                            view! {
                                <PageStatePanel
                                    kind=kind
                                    texts=state_texts
                                    nostrip:on_retry=on_retry
                                    detail=error_detail
                                />
                            }
                        })
                }}

                {move || {
                    decision
                        .get()
                        .list_mounted()
                        .then(|| {
                            view! {
                                <KeyedResultList
                                    items=items
                                    nostrip:on_select=on_select
                                    node_ref=list_ref
                                />
                            }
                        })
                }}

                <ModalAction>
                    <Button
                        style=ButtonStyle::Outline
                        attr:data-search-picker-dialog-cancel="true"
                        on:click=move |_| request_close()
                    >
                        {move || texts.with(|t| t.cancel.clone())}
                    </Button>
                </ModalAction>
            </ModalBox>
        </Modal>
    }
}

// ---------------------------------------------------------------------------
// Confirmable variant (ldui-iq0o)
// ---------------------------------------------------------------------------

/// The keys [`ConfirmableSearchPickerDialog`] forwards from the focused
/// search field to the result list.
///
/// Deliberately [`is_forwarded_navigation_key`] *minus* `Enter`. In the
/// confirmable pattern the result list receives no `on_select` callback at
/// all, so `Enter` on a row has nothing to activate; forwarding it would only
/// swallow the key. Arrow/Home/End still move the caller's accepted selected
/// key -- which is the whole of "activation" here -- without leaving the
/// search field. `Enter` never confirms: confirming is a separate, explicit
/// control, which is the entire point of this pattern.
fn is_forwarded_selection_key(key: &str) -> bool {
    is_forwarded_navigation_key(key) && key != "Enter"
}

/// How a [`ConfirmableSearchPickerDialog`] dismissal was requested. Every
/// variant is a *request*: the caller owns `open` and decides whether to
/// honour it (see the component's "Dismissing with a pending selection"
/// section).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchPickerDismissCause {
    /// The dialog's own Cancel button.
    Cancel,
    /// The user pressed `Escape`.
    Escape,
    /// The user activated the modal backdrop.
    Backdrop,
    /// Some other in-content `method="dialog"` form was submitted.
    DialogForm,
}

impl SearchPickerDismissCause {
    /// Stable slug for tests, telemetry, and consumer state machines.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancel => "cancel",
            Self::Escape => "escape",
            Self::Backdrop => "backdrop",
            Self::DialogForm => "dialog-form",
        }
    }

    /// Lifts one [`Modal`] close cause into this pattern's cause set, which
    /// additionally names the dialog's own Cancel button.
    pub const fn from_modal_cause(cause: ModalCloseCause) -> Self {
        match cause {
            ModalCloseCause::Escape => Self::Escape,
            ModalCloseCause::Backdrop => Self::Backdrop,
            ModalCloseCause::DialogForm => Self::DialogForm,
        }
    }
}

/// Why a confirmation may not proceed. Confirmation *fails closed*: the
/// confirm handler recomputes this at activation time and returns without
/// running the caller's mutation for every variant, so a stale render, a
/// mid-flight key change, or a synthesized click can never slip a write past
/// the disabled presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchPickerConfirmBlock {
    /// No result is selected.
    NoSelection,
    /// A key is selected but resolves to no typed item -- neither in the
    /// current `items` nor in the pattern's retained last-resolved selection.
    UnresolvedSelection,
    /// The caller reports a confirmation already in flight.
    Pending,
}

impl SearchPickerConfirmBlock {
    /// Stable slug emitted as the confirm control's `data-confirm-state`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoSelection => "blocked-no-selection",
            Self::UnresolvedSelection => "blocked-unresolved-selection",
            Self::Pending => "pending",
        }
    }
}

/// `data-confirm-state` for the confirm control: the block's slug, or
/// `"ready"` when confirmation may proceed.
fn confirm_state_marker(block: Option<SearchPickerConfirmBlock>) -> &'static str {
    match block {
        None => "ready",
        Some(block) => block.as_str(),
    }
}

/// `data-selection-state` for the selected-result summary.
fn selection_state_marker(has_key: bool, resolved: bool) -> &'static str {
    match (has_key, resolved) {
        (false, _) => "none",
        (true, true) => "resolved",
        (true, false) => "unresolved",
    }
}

/// Resolves the caller's accepted selected key to a typed item, preferring
/// the *current* `items` and falling back to the pattern's retained
/// last-resolved selection for the same key.
///
/// The retained fallback is what makes "select a worker, then narrow the
/// search past that worker's row" keep naming the worker: the key is still
/// accepted, the row is simply not on screen. The fresh lookup takes
/// precedence so a relabelled or repayloaded row updates the summary rather
/// than showing a stale copy -- the retained item is an identity fallback,
/// never a cache. A retained item whose key differs from the accepted key is
/// ignored, which is what makes an unknown key resolve to `None` (fail
/// closed) rather than to the previously selected worker.
pub fn resolve_search_picker_selection<T: Clone>(
    selected_key: Option<&str>,
    items: &[ResultListItem<T>],
    retained: Option<&ResultListItem<T>>,
) -> Option<ResultListItem<T>> {
    let key = selected_key?;
    if let Some(item) = items.iter().find(|item| item.key == key) {
        return Some(item.clone());
    }
    retained
        .filter(|item| item.key == key)
        .map(|item| (*item).clone())
}

/// Whether confirmation is currently blocked, and why. Pure and payload-free
/// so the disabled presentation and the activation guard cannot disagree:
/// both call this.
///
/// Precedence is `NoSelection` before `Pending` before
/// `UnresolvedSelection`: a caller reporting `pending` with no selected key
/// is reporting a state this pattern cannot have produced, and naming the
/// missing selection is more useful than naming the flight.
pub fn search_picker_confirm_block<T>(
    selected_key: Option<&str>,
    items: &[ResultListItem<T>],
    retained_key: Option<&str>,
    pending: bool,
) -> Option<SearchPickerConfirmBlock> {
    let Some(key) = selected_key else {
        return Some(SearchPickerConfirmBlock::NoSelection);
    };
    if pending {
        return Some(SearchPickerConfirmBlock::Pending);
    }
    let resolvable = items.iter().any(|item| item.key == key) || retained_key == Some(key);
    (!resolvable).then_some(SearchPickerConfirmBlock::UnresolvedSelection)
}

/// The typed item a confirmation would submit, or why it may not run.
///
/// Re-checks resolution after [`search_picker_confirm_block`] passes, so the
/// two can never disagree about whether a payload exists; a resolution that
/// somehow fails there still returns
/// [`SearchPickerConfirmBlock::UnresolvedSelection`] rather than an
/// `unwrap`.
pub fn resolve_search_picker_confirmation<T: Clone>(
    selected_key: Option<&str>,
    items: &[ResultListItem<T>],
    retained: Option<&ResultListItem<T>>,
    pending: bool,
) -> Result<ResultListItem<T>, SearchPickerConfirmBlock> {
    if let Some(block) = search_picker_confirm_block(
        selected_key,
        items,
        retained.map(|item| item.key.as_str()),
        pending,
    ) {
        return Err(block);
    }
    resolve_search_picker_selection(selected_key, items, retained)
        .ok_or(SearchPickerConfirmBlock::UnresolvedSelection)
}

/// Complete localizable copy owned by [`ConfirmableSearchPickerDialog`]
/// itself. Loading/error/empty/retry copy is a separate
/// [`PageStatePanelTexts`], reused rather than duplicated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmableSearchPickerDialogTexts {
    /// Visible label for the search field.
    pub search_label: String,
    /// Placeholder shown in the empty search field.
    pub search_placeholder: String,
    /// Label introducing the selected-result summary.
    pub selected_label: String,
    /// Summary copy shown while nothing is selected.
    pub selected_none: String,
    /// Cancel button label.
    pub cancel: String,
    /// Confirm button label while confirmation is available.
    pub confirm: String,
    /// Confirm button label while a confirmation is in flight; also the
    /// confirm control's description in that state.
    pub confirm_pending: String,
    /// Why confirmation is unavailable with nothing selected.
    pub confirm_blocked_no_selection: String,
    /// Why confirmation is unavailable when the selected identity no longer
    /// resolves to a typed item.
    pub confirm_blocked_unresolved: String,
}

impl Default for ConfirmableSearchPickerDialogTexts {
    fn default() -> Self {
        Self {
            search_label: "Search".to_owned(),
            search_placeholder: "Type to search…".to_owned(),
            selected_label: "Selected".to_owned(),
            selected_none: "No result selected yet.".to_owned(),
            cancel: "Cancel".to_owned(),
            confirm: "Confirm".to_owned(),
            confirm_pending: "Confirming…".to_owned(),
            confirm_blocked_no_selection: "Select a result to continue.".to_owned(),
            confirm_blocked_unresolved:
                "The selected result is no longer available; choose another.".to_owned(),
        }
    }
}

/// The confirm control's description for one block state. Empty when
/// confirmation is available, so the description element carries text only
/// when it has something to explain.
fn confirm_hint_text(
    texts: &ConfirmableSearchPickerDialogTexts,
    block: Option<SearchPickerConfirmBlock>,
) -> String {
    match block {
        None => String::new(),
        Some(SearchPickerConfirmBlock::NoSelection) => texts.confirm_blocked_no_selection.clone(),
        Some(SearchPickerConfirmBlock::UnresolvedSelection) => {
            texts.confirm_blocked_unresolved.clone()
        }
        Some(SearchPickerConfirmBlock::Pending) => texts.confirm_pending.clone(),
    }
}

/// # ConfirmableSearchPickerDialog
///
/// The review-before-mutation sibling of [`SearchPickerDialog`]: search,
/// select, *then* explicitly confirm. Selecting a result is reversible and
/// has no side effect; the caller's mutation runs only from the Confirm
/// control.
///
/// ## Choosing between the two
///
/// | | [`SearchPickerDialog`] | `ConfirmableSearchPickerDialog` |
/// |---|---|---|
/// | Activating a row | *is* the terminal action | only moves the selected key |
/// | Caller callback | `on_select` | `on_confirm` |
/// | Steps to act | one | two |
/// | Reach for it when | navigating, opening, filtering -- reversible | assigning, reassigning, linking, restoring -- a write |
///
/// The split is at the type level rather than behind a flag: this component
/// passes *no* `on_select` to its [`KeyedResultList`] at all, so there is no
/// configuration in which activating a row can reach a mutation callback. A
/// consumer cannot accidentally turn selection into a write.
///
/// ## Controlled everywhere
///
/// `open`, `query`, `status`, `items`, `selected_key`, and `pending` are all
/// caller-owned signals; every gesture emits a typed proposal
/// ([`KeyedResultListSelectionProposal`] for selection,
/// [`SearchPickerDismissCause`] for dismissal) and nothing is ever applied
/// optimistically. Selection ownership is [`KeyedResultList`]'s own
/// controlled model ([`KeyedResultListSelection::controlled`]), not a
/// re-implementation: a selected key absent from the current `items` renders
/// no false highlight while leaving the caller's signal untouched, and the
/// highlight returns the moment a matching row does.
///
/// ## The selected result survives a narrowing search
///
/// Search narrows `items`, so the selected row routinely leaves the visible
/// set. The selection survives that: the caller's key is untouched, the
/// summary keeps naming the selection from the pattern's retained
/// last-resolved item, and Confirm still resolves and submits it. See
/// [`resolve_search_picker_selection`] for the precedence (fresh item first,
/// retained item only as an identity fallback for the *same* key).
///
/// ## Dismissing with a pending selection
///
/// `Escape`, the backdrop, and Cancel all emit `on_close` with the
/// [`SearchPickerDismissCause`] that produced them, and nothing else. The
/// pattern never proposes clearing `selected_key` on dismissal, so a
/// selection the user made is not silently discarded: reopening the dialog
/// restores it, summary and all. Nor is dismissal ever blocked -- including
/// while a confirmation is in flight -- because `open` is caller-owned and a
/// pattern that refuses to close traps the user. A caller that wants
/// dismissal to discard the selection clears its own key from `on_close`; a
/// caller with an uncancellable write in flight simply ignores the proposal.
///
/// ## While a confirmation is in flight
///
/// `pending=true` blocks confirmation ([`SearchPickerConfirmBlock::Pending`]),
/// swaps the Confirm label for `confirm_pending`, and marks the control
/// `aria-busy`. The dialog does *not* close itself on confirm -- it has no
/// write to observe and cannot know whether one succeeded. The caller closes
/// on success; on failure it clears `pending` and supplies `confirm_error`,
/// and because the dialog stayed open with its selection intact the user's
/// context survives the failure and Confirm can simply be pressed again.
///
/// ## Confirm is `aria-disabled`, never natively `disabled`
///
/// A natively disabled button leaves the accessibility tree and the tab
/// order, which takes the explanation of *why* it is unavailable with it --
/// exactly the users who need that reason lose access to it. Worse for a
/// pending control: a button that natively disables itself under the user's
/// own focus dumps focus to `<body>` mid-interaction. So the control stays
/// focusable, reports `aria-disabled`, describes itself with the blocking
/// reason, and refuses the action in its handler. Same ruling and reasoning
/// as `RecordHeader`'s quick actions (`ldui-9d0q`).
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{KeyedResultListSelectionProposal, ResultListItem};
/// use leptos_daisyui_rs::patterns::{ConfirmableSearchPickerDialog, SearchPickerStatus};
///
/// #[derive(Clone)]
/// struct Worker { worker_id: &'static str }
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let open = RwSignal::new(false);
///     let query = RwSignal::new(String::new());
///     let status = RwSignal::new(SearchPickerStatus::Idle);
///     let items = RwSignal::new(Vec::<ResultListItem<Worker>>::new());
///     let selected_key = RwSignal::new(None::<String>);
///     let pending = RwSignal::new(false);
///
///     view! {
///         <ConfirmableSearchPickerDialog
///             open=open
///             control_id="standing-order-owner"
///             title="Assign owner"
///             query=query
///             status=status
///             items=items
///             selected_key=selected_key
///             pending=pending
///             on_query_change=Callback::new(move |q| query.set(q))
///             on_selection_change=Callback::new(move |p: KeyedResultListSelectionProposal| {
///                 selected_key.set(p.key);
///             })
///             on_confirm=Callback::new(move |item: ResultListItem<Worker>| {
///                 pending.set(true);
///                 leptos::logging::log!("assign {}", item.payload.worker_id);
///             })
///             on_close=Callback::new(move |_| open.set(false))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// Composes [`Modal`], [`Field`], [`Input`], [`KeyedResultList`], and
/// [`PageStatePanel`] verbatim -- add each of their documented classes.
///
/// ## Node References
/// - `node_ref` - References the dialog element ([HTMLDialogElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement))
#[component]
pub fn ConfirmableSearchPickerDialog<T>(
    /// Controls whether the dialog is open.
    #[prop(into)]
    open: Signal<bool>,

    /// Stable caller-supplied contract/control id. Every id and name this
    /// dialog emits is derived from it (`-title`, `-description`,
    /// `-selection`, `-confirm-hint`, `-search`, `-cancel`, `-confirm`), so
    /// two simultaneous instances with distinct contract ids never collide.
    #[prop(into)]
    control_id: String,

    /// Visible dialog title, read by `aria-labelledby`.
    #[prop(into)]
    title: Signal<String>,

    /// Optional dialog description, rendered under the title and read by
    /// `aria-describedby`. `aria-describedby` is emitted only when this is
    /// `Some`.
    #[prop(optional, into)]
    description: Signal<Option<String>>,

    /// Controlled search query text.
    #[prop(into)]
    query: Signal<String>,

    /// Caller-reported lifecycle of the current query.
    #[prop(into)]
    status: Signal<SearchPickerStatus>,

    /// Current (or retained, while refreshing) typed, keyed results.
    #[prop(into)]
    items: Signal<Vec<ResultListItem<T>>>,

    /// Caller-owned accepted selected key. Authoritative: the dialog never
    /// writes it.
    #[prop(into)]
    selected_key: Signal<Option<String>>,

    /// Whether a confirmation the caller started is still in flight.
    #[prop(optional, into)]
    pending: Signal<bool>,

    /// Fired with the post-filter query text on every edit to the search
    /// field.
    #[prop(optional)]
    on_query_change: Option<Callback<String>>,

    /// Fired with one complete replacement proposal whenever a row is
    /// clicked or keyboard navigation moves the highlight. Never a
    /// mutation.
    on_selection_change: Callback<KeyedResultListSelectionProposal>,

    /// Fired only from the Confirm control, with the typed item resolved at
    /// activation time. Required: a confirmable dialog with nothing to
    /// confirm is a configuration error, so it is not expressible.
    on_confirm: Callback<ResultListItem<T>>,

    /// Fired when dismissal is requested, with its cause. The caller owns
    /// `open`; this only requests that it become `false`.
    on_close: Callback<SearchPickerDismissCause>,

    /// Fired when the retry action is activated on an error state.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,

    /// Reactive complete localized copy for the dialog chrome.
    #[prop(optional, into, default = Signal::stored(ConfirmableSearchPickerDialogTexts::default()))]
    texts: Signal<ConfirmableSearchPickerDialogTexts>,

    /// Reactive complete localized copy for the loading/error/empty state
    /// panel, reused verbatim from [`PageStatePanel`].
    #[prop(optional, into, default = Signal::stored(PageStatePanelTexts::default()))]
    state_texts: Signal<PageStatePanelTexts>,

    /// Optional caller-provided search error detail, forwarded to
    /// [`PageStatePanel`].
    #[prop(optional, into)]
    error_detail: Signal<Option<String>>,

    /// Optional caller-provided message for a confirmation that failed,
    /// rendered in the footer as a live `role="alert"`. Caller-provided
    /// *content* (like `error_detail`), not pattern-owned copy.
    #[prop(optional, into)]
    confirm_error: Signal<Option<String>>,

    /// Additional CSS classes for the dialog element.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the dialog element.
    #[prop(optional)]
    node_ref: NodeRef<Dialog>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let title_id = format!("{control_id}-title");
    let description_id = format!("{control_id}-description");
    let summary_id = format!("{control_id}-selection");
    let confirm_hint_id = format!("{control_id}-confirm-hint");
    let search_name = format!("{control_id}-search");
    let cancel_name = format!("{control_id}-cancel");
    let confirm_name = format!("{control_id}-confirm");

    let search_ref = NodeRef::<HtmlInput>::new();
    let list_ref = NodeRef::<Div>::new();

    // Opening focuses the search field, deferred to the next animation frame
    // so it runs after Modal's own effect calls `show_modal()` -- focusing a
    // descendant before the dialog reaches the top layer is a silent no-op.
    // Same mechanism as `SearchPickerDialog`.
    Effect::new(move |_| {
        if open.get() {
            request_animation_frame(move || {
                if let Some(element) = search_ref.get_untracked() {
                    let _ = element.focus();
                }
            });
        }
    });

    // The last typed item the caller's accepted key actually resolved to.
    // Kept so a search that narrows past the selected row still names and
    // still confirms it. Refreshed whenever the key resolves against current
    // `items`, and dropped the moment the accepted key becomes something
    // this retention cannot vouch for -- which is what makes an unknown key
    // fail closed instead of confirming the previously selected item.
    let retained = RwSignal::<Option<ResultListItem<T>>>::new(None);
    Effect::new(move |_| {
        let key = selected_key.get();
        let fresh = items.with(|current| {
            key.as_deref()
                .and_then(|key| current.iter().find(|item| item.key == key).cloned())
        });
        match fresh {
            Some(item) => retained.set(Some(item)),
            None => {
                let keeps_identity = retained
                    .with_untracked(|held| held.as_ref().map(|item| item.key.clone()) == key);
                if !keeps_identity {
                    retained.set(None);
                }
            }
        }
    });

    let resolved_selection = Signal::derive(move || {
        let key = selected_key.get();
        items.with(|current| {
            retained.with(|held| {
                resolve_search_picker_selection(key.as_deref(), current, held.as_ref())
            })
        })
    });

    let confirm_block = Signal::derive(move || {
        let key = selected_key.get();
        let pending = pending.get();
        items.with(|current| {
            retained.with(|held| {
                search_picker_confirm_block(
                    key.as_deref(),
                    current,
                    held.as_ref().map(|item| item.key.as_str()),
                    pending,
                )
            })
        })
    });

    let request_close = move |cause: SearchPickerDismissCause| {
        on_close.run(cause);
    };

    // Escape, the backdrop, and any in-content `method="dialog"` submit are
    // all vetoed by `Modal` and re-emitted as one typed proposal per gesture
    // (`ldui-e0fw`/`ldui-rolc`); each is forwarded with its own cause and
    // leaves `selected_key` alone.
    let handle_close_request = move |proposal: ModalCloseProposal| {
        request_close(SearchPickerDismissCause::from_modal_cause(proposal.cause));
    };

    // Arrow/Home/End are forwarded from the focused search field to the
    // listbox as a native `keydown`, so `KeyedResultList`'s own handler runs
    // and proposes the selection change -- no navigation logic is duplicated
    // here, and focus never leaves the search field. `Enter` is deliberately
    // not forwarded; see `is_forwarded_selection_key`.
    let handle_search_keydown = move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        if !is_forwarded_selection_key(&key) {
            return;
        }
        let Some(list) = list_ref.get_untracked() else {
            return;
        };
        event.prevent_default();
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(&key);
        init.set_bubbles(true);
        init.set_cancelable(true);
        if let Ok(forwarded) =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
        {
            let _ = list.dispatch_event(&forwarded);
        }
    };

    // The mutation guard. Resolution is recomputed untracked at activation
    // time rather than read off the last render, so nothing that happened
    // between paint and click -- a landed response, a cleared key, a
    // confirmation that started -- can let a write through.
    let handle_confirm = move || {
        let key = selected_key.get_untracked();
        let pending = pending.get_untracked();
        let resolution = items.with_untracked(|current| {
            retained.with_untracked(|held| {
                resolve_search_picker_confirmation(key.as_deref(), current, held.as_ref(), pending)
            })
        });
        if let Ok(item) = resolution {
            on_confirm.run(item);
        }
    };

    // No `on_select`: activating a row proposes a selection and nothing
    // else. This is the structural half of "selecting must not write".
    let selection = KeyedResultListSelection::controlled(
        selected_key,
        Callback::new(move |proposal: KeyedResultListSelectionProposal| {
            on_selection_change.run(proposal);
        }),
    );

    let decision = Signal::derive(move || {
        search_picker_render_decision(status.get(), !items.with(Vec::is_empty))
    });

    let described_by = {
        let description_id = description_id.clone();
        Signal::derive(move || {
            description
                .with(|value| value.is_some())
                .then(|| description_id.clone())
        })
    };

    let summary_state = move || {
        selection_state_marker(
            selected_key.with(Option::is_some),
            resolved_selection.with(Option::is_some),
        )
    };

    view! {
        <Modal
            open=open
            backdrop=true
            labelled_by=title_id.clone()
            described_by=described_by
            node_ref=node_ref
            class=class
            on_close_request=Callback::new(handle_close_request)
        >
            <ModalBox
                attr:data-confirmable-search-picker-dialog="true"
                attr:data-control-id=control_id.clone()
                class="w-full max-w-2xl"
            >
                <h3 id=title_id class="text-lg font-bold">
                    {move || title.get()}
                </h3>

                {move || {
                    description
                        .get()
                        .map(|text| {
                            view! {
                                <p
                                    id=description_id.clone()
                                    class="ld-text-body text-base-content/75"
                                >
                                    {text}
                                </p>
                            }
                        })
                }}

                <ModalSearchRow>
                    <Field
                        label=Signal::derive(move || Some(texts.with(|t| t.search_label.clone())))
                        class="w-full"
                    >
                        <Input
                            input_type=InputType::Search
                            value=query
                            placeholder=Signal::derive(move || {
                                texts.with(|t| t.search_placeholder.clone())
                            })
                            name=Signal::stored(Some(search_name))
                            attr:data-confirmable-search-picker-search="true"
                            nostrip:on_input=on_query_change
                            on_keydown=Callback::new(handle_search_keydown)
                            node_ref=search_ref
                            class="w-full"
                        />
                    </Field>
                </ModalSearchRow>

                {move || {
                    decision
                        .get()
                        .panel()
                        .map(|kind| {
                            view! {
                                <PageStatePanel
                                    kind=kind
                                    texts=state_texts
                                    nostrip:on_retry=on_retry
                                    detail=error_detail
                                />
                            }
                        })
                }}

                {move || {
                    decision
                        .get()
                        .list_mounted()
                        .then(|| {
                            view! {
                                <div data-confirmable-search-picker-results="true">
                                    <KeyedResultList
                                        items=items
                                        selection=selection
                                        node_ref=list_ref
                                    />
                                </div>
                            }
                        })
                }}

                <div
                    id=summary_id
                    data-confirmable-search-picker-summary="true"
                    data-selection-state=summary_state
                    data-selected-key=move || selected_key.get()
                    aria-live="polite"
                    class="mt-4 flex flex-col gap-1 rounded-box border border-base-300 p-3"
                >
                    <span class="ld-text-caption text-base-content/75">
                        {move || texts.with(|t| t.selected_label.clone())}
                    </span>
                    <span class="ld-text-body font-semibold">
                        {move || {
                            resolved_selection
                                .get()
                                .map_or_else(
                                    || texts.with(|t| t.selected_none.clone()),
                                    |item| item.row.title.clone(),
                                )
                        }}
                    </span>
                    {move || {
                        resolved_selection
                            .get()
                            .map(|item| item.row.secondary_line().to_owned())
                            .filter(|line| !line.is_empty())
                            .map(|line| {
                                view! {
                                    <span class="ld-text-caption text-base-content/75">{line}</span>
                                }
                            })
                    }}
                </div>

                {move || {
                    confirm_error
                        .get()
                        .map(|message| {
                            view! {
                                <p
                                    role="alert"
                                    data-confirmable-search-picker-error="true"
                                    class="ld-text-caption mt-3 text-error"
                                >
                                    {message}
                                </p>
                            }
                        })
                }}

                <p
                    id=confirm_hint_id.clone()
                    data-confirmable-search-picker-confirm-hint="true"
                    class="ld-text-caption mt-3 text-base-content/75"
                >
                    {move || texts.with(|t| confirm_hint_text(t, confirm_block.get()))}
                </p>

                <ModalAction>
                    <Button
                        style=ButtonStyle::Outline
                        attr:name=cancel_name
                        attr:data-confirmable-search-picker-cancel="true"
                        on:click=move |_| request_close(SearchPickerDismissCause::Cancel)
                    >
                        {move || texts.with(|t| t.cancel.clone())}
                    </Button>
                    <Button
                        color=ButtonColor::Primary
                        attr:name=confirm_name
                        attr:data-confirmable-search-picker-confirm="true"
                        attr:data-confirm-state=move || confirm_state_marker(confirm_block.get())
                        attr:aria-disabled=move || {
                            confirm_block.with(Option::is_some).then_some("true")
                        }
                        attr:aria-busy=move || pending.get().then_some("true")
                        attr:aria-describedby=confirm_hint_id
                        on:click=move |_| handle_confirm()
                    >
                        {move || {
                            texts
                                .with(|t| {
                                    if pending.get() {
                                        t.confirm_pending.clone()
                                    } else {
                                        t.confirm.clone()
                                    }
                                })
                        }}
                    </Button>
                </ModalAction>
            </ModalBox>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_replaces_the_list_with_never_loaded() {
        let decision = search_picker_render_decision(SearchPickerStatus::Idle, false);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::NeverLoaded));
        assert!(!decision.list_mounted());
    }

    #[test]
    fn loading_replaces_the_list_regardless_of_stale_items() {
        // `has_items` reflects whatever `items` currently holds; Loading
        // always replaces the list even if a caller left stale rows in
        // place, matching InitialLoading's "no usable snapshot yet" meaning.
        let decision = search_picker_render_decision(SearchPickerStatus::Loading, true);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::InitialLoading));
        assert!(!decision.list_mounted());
    }

    #[test]
    fn error_replaces_the_list() {
        let decision = search_picker_render_decision(SearchPickerStatus::Error, false);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::InitialError));
        assert!(!decision.list_mounted());
    }

    #[test]
    fn ready_with_no_items_shows_empty_dataset_and_no_list() {
        let decision = search_picker_render_decision(SearchPickerStatus::Ready, false);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::EmptyDataset));
        assert!(!decision.list_mounted());
    }

    #[test]
    fn ready_with_items_shows_no_panel_and_mounts_the_list() {
        let decision = search_picker_render_decision(SearchPickerStatus::Ready, true);
        assert_eq!(decision.panel(), None);
        assert!(decision.list_mounted());
    }

    #[test]
    fn refreshing_retains_the_list_under_a_replacing_notice() {
        let decision = search_picker_render_decision(SearchPickerStatus::Refreshing, true);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::Replacing));
        assert!(
            decision.list_mounted(),
            "a refresh must not drop the previously visible rows"
        );
    }

    #[test]
    fn refresh_error_retains_the_list_under_a_retained_error_notice() {
        let decision = search_picker_render_decision(SearchPickerStatus::RefreshError, true);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::RetainedError));
        assert!(
            decision.list_mounted(),
            "a failed refresh must not drop the previously visible rows"
        );
    }

    /// A refresh that started from zero previous rows still can't activate
    /// an old payload (there wasn't one) -- confirms `has_items` alone, not
    /// status, decides list mounting once a request is no longer initial.
    #[test]
    fn refreshing_with_no_retained_items_still_mounts_the_list() {
        let decision = search_picker_render_decision(SearchPickerStatus::Refreshing, false);
        assert!(decision.list_mounted());
    }

    /// Same reasoning as [`refreshing_with_no_retained_items_still_mounts_the_list`]
    /// for the failed-refresh branch: `has_items` alone decides list
    /// mounting, so a `RefreshError` that started from zero retained rows
    /// still mounts the (empty) list under the retained-error notice rather
    /// than falling back to a replacement panel.
    #[test]
    fn refresh_error_with_no_retained_items_still_mounts_the_list() {
        let decision = search_picker_render_decision(SearchPickerStatus::RefreshError, false);
        assert_eq!(decision.panel(), Some(PageStatePanelKind::RetainedError));
        assert!(decision.list_mounted());
    }

    #[test]
    fn search_picker_dialog_texts_default_is_nonempty() {
        let texts = SearchPickerDialogTexts::default();
        assert!(!texts.search_label.trim().is_empty());
        assert!(!texts.search_placeholder.trim().is_empty());
        assert!(!texts.cancel.trim().is_empty());
    }

    #[test]
    fn dialog_ids_are_unique_across_instances() {
        let a = next_search_picker_dialog_id();
        let b = next_search_picker_dialog_id();
        assert_ne!(a, b, "two open dialogs must never share a heading id");
        assert!(a.starts_with("ld-search-picker-dialog-"));
    }

    #[test]
    fn forwarded_navigation_keys_match_the_result_lists_own_contract() {
        for key in ["ArrowDown", "ArrowUp", "Home", "End", "Enter"] {
            assert!(is_forwarded_navigation_key(key), "{key} must be forwarded");
        }
        for key in ["Escape", "Tab", "a", " "] {
            assert!(
                !is_forwarded_navigation_key(key),
                "{key} must not be forwarded"
            );
        }
    }

    /// Backward-compatibility pin for every activation-as-terminal caller
    /// (`ldui-iq0o`). This is an *exhaustive* struct literal with no
    /// `..Default::default()` rest: adding, removing, or renaming a single
    /// `SearchPickerDialog` prop fails to compile here. The confirmable
    /// pattern therefore cannot have been bolted onto this component --
    /// it is a separate one -- and existing `on_select` callers keep the
    /// terminal semantics they were written against.
    #[test]
    fn search_picker_dialog_builds_with_every_state() {
        #[derive(Clone)]
        struct Payload;

        for status in [
            SearchPickerStatus::Idle,
            SearchPickerStatus::Loading,
            SearchPickerStatus::Error,
            SearchPickerStatus::Ready,
            SearchPickerStatus::Refreshing,
            SearchPickerStatus::RefreshError,
        ] {
            let _ = SearchPickerDialog(SearchPickerDialogProps::<Payload> {
                open: Signal::stored(true),
                title: Signal::stored("Find a case".to_string()),
                query: Signal::stored(String::new()),
                status: Signal::stored(status),
                items: Signal::stored(Vec::new()),
                on_query_change: None,
                on_select: None,
                on_close: None,
                on_retry: None,
                texts: Signal::stored(SearchPickerDialogTexts::default()),
                state_texts: Signal::stored(PageStatePanelTexts::default()),
                error_detail: Signal::stored(None),
                class: "",
                node_ref: NodeRef::new(),
            });
        }
    }

    // -- confirmable variant (ldui-iq0o) ------------------------------------

    use crate::components::ResultRow;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Worker {
        worker_id: &'static str,
    }

    fn worker(key: &str, title: &str, worker_id: &'static str) -> ResultListItem<Worker> {
        ResultListItem::new(key, ResultRow::new(title), Worker { worker_id })
    }

    fn directory() -> Vec<ResultListItem<Worker>> {
        vec![
            worker("worker-a", "Alex Morgan", "W-100"),
            worker("worker-b", "Alex Morgan", "W-200"),
            worker("worker-c", "Priya Natarajan", "W-300"),
        ]
    }

    #[test]
    fn selection_resolves_the_exact_keyed_item_not_the_display_text() {
        let items = directory();
        let resolved = resolve_search_picker_selection(Some("worker-b"), &items, None)
            .expect("worker-b resolves");
        assert_eq!(resolved.key, "worker-b");
        assert_eq!(
            resolved.payload.worker_id, "W-200",
            "two rows share the title 'Alex Morgan'; the key alone decides"
        );
    }

    #[test]
    fn a_selection_narrowed_out_of_the_results_survives_via_retention() {
        let selected = worker("worker-b", "Alex Morgan", "W-200");
        // The user typed "Priya", so worker-b is no longer in `items`.
        let narrowed = vec![worker("worker-c", "Priya Natarajan", "W-300")];
        let resolved =
            resolve_search_picker_selection(Some("worker-b"), &narrowed, Some(&selected))
                .expect("a selection filtered out of view is still named");
        assert_eq!(resolved.payload.worker_id, "W-200");
        assert_eq!(
            search_picker_confirm_block(Some("worker-b"), &narrowed, Some("worker-b"), false),
            None,
            "and is still confirmable -- narrowing the search is not a deselection"
        );
    }

    #[test]
    fn a_fresh_item_wins_over_the_retained_copy() {
        // The row was relabelled and repayloaded by a newer response.
        let stale = worker("worker-b", "Alex Morgan", "W-200");
        let fresh = vec![worker("worker-b", "Alexandra Morgan", "W-201")];
        let resolved = resolve_search_picker_selection(Some("worker-b"), &fresh, Some(&stale))
            .expect("worker-b resolves");
        assert_eq!(resolved.row.title, "Alexandra Morgan");
        assert_eq!(
            resolved.payload.worker_id, "W-201",
            "retention is an identity fallback, never a payload cache"
        );
    }

    #[test]
    fn a_retained_item_for_a_different_key_never_stands_in() {
        let held = worker("worker-b", "Alex Morgan", "W-200");
        let items = vec![worker("worker-c", "Priya Natarajan", "W-300")];
        assert_eq!(
            resolve_search_picker_selection(Some("worker-z"), &items, Some(&held)),
            None,
            "an unknown key must not confirm the previously selected worker"
        );
    }

    #[test]
    fn no_selection_blocks_confirmation() {
        let items = directory();
        assert_eq!(
            search_picker_confirm_block(None, &items, None, false),
            Some(SearchPickerConfirmBlock::NoSelection)
        );
    }

    #[test]
    fn a_stale_key_blocks_confirmation() {
        let items = directory();
        assert_eq!(
            search_picker_confirm_block(Some("worker-gone"), &items, None, false),
            Some(SearchPickerConfirmBlock::UnresolvedSelection)
        );
    }

    #[test]
    fn a_confirmation_in_flight_blocks_another_one() {
        let items = directory();
        assert_eq!(
            search_picker_confirm_block(Some("worker-a"), &items, None, true),
            Some(SearchPickerConfirmBlock::Pending),
            "a second confirm during a caller's write must never reach the mutation"
        );
    }

    #[test]
    fn a_missing_selection_outranks_a_reported_flight() {
        let items = directory();
        assert_eq!(
            search_picker_confirm_block(None, &items, None, true),
            Some(SearchPickerConfirmBlock::NoSelection),
            "documented precedence: naming the missing selection beats naming the flight"
        );
    }

    #[test]
    fn confirmation_returns_the_current_typed_item_when_unblocked() {
        let items = directory();
        let confirmed = resolve_search_picker_confirmation(Some("worker-c"), &items, None, false)
            .expect("worker-c confirms");
        assert_eq!(confirmed.payload.worker_id, "W-300");
    }

    #[test]
    fn confirmation_fails_closed_for_every_block() {
        let items = directory();
        let held = worker("worker-b", "Alex Morgan", "W-200");
        for (key, pending, expected) in [
            (None, false, SearchPickerConfirmBlock::NoSelection),
            (Some("worker-a"), true, SearchPickerConfirmBlock::Pending),
            (
                Some("worker-gone"),
                false,
                SearchPickerConfirmBlock::UnresolvedSelection,
            ),
        ] {
            assert_eq!(
                resolve_search_picker_confirmation(key, &items, Some(&held), pending),
                Err(expected),
                "a block must never yield a payload"
            );
        }
    }

    #[test]
    fn confirm_state_markers_are_distinct_and_stable() {
        let markers = [
            confirm_state_marker(None),
            confirm_state_marker(Some(SearchPickerConfirmBlock::NoSelection)),
            confirm_state_marker(Some(SearchPickerConfirmBlock::UnresolvedSelection)),
            confirm_state_marker(Some(SearchPickerConfirmBlock::Pending)),
        ];
        assert_eq!(markers[0], "ready");
        let unique: std::collections::HashSet<&str> = markers.iter().copied().collect();
        assert_eq!(unique.len(), markers.len());
    }

    #[test]
    fn selection_state_markers_cover_every_case() {
        assert_eq!(selection_state_marker(false, false), "none");
        assert_eq!(selection_state_marker(false, true), "none");
        assert_eq!(selection_state_marker(true, true), "resolved");
        assert_eq!(selection_state_marker(true, false), "unresolved");
    }

    #[test]
    fn dismiss_causes_map_from_every_modal_cause_and_add_cancel() {
        assert_eq!(
            SearchPickerDismissCause::from_modal_cause(ModalCloseCause::Escape),
            SearchPickerDismissCause::Escape
        );
        assert_eq!(
            SearchPickerDismissCause::from_modal_cause(ModalCloseCause::Backdrop),
            SearchPickerDismissCause::Backdrop
        );
        assert_eq!(
            SearchPickerDismissCause::from_modal_cause(ModalCloseCause::DialogForm),
            SearchPickerDismissCause::DialogForm
        );
        let slugs: std::collections::HashSet<&str> = [
            SearchPickerDismissCause::Cancel,
            SearchPickerDismissCause::Escape,
            SearchPickerDismissCause::Backdrop,
            SearchPickerDismissCause::DialogForm,
        ]
        .iter()
        .map(|cause| cause.as_str())
        .collect();
        assert_eq!(slugs.len(), 4);
    }

    /// `Enter` must not reach the result list in the confirmable pattern:
    /// the list has no `on_select` to run, and confirming is a separate
    /// explicit control.
    #[test]
    fn enter_is_not_forwarded_in_the_confirmable_dialog() {
        for key in ["ArrowDown", "ArrowUp", "Home", "End"] {
            assert!(is_forwarded_selection_key(key), "{key} must be forwarded");
        }
        assert!(
            is_forwarded_navigation_key("Enter"),
            "the terminal dialog still forwards Enter"
        );
        assert!(
            !is_forwarded_selection_key("Enter"),
            "the confirmable dialog must not forward Enter"
        );
    }

    #[test]
    fn confirmable_texts_default_is_complete_and_nonempty() {
        let texts = ConfirmableSearchPickerDialogTexts::default();
        for value in [
            &texts.search_label,
            &texts.search_placeholder,
            &texts.selected_label,
            &texts.selected_none,
            &texts.cancel,
            &texts.confirm,
            &texts.confirm_pending,
            &texts.confirm_blocked_no_selection,
            &texts.confirm_blocked_unresolved,
        ] {
            assert!(
                !value.trim().is_empty(),
                "every string is user-visible copy"
            );
        }
    }

    #[test]
    fn confirm_hint_reads_from_texts_for_every_block_and_is_empty_when_ready() {
        let texts = ConfirmableSearchPickerDialogTexts {
            confirm_pending: "Asignando…".to_owned(),
            confirm_blocked_no_selection: "Elija un resultado.".to_owned(),
            confirm_blocked_unresolved: "Ya no está disponible.".to_owned(),
            ..ConfirmableSearchPickerDialogTexts::default()
        };
        assert_eq!(confirm_hint_text(&texts, None), "");
        assert_eq!(
            confirm_hint_text(&texts, Some(SearchPickerConfirmBlock::NoSelection)),
            "Elija un resultado."
        );
        assert_eq!(
            confirm_hint_text(&texts, Some(SearchPickerConfirmBlock::UnresolvedSelection)),
            "Ya no está disponible."
        );
        assert_eq!(
            confirm_hint_text(&texts, Some(SearchPickerConfirmBlock::Pending)),
            "Asignando…"
        );
    }

    #[test]
    fn confirmable_search_picker_dialog_builds_with_every_state() {
        for status in [
            SearchPickerStatus::Idle,
            SearchPickerStatus::Loading,
            SearchPickerStatus::Error,
            SearchPickerStatus::Ready,
            SearchPickerStatus::Refreshing,
            SearchPickerStatus::RefreshError,
        ] {
            for pending in [false, true] {
                let _ =
                    ConfirmableSearchPickerDialog(ConfirmableSearchPickerDialogProps::<Worker> {
                        open: Signal::stored(true),
                        control_id: "assign-owner".to_string(),
                        title: Signal::stored("Assign owner".to_string()),
                        description: Signal::stored(Some("Pick one worker.".to_string())),
                        query: Signal::stored(String::new()),
                        status: Signal::stored(status),
                        items: Signal::stored(directory()),
                        selected_key: Signal::stored(Some("worker-b".to_string())),
                        pending: Signal::stored(pending),
                        on_query_change: None,
                        on_selection_change: Callback::new(|_| {}),
                        on_confirm: Callback::new(|_| {}),
                        on_close: Callback::new(|_| {}),
                        on_retry: None,
                        texts: Signal::stored(ConfirmableSearchPickerDialogTexts::default()),
                        state_texts: Signal::stored(PageStatePanelTexts::default()),
                        error_detail: Signal::stored(None),
                        confirm_error: Signal::stored(None),
                        class: "",
                        node_ref: NodeRef::new(),
                    });
            }
        }
    }
}
