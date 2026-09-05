//! Caller-owned softphone state and pure command eligibility rules.

/// A stable identity and display values for one callable number.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoftphoneNumber {
    /// Opaque, unique identity supplied by the host.
    pub id: String,
    /// Human-readable number label.
    pub label: String,
    /// Displayed telephone number; formatting is owned by the host.
    pub number: String,
}

/// Identity displayed above the call controls.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoftphoneClient {
    /// Primary client name.
    pub name: String,
    /// Supporting client information.
    pub subtitle: String,
    /// Available numbers, each requiring a unique nonempty identity.
    pub phones: Vec<SoftphoneNumber>,
}

/// Call lifecycle confirmed by the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SoftphonePhase {
    /// No call is in progress.
    #[default]
    Ready,
    /// An outgoing call is being established.
    Dialing,
    /// A call is ringing.
    Ringing,
    /// The call is connected.
    Active,
    /// The connected call is on hold.
    Held,
    /// The host is restoring a connection.
    Reconnecting,
    /// The previous call has finished.
    Ended,
}

impl SoftphonePhase {
    /// Stable lowercase phase identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Dialing => "dialing",
            Self::Ringing => "ringing",
            Self::Active => "active",
            Self::Held => "held",
            Self::Reconnecting => "reconnecting",
            Self::Ended => "ended",
        }
    }

    /// Whether a call exists that can receive an end request.
    pub fn is_live(self) -> bool {
        !matches!(self, Self::Ready | Self::Ended)
    }
}

/// Clock specification owned by the host, independent of the call phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SoftphoneTimer {
    /// No connected duration is available.
    #[default]
    NotStarted,
    /// Measure elapsed wall-clock time, including hold and reconnect time.
    Running {
        /// Connection timestamp in milliseconds since the Unix epoch.
        connected_at_ms: i64,
    },
    /// Freeze a duration supplied by the host.
    Stopped {
        /// Completed elapsed seconds.
        seconds: u64,
    },
}

impl SoftphoneTimer {
    /// Elapsed whole seconds; future connection timestamps clamp to zero.
    pub fn elapsed_at(self, now_ms: i64) -> Option<u64> {
        match self {
            Self::NotStarted => None,
            Self::Stopped { seconds } => Some(seconds),
            Self::Running { connected_at_ms } => {
                let elapsed_ms = (i128::from(now_ms) - i128::from(connected_at_ms)).max(0);
                Some((elapsed_ms / 1_000) as u64)
            }
        }
    }
}

/// Format seconds as `MM:SS`, or `H:MM:SS` from one hour onward.
pub fn format_softphone_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    if seconds < 3_600 {
        format!("{minutes:02}:{remaining_seconds:02}")
    } else {
        let hours = seconds / 3_600;
        let remaining_minutes = minutes % 60;
        format!("{hours}:{remaining_minutes:02}:{remaining_seconds:02}")
    }
}

/// Controls the host supports; mute and keypad are enabled by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoftphoneCapabilities {
    /// Supports mute requests.
    pub mute: bool,
    /// Supports hold and resume requests.
    pub hold: bool,
    /// Supports opaque voicemail routing requests.
    pub voicemail: bool,
    /// Supports recording requests.
    pub recording: bool,
    /// Supports transcription requests.
    pub transcription: bool,
    /// Supports DTMF requests.
    pub keypad: bool,
}

impl Default for SoftphoneCapabilities {
    fn default() -> Self {
        Self {
            mute: true,
            hold: false,
            voicemail: false,
            recording: false,
            transcription: false,
            keypad: true,
        }
    }
}

/// Stable action identity used to indicate a pending host acknowledgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftphoneActionKind {
    /// Choose a number.
    SelectNumber,
    /// Start a call.
    Call,
    /// End a call.
    EndCall,
    /// Change mute state.
    Mute,
    /// Change hold state.
    Hold,
    /// Route to voicemail.
    Voicemail,
    /// Change recording state.
    Record,
    /// Change transcription state.
    Transcribe,
    /// Send a DTMF digit.
    Keypad,
}

impl SoftphoneActionKind {
    /// Stable lowercase action identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelectNumber => "select-number",
            Self::Call => "call",
            Self::EndCall => "end-call",
            Self::Mute => "mute",
            Self::Hold => "hold",
            Self::Voicemail => "voicemail",
            Self::Record => "record",
            Self::Transcribe => "transcribe",
            Self::Keypad => "keypad",
        }
    }
}

/// A request only: emitting an action never confirms its success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoftphoneAction {
    /// Select the number with this stable identity.
    SelectNumber(String),
    /// Call the selected number.
    Call {
        /// Stable identity of the selected number.
        phone_id: String,
    },
    /// End the current call.
    EndCall,
    /// Request the specified mute state.
    SetMuted(bool),
    /// Request hold or resume.
    SetHeld(bool),
    /// Ask the host to perform its voicemail routing operation.
    RouteToVoicemail,
    /// Request the specified recording state.
    SetRecording(bool),
    /// Request the specified transcription state.
    SetTranscribing(bool),
    /// Send one DTMF digit from `0123456789*#`.
    SendDigit(char),
}

impl SoftphoneAction {
    /// Identity of this request for pending-state bookkeeping.
    pub fn kind(&self) -> SoftphoneActionKind {
        match self {
            Self::SelectNumber(_) => SoftphoneActionKind::SelectNumber,
            Self::Call { .. } => SoftphoneActionKind::Call,
            Self::EndCall => SoftphoneActionKind::EndCall,
            Self::SetMuted(_) => SoftphoneActionKind::Mute,
            Self::SetHeld(_) => SoftphoneActionKind::Hold,
            Self::RouteToVoicemail => SoftphoneActionKind::Voicemail,
            Self::SetRecording(_) => SoftphoneActionKind::Record,
            Self::SetTranscribing(_) => SoftphoneActionKind::Transcribe,
            Self::SendDigit(_) => SoftphoneActionKind::Keypad,
        }
    }
}

/// Request envelope allowing the host to reject stale asynchronous work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoftphoneCommand {
    /// Opaque identity of the current client/call context.
    pub context_id: String,
    /// Requested operation.
    pub action: SoftphoneAction,
}

/// Complete host-confirmed state consumed by the softphone UI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoftphoneState {
    /// Opaque context identity; blank values disable all commands.
    pub context_id: String,
    /// Client identity and phone choices.
    pub client: SoftphoneClient,
    /// Explicit selection, or automatic selection when exactly one phone exists.
    pub selected_phone_id: Option<String>,
    /// Confirmed lifecycle phase.
    pub phase: SoftphonePhase,
    /// Host-specified elapsed clock.
    pub timer: SoftphoneTimer,
    /// Confirmed mute state.
    pub muted: bool,
    /// Confirmed recording state.
    pub recording: bool,
    /// Confirmed transcription state.
    pub transcribing: bool,
    /// Supported operations.
    pub capabilities: SoftphoneCapabilities,
    /// Request awaiting host acknowledgment.
    pub pending: Option<SoftphoneActionKind>,
    /// Host-provided failure message.
    pub error: Option<String>,
}

impl SoftphoneState {
    /// Resolve a unique, valid explicit selection or the sole available number.
    pub fn selected_number(&self) -> Option<&SoftphoneNumber> {
        let id = match self.selected_phone_id.as_deref() {
            Some(id) => id,
            None if self.client.phones.len() == 1 => &self.client.phones[0].id,
            None => return None,
        };
        self.valid_number(id)
    }

    fn valid_number(&self, id: &str) -> Option<&SoftphoneNumber> {
        if id.trim().is_empty() {
            return None;
        }
        let mut matches = self.client.phones.iter().filter(|phone| phone.id == id);
        let phone = matches.next()?;
        if matches.next().is_some() || phone.number.trim().is_empty() {
            return None;
        }
        Some(phone)
    }

    /// Validate a request against the current state without mutating it.
    pub fn can_dispatch(&self, action: &SoftphoneAction) -> bool {
        if self.context_id.trim().is_empty() {
            return false;
        }
        if matches!(action, SoftphoneAction::EndCall) {
            return self.phase.is_live() && self.pending != Some(SoftphoneActionKind::EndCall);
        }
        if self.pending.is_some() {
            return false;
        }
        let connected = matches!(self.phase, SoftphonePhase::Active | SoftphonePhase::Held);
        match action {
            SoftphoneAction::SelectNumber(id) => {
                !self.phase.is_live() && self.valid_number(id).is_some()
            }
            SoftphoneAction::Call { phone_id } => {
                !self.phase.is_live()
                    && self
                        .selected_number()
                        .is_some_and(|phone| phone.id == *phone_id)
            }
            SoftphoneAction::EndCall => {
                unreachable!("end requests are handled before pending guards")
            }
            SoftphoneAction::SetMuted(muted) => {
                connected && self.capabilities.mute && *muted != self.muted
            }
            SoftphoneAction::SetHeld(held) => {
                self.capabilities.hold
                    && matches!(
                        (self.phase, held),
                        (SoftphonePhase::Active, true) | (SoftphonePhase::Held, false)
                    )
            }
            SoftphoneAction::RouteToVoicemail => {
                self.capabilities.voicemail && (connected || self.phase == SoftphonePhase::Ringing)
            }
            SoftphoneAction::SetRecording(recording) => {
                connected && self.capabilities.recording && *recording != self.recording
            }
            SoftphoneAction::SetTranscribing(transcribing) => {
                connected && self.capabilities.transcription && *transcribing != self.transcribing
            }
            SoftphoneAction::SendDigit(digit) => {
                self.phase == SoftphonePhase::Active
                    && self.capabilities.keypad
                    && matches!(digit, '0'..='9' | '*' | '#')
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(id: &str) -> SoftphoneNumber {
        SoftphoneNumber {
            id: id.into(),
            label: "Mobile".into(),
            number: "+1 555 0100".into(),
        }
    }

    fn ready() -> SoftphoneState {
        SoftphoneState {
            context_id: "client-1/call-1".into(),
            client: SoftphoneClient {
                phones: vec![number("mobile")],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn selection_requires_unique_nonblank_identity_and_number() {
        let mut state = ready();
        assert_eq!(
            state.selected_number().map(|p| p.id.as_str()),
            Some("mobile")
        );
        assert!(state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "mobile".into()
        }));
        for phones in [
            vec![],
            vec![number("")],
            vec![number(" ")],
            vec![number("mobile"), number("mobile")],
        ] {
            state.client.phones = phones;
            assert!(state.selected_number().is_none());
            assert!(!state.can_dispatch(&SoftphoneAction::SelectNumber("mobile".into())));
            assert!(!state.can_dispatch(&SoftphoneAction::Call {
                phone_id: "mobile".into()
            }));
        }
        state.client.phones = vec![number("mobile")];
        state.client.phones[0].number = "  ".into();
        assert!(state.selected_number().is_none());
        assert!(!state.can_dispatch(&SoftphoneAction::SelectNumber("mobile".into())));
    }

    #[test]
    fn multiple_numbers_require_explicit_known_selection() {
        let mut state = ready();
        state.client.phones.push(number("work"));
        assert!(state.selected_number().is_none());
        assert!(!state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "mobile".into()
        }));
        assert!(state.can_dispatch(&SoftphoneAction::SelectNumber("work".into())));
        state.selected_phone_id = Some("missing".into());
        assert!(state.selected_number().is_none());
        state.selected_phone_id = Some("work".into());
        assert_eq!(state.selected_number().unwrap().id, "work");
        assert!(state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "work".into()
        }));
        assert!(!state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "mobile".into()
        }));
        state.client.phones.push(number("work"));
        assert!(state.selected_number().is_none());
        assert!(!state.can_dispatch(&SoftphoneAction::SelectNumber("work".into())));
    }

    #[test]
    fn number_changes_and_calls_are_locked_during_live_phases() {
        let mut state = ready();
        for phase in [
            SoftphonePhase::Ready,
            SoftphonePhase::Dialing,
            SoftphonePhase::Ringing,
            SoftphonePhase::Active,
            SoftphonePhase::Held,
            SoftphonePhase::Reconnecting,
            SoftphonePhase::Ended,
        ] {
            state.phase = phase;
            let idle = matches!(phase, SoftphonePhase::Ready | SoftphonePhase::Ended);
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::SelectNumber("mobile".into())),
                idle
            );
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::Call {
                    phone_id: "mobile".into()
                }),
                idle
            );
            assert_eq!(phase.is_live(), !idle);
            assert_eq!(state.can_dispatch(&SoftphoneAction::EndCall), !idle);
        }
    }

    #[test]
    fn pending_requests_preserve_end_escape_but_prevent_duplicate_end() {
        let mut state = ready();
        state.phase = SoftphonePhase::Active;
        state.pending = Some(SoftphoneActionKind::Mute);
        assert!(state.can_dispatch(&SoftphoneAction::EndCall));
        assert!(!state.can_dispatch(&SoftphoneAction::SetMuted(true)));
        assert!(!state.can_dispatch(&SoftphoneAction::SendDigit('1')));
        state.pending = Some(SoftphoneActionKind::EndCall);
        assert!(!state.can_dispatch(&SoftphoneAction::EndCall));
        state.phase = SoftphonePhase::Ready;
        assert!(!state.can_dispatch(&SoftphoneAction::SelectNumber("mobile".into())));
        assert!(!state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "mobile".into()
        }));
    }

    #[test]
    fn commands_require_context_and_do_not_confirm_toggles() {
        let mut state = ready();
        state.phase = SoftphonePhase::Active;
        state.capabilities.recording = true;
        state.capabilities.transcription = true;
        let before = state.clone();
        for action in [
            SoftphoneAction::SetMuted(true),
            SoftphoneAction::SetRecording(true),
            SoftphoneAction::SetTranscribing(true),
        ] {
            assert!(state.can_dispatch(&action));
        }
        assert_eq!(state, before);
        for action in [
            SoftphoneAction::SetMuted(false),
            SoftphoneAction::SetRecording(false),
            SoftphoneAction::SetTranscribing(false),
        ] {
            assert!(!state.can_dispatch(&action));
        }
        state.context_id = " ".into();
        assert!(!state.can_dispatch(&SoftphoneAction::EndCall));
        assert!(!state.can_dispatch(&SoftphoneAction::SetMuted(true)));
        state.phase = SoftphonePhase::Ready;
        assert!(!state.can_dispatch(&SoftphoneAction::Call {
            phone_id: "mobile".into()
        }));
    }

    #[test]
    fn capabilities_and_phases_gate_connected_operations() {
        let mut state = ready();
        state.capabilities = SoftphoneCapabilities {
            mute: true,
            hold: true,
            voicemail: true,
            recording: true,
            transcription: true,
            keypad: true,
        };
        for phase in [
            SoftphonePhase::Ready,
            SoftphonePhase::Dialing,
            SoftphonePhase::Ringing,
            SoftphonePhase::Active,
            SoftphonePhase::Held,
            SoftphonePhase::Reconnecting,
            SoftphonePhase::Ended,
        ] {
            state.phase = phase;
            let connected = matches!(phase, SoftphonePhase::Active | SoftphonePhase::Held);
            for action in [
                SoftphoneAction::SetMuted(true),
                SoftphoneAction::SetRecording(true),
                SoftphoneAction::SetTranscribing(true),
            ] {
                assert_eq!(state.can_dispatch(&action), connected);
            }
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::SetHeld(true)),
                phase == SoftphonePhase::Active
            );
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::SetHeld(false)),
                phase == SoftphonePhase::Held
            );
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::RouteToVoicemail),
                connected || phase == SoftphonePhase::Ringing
            );
            assert_eq!(
                state.can_dispatch(&SoftphoneAction::SendDigit('1')),
                phase == SoftphonePhase::Active
            );
        }
        state.phase = SoftphonePhase::Active;
        state.capabilities = SoftphoneCapabilities {
            mute: false,
            hold: false,
            voicemail: false,
            recording: false,
            transcription: false,
            keypad: false,
        };
        for action in [
            SoftphoneAction::SetMuted(true),
            SoftphoneAction::SetHeld(true),
            SoftphoneAction::RouteToVoicemail,
            SoftphoneAction::SetRecording(true),
            SoftphoneAction::SetTranscribing(true),
            SoftphoneAction::SendDigit('1'),
        ] {
            assert!(!state.can_dispatch(&action));
        }
    }

    #[test]
    fn keypad_accepts_only_dtmf_characters() {
        let mut state = ready();
        state.phase = SoftphonePhase::Active;
        for digit in "0123456789*#".chars() {
            assert!(state.can_dispatch(&SoftphoneAction::SendDigit(digit)));
        }
        for digit in ['A', '+', ' ', '١', '\n'] {
            assert!(!state.can_dispatch(&SoftphoneAction::SendDigit(digit)));
        }
    }

    #[test]
    fn elapsed_clock_clamps_future_and_handles_extreme_timestamps() {
        assert_eq!(SoftphoneTimer::NotStarted.elapsed_at(0), None);
        assert_eq!(
            SoftphoneTimer::Stopped { seconds: 99 }.elapsed_at(i64::MAX),
            Some(99)
        );
        let clock = SoftphoneTimer::Running {
            connected_at_ms: 1_000,
        };
        assert_eq!(clock.elapsed_at(999), Some(0));
        assert_eq!(clock.elapsed_at(1_999), Some(0));
        assert_eq!(clock.elapsed_at(2_000), Some(1));
        assert_eq!(
            SoftphoneTimer::Running {
                connected_at_ms: i64::MIN
            }
            .elapsed_at(i64::MAX),
            Some(18_446_744_073_709_551)
        );
        assert_eq!(
            SoftphoneTimer::Running {
                connected_at_ms: i64::MAX
            }
            .elapsed_at(i64::MIN),
            Some(0)
        );
    }

    #[test]
    fn duration_display_handles_minute_and_hour_boundaries() {
        for (seconds, expected) in [
            (0, "00:00"),
            (59, "00:59"),
            (60, "01:00"),
            (3599, "59:59"),
            (3600, "1:00:00"),
            (3661, "1:01:01"),
            (u64::MAX, "5124095576030431:00:15"),
        ] {
            assert_eq!(format_softphone_duration(seconds), expected);
        }
    }
}
