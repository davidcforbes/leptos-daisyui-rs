use super::*;
use crate::components::{
    MessageChannel, MessageDirection, MessageHeader, RichMessage, entity_row_action_label,
};

fn picture(alt: &str) -> MessageHeader {
    MessageHeader::Image {
        src: "/flyer.png".into(),
        alt: alt.into(),
        width: Some(480),
        height: Some(360),
    }
}

fn version(
    label: &str,
    body: &str,
    header: Option<MessageHeader>,
    template: Option<&str>,
) -> MessageVersion {
    MessageVersion {
        label: label.into(),
        sent: RichMessage {
            header: header.clone(),
            body: body.into(),
            author: Some("Should be dropped".into()),
            direction: MessageDirection::Preview,
            ..Default::default()
        },
        template: template.map(|t| RichMessage {
            header,
            body: t.into(),
            ..Default::default()
        }),
    }
}

fn campaign(channel: MessageChannel) -> MessageViewerData {
    MessageViewerData {
        channel,
        sender_name: "Diener Law".into(),
        whatsapp_template: true,
        text_only_variant: true,
        versions: vec![
            version(
                "EN",
                "Hi Ana, see you Tuesday.",
                Some(picture("Office flyer")),
                Some("Hi {{first_name}}, see you {{day}}."),
            ),
            version(
                "ES",
                "Hola Ana, nos vemos el martes.",
                Some(picture("Folleto")),
                None,
            ),
        ],
        ..Default::default()
    }
}

fn select(version: usize, with_picture: bool, mode: MessageViewerMode) -> MessageViewerSelection {
    MessageViewerSelection {
        version,
        with_picture,
        mode,
    }
}

fn shown(data: &MessageViewerData, selection: MessageViewerSelection) -> RichMessage {
    viewer_message(data, selection, &MessageViewerTexts::default()).expect("a message")
}

// ---- which message is shown --------------------------------------------

#[test]
fn the_initial_selection_shows_the_first_version_as_sent_with_its_picture() {
    let data = campaign(MessageChannel::Whatsapp);
    let m = shown(&data, MessageViewerSelection::initial());
    assert_eq!(m.body, "Hi Ana, see you Tuesday.");
    assert_eq!(m.header, Some(picture("Office flyer")));
}

#[test]
fn every_shown_message_is_laid_out_as_sent_without_an_author() {
    let data = campaign(MessageChannel::Whatsapp);
    let m = shown(&data, MessageViewerSelection::initial());
    assert_eq!(m.direction, MessageDirection::Out);
    assert_eq!(
        m.author, None,
        "the chrome names the sender, not the bubble"
    );
}

#[test]
fn the_language_switch_selects_a_version_and_an_out_of_range_index_clamps() {
    let data = campaign(MessageChannel::Whatsapp);
    assert_eq!(
        shown(&data, select(1, true, MessageViewerMode::AsSent)).body,
        "Hola Ana, nos vemos el martes."
    );
    assert_eq!(
        shown(&data, select(9, true, MessageViewerMode::AsSent)).body,
        "Hola Ana, nos vemos el martes."
    );
    assert_eq!(
        selected_index(&data, select(9, true, MessageViewerMode::AsSent)),
        1
    );
}

#[test]
fn no_versions_shows_nothing() {
    let data = MessageViewerData::default();
    assert_eq!(
        viewer_message(
            &data,
            MessageViewerSelection::initial(),
            &MessageViewerTexts::default()
        ),
        None
    );
    assert_eq!(selected_index(&data, MessageViewerSelection::initial()), 0);
}

#[test]
fn template_mode_shows_the_template_with_placeholders_recognised() {
    let data = campaign(MessageChannel::Whatsapp);
    let s = select(0, true, MessageViewerMode::Template);
    assert_eq!(shown(&data, s).body, "Hi {{first_name}}, see you {{day}}.");
    assert_eq!(viewer_placeholders(&data, s), Some(Default::default()));
}

#[test]
fn as_sent_never_recognises_placeholders() {
    let data = campaign(MessageChannel::Whatsapp);
    assert_eq!(
        viewer_placeholders(&data, MessageViewerSelection::initial()),
        None
    );
}

#[test]
fn a_version_without_a_template_ignores_template_mode() {
    let data = campaign(MessageChannel::Whatsapp);
    let s = select(1, true, MessageViewerMode::Template);
    assert!(!offers_template(&data, s));
    assert_eq!(shown(&data, s).body, "Hola Ana, nos vemos el martes.");
    assert_eq!(viewer_placeholders(&data, s), None);
}

#[test]
fn text_only_drops_the_picture_only_where_the_variant_is_offered() {
    let mut data = campaign(MessageChannel::Whatsapp);
    let s = select(0, false, MessageViewerMode::AsSent);
    assert!(offers_text_only(&data, s));
    assert_eq!(shown(&data, s).header, None);

    data.text_only_variant = false;
    assert!(!offers_text_only(&data, s));
    assert_eq!(
        shown(&data, s).header,
        Some(picture("Office flyer")),
        "not offered: the picture stays"
    );
}

#[test]
fn a_version_without_a_picture_offers_no_text_only_switch() {
    let mut data = campaign(MessageChannel::Whatsapp);
    data.versions[0].sent.header = None;
    assert!(!offers_text_only(&data, MessageViewerSelection::initial()));
}

// ---- channel and chrome -------------------------------------------------

#[test]
fn an_sms_campaign_is_mms_exactly_when_the_shown_message_has_a_picture() {
    let data = campaign(MessageChannel::Sms);
    assert_eq!(
        shown(&data, MessageViewerSelection::initial()).channel,
        MessageChannel::Mms
    );
    assert_eq!(
        shown(&data, select(0, false, MessageViewerMode::AsSent)).channel,
        MessageChannel::Sms,
        "the text-only copy of an MMS campaign was an SMS"
    );
    let mms = campaign(MessageChannel::Mms);
    assert_eq!(
        shown(&mms, select(0, false, MessageViewerMode::AsSent)).channel,
        MessageChannel::Sms
    );
}

#[test]
fn the_viewer_channel_overrides_each_versions_own() {
    let mut data = campaign(MessageChannel::Whatsapp);
    data.versions[0].sent.channel = MessageChannel::Sms;
    assert_eq!(
        shown(&data, MessageViewerSelection::initial()).channel,
        MessageChannel::Whatsapp
    );
}

#[test]
fn the_chrome_badge_names_template_session_or_mms() {
    let t = MessageViewerTexts::default();
    let mut data = campaign(MessageChannel::Whatsapp);
    let m = shown(&data, MessageViewerSelection::initial());
    assert_eq!(chrome_badge(&data, &m, &t).as_deref(), Some("Template"));
    data.whatsapp_template = false;
    assert_eq!(
        chrome_badge(&data, &m, &t).as_deref(),
        Some("Session message")
    );

    let sms = campaign(MessageChannel::Sms);
    let with_picture = shown(&sms, MessageViewerSelection::initial());
    assert_eq!(
        chrome_badge(&sms, &with_picture, &t).as_deref(),
        Some("MMS")
    );
    let text_only = shown(&sms, select(0, false, MessageViewerMode::AsSent));
    assert_eq!(chrome_badge(&sms, &text_only, &t), None);
}

// ---- picture alt ----------------------------------------------------------

#[test]
fn the_hosts_alt_is_kept() {
    let data = campaign(MessageChannel::Whatsapp);
    match shown(&data, MessageViewerSelection::initial()).header {
        Some(MessageHeader::Image { alt, .. }) => assert_eq!(alt, "Office flyer"),
        other => panic!("expected an image header, got {other:?}"),
    }
}

#[test]
fn a_picture_without_alt_is_named_for_its_channel_in_both_languages() {
    let mut data = campaign(MessageChannel::Sms);
    data.versions[0].sent.header = Some(picture("   "));
    let alt = |texts: &MessageViewerTexts| match viewer_message(
        &data,
        MessageViewerSelection::initial(),
        texts,
    )
    .and_then(|m| m.header)
    {
        Some(MessageHeader::Image { alt, .. }) => alt,
        other => panic!("expected an image header, got {other:?}"),
    };
    assert_eq!(alt(&MessageViewerTexts::default()), "MMS picture");
    assert_eq!(alt(&MessageViewerTexts::es()), "Imagen de MMS");
    assert_eq!(
        picture_alt("", MessageChannel::Whatsapp, &MessageViewerTexts::default()),
        "WhatsApp picture"
    );
}

#[test]
fn the_bubble_says_the_picture_could_not_be_shown() {
    assert_eq!(
        MessageViewerTexts::default()
            .bubble_texts()
            .image_unavailable,
        "The picture could not be shown"
    );
    assert_eq!(
        MessageViewerTexts::es().bubble_texts().image_unavailable,
        "No se pudo mostrar la imagen"
    );
    assert_eq!(
        MessageViewerTexts::es().bubble_texts().sent,
        "Enviado",
        "the rest of the table is Spanish"
    );
}

// ---- details --------------------------------------------------------------

#[test]
fn the_reach_line_joins_whatever_counts_were_given() {
    let t = MessageViewerTexts::default();
    let all = MessageCounts {
        sent_to: 132,
        with_picture: Some(120),
        text_only: Some(12),
    };
    assert_eq!(
        counts_line(all, &t),
        "Sent to 132 · with picture 120 · text only 12"
    );
    assert_eq!(
        counts_line(
            MessageCounts {
                sent_to: 5,
                ..Default::default()
            },
            &t
        ),
        "Sent to 5"
    );
    assert_eq!(
        counts_line(all, &MessageViewerTexts::es()),
        "Enviado a 132 · con imagen 120 · solo texto 12"
    );
}

#[test]
fn every_status_has_a_distinct_marker_and_a_label_in_both_languages() {
    let markers: std::collections::HashSet<_> =
        DeliveryStatus::ALL.iter().map(|s| s.as_str()).collect();
    assert_eq!(markers.len(), DeliveryStatus::ALL.len());
    let en = MessageViewerTexts::default();
    let es = MessageViewerTexts::es();
    for status in DeliveryStatus::ALL {
        assert!(!status.label(&en).is_empty(), "{status:?}");
        assert_ne!(
            status.label(&en),
            status.label(&es),
            "{status:?} is not translated"
        );
    }
    assert_eq!(DeliveryStatus::NoReceipt.as_str(), "no-receipt");
}

// ---- row action -----------------------------------------------------------

#[test]
fn the_row_action_is_named_view_message_in_both_languages() {
    let en = view_message_row_texts(&MessageViewerTexts::default());
    let es = view_message_row_texts(&MessageViewerTexts::es());
    assert_eq!(
        entity_row_action_label(&en.open, "Spring reminder"),
        "View message: Spring reminder"
    );
    assert_eq!(
        entity_row_action_label(&es.open, "Recordatorio"),
        "Ver mensaje: Recordatorio"
    );
    assert_eq!(en.delete, "Delete {name}", "only the open label changes");
}

// ---- focus trap -----------------------------------------------------------

#[test]
fn tab_wraps_from_the_last_control_to_the_first_and_back() {
    assert_eq!(focus_wrap_target(4, Some(3), false), Some(0));
    assert_eq!(focus_wrap_target(4, Some(0), true), Some(3));
    assert_eq!(
        focus_wrap_target(4, Some(1), false),
        None,
        "inside: the browser moves"
    );
    assert_eq!(focus_wrap_target(4, Some(2), true), None);
}

#[test]
fn tab_from_outside_the_controls_enters_at_the_near_end() {
    assert_eq!(focus_wrap_target(4, None, false), Some(0));
    assert_eq!(focus_wrap_target(4, None, true), Some(3));
    assert_eq!(focus_wrap_target(0, None, false), None, "nothing to focus");
    assert_eq!(focus_wrap_target(1, Some(0), false), Some(0));
}
