use super::*;

/// The rendered view, for the backward-compatibility scans below. This crate
/// has no native DOM renderer, so the uncontrolled branch's markup is held in
/// place here and proved against the real DOM by
/// `reactivity_smoke.rs`'s `a_bare_checkbox_renders_exactly_what_it_always_did`.
const COMPONENT_SRC: &str = include_str!("component.rs");

/// The uncontrolled branch's source, between the two markers `state.rs` also
/// splits on.
fn uncontrolled_branch() -> &'static str {
    COMPONENT_SRC
        .split("// ── uncontrolled ──")
        .nth(1)
        .and_then(|rest| rest.split("// ── end uncontrolled ──").next())
        .expect("the uncontrolled branch must stay marked in the source")
}

#[test]
fn the_uncontrolled_branch_keeps_the_original_class_expression() {
    // The exact string the component emitted before `ldui-fqan`. A change here
    // is a change to every existing caller's rendered classes.
    assert!(
        COMPONENT_SRC.contains(r#""checkbox ld-eased ld-focus-ring","#),
        "the base class list must not drift"
    );
    assert!(uncontrolled_branch().contains("class=input_class"));
    assert!(
        uncontrolled_branch().contains(r#"type="checkbox""#),
        "the uncontrolled branch must stay a native checkbox input"
    );
}

#[test]
fn every_attribute_added_to_the_uncontrolled_branch_is_omitted_when_unset() {
    // Each of these is driven by a prop that defaults to "absent", so a
    // `<Checkbox />` that opts into none of them renders no new attribute at
    // all. Anything emitted unconditionally would change existing markup.
    for attribute in [
        "id=move || resolved_id.get()",
        "name=move || resolved_name.get()",
        "checked=move || default_checked.get().unwrap_or(false)",
        "aria-label=move || aria_label.get()",
        "aria-describedby=described_by",
        "aria-errormessage=error_message",
        "aria-invalid=aria_invalid",
    ] {
        assert!(
            uncontrolled_branch().contains(attribute),
            "the uncontrolled branch lost `{attribute}`"
        );
    }
    // `resolve_checkbox_id` returns `None` unless the caller opted in, and
    // `mint_when_absent` is exactly `has_label` -- so a bare checkbox mints
    // nothing. (The pure functions are proved in `state.rs`.)
    assert!(
        COMPONENT_SRC
            .contains("resolve_checkbox_id(id.get(), field_id.clone(), has_label, &minted_id)"),
        "an id must be minted only when the component needs one for its own label"
    );
}

#[test]
fn the_visible_label_is_explicitly_associated_with_its_input() {
    // The daisyUI 4 wrappers that removal made no-ops are policed globally by
    // `tests/no_dead_daisyui4_classes.rs` -- naming them here would trip that
    // scan. What is specific to this component is the association itself:
    // labelling a checkbox is exactly where the dead idiom tempts you back,
    // and a wrapper that only *looks* like a label associates nothing.
    assert!(
        COMPONENT_SRC.contains("r#for=move || resolved_id.get()"),
        "the visible label must be explicitly associated with the input"
    );
}

#[test]
fn muted_label_copy_uses_a_token_alpha_rather_than_opacity() {
    assert!(COMPONENT_SRC.contains("text-base-content/75"));
    assert!(
        !COMPONENT_SRC.contains("opacity-"),
        "muted copy must use `text-base-content/75`, never an opacity utility"
    );
}

// CheckboxColor tests
#[test]
fn test_checkbox_color_default() {
    let color = CheckboxColor::default();
    assert_eq!(color.as_str(), "");
}

#[test]
fn test_checkbox_color_primary() {
    let color = CheckboxColor::Primary;
    assert_eq!(color.as_str(), "checkbox-primary");
}

#[test]
fn test_checkbox_color_secondary() {
    let color = CheckboxColor::Secondary;
    assert_eq!(color.as_str(), "checkbox-secondary");
}

#[test]
fn test_checkbox_color_accent() {
    let color = CheckboxColor::Accent;
    assert_eq!(color.as_str(), "checkbox-accent");
}

#[test]
fn test_checkbox_color_neutral() {
    let color = CheckboxColor::Neutral;
    assert_eq!(color.as_str(), "checkbox-neutral");
}

#[test]
fn test_checkbox_color_success() {
    let color = CheckboxColor::Success;
    assert_eq!(color.as_str(), "checkbox-success");
}

#[test]
fn test_checkbox_color_warning() {
    let color = CheckboxColor::Warning;
    assert_eq!(color.as_str(), "checkbox-warning");
}

#[test]
fn test_checkbox_color_info() {
    let color = CheckboxColor::Info;
    assert_eq!(color.as_str(), "checkbox-info");
}

#[test]
fn test_checkbox_color_error() {
    let color = CheckboxColor::Error;
    assert_eq!(color.as_str(), "checkbox-error");
}

#[test]
fn test_checkbox_color_clone() {
    let color1 = CheckboxColor::Primary;
    let color2 = color1.clone();
    assert_eq!(color1.as_str(), color2.as_str());
}

#[test]
fn test_checkbox_color_debug() {
    let color = CheckboxColor::Success;
    assert!(format!("{:?}", color).contains("Success"));
}

// CheckboxSize tests
#[test]
fn test_checkbox_size_default() {
    let size = CheckboxSize::default();
    assert_eq!(size.as_str(), "checkbox-md");
}

#[test]
fn test_checkbox_size_xs() {
    let size = CheckboxSize::Xs;
    assert_eq!(size.as_str(), "checkbox-xs");
}

#[test]
fn test_checkbox_size_sm() {
    let size = CheckboxSize::Sm;
    assert_eq!(size.as_str(), "checkbox-sm");
}

#[test]
fn test_checkbox_size_md() {
    let size = CheckboxSize::Md;
    assert_eq!(size.as_str(), "checkbox-md");
}

#[test]
fn test_checkbox_size_lg() {
    let size = CheckboxSize::Lg;
    assert_eq!(size.as_str(), "checkbox-lg");
}

#[test]
fn test_checkbox_size_xl() {
    let size = CheckboxSize::Xl;
    assert_eq!(size.as_str(), "checkbox-xl");
}

#[test]
fn test_checkbox_size_clone() {
    let size1 = CheckboxSize::Lg;
    let size2 = size1.clone();
    assert_eq!(size1.as_str(), size2.as_str());
}

#[test]
fn test_checkbox_size_debug() {
    let size = CheckboxSize::Xl;
    assert!(format!("{:?}", size).contains("Xl"));
}

// Comprehensive enum variant coverage tests
#[test]
fn test_all_checkbox_colors_return_valid_classes() {
    let colors = vec![
        (CheckboxColor::Default, ""),
        (CheckboxColor::Primary, "checkbox-primary"),
        (CheckboxColor::Secondary, "checkbox-secondary"),
        (CheckboxColor::Accent, "checkbox-accent"),
        (CheckboxColor::Neutral, "checkbox-neutral"),
        (CheckboxColor::Success, "checkbox-success"),
        (CheckboxColor::Warning, "checkbox-warning"),
        (CheckboxColor::Info, "checkbox-info"),
        (CheckboxColor::Error, "checkbox-error"),
    ];

    for (color, expected) in colors {
        assert_eq!(color.as_str(), expected);
    }
}

#[test]
fn test_all_checkbox_sizes_return_valid_classes() {
    let sizes = vec![
        (CheckboxSize::Xs, "checkbox-xs"),
        (CheckboxSize::Sm, "checkbox-sm"),
        (CheckboxSize::Md, "checkbox-md"),
        (CheckboxSize::Lg, "checkbox-lg"),
        (CheckboxSize::Xl, "checkbox-xl"),
    ];

    for (size, expected) in sizes {
        assert_eq!(size.as_str(), expected);
    }
}
