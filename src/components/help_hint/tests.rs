use super::component::{
    HelpHintEvent, HelpHintState, clear_active, help_hint_description_id, is_active, mark_active,
};

#[test]
fn hover_and_focus_are_transient_reveal_sources() {
    let mut state = HelpHintState::default();

    state.apply(HelpHintEvent::PointerEntered);
    assert!(state.visible());
    state.apply(HelpHintEvent::PointerLeft);
    assert!(!state.visible());

    state.apply(HelpHintEvent::FocusEntered);
    assert!(state.visible());
    state.apply(HelpHintEvent::FocusLeft);
    assert!(!state.visible());
}

#[test]
fn pointer_activation_pins_across_pointer_and_focus_departure() {
    let mut state = HelpHintState::default();
    state.apply(HelpHintEvent::PointerEntered);
    state.apply(HelpHintEvent::Activated { by_pointer: true });

    state.apply(HelpHintEvent::PointerLeft);
    state.apply(HelpHintEvent::FocusLeft);

    assert!(state.visible());
}

#[test]
fn keyboard_activation_pins_after_focus_reveal_but_blur_clears_it() {
    let mut state = HelpHintState::default();
    state.apply(HelpHintEvent::FocusEntered);
    state.apply(HelpHintEvent::Activated { by_pointer: false });
    assert!(state.visible());

    state.apply(HelpHintEvent::FocusLeft);

    assert!(!state.visible());
}

#[test]
fn second_activation_dismisses_until_a_new_reveal_transition() {
    let mut state = HelpHintState::default();
    state.apply(HelpHintEvent::PointerEntered);
    state.apply(HelpHintEvent::Activated { by_pointer: true });
    state.apply(HelpHintEvent::Activated { by_pointer: true });
    assert!(!state.visible());

    state.apply(HelpHintEvent::PointerLeft);
    assert!(!state.visible());
    state.apply(HelpHintEvent::PointerEntered);
    assert!(state.visible());
}

#[test]
fn escape_stays_dismissed_while_focus_or_hover_remains() {
    let mut state = HelpHintState::default();
    state.apply(HelpHintEvent::PointerEntered);
    state.apply(HelpHintEvent::FocusEntered);
    state.apply(HelpHintEvent::Dismissed);
    assert!(!state.visible());

    state.apply(HelpHintEvent::PointerLeft);
    assert!(!state.visible());
    state.apply(HelpHintEvent::FocusLeft);
    assert!(!state.visible());
}

#[test]
fn description_id_is_stable_and_caller_derivable() {
    assert_eq!(
        help_hint_description_id("billing-cycle-help"),
        "billing-cycle-help-description"
    );
}

#[test]
fn hidden_or_unmounted_active_hint_reveals_previous_visible_hint() {
    clear_active("multi-a");
    clear_active("multi-b");
    mark_active("multi-a");
    mark_active("multi-b");
    assert!(is_active("multi-b"));

    clear_active("multi-b");

    assert!(is_active("multi-a"));
    clear_active("multi-a");
}
