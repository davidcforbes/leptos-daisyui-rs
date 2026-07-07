use super::*;
use super::component::optional_numeric_attr;

// optional_numeric_attr tests (backs the `rows` / `maxlength` attribute plumbing)
#[test]
fn test_optional_numeric_attr_none() {
    assert_eq!(optional_numeric_attr(None), None);
}

#[test]
fn test_optional_numeric_attr_some_zero() {
    assert_eq!(optional_numeric_attr(Some(0)), Some("0".to_string()));
}

#[test]
fn test_optional_numeric_attr_some_value() {
    assert_eq!(optional_numeric_attr(Some(4)), Some("4".to_string()));
}

#[test]
fn test_optional_numeric_attr_large_value() {
    assert_eq!(optional_numeric_attr(Some(500)), Some("500".to_string()));
}

// TextareaColor tests
#[test]
fn test_textarea_color_default() {
    let color = TextareaColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_textarea_color_primary() {
    assert_eq!(TextareaColor::Primary.as_str(), "textarea-primary");
}

#[test]
fn test_textarea_color_secondary() {
    assert_eq!(TextareaColor::Secondary.as_str(), "textarea-secondary");
}

#[test]
fn test_textarea_color_accent() {
    assert_eq!(TextareaColor::Accent.as_str(), "textarea-accent");
}

#[test]
fn test_textarea_color_info() {
    assert_eq!(TextareaColor::Info.as_str(), "textarea-info");
}

#[test]
fn test_textarea_color_success() {
    assert_eq!(TextareaColor::Success.as_str(), "textarea-success");
}

#[test]
fn test_textarea_color_warning() {
    assert_eq!(TextareaColor::Warning.as_str(), "textarea-warning");
}

#[test]
fn test_textarea_color_error() {
    assert_eq!(TextareaColor::Error.as_str(), "textarea-error");
}

#[test]
fn test_textarea_color_clone() {
    let c1 = TextareaColor::Primary;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_textarea_color_debug() {
    let color = TextareaColor::Info;
    assert!(format!("{:?}", color).contains("Info"));
}

// TextareaSize tests
#[test]
fn test_textarea_size_default() {
    let size = TextareaSize::default();
    assert_eq!(size.as_str(), "textarea-md");
}

#[test]
fn test_textarea_size_xs() {
    assert_eq!(TextareaSize::Xs.as_str(), "textarea-xs");
}

#[test]
fn test_textarea_size_sm() {
    assert_eq!(TextareaSize::Sm.as_str(), "textarea-sm");
}

#[test]
fn test_textarea_size_md() {
    assert_eq!(TextareaSize::Md.as_str(), "textarea-md");
}

#[test]
fn test_textarea_size_lg() {
    assert_eq!(TextareaSize::Lg.as_str(), "textarea-lg");
}

#[test]
fn test_textarea_size_xl() {
    assert_eq!(TextareaSize::Xl.as_str(), "textarea-xl");
}

#[test]
fn test_textarea_size_clone() {
    let s1 = TextareaSize::Lg;
    let s2 = s1.clone();
    assert_eq!(s1.as_str(), s2.as_str());
}

#[test]
fn test_textarea_size_debug() {
    let size = TextareaSize::Sm;
    assert!(format!("{:?}", size).contains("Sm"));
}

#[test]
fn test_all_textarea_colors_return_valid_classes() {
    let variants = vec![
        (TextareaColor::Default, ""),
        (TextareaColor::Primary, "textarea-primary"),
        (TextareaColor::Secondary, "textarea-secondary"),
        (TextareaColor::Accent, "textarea-accent"),
        (TextareaColor::Info, "textarea-info"),
        (TextareaColor::Success, "textarea-success"),
        (TextareaColor::Warning, "textarea-warning"),
        (TextareaColor::Error, "textarea-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_textarea_sizes_return_valid_classes() {
    let variants = vec![
        (TextareaSize::Xs, "textarea-xs"),
        (TextareaSize::Sm, "textarea-sm"),
        (TextareaSize::Md, "textarea-md"),
        (TextareaSize::Lg, "textarea-lg"),
        (TextareaSize::Xl, "textarea-xl"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
