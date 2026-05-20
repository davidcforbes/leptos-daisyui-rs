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
