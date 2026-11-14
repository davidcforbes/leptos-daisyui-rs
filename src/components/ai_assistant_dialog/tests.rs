//! Native guards for `AiAssistantDialog` (ldui-kq3w). This crate has no SSR
//! renderer, so the markup contract is asserted against the source and the
//! pure helpers; the open/close and maximize behaviour needs a browser lane.

use super::component::{
    AI_ASSISTANT_DIALOG_CLASS, AI_ASSISTANT_DIALOG_MAXIMIZED_CLASS,
    AI_ASSISTANT_DIALOG_NORMAL_CLASS, AiAssistantDialogTexts, ai_assistant_dialog_box_class,
    ai_assistant_dialog_maximize_glyph,
};

const SRC: &str = include_str!("component.rs");

#[test]
fn normal_size_keeps_offices_floating_geometry() {
    // 4iiz-Office's ASSISTANT_DIALOG_CLASS / ASSISTANT_DIALOG_BOX_CLASS
    // (op-zm1ow.1): a host migrating to this component must not change size.
    assert_eq!(AI_ASSISTANT_DIALOG_CLASS, "modal-middle justify-items-end");
    for cls in [
        "h-[85vh]",
        "max-h-[85vh]",
        "w-[28rem]",
        "min-w-[24rem]",
        "max-w-[calc(100vw-2rem)]",
        "sm:mr-4",
    ] {
        assert!(
            AI_ASSISTANT_DIALOG_NORMAL_CLASS
                .split_whitespace()
                .any(|c| c == cls),
            "normal size lost `{cls}`"
        );
    }
}

#[test]
fn maximized_is_ninety_percent_of_the_window_and_centred() {
    let max = AI_ASSISTANT_DIALOG_MAXIMIZED_CLASS;
    for cls in [
        "w-[90vw]",
        "max-w-[90vw]",
        "h-[90vh]",
        "max-h-[90vh]",
        "justify-self-center",
    ] {
        assert!(
            max.split_whitespace().any(|c| c == cls),
            "maximized size lost `{cls}` (daisyUI's .modal-box caps itself at \
             32rem, so the max-w is load-bearing)"
        );
    }
    assert!(
        !max.contains("sm:mr-4"),
        "a centred box must not keep the docked right gutter"
    );
}

#[test]
fn box_class_switches_size_and_keeps_the_shared_frame() {
    let normal = ai_assistant_dialog_box_class(false);
    let wide = ai_assistant_dialog_box_class(true);
    for s in [&normal, &wide] {
        assert!(s.starts_with("modal-box "), "must stay a daisyUI modal-box");
        assert!(s.contains("flex flex-col") && s.contains("overflow-hidden"));
        assert!(s.contains("transition-[width,max-width,height,max-height]"));
    }
    assert!(normal.contains("w-[28rem]") && !normal.contains("w-[90vw]"));
    assert!(wide.contains("w-[90vw]") && !wide.contains("w-[28rem]"));
}

#[test]
fn glyph_points_where_the_left_edge_would_go() {
    // Docked RIGHT: at its normal size it grows leftwards.
    assert_eq!(ai_assistant_dialog_maximize_glyph(false), "<<");
    assert_eq!(ai_assistant_dialog_maximize_glyph(true), ">>");
}

#[test]
fn maximize_toggle_is_a_pressed_toggle_with_one_label() {
    for needle in [
        "attr:aria-pressed=move || maximized.get().to_string()",
        "attr:aria-label=move || texts.with(|t| t.maximize_label.clone())",
        "attr:data-ai-assistant-dialog-maximize=\"true\"",
    ] {
        assert!(SRC.contains(needle), "maximize toggle lost `{needle}`");
    }
}

#[test]
fn header_order_is_title_actions_maximize_close() {
    let header = SRC
        .split("<header")
        .nth(1)
        .and_then(|s| s.split("</header>").next())
        .expect("the title row must exist");
    let title = header.find("{move || title.get()}").unwrap();
    let actions = header.find("{header_actions}").unwrap();
    let maximize = header.find("data-ai-assistant-dialog-maximize").unwrap();
    let close = header.find("data-ai-assistant-dialog-close").unwrap();
    assert!(title < actions && actions < maximize && maximize < close);
}

#[test]
fn close_is_proposed_never_applied() {
    // Controlled mode: the close button proposes like Escape and the
    // backdrop do; only the host's `open` closes the dialog.
    assert!(SRC.contains("on_close_request=on_close_request"));
    assert!(SRC.contains("on_close_request.run(ModalCloseProposal::new("));
}

#[test]
fn close_glyph_is_inline_not_a_host_sprite_symbol() {
    // The icon sprite is host-owned; the showcase has no `close` symbol, so
    // an `<Icon name="close">` rendered an empty, unlabelled-looking button.
    let header = SRC.split("<header").nth(1).unwrap();
    assert!(
        !header.contains("<Icon"),
        "the chrome must not need a sprite"
    );
    assert!(header.contains("<path d=\"M18 6 6 18\" />"));
}

#[test]
fn texts_default_english_and_ship_spanish() {
    let en = AiAssistantDialogTexts::default();
    let es = AiAssistantDialogTexts::spanish();
    assert_eq!(en.maximize_label, "Expand the assistant");
    assert_ne!(en, es);
    assert!(!es.maximize_label.is_empty() && !es.close_label.is_empty());
}
