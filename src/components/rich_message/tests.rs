use super::inline::{InlineStyle, Piece, Run, RunContent, inline_runs, split_placeholders};
use super::*;

fn text(style: InlineStyle, s: &str) -> Run {
    Run {
        style,
        content: RunContent::Text(s.to_string()),
    }
}

const PLAIN: InlineStyle = InlineStyle {
    bold: false,
    italic: false,
    strike: false,
    mono: false,
};
const BOLD: InlineStyle = InlineStyle {
    bold: true,
    ..PLAIN
};
const ITALIC: InlineStyle = InlineStyle {
    italic: true,
    ..PLAIN
};
const STRIKE: InlineStyle = InlineStyle {
    strike: true,
    ..PLAIN
};
const MONO: InlineStyle = InlineStyle {
    mono: true,
    ..PLAIN
};

fn whatsapp(body: &str) -> Vec<Run> {
    inline_runs(body, true, false, false)
}

// ---- SMS segments -------------------------------------------------------

#[test]
fn gsm7_160_is_one_segment_and_161_is_two() {
    let one = sms_segments(&"a".repeat(160));
    assert_eq!(one.encoding, SmsEncoding::Gsm7);
    assert_eq!((one.segments, one.units, one.chars), (1, 160, 160));
    let two = sms_segments(&"a".repeat(161));
    assert_eq!((two.segments, two.units), (2, 161));
}

#[test]
fn gsm7_multipart_packs_153_septets() {
    assert_eq!(sms_segments(&"a".repeat(306)).segments, 2);
    assert_eq!(sms_segments(&"a".repeat(307)).segments, 3);
}

#[test]
fn ucs2_70_is_one_segment_and_71_is_two() {
    let one = sms_segments(&"ж".repeat(70));
    assert_eq!(one.encoding, SmsEncoding::Ucs2);
    assert_eq!((one.segments, one.units), (1, 70));
    let two = sms_segments(&"ж".repeat(71));
    assert_eq!((two.segments, two.units), (2, 71));
    assert_eq!(sms_segments(&"ж".repeat(134)).segments, 2);
    assert_eq!(sms_segments(&"ж".repeat(135)).segments, 3);
}

#[test]
fn one_emoji_or_curly_quote_forces_ucs2() {
    assert_eq!(sms_segments("Hi ✅").encoding, SmsEncoding::Ucs2);
    assert_eq!(sms_segments("don’t").encoding, SmsEncoding::Ucs2);
    assert_eq!(sms_segments("don't").encoding, SmsEncoding::Gsm7);
    // Accented letters in the GSM alphabet stay GSM-7.
    assert_eq!(sms_segments("Ñoño é à ü ¿?").encoding, SmsEncoding::Gsm7);
}

#[test]
fn gsm7_extension_characters_cost_two_septets_and_never_split() {
    let euro = sms_segments("€");
    assert_eq!(
        (euro.encoding, euro.units, euro.chars),
        (SmsEncoding::Gsm7, 2, 1)
    );
    // 80 euro signs = 160 septets: still one segment.
    assert_eq!(sms_segments(&"€".repeat(80)).segments, 1);
    // 152 plain septets then a euro sign: the escape pair cannot straddle
    // the 153-septet boundary, so it moves to segment two.
    let body = format!("{}€{}", "a".repeat(152), "a".repeat(10));
    assert_eq!(sms_segments(&body).segments, 2);
    let body = format!("{}€{}", "a".repeat(152), "a".repeat(152));
    assert_eq!(sms_segments(&body).units, 306);
    assert_eq!(sms_segments(&body).segments, 3);
}

#[test]
fn ucs2_surrogate_pairs_count_two_units_and_never_split() {
    let emoji = sms_segments("😀");
    assert_eq!((emoji.units, emoji.chars, emoji.segments), (2, 1, 1));
    // 66 units then an emoji: the pair moves to the next segment.
    let body = format!("{}😀{}", "ж".repeat(66), "ж".repeat(10));
    assert_eq!(sms_segments(&body).units, 78);
    assert_eq!(sms_segments(&body).segments, 2);
}

#[test]
fn chars_counts_user_perceived_characters() {
    // Thumbs up + skin tone is one character, four UTF-16 units.
    let info = sms_segments("👍🏽");
    assert_eq!((info.chars, info.units), (1, 4));
}

#[test]
fn empty_body_is_zero_segments() {
    let info = sms_segments("");
    assert_eq!((info.segments, info.units, info.chars), (0, 0, 0));
}

#[test]
fn indicator_text_reads_encoding_segments_and_length() {
    let texts = RichMessageTexts::default();
    assert_eq!(
        sms_indicator_text(sms_segments(&"a".repeat(161)), false, &texts),
        "GSM-7 · 2 segments · 161 chars"
    );
    assert_eq!(
        sms_indicator_text(sms_segments("Hi ✅"), false, &texts),
        "UCS-2 · 1 segment · 4 chars"
    );
    assert_eq!(
        sms_indicator_text(sms_segments("Hi ✅"), true, &texts),
        "Picture message · 4 chars"
    );
    assert_eq!(
        sms_indicator_text(sms_segments("Hola"), false, &RichMessageTexts::es()),
        "GSM-7 · 1 segmento · 4 caracteres"
    );
}

#[test]
fn picture_message_is_an_mms_or_an_sms_with_media() {
    let mut message = RichMessage::default();
    assert!(!message.is_picture_message());
    message.header = Some(MessageHeader::Text("Hello".into()));
    assert!(!message.is_picture_message());
    message.header = Some(MessageHeader::Image {
        src: "a.png".into(),
        alt: String::new(),
        width: None,
        height: None,
    });
    assert!(message.is_picture_message());
    message.channel = MessageChannel::Whatsapp;
    assert!(!message.is_picture_message());
    let mms = RichMessage {
        channel: MessageChannel::Mms,
        ..Default::default()
    };
    assert!(mms.is_picture_message());
}

// ---- WhatsApp formatting ------------------------------------------------

#[test]
fn whatsapp_bold_italic_strike_and_mono() {
    assert_eq!(
        whatsapp("a *b* c"),
        vec![text(PLAIN, "a "), text(BOLD, "b"), text(PLAIN, " c")]
    );
    assert_eq!(whatsapp("_it_"), vec![text(ITALIC, "it")]);
    assert_eq!(whatsapp("~gone~"), vec![text(STRIKE, "gone")]);
    assert_eq!(whatsapp("```x *y* _z_```"), vec![text(MONO, "x *y* _z_")]);
}

#[test]
fn whatsapp_mono_spans_lines() {
    assert_eq!(whatsapp("```a\nb```"), vec![text(MONO, "a\nb")]);
}

#[test]
fn whatsapp_formats_nest() {
    assert_eq!(
        whatsapp("*_both_*"),
        vec![text(
            InlineStyle {
                italic: true,
                ..BOLD
            },
            "both"
        )]
    );
    assert_eq!(
        whatsapp("*bold _and italic_*"),
        vec![
            text(BOLD, "bold "),
            text(
                InlineStyle {
                    italic: true,
                    ..BOLD
                },
                "and italic"
            )
        ]
    );
}

#[test]
fn whatsapp_markers_need_word_boundaries() {
    // Inside a word: snake_case stays literal.
    assert_eq!(
        whatsapp("snake_case_name"),
        vec![text(PLAIN, "snake_case_name")]
    );
    // A space after the opener or before the closer.
    assert_eq!(whatsapp("* not bold*"), vec![text(PLAIN, "* not bold*")]);
    assert_eq!(whatsapp("*not bold *"), vec![text(PLAIN, "*not bold *")]);
    // Arithmetic.
    assert_eq!(whatsapp("2*3*4"), vec![text(PLAIN, "2*3*4")]);
    // Empty and unclosed markers.
    assert_eq!(whatsapp("** _"), vec![text(PLAIN, "** _")]);
    assert_eq!(whatsapp("*open"), vec![text(PLAIN, "*open")]);
    assert_eq!(whatsapp("```open"), vec![text(PLAIN, "```open")]);
    // Punctuation is a boundary.
    assert_eq!(
        whatsapp("(*yes*)."),
        vec![text(PLAIN, "("), text(BOLD, "yes"), text(PLAIN, ").")]
    );
}

#[test]
fn whatsapp_markers_do_not_cross_lines() {
    assert_eq!(whatsapp("*a\nb*"), vec![text(PLAIN, "*a\nb*")]);
}

#[test]
fn sms_bodies_are_never_formatted() {
    assert_eq!(
        inline_runs("*not bold* _x_", false, false, false),
        vec![text(PLAIN, "*not bold* _x_")]
    );
}

#[test]
fn extended_characters_pass_through_unchanged() {
    let body = "✅ I’m a lifelong Democrat\n\n✅ ¿Cómo está? 你好 👍🏽";
    assert_eq!(whatsapp(body), vec![text(PLAIN, body)]);
    assert_eq!(
        inline_runs(body, false, true, true),
        vec![text(PLAIN, body)]
    );
}

// ---- Placeholders -------------------------------------------------------

fn placeholder(raw: &str, key: &str) -> Piece {
    Piece::Placeholder {
        raw: raw.into(),
        key: key.into(),
    }
}

#[test]
fn all_three_placeholder_syntaxes_are_recognised() {
    assert_eq!(
        split_placeholders("Hi {{first_name}}, ref {{1}} for ${Contacts.First Name}!"),
        vec![
            Piece::Text("Hi ".into()),
            placeholder("{{first_name}}", "first_name"),
            Piece::Text(", ref ".into()),
            placeholder("{{1}}", "1"),
            Piece::Text(" for ".into()),
            placeholder("${Contacts.First Name}", "Contacts.First Name"),
            Piece::Text("!".into()),
        ]
    );
}

#[test]
fn placeholder_keys_are_trimmed() {
    assert_eq!(
        split_placeholders("{{ name }}"),
        vec![placeholder("{{ name }}", "name")]
    );
}

#[test]
fn malformed_placeholders_stay_text() {
    for body in [
        "{{}}",
        "{{ }}",
        "{{open",
        "${open",
        "{{a\nb}}",
        "costs $5 {not}",
        "{x}",
    ] {
        assert_eq!(
            split_placeholders(body),
            vec![Piece::Text(body.into())],
            "{body:?}"
        );
    }
}

#[test]
fn placeholder_underscores_never_open_italics() {
    assert_eq!(
        inline_runs("_Hi {{first_name}}_ and {{last_name}}", true, true, false),
        vec![
            text(ITALIC, "Hi "),
            Run {
                style: ITALIC,
                content: RunContent::Placeholder {
                    raw: "{{first_name}}".into(),
                    key: "first_name".into()
                }
            },
            text(PLAIN, " and "),
            Run {
                style: PLAIN,
                content: RunContent::Placeholder {
                    raw: "{{last_name}}".into(),
                    key: "last_name".into()
                }
            },
        ]
    );
}

#[test]
fn placeholders_inside_bold_keep_the_style() {
    assert_eq!(
        inline_runs("*{{1}}*", true, true, false),
        vec![Run {
            style: BOLD,
            content: RunContent::Placeholder {
                raw: "{{1}}".into(),
                key: "1".into()
            }
        }]
    );
}

#[test]
fn placeholders_off_leaves_delimiters_as_text() {
    assert_eq!(
        inline_runs("Hi {{first_name}}", false, false, false),
        vec![text(PLAIN, "Hi {{first_name}}")]
    );
}

// ---- Links --------------------------------------------------------------

fn link(text: &str, href: &str) -> Run {
    Run {
        style: PLAIN,
        content: RunContent::Link {
            text: text.into(),
            href: href.into(),
        },
    }
}

#[test]
fn linkify_finds_urls_hosts_and_emails_and_keeps_punctuation() {
    assert_eq!(
        inline_runs(
            "Email sylvia@votelemus.com - or https://www.VoteLemus.com. (www.example.org)",
            false,
            false,
            true
        ),
        vec![
            text(PLAIN, "Email "),
            link("sylvia@votelemus.com", "mailto:sylvia@votelemus.com"),
            text(PLAIN, " - or "),
            link("https://www.VoteLemus.com", "https://www.VoteLemus.com"),
            text(PLAIN, ". ("),
            link("www.example.org", "https://www.example.org"),
            text(PLAIN, ")"),
        ]
    );
}

#[test]
fn linkify_is_off_unless_asked_and_skips_mono() {
    assert_eq!(
        inline_runs("see https://a.example", false, false, false),
        vec![text(PLAIN, "see https://a.example")]
    );
    assert_eq!(
        inline_runs("```https://a.example```", true, false, true),
        vec![text(MONO, "https://a.example")]
    );
}

#[test]
fn linkify_rejects_non_links() {
    for body in [
        "https://",
        "www.",
        "a@b",
        "@handle",
        "mail@.com",
        "javascript:alert(1)",
    ] {
        assert_eq!(
            inline_runs(body, false, false, true),
            vec![text(PLAIN, body)],
            "{body:?}"
        );
    }
}

// ---- Media, labels and actions -----------------------------------------

#[test]
fn media_fallback_names_alt_then_file_name() {
    assert_eq!(
        media_fallback_name("https://x/a/flyer.png", "Campaign flyer"),
        "Campaign flyer"
    );
    assert_eq!(
        media_fallback_name("https://x/a/flyer.png?sig=1#f", "  "),
        "flyer.png"
    );
    assert_eq!(media_fallback_name("https://x/a/", ""), "a");
    assert_eq!(media_fallback_name("", ""), "");
}

#[test]
fn header_links_allow_only_web_blob_and_relative_urls() {
    for ok in [
        "https://cdn.example.org/terms.pdf",
        "HTTP://example.org/a",
        "blob:https://app.example.org/1234",
        "/assets/terms.pdf",
        "terms.pdf",
        "//cdn.example.org/terms.pdf",
        "?download=1",
    ] {
        assert!(is_safe_link_url(ok), "{ok:?}");
    }
    for bad in [
        "javascript:alert(1)",
        " JavaScript:alert(1)",
        "java\tscript:alert(1)",
        "java\nscript:alert(1)",
        "\u{1}javascript:alert(1)",
        "vbscript:msgbox(1)",
        "data:text/html,<script>alert(1)</script>",
        "data:image/png;base64,AAAA",
        "file:///c:/secret",
        "",
        "   ",
    ] {
        assert!(!is_safe_link_url(bad), "{bad:?}");
    }
}

#[test]
fn media_urls_also_allow_inline_media_data_but_never_script() {
    assert!(is_safe_media_url("data:image/svg+xml;utf8,%3Csvg%3E"));
    assert!(is_safe_media_url("data:video/mp4;base64,AAAA"));
    assert!(is_safe_media_url("https://cdn.example.org/a.png"));
    assert!(is_safe_media_url("/missing/campaign-flyer.png"));
    assert!(!is_safe_media_url("javascript:alert(1)"));
    assert!(!is_safe_media_url(
        "data:text/html,<script>alert(1)</script>"
    ));
    assert!(!is_safe_media_url(""));
}

#[test]
fn every_header_kind_reports_whether_it_is_media() {
    let headers = [
        (MessageHeader::Text("Hi".into()), false),
        (
            MessageHeader::Image {
                src: "a.png".into(),
                alt: "a".into(),
                width: Some(640),
                height: Some(480),
            },
            true,
        ),
        (
            MessageHeader::Video {
                src: "a.mp4".into(),
                poster: None,
                alt: "a".into(),
            },
            true,
        ),
        (
            MessageHeader::Document {
                name: "terms.pdf".into(),
                src: None,
            },
            true,
        ),
    ];
    for (header, media) in headers {
        assert_eq!(header.is_media(), media, "{header:?}");
    }
}

#[test]
fn message_label_joins_direction_author_and_time() {
    let texts = RichMessageTexts::default();
    let message = RichMessage {
        direction: MessageDirection::In,
        author: Some("+1 707-415-3915".into()),
        time: Some("4:41 PM".into()),
        ..Default::default()
    };
    assert_eq!(
        message_label(&message, &texts),
        "Received, +1 707-415-3915, 4:41 PM"
    );
    let preview = RichMessage::default();
    assert_eq!(message_label(&preview, &texts), "Preview");
    assert_eq!(
        message_label(&preview, &RichMessageTexts::es()),
        "Vista previa"
    );
}

#[test]
fn truncation_note_fills_both_numbers() {
    let note = truncation_note(
        MessageTruncation {
            shown_len: 4000,
            true_len: 5210,
        },
        &RichMessageTexts::default(),
    );
    assert_eq!(note, "Showing 4000 of 5210 characters");
}

#[test]
fn preview_actions_are_inert_and_never_navigate() {
    let actions = [
        MessageAction::Url {
            label: "Visit website".into(),
            href: "https://example.org".into(),
        },
        MessageAction::Call {
            label: "Call now".into(),
            tel: "+17075550100".into(),
        },
        MessageAction::CopyCode {
            label: "Copy code".into(),
            code: "SAVE10".into(),
        },
        MessageAction::QuickReply {
            label: "Push to respond".into(),
        },
    ];
    for action in &actions {
        let plan = plan_action_button(action, false);
        assert_eq!(plan.aria_disabled, Some("true"), "{action:?}");
        assert!(plan.described_by_hint, "{action:?}");
        assert_eq!(plan.label, action.label());
        assert_eq!(plan.title.as_deref(), action.target());
        let live = plan_action_button(action, true);
        assert_eq!(live.aria_disabled, None, "{action:?}");
        assert!(!live.described_by_hint);
    }
    let kinds: Vec<_> = actions.iter().map(MessageAction::kind).collect();
    assert_eq!(kinds, ["url", "call", "copy-code", "quick-reply"]);
}

#[test]
fn spanish_texts_cover_every_field() {
    let en = RichMessageTexts::default();
    let es = RichMessageTexts::es();
    // Every field except the encoding names is translated.
    let pairs = [
        (&en.received, &es.received),
        (&en.sent, &es.sent),
        (&en.preview, &es.preview),
        (&en.image_unavailable, &es.image_unavailable),
        (&en.video_unavailable, &es.video_unavailable),
        (&en.document, &es.document),
        (&en.picture_message, &es.picture_message),
        (&en.segment, &es.segment),
        (&en.segments, &es.segments),
        (&en.chars, &es.chars),
        (&en.not_sendable, &es.not_sendable),
        (&en.unfilled, &es.unfilled),
        (&en.actions, &es.actions),
        (&en.actions_preview_hint, &es.actions_preview_hint),
        (&en.truncated, &es.truncated),
        (&en.opens_new_tab, &es.opens_new_tab),
    ];
    for (en, es) in pairs {
        assert_ne!(en, es);
        assert!(!es.is_empty());
    }
    assert!(es.truncated.contains("{shown}") && es.truncated.contains("{total}"));
}
