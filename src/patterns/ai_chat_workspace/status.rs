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
    /// The metered cost, when the backend reported one. `None` means the
    /// backend did not report a cost for this usage — never that the cost
    /// was zero.
    pub metered_cost: Option<f64>,
}

impl UsageTotals {
    /// Accumulates a turn's plan-covered usage.
    pub fn add_plan(&mut self, u: &Usage) {
        self.plan = add_usage(self.plan, u);
    }

    /// Accumulates a turn's metered usage and its cost. An absent `cost`
    /// stays absent unless a prior call already reported one; two reported
    /// costs are summed, exactly like the usage counters they describe.
    pub fn add_metered(&mut self, u: &Usage, cost: Option<f64>) {
        self.metered = add_usage(self.metered, u);
        self.metered_cost = match (self.metered_cost, cost) {
            (None, None) => None,
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(a), Some(b)) => Some(a + b),
        };
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
