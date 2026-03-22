use super::*;

// StatusColor tests
#[test]
fn test_status_color_default() {
    let color = StatusColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_status_color_neutral() {
    assert_eq!(StatusColor::Neutral.as_str(), "status-neutral");
}

#[test]
fn test_status_color_primary() {
    assert_eq!(StatusColor::Primary.as_str(), "status-primary");
}

#[test]
fn test_status_color_secondary() {
    assert_eq!(StatusColor::Secondary.as_str(), "status-secondary");
}

#[test]
fn test_status_color_accent() {
    assert_eq!(StatusColor::Accent.as_str(), "status-accent");
}

#[test]
fn test_status_color_info() {
    assert_eq!(StatusColor::Info.as_str(), "status-info");
}

#[test]
fn test_status_color_success() {
    assert_eq!(StatusColor::Success.as_str(), "status-success");
}

#[test]
fn test_status_color_warning() {
    assert_eq!(StatusColor::Warning.as_str(), "status-warning");
}

#[test]
fn test_status_color_error() {
    assert_eq!(StatusColor::Error.as_str(), "status-error");
}

#[test]
fn test_status_color_clone() {
    let c1 = StatusColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_status_color_debug() {
    let color = StatusColor::Warning;
    assert!(format!("{:?}", color).contains("Warning"));
}

// StatusSize tests
#[test]
fn test_status_size_default() {
    let size = StatusSize::default();
    assert_eq!(size.as_str(), "status-md");
}

#[test]
fn test_status_size_xs() {
    assert_eq!(StatusSize::Xs.as_str(), "status-xs");
}

#[test]
fn test_status_size_sm() {
    assert_eq!(StatusSize::Sm.as_str(), "status-sm");
}

#[test]
fn test_status_size_md() {
    assert_eq!(StatusSize::Md.as_str(), "status-md");
}

#[test]
fn test_status_size_lg() {
    assert_eq!(StatusSize::Lg.as_str(), "status-lg");
}

#[test]
fn test_status_size_xl() {
    assert_eq!(StatusSize::Xl.as_str(), "status-xl");
}

#[test]
fn test_status_size_clone() {
    let s1 = StatusSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_status_size_debug() {
    let size = StatusSize::Xs;
    assert!(format!("{:?}", size).contains("Xs"));
}

#[test]
fn test_all_status_colors_return_valid_classes() {
    let variants = vec![
        (StatusColor::Default, ""),
        (StatusColor::Neutral, "status-neutral"),
        (StatusColor::Primary, "status-primary"),
        (StatusColor::Secondary, "status-secondary"),
        (StatusColor::Accent, "status-accent"),
        (StatusColor::Info, "status-info"),
        (StatusColor::Success, "status-success"),
        (StatusColor::Warning, "status-warning"),
        (StatusColor::Error, "status-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_status_sizes_return_valid_classes() {
    let variants = vec![
        (StatusSize::Xs, "status-xs"),
        (StatusSize::Sm, "status-sm"),
        (StatusSize::Md, "status-md"),
        (StatusSize::Lg, "status-lg"),
        (StatusSize::Xl, "status-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
