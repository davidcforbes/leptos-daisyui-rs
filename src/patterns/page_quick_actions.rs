//! Opinionated, wrapping icon-action row for a [`PageHeader`](super::PageHeader)'s
//! `actions` slot.

use crate::components::{Icon, IconSize};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// Responsive behavior for a [`PageQuickActionContent`]'s visible label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageQuickActionLabelVisibility {
    /// Label text is always visible beside the icon (default).
    #[default]
    Always,
    /// Label text is visually hidden below Tailwind's `sm` breakpoint and
    /// reappears at `sm` and above. The label stays present for assistive
    /// technology at every width (`sr-only`, never `hidden`), so the
    /// surrounding Button/LinkButton's accessible name never changes --
    /// only its *visible* content collapses to the icon. Pair with a
    /// [`Tooltip`](crate::components::Tooltip) wrapping the surrounding
    /// action element so a sighted mouse/keyboard user still sees the label
    /// on hover/focus once it collapses; see [`PageQuickActionContent`]'s
    /// doc example.
    CollapseBelowSm,
}

impl PageQuickActionLabelVisibility {
    /// Stable runtime marker.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::CollapseBelowSm => "collapse-below-sm",
        }
    }

    /// Classes applied to the label `<span>`.
    const fn label_class(self) -> &'static str {
        match self {
            Self::Always => "",
            Self::CollapseBelowSm => "sr-only sm:not-sr-only sm:inline",
        }
    }
}

/// Icon-plus-label content for one [`PageQuickActions`] entry.
///
/// Place this *inside* an LDUI [`Button`](crate::components::Button),
/// [`LinkButton`](crate::components::LinkButton), or a
/// [`ButtonType::Submit`](crate::components::ButtonType::Submit) `Button`
/// wrapped in a caller-owned `<form>` -- never as a standalone control. It
/// owns icon size, icon/text gap, and alignment, plus (opt-in) responsive
/// label collapse; it never owns activation, routing, HTTP method, or
/// authorization -- those stay on the surrounding Button/LinkButton/form.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Button, ButtonColor, ButtonSize, ButtonStyle, Tooltip};
/// use leptos_daisyui_rs::patterns::{
///     PageQuickActionContent, PageQuickActionLabelVisibility, PageQuickActions,
/// };
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <PageQuickActions label="Case actions">
///             // Always-visible icon + label (the common case).
///             <Button style=ButtonStyle::Outline size=ButtonSize::Sm color=ButtonColor::Primary>
///                 <PageQuickActionContent icon="plus" label="New matter" />
///             </Button>
///
///             // Icon-only below `sm`: the Tooltip supplies the same text on
///             // hover/focus once the visible label collapses, and the label
///             // stays in the accessible name at every width via `sr-only`.
///             <Tooltip tip="Export report">
///                 <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
///                     <PageQuickActionContent
///                         icon="upload"
///                         label="Export report"
///                         label_visibility=PageQuickActionLabelVisibility::CollapseBelowSm
///                     />
///                 </Button>
///             </Tooltip>
///         </PageQuickActions>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("inline-flex items-center gap-2 sr-only sm:not-sr-only sm:inline");
/// ```
#[component]
pub fn PageQuickActionContent(
    /// Lucide icon name, translated through the shared sprite -- see
    /// [`Icon`](crate::components::Icon).
    #[prop(into)]
    icon: Signal<String>,

    /// Visible (and always accessible) label text.
    #[prop(into)]
    label: Signal<String>,

    /// Icon size. Defaults to [`IconSize::Small`] -- matched to the
    /// `ButtonSize::Sm` quick-action convention documented on
    /// [`PageQuickActions`].
    #[prop(optional, into, default = Signal::stored(IconSize::Small))]
    icon_size: Signal<IconSize>,

    /// Responsive behavior for the visible label. Defaults to
    /// [`PageQuickActionLabelVisibility::Always`].
    #[prop(optional, into)]
    label_visibility: PageQuickActionLabelVisibility,

    /// Additional classes for the outer content `<span>`.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span
            class=move || merge_classes!("inline-flex items-center gap-2", class)
            data-page-quick-action-label-visibility=label_visibility.as_str()
        >
            <Icon name=icon size=icon_size />
            <span class=label_visibility.label_class()>{move || label.get()}</span>
        </span>
    }
}

/// Opinionated, wrapping icon-action row designed for a
/// [`PageHeader`](super::PageHeader)'s `actions` slot.
///
/// `PageQuickActions` owns three things only: an accessible group name
/// (`role="group"` + `aria-label`), a consistent gap on the canonical
/// spacing scale (`gap-2`, 8px), and left-to-right wrapping (`flex-wrap`) so
/// a full row of actions moves to a second line at compact widths instead of
/// overflowing the page horizontally. It does not render buttons itself --
/// compose it with LDUI [`Button`](crate::components::Button)/
/// [`LinkButton`](crate::components::LinkButton) (each usually wrapping
/// [`PageQuickActionContent`] for icon+label content), so activation,
/// routes, HTTP method/target, and domain authorization remain entirely
/// caller-owned. The recommended visual convention for a quick-action row is
/// `ButtonStyle::Outline` at `ButtonSize::Sm` -- a consistent secondary
/// hierarchy beside a header's title, distinct from a page's one primary
/// call-to-action -- but `PageQuickActions` does not and cannot enforce this
/// on opaque children; it is a documented convention, not a constraint.
///
/// A native POST-launch action composes with no raw daisyUI button markup by
/// pairing [`ButtonType::Submit`](crate::components::ButtonType::Submit)
/// with a caller-owned `<form>`:
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{Button, ButtonStyle, ButtonSize, ButtonType};
/// use leptos_daisyui_rs::patterns::{PageQuickActionContent, PageQuickActions};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     view! {
///         <PageQuickActions label="Satellite launch actions">
///             <form action="/office/launch" method="post" target="_blank">
///                 <input type="hidden" name="doc_id" value="42" />
///                 <Button
///                     button_type=ButtonType::Submit
///                     style=ButtonStyle::Outline
///                     size=ButtonSize::Sm
///                 >
///                     <PageQuickActionContent icon="external-link" label="Open in Office" />
///                 </Button>
///             </form>
///         </PageQuickActions>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-wrap items-center gap-2");
/// ```
///
/// ## Node References
/// - `node_ref` - References the outer `<div>` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn PageQuickActions(
    /// Accessible name for the action group (`aria-label`). Localize this
    /// per page; defaults to `"Page actions"`.
    #[prop(into, default = Signal::stored("Page actions".to_owned()))]
    label: Signal<String>,

    /// Additional classes for the outer row.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the outer `<div>` element.
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// Action content -- one or more Button/LinkButton elements, each
    /// typically wrapping a [`PageQuickActionContent`].
    children: Children,
) -> impl IntoView {
    view! {
        <div
            node_ref=node_ref
            role="group"
            aria-label=move || label.get()
            class=move || merge_classes!("flex flex-wrap items-center gap-2", class)
            data-page-quick-actions="true"
        >
            {children()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_visibility_defaults_to_always_visible() {
        assert_eq!(
            PageQuickActionLabelVisibility::default(),
            PageQuickActionLabelVisibility::Always
        );
        assert_eq!(PageQuickActionLabelVisibility::default().as_str(), "always");
        assert_eq!(PageQuickActionLabelVisibility::default().label_class(), "");
    }

    #[test]
    fn collapse_below_sm_keeps_the_label_accessible_but_visually_hides_it() {
        assert_eq!(
            PageQuickActionLabelVisibility::CollapseBelowSm.as_str(),
            "collapse-below-sm"
        );
        let class = PageQuickActionLabelVisibility::CollapseBelowSm.label_class();
        // `sr-only` keeps the text in the accessibility tree at every width;
        // it must never be `hidden`, which would remove it from both.
        assert!(class.contains("sr-only"));
        assert!(!class.contains("hidden"));
        // Reappears visually at `sm` and above.
        assert!(class.contains("sm:not-sr-only"));
        assert!(class.contains("sm:inline"));
    }

    /// Guards the wrapping contract at the source level: `PageQuickActions`
    /// must never render a fixed non-wrapping row -- that is the exact bug
    /// this pattern exists to fix on `PageHeader`'s actions slot.
    #[test]
    fn page_quick_actions_root_is_flex_wrap() {
        let source = include_str!("page_quick_actions.rs");
        let component = source
            .split_once("pub fn PageQuickActions(")
            .expect("PageQuickActions component source")
            .1;
        assert!(
            component.contains(r#"merge_classes!("flex flex-wrap items-center gap-2", class)"#),
            "expected the outer row to merge in flex-wrap: {component}"
        );
    }

    /// Guards the accessible-naming contract: the group must carry both an
    /// explicit role and a reactive `aria-label`, not rely on default div
    /// semantics.
    #[test]
    fn page_quick_actions_root_has_group_role_and_aria_label() {
        let source = include_str!("page_quick_actions.rs");
        let component = source
            .split_once("pub fn PageQuickActions(")
            .expect("PageQuickActions component source")
            .1;
        assert!(component.contains(r#"role="group""#));
        assert!(component.contains("aria-label=move || label.get()"));
    }
}
