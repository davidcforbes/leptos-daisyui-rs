//! Smoke tests for the modal row helper components. These exercise the
//! component builders to catch prop-name regressions and signature drift.
//! Full DOM-rendering assertions live in the demo / visual-regression
//! pass.

use super::*;
use leptos::prelude::*;

#[test]
fn modal_info_row_builds_with_label_and_children() {
    let _ = ModalInfoRow(ModalInfoRowProps {
        label: "Source:",
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "backup-2026-05-19" }),
    });
}

#[test]
fn modal_info_row_builds_without_label() {
    let _ = ModalInfoRow(ModalInfoRowProps {
        label: "",
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "value-only" }),
    });
}

#[test]
fn modal_search_row_builds() {
    let _ = ModalSearchRow(ModalSearchRowProps {
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { <input type="text" /> }),
    });
}

#[test]
fn modal_status_row_builds() {
    let _ = ModalStatusRow(ModalStatusRowProps {
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "3 matches" }),
    });
}

#[test]
fn modal_info_row_accepts_custom_class() {
    let _ = ModalInfoRow(ModalInfoRowProps {
        label: "X:",
        class: "custom-info",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "v" }),
    });
}

#[test]
fn modal_search_row_accepts_custom_class() {
    let _ = ModalSearchRow(ModalSearchRowProps {
        class: "custom-search",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "x" }),
    });
}

#[test]
fn modal_status_row_accepts_custom_class() {
    let _ = ModalStatusRow(ModalStatusRowProps {
        class: "custom-status",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "x" }),
    });
}

// ── modal_aria_label (accessible dialog naming, ldui-nui) ──

#[test]
fn labelled_by_suppresses_aria_label() {
    // aria-label would override aria-labelledby's visible heading.
    assert_eq!(modal_aria_label(Some("Reassign".into()), true), None);
    assert_eq!(modal_aria_label(None, true), None);
}

#[test]
fn explicit_label_is_used_verbatim() {
    assert_eq!(
        modal_aria_label(Some("Reasignar expediente".into()), false),
        Some("Reasignar expediente".to_string())
    );
}

#[test]
fn no_naming_props_falls_back_to_generic_modal() {
    // An unnamed dialog is an axe violation; the legacy generic name is the
    // floor, not the goal — callers should pass label or labelled_by.
    assert_eq!(modal_aria_label(None, false), Some("Modal".to_string()));
}
