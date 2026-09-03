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
            // `ldui-q8bj`: present ONLY when the name is unmapped, carrying the
            // offending name. A typo is now visible in the DOM, assertable in a
            // test, and greppable in a screenshot's markup -- instead of being
            // indistinguishable from a working icon.
            data-icon-unresolved=move || {
                let requested = name.get();
                let unresolved = lucide_sprite_lookup(&requested).is_none();
                // `ldui-q8bj`: fail loudly in a debug build at the point of the
                // mistake, rather than surfacing as empty space in someone's
                // screenshot weeks later. Here rather than in the mapper,
                // because rendering an unmapped name is the actual caller error.
                debug_assert!(
                    !unresolved,
                    "icon name {requested:?} is not in this crate's sprite \
                     vocabulary; call lucide_sprite_names() for the supported \
                     set (ldui-q8bj)"
                );
                unresolved.then_some(requested)
            }
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
/// An unknown name returns [`UNRESOLVED_SYMBOL`] (`blank`), because an
/// unresolvable id renders an EMPTY `<svg>` with no error at all.
///
/// ⚠️ That fallback was assumed to be noticeable and is not: `ldui-q8bj` found
/// two Dashboard buttons rendering as empty space, with every signal a
/// developer would check — the box, the classes, the `data-lucide` attribute,
/// a well-formed `<use>` — looking healthy. So the miss is now *reported*
/// rather than merely absorbed: this function `debug_assert!`s, the rendered
/// element carries `data-icon-unresolved="<name>"`, and
/// [`lucide_sprite_lookup`] returns `None` for callers who want to branch.
///
/// Note the symbol vocabulary is bounded by the HOST DOCUMENT's inlined
/// sprite, not by this crate, so mapping a further Lucide name here only
/// helps if the host's sprite defines that symbol.
///
/// # Requirement on the host document
///
/// The page must inline an SVG sprite defining these symbol ids. Referencing an
/// external file instead works in Chromium but renders NOTHING in Firefox, which
/// has never supported external `<use>`.
/// Every icon name [`lucide_to_sprite`] maps, in source order of the
/// match arms below. Kept beside the map so the two cannot drift: a test
/// asserts every entry here resolves, and that the counts agree.
const SUPPORTED_ICON_NAMES: &[&str] = &[
    "activity",
    "arrow-left",
    "arrow-right",
    "bar-chart-3",
    "calendar",
    "check",
    "chevron-down",
    "chevron-left",
    "chevron-right",
    "circle",
    "circle-alert",
    "circle-check",
    "circle-play",
    "clock",
    "close",
    "copy",
    "dollar-sign",
    "envelope",
    "expand",
    "external-link",
    "eye",
    "file",
    "file-text",
    "filter",
    "floppy-disk",
    "help-circle",
    "info",
    "layout-dashboard",
    "list",
    "log-out",
    "mail",
    "message-square",
    "minus",
    "more-horizontal",
    "more-vertical",
    "paperclip",
    "pause",
    "pencil",
    "phone",
    "phone-call",
    "play",
    "plus",
    "refresh",
    "reply",
    "resume",
    "save",
    "search",
    "send",
    "settings",
    "snapshot",
    "square-pause",
    "star",
    "target",
    "thumbs-up",
    "ticket",
    "timeline",
    "trash",
    "trending-up",
    "triangle-alert",
    "upload",
    "user",
    "users",
    "whatsapp",
    "x",
];

/// Resolves a caller's icon name to a sprite symbol id, or `None` when the
/// name is not in the vocabulary (`ldui-q8bj`).
///
/// This is the honest form of [`lucide_to_sprite`]: a miss is a distinguishable
/// value rather than a plausible-looking blank. The renderer uses it to mark
/// the element, and tests can assert on it.
pub fn lucide_sprite_lookup(name: &str) -> Option<&'static str> {
    lucide_to_sprite_inner(name)
}

/// Every icon name this crate maps (`ldui-q8bj`, suggestion 3).
///
/// The map used to be the only source of truth and callers could not see it,
/// so a caller had no way to check a name before shipping it. Sorted, so a
/// diff of the supported vocabulary is readable.
pub fn lucide_sprite_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = SUPPORTED_ICON_NAMES.to_vec();
    names.sort_unstable();
    names
}

/// The symbol id an unmapped name falls back to.
pub const UNRESOLVED_SYMBOL: &str = "blank";

/// Resolves an icon name to a sprite symbol id, falling back to
/// [`UNRESOLVED_SYMBOL`] for a name this crate does not map.
///
/// Prefer [`lucide_sprite_lookup`] when you need to know whether the name was
/// actually recognised: this function cannot tell you, because `"circle"` maps
/// to the blank glyph deliberately and an unmapped name falls back to the same
/// symbol (`ldui-q8bj`).
pub fn lucide_to_sprite(name: &str) -> &'static str {
    // Deliberately NOT asserting here. This is the documented fallback API and
    // probing it with an unknown name is a legitimate thing to do -- an
    // existing test does exactly that. The caller mistake this bug is about
    // happens at RENDER time, so that is where it is reported.
    lucide_to_sprite_inner(name).unwrap_or(UNRESOLVED_SYMBOL)
}

/// The map. Returns `None` for a name it does not know.
///
/// `Option` rather than a sentinel comparison because `"circle" => "blank"` is
/// a legitimate EXPLICIT mapping to the blank glyph, and inferring "unmapped"
/// from the symbol string would misreport it as a typo. The distinction is
/// structural, so it cannot be got wrong.
fn lucide_to_sprite_inner(name: &str) -> Option<&'static str> {
    let symbol = match name {
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
        // Run-control glyphs (ldui-kybz), requested by 4iiz-etl's operator
        // portal for a Pause/Resume/Restart/Snapshot/Reap/Freshness quick-
        // action row. `resume` and `snapshot` are consumer-vocabulary
        // aliases onto the same symbols as `play`/`save`, so a caller who
        // reaches for either spelling resolves directly instead of filing a
        // second bead for the synonym.
        "pause" => "pause",
        "square-pause" => "pause",
        "play" => "play",
        "circle-play" => "play",
        "resume" => "play",
        "save" => "floppy-disk",
        "floppy-disk" => "floppy-disk",
        "snapshot" => "floppy-disk",
        _ => return None,
    };
    Some(symbol)
}
