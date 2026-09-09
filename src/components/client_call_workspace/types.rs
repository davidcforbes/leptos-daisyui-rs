use crate::components::{SoftphoneAction, SoftphonePhase, SoftphoneState};

/// Retry-safe refusal evidence explicitly confirmed by the host.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientCallRefusal {
    /// Local preflight found the agent not ready; no provider POST was attempted.
    AgentNotReady,
    /// The provider received the request and refused it because the agent was unavailable.
    AgentNotAvailable,
    /// The host confirmed that no call was submitted.
    NotSubmitted,
}

/// Evidence about this attempt, independently of any provider's media session.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClientCallAttempt {
    /// No dispatch has occurred; the host may authorize a launch.
    #[default]
    Ready,
    /// Dispatch has been reserved; duplicate launch is blocked.
    Submitting,
    /// A bridge launch was accepted and the agent's telephone is ringing.
    AgentRinging,
    /// Dispatch may have occurred. Only host reconciliation can unlock it.
    Uncertain,
    /// Explicit evidence establishes that retry is safe, subject to readiness.
    Refused,
    /// Structured retry-safe refusal; unknown dispatch results remain uncertain.
    RefusedWith(ClientCallRefusal),
    /// The host confirmed that this attempt is finished.
    Finished,
    /// The host supplies a managed session through the embedded softphone state.
    Managed,
}

impl ClientCallAttempt {
    /// Stable DOM and diagnostic identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Submitting => "submitting",
            Self::AgentRinging => "agent-ringing",
            Self::Uncertain => "uncertain",
            Self::Refused => "refused",
            Self::RefusedWith(ClientCallRefusal::AgentNotReady) => "agent-not-ready",
            Self::RefusedWith(ClientCallRefusal::AgentNotAvailable) => "agent-not-available",
            Self::RefusedWith(ClientCallRefusal::NotSubmitted) => "not-submitted",
            Self::Finished => "finished",
            Self::Managed => "managed",
        }
    }
}

/// Deliberate work result; no outcome is selected by default.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientCallOutcome {
    /// Client is interested.
    Interested,
    /// Client is not interested.
    NotInterested,
    /// No answer or a busy destination.
    NoResponseBusy,
    /// Client requested additional information.
    RequestedMoreInfo,
    /// Client requested a callback; follow-up instructions are required.
    RequestedCallBack,
    /// The destination was invalid.
    InvalidNumber,
}

impl ClientCallOutcome {
    /// Ordered outcome vocabulary displayed by the workspace.
    pub const ALL: [Self; 6] = [
        Self::Interested,
        Self::NotInterested,
        Self::NoResponseBusy,
        Self::RequestedMoreInfo,
        Self::RequestedCallBack,
        Self::InvalidNumber,
    ];

    /// Stable value for transport adapters and the native select.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Interested => "interested",
            Self::NotInterested => "not-interested",
            Self::NoResponseBusy => "no-response-busy",
            Self::RequestedMoreInfo => "requested-more-info",
            Self::RequestedCallBack => "requested-call-back",
            Self::InvalidNumber => "invalid-number",
        }
    }

    /// Resolve a known value without inventing a fallback outcome.
    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|outcome| outcome.as_str() == value)
    }
}

/// One host-authorized target for an explicit contact-number write.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallNumberTarget {
    /// Opaque unique field identity, scoped to the current contact by the host.
    pub field_id: String,
    /// Visible contact field name.
    pub label: String,
    /// Last host-confirmed field value, independent of the destination draft.
    pub saved_number: String,
    /// Present when this field cannot be written, regardless of dial permission.
    pub blocked_reason: Option<String>,
}

/// Explicit contact-write capability. Absence hides contact-write controls.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallNumberUpdate {
    /// Controlled selected target identity, resolved to the contact by the host.
    pub field_id: String,
    /// Available targets; empty retains the legacy single-field properties below.
    pub targets: Vec<ClientCallNumberTarget>,
    /// Human-readable field name shown beside the write action.
    pub label: String,
    /// Last server-confirmed value; editing a destination never changes it.
    pub saved_number: String,
    /// Present when temporarily forbidden; provide an actionable explanation.
    pub blocked_reason: Option<String>,
    /// The host has reserved an update request.
    pub pending: bool,
    /// Server failure; retains the draft and the confirmed saved value.
    pub error: Option<String>,
}

impl ClientCallNumberUpdate {
    /// Resolve the controlled selection; missing, blank or duplicate IDs fail closed.
    pub fn selected_target(&self) -> Option<ClientCallNumberTarget> {
        self.target(&self.field_id)
    }

    fn target(&self, field_id: &str) -> Option<ClientCallNumberTarget> {
        if field_id.trim().is_empty() {
            return None;
        }
        if self.targets.is_empty() {
            return (field_id == self.field_id).then(|| ClientCallNumberTarget {
                field_id: self.field_id.clone(),
                label: self.label.clone(),
                saved_number: self.saved_number.clone(),
                blocked_reason: self.blocked_reason.clone(),
            });
        }
        let mut matches = self
            .targets
            .iter()
            .filter(|target| target.field_id == field_id);
        let target = matches.next()?;
        matches.next().is_none().then(|| target.clone())
    }

    /// Whether this capability allows choosing a unique writable target now.
    pub fn can_choose_target(&self, field_id: &str) -> bool {
        !self.pending
            && self.blocked_reason.is_none()
            && self
                .target(field_id)
                .is_some_and(|target| target.blocked_reason.is_none())
    }
}

/// A reviewed script step; all content remains plain text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallScriptBeat {
    /// Stable step identity supplied by the host.
    pub id: String,
    /// Step heading.
    pub title: String,
    /// Suggested words for the operator to say.
    pub say: String,
    /// The host has no file-specific detail for this step.
    pub no_file_detail: bool,
}

/// Host-confirmed preparation state, separate from script regeneration requests.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ClientCallGuidanceState {
    /// Guidance is available.
    #[default]
    Ready,
    /// Host-supplied preparation progress; this does not start a local clock.
    Preparing {
        /// Elapsed preparation seconds measured by the host.
        elapsed_secs: u64,
        /// Typical preparation seconds supplied by the host, not a deadline.
        typical_secs: u64,
    },
    /// Preparation failed; any previous guidance remains host-owned.
    Failed {
        /// Safe, actionable explanation.
        reason: String,
    },
}

/// Host acknowledgment of the independent script-regeneration operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClientCallRegenerationState {
    /// No request is pending.
    #[default]
    Idle,
    /// The host reserved a request and is submitting it.
    Busy,
    /// A request was accepted; completion is not yet established.
    Accepted,
    /// The host confirmed regeneration completed.
    Succeeded,
    /// The host confirmed regeneration failed and permits another request.
    Failed,
}

impl ClientCallRegenerationState {
    /// Stable DOM and diagnostic identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Busy => "busy",
            Self::Accepted => "accepted",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// Optional capability and acknowledgment for explicit script regeneration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallRegeneration {
    /// Host-confirmed operation state.
    pub state: ClientCallRegenerationState,
    /// Present when the operation is unavailable; absence permits eligible requests.
    pub blocked_reason: Option<String>,
    /// Safe host failure explanation, independent of the retained guidance.
    pub error: Option<String>,
}

/// Optional plain-text call objective or reviewed script, with provenance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallGuidance {
    /// Heading describing the guidance.
    pub title: String,
    /// Plain text, never interpreted as HTML or a command.
    pub body: String,
    /// Visible origin or review label; blank values omit the source line.
    pub source: String,
    /// Ordered reviewed script steps, supplementing the plain title and body.
    pub beats: Vec<ClientCallScriptBeat>,
    /// Preparation evidence projected by the host.
    pub state: ClientCallGuidanceState,
    /// Optional explicit regeneration capability; absence hides its action.
    pub regeneration: Option<ClientCallRegeneration>,
}

/// Exact validated work record submitted to the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientCallWrapUpPayload {
    /// Explicit operator-selected result.
    pub outcome: ClientCallOutcome,
    /// Trimmed operator notes; empty remains empty.
    pub notes: String,
    /// Optional positive, manually entered minutes, not inferred talk time.
    pub duration_minutes: Option<u32>,
    /// Trimmed callback instructions, present only for callback outcomes.
    pub follow_up: Option<String>,
}

/// Host-owned draft and persistence acknowledgment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientCallWrapUp {
    /// Required before save; initially absent.
    pub outcome: Option<ClientCallOutcome>,
    /// Draft notes, limited to 8,000 characters by the UI.
    pub notes: String,
    /// Optional positive integer input. Blank means unspecified.
    pub duration_minutes: String,
    /// Callback instructions, limited to 1,000 characters; no booking is made.
    pub follow_up: String,
    /// Save has been reserved; freezes the draft and prevents duplicate saves.
    pub pending: bool,
    /// Server confirmed this exact draft was persisted; fields become read-only.
    pub saved: bool,
    /// Save failure or host business-validation explanation.
    pub error: Option<String>,
}

impl ClientCallWrapUp {
    /// Build a submission only from complete, valid operator input.
    pub fn payload(&self) -> Option<ClientCallWrapUpPayload> {
        let outcome = self.outcome?;
        if self.notes.chars().count() > 8_000 || self.follow_up.chars().count() > 1_000 {
            return None;
        }
        let duration = self.duration_minutes.trim();
        let duration_minutes = if duration.is_empty() {
            None
        } else {
            if !duration.bytes().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            Some(duration.parse::<u32>().ok().filter(|value| *value > 0)?)
        };
        let follow_up = if outcome == ClientCallOutcome::RequestedCallBack {
            let text = self.follow_up.trim();
            if text.is_empty() {
                return None;
            }
            Some(text.to_owned())
        } else {
            None
        };
        Some(ClientCallWrapUpPayload {
            outcome,
            notes: self.notes.trim().into(),
            duration_minutes,
            follow_up,
        })
    }
}

/// Atomic projection of a single client and attempt. The host owns all fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientCallWorkspaceState {
    /// Canonical identity, saved numbers, managed session and clock contract.
    pub call: SoftphoneState,
    /// Dispatch evidence; bridge acceptance is never a managed connection.
    pub attempt: ClientCallAttempt,
    /// Editable destination. The host validates and normalizes for its provider.
    pub destination: String,
    /// Readiness or authorization failure; defaults to a closed launch gate.
    pub dial_blocked_reason: Option<String>,
    /// Host destination-validation error, blocking launch and contact save.
    pub number_error: Option<String>,
    /// Safe provider explanation, supplementary to the structured attempt state.
    pub status_detail: String,
    /// Provider-confirmed talk seconds; absent is unknown, zero is confirmed zero.
    pub provider_talk_seconds: Option<u64>,
    /// Optional explicit contact field update capability.
    pub number_update: Option<ClientCallNumberUpdate>,
    /// Optional objective or reviewed script.
    pub guidance: Option<ClientCallGuidance>,
    /// Optional work-record draft; absent hides the entire wrap-up section.
    pub wrap_up: Option<ClientCallWrapUp>,
}

impl Default for ClientCallWorkspaceState {
    fn default() -> Self {
        Self {
            call: SoftphoneState::default(),
            attempt: ClientCallAttempt::Ready,
            destination: String::new(),
            dial_blocked_reason: Some("Calling is not available.".into()),
            number_error: None,
            status_detail: String::new(),
            provider_talk_seconds: None,
            number_update: None,
            guidance: None,
            wrap_up: Some(ClientCallWrapUp::default()),
        }
    }
}

/// A request, never a success notification. Draft edits also require host adoption.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientCallAction {
    /// Replace the destination draft; never writes a contact.
    EditDestination(String),
    /// Adopt a known saved number by its stable identity.
    ChooseSavedNumber(String),
    /// Propose a contact-write target by identity; never writes or changes destination.
    ChooseNumberTarget(String),
    /// Request script regeneration; does not confirm acceptance or replace guidance.
    RegenerateGuidance,
    /// Request an explicit authorized contact write.
    SaveNumber {
        /// Must match the currently exposed writable field.
        field_id: String,
        /// Must match the current destination draft.
        number: String,
    },
    /// Reserve and dispatch the current destination once.
    Dial {
        /// Current draft; normalization and dispatch remain host responsibilities.
        number: String,
    },
    /// Change the explicit work outcome; clearing returns to no selection.
    SetOutcome(Option<ClientCallOutcome>),
    /// Replace operator notes.
    SetNotes(String),
    /// Replace the optional manually entered minute string.
    SetDurationMinutes(String),
    /// Replace callback instructions; does not schedule anything.
    SetFollowUp(String),
    /// Persist the exact current validated snapshot.
    SaveWrapUp(ClientCallWrapUpPayload),
    /// Forward a guarded live-session command; launch and number selection excluded.
    Session(SoftphoneAction),
    /// Propose closing the surface; does not hang up or discard drafts.
    Dismiss,
}

/// Scoped command envelope. Host responses additionally need operation correlation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientCallCommand {
    /// Client, actor, authorization generation and attempt identity minted by host.
    pub context_id: String,
    /// Requested operation.
    pub action: ClientCallAction,
}

impl ClientCallWorkspaceState {
    /// Whether destination editing is currently safe.
    pub fn can_edit_destination(&self) -> bool {
        !self.call.context_id.trim().is_empty()
            && matches!(
                self.attempt,
                ClientCallAttempt::Ready
                    | ClientCallAttempt::Refused
                    | ClientCallAttempt::RefusedWith(_)
            )
            && !self.call.phase.is_live()
            && self.call.pending.is_none()
            && !self
                .number_update
                .as_ref()
                .is_some_and(|update| update.pending)
            && !self
                .wrap_up
                .as_ref()
                .is_some_and(|wrap| wrap.pending || wrap.saved)
    }

    /// Whether the host has established completion sufficient to save an outcome.
    pub fn finished(&self) -> bool {
        (self.attempt == ClientCallAttempt::Finished && !self.call.phase.is_live())
            || (self.attempt == ClientCallAttempt::Managed
                && self.call.phase == SoftphonePhase::Ended
                && self.call.pending.is_none())
    }

    fn valid_destination(&self) -> bool {
        !self.destination.trim().is_empty()
            && self.destination.chars().count() <= 64
            && self.number_error.is_none()
    }

    /// Repeat eligibility at the callback boundary, including exact payload matching.
    pub fn can_dispatch(&self, action: &ClientCallAction) -> bool {
        if self.call.context_id.trim().is_empty() {
            return false;
        }
        let editable_wrap = self
            .wrap_up
            .as_ref()
            .is_some_and(|wrap| !wrap.pending && !wrap.saved);
        match action {
            ClientCallAction::Dismiss => true,
            ClientCallAction::EditDestination(value) => {
                self.can_edit_destination() && value.chars().count() <= 64
            }
            ClientCallAction::ChooseSavedNumber(id) => {
                self.can_edit_destination()
                    && self
                        .call
                        .client
                        .phones
                        .iter()
                        .filter(|phone| phone.id == *id)
                        .count()
                        == 1
                    && self.call.client.phones.iter().any(|phone| {
                        phone.id == *id
                            && !id.trim().is_empty()
                            && !phone.number.trim().is_empty()
                            && phone.number.chars().count() <= 64
                            && phone.blocked_reason.is_none()
                    })
            }
            ClientCallAction::ChooseNumberTarget(field_id) => {
                self.can_edit_destination()
                    && self
                        .number_update
                        .as_ref()
                        .is_some_and(|update| update.can_choose_target(field_id))
            }
            ClientCallAction::RegenerateGuidance => self
                .guidance
                .as_ref()
                .and_then(|guidance| guidance.regeneration.as_ref())
                .is_some_and(|regeneration| {
                    regeneration.blocked_reason.is_none()
                        && matches!(
                            regeneration.state,
                            ClientCallRegenerationState::Idle
                                | ClientCallRegenerationState::Succeeded
                                | ClientCallRegenerationState::Failed
                        )
                }),
            ClientCallAction::Dial { number } => {
                self.can_edit_destination()
                    && self.valid_destination()
                    && self.dial_blocked_reason.is_none()
                    && !self.call.client.phones.iter().any(|phone| {
                        phone.blocked_reason.is_some() && phone.number.trim() == number.trim()
                    })
                    && *number == self.destination
            }
            ClientCallAction::SaveNumber { field_id, number } => {
                self.can_edit_destination()
                    && self.valid_destination()
                    && *number == self.destination
                    && self.number_update.as_ref().is_some_and(|update| {
                        update.blocked_reason.is_none()
                            && update.selected_target().is_some_and(|target| {
                                target.field_id == *field_id
                                    && target.blocked_reason.is_none()
                                    && target.saved_number.trim() != number.trim()
                            })
                    })
            }
            ClientCallAction::SetOutcome(_) => editable_wrap,
            ClientCallAction::SetNotes(value) => editable_wrap && value.chars().count() <= 8_000,
            ClientCallAction::SetDurationMinutes(value) => {
                editable_wrap && value.chars().count() <= 10
            }
            ClientCallAction::SetFollowUp(value) => {
                editable_wrap
                    && value.chars().count() <= 1_000
                    && self.wrap_up.as_ref().is_some_and(|wrap| {
                        wrap.outcome == Some(ClientCallOutcome::RequestedCallBack)
                    })
            }
            ClientCallAction::SaveWrapUp(payload) => {
                editable_wrap
                    && self.finished()
                    && self
                        .wrap_up
                        .as_ref()
                        .and_then(ClientCallWrapUp::payload)
                        .as_ref()
                        == Some(payload)
            }
            ClientCallAction::Session(action) => {
                self.attempt == ClientCallAttempt::Managed
                    && !matches!(
                        action,
                        SoftphoneAction::Call { .. } | SoftphoneAction::SelectNumber(_)
                    )
                    && self.call.can_dispatch(action)
            }
        }
    }

    /// Reject stale context envelopes before a consumer reserves any side effect.
    pub fn accepts(&self, command: &ClientCallCommand) -> bool {
        command.context_id == self.call.context_id && self.can_dispatch(&command.action)
    }
}
