use super::*;

// CollapseModifier tests
#[test]
fn test_collapse_modifier_default() {
    let modifier = CollapseModifier::default();
    assert_eq!(modifier.as_str(), "");
}

#[test]
fn test_collapse_modifier_arrow() {
    let modifier = CollapseModifier::Arrow;
    assert_eq!(modifier.as_str(), "collapse-arrow");
}

#[test]
fn test_collapse_modifier_plus() {
    let modifier = CollapseModifier::Plus;
    assert_eq!(modifier.as_str(), "collapse-plus");
}

#[test]
fn test_collapse_modifier_clone() {
    let modifier1 = CollapseModifier::Arrow;
    let modifier2 = modifier1.clone();
    assert_eq!(modifier1.as_str(), modifier2.as_str());
}

#[test]
fn test_collapse_modifier_debug() {
    let modifier = CollapseModifier::Plus;
    assert!(format!("{:?}", modifier).contains("Plus"));
}

// CollapseForceModifier tests
#[test]
fn test_collapse_force_modifier_default() {
    let force = CollapseForceModifier::default();
    assert_eq!(force.as_str(), "");
}

#[test]
fn test_collapse_force_modifier_open() {
    let force = CollapseForceModifier::Open;
    assert_eq!(force.as_str(), "collapse-open");
}

#[test]
fn test_collapse_force_modifier_close() {
    let force = CollapseForceModifier::Close;
    assert_eq!(force.as_str(), "collapse-close");
}

#[test]
fn test_collapse_force_modifier_clone() {
    let force1 = CollapseForceModifier::Open;
    let force2 = force1.clone();
    assert_eq!(force1.as_str(), force2.as_str());
}

#[test]
fn test_collapse_force_modifier_debug() {
    let force = CollapseForceModifier::Close;
    assert!(format!("{:?}", force).contains("Close"));
}

// Comprehensive enum variant coverage tests
#[test]
fn test_all_collapse_modifiers_return_valid_classes() {
    let modifiers = vec![
        (CollapseModifier::Default, ""),
        (CollapseModifier::Arrow, "collapse-arrow"),
        (CollapseModifier::Plus, "collapse-plus"),
    ];

    for (modifier, expected) in modifiers {
        assert_eq!(modifier.as_str(), expected);
    }
}

#[test]
fn test_all_collapse_force_modifiers_return_valid_classes() {
    let forces = vec![
        (CollapseForceModifier::Default, ""),
        (CollapseForceModifier::Open, "collapse-open"),
        (CollapseForceModifier::Close, "collapse-close"),
    ];

    for (force, expected) in forces {
        assert_eq!(force.as_str(), expected);
    }
}

// -- toggle identity + accessible name (ldui-3k00) ---------------------------

#[test]
fn identity_uses_explicit_id_and_defaults_name_to_it() {
    let identity = resolve_collapse_identity(Some("filters".into()), None, || {
        panic!("must not mint when an id is supplied")
    });
    assert_eq!(identity.id, "filters");
    assert_eq!(identity.name, "filters");
    assert_eq!(identity.title_id, "filters-title");
}

#[test]
fn identity_keeps_an_explicit_form_name_verbatim() {
    let identity =
        resolve_collapse_identity(Some("filters".into()), Some("show_filters".into()), || {
            panic!("must not mint when an id is supplied")
        });
    assert_eq!(identity.id, "filters");
    assert_eq!(identity.name, "show_filters");
}

#[test]
fn identity_mints_an_id_when_none_is_supplied() {
    let identity = resolve_collapse_identity(None, None, || "minted-7".to_string());
    assert_eq!(identity.id, "minted-7");
    assert_eq!(identity.name, "minted-7");
    assert_eq!(identity.title_id, "minted-7-title");
}

#[test]
fn minted_ids_are_prefixed_and_unique() {
    let a = next_collapse_id();
    let b = next_collapse_id();
    assert!(a.starts_with("ld-collapse-"), "{a}");
    assert!(b.starts_with("ld-collapse-"), "{b}");
    assert_ne!(a, b);
}

/// An explicit `aria_label` names the toggle directly and suppresses the
/// `aria-labelledby` reference, so the two can never disagree (an
/// `aria-labelledby` would otherwise win over `aria-label` in the
/// accessible-name computation and silently discard the explicit label).
#[test]
fn naming_prefers_an_explicit_aria_label() {
    let naming = collapse_input_naming(Some("Show filters".into()), "filters-title");
    assert_eq!(naming.aria_label.as_deref(), Some("Show filters"));
    assert_eq!(naming.labelled_by, None);
}

/// With no explicit label the toggle is named by the visible title, by id.
#[test]
fn naming_defaults_to_the_title_element() {
    let naming = collapse_input_naming(None, "filters-title");
    assert_eq!(naming.aria_label, None);
    assert_eq!(naming.labelled_by.as_deref(), Some("filters-title"));
}
