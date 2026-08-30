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

use super::{PageStatePanel, PageStatePanelKind, PageStatePanelTexts};
use crate::components::{
    Button, ButtonStyle, Field, Input, InputType, KeyedResultList, Modal, ModalAction, ModalBox,
    ModalSearchRow, ResultListItem,
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
/// semantics (focus trap, opening focuses the search field, Escape or
/// Cancel closes and returns focus to the trigger via the dialog's own
/// native `close()` -- see [`Modal`]), state presentation, and forwarding
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

    /// Fired when the dialog should close: `Escape`, or the Cancel button.
    /// The caller owns `open`; this only requests that it become `false`.
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

    // The browser's own default action for Escape on an open modal dialog
    // is to fire a cancelable `cancel` event and, unless it is prevented, to
    // close the dialog itself -- see MDN's `HTMLDialogElement`: "cancel"
    // event. `preventDefault` here (not on the key event -- a keydown
    // `preventDefault` does NOT suppress this; only the dialog's own
    // `cancel` event does) stops that native close, so the caller's
    // controlled `open` signal -- the single source of truth `Modal` itself
    // reads -- is always what actually closes the dialog, keeping `open`
    // from desyncing from the DOM and leaving room for a future
    // confirm-before-close policy to veto the close entirely.
    let handle_dialog_cancel = move |event: web_sys::Event| {
        event.prevent_default();
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
            labelled_by=heading_id
            node_ref=node_ref
            class=class
            on:cancel=handle_dialog_cancel
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
}
