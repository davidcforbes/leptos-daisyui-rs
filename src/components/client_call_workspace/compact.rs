//! The compact layout's derived lines (ldui-eq1e, Office op-flpq1).
//!
//! The owner measured the full panel in a 339 px side rail: 26 text blocks, an
//! h2 wrapping to four lines and `destination_locked` rendered SIX times. The
//! compact layout says each thing once. Every line it renders comes from a
//! function here, and the render path reads the same functions, so the tests
//! over them are tests of what the panel shows (the contract [`super::a11y`]
//! keeps for names and reasons).

use super::a11y::some_text;
use super::{
    ClientCallAction, ClientCallAttempt, ClientCallControl, ClientCallGuidance,
    ClientCallGuidanceState, ClientCallLayout, ClientCallWorkspaceState, ClientCallWorkspaceTexts,
};
use crate::components::{SoftphoneTimer, format_softphone_duration};

/// Recent calls the compact layout shows before "Show all".
pub const COMPACT_HISTORY_VISIBLE: usize = 3;

impl ClientCallControl {
    /// The compact launcher's call button: "Call again" after a finished
    /// attempt proposes a new one; otherwise it is the ordinary launch.
    pub fn compact_call(state: &ClientCallWorkspaceState) -> Self {
        if state.saved_only() && state.finished() {
            Self::StartAnotherCall
        } else {
            Self::Dial
        }
    }

    /// The controls `layout` renders for `state`, in document order. Compact
    /// renders the full panel's controls, except that the launcher's call
    /// button becomes [`ClientCallControl::StartAnotherCall`] after a finished
    /// attempt.
    pub fn inventory_for(
        state: &ClientCallWorkspaceState,
        layout: ClientCallLayout,
    ) -> Vec<ClientCallControl> {
        let controls = Self::inventory(state);
        if layout != ClientCallLayout::Compact {
            return controls;
        }
        let call = Self::compact_call(state);
        controls
            .into_iter()
            .map(|control| {
                if control == Self::Dial {
                    call.clone()
                } else {
                    control
                }
            })
            .collect()
    }
}

impl ClientCallWorkspaceState {
    /// The saved numbers are locked to this attempt (a call in progress or
    /// finished, a record saving) while a call context exists. With no context
    /// the shared context line says why instead.
    pub fn saved_numbers_locked(&self) -> bool {
        !self.context_missing() && !self.can_edit_destination()
    }

    /// The compact status dot's daisyUI classes. Decorative: the line's text
    /// carries the meaning.
    pub fn compact_status_tone(&self) -> &'static str {
        match self.attempt {
            ClientCallAttempt::Submitting | ClientCallAttempt::AgentRinging => "status status-info",
            ClientCallAttempt::Uncertain => "status status-warning",
            ClientCallAttempt::Refused | ClientCallAttempt::RefusedWith(_) => "status status-error",
            ClientCallAttempt::Managed if self.call.phase.is_live() => "status status-success",
            _ => "status status-neutral",
        }
    }
}

impl ClientCallWorkspaceTexts {
    /// The accessible name of `control` as `layout` renders it; equals its
    /// visible label (WCAG 2.5.3).
    pub fn control_name_for(
        &self,
        control: &ClientCallControl,
        state: &ClientCallWorkspaceState,
        layout: ClientCallLayout,
    ) -> String {
        if layout == ClientCallLayout::Compact
            && state.saved_only()
            && *control == ClientCallControl::Dial
        {
            self.compact_call_label(state)
        } else {
            self.control_name(control, state)
        }
    }

    /// The compact call button's label: "Call again" after a finished or a
    /// refused attempt, otherwise the ordinary launch label.
    pub fn compact_call_label(&self, state: &ClientCallWorkspaceState) -> String {
        if state.finished()
            || matches!(
                state.attempt,
                ClientCallAttempt::Refused | ClientCallAttempt::RefusedWith(_)
            )
        {
            self.call_again.clone()
        } else {
            self.call.clone()
        }
    }

    /// The one status line, or `None` before any attempt with nothing to
    /// explain (the full layout's op-fg6s2 rule). The host's `status_detail`
    /// beats the attempt label; then the final duration of a finished call and
    /// the provider talk time, joined by " · ". The bridge hint is dropped;
    /// the uncertain warning is [`Self::compact_uncertain_hint`].
    pub fn compact_status(&self, state: &ClientCallWorkspaceState) -> Option<String> {
        let detail = state.status_detail.trim();
        if state.attempt == ClientCallAttempt::Ready && detail.is_empty() {
            return None;
        }
        let primary = if !detail.is_empty() {
            detail.to_owned()
        } else if state.attempt == ClientCallAttempt::Managed {
            self.softphone.phase(state.call.phase)
        } else {
            self.attempt(state.attempt)
        };
        let mut parts = vec![primary];
        if let (true, SoftphoneTimer::Stopped { seconds }) = (state.finished(), state.call.timer) {
            parts.push(format_softphone_duration(seconds));
        }
        // The talk-time caveat travels with a call that happened (op-fg6s2);
        // absent is unknown, never zero.
        if state.provider_talk_seconds.is_some()
            || matches!(
                state.attempt,
                ClientCallAttempt::Managed | ClientCallAttempt::Finished
            )
        {
            let talk = state
                .provider_talk_seconds
                .map(format_softphone_duration)
                .unwrap_or_else(|| self.provider_talk_unknown.clone());
            parts.push(format!("{}: {}", self.history_talk_time, talk));
        }
        Some(parts.join(" · "))
    }

    /// The safety warning kept under the status line while dispatch is
    /// uncertain.
    pub fn compact_uncertain_hint(&self, state: &ClientCallWorkspaceState) -> Option<String> {
        (state.attempt == ClientCallAttempt::Uncertain)
            .then(|| some_text(&self.uncertain_hint))
            .flatten()
    }

    /// The ONE line under the saved numbers while they are locked. Per-number
    /// lock lines do not exist in the compact layout. After a finished call,
    /// when Call again is the next step, it says so (`numbers_locked_after_call`)
    /// rather than "during a call"; a call in progress or a pending save keeps
    /// `numbers_locked`.
    pub fn compact_lock_line(&self, state: &ClientCallWorkspaceState) -> Option<String> {
        if !(state.saved_only() && state.saved_numbers_locked()) {
            return None;
        }
        let during =
            || some_text(&self.numbers_locked).or_else(|| some_text(&self.destination_locked));
        if state.can_dispatch(&ClientCallAction::StartAnotherCall) {
            some_text(&self.numbers_locked_after_call).or_else(during)
        } else {
            during()
        }
    }

    /// A saved number's own inline line: its host blocked reason, never the
    /// lock (which [`Self::compact_lock_line`] states once).
    pub fn compact_number_line(
        &self,
        state: &ClientCallWorkspaceState,
        id: &str,
    ) -> Option<String> {
        state
            .call
            .client
            .phones
            .iter()
            .find(|phone| phone.id == id)
            .and_then(|phone| phone.blocked_reason.as_deref().and_then(some_text))
    }

    /// Why the compact call button is disabled (its hint, description and
    /// title), or `None` while it is operable. A disabled launch whose cause
    /// is the lock says what the shared lock line says.
    pub fn compact_call_reason(&self, state: &ClientCallWorkspaceState) -> Option<String> {
        let control = ClientCallControl::compact_call(state);
        let reason = self.disabled_reason(&control, state)?;
        if control == ClientCallControl::Dial
            && let Some(lock) = self.compact_lock_line(state)
        {
            return Some(lock);
        }
        Some(reason)
    }

    /// The visible line under the compact call button. Empty while the button
    /// is operable, while the shared context line explains it, and while the
    /// shared lock line does: any other reason (the host's dial-blocked
    /// reason, "choose a saved number") shows here once.
    pub fn compact_call_line(&self, state: &ClientCallWorkspaceState) -> Option<String> {
        let control = ClientCallControl::compact_call(state);
        if state.context_missing()
            || (control == ClientCallControl::Dial && self.compact_lock_line(state).is_some())
        {
            return None;
        }
        self.disabled_reason(&control, state)
    }

    /// The host line under the compact call button; blank hides it.
    pub fn compact_launch_hint(&self) -> Option<String> {
        some_text(&self.launch_hint)
    }

    /// The script disclosure's summary: the guidance title (or `script` when
    /// blank), plus the preparation state while it is not ready, so a closed
    /// disclosure still says the script is coming or failed.
    pub fn compact_script_summary(&self, guidance: &ClientCallGuidance) -> String {
        let title = some_text(&guidance.title).unwrap_or_else(|| self.script.clone());
        match guidance.state {
            ClientCallGuidanceState::Ready => title,
            ClientCallGuidanceState::Preparing { .. } => {
                format!("{title} · {}", self.script_preparing)
            }
            ClientCallGuidanceState::Failed { .. } => format!("{title} · {}", self.script_failed),
        }
    }

    /// The recent-calls toggle: "Show all (n)" while collapsed, "Show fewer"
    /// while expanded.
    pub fn history_toggle(&self, expanded: bool, total: usize) -> String {
        if expanded {
            self.history_show_fewer.clone()
        } else {
            self.history_show_all.replace("{n}", &total.to_string())
        }
    }
}
