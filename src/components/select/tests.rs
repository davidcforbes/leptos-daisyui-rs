use super::*;

// SelectStyle tests
#[test]
fn test_select_style_default() {
    let style = SelectStyle::default();
    assert_eq!(style.as_str(), "");
}

#[test]
fn test_select_style_ghost() {
    assert_eq!(SelectStyle::Ghost.as_str(), "select-ghost");
}

#[test]
fn test_select_style_clone() {
    let s1 = SelectStyle::Ghost;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_select_style_debug() {
    let style = SelectStyle::Ghost;
    assert!(format!("{:?}", style).contains("Ghost"));
}

// SelectColor tests
#[test]
fn test_select_color_default() {
    let color = SelectColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_select_color_primary() {
    assert_eq!(SelectColor::Primary.as_str(), "select-primary");
}

#[test]
fn test_select_color_secondary() {
    assert_eq!(SelectColor::Secondary.as_str(), "select-secondary");
}

#[test]
fn test_select_color_accent() {
    assert_eq!(SelectColor::Accent.as_str(), "select-accent");
}

#[test]
fn test_select_color_info() {
    assert_eq!(SelectColor::Info.as_str(), "select-info");
}

#[test]
fn test_select_color_success() {
    assert_eq!(SelectColor::Success.as_str(), "select-success");
}

#[test]
fn test_select_color_warning() {
    assert_eq!(SelectColor::Warning.as_str(), "select-warning");
}

#[test]
fn test_select_color_error() {
    assert_eq!(SelectColor::Error.as_str(), "select-error");
}

#[test]
fn test_select_color_clone() {
    let c1 = SelectColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_select_color_debug() {
    let color = SelectColor::Accent;
    assert!(format!("{:?}", color).contains("Accent"));
}

// SelectSize tests
#[test]
fn test_select_size_default() {
    let size = SelectSize::default();
    assert_eq!(size.as_str(), "select-md");
}

#[test]
fn test_select_size_xs() {
    assert_eq!(SelectSize::Xs.as_str(), "select-xs");
}

#[test]
fn test_select_size_sm() {
    assert_eq!(SelectSize::Sm.as_str(), "select-sm");
}

#[test]
fn test_select_size_md() {
    assert_eq!(SelectSize::Md.as_str(), "select-md");
}

#[test]
fn test_select_size_lg() {
    assert_eq!(SelectSize::Lg.as_str(), "select-lg");
}

#[test]
fn test_select_size_xl() {
    assert_eq!(SelectSize::Xl.as_str(), "select-xl");
}

#[test]
fn test_select_size_clone() {
    let s1 = SelectSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_select_size_debug() {
    let size = SelectSize::Xl;
    assert!(format!("{:?}", size).contains("Xl"));
}

#[test]
fn test_all_select_styles_return_valid_classes() {
    let variants = vec![
        (SelectStyle::Default, ""),
        (SelectStyle::Ghost, "select-ghost"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_select_colors_return_valid_classes() {
    let variants = vec![
        (SelectColor::Default, ""),
        (SelectColor::Primary, "select-primary"),
        (SelectColor::Secondary, "select-secondary"),
        (SelectColor::Accent, "select-accent"),
        (SelectColor::Info, "select-info"),
        (SelectColor::Success, "select-success"),
        (SelectColor::Warning, "select-warning"),
        (SelectColor::Error, "select-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_select_sizes_return_valid_classes() {
    let variants = vec![
        (SelectSize::Xs, "select-xs"),
        (SelectSize::Sm, "select-sm"),
        (SelectSize::Md, "select-md"),
        (SelectSize::Lg, "select-lg"),
        (SelectSize::Xl, "select-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
