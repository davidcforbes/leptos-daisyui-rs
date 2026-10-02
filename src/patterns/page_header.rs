//! Consistent hierarchy and slots for list-page headings.

use leptos::prelude::*;

/// Placement policy for [`PageHeader`]'s optional back-navigation slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageHeaderNavigationLayout {
    /// Preserve the historical title-cluster placement: beside the title at
    /// wider widths and stacked by the existing responsive flex rules.
    #[default]
    InlineResponsive,
    /// Render one dedicated navigation landmark above all heading content at
    /// every viewport width.
    DedicatedRow,
    /// Keep the title (left) and actions (right) on the top row, then put
    /// `back` and the `breadcrumbs` trail on the divider row beneath it,
    /// followed by a hairline rule that fills to the right edge. That row
    /// replaces the header's bottom border; with no `back` and no crumbs the
    /// header keeps its plain `divider` (ldui-rhnr, the Office page-layout
    /// rule).
    OnDivider,
}

impl PageHeaderNavigationLayout {
    /// Stable runtime marker emitted on the header root.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InlineResponsive => "inline-responsive",
            Self::DedicatedRow => "dedicated-row",
            Self::OnDivider => "on-divider",
        }
    }
}

/// Whether [`PageHeader`] renders its historical bottom rule.
///
/// Defaults to [`Self::Shown`] so every existing caller (none of which pass
/// this prop) keeps rendering the border it already has -- source-compatible
/// by construction. A base/open composition that supplies its own separator
/// (or none at all) passes `divider=PageHeaderDivider::Hidden` to omit it.
///
/// Under [`PageHeaderNavigationLayout::OnDivider`] the divider row's own
/// rule stands in for this border whenever that row renders, so exactly one
/// rule shows; `Hidden` drops that rule as well.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageHeaderDivider {
    /// Render the `border-b border-base-300` rule and its `pb-4` clearance
    /// (today's behavior, unchanged).
    #[default]
    Shown,
    /// Omit the rule and its bottom padding -- for an intentionally open
    /// base-page composition that supplies its own separation.
    Hidden,
}

impl PageHeaderDivider {
    /// Stable runtime marker emitted on the header root.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shown => "shown",
            Self::Hidden => "hidden",
        }
    }

    /// Classes contributed to the header root.
    const fn class(self) -> &'static str {
        match self {
            Self::Shown => "border-b border-base-300 pb-4",
            Self::Hidden => "",
        }
    }
}

/// Whether [`PageHeader`] renders its subtitle paragraph at all.
///
/// op-bjpst (2026-09-14): the `<p>` used to render unconditionally in both
/// navigation-layout branches, so a caller that passes no `subtitle` (the prop
/// is optional and defaults to an empty string) shipped an empty `text-sm`
/// line box under every title -- all seven account-family pages carried a
/// blank line. Whitespace-only text counts as empty: it paints nothing but
/// still reserves the line.
pub fn page_header_renders_subtitle(subtitle: &str) -> bool {
    !subtitle.trim().is_empty()
}

/// One crumb in [`PageHeader`]'s breadcrumb trail, rendered by
/// [`PageHeaderNavigationLayout::OnDivider`].
///
/// The component, not the caller, decides which crumb is the current page:
/// the LAST crumb always renders as plain text carrying
/// `aria-current="page"`, even when it was built with [`Self::link`]. An
/// earlier crumb renders as a muted link when it has an href and as muted
/// plain text when it does not.
#[derive(Clone, Debug)]
pub struct PageHeaderCrumb {
    label: Signal<String>,
    href: Option<String>,
}

impl PageHeaderCrumb {
    /// A crumb linking to `href`. `label` accepts a `&str`, a `String` or a
    /// signal (a record name that loads later). `href` is rendered verbatim
    /// as the `<a href>`: do not interpolate untrusted input (a
    /// `javascript:` scheme would execute), the same contract as
    /// `LinkButton` and `BreadcrumbItem` hrefs.
    pub fn link(label: impl Into<Signal<String>>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
        }
    }

    /// A crumb without a link -- typically the current page, last in the
    /// trail.
    pub fn text(label: impl Into<Signal<String>>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }
}

/// How [`PageHeader`] renders one breadcrumb; see [`page_header_crumb_kind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageHeaderCrumbKind {
    /// An earlier crumb with an href: a muted link.
    Link,
    /// An earlier crumb without an href: muted plain text.
    Text,
    /// The last crumb: the current page, plain text with `aria-current="page"`.
    Current,
}

impl PageHeaderCrumbKind {
    /// Stable runtime marker emitted as `data-page-header-crumb` on the crumb's `<li>`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Text => "text",
            Self::Current => "current",
        }
    }
}

/// How the crumb at `index` of a `count`-crumb trail renders: the last one is
/// always the current page, whatever href it carries.
pub fn page_header_crumb_kind(index: usize, count: usize, has_href: bool) -> PageHeaderCrumbKind {
    if index + 1 == count {
        PageHeaderCrumbKind::Current
    } else if has_href {
        PageHeaderCrumbKind::Link
    } else {
        PageHeaderCrumbKind::Text
    }
}

/// One part of the [`PageHeaderNavigationLayout::OnDivider`] divider row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageHeaderDividerPart {
    /// The caller's `back` control; it never shrinks.
    Back,
    /// The breadcrumb trail; the part that truncates first at narrow widths.
    Trail,
    /// The `flex-1` hairline rule filling the row to its right edge.
    Rule,
}

/// The parts the [`PageHeaderNavigationLayout::OnDivider`] divider row
/// renders, in render order: back, trail, rule.
///
/// Empty when there is neither a `back` control nor a crumb: no row renders
/// and the header keeps its plain `divider` instead. The rule is present
/// only for [`PageHeaderDivider::Shown`]; `Hidden` keeps the navigation and
/// drops the rule. The render path iterates this list, so the order the unit
/// tests pin is the order in the DOM.
pub fn page_header_divider_parts(
    has_back: bool,
    crumb_count: usize,
    divider: PageHeaderDivider,
) -> Vec<PageHeaderDividerPart> {
    if !has_back && crumb_count == 0 {
        return Vec::new();
    }
    [
        (has_back, PageHeaderDividerPart::Back),
        (crumb_count > 0, PageHeaderDividerPart::Trail),
        (
            divider == PageHeaderDivider::Shown,
            PageHeaderDividerPart::Rule,
        ),
    ]
    .into_iter()
    .filter_map(|(renders, part)| renders.then_some(part))
    .collect()
}

/// Divider classes the [`PageHeaderNavigationLayout::OnDivider`] header root
/// carries: none while the divider row renders (its rule replaces the
/// border, so exactly one rule shows), otherwise the plain `divider` classes
/// exactly as the other layouts emit them.
fn page_header_on_divider_root_class(
    parts: &[PageHeaderDividerPart],
    divider: PageHeaderDivider,
) -> &'static str {
    if parts.is_empty() {
        divider.class()
    } else {
        ""
    }
}

/// The `OnDivider` row: `parts` in order inside one navigation landmark, or
/// nothing when `parts` is empty. The trail is the only part that shrinks
/// (`min-w-0 truncate`); the back control is `shrink-0`, the rule keeps a
/// `min-w-8` floor, and the row never wraps.
fn page_header_divider_row(
    parts: &[PageHeaderDividerPart],
    mut back: Option<AnyView>,
    breadcrumbs: Vec<PageHeaderCrumb>,
    navigation_label: Signal<String>,
) -> Option<impl IntoView> {
    if parts.is_empty() {
        return None;
    }
    let mut breadcrumbs = Some(breadcrumbs);
    let row_parts = parts
        .iter()
        .copied()
        .map(|part| match part {
            PageHeaderDividerPart::Back => view! {
                <div class="flex shrink-0 items-center" data-page-header-back="true">{back.take()}</div>
            }
            .into_any(),
            PageHeaderDividerPart::Trail => {
                page_header_trail(breadcrumbs.take().unwrap_or_default())
            }
            PageHeaderDividerPart::Rule => view! {
                <div aria-hidden="true" class="min-w-8 flex-1 border-t border-base-300" data-page-header-rule="true"></div>
            }
            .into_any(),
        })
        .collect::<Vec<_>>();
    Some(view! {
        <nav
            class="flex min-w-0 flex-nowrap items-center gap-2"
            aria-label=move || navigation_label.get()
            data-page-header-divider-row="true"
        >
            {row_parts}
        </nav>
    })
}

/// The breadcrumb trail: an `<ol>` of inline items so `truncate` ellipsizes
/// it as one line, with muted crumbs and `/` separators at `/75` and the last
/// crumb as the current page.
fn page_header_trail(crumbs: Vec<PageHeaderCrumb>) -> AnyView {
    let count = crumbs.len();
    let items = crumbs
        .into_iter()
        .enumerate()
        .map(|(index, PageHeaderCrumb { label, href })| {
            let kind = page_header_crumb_kind(index, count, href.is_some());
            let separator = (index > 0).then(|| {
                view! {
                    <span aria-hidden="true" class="px-2 text-base-content/75" data-page-header-crumb-separator="true">"/"</span>
                }
            });
            let content = match (kind, href) {
                (PageHeaderCrumbKind::Link, Some(href)) => view! {
                    <a class="link link-hover text-base-content/75" href=href>{move || label.get()}</a>
                }
                .into_any(),
                (PageHeaderCrumbKind::Current, _) => view! {
                    <span class="font-medium text-base-content" aria-current="page">{move || label.get()}</span>
                }
                .into_any(),
                _ => view! { <span class="text-base-content/75">{move || label.get()}</span> }.into_any(),
            };
            view! {
                <li class="inline" data-page-header-crumb=kind.as_str()>
                    {separator}
                    {content}
                </li>
            }
        })
        .collect::<Vec<_>>();
    view! {
        <ol class="min-w-0 truncate p-1 text-sm" data-page-header-trail="true">
            {items}
        </ol>
    }
    .into_any()
}

/// Page heading with explicit navigation, freshness, dataset, and action slots.
///
/// `actions` wraps to a second (or later) row instead of overflowing the
/// viewport once its content no longer fits one line -- see
/// [`PageQuickActions`](super::PageQuickActions) for an opinionated
/// icon-and-label composition to place inside this slot. `divider` controls
/// whether the header renders its bottom rule; see [`PageHeaderDivider`].
///
/// `navigation_layout` places `back`: beside the title (the default), on a
/// dedicated row above it, or, with [`PageHeaderNavigationLayout::OnDivider`]
/// (the Office page-layout rule), on the divider row under the title row
/// together with the `breadcrumbs` trail:
///
/// ```text
/// Account Conversations            [Call] [SMS] [Email] [CRM]
/// < Back  Coordinator / Account Conversations ----------------
/// ```
///
/// That row is the navigation landmark (named by `navigation_label`), the
/// last crumb is the current page (`aria-current="page"`), and its hairline
/// rule replaces the header's bottom border. At narrow widths the trail
/// truncates first; the back control and the rule's floor never shrink and
/// the row never wraps. Stable hooks: `data-page-header-divider-row`,
/// `data-page-header-back`, `data-page-header-trail`,
/// `data-page-header-crumb` (`link`, `text` or `current`) and
/// `data-page-header-rule`.
#[component]
pub fn PageHeader(
    /// Primary page title.
    #[prop(into)]
    title: Signal<String>,
    /// Supporting page description.
    #[prop(optional, into)]
    subtitle: Signal<String>,
    /// Optional back-navigation action: before the title, on a row above it
    /// (`DedicatedRow`) or at the start of the divider row (`OnDivider`).
    #[prop(optional)]
    back: Option<Children>,
    /// Placement for `back`. The historical inline-responsive layout remains
    /// the default; select `DedicatedRow` for a separate row above the title,
    /// or `OnDivider` to put `back` and `breadcrumbs` on the divider row under
    /// the title row.
    #[prop(optional)]
    navigation_layout: PageHeaderNavigationLayout,
    /// Whether the header renders its bottom rule. Defaults to
    /// [`PageHeaderDivider::Shown`] -- today's behavior -- so existing
    /// callers are unaffected; pass [`PageHeaderDivider::Hidden`] for an
    /// intentionally open base-page composition.
    #[prop(optional)]
    divider: PageHeaderDivider,
    /// Localizable accessible name for the navigation landmark: the
    /// `DedicatedRow` row or the `OnDivider` divider row.
    #[prop(into, default = Signal::stored("Page navigation".to_owned()))]
    navigation_label: Signal<String>,
    /// Breadcrumb trail for `OnDivider` (the other layouts ignore it). The
    /// last crumb is always the current page: plain text with
    /// `aria-current="page"`. Earlier crumbs with an href render as links.
    #[prop(optional)]
    breadcrumbs: Vec<PageHeaderCrumb>,
    /// Optional freshness/status content adjacent to the title.
    #[prop(optional)]
    freshness: Option<Children>,
    /// Optional dataset selector capsule, separate from page filters.
    #[prop(optional)]
    dataset: Option<Children>,
    /// Optional page-level actions.
    #[prop(optional)]
    actions: Option<Children>,
    /// Additional header classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    let back = back.map(|slot| slot());
    let freshness = freshness.map(|slot| slot());
    let dataset = dataset.map(|slot| slot());
    let actions = actions.map(|slot| slot());

    match navigation_layout {
        PageHeaderNavigationLayout::InlineResponsive => view! {
            <header
                class=format!(
                    "flex flex-col gap-4 {divider_class} lg:flex-row lg:items-start lg:justify-between {class}",
                    divider_class = divider.class()
                )
                data-page-header="true"
                data-ld-pattern="PageHeader"
                data-page-header-navigation-layout=navigation_layout.as_str()
                data-page-header-divider=divider.as_str()
            >
                <div class="flex min-w-0 flex-col items-start gap-3 sm:flex-row">
                    {back.map(|back| view! { <div class="shrink-0 pt-1">{back}</div> })}
                    <div class="min-w-0 space-y-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                                {move || title.get()}
                            </h1>
                            {freshness}
                        </div>
                        {move || {
                            let subtitle = subtitle.get();
                            page_header_renders_subtitle(&subtitle).then(|| view! {
                                <p class="max-w-3xl text-sm text-base-content/75 sm:text-base" data-page-subtitle="true">
                                    {subtitle}
                                </p>
                            })
                        }}
                    </div>
                </div>
                <div class="flex flex-wrap items-end gap-2 lg:justify-end">
                    {dataset.map(|dataset| view! {
                        <div class="min-w-56" data-page-dataset-slot="true">{dataset}</div>
                    })}
                    {actions.map(|actions| view! {
                        <div class="flex flex-wrap items-center gap-2" data-page-actions-slot="true">
                            {actions}
                        </div>
                    })}
                </div>
            </header>
        }
        .into_any(),
        PageHeaderNavigationLayout::DedicatedRow => view! {
            <header
                class=format!(
                    "flex min-w-0 flex-col gap-3 {divider_class} {class}",
                    divider_class = divider.class()
                )
                data-page-header="true"
                data-ld-pattern="PageHeader"
                data-page-header-navigation-layout=navigation_layout.as_str()
                data-page-header-divider=divider.as_str()
            >
                {back.map(|back| view! {
                    <nav
                        class="flex min-w-0 flex-wrap items-center gap-2"
                        aria-label=move || navigation_label.get()
                        data-page-navigation-row="true"
                    >
                        {back}
                    </nav>
                })}
                <div class="flex min-w-0 flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div class="min-w-0 flex-1 space-y-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                                {move || title.get()}
                            </h1>
                            {freshness}
                        </div>
                        {move || {
                            let subtitle = subtitle.get();
                            page_header_renders_subtitle(&subtitle).then(|| view! {
                                <p class="max-w-3xl text-sm text-base-content/75 sm:text-base" data-page-subtitle="true">
                                    {subtitle}
                                </p>
                            })
                        }}
                    </div>
                    <div class="flex min-w-0 flex-wrap items-end gap-2 lg:justify-end">
                        {dataset.map(|dataset| view! {
                            <div class="min-w-56" data-page-dataset-slot="true">{dataset}</div>
                        })}
                        {actions.map(|actions| view! {
                            <div class="flex flex-wrap items-center gap-2" data-page-actions-slot="true">
                                {actions}
                            </div>
                        })}
                    </div>
                </div>
            </header>
        }
        .into_any(),
        PageHeaderNavigationLayout::OnDivider => {
            let parts = page_header_divider_parts(back.is_some(), breadcrumbs.len(), divider);
            let root_divider_class = page_header_on_divider_root_class(&parts, divider);
            let divider_row = page_header_divider_row(&parts, back, breadcrumbs, navigation_label);
            view! {
                <header
                    class=format!("flex min-w-0 flex-col gap-3 {root_divider_class} {class}")
                    data-page-header="true"
                    data-ld-pattern="PageHeader"
                    data-page-header-navigation-layout=navigation_layout.as_str()
                    data-page-header-divider=divider.as_str()
                >
                    <div class="flex min-w-0 flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                        <div class="min-w-0 flex-1 space-y-1">
                            <div class="flex flex-wrap items-center gap-2">
                                <h1 class="ld-text-display font-semibold tracking-tight text-base-content">
                                    {move || title.get()}
                                </h1>
                                {freshness}
                            </div>
                            {move || {
                                let subtitle = subtitle.get();
                                page_header_renders_subtitle(&subtitle).then(|| view! {
                                    <p class="max-w-3xl text-sm text-base-content/75 sm:text-base" data-page-subtitle="true">
                                        {subtitle}
                                    </p>
                                })
                            }}
                        </div>
                        <div class="flex min-w-0 flex-wrap items-end gap-2 lg:justify-end">
                            {dataset.map(|dataset| view! {
                                <div class="min-w-56" data-page-dataset-slot="true">{dataset}</div>
                            })}
                            {actions.map(|actions| view! {
                                <div class="flex flex-wrap items-center gap-2" data-page-actions-slot="true">
                                    {actions}
                                </div>
                            })}
                        </div>
                    </div>
                    {divider_row}
                </header>
            }
            .into_any()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_header_navigation_layout_keeps_inline_as_the_default() {
        assert_eq!(
            PageHeaderNavigationLayout::default(),
            PageHeaderNavigationLayout::InlineResponsive
        );
        assert_eq!(
            PageHeaderNavigationLayout::default().as_str(),
            "inline-responsive"
        );
    }

    #[test]
    fn dedicated_navigation_layout_has_a_stable_runtime_marker() {
        assert_eq!(
            PageHeaderNavigationLayout::DedicatedRow.as_str(),
            "dedicated-row"
        );
    }

    #[test]
    fn page_header_divider_defaults_to_shown_for_source_compatibility() {
        assert_eq!(PageHeaderDivider::default(), PageHeaderDivider::Shown);
        assert_eq!(PageHeaderDivider::default().as_str(), "shown");
    }

    #[test]
    fn page_header_divider_shown_renders_the_historical_border_and_padding() {
        assert_eq!(
            PageHeaderDivider::Shown.class(),
            "border-b border-base-300 pb-4"
        );
    }

    #[test]
    fn page_header_divider_hidden_omits_border_classes_and_has_a_stable_marker() {
        assert_eq!(PageHeaderDivider::Hidden.class(), "");
        assert_eq!(PageHeaderDivider::Hidden.as_str(), "hidden");
    }

    /// Guards the wrapping-actions-host contract at the source level: all
    /// three navigation-layout branches must render the actions slot with
    /// `flex-wrap`, never the old fixed non-wrapping row.
    #[test]
    fn actions_slot_wraps_in_every_navigation_layout_branch() {
        let source = include_str!("page_header.rs");
        let component = source
            .split_once("pub fn PageHeader(")
            .expect("PageHeader component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert_eq!(
            component
                .matches(r#"<div class="flex flex-wrap items-center gap-2" data-page-actions-slot="true">"#)
                .count(),
            3,
            "expected every navigation-layout branch to render a wrapping actions host: {component}"
        );
    }

    /// op-bjpst: the subtitle paragraph is a policy decision, not a fixed slot.
    /// Deliberate break: make `page_header_renders_subtitle` return
    /// `!subtitle.is_empty()` and the whitespace row fails; return `true`
    /// unconditionally and the empty row fails.
    #[test]
    fn page_header_renders_subtitle_only_when_it_has_visible_text() {
        assert!(!page_header_renders_subtitle(""));
        assert!(!page_header_renders_subtitle("   \n\t"));
        assert!(page_header_renders_subtitle("Supporting page description"));
        assert!(page_header_renders_subtitle("  x  "));
    }

    /// Source-level guard for the same rule at the render site: every subtitle
    /// `<p>` in ALL THREE navigation-layout branches sits inside the
    /// `page_header_renders_subtitle(..).then(..)` gate. A typed test cannot
    /// own this (the view is erased at compile time and this crate has no DOM
    /// harness), so the component source is the subject, split before
    /// `#[cfg(test)]` so this test cannot match itself. Deliberate break:
    /// restore the unconditional `<p>` in one branch; the paragraph count stays
    /// three while the gated count drops to two.
    #[test]
    fn subtitle_paragraph_is_gated_in_every_navigation_layout_branch() {
        let source = include_str!("page_header.rs");
        let component = source
            .split_once("pub fn PageHeader(")
            .expect("PageHeader component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        let paragraph = r#"<p class="max-w-3xl text-sm text-base-content/75 sm:text-base" data-page-subtitle="true">"#;
        let gate = "page_header_renders_subtitle(&subtitle).then(|| view! {";
        let paragraphs = component.matches(paragraph).count();
        assert_eq!(
            paragraphs, 3,
            "expected one subtitle paragraph per navigation-layout branch: {component}"
        );
        let gated = component
            .match_indices(paragraph)
            .filter(|(index, _)| {
                let mut start = index.saturating_sub(gate.len() + 96);
                while !component.is_char_boundary(start) {
                    start -= 1;
                }
                component[start..*index].contains(gate)
            })
            .count();
        assert_eq!(
            gated, paragraphs,
            "every subtitle paragraph must sit inside the non-empty gate: {component}"
        );
    }

    #[test]
    fn on_divider_navigation_layout_has_a_stable_runtime_marker() {
        assert_eq!(PageHeaderNavigationLayout::OnDivider.as_str(), "on-divider");
    }

    /// ldui-rhnr: the divider row reads back, trail, rule -- in that order --
    /// and the render path iterates this list, so this is the DOM order.
    /// Deliberate break: swap the `Back` and `Trail` entries of the array in
    /// `page_header_divider_parts`.
    #[test]
    fn on_divider_row_orders_back_then_trail_then_rule() {
        use PageHeaderDividerPart::{Back, Rule, Trail};
        assert_eq!(
            page_header_divider_parts(true, 2, PageHeaderDivider::Shown),
            [Back, Trail, Rule]
        );
        assert_eq!(
            page_header_divider_parts(true, 0, PageHeaderDivider::Shown),
            [Back, Rule]
        );
        assert_eq!(
            page_header_divider_parts(false, 1, PageHeaderDivider::Shown),
            [Trail, Rule]
        );
    }

    /// The row's rule REPLACES the header's bottom border: whenever the row
    /// renders it carries exactly one rule and the root carries neither the
    /// border nor its `pb-4`. Deliberate break: return `divider.class()`
    /// unconditionally from `page_header_on_divider_root_class`.
    #[test]
    fn on_divider_renders_exactly_one_rule_and_drops_the_header_border() {
        for (has_back, crumbs) in [(true, 0), (false, 1), (true, 2)] {
            let parts = page_header_divider_parts(has_back, crumbs, PageHeaderDivider::Shown);
            let rules = parts
                .iter()
                .filter(|part| **part == PageHeaderDividerPart::Rule)
                .count();
            assert_eq!(rules, 1, "back={has_back} crumbs={crumbs}: {parts:?}");
            assert_eq!(
                page_header_on_divider_root_class(&parts, PageHeaderDivider::Shown),
                "",
                "back={has_back} crumbs={crumbs}: the row's rule replaces the header border"
            );
        }
    }

    /// No back and no trail: no row at all, and the header renders today's
    /// plain divider unchanged (`Shown` keeps its border, `Hidden` stays open).
    #[test]
    fn on_divider_without_back_or_trail_falls_back_to_the_plain_divider() {
        for divider in [PageHeaderDivider::Shown, PageHeaderDivider::Hidden] {
            let parts = page_header_divider_parts(false, 0, divider);
            assert!(parts.is_empty(), "{divider:?}: {parts:?}");
            assert_eq!(
                page_header_on_divider_root_class(&parts, divider),
                divider.class(),
                "{divider:?}"
            );
        }
        assert_eq!(
            page_header_on_divider_root_class(&[], PageHeaderDivider::Shown),
            "border-b border-base-300 pb-4"
        );
    }

    /// `Hidden` keeps the navigation (back and trail) but draws no rule, and
    /// the root still carries no border.
    #[test]
    fn on_divider_with_a_hidden_divider_keeps_navigation_without_a_rule() {
        use PageHeaderDividerPart::{Back, Trail};
        let parts = page_header_divider_parts(true, 2, PageHeaderDivider::Hidden);
        assert_eq!(parts, [Back, Trail]);
        assert_eq!(
            page_header_on_divider_root_class(&parts, PageHeaderDivider::Hidden),
            ""
        );
    }

    /// The component owns `aria-current`: the last crumb is the current page
    /// whatever href it was built with; earlier crumbs are links when they
    /// have an href and plain text when they do not.
    #[test]
    fn last_crumb_is_the_current_page() {
        assert_eq!(
            page_header_crumb_kind(1, 2, false),
            PageHeaderCrumbKind::Current
        );
        assert_eq!(
            page_header_crumb_kind(1, 2, true),
            PageHeaderCrumbKind::Current
        );
        assert_eq!(
            page_header_crumb_kind(0, 1, true),
            PageHeaderCrumbKind::Current
        );
        assert_eq!(
            page_header_crumb_kind(0, 2, true),
            PageHeaderCrumbKind::Link
        );
        assert_eq!(
            page_header_crumb_kind(0, 3, false),
            PageHeaderCrumbKind::Text
        );
        assert_eq!(PageHeaderCrumbKind::Current.as_str(), "current");
        assert_eq!(PageHeaderCrumbKind::Link.as_str(), "link");
        assert_eq!(PageHeaderCrumbKind::Text.as_str(), "text");
    }

    /// Source-level guard: the `OnDivider` branch reads the same decision
    /// functions the tests above pin, and the row keeps its narrow-width
    /// contract (back never shrinks, the trail truncates, the rule keeps a
    /// floor, the row never wraps, the current crumb is plain text). Split
    /// before `#[cfg(test)]` so this test cannot match itself. Deliberate
    /// break: drop `shrink-0` from the back wrapper.
    #[test]
    fn on_divider_render_path_reads_the_parts_and_keeps_the_narrow_width_contract() {
        let source = include_str!("page_header.rs");
        let render = source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        let component = render
            .split_once("pub fn PageHeader(")
            .expect("PageHeader component source")
            .1;
        for call in [
            "page_header_divider_parts(",
            "page_header_on_divider_root_class(",
            "page_header_divider_row(",
        ] {
            assert_eq!(
                component.matches(call).count(),
                1,
                "the OnDivider branch must call {call}"
            );
        }
        for contract in [
            r#"<div class="flex shrink-0 items-center" data-page-header-back="true">"#,
            r#"<ol class="min-w-0 truncate p-1 text-sm" data-page-header-trail="true">"#,
            r#"class="min-w-8 flex-1 border-t border-base-300" data-page-header-rule="true""#,
            r#"class="flex min-w-0 flex-nowrap items-center gap-2""#,
            r#"<span class="font-medium text-base-content" aria-current="page">"#,
        ] {
            assert_eq!(
                render.matches(contract).count(),
                1,
                "divider-row markup contract missing: {contract}"
            );
        }
    }
}
