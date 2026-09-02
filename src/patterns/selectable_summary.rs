//! Opinionated single-selection group of compact count cards -- the
//! diagnostic check selector that Office Data Quality and its siblings
//! otherwise rebuild out of raw `<button>`-as-card markup (`ldui-l5cw`).
//!
//! The library already ships [`Pressable`](crate::components::Pressable)
//! (an unstyled action primitive), [`Card`](crate::components::Card) and
//! [`KpiCard`](super::KpiCard) (a *read-only* stat card). None of them owns
//! the thing a diagnostic page actually needs: fourteen compact count cards
//! where exactly one is chosen, the chosen one is announced as chosen, the
//! whole set is one tab stop, an unmeasured check is distinguishable from a
//! check that measured zero, and the status channel survives forced-colors
//! mode. Every consumer that hand-rolls that gets a different subset of it
//! right.
//!
//! This is a *selection* pattern, not a page generator: it fetches nothing,
//! owns no selection state, names no domain check, and renders no toolbar,
//! heading or detail panel. The caller owns accepted truth as a
//! [`Signal<Option<String>>`], the group emits a proposal through
//! `on_select`, and nothing changes optimistically -- the same controlled
//! idiom as `EntityTableSelectionProposal`.
//!
//! ## Why `role="radiogroup"` and not `aria-pressed`
//!
//! Single selection in a group of buttons has two legitimate encodings and
//! they imply *different* keyboard contracts, so the choice has to be made
//! once and implemented completely.
//!
//! - `aria-pressed` toggle buttons: every card is its own tab stop, and the
//!   "exactly one" relationship is never expressed -- a screen reader
//!   announces fourteen independent toggles that happen to be off.
//! - `role="radiogroup"` / `role="radio"`: the group is named, mutual
//!   exclusion is explicit, and the whole set costs **one** tab stop with
//!   arrow keys moving between options.
//!
//! Fourteen cards decides it. Fourteen tab stops in front of the table a
//! diagnostic page exists to show is a real cost to a keyboard user, and
//! fourteen unrelated toggles is a lie about the widget. This module
//! implements the radio contract in full ([`SelectableSummaryGroup`] lists
//! it); half a radiogroup would be worse than a correct set of toggles.

use crate::components::{Icon, IconSize, Pressable};
use crate::merge_classes;
use leptos::{ev, html::Div, prelude::*};
use wasm_bindgen::JsCast;

/// Reactive framework-owned copy for the group's own generated text.
///
/// Caller-supplied text (card labels and descriptions, the group's
/// accessible name) is **not** covered here: localize it by rebuilding the
/// [`SelectableSummaryItem`] list for the active locale, the same posture
/// as [`KpiItem`](super::KpiItem) and [`RecordStatus`](super::RecordStatus).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectableSummaryTexts {
    /// Shown in the count slot -- and spoken as the card's value -- when a
    /// check produced **no measurement at all**. This is the string that
    /// keeps "we could not measure this" from being rendered, and read
    /// aloud, as the number zero. It doubles as the spoken status word for
    /// [`SelectableSummaryStatus::Unavailable`], so the accessible name
    /// never says it twice.
    pub unavailable: String,
    /// Spoken status word for [`SelectableSummaryStatus::Clean`].
    pub clean: String,
    /// Spoken status word for [`SelectableSummaryStatus::Warning`].
    pub warning: String,
    /// Spoken status word for [`SelectableSummaryStatus::Error`].
    pub error: String,
}

impl Default for SelectableSummaryTexts {
    fn default() -> Self {
        Self {
            unavailable: "Not measured".to_owned(),
            clean: "clean".to_owned(),
            warning: "needs attention".to_owned(),
            error: "failing".to_owned(),
        }
    }
}

/// Semantic status of one summary card.
///
/// **Status is never colour-only.** Each non-neutral status renders a
/// distinct glyph *shape* (see [`Self::glyph`]) and contributes a spoken
/// word to the card's accessible name (see [`Self::word`]); the accent
/// colour is the third, removable channel. Greyscale, colour-blind and
/// forced-colors viewing all keep the meaning -- the same posture as
/// [`RecordStatusTone`](super::RecordStatusTone).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectableSummaryStatus {
    /// No semantic emphasis (default). Carries the house accent colour, no
    /// glyph, and no spoken status word.
    #[default]
    Neutral,
    /// The check ran and found nothing wrong.
    Clean,
    /// The check ran and found something worth attention.
    Warning,
    /// The check ran and found a failure.
    Error,
    /// The check could **not** be run or produced no measurement. Not the
    /// same as a measured zero, and never rendered as one.
    Unavailable,
}

impl SelectableSummaryStatus {
    /// Stable runtime marker, emitted as `data-selectable-summary-status`
    /// so a test can assert on status without reading colour.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Clean => "clean",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Unavailable => "unavailable",
        }
    }

    /// Lucide glyph paired with this status -- a *shape* channel
    /// independent of colour. `Neutral` has none deliberately: the sprite's
    /// honest neutral glyph is blank, and an invisible glyph is worse than
    /// none. A neutral card still carries its label and count, which are
    /// the channels that always exist.
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Clean => "circle-check",
            Self::Warning => "triangle-alert",
            Self::Error => "circle-alert",
            Self::Unavailable => "help-circle",
        }
    }

    /// Spoken status word, folded into the card's accessible name.
    /// `Neutral` contributes none.
    pub fn word(self, texts: &SelectableSummaryTexts) -> Option<&str> {
        match self {
            Self::Neutral => None,
            Self::Clean => Some(&texts.clean),
            Self::Warning => Some(&texts.warning),
            Self::Error => Some(&texts.error),
            Self::Unavailable => Some(&texts.unavailable),
        }
    }

    /// Left accent edge background class.
    ///
    /// `Neutral` paints the house dark blue (`--color-status-blue`), not
    /// nothing -- every card carries an accent edge and a status is what
    /// OVERRIDES it, exactly as [`KpiStatus`](super::KpiStatus) documents.
    /// Two card families in one library disagreeing about where the accent
    /// lives is the drift this pattern exists to prevent, so the edge is on
    /// the LEFT and always laid out here too.
    fn accent_bg_class(self) -> &'static str {
        match self {
            Self::Neutral => "bg-status-blue",
            Self::Clean => "bg-success",
            Self::Warning => "bg-warning",
            Self::Error => "bg-error",
            Self::Unavailable => "bg-base-300",
        }
    }

    /// Count text colour class. Empty for `Neutral`.
    fn count_text_class(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Clean => "text-success",
            Self::Warning => "text-warning",
            Self::Error => "text-error",
            Self::Unavailable => "text-base-content/75",
        }
    }

    /// Glyph colour class, matching the accent edge. `Unavailable` stays on
    /// the muted foreground (`text-base-content/75`) rather than an
    /// `opacity-*` utility, which the style audit rejects for contrast.
    fn glyph_text_class(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Clean => "text-success",
            Self::Warning => "text-warning",
            Self::Error => "text-error",
            Self::Unavailable => "text-base-content/75",
        }
    }
}

/// One selectable summary card's content -- the group's typed item model.
///
/// Plain owned data, not a `Signal`-bearing struct: the whole `items` list
/// is itself reactive on [`SelectableSummaryGroup`], the same posture as
/// [`KpiItem`](super::KpiItem) and
/// [`ActiveFilterChip`](super::ActiveFilterChip). Rebuilding the list --
/// for a refresh or a locale change -- is how a card updates.
///
/// There are two constructors and the split is the point:
/// [`Self::new`] takes a `u64`, so a measured result is always a number;
/// [`Self::unmeasured`] takes none, so "we could not measure this" cannot
/// be spelled as `0`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectableSummaryItem {
    /// Stable identity. It is the selection value, the list key, and the
    /// `data-selectable-summary-card` marker -- so selection survives a
    /// data refresh that reorders the list.
    pub id: String,
    /// Card label (always rendered). Caller-owned copy: localize by
    /// rebuilding the list.
    pub label: String,
    /// Measured count. `None` renders the unavailable presentation -- the
    /// localized [`SelectableSummaryTexts::unavailable`] placeholder, never
    /// a fabricated zero.
    pub count: Option<u64>,
    /// Presentational override for the rendered count, e.g. a
    /// locale-grouped `"12 483"`. Ignored entirely when `count` is `None`,
    /// so it can never invent a value for an unmeasured check.
    pub count_text: Option<String>,
    /// Optional supporting copy. Renders nothing when empty, and is exposed
    /// through `aria-describedby` rather than folded into the card's name.
    pub description: String,
    /// Semantic status.
    pub status: SelectableSummaryStatus,
    /// Whether the card can be selected at all. Independent of `status`:
    /// an unmeasured check is usually still worth opening (to see *why* it
    /// is unmeasured), so `unmeasured` does not imply `disabled`.
    pub disabled: bool,
}

impl SelectableSummaryItem {
    /// Creates a measured card. A count of `0` is a real, measured zero.
    pub fn new(id: impl Into<String>, label: impl Into<String>, count: u64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: Some(count),
            count_text: None,
            description: String::new(),
            status: SelectableSummaryStatus::default(),
            disabled: false,
        }
    }

    /// Creates a card for a check that produced no measurement, defaulting
    /// its status to [`SelectableSummaryStatus::Unavailable`]. Override the
    /// status afterwards if the absence itself is, say, a warning.
    pub fn unmeasured(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: None,
            count_text: None,
            description: String::new(),
            status: SelectableSummaryStatus::Unavailable,
            disabled: false,
        }
    }

    /// Sets the presentational count override. See
    /// [`Self::count_text`](SelectableSummaryItem::count_text).
    pub fn count_text(mut self, text: impl Into<String>) -> Self {
        self.count_text = Some(text.into());
        self
    }

    /// Sets the supporting description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the semantic status.
    pub fn status(mut self, status: SelectableSummaryStatus) -> Self {
        self.status = status;
        self
    }

    /// Marks the card unselectable.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Whether optional copy should render at all -- an empty string renders
/// nothing, not an empty line.
fn has_text(value: &str) -> bool {
    !value.is_empty()
}

/// Which arrow/Home/End key was pressed, resolved away from key names so
/// the navigation rule is a pure, testable function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Next,
    Previous,
    First,
    Last,
}

impl Step {
    /// Maps a `KeyboardEvent.key` value onto a navigation step. Both axes
    /// move: a wrapped grid has no single reading direction, and the APG
    /// radio-group contract is defined over the option ORDER, not over
    /// visual rows, so Down/Right agree and Up/Left agree.
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "ArrowRight" | "ArrowDown" => Some(Self::Next),
            "ArrowLeft" | "ArrowUp" => Some(Self::Previous),
            "Home" => Some(Self::First),
            "End" => Some(Self::Last),
            _ => None,
        }
    }
}

/// The id that owns the group's single tab stop.
///
/// The selected card when there is one and it is selectable, otherwise the
/// first selectable card -- the APG rule, which is what keeps a
/// fourteen-card group at one tab stop while still landing the user on
/// their current choice.
fn tab_stop_id(items: &[SelectableSummaryItem], selected: Option<&str>) -> Option<String> {
    if let Some(selected) = selected
        && let Some(item) = items
            .iter()
            .find(|item| item.id == selected && !item.disabled)
    {
        return Some(item.id.clone());
    }
    items
        .iter()
        .find(|item| !item.disabled)
        .map(|item| item.id.clone())
}

/// The id an arrow/Home/End press moves focus and selection to.
///
/// Disabled cards are skipped rather than focused-and-refused, and Next/
/// Previous wrap, both per the APG radio-group contract.
fn step_id(items: &[SelectableSummaryItem], current: &str, step: Step) -> Option<String> {
    let enabled: Vec<&SelectableSummaryItem> = items.iter().filter(|item| !item.disabled).collect();
    let len = enabled.len();
    if len == 0 {
        return None;
    }
    let index = match step {
        Step::First => 0,
        Step::Last => len - 1,
        Step::Next => (enabled.iter().position(|item| item.id == current)? + 1) % len,
        Step::Previous => (enabled.iter().position(|item| item.id == current)? + len - 1) % len,
    };
    Some(enabled[index].id.clone())
}

/// The rendered count -- the measured number, its presentational override,
/// or the localized unavailable placeholder. **Never `"0"` for an
/// unmeasured check.**
fn count_text(item: &SelectableSummaryItem, texts: &SelectableSummaryTexts) -> String {
    match item.count {
        None => texts.unavailable.clone(),
        Some(count) => item.count_text.clone().unwrap_or_else(|| count.to_string()),
    }
}

/// The card's accessible name: label, then the count (or the unavailable
/// placeholder), then the spoken status word.
///
/// The status word is suppressed when it would merely repeat the value --
/// which is exactly the `Unavailable` + no-count case, since both resolve
/// to [`SelectableSummaryTexts::unavailable`]. Selection is deliberately
/// absent from the name: `aria-checked` already carries it, and spelling it
/// into the name too would have a screen reader say it twice.
fn card_accessible_name(item: &SelectableSummaryItem, texts: &SelectableSummaryTexts) -> String {
    let value = count_text(item, texts);
    let mut name = format!("{}: {}", item.label, value);
    if let Some(word) = item.status.word(texts)
        && word != value
    {
        name.push_str(", ");
        name.push_str(word);
    }
    name
}

/// Suppresses the group's `aria-label` when a visible element already names
/// it -- an `aria-label` would override `aria-labelledby`'s target. Mirrors
/// [`modal_aria_label`](crate::components::modal_aria_label).
fn group_aria_label(label: String, has_labelled_by: bool) -> Option<String> {
    if has_labelled_by { None } else { Some(label) }
}

/// Responsive grid classes for the group.
///
/// Container query steps (`@sm`/`@lg`/`@3xl`/`@5xl`), never viewport ones:
/// the column count must follow the GROUP's own width. Fourteen compact
/// cards in a constrained column is precisely the situation where viewport
/// breakpoints render unreadable cards (`ldui-tnyq`). Two columns at the
/// narrowest width -- never a single full-bleed column, which reads as a
/// list rather than a grid -- growing to seven, so fourteen cards land as
/// two even rows.
fn group_grid_class() -> &'static str {
    "grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @3xl:grid-cols-5 @5xl:grid-cols-7 gap-3"
}

/// Card body padding/gap: internal spacing stays at or below the grid gap
/// (`p-3` <= `gap-3`) so cards never read as one group with their
/// neighbours.
fn card_body_class() -> &'static str {
    // `min-w-0` matters: the body is a flex item beside the accent edge, and
    // without it the default `min-width: auto` lets a long unbroken count
    // push the card wider than its grid track.
    "flex min-w-0 flex-1 flex-col gap-1 p-3 text-left"
}

/// The card's outer button classes.
///
/// Selection is carried by a ring (a shape that is simply absent when
/// unselected, not a hue swap) plus a primary border. Rings are box-shadows
/// and forced-colors mode removes them, so the selected card additionally
/// claims the system `Highlight` border colour while an unselected one
/// claims `CanvasText` -- two distinct system colours, so selection
/// survives in a palette the author does not control. Border WIDTH is
/// identical in both states, so selecting a card never shifts the grid.
///
/// Disabled adds a dashed border -- again a shape, not a hue.
fn card_class(selected: bool, disabled: bool) -> String {
    let selection = if selected {
        "border-primary ring-2 ring-primary bg-base-200 forced-colors:border-[Highlight]"
    } else {
        "border-base-300 bg-base-100 forced-colors:border-[CanvasText]"
    };
    let interactivity = if disabled {
        "border-dashed cursor-not-allowed"
    } else {
        "border-solid"
    };
    format!(
        "flex h-full w-full min-w-0 overflow-hidden rounded-box border ld-card-depth {selection} {interactivity}"
    )
}

/// Label classes: a bounded two-line clamp, never single-line truncation,
/// with the two line boxes reserved up front (`min-h-8` is exactly two
/// `ld-text-small` line heights) so a one-line label and a two-line label
/// leave the same box and every card's count starts at the same vertical
/// offset. Identical reasoning -- and identical numbers -- to
/// [`KpiCard`](super::KpiCard)'s label, deliberately, so the two card
/// families line up when they share a page.
fn card_label_class() -> &'static str {
    "ld-text-small font-semibold uppercase tracking-wide text-base-content/75 line-clamp-2 break-words min-h-8"
}

/// Count classes. An unmeasured card renders italic and muted, so the
/// placeholder is visibly not a number.
fn card_count_class(status: SelectableSummaryStatus, measured: bool) -> String {
    if measured {
        format!(
            "ld-text-title font-semibold tabular-nums break-words {}",
            status.count_text_class()
        )
    } else {
        "ld-text-title font-semibold break-words italic text-base-content/75".to_owned()
    }
}

/// Reads the `data-selectable-summary-card` id off whichever card an event
/// came from. Attribute lookup, never a positional query: a layout change
/// must not silently start describing a different element.
fn event_card_id(target: Option<web_sys::EventTarget>) -> Option<String> {
    let element = target?.dyn_into::<web_sys::Element>().ok()?;
    let card = element
        .closest("[data-selectable-summary-card]")
        .ok()
        .flatten()?;
    card.get_attribute("data-selectable-summary-card")
}

/// Moves DOM focus to the card carrying `id`, searched within the group the
/// event came from (so two groups on one page cannot steal each other's
/// focus) and matched by attribute value rather than by position.
fn focus_card(target: Option<web_sys::EventTarget>, id: &str) {
    let Some(element) = target.and_then(|target| target.dyn_into::<web_sys::Element>().ok()) else {
        return;
    };
    let Ok(Some(group)) = element.closest("[data-selectable-summary-group]") else {
        return;
    };
    let Ok(cards) = group.query_selector_all("[data-selectable-summary-card]") else {
        return;
    };
    for index in 0..cards.length() {
        let Some(node) = cards.item(index) else {
            continue;
        };
        let Ok(card) = node.dyn_into::<web_sys::HtmlElement>() else {
            continue;
        };
        if card
            .get_attribute("data-selectable-summary-card")
            .as_deref()
            == Some(id)
        {
            let _ = card.focus();
            return;
        }
    }
}

/// One compact, selectable count card.
///
/// **Must be rendered inside a `role="radiogroup"` container** -- normally
/// [`SelectableSummaryGroup`], which also owns the roving tab stop and the
/// arrow-key contract. A lone `role="radio"` is invalid ARIA, so reach for
/// this directly only when you are supplying the radiogroup and its
/// keyboard handling yourself.
///
/// Selection is **controlled**: `selected` is read, never written, and the
/// card only ever emits `on_select` with its own id. It holds no selection
/// state of its own, so it cannot diverge from the caller's truth.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{
///     SelectableSummaryCard, SelectableSummaryItem, SelectableSummaryStatus,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let (selected, set_selected) = signal(Some("duplicates".to_string()));
///     let item = SelectableSummaryItem::new("duplicates", "Duplicates", 12)
///         .status(SelectableSummaryStatus::Warning);
///     view! {
///         <div role="radiogroup" aria-label="Checks">
///             <SelectableSummaryCard
///                 item=item
///                 selected=Signal::derive(move || {
///                     selected.get().as_deref() == Some("duplicates")
///                 })
///                 on_select=Callback::new(move |id| set_selected.set(Some(id)))
///             />
///         </div>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex h-full w-full min-w-0 overflow-hidden rounded-box border");
///
/// `ld-card-depth` is deliberately NOT listed above: it is an authored rule
/// emitted into `styles/tokens.css` by `cargo xtask gen-tokens`, not a
/// Tailwind utility, so `@source inline(...)` cannot generate it (ldui-fg2h).
/// A consumer gets it by importing that stylesheet.
/// @source inline("border-solid border-dashed cursor-not-allowed");
/// @source inline("border-base-300 border-primary ring-2 ring-primary bg-base-100 bg-base-200");
/// @source inline("forced-colors:border-[CanvasText] forced-colors:border-[Highlight]");
/// @source inline("w-(--border-width-accent) shrink-0 self-stretch forced-colors:bg-[CanvasText]");
/// @source inline("bg-status-blue bg-success bg-warning bg-error bg-base-300");
/// @source inline("flex min-w-0 flex-1 flex-col items-center gap-1 p-3 text-left shrink-0");
/// @source inline("line-clamp-2 min-h-8 break-words tabular-nums italic");
/// @source inline("font-semibold uppercase tracking-wide sr-only");
/// @source inline("text-base-content/75 text-success text-warning text-error");
/// ```
///
/// The `ld-text-*` steps are NOT listed above on purpose: they are authored
/// rules emitted into `styles/tokens.css`, not Tailwind utilities, so
/// `@source inline(...)` cannot generate them and listing them would imply
/// the ramp was handled when it was not (`ldui-h7tw`, `ldui-fg2h`).
/// Consumers get them by importing that stylesheet.
///
/// ## Node References
/// - `node_ref` - References the `<button>` element ([HTMLButtonElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement))
#[component]
pub fn SelectableSummaryCard(
    /// The card to render. Reactive: rebuild it to refresh the count,
    /// description or status.
    #[prop(into)]
    item: Signal<SelectableSummaryItem>,

    /// Whether this card is the group's accepted selection. Read-only --
    /// the caller owns it.
    #[prop(optional, into)]
    selected: Signal<bool>,

    /// Whether this card is the group's single tab stop (`tabindex="0"`).
    /// Defaults to `true`, which is correct for a card used on its own;
    /// [`SelectableSummaryGroup`] drives it per the roving-tabindex rule.
    #[prop(optional, into, default = Signal::stored(true))]
    tab_stop: Signal<bool>,

    /// Emitted with this card's id when the user selects it. Fires on every
    /// activation, including re-selecting the already-selected card, so a
    /// consumer can treat a repeat press as a refresh request; ignore it if
    /// that is not wanted.
    #[prop(optional)]
    on_select: Option<Callback<String>>,

    /// Reactive framework-owned copy. See [`SelectableSummaryTexts`].
    #[prop(optional, into, default = Signal::stored(SelectableSummaryTexts::default()))]
    texts: Signal<SelectableSummaryTexts>,

    /// Additional CSS classes for the card button.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the `<button>` element.
    #[prop(optional)]
    node_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    let description_id = move || {
        item.with(|item| {
            has_text(&item.description)
                .then(|| format!("selectable-summary-{}-description", item.id))
        })
    };

    let accessible_name =
        move || texts.with(|texts| item.with(|item| card_accessible_name(item, texts)));

    let click_select = Callback::new(move |_: ev::MouseEvent| {
        if let Some(on_select) = on_select {
            on_select.run(item.with(|item| item.id.clone()));
        }
    });

    view! {
        <Pressable
            node_ref=node_ref
            disabled=Signal::derive(move || item.with(|item| item.disabled))
            on_click=click_select
            class=Signal::derive(move || {
                merge_classes!(
                    card_class(selected.get(), item.with(|item| item.disabled)),
                    class
                )
                    .to_class()
            })
            attr:role="radio"
            attr:aria-checked=move || if selected.get() { "true" } else { "false" }
            attr:tabindex=move || if tab_stop.get() { "0" } else { "-1" }
            attr:aria-label=accessible_name
            attr:aria-describedby=description_id
            attr:data-selectable-summary-card=move || item.with(|item| item.id.clone())
            attr:data-selectable-summary-status=move || {
                item.with(|item| item.status.as_str())
            }
            attr:data-selectable-summary-measured=move || {
                if item.with(|item| item.count.is_some()) { "true" } else { "false" }
            }
        >
            // Always laid out, and merely recoloured by status -- rendering
            // the edge conditionally would inset the body on status cards
            // and not on neutral ones, so a group mixing the two would have
            // two different text alignments. `forced-colors:bg-[CanvasText]`
            // keeps it a STRUCTURAL edge when the author's background colour
            // is thrown away.
            <span
                class=move || {
                    format!(
                        "w-(--border-width-accent) shrink-0 self-stretch forced-colors:bg-[CanvasText] {}",
                        item.with(|item| item.status.accent_bg_class()),
                    )
                }
                aria-hidden="true"
            ></span>
            <span class=card_body_class()>
                <span class="flex items-center gap-1 min-w-0">
                    <span class=card_label_class()>
                        {move || item.with(|item| item.label.clone())}
                    </span>
                    {move || {
                        let (glyph, color) = item
                            .with(|item| (item.status.glyph(), item.status.glyph_text_class()));
                        (!glyph.is_empty())
                            .then(|| {
                                view! {
                                    <span aria-hidden="true" class="inline-flex shrink-0">
                                        <Icon name=glyph size=IconSize::XSmall color=color />
                                    </span>
                                }
                            })
                    }}
                </span>
                <span
                    class=move || {
                        item.with(|item| card_count_class(item.status, item.count.is_some()))
                    }
                    data-selectable-summary-count="true"
                >
                    {move || texts.with(|texts| item.with(|item| count_text(item, texts)))}
                </span>
                {move || {
                    let description = item.with(|item| item.description.clone());
                    has_text(&description)
                        .then(|| {
                            view! {
                                <span
                                    id=description_id
                                    class="ld-text-small text-base-content/75 break-words line-clamp-2"
                                >
                                    {description}
                                </span>
                            }
                        })
                }}
            </span>
        </Pressable>
    }
}

/// Single-selection group of compact [`SelectableSummaryCard`]s -- the
/// pattern this module exists for.
///
/// Owns the accessible group name, the `role="radiogroup"` semantics, the
/// container-query grid, equal card geometry, and the full keyboard
/// contract. It owns **no selection state**: `selected` is the caller's
/// accepted truth and `on_select` is a proposal.
///
/// ## Keyboard contract (APG radio group, implemented in full)
///
/// - <kbd>Tab</kbd> enters the group at **one** stop -- the selected card,
///   or the first selectable card when nothing is selected -- and the next
///   <kbd>Tab</kbd> leaves the group entirely.
/// - <kbd>ArrowRight</kbd>/<kbd>ArrowDown</kbd> move focus to the next
///   selectable card and select it; <kbd>ArrowLeft</kbd>/<kbd>ArrowUp</kbd>
///   move to the previous and select it. Both wrap.
/// - <kbd>Home</kbd> selects the first selectable card, <kbd>End</kbd> the
///   last.
/// - <kbd>Space</kbd> and <kbd>Enter</kbd> select the focused card (native
///   `<button>` activation).
/// - Disabled cards are skipped by every one of the above rather than
///   focused and refused.
///
/// Arrow keys move focus **immediately** and emit the proposal; because
/// selection is controlled, a caller that declines the proposal leaves
/// `aria-checked` where it was while focus has moved. That is the honest
/// controlled behaviour, not a bug -- accept the proposal to keep them
/// together.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{
///     SelectableSummaryGroup, SelectableSummaryItem, SelectableSummaryStatus,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let (selected, set_selected) = signal(Some("duplicates".to_string()));
///     let items = Signal::derive(|| {
///         vec![
///             SelectableSummaryItem::new("duplicates", "Duplicate records", 12)
///                 .status(SelectableSummaryStatus::Warning)
///                 .description("Same identifier on more than one row"),
///             SelectableSummaryItem::new("orphans", "Orphaned rows", 0)
///                 .status(SelectableSummaryStatus::Clean),
///             SelectableSummaryItem::unmeasured("freshness", "Feed freshness"),
///         ]
///     });
///     view! {
///         <SelectableSummaryGroup
///             label="Diagnostic checks"
///             items=items
///             selected=selected
///             on_select=Callback::new(move |id| set_selected.set(Some(id)))
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("@container w-full grid grid-cols-2 @sm:grid-cols-3 @lg:grid-cols-4 @3xl:grid-cols-5 @5xl:grid-cols-7 gap-3");
/// ```
/// See [`SelectableSummaryCard`] for the per-card classes.
///
/// ## Node References
/// - `node_ref` - References the grid `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn SelectableSummaryGroup(
    /// Accessible name for the group (`aria-label`) -- e.g.
    /// `"Diagnostic checks"`. Required, because a radiogroup without a name
    /// announces as an anonymous set of radios. Suppressed when
    /// `labelled_by` names a visible element instead.
    #[prop(into)]
    label: Signal<String>,

    /// Id of a visible element that names the group (`aria-labelledby`) --
    /// prefer this when a heading already says what the group is.
    #[prop(optional, into)]
    labelled_by: MaybeProp<String>,

    /// The cards to render, in order. Rebuild this list to refresh counts,
    /// switch locale, or add/remove checks. Order is the keyboard order.
    #[prop(into)]
    items: Signal<Vec<SelectableSummaryItem>>,

    /// The caller's accepted selection. `None` selects nothing and puts the
    /// tab stop on the first selectable card.
    #[prop(optional, into)]
    selected: Signal<Option<String>>,

    /// Emitted with the proposed card id on pointer or keyboard selection.
    /// The group never updates `selected` itself.
    #[prop(optional)]
    on_select: Option<Callback<String>>,

    /// Reactive framework-owned copy, forwarded to every card. See
    /// [`SelectableSummaryTexts`].
    #[prop(optional, into, default = Signal::stored(SelectableSummaryTexts::default()))]
    texts: Signal<SelectableSummaryTexts>,

    /// Additional CSS classes for the grid.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the grid `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    // One relay so the cards take a plain `Callback` while the group's own
    // prop stays optional. It forwards the proposal untouched -- the group
    // never decides anything about selection.
    let relay = Callback::new(move |id: String| {
        if let Some(on_select) = on_select {
            on_select.run(id);
        }
    });

    let handle_keydown = move |ev: ev::KeyboardEvent| {
        let Some(step) = Step::from_key(&ev.key()) else {
            return;
        };
        let Some(current) = event_card_id(ev.target()) else {
            return;
        };
        let Some(next) = items.with(|items| step_id(items, &current, step)) else {
            return;
        };
        // Only now: an unhandled key must keep its default (a page scroll
        // outside the group, a Tab out of it).
        ev.prevent_default();
        focus_card(ev.target(), &next);
        if let Some(on_select) = on_select {
            on_select.run(next);
        }
    };

    view! {
        // Structural container only. An element cannot answer its OWN
        // container query, so the `@sm`/`@lg`/`@3xl`/`@5xl` steps on the
        // grid below need a container ancestor to measure (ldui-tnyq). It
        // carries no spacing of its own.
        <div class="@container w-full">
            <div
                node_ref=node_ref
                role="radiogroup"
                aria-label=move || group_aria_label(label.get(), labelled_by.get().is_some())
                aria-labelledby=move || labelled_by.get()
                data-selectable-summary-group="true"
                class=merge_classes!(group_grid_class(), class)
                on:keydown=handle_keydown
            >
                <For
                    each=move || items.get()
                    key=|item: &SelectableSummaryItem| item.id.clone()
                    children=move |item: SelectableSummaryItem| {
                        let id = item.id.clone();
                        let live_id = id.clone();
                        let fallback = item.clone();
                        // Keyed by id so the DOM node survives a data
                        // refresh -- which is what keeps focus and the
                        // roving tab stop where the user left them -- while
                        // the content stays live by re-reading the list.
                        let live = Signal::derive(move || {
                            items
                                .with(|items| {
                                    items.iter().find(|item| item.id == live_id).cloned()
                                })
                                .unwrap_or_else(|| fallback.clone())
                        });
                        let selected_id = id.clone();
                        let is_selected = Signal::derive(move || {
                            selected.get().as_deref() == Some(selected_id.as_str())
                        });
                        let tab_id = id.clone();
                        let is_tab_stop = Signal::derive(move || {
                            let current = selected.get();
                            let stop = items
                                .with(|items| tab_stop_id(items, current.as_deref()));
                            stop.as_deref() == Some(tab_id.as_str())
                        });
                        view! {
                            <SelectableSummaryCard
                                item=live
                                selected=is_selected
                                tab_stop=is_tab_stop
                                on_select=relay
                                texts=texts
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::lucide_to_sprite;

    fn sample() -> Vec<SelectableSummaryItem> {
        vec![
            SelectableSummaryItem::new("a", "Alpha", 3),
            SelectableSummaryItem::new("b", "Beta", 0).disabled(true),
            SelectableSummaryItem::new("c", "Gamma", 7),
            SelectableSummaryItem::unmeasured("d", "Delta"),
        ]
    }

    #[test]
    fn status_defaults_to_neutral() {
        assert_eq!(
            SelectableSummaryStatus::default(),
            SelectableSummaryStatus::Neutral
        );
    }

    #[test]
    fn status_markers_are_distinct_and_stable() {
        let markers = [
            SelectableSummaryStatus::Neutral.as_str(),
            SelectableSummaryStatus::Clean.as_str(),
            SelectableSummaryStatus::Warning.as_str(),
            SelectableSummaryStatus::Error.as_str(),
            SelectableSummaryStatus::Unavailable.as_str(),
        ];
        let distinct: std::collections::HashSet<&&str> = markers.iter().collect();
        assert_eq!(distinct.len(), markers.len());
        assert_eq!(SelectableSummaryStatus::Unavailable.as_str(), "unavailable");
    }

    /// The four semantic statuses must each carry a DISTINCT glyph, so
    /// status is legible as a shape and not only as a hue.
    #[test]
    fn semantic_statuses_carry_distinct_glyph_shapes() {
        let glyphs = [
            SelectableSummaryStatus::Clean.glyph(),
            SelectableSummaryStatus::Warning.glyph(),
            SelectableSummaryStatus::Error.glyph(),
            SelectableSummaryStatus::Unavailable.glyph(),
        ];
        assert!(glyphs.iter().all(|glyph| !glyph.is_empty()));
        let distinct: std::collections::HashSet<&&str> = glyphs.iter().collect();
        assert_eq!(distinct.len(), glyphs.len());
        assert_eq!(SelectableSummaryStatus::Neutral.glyph(), "");
    }

    /// Every glyph must actually resolve in the shipped sprite. An unknown
    /// Lucide name silently degrades to `blank`, which would leave status
    /// conveyed by colour alone -- the exact defect this mapping prevents.
    #[test]
    fn every_status_glyph_resolves_in_the_sprite() {
        for status in [
            SelectableSummaryStatus::Clean,
            SelectableSummaryStatus::Warning,
            SelectableSummaryStatus::Error,
            SelectableSummaryStatus::Unavailable,
        ] {
            assert_ne!(
                lucide_to_sprite(status.glyph()),
                "blank",
                "status {} has no sprite glyph",
                status.as_str()
            );
        }
    }

    /// Matches [`KpiStatus`](super::super::KpiStatus): the accent edge is
    /// universal and the house blue is the DEFAULT, not the absence of an
    /// accent. Two card families disagreeing here is the drift this pattern
    /// exists to prevent.
    #[test]
    fn neutral_paints_the_same_house_blue_accent_as_kpi_card() {
        // The literal is the contract: `KpiStatus::Neutral` maps to this
        // same class (its own unit test pins it), and the two must not
        // drift.
        assert_eq!(
            SelectableSummaryStatus::Neutral.accent_bg_class(),
            "bg-status-blue"
        );
        assert_eq!(
            SelectableSummaryStatus::Clean.accent_bg_class(),
            "bg-success"
        );
        assert_eq!(
            SelectableSummaryStatus::Unavailable.accent_bg_class(),
            "bg-base-300"
        );
    }

    #[test]
    fn new_is_measured_and_unmeasured_is_not() {
        let measured = SelectableSummaryItem::new("a", "Alpha", 0);
        assert_eq!(measured.count, Some(0));
        assert_eq!(measured.status, SelectableSummaryStatus::Neutral);

        let unmeasured = SelectableSummaryItem::unmeasured("d", "Delta");
        assert_eq!(unmeasured.count, None);
        assert_eq!(unmeasured.status, SelectableSummaryStatus::Unavailable);
    }

    #[test]
    fn builders_set_each_optional_field() {
        let item = SelectableSummaryItem::new("a", "Alpha", 12_483)
            .count_text("12 483")
            .description("Same identifier on more than one row")
            .status(SelectableSummaryStatus::Warning)
            .disabled(true);
        assert_eq!(item.count_text.as_deref(), Some("12 483"));
        assert_eq!(item.description, "Same identifier on more than one row");
        assert_eq!(item.status, SelectableSummaryStatus::Warning);
        assert!(item.disabled);
    }

    /// A measured zero renders the digit; an unmeasured check renders the
    /// localized placeholder. This is the whole point of the two
    /// constructors.
    #[test]
    fn zero_renders_a_digit_and_unmeasured_renders_the_placeholder() {
        let texts = SelectableSummaryTexts::default();
        assert_eq!(
            count_text(&SelectableSummaryItem::new("a", "Alpha", 0), &texts),
            "0"
        );
        assert_eq!(
            count_text(&SelectableSummaryItem::unmeasured("d", "Delta"), &texts),
            "Not measured"
        );
    }

    #[test]
    fn count_text_override_applies_only_to_a_measured_count() {
        let texts = SelectableSummaryTexts::default();
        let measured = SelectableSummaryItem::new("a", "Alpha", 12483).count_text("12 483");
        assert_eq!(count_text(&measured, &texts), "12 483");

        // An override can never invent a value for an unmeasured check.
        let unmeasured = SelectableSummaryItem::unmeasured("d", "Delta").count_text("12 483");
        assert_eq!(count_text(&unmeasured, &texts), "Not measured");
    }

    /// The screen-reader channel must distinguish "we measured zero" from
    /// "we could not measure this". A card announcing `0` when it means the
    /// latter is a lie.
    #[test]
    fn accessible_name_distinguishes_zero_from_unmeasured() {
        let texts = SelectableSummaryTexts::default();
        let zero = SelectableSummaryItem::new("orphans", "Orphaned rows", 0)
            .status(SelectableSummaryStatus::Clean);
        assert_eq!(
            card_accessible_name(&zero, &texts),
            "Orphaned rows: 0, clean"
        );

        let unmeasured = SelectableSummaryItem::unmeasured("freshness", "Feed freshness");
        assert_eq!(
            card_accessible_name(&unmeasured, &texts),
            "Feed freshness: Not measured"
        );
        assert!(!card_accessible_name(&unmeasured, &texts).contains('0'));
    }

    /// `Unavailable` + no count resolves both the value and the status word
    /// to the same string, so the name must not say it twice.
    #[test]
    fn accessible_name_never_repeats_the_unavailable_word() {
        let texts = SelectableSummaryTexts::default();
        let name = card_accessible_name(&SelectableSummaryItem::unmeasured("d", "Delta"), &texts);
        assert_eq!(name.matches("Not measured").count(), 1);
    }

    #[test]
    fn accessible_name_omits_a_status_word_for_neutral() {
        let texts = SelectableSummaryTexts::default();
        let name = card_accessible_name(&SelectableSummaryItem::new("a", "Alpha", 3), &texts);
        assert_eq!(name, "Alpha: 3");
    }

    #[test]
    fn accessible_name_follows_localized_texts() {
        let texts = SelectableSummaryTexts {
            unavailable: "Non mesure".to_owned(),
            clean: "propre".to_owned(),
            warning: "a verifier".to_owned(),
            error: "en echec".to_owned(),
        };
        let item = SelectableSummaryItem::new("a", "Doublons", 12)
            .status(SelectableSummaryStatus::Warning);
        assert_eq!(
            card_accessible_name(&item, &texts),
            "Doublons: 12, a verifier"
        );
        assert_eq!(
            card_accessible_name(&SelectableSummaryItem::unmeasured("d", "Fraicheur"), &texts),
            "Fraicheur: Non mesure"
        );
    }

    /// Selection is NOT folded into the accessible name -- `aria-checked`
    /// already carries it, and duplicating it makes a screen reader say it
    /// twice.
    #[test]
    fn accessible_name_never_mentions_selection() {
        let texts = SelectableSummaryTexts::default();
        let name = card_accessible_name(&SelectableSummaryItem::new("a", "Alpha", 3), &texts);
        assert!(!name.to_lowercase().contains("select"));
        assert!(!name.to_lowercase().contains("check"));
    }

    #[test]
    fn tab_stop_is_the_selected_card() {
        assert_eq!(tab_stop_id(&sample(), Some("c")).as_deref(), Some("c"));
    }

    #[test]
    fn tab_stop_falls_back_to_the_first_selectable_card() {
        assert_eq!(tab_stop_id(&sample(), None).as_deref(), Some("a"));
        // An id that is not in the list (a stale selection) must not leave
        // the group unreachable.
        assert_eq!(tab_stop_id(&sample(), Some("zzz")).as_deref(), Some("a"));
        // Nor may a disabled selection own the tab stop -- it cannot be
        // focused.
        assert_eq!(tab_stop_id(&sample(), Some("b")).as_deref(), Some("a"));
    }

    #[test]
    fn tab_stop_is_none_when_every_card_is_disabled() {
        let items = vec![
            SelectableSummaryItem::new("a", "Alpha", 1).disabled(true),
            SelectableSummaryItem::new("b", "Beta", 2).disabled(true),
        ];
        assert_eq!(tab_stop_id(&items, None), None);
    }

    #[test]
    fn arrow_keys_map_to_steps_on_both_axes() {
        assert_eq!(Step::from_key("ArrowRight"), Some(Step::Next));
        assert_eq!(Step::from_key("ArrowDown"), Some(Step::Next));
        assert_eq!(Step::from_key("ArrowLeft"), Some(Step::Previous));
        assert_eq!(Step::from_key("ArrowUp"), Some(Step::Previous));
        assert_eq!(Step::from_key("Home"), Some(Step::First));
        assert_eq!(Step::from_key("End"), Some(Step::Last));
        // Space and Enter are the native button activation path, not a
        // navigation step -- intercepting them here would double-fire.
        assert_eq!(Step::from_key(" "), None);
        assert_eq!(Step::from_key("Enter"), None);
        assert_eq!(Step::from_key("Tab"), None);
    }

    #[test]
    fn stepping_skips_disabled_cards() {
        let items = sample();
        // "b" is disabled, so Next from "a" lands on "c".
        assert_eq!(step_id(&items, "a", Step::Next).as_deref(), Some("c"));
        assert_eq!(step_id(&items, "c", Step::Previous).as_deref(), Some("a"));
    }

    #[test]
    fn stepping_wraps_at_both_ends() {
        let items = sample();
        assert_eq!(step_id(&items, "d", Step::Next).as_deref(), Some("a"));
        assert_eq!(step_id(&items, "a", Step::Previous).as_deref(), Some("d"));
    }

    #[test]
    fn home_and_end_reach_the_first_and_last_selectable_cards() {
        let items = sample();
        assert_eq!(step_id(&items, "c", Step::First).as_deref(), Some("a"));
        assert_eq!(step_id(&items, "a", Step::Last).as_deref(), Some("d"));
    }

    #[test]
    fn stepping_from_an_unknown_card_yields_nothing() {
        let items = sample();
        assert_eq!(step_id(&items, "zzz", Step::Next), None);
        assert_eq!(step_id(&[], "a", Step::First), None);
    }

    #[test]
    fn home_and_end_skip_disabled_edges() {
        let items = vec![
            SelectableSummaryItem::new("a", "Alpha", 1).disabled(true),
            SelectableSummaryItem::new("b", "Beta", 2),
            SelectableSummaryItem::new("c", "Gamma", 3),
            SelectableSummaryItem::new("d", "Delta", 4).disabled(true),
        ];
        assert_eq!(step_id(&items, "b", Step::First).as_deref(), Some("b"));
        assert_eq!(step_id(&items, "b", Step::Last).as_deref(), Some("c"));
    }

    /// Container query steps, not viewport ones: the column count must
    /// follow the GROUP's own width (`ldui-tnyq`). A plain `sm:`/`md:`/
    /// `xl:` here would ask how wide the WINDOW is, which is how an
    /// eight-card strip came to render 67px cards in a 648px column.
    #[test]
    fn grid_uses_container_queries_never_viewport_breakpoints() {
        let grid = group_grid_class();
        assert!(grid.contains("grid-cols-2"));
        assert!(grid.contains("@sm:grid-cols-3"));
        assert!(grid.contains("@lg:grid-cols-4"));
        assert!(grid.contains("@3xl:grid-cols-5"));
        assert!(grid.contains("@5xl:grid-cols-7"));
        for viewport in [" sm:", " md:", " lg:", " xl:"] {
            assert!(
                !format!(" {grid}").contains(viewport),
                "viewport breakpoint {viewport} in {grid}"
            );
        }
    }

    #[test]
    fn card_padding_never_exceeds_the_grid_gap() {
        // Internal <= external: a card's own padding must not exceed the
        // gap separating it from its neighbours, or the cards read as one
        // group.
        assert!(card_body_class().contains("p-3"));
        assert!(group_grid_class().contains("gap-3"));
    }

    /// Selection must not change the border WIDTH, or selecting a card
    /// would reflow the grid.
    #[test]
    fn selection_never_changes_the_border_width() {
        let selected = card_class(true, false);
        let unselected = card_class(false, false);
        assert!(selected.contains(" border "));
        assert!(unselected.contains(" border "));
        assert!(!selected.contains("border-2"));
        assert!(!unselected.contains("border-2"));
    }

    /// A ring is present-or-absent, not a hue swap; and because
    /// forced-colors mode drops box-shadows, the selected card claims a
    /// DIFFERENT system border colour so selection survives a palette the
    /// author does not control.
    #[test]
    fn selection_survives_forced_colors_mode() {
        let selected = card_class(true, false);
        let unselected = card_class(false, false);
        assert!(selected.contains("ring-2 ring-primary"));
        assert!(!unselected.contains("ring-"));
        assert!(selected.contains("forced-colors:border-[Highlight]"));
        assert!(unselected.contains("forced-colors:border-[CanvasText]"));
    }

    #[test]
    fn disabled_is_a_dashed_border_not_only_a_colour() {
        assert!(card_class(false, true).contains("border-dashed"));
        assert!(card_class(false, false).contains("border-solid"));
    }

    #[test]
    fn label_clamps_to_two_reserved_lines_instead_of_truncating() {
        let label = card_label_class();
        assert!(label.contains("line-clamp-2"));
        assert!(label.contains("min-h-8"));
        assert!(!label.contains("truncate"));
    }

    #[test]
    fn an_unmeasured_count_renders_italic_rather_than_as_a_number() {
        let unmeasured = card_count_class(SelectableSummaryStatus::Unavailable, false);
        assert!(unmeasured.contains("italic"));
        assert!(!unmeasured.contains("tabular-nums"));

        let measured = card_count_class(SelectableSummaryStatus::Warning, true);
        assert!(measured.contains("tabular-nums"));
        assert!(!measured.contains("italic"));
        assert!(measured.contains("text-warning"));
    }

    #[test]
    fn aria_label_is_suppressed_when_a_visible_element_names_the_group() {
        assert_eq!(
            group_aria_label("Diagnostic checks".to_owned(), false).as_deref(),
            Some("Diagnostic checks")
        );
        assert_eq!(group_aria_label("Diagnostic checks".to_owned(), true), None);
    }

    #[test]
    fn muted_text_never_uses_the_opacity_utility() {
        // Code lines only, above the tests: several doc comments NAME the
        // forbidden utility in order to explain why it is forbidden.
        let source = include_str!("selectable_summary.rs");
        let module = source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        for line in module
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
        {
            assert!(
                !line.contains("opacity-"),
                "muted copy must use text-base-content/NN, never an opacity utility: {line}"
            );
        }
    }

    /// The `ld-text-*` ramp steps are authored rules, not Tailwind
    /// utilities, so listing them in `@source inline(...)` would do nothing
    /// while implying the ramp was handled (`ldui-fg2h`).
    #[test]
    fn source_inline_never_lists_the_authored_type_ramp() {
        let source = include_str!("selectable_summary.rs");
        for line in source.lines() {
            if line.contains("@source inline(") {
                assert!(
                    !line.contains("ld-text-"),
                    "ld-text-* must never appear in @source inline: {line}"
                );
            }
        }
    }

    /// Selection is CONTROLLED: neither component may mint selection state
    /// of its own, or the rendered choice could diverge from the caller's
    /// accepted truth.
    #[test]
    fn the_components_hold_no_selection_state() {
        let source = include_str!("selectable_summary.rs");
        let module = source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        // Code lines only -- the rustdoc examples deliberately show a
        // CALLER minting its own selection signal, which is the point.
        let code: String = module
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for minting in ["RwSignal::new", "signal(", "create_signal"] {
            assert!(
                !code.contains(minting),
                "the pattern must not mint its own state ({minting})"
            );
        }
    }

    /// Keys the pattern does not handle must keep their default behaviour
    /// -- `prevent_default` has to sit AFTER the step resolves, or a Tab or
    /// a page scroll inside the group would be swallowed.
    #[test]
    fn prevent_default_runs_only_for_a_resolved_step() {
        let source = include_str!("selectable_summary.rs");
        let handler = source
            .split_once("let handle_keydown")
            .expect("keydown handler source")
            .1
            .split_once("\n    };")
            .expect("keydown handler end")
            .0;
        let step = handler.find("Step::from_key").expect("step resolution");
        let prevent = handler.find("prevent_default").expect("prevent_default");
        assert!(step < prevent, "prevent_default must follow key resolution");
    }

    /// Focus is located by the stable `data-selectable-summary-card`
    /// attribute, never by position: a positional query does not fail when
    /// layout changes, it silently describes a different element.
    #[test]
    fn focus_is_located_by_a_stable_attribute_never_by_position() {
        let source = include_str!("selectable_summary.rs");
        let module = source
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(module.contains("[data-selectable-summary-card]"));
        for positional in [":first-child", ":last-child", ":nth-child"] {
            assert!(!module.contains(positional));
        }
    }
}
