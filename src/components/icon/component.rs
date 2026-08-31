use super::style::IconSize;
use leptos::{html, prelude::*};

/// An icon component with Lucide icon support.
///
/// This component provides a wrapper for displaying icons from the Lucide icon library.
/// You need to include Lucide icons in your project separately.
///
/// # Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <Icon name="heart" size=IconSize::Large color="text-error" />
///         <Icon name="star" size=IconSize::Medium />
///     }
/// }
/// ```
///
/// # Setup: inline an SVG sprite (NOT a Lucide script)
///
/// This component renders `<use href="#id">` against a sprite the host page
/// inlines. It needs NO JavaScript and no CDN — see [`lucide_to_sprite`] for the
/// full reasoning and for the name translation.
///
/// ```html
/// <!-- inlined into index.html, e.g. by Trunk's `rel="inline"` -->
/// <svg style="display:none"><defs>
///   <symbol id="clock" viewBox="0 0 24 24"><path d="..."/></symbol>
/// </defs></svg>
/// ```
///
/// Reference an external file instead and it works in Chromium but renders
/// NOTHING in Firefox, which has never supported external `<use>`.
///
/// # CSS Classes
/// Add to your `input.css`:
/// ```css
/// @source inline("w-4 h-4 w-5 h-5 w-6 h-6 w-8 h-8 w-12 h-12");
/// @source inline("inline-block");
/// ```
#[component]
pub fn Icon(
    /// Icon name from Lucide icons (e.g., "heart", "star", "user")
    #[prop(into)]
    name: Signal<String>,
    /// Size of the icon
    #[prop(optional, into)]
    size: Signal<IconSize>,
    /// Color class for the icon (e.g., "text-primary", "text-error")
    #[prop(optional, into)]
    color: Signal<String>,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,
    /// Reference to the underlying DOM node
    #[prop(optional)]
    node_ref: NodeRef<html::I>,
) -> impl IntoView {
    let computed_class = move || {
        let mut classes = vec!["inline-block", size.get().as_str()];
        let color_val = color.get();
        if !color_val.is_empty() {
            classes.push(&color_val);
        }
        if !class.is_empty() {
            classes.push(class);
        }
        classes.join(" ")
    };

    // The `<i>` element is DELIBERATELY KEPT. Rendering an `<svg>` here instead
    // would change this component's public `node_ref: NodeRef<html::I>` and every
    // sizing class already written against `i`. The sprite goes INSIDE it, which
    // is also what Lucide's own replacement used to do in spirit — except this
    // needs no script, so the glyph is present on first paint.
    //
    // `data-lucide` is retained as an attribute for styling and test selectors
    // that already key off it. Nothing reads it as an instruction any more.
    view! {
        <i
            node_ref=node_ref
            data-lucide=move || name.get()
            class=computed_class
        >
            <svg
                width="100%"
                height="100%"
                aria-hidden="true"
                focusable="false"
            >
                <use href=move || format!("#{}", lucide_to_sprite(&name.get())) />
            </svg>
        </i>
    }
}

/// Translate a Lucide icon name to a 4Ease-sprite symbol id.
///
/// # Why this exists
///
/// [`Icon`] used to render `<i data-lucide="name">` and rely on Lucide's JS to
/// replace it with an `<svg>`. That meant a third-party CDN on the critical path,
/// a `MutationObserver` to catch elements mounted after a fetch resolved, and a
/// whole class of bug where an icon was silently absent because the scan had not
/// run yet. It now renders `<use href="#id">` against a sprite the host document
/// inlines, which resolves before any script runs.
///
/// Existing call sites pass LUCIDE names (`SlaChip` asks for `triangle-alert`),
/// so those names keep working and are translated here. That is the whole reason
/// this is a function in the library rather than a table in one consumer: the
/// library's OWN components — `SlaChip`, `AppShell`, `DataTable`, `IconTile`,
/// `Gantt` — render `Icon` internally, and a mapping held by a consumer could not
/// reach them.
///
/// An unknown name returns `blank`, the sprite's neutral glyph, because an
/// unresolvable id renders an EMPTY `<svg>` with no error at all — and a visible
/// placeholder is easier to notice than nothing.
///
/// # Requirement on the host document
///
/// The page must inline an SVG sprite defining these symbol ids. Referencing an
/// external file instead works in Chromium but renders NOTHING in Firefox, which
/// has never supported external `<use>`.
pub fn lucide_to_sprite(name: &str) -> &'static str {
    match name {
        // Same concept, same name.
        "list" => "list",
        "calendar" => "calendar",
        "clock" => "clock",
        "users" => "users",
        "search" => "search",
        "star" => "star",
        "filter" => "filter",
        "plus" => "plus",
        "minus" => "minus",
        "eye" => "eye",
        "copy" => "copy",
        "trash" => "trash",
        "phone" => "phone",
        "send" => "send",
        "refresh" => "refresh",
        "expand" => "expand",
        "close" => "close",
        "pencil" => "pencil",
        "reply" => "reply",
        "envelope" => "envelope",
        "timeline" => "timeline",
        "ticket" => "ticket",
        "upload" => "upload",
        "file" => "file",
        // Same concept under the sprite's naming.
        "triangle-alert" => "triangle-exclamation",
        "circle-alert" => "circle-exclamation-fill",
        "circle-check" => "circle-check",
        "info" => "circle-info",
        "help-circle" => "circle-question",
        "file-text" => "file-lines",
        "target" => "bullseye-arrow",
        "trending-up" => "arrow-trend-up",
        "circle" => "blank",
        "x" => "close",
        "check" => "circle-check",
        "chevron-left" => "angle-left",
        "chevron-right" => "angle-right",
        "chevron-down" => "angle-right",
        "arrow-left" => "arrow-left",
        "arrow-right" => "arrow-right",
        "external-link" => "arrow-up-right-from-square",
        "message-square" => "message",
        "mail" => "envelope",
        "paperclip" => "paper-clip",
        "more-vertical" => "ellipsis-vertical",
        "more-horizontal" => "ellipsis",
        "log-out" => "logout",
        "user" => "user",
        // No sprite equivalent exists for these; the closest honest glyph.
        "layout-dashboard" => "performance-stats",
        "activity" => "performance-stats",
        "settings" => "settings-gear",
        // Same concept under the sprite's naming (ldui-af4b).
        "dollar-sign" => "us-dollar",
        "thumbs-up" => "thumbs-up",
        "bar-chart-3" => "performance-stats",
        "phone-call" => "phone-ring",
        "whatsapp" => "whatsapp",
        _ => "blank",
    }
}
