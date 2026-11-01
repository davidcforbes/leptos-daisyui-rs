//! Showcase for `MessageViewer`, `MessageViewerDialog` and
//! `ViewMessageRowAction` (ldui-mhur).
//!
//! Browser proof: `tests/message_viewer_smoke.rs`
//! (`cargo xtask test-message-viewer`). Every fixture is keyed by
//! `data-testid`, never by document position.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use std::rc::Rc;

/// A local picture, so the showcase never fetches from another host.
const FLYER: &str = "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='480' height='360' viewBox='0 0 480 360'%3E%3Crect width='480' height='360' fill='%231f4e3d'/%3E%3Crect x='24' y='24' width='220' height='72' rx='8' fill='%23f5f0e1'/%3E%3Ctext x='134' y='70' font-family='sans-serif' font-size='28' font-weight='700' text-anchor='middle' fill='%231f4e3d'%3ERECORDATORIO%3C/text%3E%3Ctext x='40' y='190' font-family='sans-serif' font-size='30' font-weight='700' fill='%23ffd23f'%3E%E2%9C%93 Free consultation%3C/text%3E%3Ctext x='40' y='250' font-family='sans-serif' font-size='30' font-weight='700' fill='%23ffd23f'%3E%E2%9C%93 Tuesday 10:30 AM%3C/text%3E%3C/svg%3E";

/// Served by nothing: the picture fails and the fallback tile shows.
const MISSING: &str = "/missing/campaign-flyer.png";

/// 200 GSM-7 characters: two SMS segments.
const SMS_BODY: &str = "Diener Law reminder: your consultation is Tuesday at 10:30 AM at 1200 Main St, Suite 4. Please bring your ID and any letters you received. Reply C to confirm or R to reschedule. Reply STOP to opt out.";

fn image(src: &str, alt: &str) -> Option<MessageHeader> {
    Some(MessageHeader::Image {
        src: src.into(),
        alt: alt.into(),
        width: Some(480),
        height: Some(360),
    })
}

fn text(body: &str, header: Option<MessageHeader>) -> RichMessage {
    RichMessage {
        header,
        body: body.into(),
        time: Some("10:02 AM".into()),
        ..Default::default()
    }
}

fn whatsapp_template() -> MessageViewerData {
    let picture = || {
        image(
            FLYER,
            "Reminder banner: free consultation, Tuesday 10:30 AM",
        )
    };
    MessageViewerData {
        channel: MessageChannel::Whatsapp,
        sender_name: "Diener Law".into(),
        sender_number: Some("+1 707-555-0100".into()),
        whatsapp_template: true,
        text_only_variant: true,
        versions: vec![
            MessageVersion {
                label: "EN".into(),
                sent: RichMessage {
                    footer: Some("Reply STOP to opt out".into()),
                    actions: vec![MessageAction::QuickReply {
                        label: "Confirm".into(),
                    }],
                    ..text(
                        "Hi Ana 👋\n\nYour consultation is *confirmed* for Tuesday at 10:30 AM.",
                        picture(),
                    )
                },
                template: Some(RichMessage {
                    footer: Some("Reply STOP to opt out".into()),
                    actions: vec![MessageAction::QuickReply {
                        label: "Confirm".into(),
                    }],
                    ..text(
                        "Hi {{first_name}} 👋\n\nYour consultation is *confirmed* for {{appointment_date}}.",
                        picture(),
                    )
                }),
            },
            MessageVersion {
                label: "ES".into(),
                sent: RichMessage {
                    footer: Some("Responda STOP para no recibir más".into()),
                    actions: vec![MessageAction::QuickReply {
                        label: "Confirmar".into(),
                    }],
                    ..text(
                        "Hola Ana 👋\n\nSu consulta está *confirmada* para el martes a las 10:30 AM.",
                        picture(),
                    )
                },
                template: Some(RichMessage {
                    footer: Some("Responda STOP para no recibir más".into()),
                    actions: vec![MessageAction::QuickReply {
                        label: "Confirmar".into(),
                    }],
                    ..text(
                        "Hola {{first_name}} 👋\n\nSu consulta está *confirmada* para {{appointment_date}}.",
                        picture(),
                    )
                }),
            },
        ],
        sent_at: Some("Mon, Oct 27 2026, 10:02 AM".into()),
        status: Some(DeliveryStatus::Delivered),
        counts: Some(MessageCounts {
            sent_to: 132,
            with_picture: Some(120),
            text_only: Some(12),
        }),
        ..Default::default()
    }
}

fn whatsapp_session() -> MessageViewerData {
    MessageViewerData {
        channel: MessageChannel::Whatsapp,
        sender_name: "Diener Law".into(),
        sender_number: Some("+1 707-555-0100".into()),
        whatsapp_template: false,
        versions: vec![MessageVersion {
            label: "EN".into(),
            sent: text(
                "Thanks, Ana. We received your documents and will call you _tomorrow morning_.",
                None,
            ),
            template: None,
        }],
        sent_at: Some("Tue, Oct 28 2026, 4:15 PM".into()),
        status: Some(DeliveryStatus::Read),
        ..Default::default()
    }
}

fn sms_text() -> MessageViewerData {
    MessageViewerData {
        channel: MessageChannel::Sms,
        sender_name: "Front desk".into(),
        sender_number: Some("+1 707-415-3915".into()),
        versions: vec![MessageVersion {
            label: "EN".into(),
            sent: text(SMS_BODY, None),
            template: None,
        }],
        sent_at: Some("Wed, Oct 29 2026, 9:00 AM".into()),
        status: Some(DeliveryStatus::Sent),
        ..Default::default()
    }
}

fn mms_picture() -> MessageViewerData {
    MessageViewerData {
        channel: MessageChannel::Mms,
        sender_name: "Front desk".into(),
        sender_number: Some("+1 707-415-3915".into()),
        versions: vec![MessageVersion {
            label: "EN".into(),
            // No alt from the host: the viewer names it "MMS picture".
            sent: text("Our new office is open. See you there!", image(FLYER, "")),
            template: None,
        }],
        sent_at: Some("Thu, Oct 30 2026, 11:20 AM".into()),
        status: Some(DeliveryStatus::Delivered),
        ..Default::default()
    }
}

fn broken_picture() -> MessageViewerData {
    MessageViewerData {
        channel: MessageChannel::Mms,
        sender_name: "Front desk".into(),
        sender_number: Some("+1 707-415-3915".into()),
        versions: vec![MessageVersion {
            label: "EN".into(),
            sent: text(
                "Here is the flyer for Saturday's clinic.",
                image(MISSING, "Saturday clinic flyer"),
            ),
            template: None,
        }],
        sent_at: Some("Fri, Oct 31 2026, 8:45 AM".into()),
        status: Some(DeliveryStatus::Failed),
        status_reason: Some("Carrier rejected: the number is unreachable".into()),
        ..Default::default()
    }
}

fn with_replies() -> MessageViewerData {
    MessageViewerData {
        channel: MessageChannel::Sms,
        sender_name: "Front desk".into(),
        sender_number: Some("+1 707-415-3915".into()),
        versions: vec![MessageVersion {
            label: "EN".into(),
            sent: text("Hi Luis, can you come in Thursday at 2 PM to sign?", None),
            template: None,
        }],
        sent_at: Some("Thu, Oct 30 2026, 1:05 PM".into()),
        status: Some(DeliveryStatus::NoReceipt),
        replies: vec![
            RichMessage {
                body: "Yes, Thursday works.".into(),
                time: Some("1:12 PM".into()),
                ..Default::default()
            },
            RichMessage {
                body: "Should I bring my wife?".into(),
                time: Some("1:13 PM".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

#[derive(Clone)]
struct CampaignRow {
    id: &'static str,
    name: &'static str,
    channel: &'static str,
    data: MessageViewerData,
}

fn campaign_rows() -> Rc<Vec<CampaignRow>> {
    Rc::new(vec![
        CampaignRow {
            id: "spring",
            name: "Consultation reminder",
            channel: "WhatsApp",
            data: whatsapp_template(),
        },
        CampaignRow {
            id: "clinic",
            name: "Saturday clinic",
            channel: "MMS",
            data: broken_picture(),
        },
        CampaignRow {
            id: "office",
            name: "New office",
            channel: "MMS",
            data: mms_picture(),
        },
    ])
}

fn fixture(testid: &'static str, data: MessageViewerData) -> impl IntoView {
    view! {
        <div data-testid=testid>
            <MessageViewer data=data />
        </div>
    }
}

#[component]
pub fn MessageViewerDemo() -> impl IntoView {
    let rows = RwSignal::new_local(campaign_rows());
    let viewing = RwSignal::new(None::<(String, MessageViewerData)>);
    let columns = vec![
        EntityColumn::text("name", "Campaign", |row: &CampaignRow| row.name.to_owned()).required(),
        EntityColumn::text("channel", "Channel", |row: &CampaignRow| {
            row.channel.to_owned()
        }),
        EntityColumn::action("view", "Message", |_: &CampaignRow| String::new()).render_with(
            move |row: &CampaignRow| {
                let name = row.name.to_owned();
                let label = name.clone();
                let data = row.data.clone();
                view! {
                    <EntityRowAction action_id="view-message">
                        <ViewMessageRowAction
                            name=label
                            on_click=Callback::new(move |()| {
                                viewing.set(Some((name.clone(), data.clone())))
                            })
                        />
                    </EntityRowAction>
                }
                .into_any()
            },
        ),
    ];
    // The columns hold `Rc` closures; `Section`'s children must be `Send`.
    let columns = StoredValue::new_local(columns);

    view! {
        <ContentLayout
            title="MessageViewer"
            description="A sent WhatsApp, SMS or MMS message as the recipient saw it: channel chrome, the picture above the text, language / picture / template switches, delivery status and replies"
        >
            <Section title="WhatsApp template with a picture" col=true>
                <p class="text-sm text-base-content/75">
                    "An approved template in English and Spanish. Some recipients got the text only; the template shows its placeholders. Select the picture to enlarge it."
                </p>
                {fixture("viewer-whatsapp-template", whatsapp_template())}
            </Section>

            <Section title="WhatsApp session message" col=true>
                <p class="text-sm text-base-content/75">
                    "A reply inside the 24-hour window: text only, read by the recipient."
                </p>
                {fixture("viewer-whatsapp-session", whatsapp_session())}
            </Section>

            <Section title="SMS, two segments" col=true>
                <p class="text-sm text-base-content/75">"A 200-character text message and its segment count."</p>
                {fixture("viewer-sms", sms_text())}
            </Section>

            <Section title="MMS with a picture" col=true>
                <p class="text-sm text-base-content/75">
                    "The host gave no alt text, so the picture is named for its channel."
                </p>
                {fixture("viewer-mms", mms_picture())}
            </Section>

            <Section title="Broken picture" col=true>
                <p class="text-sm text-base-content/75">
                    "The picture could not be loaded, and the message failed; the status explains why."
                </p>
                {fixture("viewer-broken", broken_picture())}
            </Section>

            <Section title="Spanish" col=true>
                <p class="text-sm text-base-content/75">"The WhatsApp template with the Spanish text table."</p>
                <div data-testid="viewer-spanish">
                    <MessageViewer data=whatsapp_template() texts=MessageViewerTexts::es() />
                </div>
            </Section>

            <Section title="Reply thread" col=true>
                <p class="text-sm text-base-content/75">
                    "One recipient's copy with their replies; the route sends no delivery receipts."
                </p>
                {fixture("viewer-replies", with_replies())}
            </Section>

            <Section title="From a table row" col=true>
                <p class="text-sm text-base-content/75">
                    "The eye action on each row opens the viewer in a dialog."
                </p>
                <div class="max-w-2xl" data-testid="viewer-table">
                    <EntityTable
                        data=rows
                        columns=columns.get_value()
                        row_key=Rc::new(|row: &CampaignRow| row.id.to_owned())
                        dataset_identity="message-viewer-campaigns"
                    />
                </div>
                <MessageViewerDialog
                    open=Signal::derive(move || viewing.with(Option::is_some))
                    on_close=Callback::new(move |()| viewing.set(None))
                    title=Signal::derive(move || {
                        viewing.with(|v| v.as_ref().map(|(name, _)| name.clone()).unwrap_or_default())
                    })
                    data=Signal::derive(move || {
                        viewing.with(|v| v.as_ref().map(|(_, data)| data.clone()).unwrap_or_default())
                    })
                    attr:data-testid="viewer-dialog"
                />
            </Section>
        </ContentLayout>
    }
}
