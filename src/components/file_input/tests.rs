use super::*;

// FileInputStyle tests
#[test]
fn test_file_input_style_default() {
    let val = FileInputStyle::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_file_input_style_ghost() {
    let val = FileInputStyle::Ghost;
    assert_eq!(val.as_str(), "file-input-ghost");
}

#[test]
fn test_file_input_style_clone() {
    let v1 = FileInputStyle::Ghost;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_file_input_style_debug() {
    let val = FileInputStyle::Ghost;
    assert!(format!("{:?}", val).contains("Ghost"));
}

// FileInputColor tests
#[test]
fn test_file_input_color_default() {
    let val = FileInputColor::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_file_input_color_neutral() {
    let val = FileInputColor::Neutral;
    assert_eq!(val.as_str(), "file-input-neutral");
}

#[test]
fn test_file_input_color_primary() {
    let val = FileInputColor::Primary;
    assert_eq!(val.as_str(), "file-input-primary");
}

#[test]
fn test_file_input_color_secondary() {
    let val = FileInputColor::Secondary;
    assert_eq!(val.as_str(), "file-input-secondary");
}

#[test]
fn test_file_input_color_accent() {
    let val = FileInputColor::Accent;
    assert_eq!(val.as_str(), "file-input-accent");
}

#[test]
fn test_file_input_color_info() {
    let val = FileInputColor::Info;
    assert_eq!(val.as_str(), "file-input-info");
}

#[test]
fn test_file_input_color_success() {
    let val = FileInputColor::Success;
    assert_eq!(val.as_str(), "file-input-success");
}

#[test]
fn test_file_input_color_warning() {
    let val = FileInputColor::Warning;
    assert_eq!(val.as_str(), "file-input-warning");
}

#[test]
fn test_file_input_color_error() {
    let val = FileInputColor::Error;
    assert_eq!(val.as_str(), "file-input-error");
}

#[test]
fn test_file_input_color_clone() {
    let v1 = FileInputColor::Primary;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_file_input_color_debug() {
    let val = FileInputColor::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// FileInputSize tests
#[test]
fn test_file_input_size_default() {
    let val = FileInputSize::default();
    assert_eq!(val.as_str(), "file-input-md");
}

#[test]
fn test_file_input_size_xs() {
    let val = FileInputSize::Xs;
    assert_eq!(val.as_str(), "file-input-xs");
}

#[test]
fn test_file_input_size_sm() {
    let val = FileInputSize::Sm;
    assert_eq!(val.as_str(), "file-input-sm");
}

#[test]
fn test_file_input_size_md() {
    let val = FileInputSize::Md;
    assert_eq!(val.as_str(), "file-input-md");
}

#[test]
fn test_file_input_size_lg() {
    let val = FileInputSize::Lg;
    assert_eq!(val.as_str(), "file-input-lg");
}

#[test]
fn test_file_input_size_xl() {
    let val = FileInputSize::Xl;
    assert_eq!(val.as_str(), "file-input-xl");
}

#[test]
fn test_file_input_size_clone() {
    let v1 = FileInputSize::Lg;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_file_input_size_debug() {
    let val = FileInputSize::Xl;
    assert!(format!("{:?}", val).contains("Xl"));
}

// Comprehensive coverage tests
#[test]
fn test_all_file_input_styles_return_valid_classes() {
    let variants = vec![
        (FileInputStyle::Default, ""),
        (FileInputStyle::Ghost, "file-input-ghost"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_file_input_colors_return_valid_classes() {
    let variants = vec![
        (FileInputColor::Default, ""),
        (FileInputColor::Neutral, "file-input-neutral"),
        (FileInputColor::Primary, "file-input-primary"),
        (FileInputColor::Secondary, "file-input-secondary"),
        (FileInputColor::Accent, "file-input-accent"),
        (FileInputColor::Info, "file-input-info"),
        (FileInputColor::Success, "file-input-success"),
        (FileInputColor::Warning, "file-input-warning"),
        (FileInputColor::Error, "file-input-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_file_input_sizes_return_valid_classes() {
    let variants = vec![
        (FileInputSize::Xs, "file-input-xs"),
        (FileInputSize::Sm, "file-input-sm"),
        (FileInputSize::Md, "file-input-md"),
        (FileInputSize::Lg, "file-input-lg"),
        (FileInputSize::Xl, "file-input-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
