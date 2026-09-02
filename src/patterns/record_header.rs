//! Opinionated record identity row -- avatar, title, compact metadata,
//! primary status, and glyph quick actions in one responsive line.
//!
//! `RecordHeader` is the *record identity* row that sits between a
//! [`PageHeader`](super::PageHeader) (which owns back-navigation and the
//! page's single `<h1>`) and a controlled
//! [`TabSet`](crate::components::TabSet) (which owns the record's sections).
//! It is a typed composition pattern, not a record-page generator: it never
//! fetches, never owns tabs, and never runs a domain action -- every
//! callback stays consumer-owned.
//!
//! It exists because record-detail consumers (Office Account, No-Hire
//! Detail) each rebuilt an incompatible identity row on top of the generic
//! `PageHeader`, drifting on avatar size, metadata typography, link
//! semantics, status vocabulary, and -- most damagingly -- on whether a
//! glyph-only action carried an accessible name at all (`ldui-9d0q`).

use super::section_heading::HeadingLevel;
use crate::components::{
    Badge, BadgeColor, BadgeSize, BadgeStyle, Button, ButtonShape, ButtonSize, ButtonStyle, Icon,
    IconSize, LinkButton, Tooltip, TooltipPosition,
};
use crate::merge_classes;
use crate::widgets::{AvatarBadge, AvatarBadgeSize, initials_from_name};
use leptos::{
    html::{Div, Section},
    prelude::*,
};
use web_sys::wasm_bindgen::JsCast;

/// Reactive framework-owned copy for `RecordHeader`'s own generated text.
///
/// Caller-supplied text (title, metadata labels/values, status label,
/// badge labels, action labels, disabled reasons, feedback messages) is
/// **not** covered here: localize it by rebuilding the typed item lists for
/// the active locale, the same posture as [`KpiItem`](super::KpiItem) and
/// [`ActiveFilterChip`](super::ActiveFilterChip).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordHeaderTexts {
    /// Screen-reader prefix announced before the primary status label, so
    /// a status badge reads as "Status: Active" rather than a bare word.
    pub status_label: String,
    /// Truthful heading text used while the record is still loading. It is
    /// visually hidden behind the skeleton, but it is the region's real
    /// accessible name -- never a stale or fabricated record name.
    pub loading: String,
    /// Heading text used when the record could not be loaded. Parameterize
    /// it (it is reactive) to name the record that failed, e.g.
    /// `"Account ACC-2201 could not be loaded."`.
    pub unavailable: String,
    /// Notice shown above retained (possibly stale) identity data.
    pub retained: String,
    /// Accessible group name for the quick-action row.
    pub actions_label: String,
    /// Accessible name for the compact metadata list.
    pub metadata_label: String,
    /// Suffix folded into a pending action's accessible name and tooltip.
    pub pending: String,
    /// Suffix folded into an external link's accessible name.
    pub external_link: String,
}

impl Default for RecordHeaderTexts {
    fn default() -> Self {
        Self {
            status_label: "Status".to_owned(),
            loading: "Loading record".to_owned(),
            unavailable: "This record is unavailable.".to_owned(),
            retained: "Showing the last loaded record. It may be out of date.".to_owned(),
            actions_label: "Record actions".to_owned(),
            metadata_label: "Record details".to_owned(),
            pending: "in progress".to_owned(),
            external_link: "opens in a new tab".to_owned(),
        }
    }
}

/// Semantic tone shared by the primary status, the secondary badges, and
/// action feedback, so one record never speaks three status vocabularies.
///
/// **Tone is never the only channel.** Every tone-bearing element in this
/// pattern also renders its own always-visible text label, and the four
/// non-neutral tones add a distinct glyph shape on top of that (see
/// [`Self::glyph`]). Colour is the third, redundant channel -- removing it
/// entirely (greyscale, forced-colors, colour-blind viewing) loses no
/// meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecordStatusTone {
    /// No semantic emphasis (default).
    #[default]
    Neutral,
    /// Informational.
    Info,
    /// Good / healthy.
    Success,
    /// Needs attention.
    Warning,
    /// Blocking / failed.
    Error,
}

impl RecordStatusTone {
    /// Stable runtime marker, emitted as a `data-` attribute so a test can
    /// assert on tone without reading colour.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Lucide glyph paired with this tone -- a *shape* channel independent
    /// of colour. `Neutral` has none deliberately: the sprite's honest
    /// neutral glyph is blank, and an invisible glyph is worse than none.
    /// A neutral status still carries its visible text label, which is the
    /// channel that always exists.
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Info => "info",
            Self::Success => "circle-check",
            Self::Warning => "triangle-alert",
            Self::Error => "circle-alert",
        }
    }

    /// daisyUI badge colour for this tone.
    fn badge_color(self) -> BadgeColor {
        match self {
            Self::Neutral => BadgeColor::Neutral,
            Self::Info => BadgeColor::Info,
            Self::Success => BadgeColor::Success,
            Self::Warning => BadgeColor::Warning,
            Self::Error => BadgeColor::Error,
        }
    }

    /// Text colour class for feedback copy. `Neutral` stays on the muted
    /// foreground (`text-base-content/75`) rather than an `opacity-*`
    /// utility, which the style audit rejects for contrast.
    fn feedback_text_class(self) -> &'static str {
        match self {
            Self::Neutral => "text-base-content/75",
            Self::Info => "text-info",
            Self::Success => "text-success",
            Self::Warning => "text-warning",
            Self::Error => "text-error",
        }
    }
}

/// Presentation state for the whole identity row.
///
/// The row changes *within itself* -- the surrounding record page, its tabs,
/// and its panels stay mounted. That is the point: a refresh failure must
/// not blank a page the user is reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecordHeaderState {
    /// Identity is loaded and current (default).
    #[default]
    Ready,
    /// No identity yet. The row renders skeletons and a truthful
    /// visually-hidden heading; status, metadata, and actions are withheld
    /// because there is nothing yet to act on.
    Loading,
    /// Identity is real but possibly stale (a background refresh failed, or
    /// a newer revision is still in flight). Everything stays interactive
    /// and a notice says so.
    Retained,
    /// The record could not be loaded. The caller's title is deliberately
    /// NOT shown -- an identity that failed to load must never be presented
    /// as loaded -- so name the record in
    /// [`RecordHeaderTexts::unavailable`] instead.
    Unavailable,
}

impl RecordHeaderState {
    /// Stable runtime marker emitted on the row root.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Loading => "loading",
            Self::Retained => "retained",
            Self::Unavailable => "unavailable",
        }
    }

    /// Whether the caller's identity content (title, avatar, metadata,
    /// status, badges) is rendered at all.
    pub const fn shows_identity(self) -> bool {
        matches!(self, Self::Ready | Self::Retained)
    }

    /// Whether quick actions are rendered. Actions are withheld whenever
    /// there is no trustworthy record to act on.
    pub const fn shows_actions(self) -> bool {
        self.shows_identity()
    }

    /// Whether the row reports `aria-busy` to assistive technology.
    pub const fn is_busy(self) -> bool {
        matches!(self, Self::Loading)
    }
}

/// Avatar/persona input for the identity cluster.
///
/// Deliberately initials-only: the shared [`AvatarBadge`] palette is
/// deterministic and contrast-safe in every theme by construction, whereas
/// a remote portrait needs load/error/fallback states this row does not own.
/// A consumer with real portraits composes [`Persona`](crate::components::Persona)
/// above the row instead.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordAvatar {
    /// Display name driving the deterministic colour palette.
    pub name: String,
    /// Initials rendered inside the circle.
    pub initials: String,
}

impl RecordAvatar {
    /// Creates an avatar whose initials are derived from `name` via
    /// [`initials_from_name`].
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let initials = initials_from_name(&name);
        Self { name, initials }
    }

    /// Overrides the derived initials (e.g. an organisation's ticker).
    pub fn initials(mut self, initials: impl Into<String>) -> Self {
        self.initials = initials.into();
        self
    }
}

/// One compact metadata item -- a label/value pair with optional link
/// semantics, rendered into the row's description list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordMetaItem {
    /// Stable identity, used for the list key and `data-record-meta-item`.
    pub id: String,
    /// Short term, e.g. `"Matter"`.
    pub label: String,
    /// The value itself, e.g. `"MAT-1023"`.
    pub value: String,
    /// When set, the value renders as an anchor to this href.
    pub href: Option<String>,
    /// Whether the link opens in a new browsing context. Ignored without
    /// [`Self::link`] -- see [`Self::is_external_link`].
    pub external: bool,
    /// Optional leading Lucide glyph. Always `aria-hidden`; the label is
    /// the accessible channel.
    pub icon: String,
}

impl RecordMetaItem {
    /// Creates a plain, non-link metadata item.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            href: None,
            external: false,
            icon: String::new(),
        }
    }

    /// Gives the value link semantics.
    pub fn link(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Marks the link as opening in a new tab. Only meaningful together
    /// with [`Self::link`].
    pub fn external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Sets the leading glyph.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Whether this item actually renders an external anchor. `external`
    /// alone is not enough -- there must be an href to open.
    pub fn is_external_link(&self) -> bool {
        self.external && self.href.is_some()
    }

    /// Accessible name for the value's anchor.
    ///
    /// A bare `"MAT-1023"` link is ambiguous once a screen reader lists it
    /// out of context, so the term is folded in. The visible text stays a
    /// substring of the name, satisfying WCAG 2.5.3 (Label in Name).
    pub fn link_accessible_name(&self, texts: &RecordHeaderTexts) -> String {
        let mut name = format!("{}: {}", self.label, self.value);
        if self.is_external_link() {
            name.push_str(", ");
            name.push_str(&texts.external_link);
        }
        name
    }
}

/// The record's single primary status.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordStatus {
    /// Always-visible status word, e.g. `"Active"`. This is the channel
    /// that makes the status readable without colour.
    pub label: String,
    /// Semantic tone.
    pub tone: RecordStatusTone,
    /// Optional explanation surfaced on hover/focus and folded into the
    /// accessible name.
    pub detail: String,
}

impl RecordStatus {
    /// Creates a neutral status.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tone: RecordStatusTone::default(),
            detail: String::new(),
        }
    }

    /// Sets the semantic tone.
    pub fn tone(mut self, tone: RecordStatusTone) -> Self {
        self.tone = tone;
        self
    }

    /// Sets the explanatory detail.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    /// Screen-reader text prefixed inside the badge, so the badge announces
    /// its role rather than a bare adjective.
    pub fn accessible_name(&self, texts: &RecordHeaderTexts) -> String {
        let mut name = format!("{}: {}", texts.status_label, self.label);
        if !self.detail.is_empty() {
            name.push_str(", ");
            name.push_str(&self.detail);
        }
        name
    }
}

/// One secondary badge -- a classification that qualifies the record but is
/// not its primary status (`"VIP"`, `"Do not contact"`, `"Contract"`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordBadge {
    /// Stable identity, used for the list key and `data-record-badge`.
    pub id: String,
    /// Always-visible label.
    pub label: String,
    /// Semantic tone.
    pub tone: RecordStatusTone,
}

impl RecordBadge {
    /// Creates a neutral secondary badge.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tone: RecordStatusTone::default(),
        }
    }

    /// Sets the semantic tone.
    pub fn tone(mut self, tone: RecordStatusTone) -> Self {
        self.tone = tone;
        self
    }
}

/// Availability of one quick action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RecordQuickActionState {
    /// Actionable (default).
    #[default]
    Ready,
    /// The consumer's callback is in flight. The control keeps its place in
    /// the row, swaps its glyph for a spinner, reports `aria-busy`, and
    /// refuses further activation.
    Pending,
    /// Unavailable, carrying the reason every user needs to see.
    ///
    /// Rendered as `aria-disabled`, **not** native `disabled`: a natively
    /// disabled button is removed from the tab order, so its tooltip can
    /// never be reached by keyboard and the reason becomes invisible to
    /// exactly the users who most need it.
    Disabled(String),
}

impl RecordQuickActionState {
    /// Stable runtime marker emitted on the control.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Pending => "pending",
            Self::Disabled(_) => "disabled",
        }
    }

    /// Whether activation should reach the consumer's callback.
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// The disabled reason, when there is one.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Disabled(reason) => Some(reason.as_str()),
            _ => None,
        }
    }
}

/// Keyed, transient result of a quick action -- the consumer's action
/// outcome rendered back into the row's live region under the action's own
/// id, so a glyph-only control can report what it did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordActionFeedback {
    /// The message itself. This text, not the tone colour, carries the
    /// outcome.
    pub message: String,
    /// Semantic tone.
    pub tone: RecordStatusTone,
}

impl RecordActionFeedback {
    /// Creates neutral feedback.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tone: RecordStatusTone::default(),
        }
    }

    /// Sets the semantic tone.
    pub fn tone(mut self, tone: RecordStatusTone) -> Self {
        self.tone = tone;
        self
    }
}

/// One ordered glyph quick action.
///
/// `RecordHeader` owns the control's shape, accessible name, hover/focus
/// help, focus ring, and state presentation. It never owns what the action
/// *does*: a link action renders an anchor to `href`, and every other
/// action reports its `id` through the row's `on_action` callback.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordQuickAction {
    /// Stable identity. It keys the list, `data-record-action`, the
    /// `on_action` payload, and this action's feedback line.
    pub id: String,
    /// Lucide glyph name -- the control's only visible content.
    pub icon: String,
    /// The action's name. Never rendered as visible text (this is a
    /// glyph-only control), always rendered as the accessible name and the
    /// tooltip.
    pub label: String,
    /// When set, the action renders as a link rather than a button.
    pub href: Option<String>,
    /// Whether the link opens in a new browsing context. Ignored without
    /// [`Self::link`].
    pub external: bool,
    /// Availability.
    pub state: RecordQuickActionState,
    /// Keyed outcome copy for the row's live region.
    pub feedback: Option<RecordActionFeedback>,
}

impl RecordQuickAction {
    /// Creates a ready, button-shaped action.
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            href: None,
            external: false,
            state: RecordQuickActionState::default(),
            feedback: None,
        }
    }

    /// Gives the action link semantics.
    pub fn link(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Marks the link as opening in a new tab. Only meaningful together
    /// with [`Self::link`].
    pub fn external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Marks the action in flight.
    pub fn pending(mut self) -> Self {
        self.state = RecordQuickActionState::Pending;
        self
    }

    /// Marks the action unavailable, with the reason shown to every user.
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.state = RecordQuickActionState::Disabled(reason.into());
        self
    }

    /// Attaches keyed outcome feedback.
    pub fn feedback(mut self, feedback: RecordActionFeedback) -> Self {
        self.feedback = Some(feedback);
        self
    }

    /// Whether this action actually renders an external anchor.
    pub fn is_external_link(&self) -> bool {
        self.external && self.href.is_some()
    }

    /// Whether this action renders as an anchor. A non-`Ready` link renders
    /// as a button instead: there is no accessible way to express a
    /// disabled or in-flight anchor, and a still-navigable link would lie
    /// about its own availability.
    pub fn renders_as_link(&self) -> bool {
        self.href.is_some() && self.state.is_actionable()
    }

    /// The control's accessible name -- and, by construction, the exact
    /// text of its tooltip, so the visual and assistive channels can never
    /// disagree.
    ///
    /// A glyph carries no name of its own, so this string is the *only*
    /// name the control has. It always starts with the action's label and
    /// appends the state qualifier: `"Archive (locked while a review is
    /// open)"`.
    pub fn accessible_name(&self, texts: &RecordHeaderTexts) -> String {
        let mut name = self.label.clone();
        match &self.state {
            RecordQuickActionState::Ready => {}
            RecordQuickActionState::Pending => {
                name.push_str(" (");
                name.push_str(&texts.pending);
                name.push(')');
            }
            RecordQuickActionState::Disabled(reason) => {
                name.push_str(" (");
                name.push_str(reason);
                name.push(')');
            }
        }
        if self.is_external_link() {
            name.push_str(", ");
            name.push_str(&texts.external_link);
        }
        name
    }
}

/// Classes for a quick-action control. Unavailable actions are muted with
/// `text-base-content/75`, never an `opacity-*` utility, and never daisyUI's
/// own `btn-disabled` -- that sets `pointer-events: none`, which silently
/// kills the very tooltip carrying the reason.
fn quick_action_class(state: &RecordQuickActionState) -> &'static str {
    match state {
        RecordQuickActionState::Ready => "",
        RecordQuickActionState::Pending => "cursor-progress",
        RecordQuickActionState::Disabled(_) => "cursor-not-allowed text-base-content/75",
    }
}

/// Renders nothing for empty copy -- mirrors
/// [`SectionHeading`](super::SectionHeading)'s `has_text`.
fn has_text(value: &str) -> bool {
    !value.is_empty()
}

/// Compact identity, status, and quick-action row for a record page.
///
/// Composition, top to bottom: [`PageHeader`](super::PageHeader) owns back
/// navigation and the page `<h1>`; `RecordHeader` owns the identity row and
/// starts one heading level below it (`H2` by default -- see
/// [`HeadingLevel`]); a controlled [`TabSet`](crate::components::TabSet)
/// owns the record's sections underneath. `RecordHeader` fetches nothing and
/// owns no tab state.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{
///     PageHeader, RecordActionFeedback, RecordAvatar, RecordBadge, RecordHeader,
///     RecordHeaderState, RecordMetaItem, RecordQuickAction, RecordStatus, RecordStatusTone,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let on_action = Callback::new(|id: String| leptos::logging::log!("{id}"));
///     view! {
///         <PageHeader title="Accounts" subtitle="Every account in this office." />
///         <RecordHeader
///             id="account-identity"
///             title="Northwind Logistics"
///             avatar=Some(RecordAvatar::new("Northwind Logistics"))
///             metadata=vec![
///                 RecordMetaItem::new("owner", "Owner", "Maria Gonzalez"),
///                 RecordMetaItem::new("matter", "Matter", "MAT-1023")
///                     .link("/matters/1023")
///                     .icon("file-text"),
///             ]
///             status=Some(
///                 RecordStatus::new("Active").tone(RecordStatusTone::Success)
///             )
///             badges=vec![RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info)]
///             actions=vec![
///                 RecordQuickAction::new("call", "phone", "Call account"),
///                 RecordQuickAction::new("email", "mail", "Email account")
///                     .feedback(RecordActionFeedback::new("Draft opened")),
///                 RecordQuickAction::new("archive", "trash", "Archive account")
///                     .disabled("Locked while a review is open"),
///             ]
///             on_action=on_action
///             state=RecordHeaderState::Ready
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex min-w-0 flex-col gap-2 gap-3 lg:flex-row lg:items-center lg:justify-between");
/// @source inline("flex-1 shrink-0 items-start items-center flex-wrap lg:justify-end truncate");
/// @source inline("gap-1 gap-x-4 gap-y-1 rounded-full sr-only inline-flex");
/// @source inline("skeleton h-4 h-6 h-10 w-10 w-1/3 w-1/2 w-full");
/// @source inline("link link-hover cursor-not-allowed cursor-progress");
/// @source inline("loading loading-spinner loading-sm font-semibold tracking-tight tracking-wide uppercase");
/// @source inline("text-base-content text-base-content/75 text-info text-success text-warning text-error");
/// @source inline("forced-colors:text-[CanvasText]");
/// ```
///
/// The `ld-text-*` steps are NOT listed above on purpose: they are not
/// Tailwind utilities, so `@source inline(...)` cannot generate them. They
/// are authored rules emitted into `styles/tokens.css` by
/// `cargo xtask gen-tokens`, so a consumer gets them by IMPORTING that
/// stylesheet (ldui-h7tw, ldui-fg2h).
///
/// ## Node References
/// - `node_ref` - References the outer `<section>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn RecordHeader(
    /// Primary record title. Truncates with an ellipsis rather than
    /// wrapping into or over the status/actions edge; the full string stays
    /// available through the heading's `title` attribute.
    #[prop(into)]
    title: Signal<String>,

    /// Stable id placed on the heading. Supplying it also names the row's
    /// wrapping `<section>` through `aria-labelledby`, promoting it to a
    /// navigable region. Omitted when empty.
    #[prop(optional, into)]
    id: &'static str,

    /// Optional initials avatar for the identity cluster.
    #[prop(optional, into)]
    avatar: Signal<Option<RecordAvatar>>,

    /// Compact metadata items, in order.
    #[prop(optional, into)]
    metadata: Signal<Vec<RecordMetaItem>>,

    /// The record's single primary status.
    #[prop(optional, into)]
    status: Signal<Option<RecordStatus>>,

    /// Secondary classification badges, in order.
    #[prop(optional, into)]
    badges: Signal<Vec<RecordBadge>>,

    /// Ordered glyph quick actions.
    #[prop(optional, into)]
    actions: Signal<Vec<RecordQuickAction>>,

    /// Activation callback for non-link actions, receiving the action's
    /// `id`. Never fired for a pending or disabled action.
    #[prop(optional)]
    on_action: Option<Callback<String>>,

    /// Presentation state. See [`RecordHeaderState`].
    #[prop(optional, into)]
    state: Signal<RecordHeaderState>,

    /// Heading element and type-ramp step. Defaults to `H2`, one level
    /// below the page's own top-level heading.
    #[prop(optional)]
    level: HeadingLevel,

    /// Framework-owned copy.
    #[prop(into, default = Signal::stored(RecordHeaderTexts::default()))]
    texts: Signal<RecordHeaderTexts>,

    /// Additional classes for the outer wrapper.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer `<section>` element.
    #[prop(optional)]
    node_ref: NodeRef<Section>,
) -> impl IntoView {
    let heading_id = (!id.is_empty()).then_some(id);
    let heading = HeadingSpec {
        level,
        id: heading_id,
        class: format!(
            "{} min-w-0 truncate font-semibold tracking-tight text-base-content forced-colors:text-[CanvasText]",
            level.text_class()
        ),
    };

    view! {
        <section
            node_ref=node_ref
            class=move || merge_classes!("flex min-w-0 flex-col gap-2", class)
            data-record-header="true"
            data-record-header-state=move || state.get().as_str()
            data-record-header-level=level.as_str()
            aria-labelledby=heading_id
            aria-busy=move || state.get().is_busy().then_some("true")
        >
            <div class="flex min-w-0 flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div class="flex min-w-0 flex-1 items-center gap-3" data-record-identity="true">
                    {move || render_identity(
                        state.get(),
                        avatar.get(),
                        &heading,
                        title.get(),
                        &texts.get(),
                        metadata.get(),
                    )}
                </div>
                {move || {
                    let state = state.get();
                    let texts = texts.get();
                    let status = state.shows_identity().then(|| status.get()).flatten();
                    let badges = if state.shows_identity() { badges.get() } else { Vec::new() };
                    let actions = if state.shows_actions() { actions.get() } else { Vec::new() };
                    (status.is_some() || !badges.is_empty() || !actions.is_empty()).then(|| view! {
                        <div
                            class="flex shrink-0 flex-wrap items-center gap-2 lg:justify-end"
                            data-record-header-edge="true"
                        >
                            {status.map(|status| render_status(&status, &texts))}
                            {badges.into_iter().map(render_badge).collect_view()}
                            {(!actions.is_empty()).then(|| view! {
                                <div
                                    role="group"
                                    aria-label=texts.actions_label.clone()
                                    class="flex flex-wrap items-center gap-2"
                                    data-record-actions="true"
                                >
                                    {actions.into_iter()
                                        .map(|action| render_action(action, &texts, on_action, node_ref))
                                        .collect_view()}
                                </div>
                            })}
                        </div>
                    })
                }}
            </div>

            {move || {
                let texts = texts.get();
                match state.get() {
                    RecordHeaderState::Retained => Some(view! {
                        <p
                            class="ld-text-small text-base-content/75 forced-colors:text-[CanvasText]"
                            data-record-header-notice="retained"
                        >
                            {texts.retained}
                        </p>
                    }),
                    _ => None,
                }
            }}

            <div
                role="status"
                aria-live="polite"
                class="flex min-w-0 flex-col gap-1"
                data-record-header-feedback="true"
            >
                {move || {
                    if !state.get().shows_actions() {
                        return Vec::new().collect_view();
                    }
                    actions.get()
                        .into_iter()
                        .filter_map(|action| {
                            let feedback = action.feedback.clone()?;
                            has_text(&feedback.message).then(|| view! {
                                <p
                                    class=format!(
                                        "ld-text-small {} forced-colors:text-[CanvasText]",
                                        feedback.tone.feedback_text_class()
                                    )
                                    data-record-action-feedback=action.id.clone()
                                    data-record-action-feedback-tone=feedback.tone.as_str()
                                >
                                    {feedback.message}
                                </p>
                            })
                        })
                        .collect_view()
                }}
            </div>
        </section>
    }
}

/// The row's one heading: which element to render, the id that names the
/// wrapping region, and the resolved type-ramp classes. Bundled so every
/// presentation branch renders the same heading contract rather than
/// re-deriving it.
struct HeadingSpec {
    level: HeadingLevel,
    id: Option<&'static str>,
    class: String,
}

impl HeadingSpec {
    /// Renders the heading element matching `level`, so the row never skips
    /// a heading level relative to the page's own top-level heading
    /// (WCAG 1.3.1). `class` is passed explicitly because the loading
    /// branch renders the same heading visually hidden.
    fn render(&self, class: &str, text: String, title_attr: Option<String>) -> AnyView {
        render_heading(self.level, self.id, class, text, title_attr)
    }
}

/// Identity cluster: avatar plus heading plus metadata, or the loading /
/// unavailable substitutes. Every branch renders exactly one heading, so
/// the row's accessible name is always truthful about what is on screen.
fn render_identity(
    state: RecordHeaderState,
    avatar: Option<RecordAvatar>,
    heading: &HeadingSpec,
    title: String,
    texts: &RecordHeaderTexts,
    metadata: Vec<RecordMetaItem>,
) -> AnyView {
    match state {
        RecordHeaderState::Loading => {
            let heading = heading.render("sr-only", texts.loading.clone(), None);
            view! {
                {heading}
                <div class="flex min-w-0 flex-1 items-center gap-3" aria-hidden="true">
                    <div class="skeleton h-10 w-10 rounded-full"></div>
                    <div class="flex min-w-0 flex-1 flex-col gap-2">
                        <div class="skeleton h-6 w-1/3"></div>
                        <div class="skeleton h-4 w-1/2"></div>
                    </div>
                </div>
            }
            .into_any()
        }
        RecordHeaderState::Unavailable => {
            heading.render(&heading.class, texts.unavailable.clone(), None)
        }
        RecordHeaderState::Ready | RecordHeaderState::Retained => {
            let full_title = title.clone();
            let heading = heading.render(&heading.class, title, Some(full_title));
            view! {
                {avatar.map(|avatar| view! {
                    <span class="shrink-0" aria-hidden="true" data-record-avatar="true">
                        <AvatarBadge
                            initials=avatar.initials.clone()
                            name=avatar.name.clone()
                            size=AvatarBadgeSize::Md
                        />
                    </span>
                })}
                <div class="flex min-w-0 flex-1 flex-col gap-1">
                    {heading}
                    {(!metadata.is_empty()).then(|| {
                        let label = texts.metadata_label.clone();
                        let items = metadata
                            .into_iter()
                            .map(|item| render_meta_item(item, texts))
                            .collect_view();
                        view! {
                            <dl
                                class="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-1"
                                aria-label=label
                                data-record-metadata="true"
                            >
                                {items}
                            </dl>
                        }
                    })}
                </div>
            }
            .into_any()
        }
    }
}

/// Emits the concrete heading element for a level. Kept separate from
/// [`HeadingSpec::render`] so the level-to-tag mapping is one match arm per
/// level and nothing else.
fn render_heading(
    level: HeadingLevel,
    id: Option<&'static str>,
    class: &str,
    text: String,
    title_attr: Option<String>,
) -> AnyView {
    let class = class.to_owned();
    match level {
        HeadingLevel::H2 => view! { <h2 id=id class=class title=title_attr>{text}</h2> }.into_any(),
        HeadingLevel::H3 => view! { <h3 id=id class=class title=title_attr>{text}</h3> }.into_any(),
        HeadingLevel::H4 => view! { <h4 id=id class=class title=title_attr>{text}</h4> }.into_any(),
    }
}

/// One metadata pair. Terms and values live in a real description list, so
/// a screen reader pairs them without a fabricated separator character.
fn render_meta_item(item: RecordMetaItem, texts: &RecordHeaderTexts) -> AnyView {
    let icon = item.icon.clone();
    let value = item.value.clone();
    let accessible_name = item.link_accessible_name(texts);
    let external = item.is_external_link();
    let href = item.href.clone();
    view! {
        <div class="flex min-w-0 items-center gap-1" data-record-meta-item=item.id.clone()>
            {has_text(&icon).then(|| view! {
                <span aria-hidden="true" class="shrink-0">
                    <Icon name=icon size=IconSize::XSmall color="text-base-content/75" />
                </span>
            })}
            <dt class="ld-text-small shrink-0 font-semibold uppercase tracking-wide text-base-content/75 forced-colors:text-[CanvasText]">
                {item.label.clone()}
            </dt>
            <dd class="ld-text-caption min-w-0 truncate text-base-content forced-colors:text-[CanvasText]">
                {match href {
                    Some(href) => view! {
                        <a
                            href=href
                            class="link link-hover"
                            aria-label=accessible_name
                            target=external.then_some("_blank")
                            rel=external.then_some("noopener noreferrer")
                            data-record-meta-link="true"
                        >
                            {value}
                        </a>
                    }
                    .into_any(),
                    None => view! { <span>{value}</span> }.into_any(),
                }}
            </dd>
        </div>
    }
    .into_any()
}

/// The primary status badge: a visually-hidden role prefix, an optional
/// tone glyph, and the always-visible status word.
fn render_status(status: &RecordStatus, texts: &RecordHeaderTexts) -> AnyView {
    let accessible_name = status.accessible_name(texts);
    let glyph = status.tone.glyph();
    let label = status.label.clone();
    let tone = status.tone;
    let detail = status.detail.clone();
    let badge = view! {
        <Badge
            color=tone.badge_color()
            size=BadgeSize::Md
            class="gap-1"
            attr:data-record-status="true"
            attr:data-record-status-tone=tone.as_str()
        >
            <span class="sr-only">{accessible_name}</span>
            {(!glyph.is_empty()).then(|| view! {
                <span aria-hidden="true" class="inline-flex">
                    <Icon name=glyph size=IconSize::XSmall />
                </span>
            })}
            <span aria-hidden="true">{label}</span>
        </Badge>
    };
    if has_text(&detail) {
        view! { <Tooltip tip=detail>{badge}</Tooltip> }.into_any()
    } else {
        badge.into_any()
    }
}

/// One secondary badge. `Soft` style keeps it visibly subordinate to the
/// primary status without changing its tone vocabulary.
fn render_badge(badge: RecordBadge) -> AnyView {
    view! {
        <Badge
            style=BadgeStyle::Soft
            color=badge.tone.badge_color()
            size=BadgeSize::Sm
            attr:data-record-badge=badge.id.clone()
            attr:data-record-badge-tone=badge.tone.as_str()
        >
            {badge.label.clone()}
        </Badge>
    }
    .into_any()
}

/// Conservative estimate of half the width (CSS px) a hovering daisyUI
/// tooltip bubble will render at, from the length of the text alone.
///
/// DaisyUI's tooltip has no DOM node to measure ahead of time -- its
/// visible content is a `::before` pseudo-element whose text comes from a
/// `data-tip` attribute -- so there is nothing a real
/// `getBoundingClientRect` call can read before the control is hovered or
/// focused. This stands in: average glyph width plus the bubble's own
/// inline padding (`padding-inline: .5rem` each side in daisyUI's tooltip
/// CSS), capped at daisyUI's own `max-width: 20rem` for the bubble. It
/// leans wide on purpose -- overestimating only flips a control that would
/// in fact have fit without flipping, while underestimating lets the exact
/// spill this pattern exists to prevent back in.
fn estimated_tooltip_half_width(label_len: usize) -> f64 {
    const AVG_GLYPH_PX: f64 = 7.0;
    const BUBBLE_INLINE_PADDING_PX: f64 = 16.0;
    const MAX_BUBBLE_WIDTH_PX: f64 = 320.0;
    let natural = (label_len as f64) * AVG_GLYPH_PX + BUBBLE_INLINE_PADDING_PX;
    natural.min(MAX_BUBBLE_WIDTH_PX) / 2.0
}

/// Chooses which edge of the trigger a tooltip bubble should hang off of,
/// from real geometry -- never from an action's position in the list. A
/// trigger with less than `half_width` of room to the row's right edge
/// flips to `Left`; one with less room to the left (an unusual layout, but
/// not an impossible one) flips to `Right`; a trigger with room on both
/// sides keeps daisyUI's own default `Top`, centred on the trigger. This is
/// what keeps the fix correct as the action count changes: whichever
/// control ends up nearest an edge is the one that flips, regardless of
/// whether it happens to be first, last, or in the middle.
fn resolved_tooltip_position(
    row_left: f64,
    row_right: f64,
    trigger_left: f64,
    trigger_width: f64,
    half_width: f64,
) -> TooltipPosition {
    let trigger_center = trigger_left + trigger_width / 2.0;
    let right_room = row_right - trigger_center;
    let left_room = trigger_center - row_left;
    if right_room < half_width && left_room >= half_width {
        TooltipPosition::Left
    } else if left_room < half_width && right_room >= half_width {
        TooltipPosition::Right
    } else {
        TooltipPosition::Top
    }
}

/// Measures `row` (RecordHeader's own root section) and `trigger` (the
/// tooltip's wrapping div) and writes the resulting placement into
/// `position`. A no-op until both have mounted, so it is safe to call from
/// an effect that runs before the first paint as well as from a later
/// hover/focus event.
fn measure_and_set_tooltip_position(
    row: NodeRef<Section>,
    trigger: NodeRef<Div>,
    position: RwSignal<TooltipPosition>,
    label_len: usize,
) {
    let (Some(row_el), Some(trigger_el)) = (row.get_untracked(), trigger.get_untracked()) else {
        return;
    };
    let row_rect = row_el
        .unchecked_ref::<web_sys::Element>()
        .get_bounding_client_rect();
    let trigger_rect = trigger_el
        .unchecked_ref::<web_sys::Element>()
        .get_bounding_client_rect();
    let half_width = estimated_tooltip_half_width(label_len);
    position.set(resolved_tooltip_position(
        row_rect.left(),
        row_rect.right(),
        trigger_rect.left(),
        trigger_rect.width(),
        half_width,
    ));
}

/// One glyph quick action, wrapped in a tooltip carrying the same string as
/// its accessible name.
///
/// The tooltip's placement is measured against `row` (RecordHeader's own
/// root section) rather than hardcoded to the action's position in the
/// list -- see [`resolved_tooltip_position`]. It is computed once the
/// control mounts (a hovering-but-invisible tooltip bubble still occupies
/// layout space, so the row must never spill even before the first hover)
/// and re-measured on hover/focus in case the surrounding layout has since
/// reflowed.
fn render_action(
    action: RecordQuickAction,
    texts: &RecordHeaderTexts,
    on_action: Option<Callback<String>>,
    row: NodeRef<Section>,
) -> AnyView {
    let name = action.accessible_name(texts);
    let label_len = name.chars().count();
    let state_marker = action.state.as_str();
    let extra_class = quick_action_class(&action.state);
    let actionable = action.state.is_actionable();
    let pending = matches!(action.state, RecordQuickActionState::Pending);
    let disabled = matches!(action.state, RecordQuickActionState::Disabled(_));
    let id = action.id.clone();
    let icon = action.icon.clone();
    let external = action.is_external_link();

    let glyph = if pending {
        view! { <span class="loading loading-spinner loading-sm" aria-hidden="true"></span> }
            .into_any()
    } else {
        view! {
            <span aria-hidden="true" class="inline-flex">
                <Icon name=icon size=IconSize::Small />
            </span>
        }
        .into_any()
    };

    let control = if action.renders_as_link() {
        let href = action.href.clone().unwrap_or_default();
        view! {
            <LinkButton
                href=href
                style=ButtonStyle::Ghost
                size=ButtonSize::Sm
                shape=ButtonShape::Square
                class=extra_class
                attr:aria-label=name.clone()
                attr:target=external.then_some("_blank")
                attr:rel=external.then_some("noopener noreferrer")
                attr:data-record-action=id.clone()
                attr:data-record-action-state=state_marker
            >
                {glyph}
            </LinkButton>
        }
        .into_any()
    } else {
        let click_id = id.clone();
        view! {
            <Button
                style=ButtonStyle::Ghost
                size=ButtonSize::Sm
                shape=ButtonShape::Square
                class=extra_class
                attr:aria-label=name.clone()
                attr:aria-disabled=(pending || disabled).then_some("true")
                attr:aria-busy=pending.then_some("true")
                attr:data-record-action=id.clone()
                attr:data-record-action-state=state_marker
                on_click=Callback::new(move |_| {
                    if !actionable {
                        return;
                    }
                    if let Some(on_action) = on_action {
                        on_action.run(click_id.clone());
                    }
                })
            >
                {glyph}
            </Button>
        }
        .into_any()
    };

    let tooltip_ref = NodeRef::<Div>::new();
    let tooltip_position = RwSignal::new(TooltipPosition::default());

    // Runs once the tooltip and the row it is measured against have both
    // mounted -- before the first paint, so a spilling default `Top`
    // placement never becomes visible even for an instant.
    Effect::new(move |_| {
        if row.get().is_some() && tooltip_ref.get().is_some() {
            measure_and_set_tooltip_position(row, tooltip_ref, tooltip_position, label_len);
        }
    });

    view! {
        <Tooltip
            node_ref=tooltip_ref
            tip=name
            position=tooltip_position
            on:pointerenter=move |_| {
                measure_and_set_tooltip_position(row, tooltip_ref, tooltip_position, label_len);
            }
            on:focusin=move |_| {
                measure_and_set_tooltip_position(row, tooltip_ref, tooltip_position, label_len);
            }
        >
            {control}
        </Tooltip>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts() -> RecordHeaderTexts {
        RecordHeaderTexts::default()
    }

    // -- tones -------------------------------------------------------------

    #[test]
    fn tone_defaults_to_neutral_and_has_a_stable_marker() {
        assert_eq!(RecordStatusTone::default(), RecordStatusTone::Neutral);
        assert_eq!(RecordStatusTone::default().as_str(), "neutral");
    }

    #[test]
    fn every_tone_maps_to_a_distinct_marker_and_badge_color() {
        let tones = [
            RecordStatusTone::Neutral,
            RecordStatusTone::Info,
            RecordStatusTone::Success,
            RecordStatusTone::Warning,
            RecordStatusTone::Error,
        ];
        let markers: std::collections::HashSet<&str> =
            tones.iter().map(|tone| tone.as_str()).collect();
        assert_eq!(markers.len(), tones.len());
        assert_eq!(
            RecordStatusTone::Neutral.badge_color().as_str(),
            "badge-neutral"
        );
        assert_eq!(RecordStatusTone::Info.badge_color().as_str(), "badge-info");
        assert_eq!(
            RecordStatusTone::Success.badge_color().as_str(),
            "badge-success"
        );
        assert_eq!(
            RecordStatusTone::Warning.badge_color().as_str(),
            "badge-warning"
        );
        assert_eq!(
            RecordStatusTone::Error.badge_color().as_str(),
            "badge-error"
        );
    }

    /// The four semantic tones must each carry a DISTINCT glyph, so tone is
    /// legible as a shape and not only as a hue. `Neutral` is deliberately
    /// glyph-less -- see [`RecordStatusTone::glyph`].
    #[test]
    fn semantic_tones_carry_distinct_glyph_shapes() {
        let glyphs = [
            RecordStatusTone::Info.glyph(),
            RecordStatusTone::Success.glyph(),
            RecordStatusTone::Warning.glyph(),
            RecordStatusTone::Error.glyph(),
        ];
        assert!(glyphs.iter().all(|glyph| !glyph.is_empty()));
        let distinct: std::collections::HashSet<&&str> = glyphs.iter().collect();
        assert_eq!(distinct.len(), glyphs.len());
        assert_eq!(RecordStatusTone::Neutral.glyph(), "");
    }

    /// Every glyph must actually resolve in the shipped sprite. An unknown
    /// Lucide name silently degrades to `blank`, which would leave tone
    /// conveyed by colour alone -- the exact defect this mapping exists to
    /// prevent.
    #[test]
    fn every_tone_glyph_resolves_in_the_sprite() {
        use crate::components::lucide_to_sprite;
        for tone in [
            RecordStatusTone::Info,
            RecordStatusTone::Success,
            RecordStatusTone::Warning,
            RecordStatusTone::Error,
        ] {
            assert_ne!(
                lucide_to_sprite(tone.glyph()),
                "blank",
                "tone {} has no sprite glyph",
                tone.as_str()
            );
        }
    }

    #[test]
    fn feedback_text_uses_muted_foreground_never_an_opacity_utility() {
        for tone in [
            RecordStatusTone::Neutral,
            RecordStatusTone::Info,
            RecordStatusTone::Success,
            RecordStatusTone::Warning,
            RecordStatusTone::Error,
        ] {
            assert!(!tone.feedback_text_class().contains("opacity-"));
        }
        assert_eq!(
            RecordStatusTone::Neutral.feedback_text_class(),
            "text-base-content/75"
        );
    }

    // -- state -------------------------------------------------------------

    #[test]
    fn header_state_defaults_to_ready_with_a_stable_marker() {
        assert_eq!(RecordHeaderState::default(), RecordHeaderState::Ready);
        assert_eq!(RecordHeaderState::default().as_str(), "ready");
    }

    #[test]
    fn header_state_markers_are_distinct() {
        let states = [
            RecordHeaderState::Ready,
            RecordHeaderState::Loading,
            RecordHeaderState::Retained,
            RecordHeaderState::Unavailable,
        ];
        let markers: std::collections::HashSet<&str> =
            states.iter().map(|state| state.as_str()).collect();
        assert_eq!(markers.len(), states.len());
    }

    /// Retained data stays fully usable -- that is what separates it from a
    /// replacement state. Loading and Unavailable withhold identity and
    /// actions because there is nothing truthful to show or act on.
    #[test]
    fn only_ready_and_retained_show_identity_and_actions() {
        assert!(RecordHeaderState::Ready.shows_identity());
        assert!(RecordHeaderState::Retained.shows_identity());
        assert!(!RecordHeaderState::Loading.shows_identity());
        assert!(!RecordHeaderState::Unavailable.shows_identity());
        for state in [
            RecordHeaderState::Ready,
            RecordHeaderState::Loading,
            RecordHeaderState::Retained,
            RecordHeaderState::Unavailable,
        ] {
            assert_eq!(state.shows_actions(), state.shows_identity());
        }
    }

    #[test]
    fn only_loading_reports_busy() {
        assert!(RecordHeaderState::Loading.is_busy());
        assert!(!RecordHeaderState::Ready.is_busy());
        assert!(!RecordHeaderState::Retained.is_busy());
        assert!(!RecordHeaderState::Unavailable.is_busy());
    }

    // -- avatar ------------------------------------------------------------

    #[test]
    fn avatar_derives_initials_from_the_display_name() {
        assert_eq!(RecordAvatar::new("Maria Gonzalez").initials, "MG");
        assert_eq!(RecordAvatar::new("Cher").initials, "C");
        assert_eq!(RecordAvatar::new("").initials, "?");
    }

    #[test]
    fn avatar_initials_can_be_overridden_without_losing_the_palette_name() {
        let avatar = RecordAvatar::new("Northwind Logistics").initials("NW");
        assert_eq!(avatar.initials, "NW");
        assert_eq!(avatar.name, "Northwind Logistics");
    }

    // -- metadata ----------------------------------------------------------

    #[test]
    fn meta_item_defaults_to_a_plain_non_link_value() {
        let item = RecordMetaItem::new("owner", "Owner", "Maria Gonzalez");
        assert!(item.href.is_none());
        assert!(!item.external);
        assert!(!item.is_external_link());
        assert!(item.icon.is_empty());
    }

    #[test]
    fn meta_item_link_and_icon_builders_set_exactly_what_they_name() {
        let item = RecordMetaItem::new("matter", "Matter", "MAT-1023")
            .link("/matters/1023")
            .icon("file-text");
        assert_eq!(item.href.as_deref(), Some("/matters/1023"));
        assert_eq!(item.icon, "file-text");
        assert!(!item.is_external_link());
    }

    /// `external` without an href renders nothing external -- guarding
    /// against a `target="_blank"` on a non-link, which would be a lie in
    /// the accessible name.
    #[test]
    fn meta_item_external_requires_a_link() {
        assert!(
            !RecordMetaItem::new("a", "A", "1")
                .external()
                .is_external_link()
        );
        assert!(
            RecordMetaItem::new("a", "A", "1")
                .link("/a")
                .external()
                .is_external_link()
        );
    }

    #[test]
    fn meta_link_accessible_name_folds_in_the_term_and_the_new_tab_warning() {
        let texts = texts();
        let internal = RecordMetaItem::new("matter", "Matter", "MAT-1023").link("/matters/1023");
        assert_eq!(internal.link_accessible_name(&texts), "Matter: MAT-1023");

        let external = internal.clone().external();
        assert_eq!(
            external.link_accessible_name(&texts),
            "Matter: MAT-1023, opens in a new tab"
        );
        // WCAG 2.5.3: the visible text must remain a substring of the name.
        assert!(external.link_accessible_name(&texts).contains("MAT-1023"));
    }

    // -- status ------------------------------------------------------------

    #[test]
    fn status_defaults_to_neutral_with_no_detail() {
        let status = RecordStatus::new("Active");
        assert_eq!(status.tone, RecordStatusTone::Neutral);
        assert!(status.detail.is_empty());
    }

    #[test]
    fn status_accessible_name_prefixes_the_role_and_appends_the_detail() {
        let texts = texts();
        let plain = RecordStatus::new("Active").tone(RecordStatusTone::Success);
        assert_eq!(plain.accessible_name(&texts), "Status: Active");

        let detailed = plain.clone().detail("Renewed 12 Aug");
        assert_eq!(
            detailed.accessible_name(&texts),
            "Status: Active, Renewed 12 Aug"
        );
    }

    #[test]
    fn status_accessible_name_follows_localized_texts() {
        let mut localized = texts();
        localized.status_label = "Etat".to_owned();
        assert_eq!(
            RecordStatus::new("Actif").accessible_name(&localized),
            "Etat: Actif"
        );
    }

    // -- badges ------------------------------------------------------------

    #[test]
    fn secondary_badge_defaults_to_neutral_tone() {
        let badge = RecordBadge::new("vip", "VIP");
        assert_eq!(badge.tone, RecordStatusTone::Neutral);
        assert_eq!(badge.label, "VIP");
        assert_eq!(
            RecordBadge::new("vip", "VIP")
                .tone(RecordStatusTone::Info)
                .tone,
            RecordStatusTone::Info
        );
    }

    // -- quick actions -----------------------------------------------------

    #[test]
    fn quick_action_state_defaults_to_ready_and_is_the_only_actionable_state() {
        assert_eq!(
            RecordQuickActionState::default(),
            RecordQuickActionState::Ready
        );
        assert!(RecordQuickActionState::Ready.is_actionable());
        assert!(!RecordQuickActionState::Pending.is_actionable());
        assert!(!RecordQuickActionState::Disabled("nope".into()).is_actionable());
    }

    #[test]
    fn quick_action_state_markers_are_distinct_and_reason_is_only_on_disabled() {
        assert_eq!(RecordQuickActionState::Ready.as_str(), "ready");
        assert_eq!(RecordQuickActionState::Pending.as_str(), "pending");
        assert_eq!(
            RecordQuickActionState::Disabled("locked".into()).as_str(),
            "disabled"
        );
        assert_eq!(RecordQuickActionState::Ready.reason(), None);
        assert_eq!(RecordQuickActionState::Pending.reason(), None);
        assert_eq!(
            RecordQuickActionState::Disabled("locked".into()).reason(),
            Some("locked")
        );
    }

    #[test]
    fn quick_action_builders_set_exactly_what_they_name() {
        let action = RecordQuickAction::new("call", "phone", "Call account");
        assert_eq!(action.state, RecordQuickActionState::Ready);
        assert!(action.href.is_none());
        assert!(action.feedback.is_none());

        let pending = action.clone().pending();
        assert_eq!(pending.state, RecordQuickActionState::Pending);

        let disabled = action.clone().disabled("Locked while a review is open");
        assert_eq!(
            disabled.state.reason(),
            Some("Locked while a review is open")
        );

        let with_feedback = action
            .clone()
            .feedback(RecordActionFeedback::new("Draft opened").tone(RecordStatusTone::Success));
        let feedback = with_feedback.feedback.expect("feedback set");
        assert_eq!(feedback.message, "Draft opened");
        assert_eq!(feedback.tone, RecordStatusTone::Success);
    }

    /// A glyph-only control has no visible text, so `accessible_name` is
    /// its ONLY name. Every state must produce a non-empty one that starts
    /// with the action's label.
    #[test]
    fn every_action_state_yields_a_non_empty_name_starting_with_the_label() {
        let texts = texts();
        let base = RecordQuickAction::new("archive", "trash", "Archive account");
        for action in [
            base.clone(),
            base.clone().pending(),
            base.clone().disabled("Locked while a review is open"),
            base.clone().link("/archive"),
            base.clone().link("/archive").external(),
        ] {
            let name = action.accessible_name(&texts);
            assert!(!name.is_empty());
            assert!(
                name.starts_with("Archive account"),
                "name must lead with the action label, got {name}"
            );
        }
    }

    #[test]
    fn action_accessible_name_carries_the_state_qualifier() {
        let texts = texts();
        let base = RecordQuickAction::new("archive", "trash", "Archive account");
        assert_eq!(base.accessible_name(&texts), "Archive account");
        assert_eq!(
            base.clone().pending().accessible_name(&texts),
            "Archive account (in progress)"
        );
        assert_eq!(
            base.clone()
                .disabled("Locked while a review is open")
                .accessible_name(&texts),
            "Archive account (Locked while a review is open)"
        );
        assert_eq!(
            base.clone()
                .link("/archive")
                .external()
                .accessible_name(&texts),
            "Archive account, opens in a new tab"
        );
    }

    #[test]
    fn action_accessible_name_follows_localized_texts() {
        let mut localized = texts();
        localized.pending = "en cours".to_owned();
        localized.external_link = "ouvre un nouvel onglet".to_owned();
        let action = RecordQuickAction::new("call", "phone", "Appeler");
        assert_eq!(
            action.clone().pending().accessible_name(&localized),
            "Appeler (en cours)"
        );
        assert_eq!(
            action.link("/x").external().accessible_name(&localized),
            "Appeler, ouvre un nouvel onglet"
        );
    }

    /// A non-Ready link must degrade to a button: an anchor cannot express
    /// "disabled" accessibly, and one left navigable would contradict its
    /// own accessible name.
    #[test]
    fn only_a_ready_link_renders_as_an_anchor() {
        let action = RecordQuickAction::new("open", "external-link", "Open").link("/x");
        assert!(action.renders_as_link());
        assert!(!action.clone().pending().renders_as_link());
        assert!(!action.clone().disabled("nope").renders_as_link());
        assert!(!RecordQuickAction::new("call", "phone", "Call").renders_as_link());
    }

    #[test]
    fn action_external_requires_a_link() {
        assert!(
            !RecordQuickAction::new("a", "phone", "A")
                .external()
                .is_external_link()
        );
        assert!(
            RecordQuickAction::new("a", "phone", "A")
                .link("/a")
                .external()
                .is_external_link()
        );
    }

    // -- tooltip edge containment (ldui-q73d) -------------------------------

    #[test]
    fn tooltip_half_width_grows_with_label_length_then_caps() {
        let short = estimated_tooltip_half_width(4);
        let longer = estimated_tooltip_half_width(20);
        assert!(short < longer);
        // daisyUI's own `max-width: 20rem` (320px) bounds the bubble, so an
        // absurdly long label must not push the half-width past 160px.
        let absurd = estimated_tooltip_half_width(1000);
        assert_eq!(absurd, 160.0);
    }

    #[test]
    fn tooltip_position_stays_top_with_room_on_both_sides() {
        // A 400px-wide row, trigger centred in the middle at x=200.
        assert_eq!(
            resolved_tooltip_position(0.0, 400.0, 184.0, 32.0, 64.0),
            TooltipPosition::Top
        );
    }

    /// The defect this pattern exists to fix: a trigger flush against the
    /// row's right edge flips to `Left` rather than spilling past it.
    #[test]
    fn tooltip_position_flips_left_when_the_right_edge_has_no_room() {
        // Row is 400px wide; trigger sits at its right edge (368-400).
        assert_eq!(
            resolved_tooltip_position(0.0, 400.0, 368.0, 32.0, 64.0),
            TooltipPosition::Left
        );
    }

    /// The mirror case: a trigger flush against the row's LEFT edge flips to
    /// `Right`. Proves the decision is symmetric geometry, not a "last
    /// action" special case that only ever produces `Left`.
    #[test]
    fn tooltip_position_flips_right_when_the_left_edge_has_no_room() {
        assert_eq!(
            resolved_tooltip_position(0.0, 400.0, 0.0, 32.0, 64.0),
            TooltipPosition::Right
        );
    }

    /// A row too narrow for the bubble on either side degrades to the
    /// default `Top` rather than picking an edge that also overflows --
    /// there is no placement that fully contains it, so this is not a
    /// regression, just the least-bad fallback.
    #[test]
    fn tooltip_position_falls_back_to_top_when_neither_edge_has_room() {
        assert_eq!(
            resolved_tooltip_position(0.0, 40.0, 4.0, 32.0, 64.0),
            TooltipPosition::Top
        );
    }

    /// Disabled styling must never use `btn-disabled` (which sets
    /// `pointer-events: none` and would hide the tooltip carrying the
    /// reason) nor an `opacity-*` utility (which the style audit rejects
    /// for contrast).
    #[test]
    fn disabled_action_classes_avoid_btn_disabled_and_opacity() {
        let class = quick_action_class(&RecordQuickActionState::Disabled("locked".into()));
        assert!(!class.contains("btn-disabled"));
        assert!(!class.contains("opacity-"));
        assert!(class.contains("text-base-content/75"));
        assert_eq!(quick_action_class(&RecordQuickActionState::Ready), "");
    }

    #[test]
    fn feedback_defaults_to_neutral_and_keeps_its_message() {
        let feedback = RecordActionFeedback::new("Draft opened");
        assert_eq!(feedback.tone, RecordStatusTone::Neutral);
        assert_eq!(feedback.message, "Draft opened");
    }

    #[test]
    fn has_text_is_false_for_empty_and_true_otherwise() {
        assert!(!has_text(""));
        assert!(has_text("Active"));
    }

    // -- source-level contracts -------------------------------------------

    /// The rendering half of the file only -- from the component signature
    /// to the test module. Narrowed at both ends deliberately: a prose
    /// mention of `PageHeader`'s own `<h1>` above it, or an assertion's own
    /// literal below it, would otherwise satisfy the very check it exists
    /// to make.
    fn component_source() -> &'static str {
        let source = include_str!("record_header.rs");
        let after_signature = source
            .split_once("pub fn RecordHeader(")
            .expect("RecordHeader component source")
            .1;
        after_signature
            .split_once("\n#[cfg(test)]")
            .map_or(after_signature, |(before, _)| before)
    }

    /// The row sits BELOW `PageHeader`'s `<h1>`; rendering another would
    /// give the page two top-level headings.
    #[test]
    fn record_header_never_renders_an_h1() {
        assert!(!component_source().contains("<h1"));
    }

    /// Long identity text must truncate rather than push into the
    /// status/actions edge, and the edge must never shrink to accommodate
    /// it. Both halves of that contract are class-level and invisible to a
    /// unit test of the typed model, so they are pinned at the source.
    #[test]
    fn identity_truncates_while_the_status_actions_edge_stays_fixed() {
        let source = component_source();
        assert!(source.contains("min-w-0 truncate font-semibold"));
        assert!(source.contains(r#"class="flex min-w-0 flex-1 items-center gap-3""#));
        assert!(
            source.contains(r#"class="flex shrink-0 flex-wrap items-center gap-2 lg:justify-end""#)
        );
    }

    /// Disabled actions must stay focusable (`aria-disabled`), never
    /// natively disabled -- otherwise the tooltip carrying the reason is
    /// unreachable by keyboard.
    #[test]
    fn actions_use_aria_disabled_rather_than_the_native_disabled_attribute() {
        let render = component_source()
            .split_once("fn render_action(")
            .expect("render_action source")
            .1;
        assert!(render.contains("attr:aria-disabled"));
        assert!(!render.contains("disabled=true"));
        assert!(!render.contains("disabled=disabled"));
    }

    /// Every quick action is wrapped in a tooltip whose tip is the same
    /// string as its `aria-label`, so the sighted-hover channel and the
    /// assistive channel cannot drift apart.
    #[test]
    fn action_tooltip_and_aria_label_share_one_string() {
        let render = component_source()
            .split_once("fn render_action(")
            .expect("render_action source")
            .1;
        assert!(render.contains("let name = action.accessible_name(texts);"));
        assert!(render.contains("attr:aria-label=name.clone()"));
        assert!(render.contains("node_ref=tooltip_ref"));
        assert!(render.contains("tip=name"));
        assert!(render.contains("position=tooltip_position"));
    }

    /// The rightmost (or any edge-adjacent) action's tooltip must be placed
    /// from measured row/trigger geometry, never from the action's index in
    /// the list -- a hardcoded "last action" special case would silently
    /// stop covering the edge once a consumer adds or reorders actions.
    #[test]
    fn action_tooltip_placement_is_measured_not_indexed() {
        let render = component_source()
            .split_once("fn render_action(")
            .expect("render_action source")
            .1;
        assert!(!render.contains("is_last"));
        assert!(!render.contains(".last()"));
        assert!(render.contains(
            "measure_and_set_tooltip_position(row, tooltip_ref, tooltip_position, label_len)"
        ));
    }

    /// The feedback region must be a polite live region, or a keyed outcome
    /// on a glyph-only control is announced to nobody.
    #[test]
    fn feedback_region_is_a_polite_live_region() {
        let source = component_source();
        assert!(source.contains(r#"aria-live="polite""#));
        assert!(source.contains(r#"data-record-header-feedback="true""#));
    }
}
