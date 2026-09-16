//! One turn's lifecycle, usage and throughput, and the accumulator that
//! keeps plan-covered and metered usage from ever being added together.

use crate::components::ai_assistant_workspace::{AnswerOutcome, AttemptLifecycle};
use crate::components::ai_chat::Usage;

use super::evidence::TurnEvidence;
use super::provider::ReasoningEffort;

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

/// Something that happened to a turn which the stream itself has no
/// vocabulary for: the engine silently changed its reasoning effort, or the
/// answer was cut short. `ai_chat`'s `StreamEvent` carries neither, so a
/// backend reports them on the turn record and the composite renders them
/// beside the transcript.
///
/// This is the data, not the wording. A renderer turns each notice into a
/// localized `TranscriptAnnotation` through `AiChatWorkspaceTexts`; the
/// backend never mints presentation text.
///
/// `#[non_exhaustive]`, because a host can learn of a new kind of notice
/// before this vocabulary does — that one arrives as [`Self::Unknown`]
/// carrying the host's own code, so it is rendered generically rather than
/// dropped.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TurnNotice {
    /// The engine ran the turn at a different reasoning effort than the one
    /// that was asked for.
    Escalated {
        /// The effort the turn was submitted at.
        from: ReasoningEffort,
        /// The effort the engine actually ran.
        to: ReasoningEffort,
    },
    /// The answer stopped short of finishing — a token budget or a context
    /// limit ended it, not the model.
    Truncated,
    /// A notice this vocabulary does not name, carried by the host's own
    /// code so a renderer can show it generically.
    Unknown(String),
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
    /// What the turn produced, which is NOT the same thing as "the turn
    /// finished".
    ///
    /// A completed turn mirrors its `lifecycle`'s own outcome here. A
    /// **canceled** turn also populates this field, with
    /// `AnswerOutcome::Answered` carrying whatever partial text had already
    /// streamed when the cancel landed — and with an EMPTY `text` when the
    /// cancel discarded the partial. That is the only route a canceled
    /// turn's partial has out of a backend, because
    /// `AttemptLifecycle::Canceled` carries nothing but a boolean.
    ///
    /// A renderer must therefore gate on [`Self::lifecycle`] BEFORE
    /// presenting this as an answer: see [`completed_answer`], which returns
    /// `None` for every non-completed lifecycle, and [`canceled_partial`],
    /// which returns the partial only for a canceled one and never returns
    /// an empty string. Reading this field directly renders a canceled turn
    /// as a finished answer, and a discarded one as an empty answer bubble.
    pub outcome: Option<AnswerOutcome>,
    /// What happened to this turn that the stream could not say, in the
    /// order it happened. Empty for an ordinary turn. A composite holding a
    /// `dyn ChatWorkspaceBackend` reads these off the record — they are the
    /// only route escalation and truncation have to the transcript.
    pub notices: Vec<TurnNotice>,
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

/// The answer text a turn may be presented with as a FINISHED answer, or
/// `None`.
///
/// Gated on [`TurnRecord::lifecycle`] rather than on
/// [`TurnRecord::outcome`]'s shape, because a canceled turn also carries an
/// `AnswerOutcome::Answered` (its surviving partial). Returning that text
/// here would render a turn the actor stopped as though the engine had
/// finished it.
pub fn completed_answer(record: &TurnRecord) -> Option<&str> {
    match (&record.lifecycle, &record.outcome) {
        (AttemptLifecycle::Completed(_), Some(AnswerOutcome::Answered { text })) => {
            Some(text.as_str())
        }
        _ => None,
    }
}

/// The partial text a CANCELED turn kept, or `None`.
///
/// `None` covers both "this turn was not canceled" and "the cancel discarded
/// the partial", which the backend reports as `Answered { text: "" }`. An
/// empty string is never returned, so a renderer cannot emit an empty bubble
/// for a discarded cancel by forwarding this value.
pub fn canceled_partial(record: &TurnRecord) -> Option<&str> {
    match (&record.lifecycle, &record.outcome) {
        (AttemptLifecycle::Canceled { .. }, Some(AnswerOutcome::Answered { text }))
            if !text.is_empty() =>
        {
            Some(text.as_str())
        }
        _ => None,
    }
}
