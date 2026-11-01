//! The read-only message-viewer model (ldui-mhur).
//!
//! A host maps one SENT message (a campaign send, or one recipient's copy of
//! it) into [`MessageViewerData`]: the channel, who sent it, the versions the
//! audience could have received (languages, each with its as-sent text and
//! optionally the template it came from), the delivery outcome and any
//! replies. Every value is a display value the host has already formatted;
//! nothing here reads a clock or fetches anything.

use crate::components::{
    BadgeColor, MessageChannel, MessageDirection, MessageHeader, PlaceholderValues, RichMessage,
    RichMessageTexts,
};

/// What happened to a sent message, in the closed vocabulary every host maps
/// its provider states into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeliveryStatus {
    /// Accepted by the host, not yet handed to the network.
    Queued,
    /// Handed to the network; no delivery receipt yet.
    Sent,
    /// The network reported delivery to the handset.
    Delivered,
    /// The recipient opened it (WhatsApp read receipt).
    Read,
    /// The network refused or bounced it.
    Failed,
    /// Sent, but the network never reports delivery for this route.
    NoReceipt,
}

impl DeliveryStatus {
    /// Every status, in lifecycle order.
    pub const ALL: [DeliveryStatus; 6] = [
        DeliveryStatus::Queued,
        DeliveryStatus::Sent,
        DeliveryStatus::Delivered,
        DeliveryStatus::Read,
        DeliveryStatus::Failed,
        DeliveryStatus::NoReceipt,
    ];

    /// The `data-delivery-status` value on the status chip.
    pub const fn as_str(self) -> &'static str {
        match self {
            DeliveryStatus::Queued => "queued",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Read => "read",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::NoReceipt => "no-receipt",
        }
    }

    /// The chip's colour. Solid fills, because an outlined green or amber on
    /// the base colour is under AA for small text.
    pub const fn color(self) -> BadgeColor {
        match self {
            DeliveryStatus::Queued => BadgeColor::Neutral,
            DeliveryStatus::Sent => BadgeColor::Info,
            DeliveryStatus::Delivered | DeliveryStatus::Read => BadgeColor::Success,
            DeliveryStatus::Failed => BadgeColor::Error,
            DeliveryStatus::NoReceipt => BadgeColor::Warning,
        }
    }

    /// The status's label from a texts table.
    pub fn label(self, texts: &MessageViewerTexts) -> &str {
        match self {
            DeliveryStatus::Queued => &texts.queued,
            DeliveryStatus::Sent => &texts.sent,
            DeliveryStatus::Delivered => &texts.delivered,
            DeliveryStatus::Read => &texts.read,
            DeliveryStatus::Failed => &texts.failed,
            DeliveryStatus::NoReceipt => &texts.no_receipt,
        }
    }
}

/// Which text of a version the viewer shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MessageViewerMode {
    /// The final text the recipient received.
    #[default]
    AsSent,
    /// The template it was filled from, placeholders highlighted.
    Template,
}

impl MessageViewerMode {
    /// The `data-viewer-mode` value on the viewer root.
    pub const fn as_str(self) -> &'static str {
        match self {
            MessageViewerMode::AsSent => "as-sent",
            MessageViewerMode::Template => "template",
        }
    }
}

/// One version of the message the audience could receive, usually one
/// language.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageVersion {
    /// The switch label: "EN", "ES", "English".
    pub label: String,
    /// The text as sent. An image header is the message's picture.
    pub sent: RichMessage,
    /// The template the text was filled from. `None` hides the
    /// "As sent / Template" switch for this version.
    pub template: Option<RichMessage>,
}

impl MessageVersion {
    /// True when the as-sent message carries a picture.
    pub fn has_picture(&self) -> bool {
        has_image(&self.sent)
    }
}

/// How many recipients a campaign reached, for the meta strip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MessageCounts {
    /// Recipients the message was sent to.
    pub sent_to: u32,
    /// Of those, how many received the picture.
    pub with_picture: Option<u32>,
    /// Of those, how many received the text only.
    pub text_only: Option<u32>,
}

/// Everything [`MessageViewer`](super::MessageViewer) shows. Plain owned
/// values; a host maps its own rows into it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageViewerData {
    /// The network. It picks the chrome (WhatsApp or SMS/MMS) and overrides
    /// each version's own `channel`.
    pub channel: MessageChannel,
    /// The sender's display name (the business, or the staff member).
    pub sender_name: String,
    /// The sending number, shown under the name.
    pub sender_number: Option<String>,
    /// WhatsApp only: the message was an approved template (sent outside the
    /// 24-hour window) rather than a session message.
    pub whatsapp_template: bool,
    /// The versions, first shown first. A "Language" switch appears when
    /// there are two or more.
    pub versions: Vec<MessageVersion>,
    /// Some recipients got the text without the picture: offers the
    /// "With picture / Text only" switch when the shown version has one.
    pub text_only_variant: bool,
    /// When it was sent, formatted by the host.
    pub sent_at: Option<String>,
    /// The delivery outcome.
    pub status: Option<DeliveryStatus>,
    /// Why the status is what it is ("Carrier rejected: unreachable"); the
    /// chip's tooltip and accessible description.
    pub status_reason: Option<String>,
    /// Campaign reach, for the meta strip.
    pub counts: Option<MessageCounts>,
    /// The recipient's replies, oldest first, shown as received messages
    /// under the sent one.
    pub replies: Vec<RichMessage>,
}

/// Which version, picture and text the viewer is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MessageViewerSelection {
    /// Index into [`MessageViewerData::versions`]; clamped when shown.
    pub version: usize,
    /// `false` shows the text-only variant.
    pub with_picture: bool,
    /// As sent or template.
    pub mode: MessageViewerMode,
}

impl MessageViewerSelection {
    /// The first version, with its picture, as sent.
    pub const fn initial() -> Self {
        Self {
            version: 0,
            with_picture: true,
            mode: MessageViewerMode::AsSent,
        }
    }
}

/// Every string the viewer renders. `Default` is English;
/// [`MessageViewerTexts::es`] is Spanish. A host overrides single fields by
/// struct-update syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageViewerTexts {
    /// The bubble's own strings. `image_unavailable` is replaced by
    /// [`Self::picture_unavailable`] inside the viewer.
    pub rich: RichMessageTexts,
    /// Accessible name of the phone frame region.
    pub phone_label: String,
    /// Channel name: WhatsApp.
    pub whatsapp: String,
    /// Channel name: SMS.
    pub sms: String,
    /// Channel name: MMS (an SMS with a picture).
    pub mms: String,
    /// Channel name: social direct message.
    pub social: String,
    /// Chrome badge on an approved WhatsApp template.
    pub template_badge: String,
    /// Chrome badge on a WhatsApp session message.
    pub session_badge: String,
    /// Accessible name of the version switches.
    pub versions: String,
    /// Accessible name of the language switch.
    pub language: String,
    /// Accessible name of the picture switch.
    pub picture: String,
    /// Picture switch: with the picture.
    pub with_picture: String,
    /// Picture switch: text only.
    pub text_only: String,
    /// Accessible name of the as-sent / template switch.
    pub content: String,
    /// Content switch: the final text.
    pub as_sent: String,
    /// Content switch: the template with placeholders.
    pub template: String,
    /// Accessible name of the details list.
    pub details: String,
    /// Details term: channel.
    pub channel_term: String,
    /// Details term: when it was sent.
    pub sent_at_term: String,
    /// Details term: who sent it.
    pub from_term: String,
    /// Details term: delivery status.
    pub status_term: String,
    /// Details term: campaign reach.
    pub recipients_term: String,
    /// Status: queued.
    pub queued: String,
    /// Status: sent.
    pub sent: String,
    /// Status: delivered.
    pub delivered: String,
    /// Status: read.
    pub read: String,
    /// Status: failed.
    pub failed: String,
    /// Status: no delivery receipt.
    pub no_receipt: String,
    /// Reach: `{n}` recipients.
    pub sent_to: String,
    /// Reach: `{n}` with the picture.
    pub with_picture_count: String,
    /// Reach: `{n}` text only.
    pub text_only_count: String,
    /// Alt text for a picture whose host gave none; `{channel}` is the
    /// channel name.
    pub picture_fallback_alt: String,
    /// Fallback tile text when the picture fails to load.
    pub picture_unavailable: String,
    /// Accessible-name prefix of the enlarge button and the enlarged view.
    pub enlarge_picture: String,
    /// Close button of the enlarged picture.
    pub close_picture: String,
    /// Accessible name of the reply thread.
    pub replies: String,
    /// Close button of [`MessageViewerDialog`](super::MessageViewerDialog).
    pub close: String,
    /// The dialog backdrop's close control.
    pub backdrop_close: String,
    /// Row-action label template for
    /// [`ViewMessageRowAction`](super::ViewMessageRowAction); `{name}` is
    /// the row's name.
    pub view_message: String,
    /// Shown when the host passed no version at all.
    pub no_message: String,
}

impl Default for MessageViewerTexts {
    fn default() -> Self {
        Self {
            rich: RichMessageTexts::default(),
            phone_label: "Message as the recipient saw it".into(),
            whatsapp: "WhatsApp".into(),
            sms: "SMS".into(),
            mms: "MMS".into(),
            social: "Social message".into(),
            template_badge: "Template".into(),
            session_badge: "Session message".into(),
            versions: "Message versions".into(),
            language: "Language".into(),
            picture: "Picture".into(),
            with_picture: "With picture".into(),
            text_only: "Text only".into(),
            content: "Content".into(),
            as_sent: "As sent".into(),
            template: "Template".into(),
            details: "Message details".into(),
            channel_term: "Channel".into(),
            sent_at_term: "Sent".into(),
            from_term: "From".into(),
            status_term: "Status".into(),
            recipients_term: "Recipients".into(),
            queued: "Queued".into(),
            sent: "Sent".into(),
            delivered: "Delivered".into(),
            read: "Read".into(),
            failed: "Failed".into(),
            no_receipt: "No receipt".into(),
            sent_to: "Sent to {n}".into(),
            with_picture_count: "with picture {n}".into(),
            text_only_count: "text only {n}".into(),
            picture_fallback_alt: "{channel} picture".into(),
            picture_unavailable: "The picture could not be shown".into(),
            enlarge_picture: "Enlarge picture".into(),
            close_picture: "Close picture".into(),
            replies: "Replies".into(),
            close: "Close".into(),
            backdrop_close: "close".into(),
            view_message: "View message: {name}".into(),
            no_message: "No message to show".into(),
        }
    }
}

impl MessageViewerTexts {
    /// The Spanish table.
    pub fn es() -> Self {
        Self {
            rich: RichMessageTexts::es(),
            phone_label: "Mensaje tal como lo vio el destinatario".into(),
            whatsapp: "WhatsApp".into(),
            sms: "SMS".into(),
            mms: "MMS".into(),
            social: "Mensaje de red social".into(),
            template_badge: "Plantilla".into(),
            session_badge: "Mensaje de sesión".into(),
            versions: "Versiones del mensaje".into(),
            language: "Idioma".into(),
            picture: "Imagen".into(),
            with_picture: "Con imagen".into(),
            text_only: "Solo texto".into(),
            content: "Contenido".into(),
            as_sent: "Como se envió".into(),
            template: "Plantilla".into(),
            details: "Detalles del mensaje".into(),
            channel_term: "Canal".into(),
            sent_at_term: "Enviado".into(),
            from_term: "De".into(),
            status_term: "Estado".into(),
            recipients_term: "Destinatarios".into(),
            queued: "En cola".into(),
            sent: "Enviado".into(),
            delivered: "Entregado".into(),
            read: "Leído".into(),
            failed: "Fallido".into(),
            no_receipt: "Sin confirmación".into(),
            sent_to: "Enviado a {n}".into(),
            with_picture_count: "con imagen {n}".into(),
            text_only_count: "solo texto {n}".into(),
            picture_fallback_alt: "Imagen de {channel}".into(),
            picture_unavailable: "No se pudo mostrar la imagen".into(),
            enlarge_picture: "Ampliar imagen".into(),
            close_picture: "Cerrar imagen".into(),
            replies: "Respuestas".into(),
            close: "Cerrar".into(),
            backdrop_close: "cerrar".into(),
            view_message: "Ver mensaje: {name}".into(),
            no_message: "No hay mensaje para mostrar".into(),
        }
    }

    /// The channel's display name.
    pub fn channel_name(&self, channel: MessageChannel) -> &str {
        match channel {
            MessageChannel::Sms => &self.sms,
            MessageChannel::Mms => &self.mms,
            MessageChannel::Whatsapp => &self.whatsapp,
            MessageChannel::Social => &self.social,
        }
    }

    /// The bubble strings the viewer passes down: the rich table with the
    /// viewer's own broken-picture message.
    pub fn bubble_texts(&self) -> RichMessageTexts {
        RichMessageTexts {
            image_unavailable: self.picture_unavailable.clone(),
            ..self.rich.clone()
        }
    }
}

fn has_image(message: &RichMessage) -> bool {
    matches!(message.header, Some(MessageHeader::Image { .. }))
}

/// The alt text a picture is shown with: the host's, or "{channel} picture"
/// when the host gave none, so no picture is ever nameless.
pub fn picture_alt(alt: &str, channel: MessageChannel, texts: &MessageViewerTexts) -> String {
    let alt = alt.trim();
    if alt.is_empty() {
        texts
            .picture_fallback_alt
            .replace("{channel}", texts.channel_name(channel))
    } else {
        alt.to_string()
    }
}

/// The index of the version a selection shows: its own, clamped to the last
/// version (0 when there is none).
pub fn selected_index(data: &MessageViewerData, selection: MessageViewerSelection) -> usize {
    selection.version.min(data.versions.len().saturating_sub(1))
}

/// The version a selection points at, clamped to the last one.
pub fn selected_version(
    data: &MessageViewerData,
    selection: MessageViewerSelection,
) -> Option<&MessageVersion> {
    data.versions.get(selected_index(data, selection))
}

/// Whether the "As sent / Template" switch applies to the shown version.
pub fn offers_template(data: &MessageViewerData, selection: MessageViewerSelection) -> bool {
    selected_version(data, selection).is_some_and(|v| v.template.is_some())
}

/// Whether the "With picture / Text only" switch applies to the shown
/// version.
pub fn offers_text_only(data: &MessageViewerData, selection: MessageViewerSelection) -> bool {
    data.text_only_variant
        && selected_version(data, selection).is_some_and(MessageVersion::has_picture)
}

/// The message the bubble shows for a selection:
///
/// - the version's template in template mode (when it has one), else its
///   as-sent text;
/// - without its picture when the text-only variant is selected;
/// - every image given an alt ([`picture_alt`]);
/// - on the viewer's channel, an SMS-family message becoming MMS exactly
///   when it carries a picture;
/// - laid out as sent, without an author (the chrome names the sender).
pub fn viewer_message(
    data: &MessageViewerData,
    selection: MessageViewerSelection,
    texts: &MessageViewerTexts,
) -> Option<RichMessage> {
    let version = selected_version(data, selection)?;
    let mut message = match (selection.mode, &version.template) {
        (MessageViewerMode::Template, Some(template)) => template.clone(),
        _ => version.sent.clone(),
    };
    if !selection.with_picture && offers_text_only(data, selection) {
        message.header = None;
    }
    message.channel = if data.channel.is_sms() {
        if has_image(&message) {
            MessageChannel::Mms
        } else {
            MessageChannel::Sms
        }
    } else {
        data.channel
    };
    if let Some(MessageHeader::Image { alt, .. }) = message.header.as_mut() {
        *alt = picture_alt(alt, message.channel, texts);
    }
    message.direction = MessageDirection::Out;
    message.author = None;
    Some(message)
}

/// The placeholder values the bubble gets: an empty table in template mode,
/// which turns recognition on and shows every placeholder as a chip; `None`
/// as sent, so a literal `{{` in a sent text stays text.
pub fn viewer_placeholders(
    data: &MessageViewerData,
    selection: MessageViewerSelection,
) -> Option<PlaceholderValues> {
    (selection.mode == MessageViewerMode::Template && offers_template(data, selection))
        .then(PlaceholderValues::new)
}

/// The chrome badge: "Template" or "Session message" on WhatsApp, "MMS" on a
/// picture message, nothing on a plain SMS or social message.
pub fn chrome_badge(
    data: &MessageViewerData,
    message: &RichMessage,
    texts: &MessageViewerTexts,
) -> Option<String> {
    match data.channel {
        MessageChannel::Whatsapp => Some(if data.whatsapp_template {
            texts.template_badge.clone()
        } else {
            texts.session_badge.clone()
        }),
        _ if message.channel == MessageChannel::Mms => Some(texts.mms.clone()),
        _ => None,
    }
}

/// Where a Tab inside a dialog should send focus instead of the browser's
/// choice, over its `len` tabbable elements in document order: from the last
/// to the first (Shift: from the first to the last), and in from outside
/// them. `None` lets the browser move focus, which stays inside.
pub fn focus_wrap_target(len: usize, current: Option<usize>, backwards: bool) -> Option<usize> {
    let last = len.checked_sub(1)?;
    match (current, backwards) {
        (None, false) => Some(0),
        (None, true) => Some(last),
        (Some(i), false) if i >= last => Some(0),
        (Some(0), true) => Some(last),
        _ => None,
    }
}

/// The reach line: "Sent to 132 · with picture 120 · text only 12".
pub fn counts_line(counts: MessageCounts, texts: &MessageViewerTexts) -> String {
    let fill = |template: &str, n: u32| template.replace("{n}", &n.to_string());
    [
        Some(fill(&texts.sent_to, counts.sent_to)),
        counts
            .with_picture
            .map(|n| fill(&texts.with_picture_count, n)),
        counts.text_only.map(|n| fill(&texts.text_only_count, n)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}
