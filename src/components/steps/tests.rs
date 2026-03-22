use super::*;

// StepsDirection tests
#[test]
fn test_steps_direction_default() {
    let dir = StepsDirection::default();
    assert_eq!(dir.as_str(), "");
}

#[test]
fn test_steps_direction_horizontal() {
    assert_eq!(StepsDirection::Horizontal.as_str(), "");
}

#[test]
fn test_steps_direction_vertical() {
    assert_eq!(StepsDirection::Vertical.as_str(), "steps-vertical");
}

#[test]
fn test_steps_direction_clone() {
    let d1 = StepsDirection::Vertical;
    let d2 = d1.clone();
    assert_eq!(d1.as_str(), d2.as_str());
}

#[test]
fn test_steps_direction_debug() {
    let dir = StepsDirection::Vertical;
    assert!(format!("{:?}", dir).contains("Vertical"));
}

// StepColor tests
#[test]
fn test_step_color_default() {
    let color = StepColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_step_color_primary() {
    assert_eq!(StepColor::Primary.as_str(), "step-primary");
}

#[test]
fn test_step_color_secondary() {
    assert_eq!(StepColor::Secondary.as_str(), "step-secondary");
}

#[test]
fn test_step_color_accent() {
    assert_eq!(StepColor::Accent.as_str(), "step-accent");
}

#[test]
fn test_step_color_info() {
    assert_eq!(StepColor::Info.as_str(), "step-info");
}

#[test]
fn test_step_color_success() {
    assert_eq!(StepColor::Success.as_str(), "step-success");
}

#[test]
fn test_step_color_warning() {
    assert_eq!(StepColor::Warning.as_str(), "step-warning");
}

#[test]
fn test_step_color_error() {
    assert_eq!(StepColor::Error.as_str(), "step-error");
}

#[test]
fn test_step_color_clone() {
    let c1 = StepColor::Success;
    let c2 = c1.clone();
    assert_eq!(c1.as_str(), c2.as_str());
}

#[test]
fn test_step_color_debug() {
    let color = StepColor::Error;
    assert!(format!("{:?}", color).contains("Error"));
}

#[test]
fn test_all_steps_directions_return_valid_classes() {
    let variants = vec![
        (StepsDirection::Horizontal, ""),
        (StepsDirection::Vertical, "steps-vertical"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}

#[test]
fn test_all_step_colors_return_valid_classes() {
    let variants = vec![
        (StepColor::Default, ""),
        (StepColor::Primary, "step-primary"),
        (StepColor::Secondary, "step-secondary"),
        (StepColor::Accent, "step-accent"),
        (StepColor::Info, "step-info"),
        (StepColor::Success, "step-success"),
        (StepColor::Warning, "step-warning"),
        (StepColor::Error, "step-error"),
    ];
    for (variant, expected) in variants {
        assert_eq!(variant.as_str(), expected);
    }
}
