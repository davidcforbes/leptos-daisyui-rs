//! Opinionated responsive row of independent stat cards -- the Layer-2
//! `KpiStrip` pattern named in Future-Architecture.md.
//!
//! daisyUI's low-level [`Stats`](crate::components::Stats)/
//! [`Stat`](crate::components::Stat) pair renders a *joined* strip: a
//! shared background and internal dividers, so eight metrics read as one
//! table row rather than eight cards. `KpiStrip` builds independent
//! bordered, shadowed boxes in a responsive CSS grid instead -- each
//! `KpiCard` owns its own background and border, and the grid wraps into a
//! balanced layout at narrower widths without a horizontal scroll, a
//! collapsed gap, or unequal card sizes. `Stats`/`Stat` are unchanged and
//! remain independently usable; reach for them directly when daisyUI's own
//! joined presentation is actually what's wanted.
//!
//! Card depth is the framework's own static policy, not a stock Tailwind
//! utility: see [`kpi_card_shell_class`] and `ld-card-depth` (ldui-k4fn).

use crate::components::{
    CapacityBar, CapacityBarColor, Pressable, StatDeltaTrend, Tooltip, capacity_bar_percent,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// Reactive framework-owned copy for `KpiStrip`/`KpiCard`'s own generated
/// text -- the unavailable-value fallback, the trend-direction words folded
/// into each card's accessible name, and every sentence the baseline
/// comparison row generates. Caller-supplied [`KpiItem`] text
/// (label/value/description/help, the baseline's own name, an action's
/// label) is not covered here: localize it by rebuilding the `items` list
/// for the active locale, the same as any other reactive prop in this
/// crate.
///
/// ### Comparison templates
///
/// The five `baseline_*` fields are templates, not finished sentences.
/// Three placeholders are substituted before rendering:
///
/// - `{ratio}` -- current as a percentage OF the baseline, e.g. `112`.
/// - `{delta}` -- the unsigned deviation FROM the baseline in percentage
///   points, e.g. `12`. Which side it falls on is carried by *which*
///   template is chosen, so the number itself is never signed.
/// - `{baseline}` -- the caller's own [`KpiBaseline::label`], e.g.
///   `"12-week avg / 250"`.
///
/// In the no-baseline and settling templates there is no ratio and no
/// deviation, so `{ratio}` and `{delta}` substitute to `unavailable` rather
/// than to a fabricated `0` (ldui-ztgo).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KpiStripTexts {
    /// Shown -- and spoken via the accessible name -- in place of a `None`
    /// value.
    pub unavailable: String,
    /// Trend word for [`StatDeltaTrend::Positive`].
    pub trend_up: String,
    /// Trend word for [`StatDeltaTrend::Negative`].
    pub trend_down: String,
    /// Trend word for [`StatDeltaTrend::Neutral`].
    pub trend_steady: String,
    /// The truthful ratio readout rendered beside the comparison bar --
    /// UNBOUNDED, so it still reads `312%` when the bar itself has
    /// saturated. Default `"{ratio}%"`.
    pub baseline_ratio: String,
    /// Sentence for a value above its baseline. Default
    /// `"{delta}% above baseline"`.
    pub baseline_above: String,
    /// Sentence for a value below its baseline. Default
    /// `"{delta}% below baseline"`.
    pub baseline_below: String,
    /// Sentence for a value that rounds to its baseline. Default
    /// `"In line with baseline"`.
    pub baseline_level: String,
    /// Sentence for [`KpiBaselineAvailability::Absent`] -- and for a
    /// baseline the arithmetic cannot use. Default `"No baseline yet"`.
    pub baseline_absent: String,
    /// Sentence for [`KpiBaselineAvailability::Settling`] -- a baseline
    /// window that exists but is not yet full. Default
    /// `"Baseline still settling"`.
    pub baseline_settling: String,
}

impl Default for KpiStripTexts {
    fn default() -> Self {
        Self {
            unavailable: "Unavailable".to_owned(),
            trend_up: "trending up".to_owned(),
            trend_down: "trending down".to_owned(),
            trend_steady: "steady".to_owned(),
            baseline_ratio: "{ratio}%".to_owned(),
            baseline_above: "{delta}% above baseline".to_owned(),
            baseline_below: "{delta}% below baseline".to_owned(),
            baseline_level: "In line with baseline".to_owned(),
            baseline_absent: "No baseline yet".to_owned(),
            baseline_settling: "Baseline still settling".to_owned(),
        }
    }
}

impl KpiStripTexts {
    /// Resolves the localized trend word for a direction.
    fn trend_word(&self, direction: StatDeltaTrend) -> &str {
        match direction {
            StatDeltaTrend::Positive => &self.trend_up,
            StatDeltaTrend::Negative => &self.trend_down,
            StatDeltaTrend::Neutral => &self.trend_steady,
        }
    }

    /// The raw template for a resolved comparison state.
    fn baseline_template(&self, state: KpiBaselineState) -> &str {
        match state {
            KpiBaselineState::Above => &self.baseline_above,
            KpiBaselineState::Below => &self.baseline_below,
            KpiBaselineState::Level => &self.baseline_level,
            KpiBaselineState::NoBaseline => &self.baseline_absent,
            KpiBaselineState::Settling => &self.baseline_settling,
        }
    }

    /// The localized comparison sentence for a resolved comparison.
    ///
    /// Always returns a sentence: every one of the five states has its own
    /// template, so an unavailable baseline is DESCRIBED rather than
    /// silently omitted.
    pub fn baseline_sentence(&self, comparison: &KpiComparison, baseline_label: &str) -> String {
        self.fill_template(
            self.baseline_template(comparison.state),
            comparison,
            baseline_label,
        )
    }

    /// The localized truthful ratio readout, or `None` when the comparison
    /// carries no ratio to be truthful about.
    pub fn baseline_ratio_readout(
        &self,
        comparison: &KpiComparison,
        baseline_label: &str,
    ) -> Option<String> {
        comparison
            .ratio_percent
            .map(|_| self.fill_template(&self.baseline_ratio, comparison, baseline_label))
    }

    /// Substitutes `{ratio}`, `{delta}` and `{baseline}`. An absent number
    /// becomes `unavailable`, never a fabricated zero.
    fn fill_template(
        &self,
        template: &str,
        comparison: &KpiComparison,
        baseline_label: &str,
    ) -> String {
        let ratio = comparison
            .ratio_percent
            .map_or_else(|| self.unavailable.clone(), |value| value.to_string());
        let delta = comparison
            .deviation_percent
            .map_or_else(|| self.unavailable.clone(), |value| value.abs().to_string());
        template
            .replace("{ratio}", &ratio)
            .replace("{delta}", &delta)
            .replace("{baseline}", baseline_label)
    }
}

/// Semantic emphasis for one [`KpiCard`].
///
/// Drives the value text color and a left accent edge together, so the
/// two never disagree. `Neutral` renders no stripe at all -- a structural
/// cue, not only a color one, matching this crate's "never color-only"
/// posture elsewhere (e.g. `RosterGrid`'s state bar).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KpiStatus {
    /// No semantic emphasis (default).
    #[default]
    Neutral,
    /// Informational.
    Info,
    /// Good / on target.
    Success,
    /// Needs attention.
    Warning,
    /// Bad / off target.
    Error,
}

impl KpiStatus {
    /// Value text color class. Empty for `Neutral`.
    fn value_text_class(self) -> &'static str {
        match self {
            KpiStatus::Neutral => "",
            KpiStatus::Info => "text-info",
            KpiStatus::Success => "text-success",
            KpiStatus::Warning => "text-warning",
            KpiStatus::Error => "text-error",
        }
    }

    /// Left accent edge background class.
    ///
    /// `Neutral` is the DEFAULT and paints the house dark blue
    /// (`--color-status-blue`, `ui_tokens::color::STATUS_BLUE_FG`), not
    /// nothing: every card carries an accent edge, and a status is what
    /// OVERRIDES it when one card needs to stand out from the rest. An
    /// accent that only appears on exceptional cards would make the edge
    /// itself the signal; making it universal means the COLOUR is the
    /// signal, which is what lets one warning card read against seven
    /// ordinary ones.
    ///
    /// Deliberately the generic status blue rather than
    /// `color::table::HEADER`. They are the same hex today, but the table
    /// module exists so the table role can drift independently, and a card
    /// accent is not a table role.
    fn accent_bg_class(self) -> &'static str {
        match self {
            KpiStatus::Neutral => "bg-status-blue",
            KpiStatus::Info => "bg-info",
            KpiStatus::Success => "bg-success",
            KpiStatus::Warning => "bg-warning",
            KpiStatus::Error => "bg-error",
        }
    }

    /// Stable runtime marker, emitted as `data-kpi-card-status` so a test or
    /// a consumer can read a card's status WITHOUT reading its colour --
    /// [`RecordStatusTone::as_str`](super::RecordStatusTone::as_str)'s
    /// posture, adopted here for the same reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            KpiStatus::Neutral => "neutral",
            KpiStatus::Info => "info",
            KpiStatus::Success => "success",
            KpiStatus::Warning => "warning",
            KpiStatus::Error => "error",
        }
    }

    /// Fill colour for this card's baseline comparison bar.
    ///
    /// The bar wears the card's TYPED status and nothing else. It must not
    /// derive its colour from the numbers -- `CapacityBarColor::for_direction`
    /// exists and would paint at-or-above green and below red, which asserts
    /// that higher is better. For "days to close" or "cost per matter",
    /// higher is worse, and the framework has no way to know which it is
    /// looking at (ldui-ztgo). Favourable/unfavourable is the caller's
    /// [`KpiStatus`] to declare.
    ///
    /// `Neutral` takes `CapacityBarColor::Primary`, `CapacityBar`'s own
    /// documented default fill, rather than inventing an emphasis the card
    /// does not have.
    fn comparison_bar_color(self) -> CapacityBarColor {
        match self {
            KpiStatus::Neutral => CapacityBarColor::Primary,
            KpiStatus::Info => CapacityBarColor::Info,
            KpiStatus::Success => CapacityBarColor::Success,
            KpiStatus::Warning => CapacityBarColor::Warning,
            KpiStatus::Error => CapacityBarColor::Error,
        }
    }
}

/// Optional trend indicator for a [`KpiItem`] -- the same up/down/steady
/// vocabulary as [`StatDelta`](crate::components::StatDelta), reused here
/// rather than duplicated so the two agree on what "positive" means for a
/// given metric.
#[derive(Clone, Debug, PartialEq)]
pub struct KpiTrend {
    /// Magnitude, e.g. `12.5` for "12.5%". Sign is ignored -- `direction`
    /// conveys whether the change is good or bad, the same reasoning
    /// `StatDelta::value` documents.
    pub value: f64,
    /// Semantic direction.
    pub direction: StatDeltaTrend,
    /// Optional trailing label, e.g. `"vs last week"`. Renders nothing
    /// when empty.
    pub label: String,
}

impl KpiTrend {
    /// Creates a trend with no trailing label.
    pub fn new(value: f64, direction: StatDeltaTrend) -> Self {
        Self {
            value,
            direction,
            label: String::new(),
        }
    }

    /// Sets the trailing label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// Headroom the comparison track carries beyond the baseline, so the
/// baseline marker sits at a FIXED position rather than wherever the current
/// value happens to push it.
///
/// The track's right edge is `baseline * 1.25`, so the marker lands at
/// exactly `1 / 1.25` = 80% of the track on **every** card in a strip. That
/// fixed position is what makes "over baseline" legible across twelve cards
/// at a glance: the eye compares fill ends against one shared tick, not
/// against a tick that moved because one card's value was large.
///
/// Letting `CapacityBar` compute its own default max would defeat this --
/// its default is `cap * 1.25` *clamped up to at least `value`*, so a card at
/// 300% of baseline would rescale its own track and drop the marker to 33%
/// while its neighbour kept it at 80%.
pub const KPI_BASELINE_TRACK_HEADROOM: f64 = 1.25;

/// Whether a [`KpiBaseline`] actually has a baseline to compare against.
///
/// Three DECLARED states, never inferred from a sentinel number. "There is
/// no baseline", "the baseline window is still filling", and "the baseline
/// is 250" are three different facts about the data, and a caller that
/// cannot distinguish them in the type system ends up encoding the first two
/// as `0.0` -- which is exactly how a dashboard comes to divide by zero and
/// print `inf%`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KpiBaselineAvailability {
    /// A baseline value to compare against. Only usable when it is finite
    /// and strictly positive; see [`KpiBaseline::resolve`] for what happens
    /// otherwise.
    Available(f64),
    /// There is no baseline for this KPI -- no history, a brand-new metric,
    /// a scope with no prior period.
    Absent,
    /// A baseline window exists but is not yet full, so the average it would
    /// produce is not yet meaningful. Distinct from [`Self::Absent`]: the
    /// answer is "not yet", not "never".
    Settling,
}

/// Which sentence a resolved comparison speaks.
///
/// `Above`/`Level`/`Below` are decided from the ROUNDED ratio, so the
/// direction word can never disagree with the percentage printed beside it:
/// a value 0.16% over its baseline rounds to `100%` and therefore reads
/// "in line with", not "0% above".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KpiBaselineState {
    /// Current rounds above the baseline.
    Above,
    /// Current rounds to the baseline.
    Level,
    /// Current rounds below the baseline.
    Below,
    /// No comparison is possible and none is drawn.
    #[default]
    NoBaseline,
    /// The baseline window is still filling; no comparison is drawn.
    Settling,
}

impl KpiBaselineState {
    /// Stable runtime marker, emitted as `data-kpi-baseline-state`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Above => "above",
            Self::Level => "level",
            Self::Below => "below",
            Self::NoBaseline => "no-baseline",
            Self::Settling => "settling",
        }
    }

    /// Whether this state carries a bar and a percentage at all.
    pub const fn is_comparable(self) -> bool {
        matches!(self, Self::Above | Self::Level | Self::Below)
    }
}

/// The fully resolved outcome of one [`KpiBaseline`] -- pure data, computed
/// by [`KpiBaseline::resolve`] and rendered by [`KpiCard`].
///
/// Split out from rendering on purpose: every number below is a plain
/// function of two `f64`s and an availability, so over-baseline
/// truthfulness, zero/absent/settling handling and bar clamping are all
/// testable natively without a browser.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KpiComparison {
    /// Which sentence to speak.
    pub state: KpiBaselineState,
    /// Current as a percentage OF the baseline, rounded to a whole
    /// percent and **deliberately unbounded** -- `312` stays `312`.
    ///
    /// This is the half of the pair that stays truthful when
    /// [`Self::saturated`] pins the bar to the end of its track. `None`
    /// whenever [`Self::state`] is not comparable.
    pub ratio_percent: Option<i64>,
    /// Signed deviation from the baseline in whole percentage points --
    /// exactly `ratio_percent - 100`, so the two can never disagree by a
    /// rounding step. `None` whenever the state is not comparable.
    pub deviation_percent: Option<i64>,
    /// The usable baseline the comparison was computed against -- finite
    /// and strictly positive, or `None` when the state is not comparable.
    /// This is the bar's cap-line position in data units.
    pub baseline_value: Option<f64>,
    /// The right edge of the comparison track in data units, i.e.
    /// `baseline_value * KPI_BASELINE_TRACK_HEADROOM`. Passed to
    /// `CapacityBar` as an explicit `max` so the marker cannot move.
    pub track_max: Option<f64>,
    /// Where the fill ends, as a percentage of the track, clamped to
    /// `[0, 100]`. This is the BOUNDED half of the pair.
    pub fill_percent: f64,
    /// Where the baseline marker sits, as a percentage of the track.
    /// `80.0` for every comparable card (see
    /// [`KPI_BASELINE_TRACK_HEADROOM`]), `0.0` when nothing is drawn.
    pub marker_percent: f64,
    /// The value ran past the end of the track, so the bar is pinned at
    /// 100% while [`Self::ratio_percent`] keeps reporting the real figure.
    ///
    /// Emitted as `data-kpi-baseline-saturated` precisely so "the bar is
    /// full" and "the value is at the cap" stay distinguishable to a test,
    /// a consumer, and anyone reading the DOM.
    pub saturated: bool,
    /// The caller declared [`KpiBaselineAvailability::Available`] but handed
    /// over a number the arithmetic cannot use -- zero, negative, `NaN`, an
    /// infinity, or a non-finite current value.
    ///
    /// The card degrades to the no-baseline presentation, but LOUDLY: this
    /// flag is emitted as `data-kpi-baseline-degraded`, so a silent
    /// fabricated `0%` is not what the consumer discovers six months later.
    pub degraded: bool,
}

impl KpiComparison {
    /// The unavailable-comparison outcome: no bar, no percentage.
    const fn unavailable(state: KpiBaselineState, degraded: bool) -> Self {
        Self {
            state,
            ratio_percent: None,
            deviation_percent: None,
            baseline_value: None,
            track_max: None,
            fill_percent: 0.0,
            marker_percent: 0.0,
            saturated: false,
            degraded,
        }
    }
}

/// A current-versus-baseline comparison owned by one [`KpiItem`] -- the
/// typed model behind the card's comparison row.
///
/// ```rust
/// use leptos_daisyui_rs::patterns::{KpiBaseline, KpiBaselineState};
///
/// // 280 against a trailing 12-week average of 250.
/// let baseline = KpiBaseline::against(280.0, 250.0).label("12-week avg / 250");
/// let resolved = baseline.resolve();
/// assert_eq!(resolved.state, KpiBaselineState::Above);
/// assert_eq!(resolved.ratio_percent, Some(112));
/// assert_eq!(resolved.deviation_percent, Some(12));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct KpiBaseline {
    /// The current measured value, in the baseline's own units. Raw and
    /// unformatted: [`KpiItem::value`] carries the formatted string a human
    /// reads, this carries the number the comparison is computed from.
    pub current: f64,
    /// Whether there is a baseline, and what it is.
    pub availability: KpiBaselineAvailability,
    /// The caller's own localized name for the baseline, e.g.
    /// `"12-week avg / 250"`. Rendered beside the bar and substituted for
    /// `{baseline}` in [`KpiStripTexts`]' templates. Renders nothing when
    /// empty.
    pub label: String,
}

impl KpiBaseline {
    /// A comparison of `current` against `baseline`.
    pub fn against(current: f64, baseline: f64) -> Self {
        Self {
            current,
            availability: KpiBaselineAvailability::Available(baseline),
            label: String::new(),
        }
    }

    /// A KPI that has no baseline at all.
    pub fn absent(current: f64) -> Self {
        Self {
            current,
            availability: KpiBaselineAvailability::Absent,
            label: String::new(),
        }
    }

    /// A KPI whose baseline window is still filling.
    pub fn settling(current: f64) -> Self {
        Self {
            current,
            availability: KpiBaselineAvailability::Settling,
            label: String::new(),
        }
    }

    /// Sets the caller's localized baseline name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Resolves this baseline into the numbers the card renders.
    ///
    /// The two halves of the honesty contract are produced here, together,
    /// so they cannot drift apart:
    ///
    /// - [`KpiComparison::fill_percent`] is BOUNDED. It goes through
    ///   [`capacity_bar_percent`], the very function `CapacityBar` uses, and
    ///   is therefore clamped to `[0, 100]` exactly as the rendered bar is.
    /// - [`KpiComparison::ratio_percent`] is UNBOUNDED and is never derived
    ///   from the fill. A value at 312% of baseline reports `312` while the
    ///   bar sits at 100% and [`KpiComparison::saturated`] says so.
    ///
    /// The marker never moves and never disappears: it is pinned at 80% of
    /// the track for every comparable card, so a full bar reads as "well
    /// past the tick", never as "exactly at the cap".
    pub fn resolve(&self) -> KpiComparison {
        let baseline = match self.availability {
            KpiBaselineAvailability::Absent => {
                return KpiComparison::unavailable(KpiBaselineState::NoBaseline, false);
            }
            KpiBaselineAvailability::Settling => {
                return KpiComparison::unavailable(KpiBaselineState::Settling, false);
            }
            KpiBaselineAvailability::Available(baseline) => baseline,
        };

        // A declared baseline the arithmetic cannot use. Zero and negative
        // baselines are as unusable as NaN: `current / 0.0` is an infinity,
        // and "312% of -40" is not a sentence anyone can act on. Degrade to
        // the no-baseline presentation, and FLAG the degradation.
        if !baseline.is_finite() || baseline <= 0.0 || !self.current.is_finite() {
            return KpiComparison::unavailable(KpiBaselineState::NoBaseline, true);
        }

        let max = baseline * KPI_BASELINE_TRACK_HEADROOM;
        let ratio_percent = (self.current / baseline * 100.0).round() as i64;
        let deviation_percent = ratio_percent - 100;
        let state = match deviation_percent.signum() {
            1 => KpiBaselineState::Above,
            -1 => KpiBaselineState::Below,
            _ => KpiBaselineState::Level,
        };

        KpiComparison {
            state,
            ratio_percent: Some(ratio_percent),
            deviation_percent: Some(deviation_percent),
            baseline_value: Some(baseline),
            track_max: Some(max),
            fill_percent: capacity_bar_percent(self.current, max),
            marker_percent: capacity_bar_percent(baseline, max),
            saturated: self.current > max,
            degraded: false,
        }
    }
}

/// A caller-owned, localized activation affordance for one [`KpiItem`].
///
/// Presence of this struct is only HALF of what makes a card activatable:
/// the other half is an `on_activate` callback on [`KpiCard`]/[`KpiStrip`].
/// A card missing either one renders exactly as it did before this type
/// existed -- no button, no tab stop, no `data-kpi-card-activatable`. See
/// [`kpi_card_is_activatable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KpiAction {
    /// The control's visible label, e.g. `"View details"`. Caller-owned and
    /// caller-localized, like every other [`KpiItem`] string.
    pub label: String,
    /// Optional fuller accessible name. When empty, the framework builds
    /// one as `"<label>, <the card's accessible name>"`, which keeps the
    /// visible label as the accessible name's prefix (WCAG 2.5.3 Label in
    /// Name) while still telling a screen-reader user WHICH of twelve
    /// identically-labelled buttons this is.
    pub accessible_label: String,
    /// Whether the action is currently unavailable. Renders the native
    /// `disabled` attribute, so the control stays in the accessibility tree
    /// and out of the tab order.
    pub disabled: bool,
}

impl KpiAction {
    /// An enabled action with a visible label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            accessible_label: String::new(),
            disabled: false,
        }
    }

    /// Overrides the generated accessible name.
    pub fn accessible_label(mut self, accessible_label: impl Into<String>) -> Self {
        self.accessible_label = accessible_label.into();
        self
    }

    /// Marks the action unavailable.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// One KPI's content -- `KpiStrip`'s opinionated typed item model.
///
/// Plain owned data, not a `Signal`-bearing struct: the whole `items` list
/// is itself reactive (`Signal<Vec<KpiItem>>` on [`KpiStrip`]), the same
/// posture as `ActiveFilterChip`/`DatasetOption` elsewhere in this module.
/// Rebuilding the list -- for a data refresh or a locale change -- is how
/// a `KpiStrip` updates.
#[derive(Clone, Debug, PartialEq)]
pub struct KpiItem {
    /// Stable identity, used for the list key and `data-kpi-card`.
    pub id: String,
    /// Card label (always rendered).
    pub label: String,
    /// Current value. `None` renders the unavailable presentation instead
    /// -- a muted placeholder, never a fabricated zero or empty string.
    pub value: Option<String>,
    /// Optional supporting copy. Renders nothing when empty.
    pub description: String,
    /// Optional semantic emphasis.
    pub status: KpiStatus,
    /// Optional trend indicator.
    pub trend: Option<KpiTrend>,
    /// Optional help text. Renders nothing when empty; exposed through
    /// `aria-describedby` so it reaches assistive tech even without a
    /// hover, not only through the visible tooltip trigger.
    ///
    /// **Costs the label 20px of row width.** The trigger is a flex sibling
    /// of the label, so a card carrying help gives its label 20px less than
    /// the same card without it (measured 83px against 63px on a 117px
    /// card). That matters because the label is clamped to two lines: a
    /// roughly 20-character label needs about 70px to hold two lines, so a
    /// help-bearing card wants to be about 125px wide or more before the
    /// label starts clipping. Ordinary strips are far above that -- a card
    /// is 117px at a 1680px window in a constrained column and 182px at
    /// 2200px -- so this only bites when cards are already cramped, which
    /// is the regime `ldui-tnyq` covers. Reach for a shorter label rather
    /// than a narrower card if you hit it (`ldui-yhvf`).
    pub help: String,
    /// Optional current-versus-baseline comparison, rendered as a bounded
    /// bar with a fixed baseline marker plus a truthful, unbounded
    /// percentage and a localized sentence (ldui-ztgo).
    ///
    /// Sits BELOW the value, never above it, so a card carrying a baseline
    /// and a card without one keep identical label and value offsets --
    /// the alignment contract `ldui-tbaw` established for the label's
    /// reserved two-line box is untouched by this row.
    pub baseline: Option<KpiBaseline>,
    /// Optional activation affordance. Renders only when the card ALSO
    /// receives an `on_activate` callback; see [`KpiAction`].
    pub action: Option<KpiAction>,
}

impl KpiItem {
    /// Creates an available KPI with a label and value.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: Some(value.into()),
            description: String::new(),
            status: KpiStatus::default(),
            trend: None,
            help: String::new(),
            baseline: None,
            action: None,
        }
    }

    /// Marks the value unavailable, clearing any previously set value.
    pub fn unavailable(mut self) -> Self {
        self.value = None;
        self
    }

    /// Sets the supporting description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the semantic status.
    pub fn status(mut self, status: KpiStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets the trend indicator.
    pub fn trend(mut self, trend: KpiTrend) -> Self {
        self.trend = Some(trend);
        self
    }

    /// Sets the help text.
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }

    /// Sets the current-versus-baseline comparison.
    pub fn baseline(mut self, baseline: KpiBaseline) -> Self {
        self.baseline = Some(baseline);
        self
    }

    /// Marks the item activatable with the given caller-owned action copy.
    ///
    /// Takes effect only once the card also has an `on_activate` callback.
    pub fn action(mut self, action: KpiAction) -> Self {
        self.action = Some(action);
        self
    }
}

/// Whether a card renders its activation control.
///
/// BOTH halves are required: the item's own [`KpiAction`] copy AND a
/// callback to run. A card with neither -- which is every card written
/// before ldui-ztgo -- takes the `false` branch, keeps `role="group"` with
/// no `tabindex`, renders no `<button>`, and is therefore not focusable and
/// does not announce as a control. A caller who supplies a callback but no
/// action copy gets no button either, because a framework-invented English
/// label would be exactly the unlocalized string this pattern refuses to
/// mint.
pub fn kpi_card_is_activatable(action: Option<&KpiAction>, has_callback: bool) -> bool {
    action.is_some() && has_callback
}

/// The activation control's accessible name.
///
/// Falls back to `"<visible label>, <card accessible name>"` so that the
/// visible label is always a PREFIX of the accessible name (WCAG 2.5.3), and
/// twelve cards' worth of identically-labelled "View details" buttons are
/// still individually identifiable in a screen reader's control list.
fn kpi_action_accessible_name(action: &KpiAction, card_accessible_name: &str) -> String {
    if has_text(&action.accessible_label) {
        return action.accessible_label.clone();
    }
    format!("{}, {}", action.label, card_accessible_name)
}

/// Whether optional copy should render at all -- mirrors
/// [`SectionHeading`](super::SectionHeading)'s `has_text`: an empty string
/// renders nothing, not an empty line.
fn has_text(value: &str) -> bool {
    !value.is_empty()
}

/// The reconciliation key for one card in a [`KpiStrip`].
///
/// Covers the WHOLE item, not just [`KpiItem::id`]. Keying on the id alone
/// would be wrong here in a way that is easy to ship and hard to notice:
/// [`KpiCard`] receives its item by value and holds no reactive signal over
/// it, so an id-keyed `For` would leave a card showing yesterday's number
/// after a refresh that changed the value but not the id -- and the
/// localized-strip case (same ids, translated labels) would never
/// re-render at all.
///
/// A `Debug` rendering is used because it is total by construction: any
/// field added to [`KpiItem`] later is included automatically, so this
/// cannot silently fall behind the struct it keys.
fn kpi_item_fingerprint(item: &KpiItem) -> String {
    format!("{item:?}")
}

/// The measured floor for a card that must hold a two-line label, in CSS
/// px.
///
/// Not a taste number: `ldui-tbaw`'s fit sweep found a roughly
/// 20-character label needs about 70px of label width to hold two lines,
/// and label width is roughly card width minus 34px of body padding plus
/// accent edge -- so about 104px bare, and `ldui-tnyq` shipped its ladder
/// at 114px and up after measuring the rendered DOM at each rung. 114 is
/// therefore the floor the framework has actually validated, and every
/// rung of every profile must clear it.
pub const KPI_CARD_TWO_LINE_FLOOR_PX: f64 = 114.0;

/// The measured floor for a card that ALSO carries a help control, in CSS
/// px.
///
/// A help trigger is a flex sibling of the label and takes 20px of the
/// label's row (`ldui-yhvf`: 83px of label width became 63px on a 117px
/// card), so a help-bearing card wants about 125px before the two-line
/// label starts clipping. Ordinary operational strips sit far above this;
/// a twelve-card scorecard at six columns is exactly the cramped regime
/// where it bites, which is why [`KpiStripLayout::BalancedSix`] puts its
/// six-column rung where it does.
pub const KPI_CARD_HELP_FLOOR_PX: f64 = 125.0;

/// One rung of a [`KpiStripLayout`]'s responsive column ladder.
///
/// The ladder is DATA, not a class string that happens to encode it: the
/// grid class is asserted against this table, so the documented arithmetic
/// and the emitted utilities cannot drift apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KpiStripRung {
    /// The Tailwind container-query variant prefix, e.g. `"@lg:"`. Empty
    /// for the base rung, which has no variant.
    pub prefix: &'static str,
    /// The container width in CSS px at or above which this rung applies.
    /// These are Tailwind v4's default container sizes: `@sm` 24rem/384px,
    /// `@lg` 32rem/512px, `@4xl` 56rem/896px, `@5xl` 64rem/1024px.
    pub min_container_px: u32,
    /// Explicit column tracks at this rung.
    pub columns: u32,
}

/// `2 / 3 / 4 / 8` -- the ladder every caller written before `ldui-k3ip`
/// gets, byte for byte unchanged.
const KPI_STRIP_AUTO_EIGHT_LADDER: &[KpiStripRung] = &[
    KpiStripRung {
        prefix: "",
        min_container_px: 0,
        columns: 2,
    },
    KpiStripRung {
        prefix: "@sm:",
        min_container_px: 384,
        columns: 3,
    },
    KpiStripRung {
        prefix: "@lg:",
        min_container_px: 512,
        columns: 4,
    },
    KpiStripRung {
        prefix: "@5xl:",
        min_container_px: 1024,
        columns: 8,
    },
];

/// `2 / 3 / 4 / 6` -- the balanced scorecard ladder.
///
/// Every rung divides twelve exactly (12 = 6x2 = 4x3 = 3x4 = 2x6), which
/// is the property that makes a twelve-card set stay a balanced peer group
/// at every width rather than only at the widest one.
const KPI_STRIP_BALANCED_SIX_LADDER: &[KpiStripRung] = &[
    KpiStripRung {
        prefix: "",
        min_container_px: 0,
        columns: 2,
    },
    KpiStripRung {
        prefix: "@sm:",
        min_container_px: 384,
        columns: 3,
    },
    KpiStripRung {
        prefix: "@lg:",
        min_container_px: 512,
        columns: 4,
    },
    KpiStripRung {
        prefix: "@4xl:",
        min_container_px: 896,
        columns: 6,
    },
];

/// `2 / 3` -- the fixed three-peer-summary ladder (`ldui-orom`).
///
/// Only two rungs, deliberately: the profile's widest rung IS its column
/// cap, so there is no intermediate 4-column rung to pass through the way
/// the shared `AutoEight`/`BalancedSix` trunk does -- a fourth track would
/// never hold a fourth peer.
const KPI_STRIP_PEER_THREE_LADDER: &[KpiStripRung] = &[
    KpiStripRung {
        prefix: "",
        min_container_px: 0,
        columns: 2,
    },
    KpiStripRung {
        prefix: "@lg:",
        min_container_px: 512,
        columns: 3,
    },
];

/// Which column ladder a [`KpiStrip`] follows -- the typed layout choice
/// (`ldui-k3ip`).
///
/// A NAMED INTENT, not a column count. An integer prop would let a caller
/// ask for twelve columns of 40px, would put breakpoint policy in the
/// consumer (the exact fork this opinionated layer exists to remove), and
/// would carry no answer for what happens at narrower widths -- the
/// framework owns the whole ladder down from the widest rung, so a profile
/// has to name the shape at the top and derive the rest. It is also
/// orthogonal to `compact`, which changes padding and gap and never the
/// column count.
///
/// ### The arithmetic behind each rung
///
/// Card width is `(container - gap * (columns - 1)) / columns`, with
/// `gap-4` = 16px (`gap-3` = 12px in compact mode, which is strictly more
/// generous). Every rung must clear [`KPI_CARD_TWO_LINE_FLOOR_PX`]:
///
/// | profile | rung | container | columns | card |
/// |---|---|---|---|---|
/// | `AutoEight`, `BalancedSix` | base | 320px | 2 | 152.0px |
/// | `AutoEight`, `BalancedSix` | `@sm` | 384px | 3 | 117.3px |
/// | `AutoEight`, `BalancedSix` | `@lg` | 512px | 4 | 116.0px |
/// | `AutoEight` | `@5xl` | 1024px | 8 | 114.0px |
/// | `BalancedSix` | `@4xl` | 896px | 6 | 136.0px |
/// | `PeerThree` | base | 320px | 2 | 152.0px |
/// | `PeerThree` | `@lg` | 512px | 3 | 160.0px |
///
/// `@4xl` rather than `@3xl` (768px), and that choice is the whole reason
/// the profile is a type: at 768px six columns are 114.7px, which clears
/// the bare two-line floor but NOT [`KPI_CARD_HELP_FLOOR_PX`]. A scorecard
/// is precisely where help-bearing cards live, so the six-column rung
/// starts where a help-bearing card still holds two label lines. `@4xl`
/// gives 136.0px, 11px of slack over that floor.
///
/// `PeerThree`'s ladder DIVERGES from the shared `AutoEight`/`BalancedSix`
/// trunk at its very first rung rather than following it up through 3 and
/// 4 columns before capping: a fixed three-peer row can never render a
/// fourth track -- there is no fourth peer to fill it, and CSS Grid would
/// simply leave it empty, which is exactly the "half the row is dead
/// space" shape `ldui-orom` reports for `BalancedSix`. So the base rung
/// (2 columns, matching the "never a single full-bleed column" floor every
/// profile shares) steps directly to the profile's own widest rung, 3
/// columns, at `@lg` (512px) -- clearing both
/// [`KPI_CARD_TWO_LINE_FLOOR_PX`] and [`KPI_CARD_HELP_FLOOR_PX`] with room
/// to spare.
///
/// ### What it does to the consumer's measured strip
///
/// At the 1046px container `ldui-tnyq` measured on a 1680px window,
/// `AutoEight` gives 8 columns of 116.8px and lays twelve cards out as
/// eight then a ragged four; `BalancedSix` gives 6 columns of 161.0px and
/// lays them out as two rows of six. The balanced profile's cards are
/// WIDER, so nothing inside them -- a two-line label, a help trigger, a
/// baseline comparison bar -- gets tighter by choosing it.
///
/// At the 1617.6px container `ldui-orom` reports (a 1696px viewport), three
/// items through `AutoEight` render 8 tracks of 188.2px cards and occupy
/// only the first 596.6px of the row; through `BalancedSix` they render 6
/// tracks of roughly 256.3px cards and occupy roughly the first 800.8px,
/// still under half the row. Through `PeerThree` the same three items
/// render exactly 3 tracks of roughly 528.5px cards and span the full
/// 1617.6px row -- the profile's widest rung is also its item count's
/// natural column count, so three peers are never fewer than the tracks
/// provided for them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KpiStripLayout {
    /// `2 / 3 / 4 / 8`. The default and the pre-`ldui-k3ip` behaviour: an
    /// operational strip that fills a full row of up to eight short cards
    /// once it is wide enough.
    #[default]
    AutoEight,
    /// `2 / 3 / 4 / 6`. A balanced fixed dashboard scorecard: twelve peer
    /// cards read as two rows of six, six as one row of six.
    BalancedSix,
    /// `2 / 3`. A fixed row of exactly three peer summaries -- never more
    /// tracks than there are peers to fill them, so the row is never left
    /// half empty the way `AutoEight` and `BalancedSix` both leave it for a
    /// three-item strip (`ldui-orom`).
    PeerThree,
}

impl KpiStripLayout {
    /// Stable runtime marker, emitted as `data-kpi-strip-layout` so a test
    /// or a consumer can read the active profile WITHOUT parsing the grid's
    /// utility classes -- [`KpiStatus::as_str`]'s posture, for the same
    /// reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AutoEight => "auto-eight",
            Self::BalancedSix => "balanced-six",
            Self::PeerThree => "peer-three",
        }
    }

    /// This profile's responsive column ladder, widest rung last.
    pub const fn ladder(self) -> &'static [KpiStripRung] {
        match self {
            Self::AutoEight => KPI_STRIP_AUTO_EIGHT_LADDER,
            Self::BalancedSix => KPI_STRIP_BALANCED_SIX_LADDER,
            Self::PeerThree => KPI_STRIP_PEER_THREE_LADDER,
        }
    }

    /// The widest column count this profile ever reaches.
    pub const fn max_columns(self) -> u32 {
        match self {
            Self::AutoEight => 8,
            Self::BalancedSix => 6,
            Self::PeerThree => 3,
        }
    }

    /// How many columns this profile renders in a container of the given
    /// width -- the Rust mirror of what the container queries do in the
    /// browser, so a native test can assert geometry rather than class
    /// names.
    pub fn columns_at(self, container_px: f64) -> u32 {
        self.ladder()
            .iter()
            .filter(|rung| container_px >= f64::from(rung.min_container_px))
            .map(|rung| rung.columns)
            .next_back()
            .unwrap_or(1)
    }
}

/// The grid gap in CSS px: `gap-4` normally, `gap-3` in compact mode.
pub const fn kpi_strip_gap_px(compact: bool) -> f64 {
    if compact { 12.0 } else { 16.0 }
}

/// One card's rendered width in a strip of the given container width,
/// column count and gap.
///
/// Pure geometry, exported so a consumer sizing a dashboard column can ask
/// the same question the framework's own rung derivation asks instead of
/// guessing.
pub fn kpi_strip_card_width_px(container_px: f64, columns: u32, gap_px: f64) -> f64 {
    if columns == 0 {
        return 0.0;
    }
    (container_px - gap_px * f64::from(columns - 1)) / f64::from(columns)
}

/// How a list of items lands in a fixed number of column tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KpiStripRowFit {
    /// Explicit column tracks at the width being asked about.
    pub columns: u32,
    /// Rows that are completely filled.
    pub full_rows: u32,
    /// Cards in the final, possibly short, row. Zero only when there are no
    /// items at all.
    pub last_row: u32,
}

impl KpiStripRowFit {
    /// The last row is short, so the strip ends on a ragged edge.
    pub const fn is_ragged(self) -> bool {
        self.last_row > 0 && self.last_row < self.columns
    }

    /// Total rows rendered.
    pub const fn rows(self) -> u32 {
        self.full_rows + if self.last_row > 0 { 1 } else { 0 }
    }
}

/// How `items` cards land in `columns` tracks.
///
/// A count that does not divide evenly leaves a RAGGED LAST ROW, and that
/// is the deliberate behaviour, not an emergent one. The tracks are
/// explicit (`grid-cols-6`, never `auto-fit`/`auto-fill` with a minmax),
/// so seven cards in a balanced-six strip render six then one, and that
/// one keeps its own one-sixth track: it does not stretch across the row,
/// and its neighbours above it do not shrink. Equal card geometry is the
/// property being protected, so a short final row is preferred over
/// stretching -- a stretched last card would read as a different, more
/// important thing than its peers.
pub fn kpi_strip_row_fit(items: usize, columns: u32) -> KpiStripRowFit {
    if columns == 0 {
        return KpiStripRowFit {
            columns,
            full_rows: 0,
            last_row: 0,
        };
    }
    let items = u32::try_from(items).unwrap_or(u32::MAX);
    KpiStripRowFit {
        columns,
        full_rows: items / columns,
        last_row: items % columns,
    }
}

/// Responsive grid classes for the strip.
///
/// Two columns at the narrowest width (never a single full-bleed column,
/// which reads as a list rather than a grid of cards), growing to the
/// profile's widest rung once the STRIP is wide enough, not the window
/// (`ldui-tnyq`). When there are fewer items than columns, CSS Grid leaves
/// the remaining explicit-column tracks empty rather than stretching the
/// existing cards to fill them, so card size stays equal regardless of
/// count.
///
/// Composed from the profile's own [`KpiStripLayout::ladder`] -- pinned by
/// `kpi_strip_grid_class_is_composed_from_the_declared_ladder`, so a rung
/// cannot be edited in one place and left stale in the other.
const fn kpi_strip_grid_class(layout: KpiStripLayout, compact: bool) -> &'static str {
    match (layout, compact) {
        (KpiStripLayout::AutoEight, false) => {
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @5xl:grid-cols-8 gap-4"
        }
        (KpiStripLayout::AutoEight, true) => {
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @5xl:grid-cols-8 gap-3"
        }
        (KpiStripLayout::BalancedSix, false) => {
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @4xl:grid-cols-6 gap-4"
        }
        (KpiStripLayout::BalancedSix, true) => {
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @4xl:grid-cols-6 gap-3"
        }
        (KpiStripLayout::PeerThree, false) => "grid grid-cols-2 @lg:grid-cols-3 gap-4",
        (KpiStripLayout::PeerThree, true) => "grid grid-cols-2 @lg:grid-cols-3 gap-3",
    }
}

/// Card body padding/gap: internal spacing stays at or below the grid gap
/// (`p-4` <= `gap-4`, `p-3` <= `gap-3`) so cards never read as a single
/// group with their neighbours.
fn kpi_card_body_class(compact: bool) -> &'static str {
    // `min-w-0` matters: the body is a flex item beside the accent edge, and
    // without it the default `min-width: auto` lets a long unbroken value push
    // the card wider than its grid track.
    if compact {
        "flex min-w-0 flex-1 flex-col gap-1 p-3"
    } else {
        "flex min-w-0 flex-1 flex-col gap-2 p-4"
    }
}

/// The card shell: geometry, border, background, and the framework's static
/// card-elevation policy.
///
/// Depth comes from `ld-card-depth`, never a stock Tailwind `shadow-*`
/// utility (ldui-k4fn). The class is an authored rule generated into
/// `styles/tokens.css` (and mirrored by `UiTokensPreamble`) that paints
/// `var(--ld-card-shadow, var(--ld-elevation-4))`, so the resting depth is
/// `ui_tokens::elevation`'s declared "card resting elevation" and a product
/// theme can substitute its own approved card shadow by setting
/// `--ld-card-shadow` once, without forking the class or reaching into
/// KpiCard's markup with a descendant selector.
///
/// Deliberately not `ld-elevated`: that class lifts on hover and would make
/// a read-only tile look interactive.
fn kpi_card_shell_class() -> &'static str {
    "flex rounded-box border border-base-300 bg-base-100 ld-card-depth h-full min-w-0 overflow-hidden forced-colors:border-[CanvasText]"
}

/// Value type-ramp step: the large display size normally, stepping down
/// one rung in `compact` mode.
fn kpi_card_value_size_class(compact: bool) -> &'static str {
    if compact {
        "ld-text-title"
    } else {
        "ld-text-display"
    }
}

/// Label classes: a bounded two-line clamp, never single-line truncation
/// (ldui-tbaw). `ld-text-small`'s line height is exactly `1rem`
/// (`--text-small--line-height: calc(16 / 11)` times `--text-small:
/// 0.6875rem`), so `min-h-8` (32px/2rem, the canonical spacing scale's
/// `32` step) reserves precisely two line boxes regardless of whether the
/// label actually needs one line or two. That reservation -- not the
/// clamp alone -- is what keeps a one-line-label card and a two-line-label
/// card the same height in the same grid row, and keeps every card's value/
/// description/help control starting at the identical vertical offset:
/// clamping alone would still let a short label leave a shorter, unreserved
/// box than a wrapped one. Same size regardless of `compact`, since the
/// label's own font size does not change in compact mode.
fn kpi_card_label_class() -> &'static str {
    "ld-text-small font-semibold uppercase tracking-wide text-base-content/75 line-clamp-2 break-words min-h-8"
}

/// Builds the card's accessible name from its label, value (or the
/// unavailable fallback), and trend, so a screen reader announces one
/// coherent phrase for the card rather than reading unrelated child text
/// nodes.
fn kpi_card_accessible_name(
    label: &str,
    value: Option<&str>,
    trend: Option<&KpiTrend>,
    texts: &KpiStripTexts,
) -> String {
    let mut name = label.to_owned();
    name.push_str(": ");
    match value {
        Some(value) if !value.is_empty() => name.push_str(value),
        _ => name.push_str(&texts.unavailable),
    }
    if let Some(trend) = trend {
        name.push_str(", ");
        name.push_str(texts.trend_word(trend.direction));
    }
    name
}

/// One independent, self-contained KPI card -- `KpiStrip`'s opinionated
/// item presentation. Exported separately so a caller with an unusual
/// layout need (a single featured metric, a two-up summary) can reach for
/// it directly without `KpiStrip`'s grid.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{KpiCard, KpiItem, KpiStatus};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <KpiCard item=KpiItem::new("open-tickets", "Open tickets", "42")
///             .description("Across every queue")
///             .status(KpiStatus::Warning) />
///     }
/// }
/// ```
///
/// ### The typed baseline + activation pattern (ldui-ztgo)
///
/// A dashboard card that compares a current value against a trailing
/// average AND launches into the rows behind the number needs no card
/// markup, no `Card`/`Badge` composition, and no consumer CSS:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{
///     KpiAction, KpiBaseline, KpiCard, KpiItem, KpiStatus,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let open_detail = Callback::new(|id: String| {
///         // The framework emits the stable `KpiItem::id` and nothing else.
///         // Mapping it to a route, a request, or a selected scope is the
///         // caller's job -- this pattern never navigates or fetches.
///         let _ = id;
///     });
///
///     let item = KpiItem::new("intakes", "Intakes", "280")
///         .status(KpiStatus::Success)
///         .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"))
///         .action(KpiAction::new("View details"));
///
///     view! { <KpiCard item=item on_activate=open_detail /> }
/// }
/// ```
///
/// That renders `280`, a bounded bar whose baseline tick sits at a fixed
/// 80% of its track, the truthful readout `112%`, the localized sentence
/// `12% above baseline`, and one `View details` button. Swap
/// [`KpiBaseline::against`] for [`KpiBaseline::absent`] or
/// [`KpiBaseline::settling`] and the bar disappears while the card keeps
/// its own localized sentence -- see [`KpiBaseline::resolve`].
///
/// ### Add to `input.css`
/// ```css
/// @source inline("rounded-box border border-base-300 bg-base-100 h-full min-w-0 overflow-hidden");
/// @source inline("forced-colors:border-[CanvasText]");
/// @source inline("w-(--border-width-accent) shrink-0 self-stretch forced-colors:bg-[CanvasText]");
/// @source inline("bg-status-blue bg-info bg-success bg-warning bg-error");
/// @source inline("flex flex-col items-center gap-1 gap-2 p-3 p-4 min-w-0 shrink-0");
/// @source inline("line-clamp-2 min-h-8");
/// @source inline("font-semibold uppercase tracking-wide tabular-nums break-words italic");
/// @source inline("text-base-content text-base-content/75 text-base-content/40 text-base-content/60 text-info text-success text-warning text-error");
/// @source inline("tooltip tooltip-top inline-flex h-4 w-4 items-center justify-center rounded-full border sr-only");
/// @source inline("self-start text-left underline underline-offset-2 rounded-field");
/// @source inline("relative h-3 w-full overflow-hidden rounded-full bg-base-200");
/// @source inline("absolute inset-y-0 left-0 top-0 h-full rounded-full w-0.5");
/// @source inline("bg-base-content/80 bg-neutral bg-primary");
/// ```
///
/// The last three lines are `CapacityBar`'s own classes: a card carrying a
/// [`KpiBaseline`] renders one, so a consumer that safelists only the card's
/// classes gets an invisible comparison bar rather than an error.
///
/// The `ld-text-*` steps and `ld-card-depth` are NOT listed above on
/// purpose: they are not Tailwind utilities, so `@source inline(...)`
/// cannot generate them. They are authored rules emitted into
/// `styles/tokens.css` by `cargo xtask gen-tokens`, so a consumer gets them
/// by IMPORTING that stylesheet (see the crate docs). Listing them here
/// would do nothing while implying they were handled (ldui-h7tw,
/// ldui-fg2h, ldui-k4fn).
///
/// ### Card elevation
///
/// The card's resting depth is `ld-card-depth`, which paints
/// `var(--ld-card-shadow, var(--ld-elevation-4))` -- `ui_tokens::elevation`'s
/// declared card resting level, not a stock Tailwind `shadow-*` utility. A
/// product theme that must supply its own approved card shadow sets the one
/// custom property and every card follows:
///
/// ```css
/// :root { --ld-card-shadow: 0 1px 4px rgba(0, 0, 0, 0.16); }
/// ```
///
/// The framework never declares `--ld-card-shadow`, so that override needs
/// no `!important`, no descendant selector into KpiCard's markup, and no
/// page-local fork of the class.
///
/// ## Node References
/// - `node_ref` - References the outer `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn KpiCard(
    /// The KPI to render.
    item: KpiItem,

    /// Tighter padding/gap and a smaller value type step, for dense
    /// contexts (a sidebar summary, an embedded card).
    #[prop(optional, into)]
    compact: Signal<bool>,

    /// Reactive framework-owned copy. See [`KpiStripTexts`].
    #[prop(optional, into, default = Signal::stored(KpiStripTexts::default()))]
    texts: Signal<KpiStripTexts>,

    /// Additional CSS classes for the card's outer wrapper.
    #[prop(optional, into)]
    class: &'static str,

    /// Activation callback, receiving the stable [`KpiItem::id`].
    ///
    /// One callback, one control: supplying this AND an item [`KpiAction`]
    /// renders a single framework-owned `Pressable` inside the card. Omit
    /// either and the card renders exactly as it did before ldui-ztgo.
    #[prop(optional)]
    on_activate: Option<Callback<String>>,

    /// Node reference for the outer `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let KpiItem {
        id,
        label,
        value,
        description,
        status,
        trend,
        help,
        baseline,
        action,
    } = item;

    let available = value.is_some();
    let activatable = kpi_card_is_activatable(action.as_ref(), on_activate.is_some());
    let help_id = has_text(&help).then(|| format!("kpi-card-help-{id}"));

    let name_label = label.clone();
    let name_value = value.clone();
    let name_trend = trend.clone();
    let accessible_name = move || {
        texts.with(|texts| {
            kpi_card_accessible_name(
                &name_label,
                name_value.as_deref(),
                name_trend.as_ref(),
                texts,
            )
        })
    };

    let display_value = value.clone();
    let value_node = move || {
        texts.with(|texts| {
            display_value
                .clone()
                .unwrap_or_else(|| texts.unavailable.clone())
        })
    };

    let value_status_class = status.value_text_class();
    let value_class = move || {
        let size = kpi_card_value_size_class(compact.get());
        if available {
            format!("{size} font-semibold tabular-nums break-words {value_status_class}")
        } else {
            format!("{size} font-semibold tabular-nums break-words italic text-base-content/60")
        }
    };

    // ldui-beqs: the supporting description and trend line sit one rung
    // below the card's label -- `ld-text-small`, the ramp's smallest step
    // (11px, below `ld-text-caption`'s 12px) -- so both read as
    // subordinate to the label (also `ld-text-small`) and value, never
    // competing with them. Label and value sizing are unchanged.
    let has_description = has_text(&description);
    let description_node = has_description.then(|| {
        view! {
            <p class="ld-text-small text-base-content/75 break-words">{description}</p>
        }
    });

    let trend_node = trend.map(|trend| {
        let color_class = trend.direction.as_str();
        let arrow = trend.direction.arrow();
        let magnitude = format!("{:.1}%", trend.value.abs());
        let text = if trend.label.is_empty() {
            format!("{arrow} {magnitude}")
        } else {
            format!("{arrow} {magnitude} {}", trend.label)
        };
        view! {
            <p class=format!("ld-text-small {color_class}")>{text}</p>
        }
    });

    // ldui-ztgo. Two numbers are produced here and they answer different
    // questions on purpose:
    //
    //   * the BAR is bounded -- `fill_percent` is clamped to the track, and
    //     the baseline marker is pinned at 80% of it on every card, so the
    //     twelve cards of a dashboard all compare against the same tick;
    //   * the READOUT is not -- `ratio_percent` says 312% when the value is
    //     312% of baseline, while the bar sits full and
    //     `data-kpi-baseline-saturated` records that the geometry ran out
    //     before the number did.
    //
    // A bar that ends at the track's edge therefore never means "at the
    // cap": the cap is the tick at 80%, which is still visible behind the
    // fill, and the true figure is printed immediately beneath it.
    let comparison_node = baseline.map(|baseline| {
        let resolved = baseline.resolve();
        let baseline_label = baseline.label.clone();
        let has_baseline_label = has_text(&baseline_label);
        let bar_color = status.comparison_bar_color();
        let current = baseline.current;

        let bar = resolved
            .baseline_value
            .zip(resolved.track_max)
            .map(|(cap, max)| {
                // Explicit `max`, never `CapacityBar`'s own default: the default
                // grows the track to fit an over-baseline value, which would slide
                // the marker left on exactly the cards where its position matters
                // most. See `KPI_BASELINE_TRACK_HEADROOM`.
                //
                // `aria-hidden` because the bar is redundant reinforcement of text
                // that already states the ratio, the direction and the baseline's
                // name -- and because a saturated bar's honest `aria-valuenow`
                // would exceed its own `aria-valuemax`. Twelve unnamed progressbars
                // would be noise; twelve out-of-range ones would be wrong.
                view! {
                    <CapacityBar
                        value=current
                        cap=cap
                        max=Some(max)
                        color=bar_color
                        over_color=bar_color
                        attr:aria-hidden="true"
                        attr:data-kpi-baseline-bar="true"
                    />
                }
            });

        let readout_label = baseline_label.clone();
        let readout =
            move || texts.with(|texts| texts.baseline_ratio_readout(&resolved, &readout_label));
        let sentence_label = baseline_label.clone();
        let sentence =
            move || texts.with(|texts| texts.baseline_sentence(&resolved, &sentence_label));

        view! {
            <div
                class="flex flex-col gap-1 min-w-0"
                data-kpi-card-comparison="true"
                data-kpi-baseline-state=resolved.state.as_str()
                data-kpi-baseline-percent=resolved
                    .ratio_percent
                    .map(|percent| percent.to_string())
                data-kpi-baseline-saturated=resolved.saturated.then_some("true")
                data-kpi-baseline-degraded=resolved.degraded.then_some("true")
            >
                {bar}
                // Conditional, not merely empty: with no ratio to print and
                // no baseline name to print, an unconditional `<p>` would
                // still consume the column's `gap-1` and push the sentence
                // 4px down on exactly the no-baseline cards, which is the
                // kind of one-card-only offset `ldui-tbaw` exists to stop.
                {(resolved.state.is_comparable() || has_baseline_label)
                    .then(|| {
                        view! {
                            <p class="ld-text-small text-base-content/75 break-words">
                                <span
                                    class="font-semibold tabular-nums text-base-content"
                                    data-kpi-baseline-readout="true"
                                >
                                    {readout}
                                </span>
                                {has_baseline_label
                                    .then(|| {
                                        view! { <span>{format!(" {baseline_label}")}</span> }
                                    })}
                            </p>
                        }
                    })}
                <p
                    class="ld-text-small text-base-content/75 break-words"
                    data-kpi-baseline-sentence="true"
                >
                    {sentence}
                </p>
            </div>
        }
    });

    let help_button = help_id.clone().map(|_| {
        view! {
            <Tooltip tip=help.clone() class="shrink-0">
                <span
                    class="inline-flex h-4 w-4 items-center justify-center rounded-full border border-base-content/40 text-base-content/75 ld-text-small"
                    aria-hidden="true"
                >
                    "?"
                </span>
            </Tooltip>
        }
    });
    let help_description = help_id.clone().map(|id| {
        view! { <span id=id class="sr-only">{help}</span> }
    });

    // A LEFT edge, not a top stripe: left-edge accents are the prevailing
    // convention for stat cards, and the geometry is better behaved, because
    // a vertical edge cannot shift the body's vertical rhythm at all.
    //
    // Always laid out, and merely uncoloured when the card has no status
    // (`accent_bg_class` is "" for `Neutral`). The reason is the same as it
    // was for the top stripe, just on the other axis: rendering it
    // conditionally would inset the body on status cards and not on neutral
    // ones, so a KpiStrip mixing the two would have two different text
    // alignments. As a top stripe this cost 3px of VERTICAL offset and put
    // values in one row at 567 and 570 (ldui-tbaw); as a left edge the same
    // mistake would cost 3px of horizontal inset. Equal geometry has to be
    // structural, not a side effect of which cards carry a status.
    let accent = view! {
        <div
            class=format!(
                // `forced-colors:bg-[CanvasText]` keeps a STRUCTURAL edge in
                // forced-colors mode. Forced colors overrides author
                // background-color with system colors, so a bar whose only
                // presence is a `bg-*` would vanish entirely and the card
                // would lose the edge that carries its status. Painting
                // CanvasText restores a visible edge; the colour distinction
                // between statuses is intentionally NOT preserved, because
                // forced-colors exists precisely to replace author colour
                // with the user's own palette.
                "w-(--border-width-accent) shrink-0 self-stretch forced-colors:bg-[CanvasText] {}",
                status.accent_bg_class(),
            )
            aria-hidden="true"
        ></div>
    };

    // ldui-ztgo. ONE interactive descendant, or none at all.
    //
    // Deliberately NOT whole-card activation. Three reasons, in order of
    // how expensive each would be to discover later:
    //
    // 1. `<button>` takes phrasing content, and the card body is built from
    //    `<p>` elements plus an `sr-only` help `<span id>`. Wrapping the body
    //    in a button is invalid HTML, and browsers reparent their way out of
    //    it in ways that break the layout.
    // 2. The card's accessible-name grammar is `role="group"` +
    //    `aria-label`, and every existing caller depends on it. A card that
    //    became a button would announce as a control even where nothing has
    //    changed but the framework version.
    // 3. A whole-card click handler that is not a real control needs a
    //    duplicated keyboard path and a synthetic tab stop, and then the
    //    help affordance sits INSIDE a control -- the nested-interactive
    //    defect this bead explicitly prohibits.
    //
    // The help control is already non-interactive (an `aria-hidden` span in
    // a CSS-hover `Tooltip`, with the real text exposed through
    // `aria-describedby`), so an activatable card has exactly one focusable
    // element and one tab stop: this `Pressable`.
    let action_node = activatable
        .then(|| action.zip(on_activate))
        .flatten()
        .map(|(action, on_activate)| {
            let activation_id = id.clone();
            let action_label = action.label.clone();
            let accessible_action_name = {
                let name = accessible_name();
                kpi_action_accessible_name(&action, &name)
            };
            view! {
                <Pressable
                    disabled=action.disabled
                    // `text-info` + a permanent underline: the affordance is
                    // carried by the underline and the label, with colour
                    // third, so it survives greyscale and forced-colors.
                    class="ld-text-small self-start text-left font-semibold text-info underline underline-offset-2 rounded-field"
                    on_click=Callback::new(move |_| on_activate.run(activation_id.clone()))
                    attr:aria-label=accessible_action_name
                    attr:data-kpi-card-action="true"
                >
                    {action_label}
                </Pressable>
            }
        });

    view! {
        <div
            node_ref=node_ref
            role="group"
            aria-label=accessible_name
            aria-describedby=help_id
            data-kpi-card=id
            data-kpi-card-unavailable=(!available).then_some("true")
            // Status without colour: a machine-readable marker so a test or
            // a consumer can read a card's semantic emphasis without
            // sampling a pixel (RecordHeader's `data-record-status-tone`
            // precedent).
            data-kpi-card-status=status.as_str()
            data-kpi-card-activatable=activatable.then_some("true")
            class=merge_classes!(kpi_card_shell_class(), class)
        >
            {accent}
            <div class=move || kpi_card_body_class(compact.get())>
                <div class="flex items-center gap-1 min-w-0">
                    <span class=kpi_card_label_class()>{label}</span>
                    {help_button}
                </div>
                <p class=value_class data-kpi-card-value="true">{value_node}</p>
                {comparison_node}
                {description_node}
                {trend_node}
                {action_node}
                {help_description}
            </div>
        </div>
    }
}

/// Responsive row of independent [`KpiCard`]s -- the pattern this module
/// exists for.
///
/// Owns the grid, its typed [`KpiStripLayout`] ladder, equal card
/// geometry, spacing, and `compact` behavior;
/// each card owns its own label, value, description, and optional
/// status/trend/help. Section headings and period selection (a date-range
/// picker, a "This week" toggle) stay caller-owned through ordinary
/// composition above the strip -- `KpiStrip` renders only the cards.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::StatDeltaTrend;
/// use leptos_daisyui_rs::patterns::{KpiItem, KpiStatus, KpiStrip, KpiTrend};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let items = Signal::derive(|| {
///         vec![
///             KpiItem::new("open", "Open matters", "128")
///                 .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
///             KpiItem::new("overdue", "Overdue tasks", "6")
///                 .status(KpiStatus::Warning)
///                 .help("Tasks past their due date, across every assignee."),
///             KpiItem::new("revenue", "Revenue booked", "$18,400")
///                 .status(KpiStatus::Success),
///             KpiItem::new("sync", "Last sync", "").unavailable(),
///         ]
///     });
///
///     view! { <KpiStrip items=items /> }
/// }
/// ```
///
/// ### Mixing baseline and activatable cards (ldui-ztgo)
///
/// One `on_activate` callback serves the whole strip and receives the
/// activated [`KpiItem::id`]; only the items that carry their own
/// [`KpiAction`] copy become activatable, so there is no parallel array
/// whose positions have to line up with `items`:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{
///     KpiAction, KpiBaseline, KpiItem, KpiStatus, KpiStrip,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let open_detail = Callback::new(|id: String| {
///         let _ = id;
///     });
///
///     let items = Signal::derive(|| {
///         vec![
///             // Above baseline, activatable.
///             KpiItem::new("intakes", "Intakes", "280")
///                 .status(KpiStatus::Success)
///                 .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"))
///                 .action(KpiAction::new("View details")),
///             // Below baseline, activatable.
///             KpiItem::new("closes", "Closes", "5,739")
///                 .status(KpiStatus::Warning)
///                 .baseline(KpiBaseline::against(5739.0, 6705.0).label("12-week avg / 6,705"))
///                 .action(KpiAction::new("View details")),
///             // A brand-new metric: no baseline exists to compare against.
///             KpiItem::new("referrals", "Referrals", "12")
///                 .baseline(KpiBaseline::absent(12.0)),
///             // A window that exists but is not yet full.
///             KpiItem::new("retention", "Retention", "88%")
///                 .baseline(KpiBaseline::settling(88.0)),
///             // No baseline row and no action: unchanged from before.
///             KpiItem::new("last-sync", "Last sync", "").unavailable(),
///         ]
///     });
///
///     view! { <KpiStrip items=items on_activate=open_detail /> }
/// }
/// ```
///
/// ### A balanced twelve-card scorecard (ldui-k3ip)
///
/// A fixed dashboard set of peer KPIs is not an operational strip: twelve
/// cards through the default ladder become eight and then a ragged four,
/// which reads as a primary row and a secondary one even though all twelve
/// are peers. [`KpiStripLayout::BalancedSix`] is the typed way to say so --
/// no CSS classes, no consumer breakpoints:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{KpiItem, KpiStrip, KpiStripLayout};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let items = Signal::derive(|| {
///         (0..12)
///             .map(|n| KpiItem::new(format!("kpi-{n}"), format!("Measure {n}"), "0"))
///             .collect::<Vec<_>>()
///     });
///
///     view! { <KpiStrip items=items layout=KpiStripLayout::BalancedSix /> }
/// }
/// ```
///
/// Twelve cards render as two rows of six once the strip is 896px wide or
/// more; six cards render as one row of six; and the ladder steps down
/// through four, three and two below that, so a strip beside a 360px
/// assistant rail still fits. `compact` remains a separate padding/gap
/// choice and never changes the column count.
///
/// ### Three peer summaries filling a desktop row (ldui-orom)
///
/// Exactly three peer summaries -- the office Pending reconciliation's own
/// shape -- read wrong through either existing profile: `AutoEight` opens
/// eight tracks and leaves the row mostly empty; `BalancedSix` opens six
/// and still leaves half of it empty. [`KpiStripLayout::PeerThree`] caps at
/// three tracks, so three peers always fill the row they are given:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{KpiItem, KpiStrip, KpiStripLayout};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let items = Signal::derive(|| {
///         vec![
///             KpiItem::new("open", "Open items", "42"),
///             KpiItem::new("closed", "Closed items", "310"),
///             KpiItem::new("aging", "Aging over 30 days", "5"),
///         ]
///     });
///
///     view! { <KpiStrip items=items layout=KpiStripLayout::PeerThree /> }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("@container grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 gap-3 gap-4 w-full");
/// @source inline("@5xl:grid-cols-8 @4xl:grid-cols-6 @lg:grid-cols-3");
/// ```
/// The second line carries all three profiles' widest rungs. A consumer
/// that safelists only its own profile's rung gets a strip stuck at four
/// columns the day it switches -- silently, since a missing utility is not
/// an error.
///
/// See [`KpiCard`] for the per-card classes.
///
/// ## Node References
/// - `node_ref` - References the outer grid `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn KpiStrip(
    /// The KPIs to render, in order. Rebuild this list to update values,
    /// switch locale, or add/remove cards -- the strip re-renders from it.
    #[prop(into)]
    items: Signal<Vec<KpiItem>>,

    /// Tighter card padding/gap/type step for dense contexts. Forwarded to
    /// every card.
    ///
    /// Independent of `layout`: it changes padding, gap and the value's
    /// type step, and never the column count.
    #[prop(optional, into)]
    compact: Signal<bool>,

    /// Which typed column ladder the strip follows (`ldui-k3ip`).
    ///
    /// Defaults to [`KpiStripLayout::AutoEight`], which is exactly the
    /// `2 / 3 / 4 / 8` grid every caller had before this prop existed.
    /// [`KpiStripLayout::BalancedSix`] is the balanced fixed-scorecard
    /// ladder. [`KpiStripLayout::PeerThree`] is the fixed three-peer-row
    /// ladder (`ldui-orom`).
    #[prop(optional, into)]
    layout: Signal<KpiStripLayout>,

    /// Reactive framework-owned copy, forwarded to every card. See
    /// [`KpiStripTexts`].
    #[prop(optional, into, default = Signal::stored(KpiStripTexts::default()))]
    texts: Signal<KpiStripTexts>,

    /// Activation callback, receiving the activated [`KpiItem::id`].
    ///
    /// Forwarded to every card, but only the items carrying their own
    /// [`KpiAction`] copy become activatable -- so one strip can mix
    /// activatable and read-only cards without a parallel array whose
    /// positions must accidentally line up with `items`.
    #[prop(optional)]
    on_activate: Option<Callback<String>>,

    /// Additional CSS classes for the grid wrapper.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer grid `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        // Structural container only. An element cannot answer its OWN
        // container query, so the `@sm`/`@lg`/`@4xl`/`@5xl` steps on the grid
        // below need a container ancestor to measure (ldui-tnyq). It carries
        // no spacing of its own, so it cannot affect the strip's geometry.
        //
        // ⚠️ GIVE THE STRIP'S PARENT A WIDTH. `@container` sets
        // `container-type: inline-size`, which makes this element's inline
        // size independent of its own contents. Put the strip in a parent
        // that sizes to its content -- a bare `<div>` inside a `flex` row,
        // say -- and that parent measures the strip as contributing nothing,
        // collapses to zero, and `w-full` here then resolves to zero too. The
        // cards render about 2px wide and no container step ever fires. It
        // fails silently: nothing errors, the markup is correct, and the
        // classes are all present. Six demo fixtures hit exactly this
        // (ldui-k3ip); the fix is `w-full` or an explicit width on the
        // PARENT, which this component cannot supply for you.
        <div class="@container w-full" data-kpi-strip-container="true">
        <div
            node_ref=node_ref
            class=move || merge_classes!(kpi_strip_grid_class(layout.get(), compact.get()), class)
            data-kpi-strip="true"
            // The active profile, readable without parsing utility classes
            // -- so a test asserts the LADDER it asked for and then measures
            // the geometry that ladder produced, rather than asserting on a
            // class string that may or may not have reached the stylesheet.
            data-kpi-strip-layout=move || layout.get().as_str()
        >
            // Keyed, not `collect_view()` (ldui-ztgo). `collect_view()`
            // rebuilds EVERY card on any change to the list, which destroys
            // the focused activation button whenever a poll refreshes one
            // unrelated KPI. `KpiCard` takes its item by value and is not
            // internally reactive, so the key has to cover the whole item,
            // not just its id: an unchanged card then keeps its DOM (and its
            // focus), while a card whose data moved is rebuilt, which is
            // exactly what a non-reactive child needs.
            <For each=move || items.get() key=kpi_item_fingerprint let:item>
                {
                    // `on_activate` is an `Option`, and an optional component
                    // prop takes the INNER type, so the option is unwrapped
                    // here rather than forwarded. Both arms render the same
                    // element; only the callback differs.
                    match on_activate {
                        Some(on_activate) => {
                            view! {
                                <KpiCard
                                    item=item
                                    compact=compact
                                    texts=texts
                                    on_activate=on_activate
                                />
                            }
                                .into_any()
                        }
                        None => {
                            view! { <KpiCard item=item compact=compact texts=texts /> }.into_any()
                        }
                    }
                }
            </For>
        </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The module body above the test module -- used by the source-level
    /// guards so a test's own explanatory prose is never scanned as if it
    /// were rendered markup.
    fn module_source() -> &'static str {
        let source = include_str!("kpi_strip.rs");
        source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before)
    }

    /// Just `KpiCard`'s body.
    fn kpi_card_source() -> &'static str {
        module_source()
            .split_once("pub fn KpiCard(")
            .expect("KpiCard component source")
            .1
    }

    /// ldui-k4fn: the card's depth is the framework's semantic elevation
    /// class, and no stock Tailwind shadow utility survives anywhere on the
    /// shell.
    ///
    /// A substring check on `shadow-` is deliberately broad: the defect this
    /// pins was `shadow-sm`, but `shadow-md`/`shadow-lg`/`shadow-xl` would be
    /// the same violation, and so would a variant-prefixed one
    /// (`hover:shadow-md`). `ld-card-depth` contains no `shadow-` substring,
    /// so the two assertions do not fight.
    #[test]
    fn kpi_card_shell_uses_the_semantic_elevation_class_not_a_stock_shadow() {
        let shell = kpi_card_shell_class();
        assert!(
            shell.split_whitespace().any(|c| c == "ld-card-depth"),
            "the KPI card must carry the framework's static elevation class: {shell}"
        );
        assert!(
            !shell.contains("shadow-"),
            "the KPI card must not carry a stock Tailwind shadow utility \
             (doc/visual-quality/ad-hoc-shadow.md): {shell}"
        );
        assert!(
            !shell.split_whitespace().any(|c| c == "ld-elevated"),
            "ld-elevated lifts on hover and would make a read-only card look \
             interactive: {shell}"
        );
    }

    /// The documented `@source inline(...)` contract must not tell consumers
    /// to safelist a class the component no longer emits, and must not list
    /// the authored `ld-*` rules `@source inline` cannot generate anyway
    /// (ldui-fg2h).
    #[test]
    fn the_documented_source_inline_contract_matches_what_is_rendered() {
        let doc = include_str!("kpi_strip.rs");
        for line in doc.lines() {
            let t = line.trim_start();
            if !t.starts_with("/// @source inline(") {
                continue;
            }
            assert!(
                !t.contains("shadow-"),
                "the @source inline contract still safelists a stock shadow \
                 utility the card no longer renders: {t}"
            );
            assert!(
                !t.contains("ld-"),
                "authored ld-* rules cannot be generated by @source inline \
                 (ldui-fg2h); they ship in styles/tokens.css: {t}"
            );
        }
    }

    #[test]
    fn kpi_status_defaults_to_neutral() {
        assert_eq!(KpiStatus::default(), KpiStatus::Neutral);
    }

    #[test]
    fn kpi_status_value_text_class_maps_each_variant() {
        assert_eq!(KpiStatus::Neutral.value_text_class(), "");
        assert_eq!(KpiStatus::Info.value_text_class(), "text-info");
        assert_eq!(KpiStatus::Success.value_text_class(), "text-success");
        assert_eq!(KpiStatus::Warning.value_text_class(), "text-warning");
        assert_eq!(KpiStatus::Error.value_text_class(), "text-error");
    }

    #[test]
    fn kpi_status_neutral_paints_the_default_blue_accent() {
        // Neutral is the default accent, not the absence of one: it paints
        // the house dark blue, and a status overrides it.
        assert_eq!(KpiStatus::Neutral.accent_bg_class(), "bg-status-blue");
        assert_eq!(KpiStatus::Success.accent_bg_class(), "bg-success");
    }

    #[test]
    fn kpi_item_new_is_available_with_no_optional_fields() {
        let item = KpiItem::new("open", "Open matters", "128");
        assert_eq!(item.id, "open");
        assert_eq!(item.label, "Open matters");
        assert_eq!(item.value.as_deref(), Some("128"));
        assert_eq!(item.description, "");
        assert_eq!(item.status, KpiStatus::Neutral);
        assert!(item.trend.is_none());
        assert_eq!(item.help, "");
    }

    #[test]
    fn kpi_item_unavailable_clears_the_value() {
        let item = KpiItem::new("sync", "Last sync", "2 min ago").unavailable();
        assert_eq!(item.value, None);
    }

    #[test]
    fn kpi_item_builders_set_each_optional_field() {
        let item = KpiItem::new("overdue", "Overdue tasks", "6")
            .description("Past due date")
            .status(KpiStatus::Warning)
            .trend(KpiTrend::new(-2.0, StatDeltaTrend::Negative).label("vs yesterday"))
            .help("Tasks past their due date.");
        assert_eq!(item.description, "Past due date");
        assert_eq!(item.status, KpiStatus::Warning);
        let trend = item.trend.expect("trend set");
        assert_eq!(trend.value, -2.0);
        assert_eq!(trend.direction, StatDeltaTrend::Negative);
        assert_eq!(trend.label, "vs yesterday");
        assert_eq!(item.help, "Tasks past their due date.");
    }

    #[test]
    fn kpi_trend_new_has_no_label() {
        let trend = KpiTrend::new(1.5, StatDeltaTrend::Positive);
        assert_eq!(trend.label, "");
    }

    #[test]
    fn has_text_is_false_for_empty_and_true_otherwise() {
        assert!(!has_text(""));
        assert!(has_text(" "));
        assert!(has_text("Overdue tasks"));
    }

    #[test]
    fn kpi_strip_grid_class_wraps_from_two_to_eight_columns() {
        let normal = kpi_strip_grid_class(KpiStripLayout::AutoEight, false);
        assert!(normal.contains("grid-cols-2"));
        // Container steps, not viewport ones: the column count must follow
        // the strip's own width (ldui-tnyq). A plain `sm:`/`md:`/`xl:` here
        // would mean the strip asks how wide the WINDOW is, which is how an
        // eight-card strip came to render 67px cards in a 648px column.
        assert!(normal.contains("@sm:grid-cols-3"));
        assert!(normal.contains("@lg:grid-cols-4"));
        assert!(normal.contains("@5xl:grid-cols-8"));
        assert!(!normal.contains("xl:grid-cols-8") || normal.contains("@5xl:grid-cols-8"));
        assert!(normal.contains("gap-4"));
    }

    #[test]
    fn kpi_strip_grid_class_compact_uses_a_tighter_gap() {
        let compact = kpi_strip_grid_class(KpiStripLayout::AutoEight, true);
        assert!(compact.contains("gap-3"));
        assert!(!compact.contains("gap-4"));
    }

    #[test]
    fn kpi_card_body_padding_never_exceeds_the_strip_gap() {
        // Internal <= external (this crate's spacing rule): a card's own
        // padding must not exceed the grid gap separating it from its
        // neighbours, or the cards read as one group.
        // Both profiles, since a new ladder must not smuggle in a new gap.
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            assert!(kpi_card_body_class(false).contains("p-4"));
            assert!(kpi_strip_grid_class(layout, false).contains("gap-4"));
            assert!(kpi_card_body_class(true).contains("p-3"));
            assert!(kpi_strip_grid_class(layout, true).contains("gap-3"));
        }
    }

    #[test]
    fn kpi_card_value_size_class_steps_down_in_compact_mode() {
        assert_eq!(kpi_card_value_size_class(false), "ld-text-display");
        assert_eq!(kpi_card_value_size_class(true), "ld-text-title");
    }

    /// ldui-tbaw: the label must wrap up to two lines rather than ellipsize
    /// on a single line -- `line-clamp-2`, never `truncate`.
    #[test]
    fn kpi_card_label_class_clamps_to_two_lines_instead_of_truncating() {
        let label_class = kpi_card_label_class();
        assert!(label_class.contains("line-clamp-2"));
        assert!(
            !label_class.contains("truncate"),
            "single-line truncate must not reappear on the label: {label_class}"
        );
    }

    /// ldui-tbaw: the label box always reserves two full `ld-text-small`
    /// line heights (`min-h-8` = 32px = 2 * 1rem), so a one-line label and a
    /// two-line label leave identically sized boxes -- the mechanism behind
    /// equal card height and aligned values/descriptions/help controls
    /// across a row of cards with differing label lengths.
    #[test]
    fn kpi_card_label_class_reserves_a_fixed_two_line_height() {
        assert!(kpi_card_label_class().contains("min-h-8"));
    }

    #[test]
    fn kpi_card_accessible_name_combines_label_and_value() {
        let texts = KpiStripTexts::default();
        let name = kpi_card_accessible_name("Open matters", Some("128"), None, &texts);
        assert_eq!(name, "Open matters: 128");
    }

    #[test]
    fn kpi_card_accessible_name_falls_back_to_the_unavailable_text() {
        let texts = KpiStripTexts::default();
        let name = kpi_card_accessible_name("Last sync", None, None, &texts);
        assert_eq!(name, "Last sync: Unavailable");
    }

    #[test]
    fn kpi_card_accessible_name_appends_the_trend_word() {
        let texts = KpiStripTexts::default();
        let trend = KpiTrend::new(4.0, StatDeltaTrend::Positive);
        let name = kpi_card_accessible_name("Open matters", Some("128"), Some(&trend), &texts);
        assert_eq!(name, "Open matters: 128, trending up");
    }

    /// ldui-tbaw: a label long enough to clamp visually after two lines
    /// must still reach assistive tech in full -- `kpi_card_accessible_name`
    /// never truncates or clamps, since visual clamping is a CSS-only
    /// (`line-clamp-2`) presentation of the label span's own full text, not
    /// a change to the label string this function receives.
    #[test]
    fn kpi_card_accessible_name_preserves_an_over_long_label_in_full() {
        let texts = KpiStripTexts::default();
        let long_label =
            "Average time to first response across every active support queue this quarter";
        let name = kpi_card_accessible_name(long_label, Some("3h 12m"), None, &texts);
        assert_eq!(name, format!("{long_label}: 3h 12m"));
    }

    #[test]
    fn kpi_strip_texts_trend_word_maps_every_direction() {
        let texts = KpiStripTexts::default();
        assert_eq!(texts.trend_word(StatDeltaTrend::Positive), "trending up");
        assert_eq!(texts.trend_word(StatDeltaTrend::Negative), "trending down");
        assert_eq!(texts.trend_word(StatDeltaTrend::Neutral), "steady");
    }

    /// Guards the "empty optional regions render nothing" contract at the
    /// source level, the same invariant `SectionHeading` pins: the
    /// description block must stay conditional on `has_text`, never
    /// unconditionally rendered.
    #[test]
    fn description_renders_conditionally_on_has_text() {
        let source = include_str!("kpi_strip.rs");
        let component = source
            .split_once("pub fn KpiCard(")
            .expect("KpiCard component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(
            component.contains("has_description.then"),
            "expected the description block to gate on has_text: {component}"
        );
    }

    /// ldui-beqs: description and trend copy must sit on the ramp's
    /// smallest step (`ld-text-small`, below `ld-text-caption`) so they
    /// read as clearly subordinate to the label/value, while the value's
    /// own size class (`kpi_card_value_size_class`, asserted elsewhere)
    /// and the label's `ld-text-small` stay exactly as they were. This is
    /// a source-level guard because the description/trend nodes are built
    /// as static class strings, not through a helper function like
    /// `kpi_card_value_size_class`.
    #[test]
    fn description_and_trend_use_the_smaller_supporting_copy_ramp_step() {
        let source = include_str!("kpi_strip.rs");
        let component = source
            .split_once("pub fn KpiCard(")
            .expect("KpiCard component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(
            component.contains(
                r#"class="ld-text-small text-base-content/75 break-words">{description}"#
            ),
            "expected the description <p> to use ld-text-small: {component}"
        );
        assert!(
            component.contains(r#"format!("ld-text-small {color_class}")"#),
            "expected the trend <p> to use ld-text-small: {component}"
        );
        assert!(
            !component.contains(r#"class="ld-text-caption"#)
                && !component.contains(r#"format!("ld-text-caption"#),
            "ld-text-caption must not reappear as a class on KpiCard's description/trend copy: {component}"
        );
    }

    /// `KpiStrip` must never expose daisyUI's joined `stats`/`stat`
    /// classes -- that is exactly the appearance this pattern exists to
    /// replace.
    #[test]
    fn kpi_strip_never_emits_the_joined_stats_classes() {
        let source = include_str!("kpi_strip.rs");
        let component = source
            .split_once("pub fn KpiStrip(")
            .expect("KpiStrip component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(!component.contains("\"stats\""));
        assert!(!component.contains("\"stat\""));
    }

    // ------------------------------------------------------------------
    // ldui-ztgo: typed baseline comparison.
    // ------------------------------------------------------------------

    /// The bead's own worked example: 280 against a trailing 12-week
    /// average of 250 is 112% of baseline and 12% above it. The two numbers
    /// are derived from one rounding, so they can never disagree.
    #[test]
    fn baseline_reports_the_worked_office_example() {
        let resolved = KpiBaseline::against(280.0, 250.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::Above);
        assert_eq!(resolved.ratio_percent, Some(112));
        assert_eq!(resolved.deviation_percent, Some(12));
        assert!(!resolved.saturated);
        assert!(!resolved.degraded);
    }

    /// THE truthfulness property. The bar is bounded and the number is not:
    /// a value at 312% of baseline pins the fill to the end of the track
    /// while the readout still says 312, and `saturated` records that the
    /// geometry -- not the value -- ran out.
    #[test]
    fn over_baseline_saturates_the_bar_while_the_percentage_stays_truthful() {
        let resolved = KpiBaseline::against(780.0, 250.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::Above);
        assert_eq!(
            resolved.ratio_percent,
            Some(312),
            "the readout must report the real ratio, not the clamped bar"
        );
        assert_eq!(resolved.deviation_percent, Some(212));
        assert_eq!(
            resolved.fill_percent, 100.0,
            "the bar is bounded by its track"
        );
        assert!(
            resolved.saturated,
            "a bar that ran out of track must SAY so, so 'full' is never \
             mistaken for 'at the baseline'"
        );
    }

    /// The marker is the reason a saturated bar is still readable. It sits
    /// at a fixed 80% of the track (1 / 1.25) on EVERY comparable card --
    /// below, level, at 112%, and at 312% -- so a full bar is visibly far
    /// past the tick rather than sitting on it.
    #[test]
    fn the_baseline_marker_never_moves_across_cards() {
        let cards = [
            KpiBaseline::against(0.0, 250.0),
            KpiBaseline::against(125.0, 250.0),
            KpiBaseline::against(250.0, 250.0),
            KpiBaseline::against(280.0, 250.0),
            KpiBaseline::against(780.0, 250.0),
            // A completely different scale: the marker position is a ratio,
            // so it is identical here too.
            KpiBaseline::against(9.0, 4.0),
        ];
        for card in cards {
            let resolved = card.resolve();
            assert!(
                (resolved.marker_percent - 80.0).abs() < 1e-9,
                "marker moved to {} for {card:?}",
                resolved.marker_percent
            );
        }
    }

    /// At exactly the baseline the fill reaches the marker and stops: no
    /// overflow, not saturated, and the sentence is the `Level` one rather
    /// than a nonsensical "0% above".
    #[test]
    fn a_value_exactly_at_the_baseline_is_level_and_stops_at_the_marker() {
        let resolved = KpiBaseline::against(250.0, 250.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::Level);
        assert_eq!(resolved.ratio_percent, Some(100));
        assert_eq!(resolved.deviation_percent, Some(0));
        assert!((resolved.fill_percent - 80.0).abs() < 1e-9);
        assert!(!resolved.saturated);
    }

    /// A value a hair over the baseline rounds to 100% and must therefore
    /// speak the `Level` sentence -- "in line with", never "0% above",
    /// which is the sentence a raw greater-than comparison would produce.
    #[test]
    fn a_hair_over_the_baseline_rounds_to_level_not_zero_percent_above() {
        let resolved = KpiBaseline::against(250.4, 250.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::Level);
        assert_eq!(resolved.deviation_percent, Some(0));
    }

    #[test]
    fn below_baseline_reports_the_deviation_unsigned_via_the_template() {
        let resolved = KpiBaseline::against(5739.0, 6705.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::Below);
        assert_eq!(resolved.ratio_percent, Some(86));
        assert_eq!(resolved.deviation_percent, Some(-14));
        let texts = KpiStripTexts::default();
        assert_eq!(
            texts.baseline_sentence(&resolved, ""),
            "14% below baseline",
            "the sentence carries direction; the number is never signed twice"
        );
    }

    /// The bar's own geometry never leaves the track, in either direction.
    #[test]
    fn the_fill_is_clamped_to_the_track_in_both_directions() {
        for (current, baseline) in [
            (-500.0_f64, 250.0_f64),
            (0.0, 250.0),
            (250.0, 250.0),
            (312.5, 250.0),
            (1_000_000.0, 250.0),
        ] {
            let resolved = KpiBaseline::against(current, baseline).resolve();
            assert!(
                (0.0..=100.0).contains(&resolved.fill_percent),
                "fill escaped the track: {current} vs {baseline} -> {}",
                resolved.fill_percent
            );
        }
    }

    /// A zero baseline is a declared `Available(0.0)`, which the arithmetic
    /// cannot use. It must NOT divide, must NOT fabricate a percentage, and
    /// must not degrade silently -- `degraded` is what makes it findable.
    #[test]
    fn a_zero_baseline_never_divides_and_never_fabricates_a_percentage() {
        let resolved = KpiBaseline::against(42.0, 0.0).resolve();
        assert_eq!(resolved.state, KpiBaselineState::NoBaseline);
        assert_eq!(resolved.ratio_percent, None);
        assert_eq!(resolved.deviation_percent, None);
        assert_eq!(resolved.fill_percent, 0.0);
        assert!(!resolved.saturated);
        assert!(
            resolved.degraded,
            "an unusable baseline must be flagged, not quietly swallowed"
        );
    }

    /// Negative and non-finite baselines, and a non-finite current, take the
    /// same guarded path. None of them may produce NaN or an infinity.
    #[test]
    fn negative_and_non_finite_inputs_are_all_guarded() {
        for (current, baseline) in [
            (42.0_f64, -40.0_f64),
            (42.0, f64::NAN),
            (42.0, f64::INFINITY),
            (f64::NAN, 250.0),
            (f64::INFINITY, 250.0),
        ] {
            let resolved = KpiBaseline::against(current, baseline).resolve();
            assert_eq!(
                resolved.state,
                KpiBaselineState::NoBaseline,
                "{current} vs {baseline}"
            );
            assert_eq!(resolved.ratio_percent, None, "{current} vs {baseline}");
            assert!(resolved.degraded, "{current} vs {baseline}");
            assert!(
                resolved.fill_percent.is_finite() && resolved.track_max.is_none(),
                "{current} vs {baseline} produced non-finite geometry"
            );
        }
    }

    /// The three unavailable-comparison states are DISTINCT, each with its
    /// own sentence. "There is no baseline", "the window is still filling",
    /// and "the caller handed over an unusable number" are different facts,
    /// and only the last is a defect.
    #[test]
    fn absent_settling_and_degraded_are_three_distinguishable_states() {
        let absent = KpiBaseline::absent(12.0).resolve();
        let settling = KpiBaseline::settling(88.0).resolve();
        let degraded = KpiBaseline::against(42.0, 0.0).resolve();

        assert_eq!(absent.state, KpiBaselineState::NoBaseline);
        assert!(!absent.degraded);
        assert_eq!(settling.state, KpiBaselineState::Settling);
        assert!(!settling.degraded);
        assert_eq!(degraded.state, KpiBaselineState::NoBaseline);
        assert!(degraded.degraded);

        // Absent and settling never collapse into one another's copy.
        let texts = KpiStripTexts::default();
        assert_eq!(texts.baseline_sentence(&absent, ""), "No baseline yet");
        assert_eq!(
            texts.baseline_sentence(&settling, ""),
            "Baseline still settling"
        );
        assert_ne!(
            texts.baseline_sentence(&absent, ""),
            texts.baseline_sentence(&settling, "")
        );
    }

    /// Every one of the five states speaks. None of them renders an empty
    /// comparison row, which is the silent-degradation shape this repo has
    /// repeatedly paid for.
    #[test]
    fn every_baseline_state_has_its_own_non_empty_sentence() {
        let texts = KpiStripTexts::default();
        let sentences: Vec<String> = [
            KpiBaseline::against(280.0, 250.0),
            KpiBaseline::against(250.0, 250.0),
            KpiBaseline::against(200.0, 250.0),
            KpiBaseline::absent(1.0),
            KpiBaseline::settling(1.0),
        ]
        .iter()
        .map(|baseline| texts.baseline_sentence(&baseline.resolve(), "12-week avg"))
        .collect();
        for sentence in &sentences {
            assert!(!sentence.is_empty(), "empty comparison sentence");
            assert!(
                !sentence.contains('{'),
                "an unsubstituted placeholder leaked into the copy: {sentence}"
            );
        }
        let unique: std::collections::BTreeSet<&String> = sentences.iter().collect();
        assert_eq!(
            unique.len(),
            5,
            "five states, five sentences: {sentences:?}"
        );
    }

    /// `{ratio}`/`{delta}` in a no-baseline template substitute to the
    /// localized `unavailable` word, never to a fabricated `0`.
    #[test]
    fn absent_templates_substitute_unavailable_rather_than_zero() {
        let texts = KpiStripTexts {
            baseline_absent: "{ratio} / {delta} for {baseline}".to_owned(),
            ..KpiStripTexts::default()
        };
        let resolved = KpiBaseline::absent(5.0).resolve();
        assert_eq!(
            texts.baseline_sentence(&resolved, "12-week avg"),
            "Unavailable / Unavailable for 12-week avg"
        );
    }

    /// The ratio readout exists only where there is a ratio to report.
    #[test]
    fn the_ratio_readout_is_absent_when_there_is_nothing_to_be_truthful_about() {
        let texts = KpiStripTexts::default();
        let comparable = KpiBaseline::against(780.0, 250.0).resolve();
        assert_eq!(
            texts.baseline_ratio_readout(&comparable, ""),
            Some("312%".to_owned())
        );
        for unavailable in [
            KpiBaseline::absent(1.0).resolve(),
            KpiBaseline::settling(1.0).resolve(),
            KpiBaseline::against(1.0, 0.0).resolve(),
        ] {
            assert_eq!(texts.baseline_ratio_readout(&unavailable, ""), None);
        }
    }

    /// All three placeholders are substituted in every template, so a
    /// locale is free to order them however its grammar needs.
    #[test]
    fn every_placeholder_is_substituted_in_a_localized_template() {
        let texts = KpiStripTexts {
            baseline_above: "{baseline}: {ratio}% ({delta} pts arriba)".to_owned(),
            ..KpiStripTexts::default()
        };
        let resolved = KpiBaseline::against(280.0, 250.0).resolve();
        assert_eq!(
            texts.baseline_sentence(&resolved, "promedio de 12 semanas"),
            "promedio de 12 semanas: 112% (12 pts arriba)"
        );
    }

    /// The bar's colour is the card's TYPED status, never a function of the
    /// numbers. `CapacityBarColor::for_direction` exists and would paint
    /// above-baseline green, which asserts higher-is-better -- false for
    /// "days to close" or "cost per matter".
    #[test]
    fn the_comparison_bar_takes_its_colour_from_status_not_from_the_numbers() {
        assert_eq!(
            KpiStatus::Warning.comparison_bar_color(),
            CapacityBarColor::Warning
        );
        assert_eq!(
            KpiStatus::Success.comparison_bar_color(),
            CapacityBarColor::Success
        );
        assert_eq!(
            KpiStatus::Neutral.comparison_bar_color(),
            CapacityBarColor::Primary
        );

        let module = module_source();
        assert!(
            !module.contains("for_direction("),
            "the KPI comparison must not infer good/bad from the values: \
             CapacityBarColor's direction helper encodes higher-is-better"
        );
        assert!(
            module.contains("over_color=bar_color"),
            "the overflow band must share the fill's colour, so exceeding the \
             baseline carries no inferred valence -- 'over' is signalled by \
             the fill crossing the marker and by the sentence, not by an \
             alarm colour the framework has no basis to choose"
        );
    }

    /// The comparison row sits BELOW the value, so a card with a baseline
    /// and one without keep identical label and value offsets (the
    /// `ldui-tbaw` alignment contract).
    #[test]
    fn the_comparison_row_never_renders_above_the_value() {
        let component = kpi_card_source();
        let value = component
            .find("data-kpi-card-value")
            .expect("the value paragraph");
        let comparison = component
            .find("{comparison_node}")
            .expect("the comparison node in the card body");
        assert!(
            value < comparison,
            "the comparison row must follow the value; anything inserted \
             ABOVE it would shift every baseline-bearing card's value out of \
             line with its neighbours (ldui-tbaw)"
        );
        // And the two mechanisms that keep mixed cards aligned are untouched.
        assert!(kpi_card_label_class().contains("min-h-8"));
        assert!(kpi_card_shell_class().contains("h-full"));
    }

    // ------------------------------------------------------------------
    // ldui-ztgo: activation.
    // ------------------------------------------------------------------

    /// Activation is OPT-IN and needs BOTH halves. Every card written
    /// before this bead has neither, so none of them becomes focusable or
    /// announces as a control.
    #[test]
    fn activation_requires_both_the_action_copy_and_a_callback() {
        let action = KpiAction::new("View details");
        assert!(!kpi_card_is_activatable(None, false));
        assert!(
            !kpi_card_is_activatable(None, true),
            "a callback alone must not mint an unlocalized English button"
        );
        assert!(
            !kpi_card_is_activatable(Some(&action), false),
            "action copy alone has nothing to run"
        );
        assert!(kpi_card_is_activatable(Some(&action), true));
    }

    /// An item built the old way carries neither half, so the gate is
    /// false for it by construction -- the proof that existing callers are
    /// untouched rather than an assertion that they are.
    #[test]
    fn items_built_without_the_new_builders_are_never_activatable() {
        let legacy = KpiItem::new("open", "Open matters", "128")
            .description("Across every queue")
            .status(KpiStatus::Warning)
            .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive))
            .help("Everything still on the desk.");
        assert!(legacy.action.is_none());
        assert!(legacy.baseline.is_none());
        assert!(!kpi_card_is_activatable(legacy.action.as_ref(), true));
        assert!(!kpi_card_is_activatable(legacy.action.as_ref(), false));
    }

    /// A non-activatable card renders no button, no `tabindex`, and keeps
    /// `role="group"`. Pinned at the source level because the markers that
    /// would make a card focusable are all conditional on the same gate.
    #[test]
    fn a_non_activatable_card_stays_a_non_focusable_group() {
        let component = kpi_card_source();
        assert!(
            component.contains(r#"role="group""#),
            "the card's accessible-name grammar must not change"
        );
        assert!(
            !component.contains("tabindex"),
            "the card must never mint a synthetic tab stop: {component}"
        );
        assert!(
            component.contains("let action_node = activatable"),
            "the action control must be gated on `activatable`"
        );
        assert!(
            component.contains(r#"data-kpi-card-activatable=activatable.then_some("true")"#),
            "the activatable marker must be conditional, so a read-only card \
             does not advertise a control it does not have"
        );
    }

    /// Exactly one interactive descendant, and only when activatable. The
    /// help affordance stays a non-interactive `aria-hidden` span whose real
    /// text reaches assistive tech through `aria-describedby`, so there is
    /// never a control inside a control.
    #[test]
    fn an_activatable_card_has_exactly_one_interactive_descendant() {
        let component = kpi_card_source();
        assert_eq!(
            component.matches("<Pressable").count(),
            1,
            "one activation control per card, not one per affordance"
        );
        assert_eq!(
            component.matches("<Button").count(),
            0,
            "a second control would nest interactive elements inside the card"
        );
        assert!(
            component.contains(r#"aria-hidden="true""#),
            "the help glyph must stay non-interactive and hidden"
        );
        assert!(
            component.contains("aria-describedby=help_id"),
            "help text reaches assistive tech without a second tab stop"
        );
    }

    /// The activation control's accessible name always begins with its
    /// visible label (WCAG 2.5.3 Label in Name) and still distinguishes one
    /// of twelve identically-labelled buttons.
    #[test]
    fn the_action_accessible_name_extends_the_visible_label() {
        let action = KpiAction::new("View details");
        let name = kpi_action_accessible_name(&action, "Intakes: 280");
        assert!(
            name.starts_with("View details"),
            "the visible label must be a prefix of the accessible name: {name}"
        );
        assert_eq!(name, "View details, Intakes: 280");
    }

    #[test]
    fn an_explicit_accessible_label_wins() {
        let action = KpiAction::new("View details").accessible_label("Open the intake detail view");
        assert_eq!(
            kpi_action_accessible_name(&action, "Intakes: 280"),
            "Open the intake detail view"
        );
    }

    #[test]
    fn kpi_action_defaults_to_enabled_with_a_generated_name() {
        let action = KpiAction::new("View details");
        assert!(!action.disabled);
        assert_eq!(action.accessible_label, "");
        assert!(KpiAction::new("x").disabled(true).disabled);
    }

    /// ldui-k4fn, revisited for ldui-ztgo. An ACTIVATABLE card keeps the
    /// STATIC card elevation; it does not adopt `ld-elevated`'s hover lift.
    ///
    /// The card is not the control -- one `Pressable` inside it is. Lifting
    /// the whole card on hover would promise that pressing anywhere on it
    /// does something, which is a bigger lie than the read-only tile
    /// `ld-card-depth` was chosen to avoid. The interactive affordance lives
    /// exactly where the interaction does: `Pressable` already carries
    /// `ld-pressable` (press scale) and `ld-focus-ring` (focus-visible
    /// ring), eased by `ld-eased`.
    #[test]
    fn an_activatable_card_keeps_the_static_elevation_not_the_interactive_lift() {
        let shell = kpi_card_shell_class();
        assert!(shell.split_whitespace().any(|c| c == "ld-card-depth"));
        assert!(!shell.split_whitespace().any(|c| c == "ld-elevated"));
        let component = kpi_card_source();
        assert!(
            !component.contains("ld-elevated"),
            "no branch of the card may swap in the interactive elevation: {component}"
        );
        assert!(
            component.contains("<Pressable"),
            "the focus/press affordance must live on the control"
        );
    }

    /// Status is readable without colour: a stable `data-` marker, the
    /// `RecordHeader` precedent.
    #[test]
    fn status_is_exposed_as_a_colour_independent_marker() {
        assert_eq!(KpiStatus::Neutral.as_str(), "neutral");
        assert_eq!(KpiStatus::Error.as_str(), "error");
        assert!(kpi_card_source().contains("data-kpi-card-status=status.as_str()"));
    }

    #[test]
    fn baseline_state_markers_are_stable() {
        assert_eq!(KpiBaselineState::Above.as_str(), "above");
        assert_eq!(KpiBaselineState::Level.as_str(), "level");
        assert_eq!(KpiBaselineState::Below.as_str(), "below");
        assert_eq!(KpiBaselineState::NoBaseline.as_str(), "no-baseline");
        assert_eq!(KpiBaselineState::Settling.as_str(), "settling");
        assert!(KpiBaselineState::Above.is_comparable());
        assert!(!KpiBaselineState::NoBaseline.is_comparable());
        assert!(!KpiBaselineState::Settling.is_comparable());
    }

    #[test]
    fn kpi_item_builders_set_the_new_typed_capabilities() {
        let item = KpiItem::new("intakes", "Intakes", "280")
            .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"))
            .action(KpiAction::new("View details"));
        let baseline = item.baseline.expect("baseline set");
        assert_eq!(baseline.current, 280.0);
        assert_eq!(
            baseline.availability,
            KpiBaselineAvailability::Available(250.0)
        );
        assert_eq!(baseline.label, "12-week avg / 250");
        assert_eq!(item.action.expect("action set").label, "View details");
    }

    /// The strip reconciles on the WHOLE item, not on the id. An id-only key
    /// would leave a card showing a stale number after a refresh, and would
    /// never re-render a locale change at all (same ids, new labels).
    #[test]
    fn the_reconciliation_key_covers_every_field_not_just_the_id() {
        let base = KpiItem::new("intakes", "Intakes", "280");
        assert_eq!(kpi_item_fingerprint(&base), kpi_item_fingerprint(&base));

        let same_id_new_value = KpiItem::new("intakes", "Intakes", "281");
        let same_id_new_locale = KpiItem::new("intakes", "Admisiones", "280");
        let same_id_new_status = KpiItem::new("intakes", "Intakes", "280").status(KpiStatus::Error);
        let same_id_new_baseline =
            KpiItem::new("intakes", "Intakes", "280").baseline(KpiBaseline::against(280.0, 250.0));
        for changed in [
            same_id_new_value,
            same_id_new_locale,
            same_id_new_status,
            same_id_new_baseline,
        ] {
            assert_ne!(
                kpi_item_fingerprint(&base),
                kpi_item_fingerprint(&changed),
                "a changed card must get a new key or it will never re-render"
            );
        }
    }

    /// The track's right edge is a fixed multiple of the baseline, and the
    /// card passes it explicitly. `CapacityBar`'s own default max is
    /// `cap * 1.25` CLAMPED UP TO `value`, which would rescale the track --
    /// and slide the marker -- on exactly the over-baseline cards where the
    /// marker's position matters most.
    #[test]
    fn the_comparison_track_max_is_pinned_and_passed_explicitly() {
        assert_eq!(KPI_BASELINE_TRACK_HEADROOM, 1.25);
        let resolved = KpiBaseline::against(780.0, 250.0).resolve();
        assert_eq!(resolved.track_max, Some(312.5));
        assert_eq!(resolved.baseline_value, Some(250.0));
        assert!(
            kpi_card_source().contains("max=Some(max)"),
            "the card must override CapacityBar's value-dependent default max"
        );
    }

    /// Guards against `opacity-*` utilities creeping back in for muted
    /// text -- this crate's `test-style` axe gate fails `opacity-60`/
    /// `opacity-50` text for insufficient contrast; the approved idiom is
    /// a `text-base-content/NN` alpha-mixed color.
    #[test]
    fn muted_text_never_uses_the_opacity_utility() {
        // Scan only the module body above the tests, not this test's own
        // doc comment (which names the forbidden classes to explain why).
        let source = include_str!("kpi_strip.rs");
        let module = source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(!module.contains("opacity-"));
    }

    // ------------------------------------------------------------------
    // ldui-k3ip: the typed layout profile.
    //
    // These assert GEOMETRY -- column counts and computed card widths --
    // rather than class names, because `/components/kpi_strip` is not in
    // the layout/style audit page sets (ldui-ddhr), so nothing else in this
    // repo measures this pattern. A test that only compared class strings
    // would pass while the strip rendered 40px cards.
    // ------------------------------------------------------------------

    /// The card width at which a comparison bar is drawn: the card, less
    /// the always-laid-out 3px accent edge (`--border-width-accent`) and
    /// the body's own padding on both sides (`p-4` = 16px, `p-3` = 12px).
    fn comparison_bar_width_px(card_px: f64, compact: bool) -> f64 {
        let padding = if compact { 12.0 } else { 16.0 };
        card_px - 3.0 - padding * 2.0
    }

    /// SOURCE COMPATIBILITY, proved rather than asserted: the default
    /// profile's grid classes are the pre-`ldui-k3ip` literals, character
    /// for character. Every existing caller passes no `layout` at all and
    /// therefore takes `Signal::default()`, which is
    /// [`KpiStripLayout::AutoEight`].
    ///
    /// The other half of the proof is that this module's own doctests and
    /// the showcase compile untouched -- neither passes `layout`.
    #[test]
    fn the_default_profile_is_the_untouched_pre_bead_grid() {
        assert_eq!(KpiStripLayout::default(), KpiStripLayout::AutoEight);
        assert_eq!(
            kpi_strip_grid_class(KpiStripLayout::default(), false),
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @5xl:grid-cols-8 gap-4"
        );
        assert_eq!(
            kpi_strip_grid_class(KpiStripLayout::default(), true),
            "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @5xl:grid-cols-8 gap-3"
        );
        // And the prop stays optional, so no call site is forced to name it.
        assert!(
            module_source().contains("layout: Signal<KpiStripLayout>"),
            "the layout prop must stay an optional signal, defaulting to AutoEight"
        );
    }

    /// The emitted utilities are composed from the declared ladder, so a
    /// rung cannot be changed in the table and left stale in the class
    /// string (or the reverse).
    #[test]
    fn kpi_strip_grid_class_is_composed_from_the_declared_ladder() {
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            for (compact, gap) in [(false, "gap-4"), (true, "gap-3")] {
                let rungs: Vec<String> = layout
                    .ladder()
                    .iter()
                    .map(|rung| format!("{}grid-cols-{}", rung.prefix, rung.columns))
                    .collect();
                let expected = format!("grid {} {gap}", rungs.join(" "));
                assert_eq!(
                    kpi_strip_grid_class(layout, compact),
                    expected,
                    "{layout:?} compact={compact}"
                );
            }
        }
    }

    /// Container queries, never viewport breakpoints (`ldui-tnyq`), on
    /// BOTH profiles -- a new ladder is exactly where a bare `xl:` would
    /// slip back in.
    #[test]
    fn every_profile_uses_container_queries_not_viewport_breakpoints() {
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            for rung in layout.ladder() {
                assert!(
                    rung.prefix.is_empty() || rung.prefix.starts_with('@'),
                    "{layout:?} rung {rung:?} is a viewport breakpoint; the strip \
                     must ask how wide IT is, not how wide the window is"
                );
            }
            for compact in [false, true] {
                for token in kpi_strip_grid_class(layout, compact).split_whitespace() {
                    assert!(
                        !token.contains(':') || token.starts_with('@'),
                        "{layout:?}: {token} is a viewport variant"
                    );
                }
            }
        }
    }

    /// Every rung of every profile clears the measured two-line label floor
    /// at its own threshold width, computed rather than eyeballed.
    ///
    /// The base rung is checked at 320px, the narrowest supported viewport;
    /// a container starved below that is `ldui-kwup`, not a ladder defect.
    #[test]
    fn every_rung_clears_the_measured_two_line_card_floor() {
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            for rung in layout.ladder() {
                let container = f64::from(rung.min_container_px).max(320.0);
                for compact in [false, true] {
                    let card =
                        kpi_strip_card_width_px(container, rung.columns, kpi_strip_gap_px(compact));
                    assert!(
                        card >= KPI_CARD_TWO_LINE_FLOOR_PX,
                        "{layout:?} {}{} columns at {container}px gives {card}px cards, \
                         below the measured {KPI_CARD_TWO_LINE_FLOOR_PX}px floor",
                        rung.prefix,
                        rung.columns
                    );
                }
            }
        }
    }

    /// THE RUNG CHOICE, with its arithmetic. Six columns start at `@4xl`
    /// (896px) because that is where a help-bearing card still holds two
    /// label lines; `@3xl` (768px) would clear the bare floor and fail the
    /// help floor, which is the regime a twelve-card scorecard lives in.
    #[test]
    fn the_six_column_rung_sits_where_a_help_bearing_card_still_fits() {
        let gap = kpi_strip_gap_px(false);
        let shipped = kpi_strip_card_width_px(896.0, 6, gap);
        assert!((shipped - 136.0).abs() < 1e-9, "{shipped}");
        assert!(
            shipped >= KPI_CARD_HELP_FLOOR_PX,
            "the shipped six-column rung must hold a help-bearing label"
        );

        // The rung that was considered and rejected.
        let rejected = kpi_strip_card_width_px(768.0, 6, gap);
        assert!((rejected - (768.0 - 80.0) / 6.0).abs() < 1e-9, "{rejected}");
        assert!(
            rejected >= KPI_CARD_TWO_LINE_FLOOR_PX,
            "@3xl clears the bare two-line floor ..."
        );
        assert!(
            rejected < KPI_CARD_HELP_FLOOR_PX,
            "... but not the help-bearing one, which is why the rung is @4xl"
        );

        assert_eq!(
            KpiStripLayout::BalancedSix
                .ladder()
                .last()
                .expect("a widest rung")
                .min_container_px,
            896
        );
    }

    /// The consumer's reproduction, in numbers. At the 1046px container
    /// `ldui-tnyq` measured on a 1680px window, `BalancedSix` lays twelve
    /// peer cards out as two full rows of six.
    ///
    /// The second half is the NEGATIVE CONTROL the bead asks for: the same
    /// assertion run against the old hard-coded eight-column ladder fails,
    /// and fails in exactly the reported way -- eight then a ragged four.
    #[test]
    fn twelve_cards_are_two_rows_of_six_and_the_eight_column_ladder_cannot_be() {
        const CONSUMER_CONTAINER_PX: f64 = 1046.0;

        let balanced = KpiStripLayout::BalancedSix.columns_at(CONSUMER_CONTAINER_PX);
        assert_eq!(balanced, 6);
        let fit = kpi_strip_row_fit(12, balanced);
        assert_eq!(fit.full_rows, 2);
        assert_eq!(fit.last_row, 0);
        assert_eq!(fit.rows(), 2);
        assert!(!fit.is_ragged(), "twelve peers must not end ragged");
        let card =
            kpi_strip_card_width_px(CONSUMER_CONTAINER_PX, balanced, kpi_strip_gap_px(false));
        assert!((card - 161.0).abs() < 1e-9, "{card}");

        // The old contract, at the identical width.
        let auto = KpiStripLayout::AutoEight.columns_at(CONSUMER_CONTAINER_PX);
        assert_eq!(auto, 8);
        let auto_fit = kpi_strip_row_fit(12, auto);
        // Two rows either way -- which is exactly why "two rows" alone is not
        // the assertion. The property is TWO ROWS OF SIX: two FULL rows of
        // equal peers. The eight-column ladder cannot express it, and the
        // shape it produces instead is the reported defect.
        assert_eq!(auto_fit.rows(), 2);
        assert_ne!(
            auto_fit.full_rows, 2,
            "the eight-column ladder cannot produce two full rows; if this ever \
             passes, the balanced-six assertion above is measuring nothing"
        );
        assert_eq!(auto_fit.full_rows, 1);
        assert_eq!(auto_fit.last_row, 4);
        assert!(
            auto_fit.is_ragged(),
            "eight then four is the reported defect: a ragged second row that \
             reads as a secondary group"
        );
        let auto_card =
            kpi_strip_card_width_px(CONSUMER_CONTAINER_PX, auto, kpi_strip_gap_px(false));
        assert!(
            card > auto_card,
            "the balanced profile must not make cards narrower: {card} vs {auto_card}"
        );
    }

    /// THE BEAD'S OWN REPRODUCTION, reproduced exactly: at the 1617.6px
    /// container measured on a 1696px viewport (ldui-orom), `AutoEight`
    /// renders eight tracks of 188.2px cards and the three summary cards
    /// occupy only the first 596.6px; `BalancedSix` renders six tracks and
    /// still leaves roughly half the row empty. `PeerThree` renders exactly
    /// three tracks that span the whole 1617.6px row.
    #[test]
    fn three_peer_summaries_fill_the_desktop_row() {
        const CONSUMER_CONTAINER_PX: f64 = 1617.6;
        let gap = kpi_strip_gap_px(false);

        // THE FIX: three peers through the new profile fill one full row.
        let peer = KpiStripLayout::PeerThree.columns_at(CONSUMER_CONTAINER_PX);
        assert_eq!(peer, 3);
        let fit = kpi_strip_row_fit(3, peer);
        assert_eq!(fit.full_rows, 1);
        assert_eq!(fit.last_row, 0);
        assert_eq!(fit.rows(), 1);
        assert!(!fit.is_ragged(), "three peers must fill one full row");
        let card = kpi_strip_card_width_px(CONSUMER_CONTAINER_PX, peer, gap);
        let spanned = card * 3.0 + gap * 2.0;
        assert!(
            (spanned - CONSUMER_CONTAINER_PX).abs() < 1e-9,
            "three tracks must span the full strip: {spanned} vs {CONSUMER_CONTAINER_PX}"
        );

        // THE NEGATIVE CONTROL: the bead's own reported numbers for
        // AutoEight, reproduced exactly -- 188.2px cards occupying only the
        // first 596.6px of the row.
        let auto = KpiStripLayout::AutoEight.columns_at(CONSUMER_CONTAINER_PX);
        assert_eq!(auto, 8);
        let auto_fit = kpi_strip_row_fit(3, auto);
        assert!(
            auto_fit.is_ragged(),
            "three cards in eight tracks read as an incomplete row"
        );
        let auto_card = kpi_strip_card_width_px(CONSUMER_CONTAINER_PX, auto, gap);
        assert!((auto_card - 188.2).abs() < 0.05, "{auto_card}");
        let auto_spanned = auto_card * 3.0 + gap * 2.0;
        assert!((auto_spanned - 596.6).abs() < 0.05, "{auto_spanned}");

        // BalancedSix still leaves roughly half the row empty, per the
        // bead's description.
        let balanced = KpiStripLayout::BalancedSix.columns_at(CONSUMER_CONTAINER_PX);
        assert_eq!(balanced, 6);
        let balanced_fit = kpi_strip_row_fit(3, balanced);
        assert!(
            balanced_fit.is_ragged(),
            "three cards in six tracks still leave half the row empty"
        );
        let balanced_card = kpi_strip_card_width_px(CONSUMER_CONTAINER_PX, balanced, gap);
        let balanced_spanned = balanced_card * 3.0 + gap * 2.0;
        assert!(
            balanced_spanned < CONSUMER_CONTAINER_PX * 0.55,
            "BalancedSix must still leave roughly half the row empty: {balanced_spanned}"
        );

        // AutoEight and BalancedSix are unchanged by this bead: same class
        // strings, same ladders, as pinned by every other geometry test in
        // this module. This test only adds the third profile.
    }

    /// Six items are one full row of six; the AC's other counts land
    /// deliberately.
    #[test]
    fn six_five_and_empty_item_sets_land_deliberately() {
        let columns = KpiStripLayout::BalancedSix.columns_at(1046.0);

        let six = kpi_strip_row_fit(6, columns);
        assert_eq!(six.rows(), 1);
        assert_eq!(six.full_rows, 1);
        assert_eq!(six.last_row, 0);
        assert!(!six.is_ragged());

        // A count that does not divide is a RAGGED LAST ROW, on purpose:
        // the tracks are explicit, so the five cards keep their sixth-width
        // tracks and the sixth track is simply empty. Stretching them would
        // make a five-card strip's cards a different size from a six-card
        // strip's, which is the equal-geometry property this pattern owns.
        let five = kpi_strip_row_fit(5, columns);
        assert_eq!(five.rows(), 1);
        assert_eq!(five.full_rows, 0);
        assert_eq!(five.last_row, 5);
        assert!(five.is_ragged());

        // Seven: the bead's own divisibility question.
        let seven = kpi_strip_row_fit(7, columns);
        assert_eq!(seven.rows(), 2);
        assert_eq!(seven.last_row, 1);
        assert!(seven.is_ragged());

        let empty = kpi_strip_row_fit(0, columns);
        assert_eq!(empty.rows(), 0);
        assert_eq!(empty.last_row, 0);
        assert!(
            !empty.is_ragged(),
            "an empty strip is not a ragged one; it has no rows at all"
        );
    }

    /// The ladder steps down without ever overflowing its container, and
    /// never exceeds the profile's declared widest rung.
    #[test]
    fn narrow_containers_step_down_without_horizontal_overflow() {
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            for container in [
                320.0_f64, 383.0, 384.0, 511.0, 512.0, 767.0, 895.0, 896.0, 1023.0, 1024.0, 1046.0,
                1680.0, 2560.0,
            ] {
                let columns = layout.columns_at(container);
                assert!(
                    columns >= 2,
                    "{layout:?} at {container}px fell below two columns"
                );
                assert!(
                    columns <= layout.max_columns(),
                    "{layout:?} at {container}px exceeded its declared widest rung"
                );
                for compact in [false, true] {
                    let card =
                        kpi_strip_card_width_px(container, columns, kpi_strip_gap_px(compact));
                    assert!(
                        card >= KPI_CARD_TWO_LINE_FLOOR_PX,
                        "{layout:?} at {container}px gives {card}px cards, below the \
                         measured floor -- the step-down happened too late"
                    );
                }
            }
        }
    }

    /// A rung applies AT its threshold, not one pixel past it -- the `>=`
    /// semantics of a `@container (width >= 56rem)` rule.
    #[test]
    fn a_rung_applies_at_its_own_threshold_width() {
        assert_eq!(KpiStripLayout::BalancedSix.columns_at(895.0), 4);
        assert_eq!(KpiStripLayout::BalancedSix.columns_at(896.0), 6);
        assert_eq!(KpiStripLayout::AutoEight.columns_at(1023.0), 4);
        assert_eq!(KpiStripLayout::AutoEight.columns_at(1024.0), 8);
        // Between 896 and 1024 the two profiles genuinely disagree, which
        // is the point of having two.
        assert_eq!(KpiStripLayout::BalancedSix.columns_at(960.0), 6);
        assert_eq!(KpiStripLayout::AutoEight.columns_at(960.0), 4);
    }

    /// `compact` is orthogonal: it moves padding and gap, never the column
    /// count.
    #[test]
    fn compact_changes_spacing_and_never_the_column_count() {
        for layout in [
            KpiStripLayout::AutoEight,
            KpiStripLayout::BalancedSix,
            KpiStripLayout::PeerThree,
        ] {
            let normal: Vec<&str> = kpi_strip_grid_class(layout, false)
                .split_whitespace()
                .filter(|token| token.contains("grid-cols"))
                .collect();
            let compact: Vec<&str> = kpi_strip_grid_class(layout, true)
                .split_whitespace()
                .filter(|token| token.contains("grid-cols"))
                .collect();
            assert_eq!(normal, compact, "{layout:?}: compact moved a column rung");
            assert_ne!(
                kpi_strip_grid_class(layout, false),
                kpi_strip_grid_class(layout, true),
                "{layout:?}: compact must still change the gap"
            );
        }
    }

    /// ldui-ztgo's baseline comparison row still reads at six-column width.
    ///
    /// Six columns are WIDER than the eight the default already ships at
    /// the consumer's container, so the bar and its fixed 80% marker gain
    /// room rather than losing it. The narrowest a balanced-six bar ever
    /// gets is at the six-column rung itself.
    #[test]
    fn the_baseline_bar_reads_at_six_column_width() {
        // At the six-column rung: 136px card -> 101px of bar.
        let narrowest = comparison_bar_width_px(
            kpi_strip_card_width_px(896.0, 6, kpi_strip_gap_px(false)),
            false,
        );
        assert!((narrowest - 101.0).abs() < 1e-9, "{narrowest}");

        // What the default already ships at the consumer's own 1046px
        // container: 116.75px card -> 81.75px of bar. The balanced profile
        // is strictly more room than the shipped baseline bar has today.
        let shipped_today = comparison_bar_width_px(
            kpi_strip_card_width_px(1046.0, 8, kpi_strip_gap_px(false)),
            false,
        );
        assert!(
            narrowest > shipped_today,
            "a six-column baseline bar must not be tighter than the eight-column \
             one already in production: {narrowest} vs {shipped_today}"
        );

        // The marker sits at a fixed 80% of the track on every card
        // (KPI_BASELINE_TRACK_HEADROOM), so at the narrowest balanced-six
        // width it is still ~81px from the bar's left edge and clear of the
        // 2px marker's own width.
        let marker_offset = narrowest * 0.8;
        assert!(marker_offset > 80.0, "{marker_offset}");
        assert!(
            narrowest - marker_offset >= 2.0,
            "the fixed marker must not be flush against the track's right edge"
        );

        // And in compact mode, where padding drops with the gap.
        let compact = comparison_bar_width_px(
            kpi_strip_card_width_px(896.0, 6, kpi_strip_gap_px(true)),
            true,
        );
        assert!(compact > narrowest, "{compact} vs {narrowest}");
    }

    /// Layout markers are stable and distinct, so a browser test can read
    /// the active profile without parsing utility classes.
    #[test]
    fn layout_markers_are_stable_and_distinct() {
        assert_eq!(KpiStripLayout::AutoEight.as_str(), "auto-eight");
        assert_eq!(KpiStripLayout::BalancedSix.as_str(), "balanced-six");
        assert_eq!(KpiStripLayout::PeerThree.as_str(), "peer-three");
        assert_ne!(
            KpiStripLayout::AutoEight.as_str(),
            KpiStripLayout::BalancedSix.as_str()
        );
        assert_ne!(
            KpiStripLayout::BalancedSix.as_str(),
            KpiStripLayout::PeerThree.as_str()
        );
        assert!(
            module_source().contains("data-kpi-strip-layout=move || layout.get().as_str()"),
            "the strip must emit its active profile as a data marker"
        );
    }

    /// The profile is a NAMED INTENT, not a column integer. A `columns:
    /// usize` prop would accept twelve columns of 40px; there is no such
    /// door.
    #[test]
    fn the_layout_prop_is_typed_rather_than_a_raw_column_count() {
        let module = module_source();
        assert!(
            !module.contains("columns: usize"),
            "a raw column count would let a caller ask for an unrenderable grid"
        );
        assert!(
            !module.contains("grid_class: &'static str") && !module.contains("grid_class: String"),
            "an arbitrary class fragment is not a composition contract either"
        );
        // Three profiles today; the enum is what made the third additive.
        assert_eq!(KpiStripLayout::AutoEight.max_columns(), 8);
        assert_eq!(KpiStripLayout::BalancedSix.max_columns(), 6);
        assert_eq!(KpiStripLayout::PeerThree.max_columns(), 3);
    }

    /// Degenerate inputs never divide by zero or panic.
    #[test]
    fn the_geometry_helpers_are_total() {
        assert_eq!(kpi_strip_card_width_px(1024.0, 0, 16.0), 0.0);
        let fit = kpi_strip_row_fit(12, 0);
        assert_eq!(fit.rows(), 0);
        assert!(!fit.is_ragged());
        assert_eq!(kpi_strip_row_fit(usize::MAX, 6).columns, 6);
    }
}
