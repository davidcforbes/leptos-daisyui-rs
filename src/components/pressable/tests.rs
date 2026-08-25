//! Source-contract pins for the Pressable primitive. The component's whole
//! value is a handful of emitted invariants (marker, type, behavioral-only
//! classes); these read `component.rs` itself so a refactor that drops one
//! fails here rather than silently shipping an unauditable raw button.

const SRC: &str = include_str!("component.rs");

#[test]
fn pressable_emits_the_audit_marker() {
    // The `ldui-audit` drift sweep's `button-without-btn` rule recognizes
    // exactly this attribute (see `audit/src/drift.js`); the two must agree.
    assert!(
        SRC.contains(r#"data-pressable="true""#),
        "Pressable must carry the data-pressable marker the audit recognizes"
    );
}

#[test]
fn pressable_is_never_an_implicit_submit() {
    assert!(SRC.contains(r#"type="button""#));
}

#[test]
fn pressable_applies_behavioral_classes_without_btn_geometry() {
    // Exactly Button's behavioral classes, minus `.btn`.
    assert!(
        SRC.contains(r#""ld-eased ld-pressable ld-focus-ring""#),
        "the focus/press contract must be present"
    );
    // No base-class string may smuggle `.btn` geometry in: the only
    // double-quote-adjacent `btn` in the source must be inside the
    // behavioral-class string above (which contains none). The caller's
    // `class` prop is the only styling channel.
    assert!(
        !SRC.contains("\"btn"),
        "Pressable is unstyled by design: it must never emit .btn"
    );
}

#[test]
fn pressable_wires_disabled_and_callback() {
    assert!(SRC.contains("disabled=disabled"));
    assert!(SRC.contains("callback.run(ev)"));
}
