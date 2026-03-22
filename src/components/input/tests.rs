use super::*;

// InputStyle tests
#[test]
fn test_input_style_default() {
    let val = InputStyle::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_input_style_ghost() {
    let val = InputStyle::Ghost;
    assert_eq!(val.as_str(), "input-ghost");
}

#[test]
fn test_input_style_clone() {
    let v1 = InputStyle::Ghost;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_input_style_debug() {
    let val = InputStyle::Ghost;
    assert!(format!("{:?}", val).contains("Ghost"));
}

// InputColor tests
#[test]
fn test_input_color_default() {
    let val = InputColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_input_color_neutral() {
    let val = InputColor::Neutral;
    assert_eq!(val.as_str(), "input-neutral");
}

#[test]
fn test_input_color_primary() {
    let val = InputColor::Primary;
    assert_eq!(val.as_str(), "input-primary");
}

#[test]
fn test_input_color_secondary() {
    let val = InputColor::Secondary;
    assert_eq!(val.as_str(), "input-secondary");
}

#[test]
fn test_input_color_accent() {
    let val = InputColor::Accent;
    assert_eq!(val.as_str(), "input-accent");
}

#[test]
fn test_input_color_info() {
    let val = InputColor::Info;
    assert_eq!(val.as_str(), "input-info");
}

#[test]
fn test_input_color_success() {
    let val = InputColor::Success;
    assert_eq!(val.as_str(), "input-success");
}

#[test]
fn test_input_color_warning() {
    let val = InputColor::Warning;
    assert_eq!(val.as_str(), "input-warning");
}

#[test]
fn test_input_color_error() {
    let val = InputColor::Error;
    assert_eq!(val.as_str(), "input-error");
}

#[test]
fn test_input_color_clone() {
    let v1 = InputColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_input_color_debug() {
    let val = InputColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// InputSize tests
#[test]
fn test_input_size_default() {
    let val = InputSize::default();
    assert_eq!(val.as_str(), "input-md");
}

#[test]
fn test_input_size_xs() {
    let val = InputSize::Xs;
    assert_eq!(val.as_str(), "input-xs");
}

#[test]
fn test_input_size_sm() {
    let val = InputSize::Sm;
    assert_eq!(val.as_str(), "input-sm");
}

#[test]
fn test_input_size_md() {
    let val = InputSize::Md;
    assert_eq!(val.as_str(), "input-md");
}

#[test]
fn test_input_size_lg() {
    let val = InputSize::Lg;
    assert_eq!(val.as_str(), "input-lg");
}

#[test]
fn test_input_size_xl() {
    let val = InputSize::Xl;
    assert_eq!(val.as_str(), "input-xl");
}

#[test]
fn test_input_size_clone() {
    let v1 = InputSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_input_size_debug() {
    let val = InputSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage tests
#[test]
fn test_all_input_styles_return_valid_classes() {
    let variants = vec![
        (InputStyle::Default, ""),
        (InputStyle::Ghost, "input-ghost"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_input_colors_return_valid_classes() {
    let variants = vec![
        (InputColor::Default, ""),
        (InputColor::Neutral, "input-neutral"),
        (InputColor::Primary, "input-primary"),
        (InputColor::Secondary, "input-secondary"),
        (InputColor::Accent, "input-accent"),
        (InputColor::Info, "input-info"),
        (InputColor::Success, "input-success"),
        (InputColor::Warning, "input-warning"),
        (InputColor::Error, "input-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_input_sizes_return_valid_classes() {
    let variants = vec![
        (InputSize::Xs, "input-xs"),
        (InputSize::Sm, "input-sm"),
        (InputSize::Md, "input-md"),
        (InputSize::Lg, "input-lg"),
        (InputSize::Xl, "input-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
