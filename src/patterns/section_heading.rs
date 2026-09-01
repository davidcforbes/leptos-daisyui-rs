//! Opinionated eyebrow/title/description composition for content beneath a
//! [`PageHeader`](super::PageHeader).

use leptos::{html::Div, prelude::*};

/// Heading element (and matching `.ld-text-*` weight) rendered by
/// [`SectionHeading`].
///
/// `PageHeader` owns the page's single `<h1>`, so a `SectionHeading` always
/// starts one level below it: `H2` is the default. Nesting a subsection
/// under an existing `SectionHeading` should step down to `H3`/`H4` rather
/// than skip back up to `H2` -- WCAG 1.3.1 forbids *skipped* heading levels
/// (jumping from `H2` straight to `H4`), which is why the level is an
/// explicit enum a caller must choose deliberately rather than a numeric
/// prop that would let a skip compile silently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    /// Section heading directly under a page's `<h1>`.
    #[default]
    H2,
    /// Subsection heading nested one level under an `H2` `SectionHeading`.
    H3,
    /// Subsection heading nested one level under an `H3` `SectionHeading`.
    H4,
}

impl HeadingLevel {
    /// The HTML tag name this level renders as.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
        }
    }

    /// The `.ld-text-*` type-ramp step used for this level's title text.
    ///
    /// `pub(crate)` so sibling patterns that render their own heading at a
    /// caller-chosen [`HeadingLevel`] -- [`RecordHeader`](super::RecordHeader)
    /// -- step down the same ramp instead of re-deriving a second mapping
    /// that could drift.
    pub(crate) const fn text_class(self) -> &'static str {
        match self {
            Self::H2 => "ld-text-title",
            Self::H3 => "ld-text-subtitle",
            Self::H4 => "ld-text-body",
        }
    }
}

/// Whether optional eyebrow/description copy should render at all. An empty
/// string renders nothing -- not an empty `<p>` -- so an unused slot never
/// contributes a `space-y` gap.
const fn has_text(value: &str) -> bool {
    !value.is_empty()
}

/// Where [`SectionHeading`]'s optional `status` content renders relative to
/// the title (`ldui-17rz`).
///
/// A `bool` (e.g. `status_trailing`) would name the mechanism but not the
/// intent, and would leave no room for a third placement later without a
/// breaking signature change. An enum names what the placement *means* and
/// is exhaustively matched, so adding a case is a compile-time prompt to
/// handle it everywhere rather than a silently-ignored flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionHeadingStatusPlacement {
    /// Status renders inline with the title, in the same flex row as
    /// [`HeadingLevel`]'s heading element -- the default, and the only
    /// placement that existed before `ldui-17rz`. Right for a compact badge
    /// that belongs immediately beside the heading text.
    #[default]
    Inline,
    /// Status renders as a separate sibling after the title/description
    /// group and before any `actions`, aligned to the trailing edge at
    /// desktop widths and wrapping onto its own row at compact widths.
    /// Right for an established, full-width section header whose
    /// noninteractive maturity/freshness text belongs at the far right
    /// instead of crowding the title.
    Trailing,
}

impl SectionHeadingStatusPlacement {
    /// Stable runtime marker, mirrored onto the outer wrapper via
    /// `data-section-heading-status-placement` for test assertions.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Trailing => "trailing",
        }
    }
}

/// Eyebrow/title/description heading for content beneath a `PageHeader`.
///
/// LDUI owns the semantic spacing, typography, heading hierarchy, and
/// responsive wrapping; the caller owns copy, status, and actions. Every
/// text prop is reactive (`Signal<String>`), so localized copy re-renders
/// when the active locale changes -- callers do not re-mount the component
/// to switch languages.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Badge, BadgeColor, Button};
/// use leptos_daisyui_rs::patterns::{HeadingLevel, SectionHeading};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <section aria-labelledby="roster-heading">
///             <SectionHeading
///                 id="roster-heading"
///                 eyebrow="STAFF"
///                 title="Team roster"
///                 description="Everyone currently assigned to this office."
///                 level=HeadingLevel::H2
///                 status=Box::new(|| view! {
///                     <Badge color=BadgeColor::Success>"Synced"</Badge>
///                 }.into_any())
///                 actions=Box::new(|| view! {
///                     <Button>"Add member"</Button>
///                 }.into_any())
///             />
///         </section>
///     }
/// }
/// ```
///
/// A noninteractive maturity/freshness note that belongs at the far right of
/// an established, full-width section header -- rather than crowding the
/// title -- uses `status_placement=SectionHeadingStatusPlacement::Trailing`.
/// The `status` slot itself is unchanged; only where it renders moves:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::patterns::{SectionHeading, SectionHeadingStatusPlacement};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <SectionHeading
///             title="Commitments"
///             status_placement=SectionHeadingStatusPlacement::Trailing
///             status=Box::new(|| view! {
///                 <span class="ld-text-caption text-base-content/75">
///                     "Provisional -- pending measure review"
///                 </span>
///             }.into_any())
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-start sm:justify-between");
/// @source inline("min-w-0 flex-1 space-y-1 flex flex-wrap items-center gap-2 sm:shrink-0");
/// @source inline("font-semibold tracking-tight tracking-wide uppercase text-base-content");
/// @source inline("text-base-content/75 max-w-3xl forced-colors:text-[CanvasText]");
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
pub fn SectionHeading(
    /// Section title text.
    #[prop(into)]
    title: Signal<String>,

    /// Optional small label rendered above the title (a "kicker"/eyebrow).
    /// Renders nothing when empty -- not an empty line.
    #[prop(optional, into)]
    eyebrow: Signal<String>,

    /// Optional supporting copy rendered below the title. Renders nothing
    /// when empty -- not an empty line.
    #[prop(optional, into)]
    description: Signal<String>,

    /// Heading element + weight. See [`HeadingLevel`] for why `H2` is the
    /// default and why it is an explicit enum rather than a numeric prop.
    #[prop(optional)]
    level: HeadingLevel,

    /// Stable id placed on the heading element itself, so a wrapping
    /// section element's `aria-labelledby` can point at this heading
    /// without duplicating its text. Omitted when empty.
    #[prop(optional, into)]
    id: &'static str,

    /// Optional status/freshness content, rendered inline with the title
    /// when there's room and wrapped onto its own line -- never squeezing
    /// the title -- when there isn't.
    #[prop(optional)]
    status: Option<Children>,

    /// Where `status` renders. Defaults to
    /// [`SectionHeadingStatusPlacement::Inline`], which reproduces exactly
    /// today's rendering -- this prop exists so a caller can opt into
    /// [`SectionHeadingStatusPlacement::Trailing`] without any change for
    /// existing callers. See [`SectionHeadingStatusPlacement`].
    #[prop(optional)]
    status_placement: SectionHeadingStatusPlacement,

    /// Optional action controls (buttons, menus) for this section. Wraps
    /// onto its own row at compact widths instead of shrinking the title
    /// or description. DOM order is always title/description, then
    /// `status` when [`SectionHeadingStatusPlacement::Trailing`], then
    /// `actions`.
    #[prop(optional)]
    actions: Option<Children>,

    /// Additional classes for the outer wrapper.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let status = status.map(|slot| slot());
    let actions = actions.map(|slot| slot());
    // Splitting on `status_placement` here -- rather than branching later in
    // the view -- means the `Inline` arm produces the exact same
    // `(Some(view), None)` shape this component always has, so the markup
    // below is untouched for every existing caller (none of whom pass
    // `status_placement` at all).
    let (inline_status, trailing_status) = match status_placement {
        SectionHeadingStatusPlacement::Inline => (status, None),
        SectionHeadingStatusPlacement::Trailing => (None, status),
    };
    let heading_id = (!id.is_empty()).then_some(id);
    let heading_class = format!(
        "{} font-semibold tracking-tight text-base-content forced-colors:text-[CanvasText]",
        level.text_class()
    );
    let heading = match level {
        HeadingLevel::H2 => view! {
            <h2 id=heading_id class=heading_class>{move || title.get()}</h2>
        }
        .into_any(),
        HeadingLevel::H3 => view! {
            <h3 id=heading_id class=heading_class>{move || title.get()}</h3>
        }
        .into_any(),
        HeadingLevel::H4 => view! {
            <h4 id=heading_id class=heading_class>{move || title.get()}</h4>
        }
        .into_any(),
    };

    view! {
        <div
            node_ref=node_ref
            class=format!(
                "flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-start sm:justify-between {class}"
            )
            data-section-heading="true"
            data-section-heading-level=level.as_str()
            data-section-heading-status-placement=status_placement.as_str()
        >
            <div class="min-w-0 flex-1 space-y-1">
                {move || {
                    let text = eyebrow.get();
                    has_text(&text).then(|| view! {
                        <p class="ld-text-small font-semibold uppercase tracking-wide text-base-content/75 forced-colors:text-[CanvasText]">
                            {text}
                        </p>
                    })
                }}
                <div class="flex flex-wrap items-center gap-2">
                    {heading}
                    {inline_status}
                </div>
                {move || {
                    let text = description.get();
                    has_text(&text).then(|| view! {
                        <p class="max-w-3xl ld-text-body text-base-content/75 forced-colors:text-[CanvasText]">
                            {text}
                        </p>
                    })
                }}
            </div>
            {trailing_status.map(|status| view! {
                <div class="flex flex-wrap items-center gap-2 sm:shrink-0" data-section-heading-status="true">
                    {status}
                </div>
            })}
            {actions.map(|actions| view! {
                <div class="flex flex-wrap items-center gap-2 sm:shrink-0" data-section-heading-actions="true">
                    {actions}
                </div>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_level_defaults_to_h2_one_below_page_headers_h1() {
        assert_eq!(HeadingLevel::default(), HeadingLevel::H2);
    }

    #[test]
    fn heading_level_as_str_maps_to_the_matching_tag_name() {
        assert_eq!(HeadingLevel::H2.as_str(), "h2");
        assert_eq!(HeadingLevel::H3.as_str(), "h3");
        assert_eq!(HeadingLevel::H4.as_str(), "h4");
    }

    #[test]
    fn heading_level_text_class_steps_down_the_type_ramp_with_the_tag() {
        assert_eq!(HeadingLevel::H2.text_class(), "ld-text-title");
        assert_eq!(HeadingLevel::H3.text_class(), "ld-text-subtitle");
        assert_eq!(HeadingLevel::H4.text_class(), "ld-text-body");
    }

    #[test]
    fn has_text_is_false_for_empty_and_true_otherwise() {
        assert!(!has_text(""));
        assert!(has_text(" "));
        assert!(has_text("Case overview"));
    }

    /// Guards the "empty optional regions leave no spacing" contract at the
    /// source level: the eyebrow/description blocks must stay conditional
    /// on [`has_text`], never unconditionally rendered, or an empty caller
    /// prop would still occupy a `space-y-1` gap.
    #[test]
    fn eyebrow_and_description_render_conditionally_on_has_text() {
        let source = include_str!("section_heading.rs");
        let component = source
            .split_once("pub fn SectionHeading(")
            .expect("SectionHeading component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert_eq!(
            component.matches("has_text(&text)").count(),
            2,
            "expected exactly the eyebrow and description blocks to gate on has_text: {component}"
        );
    }

    /// Guards against silently duplicating `PageHeader`'s own `<h1>`/page
    /// title semantics: `SectionHeading` must never render an `h1`.
    #[test]
    fn section_heading_never_renders_an_h1() {
        let source = include_str!("section_heading.rs");
        let component = source
            .split_once("pub fn SectionHeading(")
            .expect("SectionHeading component source")
            .1
            .split_once("\n#[cfg(test)]")
            .map_or(source, |(before, _)| before);
        assert!(
            !component.contains("<h1"),
            "SectionHeading must not duplicate PageHeader's <h1>"
        );
    }

    /// The rendering half of the file only, narrowed the same way
    /// [`RecordHeader`](super::RecordHeader)'s own `component_source` is
    /// narrowed: from the component signature to the test module.
    fn component_source() -> &'static str {
        let source = include_str!("section_heading.rs");
        let after_signature = source
            .split_once("pub fn SectionHeading(")
            .expect("SectionHeading component source")
            .1;
        after_signature
            .split_once("\n#[cfg(test)]")
            .map_or(after_signature, |(before, _)| before)
    }

    // -- SectionHeadingStatusPlacement (ldui-17rz) --------------------------

    #[test]
    fn status_placement_defaults_to_inline_for_source_compatibility() {
        assert_eq!(
            SectionHeadingStatusPlacement::default(),
            SectionHeadingStatusPlacement::Inline
        );
    }

    #[test]
    fn status_placement_markers_are_distinct() {
        assert_eq!(SectionHeadingStatusPlacement::Inline.as_str(), "inline");
        assert_eq!(SectionHeadingStatusPlacement::Trailing.as_str(), "trailing");
        assert_ne!(
            SectionHeadingStatusPlacement::Inline.as_str(),
            SectionHeadingStatusPlacement::Trailing.as_str()
        );
    }

    /// The `Inline` arm of the placement match must hand `status` straight
    /// through untouched and leave `trailing_status` empty -- i.e. the
    /// default caller (nobody passes `status_placement`) gets back exactly
    /// the `(Some(view), None)` shape this component always produced, so the
    /// title-row markup below it is provably unreached by the new branch.
    #[test]
    fn inline_placement_passes_status_through_and_trailing_status_stays_empty() {
        let source = component_source();
        assert!(
            source.contains("SectionHeadingStatusPlacement::Inline => (status, None),"),
            "Inline must forward status unchanged and leave trailing_status empty: {source}"
        );
    }

    /// Guards DOM order (title/description, then status, then actions) and
    /// that the trailing status slot is never duplicated into the actions
    /// wrapper: the two wrappers use distinct data attributes and status is
    /// written to source before actions.
    #[test]
    fn trailing_status_wrapper_precedes_actions_wrapper_and_uses_a_distinct_attribute() {
        let source = component_source();
        let status_pos = source
            .find(r#"data-section-heading-status="true""#)
            .expect("trailing status wrapper attribute");
        let actions_pos = source
            .find(r#"data-section-heading-actions="true""#)
            .expect("actions wrapper attribute");
        assert!(
            status_pos < actions_pos,
            "trailing status must be written (and therefore rendered) before actions: {source}"
        );
        // Exactly one occurrence each -- neither wrapper is duplicated, and
        // the status wrapper's own attribute name never collides with the
        // actions wrapper's.
        assert_eq!(
            source
                .matches(r#"data-section-heading-status="true""#)
                .count(),
            1
        );
        assert_eq!(
            source
                .matches(r#"data-section-heading-actions="true""#)
                .count(),
            1
        );
    }

    /// Neither the trailing status wrapper nor the actions wrapper may carry
    /// an interactive role or a tab stop of its own -- status must never
    /// read to assistive tech as a control, and the two slots must never be
    /// indistinguishable from one another by role.
    #[test]
    fn trailing_status_and_actions_wrappers_carry_no_interactive_role_or_tabindex() {
        let source = component_source();
        assert!(!source.contains(r#"role="button""#));
        assert!(!source.contains("tabindex="));
    }
}
