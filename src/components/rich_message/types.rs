//! The neutral rich-message model (ldui-g8hh).
//!
//! Hosts map their own provider shapes into [`RichMessage`]: a Meta
//! WhatsApp template's HEADER/BODY/FOOTER/BUTTONS components, a Zoho Desk
//! canned message, a CTM SMS/MMS, or a stored conversation row. Nothing here
//! names a provider; timestamps arrive as display strings, so the component
//! never touches `std::time` (which panics in a browser Wasm build).

use std::collections::HashMap;

/// Which network the message travels on. It picks the text renderer: SMS
/// and MMS bodies are plain text, WhatsApp bodies use WhatsApp's inline
/// formatting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MessageChannel {
    /// Plain text message. A media header turns it into a picture message
    /// for the segment indicator.
    #[default]
    Sms,
    /// Picture (multimedia) message.
    Mms,
    /// WhatsApp message or template, with `*bold*`, `_italic_`, `~strike~`
    /// and triple-backtick monospace.
    Whatsapp,
}

impl MessageChannel {
    /// The `data-channel` value on the message root.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sms => "sms",
            Self::Mms => "mms",
            Self::Whatsapp => "whatsapp",
        }
    }
}

/// Who the message belongs to. `Preview` is "what the recipient will see":
/// it is laid out as a received message, its actions are inert, and its
/// placeholders are highlighted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MessageDirection {
    /// Received from the contact.
    In,
    /// Sent by the business.
    Out,
    /// A composer or template preview.
    #[default]
    Preview,
}

impl MessageDirection {
    /// The `data-direction` value on the message root.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
            Self::Preview => "preview",
        }
    }
}

/// The part above the body: a WhatsApp template HEADER, or an MMS picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageHeader {
    /// A bold text line.
    Text(String),
    /// A picture. `width`/`height` are the intrinsic size, used only to
    /// reserve the aspect ratio before the image loads.
    Image {
        /// Image URL, used as given.
        src: String,
        /// Alt text; also names the fallback tile.
        alt: String,
        /// Intrinsic width in pixels.
        width: Option<u32>,
        /// Intrinsic height in pixels.
        height: Option<u32>,
    },
    /// A video. It never preloads; the host's `src` is fetched only on play.
    Video {
        /// Video URL, used as given.
        src: String,
        /// Still image shown before play.
        poster: Option<String>,
        /// Accessible name; also names the fallback tile.
        alt: String,
    },
    /// A document attachment, shown as a file tile. With a `src` the name is
    /// a link that opens in a new tab.
    Document {
        /// File name shown on the tile.
        name: String,
        /// Where the file opens, if anywhere.
        src: Option<String>,
    },
}

impl MessageHeader {
    /// True for every header except [`MessageHeader::Text`].
    pub fn is_media(&self) -> bool {
        !matches!(self, Self::Text(_))
    }
}

/// A button under the message. Every kind renders as a `<button>` that never
/// navigates by itself: without an `on_action` callback it is an
/// `aria-disabled` preview, and with one the host decides what a press does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageAction {
    /// "Visit website".
    Url {
        /// Button label.
        label: String,
        /// The website the recipient's button opens.
        href: String,
    },
    /// "Call now".
    Call {
        /// Button label.
        label: String,
        /// The number the recipient's button dials.
        tel: String,
    },
    /// "Copy code".
    CopyCode {
        /// Button label.
        label: String,
        /// The code the recipient's button copies.
        code: String,
    },
    /// A quick-reply button ("Push to respond", "Yes", "Stop").
    QuickReply {
        /// Button label, which is also the reply text.
        label: String,
    },
}

impl MessageAction {
    /// The visible button label.
    pub fn label(&self) -> &str {
        match self {
            Self::Url { label, .. }
            | Self::Call { label, .. }
            | Self::CopyCode { label, .. }
            | Self::QuickReply { label } => label,
        }
    }

    /// The `data-message-action` value on the button.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Url { .. } => "url",
            Self::Call { .. } => "call",
            Self::CopyCode { .. } => "copy-code",
            Self::QuickReply { .. } => "quick-reply",
        }
    }

    /// Where the button leads (the URL, number or code), shown as the
    /// button's tooltip so a preview says what the recipient's button does.
    /// `None` for a quick reply.
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Url { href, .. } => Some(href),
            Self::Call { tel, .. } => Some(tel),
            Self::CopyCode { code, .. } => Some(code),
            Self::QuickReply { .. } => None,
        }
    }
}

/// The host showed only part of the body: `shown_len` of `true_len`
/// characters (Office caps stored content at 4,000).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageTruncation {
    /// Characters shown.
    pub shown_len: usize,
    /// Characters in the whole message.
    pub true_len: usize,
}

/// One message, in the neutral shape every channel maps into.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichMessage {
    /// The network, which picks the text renderer.
    pub channel: MessageChannel,
    /// Received, sent or preview.
    pub direction: MessageDirection,
    /// Display name or number of the sender.
    pub author: Option<String>,
    /// Display time, already formatted by the host.
    pub time: Option<String>,
    /// Text or media above the body.
    pub header: Option<MessageHeader>,
    /// Message text. Line breaks are kept; emoji and non-Latin text render
    /// as given.
    pub body: String,
    /// The small line under the body (a WhatsApp template FOOTER).
    pub footer: Option<String>,
    /// Buttons under the message.
    pub actions: Vec<MessageAction>,
    /// Quick replies offered as plain labels, rendered after `actions`.
    pub quick_replies: Vec<String>,
    /// Set when the host shows only part of the body.
    pub truncated: Option<MessageTruncation>,
}

impl RichMessage {
    /// True when the segment indicator should describe this message as a
    /// picture message: an MMS, or an SMS that carries a media header.
    pub fn is_picture_message(&self) -> bool {
        match self.channel {
            MessageChannel::Mms => true,
            MessageChannel::Sms => self.header.as_ref().is_some_and(MessageHeader::is_media),
            MessageChannel::Whatsapp => false,
        }
    }
}

/// Values for body placeholders, keyed by the text inside the delimiters:
/// `first_name` for `{{first_name}}`, `1` for `{{1}}`, and
/// `Contacts.First Name` for `${Contacts.First Name}`. Keys match exactly
/// after trimming the surrounding spaces.
pub type PlaceholderValues = HashMap<String, String>;

/// Every string the rich-message components render. `Default` is English;
/// [`RichMessageTexts::es`] is Spanish. A host overrides single fields by
/// struct-update syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichMessageTexts {
    /// Accessible-name prefix of a received message.
    pub received: String,
    /// Accessible-name prefix of a sent message.
    pub sent: String,
    /// Accessible-name prefix of a preview.
    pub preview: String,
    /// Fallback tile text when an image fails to load.
    pub image_unavailable: String,
    /// Fallback tile text when a video fails to load.
    pub video_unavailable: String,
    /// Accessible prefix of a document tile.
    pub document: String,
    /// Segment indicator: a picture message.
    pub picture_message: String,
    /// Segment indicator: the GSM 7-bit encoding.
    pub gsm7: String,
    /// Segment indicator: the UCS-2 (16-bit) encoding.
    pub ucs2: String,
    /// Segment indicator: one segment.
    pub segment: String,
    /// Segment indicator: several segments.
    pub segments: String,
    /// Segment indicator: the character-count unit.
    pub chars: String,
    /// Badge on a template the host cannot send.
    pub not_sendable: String,
    /// Screen-reader suffix of an unfilled placeholder chip.
    pub unfilled: String,
    /// Accessible name of the button group.
    pub actions: String,
    /// Description of an inert preview button.
    pub actions_preview_hint: String,
    /// Truncation note; `{shown}` and `{total}` are replaced with numbers.
    pub truncated: String,
    /// Screen-reader suffix of a link that opens a new tab.
    pub opens_new_tab: String,
}

impl Default for RichMessageTexts {
    fn default() -> Self {
        Self {
            received: "Received".into(),
            sent: "Sent".into(),
            preview: "Preview".into(),
            image_unavailable: "Image not available".into(),
            video_unavailable: "Video not available".into(),
            document: "Document".into(),
            picture_message: "Picture message".into(),
            gsm7: "GSM-7".into(),
            ucs2: "UCS-2".into(),
            segment: "segment".into(),
            segments: "segments".into(),
            chars: "chars".into(),
            not_sendable: "Not sendable".into(),
            unfilled: "unfilled".into(),
            actions: "Message buttons".into(),
            actions_preview_hint: "Preview only: this is the recipient's button".into(),
            truncated: "Showing {shown} of {total} characters".into(),
            opens_new_tab: "(opens in a new tab)".into(),
        }
    }
}

impl RichMessageTexts {
    /// The Spanish table.
    pub fn es() -> Self {
        Self {
            received: "Recibido".into(),
            sent: "Enviado".into(),
            preview: "Vista previa".into(),
            image_unavailable: "Imagen no disponible".into(),
            video_unavailable: "Video no disponible".into(),
            document: "Documento".into(),
            picture_message: "Mensaje con imagen".into(),
            gsm7: "GSM-7".into(),
            ucs2: "UCS-2".into(),
            segment: "segmento".into(),
            segments: "segmentos".into(),
            chars: "caracteres".into(),
            not_sendable: "No se puede enviar".into(),
            unfilled: "sin completar".into(),
            actions: "Botones del mensaje".into(),
            actions_preview_hint: "Solo vista previa: es el botón del destinatario".into(),
            truncated: "Se muestran {shown} de {total} caracteres".into(),
            opens_new_tab: "(se abre en una pestaña nueva)".into(),
        }
    }

    /// The direction's accessible-name prefix.
    pub fn direction(&self, direction: MessageDirection) -> &str {
        match direction {
            MessageDirection::In => &self.received,
            MessageDirection::Out => &self.sent,
            MessageDirection::Preview => &self.preview,
        }
    }
}

/// The accessible name of a message: direction, author and time, comma
/// separated, skipping whichever are absent.
pub fn message_label(message: &RichMessage, texts: &RichMessageTexts) -> String {
    [
        Some(texts.direction(message.direction)),
        message.author.as_deref(),
        message.time.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(", ")
}

/// The truncation note with its numbers filled in.
pub fn truncation_note(truncation: MessageTruncation, texts: &RichMessageTexts) -> String {
    texts
        .truncated
        .replace("{shown}", &truncation.shown_len.to_string())
        .replace("{total}", &truncation.true_len.to_string())
}

/// The URL with every ASCII whitespace and control character removed and
/// lowercased: browsers ignore a tab or newline inside a scheme, so
/// `java`, tab, `script:` is still a script URL.
fn normalised_url(url: &str) -> String {
    url.chars()
        .filter(|c| !c.is_ascii_whitespace() && !c.is_ascii_control())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// The URL's scheme, when it has one: text before a `:` that comes before
/// any `/`, `?` or `#`.
fn url_scheme(normalised: &str) -> Option<&str> {
    let end = normalised.find([':', '/', '?', '#'])?;
    normalised[end..]
        .starts_with(':')
        .then(|| &normalised[..end])
}

/// Whether `url` may become a link (`href`): `http`, `https`, `blob` or a
/// relative URL. Header URLs come from inbound messages, so a `javascript:`,
/// `vbscript:` or `data:` URL is refused rather than rendered (ldui-g8hh).
pub fn is_safe_link_url(url: &str) -> bool {
    let normalised = normalised_url(url);
    !normalised.is_empty()
        && url_scheme(&normalised).is_none_or(|scheme| matches!(scheme, "http" | "https" | "blob"))
}

/// Whether `url` may be a media `src` or `poster`: anything
/// [`is_safe_link_url`] allows, plus inline `data:image/`, `data:video/`
/// and `data:audio/` URLs. A refused URL shows the fallback tile.
pub fn is_safe_media_url(url: &str) -> bool {
    let normalised = normalised_url(url);
    is_safe_link_url(url)
        || ["data:image/", "data:video/", "data:audio/"]
            .iter()
            .any(|prefix| normalised.starts_with(prefix))
}

/// What a failed media header's fallback tile names: the alt text, else the
/// file name at the end of `src` (query and fragment dropped), else `src`.
pub fn media_fallback_name(src: &str, alt: &str) -> String {
    let alt = alt.trim();
    if !alt.is_empty() {
        return alt.to_string();
    }
    let path = src.split(['?', '#']).next().unwrap_or_default();
    path.rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(src)
        .to_string()
}
