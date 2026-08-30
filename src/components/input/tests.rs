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
        (InputType::Date, "date"),
        (InputType::Time, "time"),
        (InputType::Month, "month"),
        (InputType::Week, "week"),
        (InputType::DateTimeLocal, "datetime-local"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

// Temporal InputType variants (ldui-z16). Each must emit the exact valid
// HTML `type` token -- verbatim, not a caller-facing format string. Value
// parsing/validation/timezone/min-max-step stay caller-owned (attr:min,
// attr:max, attr:step); these variants only decide the DOM `type`.
#[test]
fn test_input_type_date_emits_date_token() {
    assert_eq!(InputType::Date.as_str(), "date");
}

#[test]
fn test_input_type_time_emits_time_token() {
    assert_eq!(InputType::Time.as_str(), "time");
}

#[test]
fn test_input_type_month_emits_month_token() {
    assert_eq!(InputType::Month.as_str(), "month");
}

#[test]
fn test_input_type_week_emits_week_token() {
    assert_eq!(InputType::Week.as_str(), "week");
}

#[test]
fn test_input_type_datetime_local_emits_datetime_local_token() {
    // The one variant whose token is not a lowercased rename of the Rust
    // identifier -- `DateTimeLocal` -> `datetime-local`, hyphenated per the
    // HTML spec, not `datetimelocal`.
    assert_eq!(InputType::DateTimeLocal.as_str(), "datetime-local");
}

#[test]
fn test_temporal_input_types_clone_and_eq() {
    let v1 = InputType::DateTimeLocal;
    let v2 = v1.clone();
    assert_eq!(v1, v2);
    assert_ne!(InputType::Date, InputType::Time);
    assert_ne!(InputType::Month, InputType::Week);
}

#[test]
fn test_temporal_input_types_debug() {
    assert!(format!("{:?}", InputType::Date).contains("Date"));
    assert!(format!("{:?}", InputType::DateTimeLocal).contains("DateTimeLocal"));
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

// resolve_effective_type tests (ldui-z16) -- the DOM `type` attribute Input
// actually applies, extracted for native (host-target) testability, mirroring
// Button's `resolve_native_disabled` (ldui-9vs). Only `InputType::Password`
// is affected by the reveal toggle; every other variant -- including all
// five temporal variants -- must pass through unchanged regardless of
// `revealable`/`revealed`.
use super::component::resolve_effective_type;

#[test]
fn test_resolve_effective_type_password_not_revealed_stays_password() {
    assert_eq!(
        resolve_effective_type(InputType::Password, true, false),
        "password"
    );
}

#[test]
fn test_resolve_effective_type_password_revealed_flips_to_text() {
    assert_eq!(
        resolve_effective_type(InputType::Password, true, true),
        "text"
    );
}

#[test]
fn test_resolve_effective_type_password_revealed_but_not_revealable_stays_password() {
    // `revealed` alone (internal reveal signal) never flips the type -- only
    // when the caller opted in via `revealable=true` too.
    assert_eq!(
        resolve_effective_type(InputType::Password, false, true),
        "password"
    );
}

#[test]
fn test_resolve_effective_type_temporal_variants_are_never_affected_by_reveal() {
    for (variant, expected) in [
        (InputType::Date, "date"),
        (InputType::Time, "time"),
        (InputType::Month, "month"),
        (InputType::Week, "week"),
        (InputType::DateTimeLocal, "datetime-local"),
    ] {
        for &revealable in &[false, true] {
            for &revealed in &[false, true] {
                assert_eq!(
                    resolve_effective_type(variant.clone(), revealable, revealed),
                    expected,
                    "variant={variant:?}, revealable={revealable}, revealed={revealed}"
                );
            }
        }
    }
}

#[test]
fn test_resolve_effective_type_non_password_non_temporal_variants_pass_through() {
    for (variant, expected) in [
        (InputType::Text, "text"),
        (InputType::Email, "email"),
        (InputType::Number, "number"),
        (InputType::Tel, "tel"),
        (InputType::Search, "search"),
        (InputType::Url, "url"),
    ] {
        assert_eq!(
            resolve_effective_type(variant.clone(), true, true),
            expected
        );
    }
}
