use super::inline::{InlineStyle, Run, RunContent, inline_runs};
use super::sms::{sms_indicator_text, sms_segments};
use super::types::{
    MessageAction, MessageChannel, MessageDirection, MessageHeader, PlaceholderValues, RichMessage,
    RichMessageTexts, is_safe_link_url, is_safe_media_url, media_fallback_name, message_label,
    truncation_note,
};
use crate::merge_classes;
use leptos::{html, prelude::*};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_HINT_ID: AtomicUsize = AtomicUsize::new(0);

fn next_hint_id() -> String {
    format!(
        "ld-rich-message-hint-{}",
        NEXT_HINT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

// Lucide glyphs (ISC), stroked in currentColor so they follow the bubble.
const ICON_EXTERNAL: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>"#;
const ICON_PHONE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"/></svg>"#;
const ICON_COPY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>"#;
const ICON_REPLY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><polyline points="9 14 4 9 9 4"/><path d="M20 20v-7a4 4 0 0 0-4-4H4"/></svg>"#;
const ICON_IMAGE_OFF: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><line x1="2" x2="22" y1="2" y2="22"/><path d="M10.41 10.41a2 2 0 1 1-2.83-2.83"/><line x1="13.5" x2="6" y1="13.5" y2="21"/><line x1="18" x2="21" y1="12" y2="15"/><path d="M3.59 3.59A1.99 1.99 0 0 0 3 5v14a2 2 0 0 0 2 2h14c.55 0 1.052-.22 1.41-.59"/><path d="M21 15V5a2 2 0 0 0-2-2H9"/></svg>"#;
const ICON_FILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="100%" height="100%"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/></svg>"#;

fn glyph(svg: &'static str) -> impl IntoView {
    view! { <span class="inline-block h-4 w-4 shrink-0" aria-hidden="true" inner_html=svg></span> }
}

fn action_icon(action: &MessageAction) -> &'static str {
    match action {
        MessageAction::Url { .. } => ICON_EXTERNAL,
        MessageAction::Call { .. } => ICON_PHONE,
        MessageAction::CopyCode { .. } => ICON_COPY,
        MessageAction::QuickReply { .. } => ICON_REPLY,
    }
}

/// How one action button is rendered. A button never carries an `href`:
/// a preview is `aria-disabled` and described by the preview hint, and a
/// live button only reports the press to the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionButtonPlan {
    /// The `data-message-action` value.
    pub kind: &'static str,
    /// The visible label.
    pub label: String,
    /// The button's tooltip: where the recipient's button leads.
    pub title: Option<String>,
    /// `Some("true")` for an inert preview button.
    pub aria_disabled: Option<&'static str>,
    /// True when the button references the preview hint.
    pub described_by_hint: bool,
}

/// Plans the button for `action`; `live` is whether the host supplied an
/// `on_action` callback.
pub fn plan_action_button(action: &MessageAction, live: bool) -> ActionButtonPlan {
    ActionButtonPlan {
        kind: action.kind(),
        label: action.label().to_string(),
        title: action.target().map(str::to_string),
        aria_disabled: (!live).then_some("true"),
        described_by_hint: !live,
    }
}

fn run_view(run: Run, values: Option<&PlaceholderValues>, texts: &RichMessageTexts) -> AnyView {
    let InlineStyle {
        bold,
        italic,
        strike,
        mono,
    } = run.style;
    let mut node = match run.content {
        RunContent::Text(text) => text.into_any(),
        RunContent::Link { text, href } => {
            let suffix = format!(" {}", texts.opens_new_tab);
            view! {
                <a class="link" href=href target="_blank" rel="noopener noreferrer">
                    {text}
                    <span class="sr-only select-none">{suffix}</span>
                </a>
            }
            .into_any()
        }
        RunContent::Placeholder { raw, key } => {
            match values
                .and_then(|v| v.get(&key))
                .filter(|v| !v.trim().is_empty())
            {
                Some(value) => view! {
                    <mark
                        class="rounded-selector bg-warning px-1 text-warning-content"
                        data-placeholder=key
                        data-placeholder-state="filled"
                    >
                        {value.clone()}
                    </mark>
                }
                .into_any(),
                None => {
                    let suffix = format!(" ({})", texts.unfilled);
                    view! {
                        <span
                            class="badge badge-sm badge-outline font-mono"
                            data-placeholder=key
                            data-placeholder-state="unfilled"
                        >
                            {raw}
                            <span class="sr-only select-none">{suffix}</span>
                        </span>
                    }
                    .into_any()
                }
            }
        }
    };
    if mono {
        node = view! { <span class="font-mono">{node}</span> }.into_any();
    }
    if strike {
        node = view! { <s>{node}</s> }.into_any();
    }
    if italic {
        node = view! { <em>{node}</em> }.into_any();
    }
    if bold {
        node = view! { <strong>{node}</strong> }.into_any();
    }
    node
}

fn runs_view(
    text: &str,
    whatsapp: bool,
    placeholders: bool,
    linkify: bool,
    values: Option<&PlaceholderValues>,
    texts: &RichMessageTexts,
) -> Vec<AnyView> {
    inline_runs(text, whatsapp, placeholders, linkify)
        .into_iter()
        .map(|run| run_view(run, values, texts))
        .collect()
}

fn media_fallback(label: String, name: String) -> impl IntoView {
    view! {
        <div
            class="flex items-center gap-2 rounded-field border border-current p-3 text-sm"
            data-media-fallback="true"
        >
            {glyph(ICON_IMAGE_OFF)}
            <span class="flex flex-col">
                <span class="font-semibold">{label}</span>
                <span class="break-all">{name}</span>
            </span>
        </div>
    }
}

fn header_view(
    header: MessageHeader,
    placeholders: bool,
    values: Option<&PlaceholderValues>,
    texts: &RichMessageTexts,
) -> AnyView {
    match header {
        MessageHeader::Text(text) => {
            let runs = runs_view(&text, false, placeholders, false, values, texts);
            view! { <p class="font-semibold" data-message-header="text">{runs}</p> }.into_any()
        }
        MessageHeader::Image {
            src,
            alt,
            width,
            height,
        } => {
            let failed = RwSignal::new(!is_safe_media_url(&src));
            let label = texts.image_unavailable.clone();
            let name = media_fallback_name(&src, &alt);
            let width = width.map(|w| w.to_string());
            let height = height.map(|h| h.to_string());
            view! {
                <div data-message-header="image">
                    {move || {
                        if failed.get() {
                            media_fallback(label.clone(), name.clone()).into_any()
                        } else {
                            view! {
                                <img
                                    class="block h-auto max-w-full rounded-field"
                                    src=src.clone()
                                    alt=alt.clone()
                                    width=width.clone()
                                    height=height.clone()
                                    decoding="async"
                                    on:error=move |_| failed.set(true)
                                />
                            }
                            .into_any()
                        }
                    }}
                </div>
            }
            .into_any()
        }
        MessageHeader::Video { src, poster, alt } => {
            let failed = RwSignal::new(!is_safe_media_url(&src));
            let label = texts.video_unavailable.clone();
            let poster = poster.filter(|p| is_safe_media_url(p));
            let name = media_fallback_name(&src, &alt);
            view! {
                <div data-message-header="video">
                    {move || {
                        if failed.get() {
                            media_fallback(label.clone(), name.clone()).into_any()
                        } else {
                            view! {
                                <video
                                    class="block max-w-full rounded-field"
                                    src=src.clone()
                                    poster=poster.clone()
                                    aria-label=alt.clone()
                                    controls=true
                                    preload="none"
                                    on:error=move |_| failed.set(true)
                                ></video>
                            }
                            .into_any()
                        }
                    }}
                </div>
            }
            .into_any()
        }
        MessageHeader::Document { name, src } => {
            let prefix = format!("{}: ", texts.document);
            let opens = format!(" {}", texts.opens_new_tab);
            let label = match src.filter(|s| is_safe_link_url(s)) {
                Some(src) => view! {
                    <a class="link break-all" href=src target="_blank" rel="noopener noreferrer">
                        {name}
                        <span class="sr-only select-none">{opens}</span>
                    </a>
                }
                .into_any(),
                None => view! { <span class="break-all">{name}</span> }.into_any(),
            };
            view! {
                <div
                    class="flex items-center gap-2 rounded-field border border-current p-3 text-sm"
                    data-message-header="document"
                >
                    {glyph(ICON_FILE)}
                    <span>
                        <span class="sr-only select-none">{prefix}</span>
                        {label}
                    </span>
                </div>
            }
            .into_any()
        }
    }
}

/// # Rich Message Component
///
/// One SMS, MMS, WhatsApp or social direct message as the recipient sees
/// it: an optional header (text, image, video or document, with a fallback
/// tile when the media fails to load), the body with emoji and line breaks
/// kept, a footer, and the message's buttons. Built on daisyUI's `chat`
/// bubble.
///
/// - **WhatsApp bodies** render `*bold*`, `_italic_`, `~strike~` and
///   triple-backtick monospace by WhatsApp's rules; SMS, MMS and social
///   bodies stay plain.
/// - **Placeholders** (`{{name}}`, `{{1}}`, `${Contacts.First Name}`) are
///   recognised in a preview, or whenever `placeholders` is set. A key with
///   a value renders as a highlighted value; a key without one renders as
///   an "unfilled" chip.
/// - **Buttons** never navigate. Without `on_action` they are
///   `aria-disabled` previews described by a hint; with it, a press reports
///   the [`MessageAction`] to the host.
/// - **SMS indicator**: a preview of an SMS or MMS shows its encoding,
///   segments and length ("UCS-2 · 11 segments · 712 chars"); the host can
///   replace the text through `sms_indicator_override`.
///
/// Every string comes from `texts` ([`RichMessageTexts`], English by
/// default, [`RichMessageTexts::es`] for Spanish). Timestamps are display
/// strings; nothing here reads a clock. The component fetches nothing but
/// the media `src` the host gives it.
///
/// ```rust,ignore
/// let message = RichMessage {
///     channel: MessageChannel::Whatsapp,
///     body: "Hi {{first_name}}, your appointment is *confirmed* ✅".into(),
///     actions: vec![MessageAction::Call { label: "Call now".into(), tel: "+17075550100".into() }],
///     ..Default::default()
/// };
/// view! { <RichMessageBubble message=message /> }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("chat chat-start chat-end chat-header chat-bubble chat-bubble-neutral chat-footer badge badge-sm badge-outline badge-warning btn btn-sm link");
/// ```
///
/// ## Node References
/// - `node_ref` - References the root `<article>` element ([HTMLElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement))
#[component]
pub fn RichMessageBubble(
    /// The message to render.
    #[prop(into)]
    message: Signal<RichMessage>,

    /// Every rendered string. Defaults to English.
    #[prop(into, default = Signal::stored(RichMessageTexts::default()))]
    texts: Signal<RichMessageTexts>,

    /// Placeholder values. `Some` turns placeholder recognition on for any
    /// direction; a preview recognises placeholders even when this is
    /// `None`, showing each as unfilled.
    #[prop(optional, into)]
    placeholders: Signal<Option<PlaceholderValues>>,

    /// Turn URLs, `www.` hosts and email addresses in the body into links
    /// that open in a new tab.
    #[prop(optional, into)]
    linkify: Signal<bool>,

    /// `false` shows the "Not sendable" badge, for a template the host
    /// refuses to send.
    #[prop(into, default = Signal::stored(true))]
    sendable: Signal<bool>,

    /// Show the SMS segment indicator. Unset: shown for an SMS or MMS
    /// preview, hidden otherwise. Never shown for WhatsApp or social
    /// messages, whatever this says.
    #[prop(optional, into)]
    show_sms_indicator: Option<Signal<bool>>,

    /// Replaces the computed indicator text, for a carrier whose limits
    /// differ from 3GPP's.
    #[prop(optional, into)]
    sms_indicator_override: Signal<Option<String>>,

    /// Makes the buttons live: a press reports its action, and the host
    /// decides what happens. Unset, every button is an inert preview.
    #[prop(optional)]
    on_action: Option<Callback<MessageAction>>,

    /// Additional CSS classes for the root `<article>`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the root `<article>`.
    #[prop(optional)]
    node_ref: NodeRef<html::Article>,
) -> impl IntoView {
    let hint_id = next_hint_id();
    let live = on_action.is_some();

    let placement = move || {
        if message.with(|m| m.direction == MessageDirection::Out) {
            "chat-end"
        } else {
            "chat-start"
        }
    };

    let content = move || {
        let m = message.get();
        let t = texts.get();
        let values = placeholders.get();
        let recognise = values.is_some() || m.direction == MessageDirection::Preview;
        let whatsapp = m.channel == MessageChannel::Whatsapp;
        let outbound = m.direction == MessageDirection::Out;
        let picture = m.is_picture_message();

        let author = m.author.clone();
        let time = m.time.clone();
        let not_sendable = (!sendable.get()).then(|| t.not_sendable.clone());
        let has_header = author.is_some() || time.is_some() || not_sendable.is_some();
        let header_line = has_header.then(|| {
            view! {
                <div class="chat-header flex-wrap items-center">
                    {author.map(|a| view! { <span class="font-semibold">{a}</span> })}
                    {time.map(|t| view! { <span>{t}</span> })}
                    {not_sendable.map(|label| {
                        view! {
                            <span class="badge badge-sm badge-warning" data-message-not-sendable="true">
                                {label}
                            </span>
                        }
                    })}
                </div>
            }
        });

        let header = m
            .header
            .clone()
            .map(|h| header_view(h, recognise, values.as_ref(), &t));
        let body = (!m.body.is_empty()).then(|| {
            let runs = runs_view(
                &m.body,
                whatsapp,
                recognise,
                linkify.get(),
                values.as_ref(),
                &t,
            );
            view! { <p class="whitespace-pre-wrap break-words" data-message-body="true">{runs}</p> }
        });
        let footer = m.footer.clone().map(|f| {
            view! { <p class="text-xs" data-message-footer="true">{f}</p> }
        });

        let truncated = m.truncated.map(|tr| {
            view! { <p data-message-truncated="true">{truncation_note(tr, &t)}</p> }
        });

        let mut actions = m.actions.clone();
        actions.extend(
            m.quick_replies
                .iter()
                .map(|label| MessageAction::QuickReply {
                    label: label.clone(),
                }),
        );
        let has_actions = !actions.is_empty();
        let buttons = actions
            .into_iter()
            .map(|action| {
                let plan = plan_action_button(&action, live);
                let icon = action_icon(&action);
                let described = plan.described_by_hint.then(|| hint_id.clone());
                view! {
                    <button
                        type="button"
                        class="btn btn-sm"
                        data-message-action=plan.kind
                        title=plan.title
                        aria-disabled=plan.aria_disabled
                        aria-describedby=described
                        on:click=move |_| {
                            if let Some(callback) = on_action {
                                callback.run(action.clone());
                            }
                        }
                    >
                        {glyph(icon)}
                        {plan.label}
                    </button>
                }
            })
            .collect::<Vec<_>>();
        let actions_group = has_actions.then(|| {
            let group_label = t.actions.clone();
            let hint = (!live).then(|| {
                view! {
                    <span id=hint_id.clone() class="sr-only select-none">
                        {t.actions_preview_hint.clone()}
                    </span>
                }
            });
            view! {
                <div
                    role="group"
                    aria-label=group_label
                    class="flex flex-wrap gap-2"
                    class:justify-end=outbound
                    data-message-actions="true"
                >
                    {buttons}
                    {hint}
                </div>
            }
        });

        let indicator_visible = m.channel.is_sms()
            && show_sms_indicator.map_or(m.direction == MessageDirection::Preview, |s| s.get());
        let indicator = indicator_visible.then(|| {
            let info = sms_segments(&m.body);
            let text = sms_indicator_override
                .get()
                .unwrap_or_else(|| sms_indicator_text(info, picture, &t));
            view! {
                <p
                    data-sms-indicator="true"
                    data-sms-encoding=info.encoding.as_str()
                    data-sms-segments=info.segments.to_string()
                    aria-live="polite"
                >
                    {text}
                </p>
            }
        });

        let has_footer_line = truncated.is_some() || has_actions || indicator.is_some();
        let footer_line = has_footer_line.then(|| {
            view! {
                <div
                    class="chat-footer flex-col gap-2"
                    class:items-end=outbound
                    class:items-start={!outbound}
                >
                    {truncated}
                    {actions_group}
                    {indicator}
                </div>
            }
        });

        view! {
            {header_line}
            <div
                class="chat-bubble flex max-w-[min(90%,36rem)] flex-col gap-2"
                // Not primary: daisyUI's dark theme sets primary-content on
                // primary at 4.12:1, under AA for body text (ldui-g8hh lane).
                class:chat-bubble-neutral=outbound
            >
                {header}
                {body}
                {footer}
            </div>
            {footer_line}
        }
    };

    view! {
        <article
            node_ref=node_ref
            class=move || merge_classes!("chat", placement(), class)
            aria-label=move || message.with(|m| texts.with(|t| message_label(m, t)))
            data-rich-message="true"
            data-channel=move || message.with(|m| m.channel.as_str())
            data-direction=move || message.with(|m| m.direction.as_str())
        >
            {content}
        </article>
    }
}

/// # Rich Message Thread Component
///
/// A conversation as an ordered list of [`RichMessageBubble`]s, oldest
/// first. Received messages sit at the start, sent ones at the end. Every
/// bubble shares the thread's `texts`, `placeholders`, `linkify` and
/// `on_action`.
///
/// ### Add to `input.css`
/// See [`RichMessageBubble`].
///
/// ## Node References
/// - `node_ref` - References the `<ol>` element ([HTMLOListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLOListElement))
#[component]
pub fn RichMessageThread(
    /// The messages, oldest first.
    #[prop(into)]
    messages: Signal<Vec<RichMessage>>,

    /// Accessible name of the list ("Conversation with …").
    #[prop(into)]
    label: Signal<String>,

    /// Every rendered string. Defaults to English.
    #[prop(into, default = Signal::stored(RichMessageTexts::default()))]
    texts: Signal<RichMessageTexts>,

    /// Placeholder values; see [`RichMessageBubble`].
    #[prop(optional, into)]
    placeholders: Signal<Option<PlaceholderValues>>,

    /// Auto-link URLs and email addresses in every body.
    #[prop(optional, into)]
    linkify: Signal<bool>,

    /// Makes every bubble's buttons live; see [`RichMessageBubble`].
    #[prop(optional)]
    on_action: Option<Callback<MessageAction>>,

    /// Additional CSS classes for the `<ol>`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the `<ol>`.
    #[prop(optional)]
    node_ref: NodeRef<html::Ol>,
) -> impl IntoView {
    view! {
        <ol
            node_ref=node_ref
            class=merge_classes!("flex flex-col gap-2", class)
            aria-label=move || label.get()
            data-rich-message-thread="true"
        >
            {move || {
                messages
                    .get()
                    .into_iter()
                    .map(|m| {
                        // A thread shows sent and received messages, never a
                        // composer, so the segment indicator stays off. The
                        // callback is passed only when the host gave one: any
                        // callback at all makes the buttons live.
                        let bubble = match on_action {
                            Some(on_action) => view! {
                                <RichMessageBubble
                                    message=Signal::stored(m)
                                    texts=texts
                                    placeholders=placeholders
                                    linkify=linkify
                                    show_sms_indicator=Signal::stored(false)
                                    on_action=on_action
                                />
                            }
                            .into_any(),
                            None => view! {
                                <RichMessageBubble
                                    message=Signal::stored(m)
                                    texts=texts
                                    placeholders=placeholders
                                    linkify=linkify
                                    show_sms_indicator=Signal::stored(false)
                                />
                            }
                            .into_any(),
                        };
                        view! { <li>{bubble}</li> }
                    })
                    .collect_view()
            }}
        </ol>
    }
}

/// # SMS Segment Indicator Component
///
/// The composer's "UCS-2 · 11 segments · 712 chars" line for a body being
/// typed, or "Picture message · 712 chars" when `picture` is set. Counting
/// follows 3GPP TS 23.038 (see [`sms_segments`]); `override_text` replaces
/// it for a carrier with different limits. The line is a polite live region,
/// so a screen reader hears the count settle as the user types.
///
/// ## Node References
/// - `node_ref` - References the `<p>` element ([HTMLParagraphElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLParagraphElement))
#[component]
pub fn SmsSegmentIndicator(
    /// The body being measured.
    #[prop(into)]
    text: Signal<String>,

    /// Describe it as a picture message instead of counting segments.
    #[prop(optional, into)]
    picture: Signal<bool>,

    /// Replaces the computed text.
    #[prop(optional, into)]
    override_text: Signal<Option<String>>,

    /// Every rendered string. Defaults to English.
    #[prop(into, default = Signal::stored(RichMessageTexts::default()))]
    texts: Signal<RichMessageTexts>,

    /// Additional CSS classes for the `<p>`.
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the `<p>`.
    #[prop(optional)]
    node_ref: NodeRef<html::P>,
) -> impl IntoView {
    let info = Memo::new(move |_| text.with(|t| sms_segments(t)));
    view! {
        <p
            node_ref=node_ref
            class=merge_classes!("text-xs", class)
            data-sms-indicator="true"
            data-sms-encoding=move || info.get().encoding.as_str()
            data-sms-segments=move || info.get().segments.to_string()
            aria-live="polite"
        >
            {move || {
                override_text
                    .get()
                    .unwrap_or_else(|| texts.with(|t| sms_indicator_text(info.get(), picture.get(), t)))
            }}
        </p>
    }
}
