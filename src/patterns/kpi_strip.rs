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

use crate::components::{StatDeltaTrend, Tooltip};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// Reactive framework-owned copy for `KpiStrip`/`KpiCard`'s own generated
/// text -- the unavailable-value fallback and the trend-direction words
/// folded into each card's accessible name. Caller-supplied [`KpiItem`]
/// text (label/value/description/help) is not covered here: localize it by
/// rebuilding the `items` list for the active locale, the same as any
/// other reactive prop in this crate.
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
}

impl Default for KpiStripTexts {
    fn default() -> Self {
        Self {
            unavailable: "Unavailable".to_owned(),
            trend_up: "trending up".to_owned(),
            trend_down: "trending down".to_owned(),
            trend_steady: "steady".to_owned(),
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
}

/// Semantic emphasis for one [`KpiCard`].
///
/// Drives the value text color and a top accent stripe together, so the
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

    /// Top accent stripe background class. Empty for `Neutral`, which
    /// renders no stripe at all.
    fn accent_bg_class(self) -> &'static str {
        match self {
            KpiStatus::Neutral => "",
            KpiStatus::Info => "bg-info",
            KpiStatus::Success => "bg-success",
            KpiStatus::Warning => "bg-warning",
            KpiStatus::Error => "bg-error",
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
}

/// Whether optional copy should render at all -- mirrors
/// [`SectionHeading`](super::SectionHeading)'s `has_text`: an empty string
/// renders nothing, not an empty line.
fn has_text(value: &str) -> bool {
    !value.is_empty()
}

/// Responsive grid classes for the strip.
///
/// Two columns at the narrowest width (never a single full-bleed column,
/// which reads as a list rather than a grid of cards), growing to eight --
/// a full row -- at `xl`. When there are fewer than eight items, CSS Grid
/// leaves the remaining explicit-column tracks empty rather than
/// stretching the existing cards to fill them, so card size stays equal
/// regardless of count.
fn kpi_strip_grid_class(compact: bool) -> &'static str {
    if compact {
        "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-8 gap-3"
    } else {
        "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-8 gap-4"
    }
}

/// Card body padding/gap: internal spacing stays at or below the grid gap
/// (`p-4` <= `gap-4`, `p-3` <= `gap-3`) so cards never read as a single
/// group with their neighbours.
fn kpi_card_body_class(compact: bool) -> &'static str {
    if compact {
        "flex flex-col gap-1 p-3"
    } else {
        "flex flex-col gap-2 p-4"
    }
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
/// ### Add to `input.css`
/// ```css
/// @source inline("rounded-box border border-base-300 bg-base-100 shadow-sm h-full min-w-0 overflow-hidden");
/// @source inline("forced-colors:border-[CanvasText]");
/// @source inline("h-(--border-width-accent) w-full");
/// @source inline("bg-info bg-success bg-warning bg-error");
/// @source inline("flex flex-col items-center gap-1 gap-2 p-3 p-4 min-w-0 shrink-0");
/// @source inline("line-clamp-2 min-h-8");
/// @source inline("font-semibold uppercase tracking-wide tabular-nums break-words italic");
/// @source inline("text-base-content/75 text-base-content/40 text-base-content/60 text-info text-success text-warning text-error");
/// @source inline("tooltip tooltip-top inline-flex h-4 w-4 items-center justify-center rounded-full border sr-only");
/// ```
///
/// The `ld-text-*` steps are NOT listed above on purpose: they are not
/// Tailwind utilities, so `@source inline(...)` cannot generate them.
/// They are authored rules emitted into `styles/tokens.css` by
/// `cargo xtask gen-tokens`, so a consumer gets them by IMPORTING that
/// stylesheet (see the crate docs). Listing them here would do nothing
/// while implying the ramp was handled (ldui-h7tw, ldui-fg2h).
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
    } = item;

    let available = value.is_some();
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

    // Always laid out, and merely uncoloured when the card has no status
    // (`accent_bg_class` is "" for `Neutral`). Rendering it conditionally
    // made the strip 3px tall on some cards and absent on others, so a
    // KpiStrip mixing status and neutral cards started their bodies at
    // different offsets and their values sat 3px apart -- measured at
    // valueTop 567 vs 570 across one row (ldui-tbaw, whose acceptance
    // requires values to retain their alignment). Reserving the space
    // unconditionally is the same reasoning as `min-h-8` on the label:
    // equal geometry has to be structural, not a side effect of content.
    let accent = view! {
        <div
            class=format!("h-(--border-width-accent) w-full {}", status.accent_bg_class())
            aria-hidden="true"
        ></div>
    };

    view! {
        <div
            node_ref=node_ref
            role="group"
            aria-label=accessible_name
            aria-describedby=help_id
            data-kpi-card=id
            data-kpi-card-unavailable=(!available).then_some("true")
            class=merge_classes!(
                "rounded-box border border-base-300 bg-base-100 shadow-sm h-full min-w-0 overflow-hidden forced-colors:border-[CanvasText]",
                class
            )
        >
            {accent}
            <div class=move || kpi_card_body_class(compact.get())>
                <div class="flex items-center gap-1 min-w-0">
                    <span class=kpi_card_label_class()>{label}</span>
                    {help_button}
                </div>
                <p class=value_class data-kpi-card-value="true">{value_node}</p>
                {description_node}
                {trend_node}
                {help_description}
            </div>
        </div>
    }
}

/// Responsive row of independent [`KpiCard`]s -- the pattern this module
/// exists for.
///
/// Owns the grid, equal card geometry, spacing, and `compact` behavior;
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
/// ### Add to `input.css`
/// ```css
/// @source inline("grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-8 gap-3 gap-4");
/// ```
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
    #[prop(optional, into)]
    compact: Signal<bool>,

    /// Reactive framework-owned copy, forwarded to every card. See
    /// [`KpiStripTexts`].
    #[prop(optional, into, default = Signal::stored(KpiStripTexts::default()))]
    texts: Signal<KpiStripTexts>,

    /// Additional CSS classes for the grid wrapper.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer grid `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!(kpi_strip_grid_class(compact.get()), class)
            data-kpi-strip="true"
        >
            {move || {
                items
                    .get()
                    .into_iter()
                    .map(|item| {
                        view! { <KpiCard item=item compact=compact texts=texts /> }
                    })
                    .collect_view()
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn kpi_status_neutral_renders_no_accent_stripe() {
        assert_eq!(KpiStatus::Neutral.accent_bg_class(), "");
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
        let normal = kpi_strip_grid_class(false);
        assert!(normal.contains("grid-cols-2"));
        assert!(normal.contains("sm:grid-cols-3"));
        assert!(normal.contains("md:grid-cols-4"));
        assert!(normal.contains("xl:grid-cols-8"));
        assert!(normal.contains("gap-4"));
    }

    #[test]
    fn kpi_strip_grid_class_compact_uses_a_tighter_gap() {
        let compact = kpi_strip_grid_class(true);
        assert!(compact.contains("gap-3"));
        assert!(!compact.contains("gap-4"));
    }

    #[test]
    fn kpi_card_body_padding_never_exceeds_the_strip_gap() {
        // Internal <= external (this crate's spacing rule): a card's own
        // padding must not exceed the grid gap separating it from its
        // neighbours, or the cards read as one group.
        assert!(kpi_card_body_class(false).contains("p-4"));
        assert!(kpi_strip_grid_class(false).contains("gap-4"));
        assert!(kpi_card_body_class(true).contains("p-3"));
        assert!(kpi_strip_grid_class(true).contains("gap-3"));
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
}
