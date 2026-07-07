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

// InputType tests (ldui-jcs.17)
#[test]
fn test_input_type_default_is_text() {
    assert_eq!(InputType::default(), InputType::Text);
    assert_eq!(InputType::default().as_str(), "text");
}

#[test]
fn test_all_input_types_return_valid_html_type_values() {
    let variants = vec![
        (InputType::Text, "text"),
        (InputType::Password, "password"),
        (InputType::Email, "email"),
        (InputType::Number, "number"),
        (InputType::Tel, "tel"),
        (InputType::Search, "search"),
        (InputType::Url, "url"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_input_type_clone_and_eq() {
    let v1 = InputType::Password;
    let v2 = v1.clone();
    assert_eq!(v1, v2);
    assert_ne!(InputType::Password, InputType::Text);
}

#[test]
fn test_input_type_debug() {
    let val = InputType::Email;
    assert!(format!("{:?}", val).contains("Email"));
}

// InputFilter tests (ldui-jcs.17) -- pure logic, TDD per conventions.md
#[test]
fn test_input_filter_default_is_none() {
    assert_eq!(InputFilter::default(), InputFilter::None);
}

#[test]
fn test_input_filter_none_permits_everything() {
    let f = InputFilter::None;
    for c in ['a', 'Z', '5', '-', ' ', '@', '#'] {
        assert!(f.permits(c), "None filter should permit {c:?}");
    }
}

#[test]
fn test_input_filter_none_apply_is_identity() {
    assert_eq!(
        InputFilter::None.apply("Hello, World! 123"),
        "Hello, World! 123"
    );
}

#[test]
fn test_input_filter_numeric_permits_digits_only() {
    let f = InputFilter::Numeric;
    assert!(f.permits('0'));
    assert!(f.permits('9'));
    assert!(!f.permits('a'));
    assert!(!f.permits('-'));
    assert!(!f.permits('.'));
    assert!(!f.permits(' '));
}

#[test]
fn test_input_filter_numeric_apply_strips_non_digits() {
    assert_eq!(InputFilter::Numeric.apply("-12.3ab,4"), "1234");
    assert_eq!(InputFilter::Numeric.apply("(555) 123-4567"), "5551234567");
}

#[test]
fn test_input_filter_phone_permits_digits_and_punctuation() {
    let f = InputFilter::Phone;
    for c in ['0', '9', '(', ')', '+', '-', ' '] {
        assert!(f.permits(c), "Phone filter should permit {c:?}");
    }
    assert!(!f.permits('a'));
    assert!(!f.permits('.'));
    assert!(!f.permits(','));
}

#[test]
fn test_input_filter_phone_apply_strips_letters() {
    assert_eq!(InputFilter::Phone.apply("(555) 12-a"), "(555) 12-");
    assert_eq!(
        InputFilter::Phone.apply("+1 (800) 555-0100"),
        "+1 (800) 555-0100"
    );
}

#[test]
fn test_input_filter_clone_copy_eq() {
    let f1 = InputFilter::Numeric;
    let f2 = f1;
    assert_eq!(f1, f2);
    assert_ne!(InputFilter::Numeric, InputFilter::Phone);
}

// optional_numeric_attr tests (backs the `maxlength` attribute plumbing,
// mirroring textarea's helper of the same name)
#[test]
fn test_optional_numeric_attr_none() {
    assert_eq!(super::component::optional_numeric_attr(None), None);
}

#[test]
fn test_optional_numeric_attr_some_value() {
    assert_eq!(
        super::component::optional_numeric_attr(Some(10)),
        Some("10".to_string())
    );
}
