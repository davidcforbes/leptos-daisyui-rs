/// Root classes for `AppShell`. When a status bar slot is present, the root
/// switches from a single-row layout to a column layout so the main
/// 3-panel row and the status bar can stack vertically; the row itself then
/// gets `flex-1 min-h-0` (applied separately in the component) so it fills
/// the remaining height above the status bar. When no status bar is given,
/// this is unchanged from the shell's original single-row root class.
pub fn app_shell_root_class(has_status_bar: bool) -> &'static str {
    if has_status_bar {
        "flex flex-col h-full w-full"
    } else {
        "flex h-full w-full"
    }
}

/// Inline `style` for `AppShellIconNav`'s branded background: a base colour, an
/// optional gradient over it, and an optional texture image on top.
///
/// Returns `""` when no layer is asked for, so a rail with no branding renders
/// exactly the markup it always did.
///
/// ## Why the base colour is inline rather than a class
///
/// The rail hard-codes `bg-base-300`, and a consumer's own `bg-[#0b1e3a]` class
/// does *not* reliably beat it — both are plain utilities in the same cascade
/// layer, so which one wins is down to their order in the generated sheet
/// (measured: `bg-base-300` wins, and the branded rail stays grey). An inline
/// `background-color` outranks every class, so `color` here is the one way a
/// consumer can actually set the rail's base colour.
///
/// ## Why this composites correctly
///
/// Every layer goes on the `<nav>`'s own background, and an element's
/// background always paints *beneath* its content. Nav-item hover/active
/// backgrounds and count badges are content — they paint on top, unobscured,
/// with no z-index or overlay element needed.
///
/// `background-image` is ordered front-to-back, so the texture is listed first
/// and the gradient behind it: colour → gradient → texture, which is the
/// layering the branded desktop rail uses.
///
/// ## Escaping
///
/// `image` is a URL and goes inside a quoted CSS `url("…")`, so it is escaped
/// ([`css_string_escape`]) and cannot break out of the string to inject
/// declarations. `color` is validated by [`sanitize_css_color`] and dropped if
/// it is not colour-shaped. `gradient` is raw CSS by design (it has to be — it
/// is a `linear-gradient(…)` expression) and is therefore trusted input,
/// exactly like the `class` prop it sits next to: never build it from user
/// data.
pub fn icon_nav_background_style(
    color: &str,
    image: &str,
    gradient: &str,
    size: &str,
    repeat: &str,
) -> String {
    let image = image.trim();
    let gradient = gradient.trim();

    let mut style = String::new();

    if let Some(color) = sanitize_css_color(color) {
        style.push_str(&format!("background-color: {color};"));
    }

    let mut layers: Vec<String> = Vec::with_capacity(2);
    if !image.is_empty() {
        layers.push(format!("url(\"{}\")", css_string_escape(image)));
    }
    if !gradient.is_empty() {
        layers.push(gradient.to_string());
    }

    if !layers.is_empty() {
        if !style.is_empty() {
            style.push(' ');
        }
        style.push_str(&format!("background-image: {};", layers.join(", ")));
        // Sizing only means anything once there is an image layer; a gradient
        // fills its box regardless.
        if !image.is_empty() {
            style.push_str(&format!(
                " background-size: {size}; background-repeat: {repeat}; background-position: center;"
            ));
        }
    }

    style
}

/// Accept `value` only if it is shaped like a CSS colour, so it cannot smuggle
/// a second declaration into the `style` attribute (`"red; background-image:
/// url(evil)"`).
///
/// Permits what colours are actually written with — hex (`#0b1e3a`), functional
/// notation (`rgb(…)`, `oklch(…)`, `color-mix(…)`), `var(--brand)`, and named
/// colours — and rejects anything containing `;`, `{`, `}`, or a quote, which
/// are the only characters that could end the declaration. Returns `None` for
/// an empty or rejected value, meaning "emit no colour".
pub fn sanitize_css_color(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let colour_shaped = value.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '#' | '%' | '.' | ',' | '(' | ')' | '-' | '_' | ' ' | '/')
    });
    colour_shaped.then(|| value.to_string())
}

/// Escape a string for interpolation into a double-quoted CSS string, per the
/// CSS syntax spec: backslashes and quotes are escaped, and newlines (which
/// terminate a CSS string and would let the rest of the value be re-parsed as
/// declarations) are dropped along with the other control characters.
pub fn css_string_escape(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            other => vec![other],
        })
        .collect()
}

/// Classes for a group of `AppShellIconNavItem`s within `AppShellIconNav`.
/// `pinned` appends `mt-auto`, pushing the group (and anything after it) to
/// the bottom of the icon nav strip -- the CSS equivalent of d2d-ui's
/// `AppShell::add_bottom_nav_item`, which stacked a second cluster upward
/// from the rail's foot.
pub fn nav_group_class(pinned: bool) -> &'static str {
    if pinned {
        "flex flex-col items-center gap-1 mt-auto"
    } else {
        "flex flex-col items-center gap-1"
    }
}
