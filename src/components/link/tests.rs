use super::*;

// LinkColor tests
#[test]
fn test_link_color_default() {
    let val = LinkColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_link_color_neutral() {
    let val = LinkColor::Neutral;
    assert_eq!(val.as_str(), "link-neutral");
}

#[test]
fn test_link_color_primary() {
    let val = LinkColor::Primary;
    assert_eq!(val.as_str(), "link-primary");
}

#[test]
fn test_link_color_secondary() {
    let val = LinkColor::Secondary;
    assert_eq!(val.as_str(), "link-secondary");
}

#[test]
fn test_link_color_accent() {
    let val = LinkColor::Accent;
    assert_eq!(val.as_str(), "link-accent");
}

#[test]
fn test_link_color_success() {
    let val = LinkColor::Success;
    assert_eq!(val.as_str(), "link-success");
}

#[test]
fn test_link_color_info() {
    let val = LinkColor::Info;
    assert_eq!(val.as_str(), "link-info");
}

#[test]
fn test_link_color_warning() {
    let val = LinkColor::Warning;
    assert_eq!(val.as_str(), "link-warning");
}

#[test]
fn test_link_color_error() {
    let val = LinkColor::Error;
    assert_eq!(val.as_str(), "link-error");
}

#[test]
fn test_link_color_clone() {
    let v1 = LinkColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_link_color_debug() {
    let val = LinkColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// Comprehensive coverage test
#[test]
fn test_all_link_colors_return_valid_classes() {
    let variants = vec![
        (LinkColor::Default, ""),
        (LinkColor::Neutral, "link-neutral"),
        (LinkColor::Primary, "link-primary"),
        (LinkColor::Secondary, "link-secondary"),
        (LinkColor::Accent, "link-accent"),
        (LinkColor::Success, "link-success"),
        (LinkColor::Info, "link-info"),
        (LinkColor::Warning, "link-warning"),
        (LinkColor::Error, "link-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
