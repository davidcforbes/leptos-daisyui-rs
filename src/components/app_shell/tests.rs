use super::*;

// app_shell_root_class tests

#[test]
fn test_app_shell_root_class_no_status_bar_is_unchanged() {
    assert_eq!(app_shell_root_class(false), "flex h-full w-full");
}

#[test]
fn test_app_shell_root_class_with_status_bar_switches_to_column() {
    let class = app_shell_root_class(true);
    assert_eq!(class, "flex flex-col h-full w-full");
    assert!(class.contains("flex-col"));
    assert_ne!(class, app_shell_root_class(false));
}

// icon_nav_background_style tests

#[test]
fn test_icon_nav_background_style_is_empty_without_layers() {
    // A rail with no branding must render exactly as it always did -- and with
    // no `style` attribute at all, which the component keys off "".
    assert_eq!(
        icon_nav_background_style("", "", "", "cover", "no-repeat"),
        ""
    );
    assert_eq!(
        icon_nav_background_style("  ", "   ", "  ", "cover", "no-repeat"),
        ""
    );
}

#[test]
fn test_icon_nav_background_style_image_only() {
    let style = icon_nav_background_style("", "/brand/sidebar-bg.png", "", "cover", "no-repeat");
    assert_eq!(
        style,
        "background-image: url(\"/brand/sidebar-bg.png\"); \
         background-size: cover; background-repeat: no-repeat; background-position: center;"
    );
}

#[test]
fn test_icon_nav_background_style_color_only() {
    // The base colour must be inline: a bg-[#0b1e3a] class loses to the rail's
    // own bg-base-300, so this declaration is the only thing that can win.
    let style = icon_nav_background_style("#0b1e3a", "", "", "cover", "no-repeat");
    assert_eq!(style, "background-color: #0b1e3a;");
}

#[test]
fn test_icon_nav_background_style_all_three_layers() {
    let style = icon_nav_background_style(
        "#0b1e3a",
        "/brand/dot-wave.png",
        "linear-gradient(180deg, rgba(255,255,255,0.06), rgba(0,0,0,0.25))",
        "auto",
        "repeat",
    );
    assert!(style.starts_with("background-color: #0b1e3a;"));
    assert!(style.contains("background-image: url(\"/brand/dot-wave.png\"), linear-gradient("));
}

#[test]
fn test_icon_nav_background_style_gradient_only_omits_image_sizing() {
    let style = icon_nav_background_style(
        "",
        "",
        "linear-gradient(180deg, #0b1e3a, #071429)",
        "cover",
        "no-repeat",
    );
    assert_eq!(
        style,
        "background-image: linear-gradient(180deg, #0b1e3a, #071429);"
    );
    assert!(!style.contains("background-size"));
}

#[test]
fn test_icon_nav_background_style_lists_texture_in_front_of_gradient() {
    // background-image is ordered front-to-back: the texture must sit ON the
    // gradient, not behind it, or the branded rail loses its dot-wave.
    let style = icon_nav_background_style(
        "",
        "/brand/dot-wave.png",
        "linear-gradient(180deg, #0b1e3a, #071429)",
        "auto",
        "repeat",
    );
    let texture = style.find("url(").expect("texture layer");
    let gradient = style.find("linear-gradient").expect("gradient layer");
    assert!(texture < gradient, "texture must precede gradient: {style}");
}

#[test]
fn test_icon_nav_background_style_honors_size_and_repeat() {
    let style = icon_nav_background_style("", "/t.png", "", "auto", "repeat");
    assert!(style.contains("background-size: auto;"));
    assert!(style.contains("background-repeat: repeat;"));
}

// sanitize_css_color tests

#[test]
fn test_sanitize_css_color_accepts_the_notations_colors_are_written_in() {
    for ok in [
        "#0b1e3a",
        "rgb(11, 30, 58)",
        "rgba(11,30,58,0.5)",
        "oklch(0.16 0.03 260)",
        "var(--brand-navy)",
        "midnightblue",
        "color-mix(in oklch, red 40%, blue)",
    ] {
        assert_eq!(
            sanitize_css_color(ok),
            Some(ok.to_string()),
            "rejected {ok}"
        );
    }
}

#[test]
fn test_sanitize_css_color_rejects_a_smuggled_declaration() {
    // A ';' would end background-color and start an attacker's own declaration.
    assert_eq!(
        sanitize_css_color("red; background-image: url(https://evil.example/x.png)"),
        None
    );
    assert_eq!(sanitize_css_color("red\"}"), None);
    assert_eq!(sanitize_css_color(""), None);
    assert_eq!(sanitize_css_color("   "), None);
}

#[test]
fn test_icon_nav_background_style_drops_a_hostile_color_entirely() {
    let style = icon_nav_background_style(
        "red; background-image: url(https://evil.example/x.png)",
        "",
        "",
        "cover",
        "no-repeat",
    );
    assert_eq!(style, "", "a rejected colour must emit nothing: {style}");
}

// css_string_escape tests

#[test]
fn test_css_string_escape_passes_ordinary_urls_through() {
    assert_eq!(
        css_string_escape("/brand/sidebar-bg.png"),
        "/brand/sidebar-bg.png"
    );
    // Data URIs contain ';' and '/', which are harmless inside a quoted string.
    assert_eq!(
        css_string_escape("data:image/png;base64,iVBORw0KGgo="),
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn test_css_string_escape_neutralizes_a_quote_breakout() {
    // Without escaping, this would close the url() string and inject a second
    // declaration into the style attribute.
    let hostile = "x.png\"); background: red; content: \"";
    let escaped = css_string_escape(hostile);

    // Every quote that survives must be backslash-escaped, so none of them can
    // terminate the url("...") string the value is interpolated into.
    let mut prev = ' ';
    for c in escaped.chars() {
        assert!(
            c != '"' || prev == '\\',
            "unescaped quote survived: {escaped}"
        );
        prev = c;
    }
    assert_eq!(escaped, "x.png\\\"); background: red; content: \\\"");

    let style = icon_nav_background_style("", hostile, "", "cover", "no-repeat");
    assert!(
        style.starts_with("background-image: url(\"x.png\\\""),
        "url string must stay one string: {style}"
    );
}

#[test]
fn test_css_string_escape_escapes_backslashes_and_drops_newlines() {
    assert_eq!(css_string_escape("a\\b"), "a\\\\b");
    // A raw newline terminates a CSS string; anything after it would be parsed
    // as further declarations.
    assert_eq!(css_string_escape("a\nb"), "ab");
    assert_eq!(css_string_escape("a\r\tb"), "ab");
}

// nav_group_class tests

#[test]
fn test_nav_group_class_unpinned() {
    assert_eq!(nav_group_class(false), "flex flex-col items-center gap-1");
}

#[test]
fn test_nav_group_class_pinned_appends_mt_auto() {
    let class = nav_group_class(true);
    assert!(class.contains("mt-auto"));
    assert_ne!(class, nav_group_class(false));
}
