use super::*;

// FieldState tests
#[test]
fn test_field_state_default() {
    let val = FieldState::default();
    assert_eq!(val.as_str(), "");
}

#[test]
fn test_field_state_error() {
    let val = FieldState::Error;
    assert_eq!(val.as_str(), "error");
}

#[test]
fn test_field_state_success() {
    let val = FieldState::Success;
    assert_eq!(val.as_str(), "success");
}

#[test]
fn test_field_state_warning() {
    let val = FieldState::Warning;
    assert_eq!(val.as_str(), "warning");
}

#[test]
fn test_field_state_clone() {
    let v1 = FieldState::Error;
    let v2 = v1.clone();
    assert_eq!(v1.as_str(), v2.as_str());
}

#[test]
fn test_field_state_debug() {
    let val = FieldState::Success;
    assert!(format!("{:?}", val).contains("Success"));
}

// Comprehensive coverage test
#[test]
fn test_all_field_states_return_valid_classes() {
    let variants = vec![
        (FieldState::Default, ""),
        (FieldState::Error, "error"),
        (FieldState::Success, "success"),
        (FieldState::Warning, "warning"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
