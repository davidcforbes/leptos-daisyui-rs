use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// A local picture header, so the showcase never fetches from another host.
const FLYER: &str = "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='480' height='360' viewBox='0 0 480 360'%3E%3Crect width='480' height='360' fill='%231f4e3d'/%3E%3Crect x='24' y='24' width='200' height='72' rx='8' fill='%23f5f0e1'/%3E%3Ctext x='124' y='70' font-family='sans-serif' font-size='28' font-weight='700' text-anchor='middle' fill='%231f4e3d'%3EFLYER%3C/text%3E%3Ctext x='40' y='190' font-family='sans-serif' font-size='30' font-weight='700' fill='%23ffd23f'%3E%E2%9C%93 Real Leadership%3C/text%3E%3Ctext x='40' y='250' font-family='sans-serif' font-size='30' font-weight='700' fill='%23ffd23f'%3E%E2%9C%93 Proven Experience%3C/text%3E%3C/svg%3E";

/// Served by nothing: the image fails and the fallback tile shows.
const MISSING_IMAGE: &str = "/missing/campaign-flyer.png";

const SMS_BODY: &str = "Hi, this is Sylvia Lemus, Candidate for Sonoma County Supervisor.\n\nWith less than 2 weeks before Election Day, if you haven’t already, I respectfully ask for your vote.\n\n✅ I’m a lifelong Democrat\n\n✅ I will be an advocate for affordability while balancing the budget\n\n✅ I will be a voice of reason, leadership, and experience\n\nIf you have any questions, please don’t hesitate to contact me. My email is sylvia@votelemus.com - you can also learn more at https://www.VoteLemus.com\n\nPaid for by Sylvia Lemus for Supervisor 2026. Reply STOP to opt out.";

fn picture_sms() -> RichMessage {
    RichMessage {
        channel: MessageChannel::Mms,
        direction: MessageDirection::Preview,
        author: Some("+1 707-415-3915".into()),
        time: Some("4:41 PM".into()),
        header: Some(MessageHeader::Image {
            src: FLYER.into(),
            alt: "Campaign flyer: Real Leadership, Proven Experience".into(),
            width: Some(480),
            height: Some(360),
        }),
        body: SMS_BODY.into(),
        ..Default::default()
    }
}

fn whatsapp_template() -> RichMessage {
    RichMessage {
        channel: MessageChannel::Whatsapp,
        direction: MessageDirection::Preview,
        header: Some(MessageHeader::Image {
            src: FLYER.into(),
            alt: "Appointment reminder banner".into(),
            width: Some(480),
            height: Some(360),
        }),
        body: "Hi {{first_name}} 👋\n\nYour consultation is *confirmed* for {{appointment_date}} at {{appointment_time}}.\n\n_Please bring_ your ID and ~old~ *new* paperwork. Reference: ```{{1}}```\n\n${Contacts.First Name}, reply below if you need to reschedule.".into(),
        footer: Some("Diener Law · Reply STOP to opt out".into()),
        actions: vec![
            MessageAction::Url {
                label: "Visit website".into(),
                href: "https://example.org/appointments".into(),
            },
            MessageAction::Call {
                label: "Call now".into(),
                tel: "+17075550100".into(),
            },
            MessageAction::CopyCode {
                label: "Copy code".into(),
                code: "CONF-2026".into(),
            },
        ],
        quick_replies: vec!["Push to respond".into()],
        ..Default::default()
    }
}

fn sample_values() -> PlaceholderValues {
    [
        ("first_name", "María"),
        ("appointment_date", "Oct 28"),
        ("1", "A-1042"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn thread() -> Vec<RichMessage> {
    vec![
        RichMessage {
            channel: MessageChannel::Sms,
            direction: MessageDirection::Out,
            author: Some("Diener Law".into()),
            time: Some("Mon 9:02 AM".into()),
            body: "Hi María, this is Diener Law confirming your consultation tomorrow at 10 AM. Reply C to confirm.".into(),
            ..Default::default()
        },
        RichMessage {
            channel: MessageChannel::Sms,
            direction: MessageDirection::In,
            author: Some("+1 707-555-0142".into()),
            time: Some("Mon 9:15 AM".into()),
            body: "C 👍🏽 ¡Gracias!".into(),
            ..Default::default()
        },
        RichMessage {
            channel: MessageChannel::Mms,
            direction: MessageDirection::In,
            author: Some("+1 707-555-0142".into()),
            time: Some("Mon 9:16 AM".into()),
            header: Some(MessageHeader::Image {
                src: MISSING_IMAGE.into(),
                alt: String::new(),
                width: None,
                height: None,
            }),
            body: "Here is the letter I received.".into(),
            ..Default::default()
        },
        RichMessage {
            channel: MessageChannel::Whatsapp,
            direction: MessageDirection::Out,
            author: Some("Diener Law".into()),
            time: Some("Mon 9:20 AM".into()),
            header: Some(MessageHeader::Document {
                name: "intake-checklist.pdf".into(),
                src: None,
            }),
            body: "Thanks! Please review the *intake checklist* before tomorrow.".into(),
            ..Default::default()
        },
        RichMessage {
            channel: MessageChannel::Whatsapp,
            direction: MessageDirection::In,
            author: Some("María López".into()),
            time: Some("Mon 9:41 AM".into()),
            body: "I read it. One question about item 4: ".repeat(6),
            truncated: Some(MessageTruncation {
                shown_len: 240,
                true_len: 4310,
            }),
            ..Default::default()
        },
    ]
}

#[component]
pub fn RichMessageDemo() -> impl IntoView {
    let (pressed, set_pressed) = signal(String::new());
    let (draft, set_draft) =
        signal("Hi {{first_name}}, your appointment is confirmed.".to_string());
    let live = RichMessage {
        channel: MessageChannel::Whatsapp,
        direction: MessageDirection::In,
        author: Some("Host-wired buttons".into()),
        body: "These buttons report the press to the host.".into(),
        actions: vec![MessageAction::Call {
            label: "Call now".into(),
            tel: "+17075550100".into(),
        }],
        quick_replies: vec!["Push to respond".into()],
        ..Default::default()
    };

    view! {
        <ContentLayout
            title="RichMessage"
            description="SMS, MMS and WhatsApp messages as the recipient sees them: media headers with a fallback, extended characters, WhatsApp formatting, placeholders, preview buttons and the SMS segment indicator"
        >
            <Section title="Picture message preview" col=true>
                <p class="text-sm text-base-content/75">
                    "An MMS preview with an image header, emoji, curly quotes and auto-linked contacts."
                </p>
                <div class="max-w-2xl" data-testid="rich-sms-preview">
                    <RichMessageBubble message=picture_sms() linkify=true />
                </div>
            </Section>

            <Section title="WhatsApp template preview" col=true>
                <p class="text-sm text-base-content/75">
                    "WhatsApp formatting, filled and unfilled placeholders, a footer and the template's buttons as an inert preview. The host refuses media-header sends, so the template is marked not sendable."
                </p>
                <div class="max-w-2xl" data-testid="rich-whatsapp-preview">
                    <RichMessageBubble
                        message=whatsapp_template()
                        placeholders=Some(sample_values())
                        sendable=false
                    />
                </div>
            </Section>

            <Section title="Conversation thread" col=true>
                <p class="text-sm text-base-content/75">
                    "Received and sent messages across channels, a picture that failed to load, a document and a truncated message."
                </p>
                <div class="max-w-2xl" data-testid="rich-thread">
                    <RichMessageThread
                        messages=thread()
                        label="Conversation with María López"
                        linkify=true
                    />
                </div>
            </Section>

            <Section title="Host-wired buttons" col=true>
                <p class="text-sm text-base-content/75">
                    "With an on_action callback the buttons are live; the host decides what a press does."
                </p>
                <div class="max-w-2xl" data-testid="rich-live">
                    <RichMessageBubble
                        message=live
                        on_action=Callback::new(move |action: MessageAction| {
                            set_pressed.set(action.label().to_string())
                        })
                    />
                    <p class="text-sm" data-testid="rich-live-pressed">
                        {move || {
                            let label = pressed.get();
                            if label.is_empty() {
                                "No button pressed yet".to_string()
                            } else {
                                format!("Pressed: {label}")
                            }
                        }}
                    </p>
                </div>
            </Section>

            <Section title="Composer segment indicator" col=true>
                <p class="text-sm text-base-content/75">
                    "Type to see the encoding and segment count change; an emoji or a curly quote switches to UCS-2."
                </p>
                <div class="flex max-w-2xl flex-col gap-2" data-testid="rich-composer">
                    <label class="flex flex-col gap-2">
                        <span class="text-sm font-semibold">"Message"</span>
                        <textarea
                            class="textarea w-full"
                            prop:value=draft
                            on:input=move |ev| set_draft.set(event_target_value(&ev))
                        ></textarea>
                    </label>
                    <SmsSegmentIndicator text=draft />
                </div>
            </Section>

            <Section title="Spanish" col=true>
                <p class="text-sm text-base-content/75">
                    "The same preview with the Spanish text table."
                </p>
                <div class="max-w-2xl" data-testid="rich-spanish">
                    <RichMessageBubble
                        message=RichMessage {
                            channel: MessageChannel::Sms,
                            body: "Hola {{first_name}}, su cita está confirmada.".into(),
                            actions: vec![MessageAction::QuickReply { label: "Confirmar".into() }],
                            ..Default::default()
                        }
                        texts=RichMessageTexts::es()
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
