//! SMS encoding and segment counting (3GPP TS 23.038).
//!
//! A body that uses only the GSM 7-bit default alphabet (plus its extension
//! table, where each character costs two septets) travels as GSM-7: 160
//! septets in one segment, 153 per segment once it is split. Anything else
//! (an emoji, a curly quote, a non-Latin letter) forces UCS-2: 70 code units
//! in one segment, 67 per segment once split. Segments are packed the way a
//! carrier packs them: an extension escape pair or a UTF-16 surrogate pair is
//! never split across a segment boundary.

use unicode_segmentation::UnicodeSegmentation;

use super::types::RichMessageTexts;

/// The GSM 7-bit default alphabet, minus the escape code itself.
const GSM7_BASIC: &str = "@£$¥èéùìòÇ\nØø\rÅåΔ_ΦΓΛΩΠΨΣΘΞÆæßÉ !\"#¤%&'()*+,-./0123456789:;<=>?¡ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÑÜ§¿abcdefghijklmnopqrstuvwxyzäöñüà";

/// The extension table: each costs an escape plus the character.
const GSM7_EXTENSION: &str = "\u{0C}^{}\\[~]|€";

const GSM7_SINGLE: usize = 160;
const GSM7_MULTI: usize = 153;
const UCS2_SINGLE: usize = 70;
const UCS2_MULTI: usize = 67;

/// How the body would be encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmsEncoding {
    /// The GSM 7-bit default alphabet.
    Gsm7,
    /// 16-bit UCS-2.
    Ucs2,
}

impl SmsEncoding {
    /// The `data-sms-encoding` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gsm7 => "gsm7",
            Self::Ucs2 => "ucs2",
        }
    }
}

/// What [`sms_segments`] measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmsSegmentInfo {
    /// The encoding the body needs.
    pub encoding: SmsEncoding,
    /// Segments the body needs; 0 for an empty body.
    pub segments: usize,
    /// Encoded length: septets for GSM-7, UTF-16 code units for UCS-2.
    pub units: usize,
    /// User-perceived characters (grapheme clusters), so an emoji with a
    /// skin-tone modifier counts once.
    pub chars: usize,
}

/// Septets one character costs in GSM-7, or `None` when it is outside the
/// alphabet.
fn gsm7_cost(c: char) -> Option<usize> {
    if GSM7_BASIC.contains(c) {
        Some(1)
    } else if GSM7_EXTENSION.contains(c) {
        Some(2)
    } else {
        None
    }
}

/// Segments needed for characters of the given costs.
fn pack(costs: impl Iterator<Item = usize>, total: usize, single: usize, multi: usize) -> usize {
    if total == 0 {
        return 0;
    }
    if total <= single {
        return 1;
    }
    let mut segments = 1;
    let mut used = 0;
    for cost in costs {
        if used + cost > multi {
            segments += 1;
            used = 0;
        }
        used += cost;
    }
    segments
}

/// Encoding, segment count and lengths of an SMS body.
pub fn sms_segments(text: &str) -> SmsSegmentInfo {
    let chars = text.graphemes(true).count();
    let gsm: Option<Vec<usize>> = text.chars().map(gsm7_cost).collect();
    match gsm {
        Some(costs) => {
            let units = costs.iter().sum();
            SmsSegmentInfo {
                encoding: SmsEncoding::Gsm7,
                segments: pack(costs.into_iter(), units, GSM7_SINGLE, GSM7_MULTI),
                units,
                chars,
            }
        }
        None => {
            let units = text.encode_utf16().count();
            SmsSegmentInfo {
                encoding: SmsEncoding::Ucs2,
                segments: pack(
                    text.chars().map(char::len_utf16),
                    units,
                    UCS2_SINGLE,
                    UCS2_MULTI,
                ),
                units,
                chars,
            }
        }
    }
}

/// The indicator line: "UCS-2 · 11 segments · 712 chars", or
/// "Picture message · 712 chars" for a picture message (an MMS is not
/// segmented).
pub fn sms_indicator_text(info: SmsSegmentInfo, picture: bool, texts: &RichMessageTexts) -> String {
    if picture {
        return format!("{} · {} {}", texts.picture_message, info.chars, texts.chars);
    }
    let encoding = match info.encoding {
        SmsEncoding::Gsm7 => &texts.gsm7,
        SmsEncoding::Ucs2 => &texts.ucs2,
    };
    let unit = if info.segments == 1 {
        &texts.segment
    } else {
        &texts.segments
    };
    format!(
        "{encoding} · {} {unit} · {} {}",
        info.segments, info.chars, texts.chars
    )
}
