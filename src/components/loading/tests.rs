use super::*;

// LoadingColor tests
#[test]
fn test_loading_color_default() {
    let val = LoadingColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_loading_color_neutral() {
    let val = LoadingColor::Neutral;
    assert_eq!(val.as_str(), "text-neutral");
}

#[test]
fn test_loading_color_primary() {
    let val = LoadingColor::Primary;
    assert_eq!(val.as_str(), "text-primary");
}

#[test]
fn test_loading_color_secondary() {
    let val = LoadingColor::Secondary;
    assert_eq!(val.as_str(), "text-secondary");
}

#[test]
fn test_loading_color_accent() {
    let val = LoadingColor::Accent;
    assert_eq!(val.as_str(), "text-accent");
}

#[test]
fn test_loading_color_info() {
    let val = LoadingColor::Info;
    assert_eq!(val.as_str(), "text-info");
}

#[test]
fn test_loading_color_success() {
    let val = LoadingColor::Success;
    assert_eq!(val.as_str(), "text-success");
}

#[test]
fn test_loading_color_warning() {
    let val = LoadingColor::Warning;
    assert_eq!(val.as_str(), "text-warning");
}

#[test]
fn test_loading_color_error() {
    let val = LoadingColor::Error;
    assert_eq!(val.as_str(), "text-error");
}

#[test]
fn test_loading_color_clone() {
    let v1 = LoadingColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_loading_color_debug() {
    let val = LoadingColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// LoadingType tests
#[test]
fn test_loading_type_default() {
    let val = LoadingType::default();
    assert_eq!(val.as_str(), "loading-spinner");
}

#[test]
fn test_loading_type_spinner() {
    let val = LoadingType::Spinner;
    assert_eq!(val.as_str(), "loading-spinner");
}

#[test]
fn test_loading_type_dots() {
    let val = LoadingType::Dots;
    assert_eq!(val.as_str(), "loading-dots");
}

#[test]
fn test_loading_type_ring() {
    let val = LoadingType::Ring;
    assert_eq!(val.as_str(), "loading-ring");
}

#[test]
fn test_loading_type_ball() {
    let val = LoadingType::Ball;
    assert_eq!(val.as_str(), "loading-ball");
}

#[test]
fn test_loading_type_bars() {
    let val = LoadingType::Bars;
    assert_eq!(val.as_str(), "loading-bars");
}

#[test]
fn test_loading_type_infinity() {
    let val = LoadingType::Infinity;
    assert_eq!(val.as_str(), "loading-infinity");
}

#[test]
fn test_loading_type_clone() {
    let v1 = LoadingType::Ring;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_loading_type_debug() {
    let val = LoadingType::Infinity;
    assert!(format!("{:?}", val).contains("Infinity"));
}

// LoadingSize tests
#[test]
fn test_loading_size_default() {
    let val = LoadingSize::default();
    assert_eq!(val.as_str(), "loading-md");
}

#[test]
fn test_loading_size_xs() {
    let val = LoadingSize::Xs;
    assert_eq!(val.as_str(), "loading-xs");
}

#[test]
fn test_loading_size_sm() {
    let val = LoadingSize::Sm;
    assert_eq!(val.as_str(), "loading-sm");
}

#[test]
fn test_loading_size_md() {
    let val = LoadingSize::Md;
    assert_eq!(val.as_str(), "loading-md");
}

#[test]
fn test_loading_size_lg() {
    let val = LoadingSize::Lg;
    assert_eq!(val.as_str(), "loading-lg");
}

#[test]
fn test_loading_size_xl() {
    let val = LoadingSize::Xl;
    assert_eq!(val.as_str(), "loading-xl");
}

#[test]
fn test_loading_size_clone() {
    let v1 = LoadingSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_loading_size_debug() {
    let val = LoadingSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage tests
#[test]
fn test_all_loading_colors_return_valid_classes() {
    let variants = vec![
        (LoadingColor::Default, ""),
        (LoadingColor::Neutral, "text-neutral"),
        (LoadingColor::Primary, "text-primary"),
        (LoadingColor::Secondary, "text-secondary"),
        (LoadingColor::Accent, "text-accent"),
        (LoadingColor::Info, "text-info"),
        (LoadingColor::Success, "text-success"),
        (LoadingColor::Warning, "text-warning"),
        (LoadingColor::Error, "text-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_loading_types_return_valid_classes() {
    let variants = vec![
        (LoadingType::Spinner, "loading-spinner"),
        (LoadingType::Dots, "loading-dots"),
        (LoadingType::Ring, "loading-ring"),
        (LoadingType::Ball, "loading-ball"),
        (LoadingType::Bars, "loading-bars"),
        (LoadingType::Infinity, "loading-infinity"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_loading_sizes_return_valid_classes() {
    let variants = vec![
        (LoadingSize::Xs, "loading-xs"),
        (LoadingSize::Sm, "loading-sm"),
        (LoadingSize::Md, "loading-md"),
        (LoadingSize::Lg, "loading-lg"),
        (LoadingSize::Xl, "loading-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
