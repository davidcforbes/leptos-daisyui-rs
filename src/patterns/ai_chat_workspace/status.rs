//! One turn's lifecycle, usage and throughput, and the accumulator that
//! keeps plan-covered and metered usage from ever being added together.

use crate::components::ai_assistant_workspace::{AnswerOutcome, AttemptLifecycle};
use crate::components::ai_chat::Usage;

use super::evidence::TurnEvidence;

fn add_usage(a: Usage, b: &Usage) -> Usage {
    Usage {
        cost_usd: a.cost_usd + b.cost_usd,
        input_tokens: a.input_tokens + b.input_tokens,
        output_tokens: a.output_tokens + b.output_tokens,
        reasoning_tokens: a.reasoning_tokens + b.reasoning_tokens,
        cache_read_tokens: a.cache_read_tokens + b.cache_read_tokens,
        cache_creation_tokens: a.cache_creation_tokens + b.cache_creation_tokens,
    }
}

/// Plan-covered and metered usage, kept in separate fields forever. They
/// answer different questions ("how much of my plan did this use?" vs "how
/// much will I be billed?") and a workspace that added them together would
/// misreport both.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UsageTotals {
    /// Usage covered by the actor's plan.
    pub plan: Usage,
    /// Usage billed separately (pay-as-you-go).
    pub metered: Usage,
    /// The sum of every metered cost actually reported so far. `None` until
    /// the first contribution that reports one. Once at least one
    /// contribution lacked a cost, this number is a floor, not a total —
    /// see [`Self::cost_incomplete`], which a renderer MUST check before
    /// presenting this as "the cost" rather than "at least this much".
    pub metered_cost: Option<f64>,
    /// True once at least one call to [`Self::add_metered`] reported
    /// `cost: None`. Deliberately sticky (never clears back to `false`): a
    /// later contribution that DOES report a cost does not retroactively make
    /// the earlier unknown one known, so a partial total must keep reading as
    /// partial for the rest of this turn's life. Absent this flag,
    /// `(None, Some(a))` and `(Some(a), Some(a))` would both resolve to
    /// `Some(a)` and be indistinguishable — a complete total and an
    /// accidental floor would render identically.
    pub cost_incomplete: bool,
}

impl UsageTotals {
    /// Accumulates a turn's plan-covered usage.
    pub fn add_plan(&mut self, u: &Usage) {
        self.plan = add_usage(self.plan, u);
    }

    /// Accumulates a turn's metered usage and its cost. A reported cost is
    /// added into the running [`Self::metered_cost`]; an absent one leaves
    /// the running sum untouched but permanently sets
    /// [`Self::cost_incomplete`], so the accumulated number is never
    /// mistaken for a complete total once any contribution was unpriced.
    pub fn add_metered(&mut self, u: &Usage, cost: Option<f64>) {
        self.metered = add_usage(self.metered, u);
        match cost {
            Some(reported) => {
                self.metered_cost = Some(self.metered_cost.unwrap_or(0.0) + reported);
            }
            None => {
                self.cost_incomplete = true;
            }
        }
    }

    /// Output tokens per second of wall time, or `None` when the elapsed
    /// time is not strictly positive (unknown or zero-length), rather than
    /// a misleading zero or an infinite rate.
    pub fn tokens_per_sec(output_tokens: u64, elapsed_ms: i64) -> Option<f64> {
        if elapsed_ms <= 0 {
            return None;
        }
        Some(output_tokens as f64 / (elapsed_ms as f64 / 1000.0))
    }
}

/// One turn's accepted lifecycle, usage and evidence, keyed by the host's
/// turn id.
#[derive(Clone, Debug, PartialEq)]
pub struct TurnRecord {
    /// Stable turn identity.
    pub id: String,
    /// Which engine ran this turn.
    pub engine_id: String,
    /// The accepted durable lifecycle.
    pub lifecycle: AttemptLifecycle,
    /// Accumulated usage, when the backend reports it separately from the
    /// lifecycle's own answer.
    pub usage: Option<UsageTotals>,
    /// Precomputed throughput, when both an output count and an elapsed
    /// time were available.
    pub tokens_per_sec: Option<f64>,
    /// What backs the answer, when the turn completed.
    pub evidence: Option<TurnEvidence>,
    /// The completed outcome, mirrored out of `lifecycle` for convenience
    /// when a consumer only cares about outcome, not the full lifecycle.
    pub outcome: Option<AnswerOutcome>,
}

/// Stable id for one lifecycle state, for `data-*` hooks and telemetry.
/// `Completed` collapses to `"completed"` regardless of its nested outcome
/// (answered vs. declined) — that distinction is presentation, made by
/// `AiChatWorkspaceTexts::lifecycle_label`, not identity.
pub fn lifecycle_id(l: &AttemptLifecycle) -> &'static str {
    match l {
        AttemptLifecycle::Admitted => "admitted",
        AttemptLifecycle::Queued => "queued",
        AttemptLifecycle::Running => "running",
        AttemptLifecycle::Validating => "validating",
        AttemptLifecycle::Completed(_) => "completed",
        AttemptLifecycle::Denied { .. } => "denied",
        AttemptLifecycle::Unavailable { .. } => "unavailable",
        AttemptLifecycle::Failed { .. } => "failed",
        AttemptLifecycle::Canceled { .. } => "canceled",
        AttemptLifecycle::Interrupted { .. } => "interrupted",
        AttemptLifecycle::Unknown(_) => "unknown",
    }
}
