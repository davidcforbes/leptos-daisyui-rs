use crate::components::{Button, ButtonShape, ButtonSize, ButtonStyle, Tooltip, TooltipPosition};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure};

thread_local! {
    static ACTIVE_HELP_HINTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct HelpHintState {
    hovered: bool,
    focused: bool,
    pinned: bool,
    pinned_by_pointer: bool,
    dismissed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HelpHintEvent {
    PointerEntered,
    PointerLeft,
    FocusEntered,
    FocusLeft,
    Activated { by_pointer: bool },
    Dismissed,
}

impl HelpHintState {
    pub(super) fn apply(&mut self, event: HelpHintEvent) {
        match event {
            HelpHintEvent::PointerEntered => {
                self.hovered = true;
                self.dismissed = false;
            }
            HelpHintEvent::PointerLeft => self.hovered = false,
            HelpHintEvent::FocusEntered => {
                self.focused = true;
                self.dismissed = false;
            }
            HelpHintEvent::FocusLeft => {
                self.focused = false;
                if self.pinned && !self.pinned_by_pointer {
                    self.pinned = false;
                }
            }
            HelpHintEvent::Activated { by_pointer } => {
                if self.pinned {
                    self.pinned = false;
                    self.pinned_by_pointer = false;
                    self.dismissed = true;
                } else {
                    self.pinned = true;
                    self.pinned_by_pointer = by_pointer;
                    self.dismissed = false;
                }
            }
            HelpHintEvent::Dismissed => {
                self.pinned = false;
                self.pinned_by_pointer = false;
                self.dismissed = true;
            }
        }
    }

    pub(super) fn visible(self) -> bool {
        !self.dismissed && (self.hovered || self.focused || self.pinned)
    }
}

pub(super) fn mark_active(id: &str) {
    ACTIVE_HELP_HINTS.with_borrow_mut(|active| {
        active.retain(|candidate| candidate != id);
        active.push(id.to_string());
    });
}

pub(super) fn is_active(id: &str) -> bool {
    ACTIVE_HELP_HINTS.with_borrow(|active| active.last().is_some_and(|active| active == id))
}

pub(super) fn clear_active(id: &str) {
    ACTIVE_HELP_HINTS.with_borrow_mut(|active| active.retain(|candidate| candidate != id));
}

fn apply_and_track(state: RwSignal<HelpHintState>, id: &str, event: HelpHintEvent) {
    state.update(|state| state.apply(event));
    if state.get_untracked().visible() {
        mark_active(id);
    } else {
        clear_active(id);
    }
}

fn install_escape_listener(id: String, state: RwSignal<HelpHintState>) {
    if is_server() {
        return;
    }

    let callback_id = id.clone();
    let callback =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Escape" && is_active(&callback_id) && state.get_untracked().visible()
            {
                apply_and_track(state, &callback_id, HelpHintEvent::Dismissed);
                // Capture-phase handling prevents this Escape from reaching an unrelated
                // dialog or menu. A hidden hint never intercepts the key.
                event.stop_immediate_propagation();
            }
        });
    window()
        .add_event_listener_with_callback_and_bool(
            "keydown",
            callback.as_ref().unchecked_ref(),
            true,
        )
        .expect("register HelpHint Escape listener");
    let callback = SendWrapper::new(callback);
    on_cleanup(move || {
        window()
            .remove_event_listener_with_callback_and_bool(
                "keydown",
                callback.as_ref().unchecked_ref(),
                true,
            )
            .expect("remove HelpHint Escape listener");
    });
}

/// Derives the stable DOM id used by [`HelpHint`] for its description.
///
/// Apply the returned id to a subject's `aria-describedby` when the optional
/// subject content needs the same supplementary description as the trigger.
pub fn help_hint_description_id(id: &str) -> String {
    format!("{id}-description")
}

/// Supplementary help that is available by hover, focus, click, or tap.
///
/// `id` must be unique in the mounted document. The trigger always receives
/// the accessible `label`; `compact=true` visually reduces it to a `?` while
/// retaining that name. Optional children are rendered beside the trigger and
/// keep their ordinary focus and activation behavior. Call
/// [`help_hint_description_id`] to associate a subject with the description.
///
/// Hover and focus reveal transiently. Pointer or touch activation pins the
/// help across pointer and focus departure; keyboard activation pins until
/// focus leaves this composition. A second activation or Escape dismisses the
/// help and it stays dismissed until a new hover/focus transition. Use
/// always-visible text for essential instructions, and use [`Tooltip`] alone
/// for a short action label that does not need this disclosure lifecycle.
#[component]
pub fn HelpHint(
    /// Stable, page-unique id for the trigger.
    id: String,
    /// Accessible trigger name; visibly rendered unless `compact` is true.
    #[prop(into)]
    label: Signal<String>,
    /// Supplementary description rendered as a real `role="tooltip"` node.
    #[prop(into)]
    text: Signal<String>,
    /// Optional interactive or non-interactive subject beside the trigger.
    #[prop(optional)]
    children: Option<Children>,
    /// Render an icon-style `?` trigger while retaining `label` for AT.
    #[prop(default = false)]
    compact: bool,
) -> impl IntoView {
    let state = RwSignal::new(HelpHintState::default());
    let description_id = help_hint_description_id(&id);
    let root_attr_id = id.clone();
    let trigger_id = id.clone();
    let pointer_id = id.clone();
    let pointer_leave_id = id.clone();
    let focus_id = id.clone();
    let focus_out_id = id.clone();
    let click_id = id.clone();
    let cleanup_id = id.clone();

    install_escape_listener(id.clone(), state);
    on_cleanup(move || clear_active(&cleanup_id));

    view! {
        <div
            class="inline-flex max-w-full items-center gap-1"
            data-help-hint=root_attr_id
            data-help-hint-open=move || state.get().visible().to_string()
            on:pointerenter={
                let id = pointer_id;
                move |_| {
                    apply_and_track(state, &id, HelpHintEvent::PointerEntered);
                }
            }
            on:pointerleave={
                let id = pointer_leave_id;
                move |_| apply_and_track(state, &id, HelpHintEvent::PointerLeft)
            }
            on:focusin={
                let id = focus_id;
                move |event: web_sys::FocusEvent| {
                    let came_from_inside = event
                        .current_target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .zip(
                            event
                                .related_target()
                                .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
                        )
                        .is_some_and(|(root, previous)| root.contains(Some(&previous)));
                    if !came_from_inside {
                        apply_and_track(state, &id, HelpHintEvent::FocusEntered);
                    }
                }
            }
            on:focusout={
                let id = focus_out_id;
                move |event: web_sys::FocusEvent| {
                let remains_inside = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .zip(
                        event
                            .related_target()
                            .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
                    )
                    .is_some_and(|(root, next)| root.contains(Some(&next)));
                if !remains_inside {
                    apply_and_track(state, &id, HelpHintEvent::FocusLeft);
                }
            }
            }
        >
            <Tooltip
                position=TooltipPosition::Bottom
                open=Signal::derive(move || state.get().visible())
                class="before:hidden after:hidden"
            >
                {children.map(|children| children())}
                <Button
                    style=ButtonStyle::Ghost
                    size=ButtonSize::Sm
                    shape=if compact { ButtonShape::Square } else { ButtonShape::Default }
                    attr:id=trigger_id
                    attr:data-help-hint-trigger="true"
                    attr:aria-describedby=description_id.clone()
                    attr:aria-expanded=move || state.get().visible().to_string()
                    on_click=Callback::new({
                        let id = click_id;
                        move |event: web_sys::MouseEvent| {
                            apply_and_track(
                                state,
                                &id,
                                HelpHintEvent::Activated {
                                    by_pointer: event.detail() > 0,
                                },
                            );
                        }
                    })
                >
                    <span class=if compact { "sr-only" } else { "" }>{move || label.get()}</span>
                    {compact.then(|| view! { <span aria-hidden="true">"?"</span> })}
                </Button>
                <span
                    id=description_id
                    role="tooltip"
                    class="tooltip-content pointer-events-auto text-left whitespace-normal"
                    style:opacity=move || if state.get().visible() { "1" } else { "0" }
                    style:visibility=move || if state.get().visible() { "visible" } else { "hidden" }
                    style:pointer-events=move || if state.get().visible() { "auto" } else { "none" }
                    style:top="100%"
                    style:right="auto"
                    style:left="0"
                    style:max-width="min(20rem, calc(100vw - 2rem))"
                    style:transform="translateY(0)"
                >
                    {move || text.get()}
                </span>
            </Tooltip>
        </div>
    }
}
