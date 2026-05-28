use super::style::StatDeltaTrend;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Stats Component
///
/// A reactive Leptos wrapper for daisyUI's stats component that provides a container
/// for displaying statistical data in organized, visually appealing card layouts.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("stats stats-horizontal stats-vertical stat stat-title stat-value stat-desc stat-figure stat-actions text-success text-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Stats(
    /// Toggle vertical layout
    #[prop(optional, into)]
    vertical: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Child components (typically Stat components)
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("stats", class)
            class:stats-vertical=vertical
        >
            {children()}
        </div>
    }
}

/// Individual statistic item within a Stats container.
#[component]
pub fn Stat(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Child components (StatTitle, StatValue, StatDesc, etc.)
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat", class)>
            {children()}
        </div>
    }
}

/// Title/label component for a statistic.
#[component]
pub fn StatTitle(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,
    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
    /// Title content
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat-title", class)>
            {children()}
        </div>
    }
}

/// Primary value display component for a statistic.
#[component]
pub fn StatValue(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// The primary value content
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat-value", class)>
            {children()}
        </div>
    }
}

/// Description component providing additional context for a statistic.
#[component]
pub fn StatDesc(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Description content
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat-desc", class)>
            {children()}
        </div>
    }
}

/// Figure component for displaying icons, images, or visual elements in a statistic.
#[component]
pub fn StatFigure(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Visual content (icons, images, etc.)
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat-figure", class)>
            {children()}
        </div>
    }
}

/// # StatDelta Component
///
/// Convenience helper for the up/down-percentage trend indicator that
/// the daisyUI Stats demos hand-type as `"↗︎ 12% increase"`.  Renders
/// as a sibling of `<StatDesc>` inside a `<Stat>` — a `<div class="stat-desc
/// text-success|error">` with arrow + absolute-value-percent + optional
/// suffix label.
///
/// Mirrors d2d-ui's `Stats::with_delta(value, positive)` API so the
/// same backing data can render on both stacks.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("stat-desc text-success text-error");
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// <Stat>
///     <StatTitle>"Revenue"</StatTitle>
///     <StatValue>"$125,430"</StatValue>
///     <StatDesc>"Last 30 days"</StatDesc>
///     <StatDelta value=12.5 trend=StatDeltaTrend::Positive label="increase" />
/// </Stat>
/// ```
///
/// ## Node References
/// - `node_ref` - References the div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn StatDelta(
    /// Percentage to display. Sign is ignored — `abs()` is rendered;
    /// direction is conveyed by `trend` instead, because in real
    /// dashboards "going up" is not always "good" (e.g. churn).
    #[prop(into)]
    value: Signal<f64>,

    /// Semantic trend.  Drives both the colour and the arrow direction;
    /// defaults to `Positive` so the common "good growth" case is
    /// zero-config.
    #[prop(optional, into)]
    trend: Signal<StatDeltaTrend>,

    /// Optional trailing label appended after the percentage, e.g.
    /// `"increase"` for `↗︎ 12.5% increase` or `"(28 today)"`.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Additional CSS classes appended to `stat-desc text-{trend}`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("stat-desc", trend.get().as_str(), class)
        >
            {move || {
                let arrow = trend.get().arrow();
                let pct = format!("{:.1}%", value.get().abs());
                match label.get() {
                    Some(l) if !l.is_empty() => format!("{} {} {}", arrow, pct, l),
                    _ => format!("{} {}", arrow, pct),
                }
            }}
        </div>
    }
}

/// Actions container for interactive elements within a statistic.
#[component]
pub fn StatActions(
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Interactive content (buttons, links, etc.)
    children: Children,
) -> impl IntoView {
    view! {
        <div node_ref=node_ref class=move || merge_classes!("stat-actions", class)>
            {children()}
        </div>
    }
}
