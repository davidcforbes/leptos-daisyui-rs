//! Accessible names and disabled reasons for every workspace control.
//!
//! One closed control inventory and three pure functions — [`ClientCallWorkspaceState::control_enabled`],
//! [`ClientCallWorkspaceTexts::control_name`] and [`ClientCallWorkspaceTexts::disabled_reason`] —
//! that the render path uses for the `disabled` state, the `aria-label` and the visible
//! reason bound by `aria-describedby`. Because the component reads the same functions,
//! the tests over them are tests of what the rendered controls announce (Office op-stjm9:
//! nine disabled call-panel controls announced no name and no reason).

use super::{
    ClientCallAction, ClientCallWorkspaceState, ClientCallWorkspaceTexts, ClientCallWrapUp,
};

/// Every interactive control the workspace itself renders (the nested live
/// `Softphone` console owns its own controls).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientCallControl {
    /// Header dismissal proposal.
    Dismiss,
    /// Destination number input.
    Destination,
    /// One saved-number choice, by phone identity.
    SavedNumber(String),
    /// Number pad disclosure.
    Keypad,
    /// One number pad digit key.
    Digit(char),
    /// Number pad backspace.
    Backspace,
    /// Contact field chooser for the explicit contact write.
    NumberTarget,
    /// Explicit contact write.
    SaveNumber,
    /// Launch the call.
    Dial,
    /// Script regeneration request.
    Regenerate,
    /// Wrap-up outcome select.
    Outcome,
    /// Wrap-up notes.
    Notes,
    /// Wrap-up manual duration.
    Duration,
    /// Wrap-up callback instructions.
    FollowUp,
    /// Save the call record.
    SaveWrapUp,
}

impl ClientCallControl {
    /// The number pad keys, in render order.
    pub const DIGITS: [char; 12] = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '*', '0', '#'];

    /// The controls rendered for `state`, in document order (the destination
    /// block is replaced by the live console during a managed live call; the
    /// number pad is listed whether or not it is expanded).
    pub fn inventory(state: &ClientCallWorkspaceState) -> Vec<ClientCallControl> {
        let mut controls = vec![ClientCallControl::Dismiss];
        let console_live =
            state.attempt == super::ClientCallAttempt::Managed && state.call.phase.is_live();
        if !console_live {
            controls.push(ClientCallControl::Destination);
            controls.extend(
                state
                    .call
                    .client
                    .phones
                    .iter()
                    .map(|phone| ClientCallControl::SavedNumber(phone.id.clone())),
            );
            controls.push(ClientCallControl::Keypad);
            controls.extend(Self::DIGITS.into_iter().map(ClientCallControl::Digit));
            controls.push(ClientCallControl::Backspace);
            if let Some(update) = &state.number_update {
                if !update.targets.is_empty() {
                    controls.push(ClientCallControl::NumberTarget);
                }
                controls.push(ClientCallControl::SaveNumber);
            }
            controls.push(ClientCallControl::Dial);
        }
        if state
            .guidance
            .as_ref()
            .is_some_and(|g| g.regeneration.is_some())
        {
            controls.push(ClientCallControl::Regenerate);
        }
        if let Some(wrap) = &state.wrap_up {
            controls.extend([
                ClientCallControl::Outcome,
                ClientCallControl::Notes,
                ClientCallControl::Duration,
            ]);
            if wrap.outcome == Some(super::ClientCallOutcome::RequestedCallBack) {
                controls.push(ClientCallControl::FollowUp);
            }
            controls.push(ClientCallControl::SaveWrapUp);
        }
        controls
    }
}

impl ClientCallWorkspaceState {
    /// Whether `control` is operable now. The render path's `disabled` is its negation.
    pub fn control_enabled(&self, control: &ClientCallControl) -> bool {
        match control {
            ClientCallControl::Dismiss => self.can_dispatch(&ClientCallAction::Dismiss),
            ClientCallControl::Destination | ClientCallControl::Keypad => {
                self.can_edit_destination()
            }
            ClientCallControl::SavedNumber(id) => {
                self.can_dispatch(&ClientCallAction::ChooseSavedNumber(id.clone()))
            }
            ClientCallControl::Digit(_) => {
                self.can_edit_destination() && self.destination.chars().count() < 64
            }
            ClientCallControl::Backspace => {
                self.can_edit_destination() && !self.destination.is_empty()
            }
            ClientCallControl::NumberTarget => {
                self.can_edit_destination()
                    && self
                        .number_update
                        .as_ref()
                        .is_some_and(|u| u.blocked_reason.is_none())
            }
            ClientCallControl::SaveNumber => self.number_update.as_ref().is_some_and(|u| {
                self.can_dispatch(&ClientCallAction::SaveNumber {
                    field_id: u.field_id.clone(),
                    number: self.destination.clone(),
                })
            }),
            ClientCallControl::Dial => self.can_dispatch(&ClientCallAction::Dial {
                number: self.destination.clone(),
            }),
            ClientCallControl::Regenerate => {
                self.can_dispatch(&ClientCallAction::RegenerateGuidance)
            }
            ClientCallControl::Outcome
            | ClientCallControl::Notes
            | ClientCallControl::Duration
            | ClientCallControl::FollowUp => self.can_dispatch(&ClientCallAction::SetOutcome(None)),
            ClientCallControl::SaveWrapUp => self
                .wrap_up
                .as_ref()
                .and_then(ClientCallWrapUp::payload)
                .is_some_and(|payload| self.can_dispatch(&ClientCallAction::SaveWrapUp(payload))),
        }
    }

    /// No call context has been minted by the host, so no control can act.
    pub fn context_missing(&self) -> bool {
        self.call.context_id.trim().is_empty()
    }
}

fn some_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

impl ClientCallWorkspaceTexts {
    /// The accessible name of `control`; equals its visible label so the name
    /// contains the visible text (WCAG 2.5.3).
    pub fn control_name(
        &self,
        control: &ClientCallControl,
        state: &ClientCallWorkspaceState,
    ) -> String {
        match control {
            ClientCallControl::Dismiss => self.close.clone(),
            ClientCallControl::Destination => self.destination.clone(),
            ClientCallControl::SavedNumber(id) => state
                .call
                .client
                .phones
                .iter()
                .find(|phone| phone.id == *id)
                .map(|phone| format!("{} · {}", phone.label, phone.number))
                .filter(|name| !name.trim_matches([' ', '·']).is_empty())
                .unwrap_or_else(|| self.saved_numbers.clone()),
            ClientCallControl::Keypad => self.keypad.clone(),
            ClientCallControl::Digit(digit) => digit.to_string(),
            ClientCallControl::Backspace => self.backspace.clone(),
            ClientCallControl::NumberTarget => self.number_target.clone(),
            ClientCallControl::SaveNumber => {
                if state.number_update.as_ref().is_some_and(|u| u.pending) {
                    self.saving.clone()
                } else {
                    self.save_number.clone()
                }
            }
            ClientCallControl::Dial => self.call.clone(),
            ClientCallControl::Regenerate => self.regenerate.clone(),
            ClientCallControl::Outcome => self.outcome.clone(),
            ClientCallControl::Notes => self.notes.clone(),
            ClientCallControl::Duration => self.duration.clone(),
            ClientCallControl::FollowUp => self.follow_up.clone(),
            ClientCallControl::SaveWrapUp => {
                let draft = state.wrap_up.clone().unwrap_or_default();
                if draft.saved {
                    self.saved.clone()
                } else if draft.pending {
                    self.saving.clone()
                } else {
                    self.save_wrap_up.clone()
                }
            }
        }
    }

    /// Why `control` is disabled, or `None` while it is operable. A disabled
    /// control always gets a reason: host-supplied explanations first, then
    /// this crate's copy.
    pub fn disabled_reason(
        &self,
        control: &ClientCallControl,
        state: &ClientCallWorkspaceState,
    ) -> Option<String> {
        if state.control_enabled(control) {
            return None;
        }
        if state.context_missing() {
            return Some(self.not_ready.clone());
        }
        let locked = |s: &ClientCallWorkspaceState| {
            (!s.can_edit_destination()).then(|| self.destination_locked.clone())
        };
        let reason = match control {
            // Dismiss is enabled whenever a context exists.
            ClientCallControl::Dismiss => None,
            ClientCallControl::Destination | ClientCallControl::Keypad => locked(state),
            ClientCallControl::SavedNumber(id) => locked(state).or_else(|| {
                state
                    .call
                    .client
                    .phones
                    .iter()
                    .find(|phone| phone.id == *id)
                    .and_then(|phone| phone.blocked_reason.clone())
            }),
            ClientCallControl::Digit(_) => {
                locked(state).or_else(|| Some(self.destination_full.clone()))
            }
            ClientCallControl::Backspace => {
                locked(state).or_else(|| Some(self.needs_number.clone()))
            }
            ClientCallControl::NumberTarget => state
                .number_update
                .as_ref()
                .and_then(|u| u.blocked_reason.clone())
                .or_else(|| locked(state)),
            ClientCallControl::SaveNumber => {
                let update = state.number_update.as_ref();
                update
                    .and_then(|u| u.blocked_reason.clone())
                    .or_else(|| {
                        update
                            .and_then(|u| u.selected_target())
                            .and_then(|t| t.blocked_reason)
                    })
                    .or_else(|| locked(state))
                    .or_else(|| state.number_error.clone())
                    .or_else(|| {
                        state
                            .destination
                            .trim()
                            .is_empty()
                            .then(|| self.needs_number.clone())
                    })
                    .or_else(|| Some(self.save_number_hint.clone()))
            }
            ClientCallControl::Dial => locked(state)
                .or_else(|| state.dial_blocked_reason.clone())
                .or_else(|| state.number_error.clone())
                .or_else(|| {
                    state
                        .call
                        .client
                        .phones
                        .iter()
                        .find(|phone| phone.number.trim() == state.destination.trim())
                        .and_then(|phone| phone.blocked_reason.clone())
                }),
            ClientCallControl::Regenerate => state
                .guidance
                .as_ref()
                .and_then(|g| g.regeneration.clone())
                .map(|r| {
                    r.blocked_reason
                        .unwrap_or_else(|| self.regeneration(r.state))
                }),
            ClientCallControl::Outcome
            | ClientCallControl::Notes
            | ClientCallControl::Duration
            | ClientCallControl::FollowUp => Some(self.wrap_up_lock_reason(state)),
            ClientCallControl::SaveWrapUp => {
                let wrap = state.wrap_up.clone().unwrap_or_default();
                if wrap.pending || wrap.saved {
                    Some(self.wrap_up_lock_reason(state))
                } else if !state.finished() {
                    Some(self.finish_hint.clone())
                } else {
                    Some(self.wrap_up_incomplete.clone())
                }
            }
        };
        reason
            .and_then(|r| some_text(&r))
            .or_else(|| {
                some_text(&self.needs_number).filter(|_| {
                    matches!(
                        control,
                        ClientCallControl::Dial | ClientCallControl::SavedNumber(_)
                    )
                })
            })
            .or_else(|| some_text(&self.not_ready))
    }

    fn wrap_up_lock_reason(&self, state: &ClientCallWorkspaceState) -> String {
        if state.wrap_up.as_ref().is_some_and(|w| w.pending) {
            self.saving.clone()
        } else {
            self.wrap_up_locked.clone()
        }
    }
}
