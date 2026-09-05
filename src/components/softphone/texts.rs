use super::{SoftphoneActionKind, SoftphonePhase, SoftphoneState};

/// Localized copy for every framework-owned softphone label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoftphoneTexts {
    /// Accessible region name.
    pub label: String,
    /// Number field label.
    pub phone_number: String,
    /// Placeholder when no number is selected.
    pub choose_number: String,
    /// Empty phone list feedback.
    pub no_number: String,
    /// Elapsed timer accessible label.
    pub duration: String,
    /// Timer before connection.
    pub not_started: String,
    /// Primary dial label.
    pub call: String,
    /// Termination label.
    pub end_call: String,
    /// Local microphone suppression label.
    pub mute: String,
    /// Restore microphone label.
    pub unmute: String,
    /// Hold request label.
    pub hold: String,
    /// Resume request label.
    pub resume: String,
    /// Voicemail routing request label.
    pub voicemail: String,
    /// Start recording label.
    pub record: String,
    /// Stop recording label.
    pub stop_recording: String,
    /// Start transcription label.
    pub transcribe: String,
    /// Stop transcription label.
    pub stop_transcribing: String,
    /// Keypad toggle and group label.
    pub keypad: String,
    /// Keypad digit name template, with `{digit}`.
    pub digit: String,
    /// Prefix for a command awaiting caller confirmation.
    pub pending: String,
    /// Confirmed recording indicator.
    pub recording: String,
    /// Confirmed transcription indicator.
    pub transcribing: String,
    /// Ready phase label.
    pub ready: String,
    /// Dialing phase label.
    pub dialing: String,
    /// Ringing phase label.
    pub ringing: String,
    /// Connected phase label.
    pub active: String,
    /// Held phase label.
    pub held: String,
    /// Reconnection phase label.
    pub reconnecting: String,
    /// Ended phase label.
    pub ended: String,
}

impl Default for SoftphoneTexts {
    fn default() -> Self {
        Self {
            label: "Client call".into(),
            phone_number: "Phone number".into(),
            choose_number: "Choose a number".into(),
            no_number: "No phone number available".into(),
            duration: "Call duration".into(),
            not_started: "--:--".into(),
            call: "Call".into(),
            end_call: "End call".into(),
            mute: "Mute".into(),
            unmute: "Unmute".into(),
            hold: "Hold".into(),
            resume: "Resume".into(),
            voicemail: "Route to voicemail".into(),
            record: "Record".into(),
            stop_recording: "Stop recording".into(),
            transcribe: "Transcribe".into(),
            stop_transcribing: "Stop transcription".into(),
            keypad: "Keypad".into(),
            digit: "Send {digit}".into(),
            pending: "Waiting for confirmation".into(),
            recording: "Recording".into(),
            transcribing: "Transcribing".into(),
            ready: "Ready to call".into(),
            dialing: "Dialing".into(),
            ringing: "Ringing".into(),
            active: "In call".into(),
            held: "On hold".into(),
            reconnecting: "Reconnecting".into(),
            ended: "Call ended".into(),
        }
    }
}

impl SoftphoneTexts {
    pub(crate) fn phase(&self, phase: SoftphonePhase) -> String {
        match phase {
            SoftphonePhase::Ready => &self.ready,
            SoftphonePhase::Dialing => &self.dialing,
            SoftphonePhase::Ringing => &self.ringing,
            SoftphonePhase::Active => &self.active,
            SoftphonePhase::Held => &self.held,
            SoftphonePhase::Reconnecting => &self.reconnecting,
            SoftphonePhase::Ended => &self.ended,
        }
        .clone()
    }

    pub(crate) fn action(&self, kind: SoftphoneActionKind, state: &SoftphoneState) -> String {
        match kind {
            SoftphoneActionKind::SelectNumber => &self.phone_number,
            SoftphoneActionKind::Call => &self.call,
            SoftphoneActionKind::EndCall => &self.end_call,
            SoftphoneActionKind::Mute => {
                if state.muted {
                    &self.unmute
                } else {
                    &self.mute
                }
            }
            SoftphoneActionKind::Hold => {
                if state.phase == SoftphonePhase::Held {
                    &self.resume
                } else {
                    &self.hold
                }
            }
            SoftphoneActionKind::Voicemail => &self.voicemail,
            SoftphoneActionKind::Record => {
                if state.recording {
                    &self.stop_recording
                } else {
                    &self.record
                }
            }
            SoftphoneActionKind::Transcribe => {
                if state.transcribing {
                    &self.stop_transcribing
                } else {
                    &self.transcribe
                }
            }
            SoftphoneActionKind::Keypad => &self.keypad,
        }
        .clone()
    }
}
