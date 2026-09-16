//! Provider cards: what engine a workspace can pick, how its model list
//! behaves, and what tuning knobs it exposes. Also the fixed catalogue this
//! epic's desktop host actually offers, and the availability-reason
//! vocabulary a refused engine reports.

use crate::components::ai_assistant_workspace::{
    AssistantCapabilities, AssistantCapability, AssistantConnection, AssistantEngine,
    ConnectionState, EngineAvailability, RefusalNextAction, SignInShape,
};
use crate::components::ai_chat::Capabilities;

/// One selectable engine: its generic capabilities, its picker label, how
/// its model list should be presented, and which tuning knobs it exposes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderCard {
    /// The governed engine projection (sign-in, availability, grants).
    pub engine: AssistantEngine,
    /// The generic transport capabilities (models, permission modes, …).
    pub capabilities: Capabilities,
    /// Short label for the engine picker, distinct from the fuller
    /// `capabilities.label` shown as answer attribution.
    pub picker_label: String,
    /// How the model picker should behave for this engine.
    pub model_source: ModelSource,
    /// Which tuning controls the composer should expose for this engine.
    pub tuning: TuningSchema,
}

/// How a model picker should treat an engine's model list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSource {
    /// No fixed list; the operator types a model id by hand.
    FreeText,
    /// A fixed, hardcoded list of selectable models.
    Fixed,
    /// The engine is pinned to exactly one model, never offered as a choice.
    Pinned(String),
    /// The model list came from a live probe as of this timestamp, not a
    /// hardcoded catalogue.
    Discovered {
        /// Host-localized or ISO-8601 freshness of the discovery probe.
        as_of: String,
    },
}

impl ModelSource {
    /// Derives how a picker should treat a backend's model list: an empty
    /// list always means free text, regardless of any pin or discovery hint
    /// (there is nothing to pin or discover into); otherwise an explicit
    /// pin outranks a discovery timestamp, and a plain non-empty list with
    /// neither is a fixed catalogue.
    pub fn from_capabilities(
        caps: &Capabilities,
        pinned: Option<&str>,
        discovered_as_of: Option<&str>,
    ) -> Self {
        if caps.models.is_empty() {
            Self::FreeText
        } else if let Some(model) = pinned {
            Self::Pinned(model.to_owned())
        } else if let Some(as_of) = discovered_as_of {
            Self::Discovered {
                as_of: as_of.to_owned(),
            }
        } else {
            Self::Fixed
        }
    }
}

/// Provider-specific tuning a host may apply when opening or reconfiguring
/// a session. Every field is optional because most engines use only a
/// subset; [`TuningSchema`] tells a consumer which subset applies.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderTuning {
    /// A CLI-style permission mode (`acceptEdits`, `dontAsk`, …).
    pub permission_mode: Option<String>,
    /// A reasoning-effort level, for engines that expose one.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Groq-specific sampling tuning.
    pub groq: Option<GroqTuning>,
    /// Codex-specific execution levers.
    pub codex: Option<CodexLevers>,
}

/// A reasoning-effort level. `Unknown` preserves a host value this
/// vocabulary does not yet name, rather than discarding it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReasoningEffort {
    /// Fastest, least deliberation.
    Low,
    /// The engine's default balance.
    Medium,
    /// Slowest, most deliberation.
    High,
    /// An engine-reported level this vocabulary does not name.
    Unknown(String),
}

impl ReasoningEffort {
    /// The wire string this variant round-trips to; `Unknown` returns the
    /// original string it was parsed from.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// Parses a host's wire string, preserving an unrecognized one verbatim.
    pub fn parse(s: &str) -> Self {
        match s {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Groq-specific sampling tuning.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroqTuning {
    /// Sampling temperature.
    pub temperature: f32,
}

/// Codex CLI execution levers a host may toggle.
///
/// The Codex CLI never offers MCP tool access at all — that is an immutable
/// fact about this engine, never a lever a consumer can flip, so no field
/// here represents it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CodexLevers {
    /// Whether the CLI's built-in web-search tool is enabled.
    pub web_search: bool,
    /// Whether the CLI's plugin surface is suppressed.
    pub suppress_plugins: bool,
    /// Whether the CLI's sandboxed code-execution mode is disabled.
    pub disable_code_mode: bool,
}

/// Which tuning controls a composer should expose for one engine. A `false`
/// field means the control must not render at all, not that it renders
/// disabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TuningSchema {
    /// Whether a reasoning-effort control applies.
    pub reasoning_effort: bool,
    /// Whether a temperature control applies.
    pub temperature: bool,
    /// Whether the Codex-specific lever controls apply.
    pub codex_levers: bool,
    /// Whether a permission-mode control applies.
    pub permission_mode: bool,
}

/// Why an engine is currently unavailable to ask a question. `Unknown`
/// preserves a host code this vocabulary does not yet name.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityReasonCode {
    /// The account's tier does not include this effect.
    TierEffectsDisabled,
    /// The engine has not been armed for use.
    NotArmed,
    /// The engine's CLI is not installed.
    CliMissing,
    /// The installed CLI version is too old.
    CliVersionUnsupported,
    /// The installed CLI version has not been verified for this workspace
    /// (an environment fact, not a credential problem).
    CliVersionUntested,
    /// The CLI speaks a protocol this workspace does not support.
    CliProtocolUnsupported,
    /// No credential key is configured for this engine.
    CredentialKeyUnavailable,
    /// The actor is not signed in to this engine.
    NotSignedIn,
    /// The actor's sign-in for this engine has expired.
    SignInExpired,
    /// This engine's budget has been used up.
    BudgetExhausted,
    /// This engine is busy with another request.
    EngineBusy,
    /// This engine's sign-in mechanism is itself unavailable.
    SignInShapeUnavailable,
    /// The engine's local runtime process is not running, so nothing is
    /// listening to answer. Deliberately NOT named after any one engine: it
    /// is the shape every locally hosted runtime has (a model server the
    /// actor starts themselves), and naming it `Ollama…` would force the
    /// next local runtime to invent a synonym.
    EngineProcessNotRunning,
    /// The engine is reachable but the requested model is not installed on
    /// this machine. Distinct from [`Self::EngineProcessNotRunning`]: the
    /// runtime answered, and the fix is to fetch a model rather than to
    /// start a process.
    ModelNotInstalled,
    /// A host reason this vocabulary does not name.
    Unknown(String),
}

impl AvailabilityReasonCode {
    /// Parses the host's wire code; an unrecognized code is preserved
    /// verbatim (never discarded), so a newer host reason cannot silently
    /// disappear.
    pub fn parse(code: &str) -> Self {
        match code {
            "tier_effects_disabled" => Self::TierEffectsDisabled,
            "not_armed" => Self::NotArmed,
            "cli_missing" => Self::CliMissing,
            "cli_version_unsupported" => Self::CliVersionUnsupported,
            "cli_version_untested" => Self::CliVersionUntested,
            "cli_protocol_unsupported" => Self::CliProtocolUnsupported,
            "credential_key_unavailable" => Self::CredentialKeyUnavailable,
            "not_signed_in" => Self::NotSignedIn,
            "sign_in_expired" => Self::SignInExpired,
            "budget_exhausted" => Self::BudgetExhausted,
            "engine_busy" => Self::EngineBusy,
            "sign_in_shape_unavailable" => Self::SignInShapeUnavailable,
            "engine_process_not_running" => Self::EngineProcessNotRunning,
            "model_not_installed" => Self::ModelNotInstalled,
            other => Self::Unknown(other.to_owned()),
        }
    }

    /// The wire code this variant round-trips to; `Unknown` returns the
    /// original code it was parsed from.
    pub fn as_code(&self) -> &str {
        match self {
            Self::TierEffectsDisabled => "tier_effects_disabled",
            Self::NotArmed => "not_armed",
            Self::CliMissing => "cli_missing",
            Self::CliVersionUnsupported => "cli_version_unsupported",
            Self::CliVersionUntested => "cli_version_untested",
            Self::CliProtocolUnsupported => "cli_protocol_unsupported",
            Self::CredentialKeyUnavailable => "credential_key_unavailable",
            Self::NotSignedIn => "not_signed_in",
            Self::SignInExpired => "sign_in_expired",
            Self::BudgetExhausted => "budget_exhausted",
            Self::EngineBusy => "engine_busy",
            Self::SignInShapeUnavailable => "sign_in_shape_unavailable",
            Self::EngineProcessNotRunning => "engine_process_not_running",
            Self::ModelNotInstalled => "model_not_installed",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// True only for the three reasons an actor fixes by presenting a
    /// credential again (a new key, a fresh sign-in). An untested CLI
    /// version is an environment fact rather than a stale credential, so it
    /// deliberately returns `false` here even though it also blocks the
    /// engine.
    pub fn is_credential_failure(&self) -> bool {
        matches!(
            self,
            Self::CredentialKeyUnavailable | Self::NotSignedIn | Self::SignInExpired
        )
    }

    /// Safe next-step guidance for a refusal carrying this reason.
    ///
    /// The three credential reasons map to [`RefusalNextAction::OpenSettings`]
    /// (an actor fixes them there); [`AvailabilityReasonCode::BudgetExhausted`],
    /// [`AvailabilityReasonCode::EngineBusy`] and [`AvailabilityReasonCode::NotArmed`]
    /// map to [`RefusalNextAction::RetryLater`] (nothing to configure, just
    /// wait). The CLI/protocol environment reasons and
    /// [`AvailabilityReasonCode::SignInShapeUnavailable`] also map to
    /// `RetryLater`: none of them is fixed by a settings edit, so offering
    /// `OpenSettings` would be dishonest.
    /// [`AvailabilityReasonCode::EngineProcessNotRunning`] and
    /// [`AvailabilityReasonCode::ModelNotInstalled`] map to `RetryLater` for
    /// the same reason: both are fixed OUTSIDE this workspace (start the
    /// runtime, fetch the model), and a settings pane that cannot do either
    /// must not be offered as the remedy. [`AvailabilityReasonCode::TierEffectsDisabled`]
    /// maps to `OpenSettings` because that is where an actor checks
    /// tier/plan state, even though nothing there lifts the restriction
    /// directly. An unrecognized reason maps to
    /// [`RefusalNextAction::NewConversation`] — the one action that neither
    /// promises a retry will help nor implies settings can fix a problem it
    /// cannot identify.
    pub fn next_action(&self) -> RefusalNextAction {
        match self {
            Self::CredentialKeyUnavailable | Self::NotSignedIn | Self::SignInExpired => {
                RefusalNextAction::OpenSettings
            }
            Self::BudgetExhausted | Self::EngineBusy | Self::NotArmed => {
                RefusalNextAction::RetryLater
            }
            Self::CliMissing
            | Self::CliVersionUnsupported
            | Self::CliVersionUntested
            | Self::CliProtocolUnsupported
            | Self::SignInShapeUnavailable
            | Self::EngineProcessNotRunning
            | Self::ModelNotInstalled => RefusalNextAction::RetryLater,
            Self::TierEffectsDisabled => RefusalNextAction::OpenSettings,
            Self::Unknown(_) => RefusalNextAction::NewConversation,
        }
    }
}

fn fixture_connection(needs_api_key: bool) -> AssistantConnection {
    if needs_api_key {
        AssistantConnection {
            state: ConnectionState::SignedIn,
            shape: SignInShape::Paste,
            device: None,
            account_label: None,
            verified_at: None,
            expires_at: None,
            reason: None,
        }
    } else {
        AssistantConnection {
            state: ConnectionState::NotApplicable,
            shape: SignInShape::Operator,
            device: None,
            account_label: None,
            verified_at: None,
            expires_at: None,
            reason: None,
        }
    }
}

/// The engine-level grants every fixture card carries: asking a question,
/// canceling admitted work, starting a fresh conversation, and copying
/// visible output. Without at least `Ask` granted here,
/// `engine_ready_for_ask` refuses every one of these engines regardless of
/// sign-in or availability — `AssistantCapabilities::allows` requires an
/// explicit grant, never inferring one from an engine merely existing.
fn engine_grants() -> AssistantCapabilities {
    AssistantCapabilities {
        granted: vec![
            AssistantCapability::Ask,
            AssistantCapability::Cancel,
            AssistantCapability::NewConversation,
            AssistantCapability::Copy,
        ],
        details: vec![],
    }
}

fn card(
    picker_label: &str,
    capabilities: Capabilities,
    pinned: Option<&str>,
    discovered_as_of: Option<&str>,
    tuning: TuningSchema,
) -> ProviderCard {
    let model_source = ModelSource::from_capabilities(&capabilities, pinned, discovered_as_of);
    let engine = AssistantEngine {
        id: capabilities.id.clone(),
        revision: 1,
        label: capabilities.label.clone(),
        availability: EngineAvailability::Enabled,
        connection: fixture_connection(capabilities.needs_api_key),
        capabilities: engine_grants(),
    };
    ProviderCard {
        engine,
        capabilities,
        picker_label: picker_label.to_owned(),
        model_source,
        tuning,
    }
}

/// The five engines this epic's desktop host actually offers, with their
/// exact capability truth. A fixture for tests and the demo, not a plugin
/// registry — a real host supplies its own list through
/// `super::backend::ChatWorkspaceBackend::providers`.
pub fn desktop_provider_catalogue() -> Vec<ProviderCard> {
    let codex_models: Vec<String> = [
        "gpt-5.6-sol",
        "gpt-5.6-terra",
        "gpt-5.6-luna",
        "gpt-5.6-pro",
        "gpt-5.3-codex-spark",
        "gpt-5.6",
        "gpt-5.5",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    vec![
        card(
            "Claude",
            Capabilities {
                id: "claude-code".into(),
                label: "Claude Code (local)".into(),
                needs_api_key: false,
                models: vec![],
                permission_modes: ["acceptEdits", "dontAsk", "default", "skip"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                supports_thinking: true,
                supports_tool_calls: true,
            },
            None,
            None,
            TuningSchema {
                reasoning_effort: true,
                permission_mode: true,
                ..TuningSchema::default()
            },
        ),
        card(
            "Codex",
            Capabilities {
                id: "codex-cli".into(),
                label: "Codex CLI (OpenAI)".into(),
                needs_api_key: false,
                models: codex_models.clone(),
                permission_modes: vec![],
                supports_thinking: true,
                supports_tool_calls: true,
            },
            None,
            None,
            TuningSchema {
                reasoning_effort: true,
                codex_levers: true,
                ..TuningSchema::default()
            },
        ),
        card(
            "Spark",
            Capabilities {
                id: "codex-spark".into(),
                label: "Codex Spark".into(),
                needs_api_key: false,
                models: codex_models,
                permission_modes: vec![],
                supports_thinking: true,
                supports_tool_calls: true,
            },
            Some("gpt-5.3-codex-spark"),
            None,
            TuningSchema {
                reasoning_effort: true,
                ..TuningSchema::default()
            },
        ),
        card(
            "Groq",
            Capabilities {
                id: "groq-gpt-oss-120b".into(),
                label: "Groq gpt-oss-120b".into(),
                needs_api_key: true,
                models: vec!["openai/gpt-oss-120b".into()],
                permission_modes: vec![],
                supports_thinking: true,
                supports_tool_calls: true,
            },
            Some("openai/gpt-oss-120b"),
            None,
            TuningSchema {
                reasoning_effort: true,
                temperature: true,
                ..TuningSchema::default()
            },
        ),
        card(
            "Ollama",
            Capabilities {
                id: "ollama".into(),
                label: "Ollama (local)".into(),
                needs_api_key: false,
                models: vec!["llama3.1:8b".into(), "qwen2.5:7b".into()],
                permission_modes: vec![],
                supports_thinking: false,
                supports_tool_calls: false,
            },
            None,
            // Fixture-only placeholder: a real host stamps this with the
            // actual last-discovery time from its local model probe.
            Some("1970-01-01T00:00:00Z"),
            TuningSchema::default(),
        ),
    ]
}

/// Whether ONE provider card may currently be asked a question.
///
/// This is the per-card readiness the provider picker gates on, and it is a
/// different question from the one
/// `crate::components::ai_assistant_workspace::engine_ready_for_ask` answers:
/// that function takes the whole `AssistantSettings` and reports only whether
/// the actor's single ACCEPTED engine (`settings.accepted.engine_id`) is
/// ready, so calling it once per card neither type-checks nor computes
/// per-card readiness. The three conditions below are exactly the ones it
/// checks on the engine it selects, minus the accepted-engine-id filter:
///
/// * the engine's availability is [`EngineAvailability::Enabled`],
/// * its engine-level grants allow [`AssistantCapability::Ask`], and
/// * its connection is `SignedIn` or `NotApplicable`.
///
/// Tier and budget are deliberately NOT part of this: they are properties of
/// the account, not of a card, and a picker that hid every card on a denied
/// tier would leave the actor with nothing to read the refusal against.
pub fn card_ready_for_ask(card: &ProviderCard) -> bool {
    card.engine.availability == EngineAvailability::Enabled
        && card.engine.capabilities.allows(&AssistantCapability::Ask)
        && matches!(
            card.engine.connection.state,
            ConnectionState::SignedIn | ConnectionState::NotApplicable
        )
}

/// The availability reason a card that is not ready should show, derived from
/// whichever gate [`card_ready_for_ask`] failed on. Returns `None` for a card
/// that IS ready, so a caller cannot accidentally render a reason beside a
/// usable engine.
pub fn card_unready_reason(card: &ProviderCard) -> Option<AvailabilityReasonCode> {
    if card_ready_for_ask(card) {
        return None;
    }
    if let EngineAvailability::Disabled { reason_code, .. } = &card.engine.availability {
        return Some(AvailabilityReasonCode::parse(reason_code));
    }
    Some(match card.engine.connection.state {
        ConnectionState::Expired => AvailabilityReasonCode::SignInExpired,
        ConnectionState::SignedIn | ConnectionState::NotApplicable => {
            // Availability is Enabled and the connection is fine, so the only
            // remaining gate is the missing `Ask` grant. That is an engine
            // that exists but has not been armed for use — never a credential
            // problem, so it must not read as one.
            AvailabilityReasonCode::NotArmed
        }
        _ => AvailabilityReasonCode::NotSignedIn,
    })
}
