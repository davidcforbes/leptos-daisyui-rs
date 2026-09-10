//! Accepted preferences and host-owned engine connection projections.

use super::model::{require_nonblank, unique_identities, validate_reason};
use super::*;

/// Proposed and accepted settings use this same shape but different revisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssistantPreferences {
    /// Explicit host-selected engine; the component never selects a fallback.
    pub engine_id: Option<String>,
    /// Whether confirmed actor memory may be used.
    pub memory_use: bool,
    /// Whether capture is allowed; defaults off independently of memory use.
    pub memory_capture: bool,
}

/// A configured budget projection, not an invented quota or purchase offer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantBudget {
    /// Host policy name.
    pub policy_label: String,
    /// Host-localized configured limit, including its accounting basis.
    pub limit_display: String,
    /// Known remaining allocation; absence stays unknown.
    pub remaining_display: Option<String>,
    /// Host-localized projection freshness.
    pub as_of: String,
}

/// Independent accepted and editable settings owned by the authenticated actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantSettings {
    /// Authenticated operator identity, never the viewed subject.
    pub owner_actor_id: String,
    /// Accepted read-back revision.
    pub accepted_revision: u64,
    /// Current proposed form revision; every accepted edit advances it.
    pub proposed_revision: u64,
    /// Current accepted preference, also used by the workspace header.
    pub accepted: AssistantPreferences,
    /// Unsaved host-controlled form; no optimistic promotion to accepted state.
    pub proposed: AssistantPreferences,
    /// Tier verdict takes precedence over every engine connection state.
    pub reasoning_tier: AssistantAccess,
    /// Explicit host engine inventory, not a component plugin registry.
    pub engines: Vec<AssistantEngine>,
    /// Configured budget only; absent means no confirmed policy projection.
    pub budget: Option<AssistantBudget>,
    /// Settings-level grants, additionally restricted by page and engine grants.
    pub capabilities: AssistantCapabilities,
}

/// Host availability keeps protocol and untested-CLI reasons distinguishable.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineAvailability {
    /// The configured engine is enabled; connection still gates execution.
    Enabled,
    /// An explicit host restriction, independent of sign-in state.
    Disabled {
        /// Opaque availability reason identity.
        reason_code: String,
        /// Sanitized host-localized explanation.
        text: String,
    },
    /// Unsupported availability is never treated as enabled.
    Unknown(String),
}

/// A versioned engine configuration; credentials remain entirely in the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantEngine {
    /// Opaque engine identity.
    pub id: String,
    /// Accepted configuration revision.
    pub revision: u64,
    /// Display-safe engine name.
    pub label: String,
    /// Configuration availability outranks connection readiness.
    pub availability: EngineAvailability,
    /// Accepted host worker connection projection.
    pub connection: AssistantConnection,
    /// Explicit engine-specific grants.
    pub capabilities: AssistantCapabilities,
}

/// Sign-in mechanism, independent of its current connection state.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignInShape {
    /// Explicit named host handoff; this UI never collects a token.
    Paste,
    /// Host-provided device verification code and expiry.
    Device,
    /// Operator-assisted sign-in, never simulated as a native device flow.
    Operator,
    /// Unsupported mechanism cannot launch a sign-in action.
    Unknown(String),
}

/// All host connection states are distinct from engine/tier availability.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    /// No accepted sign-in.
    NotSignedIn,
    /// A host-owned sign-in flow is waiting for completion.
    Pending,
    /// The host is verifying its worker's connection.
    Probing,
    /// Actual accepted signed-in state, not a launched-flow acknowledgment.
    SignedIn,
    /// Previous connection or flow expired.
    Expired,
    /// The sign-in attempt was rejected.
    Rejected,
    /// Prior worker access was revoked.
    Revoked,
    /// This configured engine does not require sign-in.
    NotApplicable,
    /// Unrecognized state, never an implicit signed-in verdict.
    Unknown(String),
}

/// Non-secret device-flow presentation supplied by an authorized host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantDeviceFlow {
    /// Displayed destination; navigation is an explicit host-validated command.
    pub verification_url: String,
    /// Public short-lived verification code, never an access token.
    pub user_code: String,
    /// Authoritative host expiry in Unix milliseconds.
    pub expires_at_ms: u64,
    /// Host-localized expiry display, never parsed for authority.
    pub expires_at: String,
}

/// Host connection read-back; sign-out cleanup does not mean memory withdrawal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantConnection {
    /// Accepted worker connection state.
    pub state: ConnectionState,
    /// Named host sign-in mechanism.
    pub shape: SignInShape,
    /// Required for pending Device; all other mechanisms need no fake payload.
    pub device: Option<AssistantDeviceFlow>,
    /// Safe display-only account label when known.
    pub account_label: Option<String>,
    /// Host-localized last verified time when known.
    pub verified_at: Option<String>,
    /// Host-localized connection expiry when known.
    pub expires_at: Option<String>,
    /// Sanitized host explanation, never a provider diagnostic dump.
    pub reason: Option<AssistantReason>,
}

/// Explicit worker cleanup choice, independent of governed actor-memory policy.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerCleanup {
    /// Sign out while retaining any host-permitted worker-local state.
    SignOutOnly,
    /// Ask the host to remove worker-local session material too.
    RemoveWorkerSession,
    /// Unknown cleanup policy cannot authorize a sign-out request.
    Unknown(String),
}

/// Validates a connection's typed shape without promoting expiry to sign-in.
pub fn validate_connection(connection: &AssistantConnection) -> Result<(), AssistantContractError> {
    if matches!(connection.state, ConnectionState::Unknown(_))
        || matches!(connection.shape, SignInShape::Unknown(_))
    {
        return Err(AssistantContractError::UnknownState);
    }
    if matches!(
        connection.state,
        ConnectionState::Expired | ConnectionState::Rejected
    ) && connection.reason.is_none()
    {
        return Err(AssistantContractError::InvalidConnection);
    }
    if connection.state == ConnectionState::Pending
        && connection.shape == SignInShape::Device
        && connection.device.is_none()
    {
        return Err(AssistantContractError::InvalidConnection);
    }
    if let Some(device) = &connection.device
        && (connection.shape != SignInShape::Device
            || device.verification_url.trim().is_empty()
            || device.user_code.trim().is_empty()
            || device.expires_at.trim().is_empty())
    {
        return Err(AssistantContractError::InvalidConnection);
    }
    for value in [
        &connection.account_label,
        &connection.verified_at,
        &connection.expires_at,
    ]
    .into_iter()
    .flatten()
    {
        require_nonblank(value)?;
    }
    if let Some(reason) = &connection.reason {
        validate_reason(reason)?;
    }
    Ok(())
}

/// Validates this actor's independently controlled accepted/proposed settings.
pub fn validate_settings(
    settings: &AssistantSettings,
    actor_id: &str,
) -> Result<(), AssistantContractError> {
    require_nonblank(actor_id)?;
    require_nonblank(&settings.owner_actor_id)?;
    if settings.owner_actor_id != actor_id {
        return Err(AssistantContractError::ScopeMismatch);
    }
    settings.capabilities.validate()?;
    match &settings.reasoning_tier {
        AssistantAccess::Unknown(_) => return Err(AssistantContractError::UnknownState),
        AssistantAccess::Denied { reason } | AssistantAccess::Unresolved { reason } => {
            validate_reason(reason)?
        }
        AssistantAccess::Granted => {}
    }
    unique_identities(settings.engines.iter().map(|engine| engine.id.as_str()))?;
    for engine in &settings.engines {
        validate_engine(engine)?;
    }
    for preferences in [&settings.accepted, &settings.proposed] {
        if let Some(id) = &preferences.engine_id {
            require_nonblank(id)?;
            if !settings.engines.iter().any(|engine| engine.id == *id) {
                return Err(AssistantContractError::MissingIdentity);
            }
        }
    }
    if let Some(budget) = &settings.budget {
        require_nonblank(&budget.policy_label)?;
        require_nonblank(&budget.limit_display)?;
        require_nonblank(&budget.as_of)?;
        if let Some(remaining) = &budget.remaining_display {
            require_nonblank(remaining)?;
        }
    }
    Ok(())
}

fn validate_engine(engine: &AssistantEngine) -> Result<(), AssistantContractError> {
    require_nonblank(&engine.id)?;
    require_nonblank(&engine.label)?;
    engine.capabilities.validate()?;
    match &engine.availability {
        EngineAvailability::Enabled => {}
        EngineAvailability::Disabled { reason_code, text } => {
            require_nonblank(reason_code)?;
            require_nonblank(text)?;
        }
        EngineAvailability::Unknown(_) => return Err(AssistantContractError::UnknownState),
    }
    validate_connection(&engine.connection)
}

/// Returns the accepted engine only after tier, configuration and sign-in gates.
pub fn engine_ready_for_ask<'a>(
    settings: &'a AssistantSettings,
    actor_id: &str,
) -> Option<&'a AssistantEngine> {
    // A signed-in worker cannot override the page's reasoning-tier verdict.
    if settings.reasoning_tier != AssistantAccess::Granted
        || !settings.capabilities.allows(&AssistantCapability::Ask)
    {
        return None;
    }
    // Proposed edits, budget display and unrelated engines are independently
    // rejectable Settings data, not authority for this accepted Ask preference.
    require_nonblank(actor_id).ok()?;
    if settings.owner_actor_id != actor_id {
        return None;
    }
    let selected_id = settings.accepted.engine_id.as_ref()?;
    require_nonblank(selected_id).ok()?;
    let mut matches = settings
        .engines
        .iter()
        .filter(|engine| engine.id == *selected_id);
    let engine = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    validate_engine(engine).ok()?;
    (engine.availability == EngineAvailability::Enabled
        && engine.capabilities.allows(&AssistantCapability::Ask)
        && matches!(
            engine.connection.state,
            ConnectionState::SignedIn | ConnectionState::NotApplicable
        ))
    .then_some(engine)
}

/// Tests device payload currency; the dispatcher additionally requires grants.
pub fn device_flow_is_current(connection: &AssistantConnection, now_ms: u64) -> bool {
    validate_connection(connection).is_ok()
        && connection.state == ConnectionState::Pending
        && connection.shape == SignInShape::Device
        && connection
            .device
            .as_ref()
            .is_some_and(|device| now_ms < device.expires_at_ms)
}
