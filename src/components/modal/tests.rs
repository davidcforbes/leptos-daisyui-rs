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

// ── controlled close proposals (ldui-e0fw) ──

#[test]
fn backdrop_submit_is_a_backdrop_close() {
    assert_eq!(
        modal_submit_close_cause("dialog", true),
        Some(ModalCloseCause::Backdrop)
    );
}

#[test]
fn in_content_dialog_form_submit_is_its_own_cause() {
    // daisyUI's documented close button is a `method="dialog"` form inside
    // the modal box. It closes the dialog just as silently as Escape, so it
    // is proposed rather than left as a second undetected drift path.
    assert_eq!(
        modal_submit_close_cause("dialog", false),
        Some(ModalCloseCause::DialogForm)
    );
}

#[test]
fn a_real_form_submit_is_never_a_close() {
    // The regression this guards: vetoing every submit inside a modal would
    // silently break a login or search form the caller put there.
    assert_eq!(modal_submit_close_cause("post", false), None);
    assert_eq!(modal_submit_close_cause("get", false), None);
    assert_eq!(modal_submit_close_cause("post", true), None);
    assert_eq!(modal_submit_close_cause("", false), None);
}

#[test]
fn form_method_match_is_case_insensitive() {
    // `HTMLFormElement.method` is a limited-to-known-values reflection and
    // lowercases, but the helper is public and takes a plain string.
    assert_eq!(
        modal_submit_close_cause("DIALOG", false),
        Some(ModalCloseCause::DialogForm)
    );
}

#[test]
fn close_while_target_open_is_drift_only_when_controlled() {
    // Controlled + still-open target is the exact defect: the DOM closed
    // without the owner accepting it.
    assert!(modal_close_is_state_drift(true, true));
    // Accepted close — the component's own `close()` after open=false.
    assert!(!modal_close_is_state_drift(true, false));
    // Uncontrolled dialogs keep the native behaviour untouched.
    assert!(!modal_close_is_state_drift(false, true));
    assert!(!modal_close_is_state_drift(false, false));
}

#[test]
fn close_mode_marker_names_the_contract() {
    assert_eq!(modal_close_mode_attr(true), "controlled");
    assert_eq!(modal_close_mode_attr(false), "uncontrolled");
}

#[test]
fn close_causes_have_stable_distinct_slugs() {
    let slugs = [
        ModalCloseCause::Escape.as_str(),
        ModalCloseCause::Backdrop.as_str(),
        ModalCloseCause::DialogForm.as_str(),
    ];
    assert_eq!(slugs, ["escape", "backdrop", "dialog-form"]);
    let unique: std::collections::BTreeSet<_> = slugs.iter().collect();
    assert_eq!(unique.len(), slugs.len());
}

#[test]
fn proposal_carries_its_cause() {
    let proposal = ModalCloseProposal::new(ModalCloseCause::Escape);
    assert_eq!(proposal.cause, ModalCloseCause::Escape);
    assert_eq!(proposal, ModalCloseProposal::new(ModalCloseCause::Escape));
    assert_ne!(proposal, ModalCloseProposal::new(ModalCloseCause::Backdrop));
}

#[test]
fn backdrop_close_copy_defaults_to_daisyui_wording() {
    assert_eq!(ModalTexts::default().backdrop_close, "close");
}

#[test]
fn backdrop_builds_with_default_and_custom_texts() {
    let _ = ModalBackdrop(ModalBackdropProps {
        class: "",
        texts: Signal::stored(ModalTexts::default()),
        node_ref: NodeRef::new(),
    });
    let _ = ModalBackdrop(ModalBackdropProps {
        class: "custom-backdrop",
        texts: Signal::stored(ModalTexts {
            backdrop_close: "cerrar".to_owned(),
        }),
        node_ref: NodeRef::new(),
    });
}

#[test]
fn modal_builds_in_uncontrolled_and_controlled_modes() {
    let _ = Modal(ModalProps {
        open: Signal::stored(false),
        backdrop: Signal::stored(true),
        on_close_request: None,
        texts: Signal::stored(ModalTexts::default()),
        label: MaybeProp::default(),
        labelled_by: MaybeProp::default(),
        described_by: MaybeProp::default(),
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "uncontrolled" }),
    });

    let _ = Modal(ModalProps {
        open: Signal::stored(true),
        backdrop: Signal::stored(true),
        on_close_request: Some(Callback::new(|_proposal: ModalCloseProposal| {})),
        texts: Signal::stored(ModalTexts::default()),
        label: MaybeProp::from("Reassign matter".to_string()),
        labelled_by: MaybeProp::default(),
        described_by: MaybeProp::default(),
        class: "",
        node_ref: NodeRef::new(),
        children: ToChildren::to_children(|| view! { "controlled" }),
    });
}
