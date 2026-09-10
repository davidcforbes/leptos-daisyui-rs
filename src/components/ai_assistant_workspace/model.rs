//! Pure structural checks and fail-closed interaction rules.

use super::*;

/// Validates the complete workspace boundary while keeping optional services independent.
pub fn validate_workspace(state: &AssistantWorkspaceState) -> Result<(), AssistantContractError> {
    match &state.access {
        AssistantAccess::Granted => {}
        AssistantAccess::Denied { reason } | AssistantAccess::Unresolved { reason } => {
            validate_reason(reason)?
        }
        AssistantAccess::Unknown(_) => return Err(AssistantContractError::UnknownState),
    }
    let context = state
        .context
        .as_ref()
        .ok_or(AssistantContractError::InvalidReceipt)?;
    validate_context(context)?;
    let conversation = match &state.conversation {
        AssistantLoad::Ready(value) => value,
        AssistantLoad::Denied { reason } | AssistantLoad::Unavailable { reason } => {
            validate_reason(reason)?;
            return Ok(());
        }
        AssistantLoad::Loading => return Ok(()),
        AssistantLoad::ContractError(error) => return Err(*error),
    };
    validate_conversation(conversation)?;
    if conversation.context_id != context.id || conversation.scope_id != context.scope.id {
        return Err(AssistantContractError::ScopeMismatch);
    }
    if let AssistantLoad::Ready(settings) = &state.settings {
        validate_settings(settings, &context.actor.id)?;
    }
    Ok(())
}

/// Returns whether an explicit Ask is currently admissible by every required owner.
pub fn can_dispatch(state: &AssistantWorkspaceState, action: &AssistantAction) -> bool {
    let Some(context) = state.context.as_ref() else {
        return false;
    };
    if state.access != AssistantAccess::Granted
        || !context.capabilities.allows(&AssistantCapability::Ask)
        || action.context != context.stamp()
    {
        return false;
    }
    let AssistantActionKind::Ask = action.kind else {
        return false;
    };
    let AssistantIntentTarget::Conversation {
        conversation: conversation_target,
        draft: draft_target,
    } = &action.target
    else {
        return false;
    };
    let AssistantLoad::Ready(conversation) = &state.conversation else {
        return false;
    };
    if conversation.closed
        || !conversation.capabilities.allows(&AssistantCapability::Ask)
        || conversation.id != conversation_target.id
        || conversation.revision != conversation_target.revision
        || conversation.draft.id != draft_target.id
        || conversation.draft.revision != draft_target.revision
        || conversation.draft.context_id != context.id
        || conversation.draft.context_revision != context.revision
        || conversation.draft.text.trim().is_empty()
        || conversation.draft.text.chars().count() > conversation.draft.max_chars
    {
        return false;
    }
    if !matches!(conversation.submission, SubmissionDisposition::Idle) {
        return false;
    }
    let Some(request_id) = state.next_request_id.as_ref() else {
        return false;
    };
    if request_id.trim().is_empty() {
        return false;
    }
    let Some(settings) = state.settings_ready() else {
        return false;
    };
    engine_ready_for_ask(settings, &context.actor.id).is_some()
}

/// Builds a typed command from an admissible intent without changing state.
pub fn command_for(
    state: &AssistantWorkspaceState,
    action: AssistantAction,
) -> Option<AssistantCommand> {
    if !can_dispatch(state, &action) {
        return None;
    }
    let context = state.context.as_ref()?;
    let conversation = match &state.conversation {
        AssistantLoad::Ready(value) => value,
        _ => return None,
    };
    let settings = state.settings_ready()?;
    let engine = engine_ready_for_ask(settings, &context.actor.id)?;
    let request = AssistantRequest {
        id: state.next_request_id.as_ref()?.clone(),
        context: action.context.clone(),
        target: action.target.clone(),
    };
    Some(AssistantCommand {
        action,
        request: Some(request),
        payload: AssistantCommandPayload::Ask {
            question: conversation.draft.text.clone(),
            selected_engine_id: engine.id.clone(),
            scope: context.scope.clone(),
        },
    })
}

/// Checks that a command still matches the current host projection and allocation.
pub fn accepts_command(state: &AssistantWorkspaceState, command: &AssistantCommand) -> bool {
    let Some(expected) = command_for(state, command.action.clone()) else {
        return false;
    };
    expected == *command
}

impl AssistantWorkspaceState {
    fn settings_ready(&self) -> Option<&AssistantSettings> {
        match &self.settings {
            AssistantLoad::Ready(value) => Some(value),
            _ => None,
        }
    }
}

/// Validates an answer against its admitted scope, not the currently viewed page.
pub fn validate_answer(
    answer: &AssistantAnswer,
    accepted_scope: &AssistantScope,
) -> Result<(), AssistantContractError> {
    require_nonblank(&answer.id)?;
    require_nonblank(&answer.as_of)?;
    validate_scope(accepted_scope)?;
    validate_scope(&answer.answered_scope)?;
    if answer.answered_scope != *accepted_scope {
        return Err(AssistantContractError::ScopeMismatch);
    }
    match &answer.outcome {
        AnswerOutcome::Answered { text } if !text.trim().is_empty() => {}
        AnswerOutcome::Declined { limitations, as_of }
            if !limitations.is_empty()
                && limitations.iter().all(|value| !value.trim().is_empty())
                && !as_of.trim().is_empty() => {}
        AnswerOutcome::Unknown(_) => return Err(AssistantContractError::UnknownState),
        _ => return Err(AssistantContractError::InvalidAnswer),
    }
    if answer
        .limitations
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(AssistantContractError::InvalidAnswer);
    }
    unique_identities(answer.evidence.iter().map(|value| value.id.as_str()))?;
    for evidence in &answer.evidence {
        validate_evidence(evidence)?;
    }
    validate_provenance(&answer.provenance)
}

/// Validates a host attempt without consuming raw transport events.
pub fn validate_attempt(attempt: &AssistantAttempt) -> Result<(), AssistantContractError> {
    require_nonblank(&attempt.id)?;
    require_nonblank(&attempt.request_id)?;
    require_nonblank(&attempt.question)?;
    validate_context(&attempt.accepted_context)?;
    if let Some(engine) = &attempt.requested_engine {
        require_nonblank(engine)?;
    }
    // Request acknowledgment is not terminal cancellation. It does, however,
    // prevent a late provider payload from becoming an accepted final answer.
    if attempt.cancel_requested && matches!(attempt.lifecycle, AttemptLifecycle::Completed(_)) {
        return Err(AssistantContractError::InvalidAnswer);
    }
    match &attempt.eligibility {
        AssistantEligibility::Unknown(_) => return Err(AssistantContractError::UnknownState),
        AssistantEligibility::Ineligible { reason } => validate_reason(reason)?,
        AssistantEligibility::Eligible => {}
    }
    match &attempt.transport {
        AssistantTransport::Unknown(_)
        | AssistantTransport::Recovering {
            cause: RecoveryCause::Unknown(_),
            ..
        } => {
            return Err(AssistantContractError::UnknownState);
        }
        AssistantTransport::Disconnected { reason, .. }
        | AssistantTransport::Unreadable { reason, .. } => validate_reason(reason)?,
        AssistantTransport::Connected { .. } | AssistantTransport::Recovering { .. } => {}
    }
    match &attempt.lifecycle {
        AttemptLifecycle::Completed(answer) => {
            validate_answer(answer, &attempt.accepted_context.scope)
        }
        AttemptLifecycle::Unknown(_) => Err(AssistantContractError::UnknownState),
        AttemptLifecycle::Denied { reason }
        | AttemptLifecycle::Unavailable { reason }
        | AttemptLifecycle::Failed { reason }
        | AttemptLifecycle::Interrupted { reason } => validate_reason(reason),
        AttemptLifecycle::Admitted
        | AttemptLifecycle::Queued
        | AttemptLifecycle::Running
        | AttemptLifecycle::Validating
        | AttemptLifecycle::Canceled { .. } => Ok(()),
    }
}

/// Validates one continuity projection without silently rebinding its draft.
pub fn validate_conversation(
    conversation: &AssistantConversation,
) -> Result<(), AssistantContractError> {
    require_nonblank(&conversation.id)?;
    require_nonblank(&conversation.context_id)?;
    require_nonblank(&conversation.scope_id)?;
    require_nonblank(&conversation.draft.id)?;
    require_nonblank(&conversation.draft.context_id)?;
    if conversation.draft.context_id != conversation.context_id {
        return Err(AssistantContractError::ScopeMismatch);
    }
    conversation.capabilities.validate()?;
    if matches!(conversation.continuity, ConversationContinuity::Unknown(_)) {
        return Err(AssistantContractError::UnknownState);
    }
    unique_identities(conversation.attempts.iter().map(|value| value.id.as_str()))?;
    unique_identities(
        conversation
            .attempts
            .iter()
            .map(|value| value.request_id.as_str()),
    )?;
    unique_identities(
        conversation
            .attempts
            .iter()
            .filter_map(|value| match &value.lifecycle {
                AttemptLifecycle::Completed(answer) => Some(answer.id.as_str()),
                _ => None,
            }),
    )?;
    for attempt in &conversation.attempts {
        validate_attempt(attempt)?;
    }
    if let SubmissionDisposition::Submitting { request_id }
    | SubmissionDisposition::Uncertain { request_id, .. }
    | SubmissionDisposition::Refused { request_id, .. } = &conversation.submission
        && conversation
            .attempts
            .iter()
            .any(|attempt| attempt.request_id == *request_id)
    {
        return Err(AssistantContractError::InvalidReceipt);
    }
    match &conversation.submission {
        SubmissionDisposition::Idle => Ok(()),
        SubmissionDisposition::Submitting { request_id } => require_nonblank(request_id),
        SubmissionDisposition::Uncertain { request_id, reason } => {
            require_nonblank(request_id)?;
            validate_reason(reason)
        }
        SubmissionDisposition::Admitted {
            request_id,
            attempt_id,
        } => {
            require_nonblank(request_id)?;
            require_nonblank(attempt_id)?;
            if conversation
                .attempts
                .iter()
                .any(|value| value.id == *attempt_id && value.request_id == *request_id)
            {
                Ok(())
            } else {
                Err(AssistantContractError::InvalidReceipt)
            }
        }
        SubmissionDisposition::Refused { request_id, reason } => {
            require_nonblank(request_id)?;
            if reason.next_action().is_none() {
                Err(AssistantContractError::UnknownState)
            } else {
                Ok(())
            }
        }
        SubmissionDisposition::Unknown(_) => Err(AssistantContractError::UnknownState),
    }
}

/// Validates a source's display identity without treating its label as authority.
pub fn validate_evidence(evidence: &AssistantEvidence) -> Result<(), AssistantContractError> {
    require_nonblank(&evidence.id)?;
    require_nonblank(&evidence.label)?;
    require_nonblank(&evidence.source_kind)?;
    require_nonblank(&evidence.as_of)?;
    require_nonblank(&evidence.qualification)?;
    match &evidence.availability {
        AssistantAccess::Unknown(_) => Err(AssistantContractError::UnknownState),
        AssistantAccess::Denied { reason } | AssistantAccess::Unresolved { reason } => {
            validate_reason(reason)
        }
        AssistantAccess::Granted => Ok(()),
    }
}

/// Validates origin identities while preserving absence as unknown metadata.
pub fn validate_provenance(provenance: &AssistantProvenance) -> Result<(), AssistantContractError> {
    require_nonblank(&provenance.built_at)?;
    for omission in &provenance.omissions {
        require_nonblank(omission)?;
    }
    if matches!(provenance.mode, ProvenanceMode::Unknown(_)) {
        return Err(AssistantContractError::UnknownState);
    }
    for value in [
        &provenance.actual_engine,
        &provenance.model_version,
        &provenance.prompt_version,
        &provenance.skill_version,
    ]
    .into_iter()
    .flatten()
    {
        require_nonblank(value)?;
    }
    for items in [
        &provenance.memory_revisions,
        &provenance.foundation_revisions,
        &provenance.supplied_sources,
    ] {
        unique_identities(items.iter().map(|value| value.id.as_str()))?;
    }
    Ok(())
}

pub(crate) fn unique_identities<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), AssistantContractError> {
    let mut seen = std::collections::HashSet::new();
    for value in values {
        require_nonblank(value)?;
        if !seen.insert(value) {
            return Err(AssistantContractError::MissingIdentity);
        }
    }
    Ok(())
}

pub(crate) fn validate_reason(reason: &AssistantReason) -> Result<(), AssistantContractError> {
    require_nonblank(&reason.code)?;
    require_nonblank(&reason.message)
}

pub(crate) fn validate_eligibility(
    value: &AssistantEligibility,
) -> Result<(), AssistantContractError> {
    match value {
        AssistantEligibility::Eligible => Ok(()),
        AssistantEligibility::Ineligible { reason } => validate_reason(reason),
        AssistantEligibility::Unknown(_) => Err(AssistantContractError::UnknownState),
    }
}

/// Validates a captured identity without treating the stamp itself as a grant.
pub fn validate_context_stamp(stamp: &AssistantContextStamp) -> Result<(), AssistantContractError> {
    require_nonblank(&stamp.context_id)?;
    require_nonblank(&stamp.actor_id)?;
    require_nonblank(&stamp.scope_id)
}

/// Validates the current actor, population and page capability boundary.
pub fn validate_context(context: &AssistantContext) -> Result<(), AssistantContractError> {
    require_nonblank(&context.id)?;
    require_nonblank(&context.actor.id)?;
    require_nonblank(&context.actor.label)?;
    if let Some(subject) = &context.subject {
        require_nonblank(&subject.id)?;
        require_nonblank(&subject.label)?;
    }
    validate_scope(&context.scope)?;
    context.capabilities.validate()
}

pub(crate) fn require_nonblank(value: &str) -> Result<(), AssistantContractError> {
    if value.trim().is_empty() {
        Err(AssistantContractError::MissingIdentity)
    } else {
        Ok(())
    }
}

/// Requires an explicit population or expansion authorization identity.
pub fn validate_scope(scope: &AssistantScope) -> Result<(), AssistantContractError> {
    require_nonblank(&scope.id)?;
    require_nonblank(&scope.label)?;
    match &scope.basis {
        ScopeBasis::PagePopulation {
            population_id,
            description,
            ..
        } => {
            require_nonblank(population_id)?;
            require_nonblank(description)
        }
        ScopeBasis::Expanded {
            authorization_id,
            description,
            ..
        } => {
            require_nonblank(authorization_id)?;
            require_nonblank(description)
        }
        ScopeBasis::Unknown(_) => Err(AssistantContractError::UnknownState),
    }
}
