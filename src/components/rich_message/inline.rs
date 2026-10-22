//! Body text to styled runs: placeholders, WhatsApp inline formatting and
//! optional auto-linking.
//!
//! Placeholders are found first and kept as opaque atoms, so the underscore
//! in `{{first_name}}` can never open an italic run. WhatsApp formatting then
//! follows WhatsApp's own rules, not Markdown's:
//! - `*bold*`, `_italic_` and `~strike~` open after the start of the text,
//!   whitespace or punctuation, must be followed by a non-space, close on a
//!   marker preceded by a non-space and followed by the end, whitespace or
//!   punctuation, and never cross a line break;
//! - they nest (`*_both_*`);
//! - ```` ```mono``` ```` may span lines, and nothing inside it is formatted.

/// A body split into literal text and placeholders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    /// Literal text.
    Text(String),
    /// `raw` is the placeholder as written (`{{first_name}}`); `key` is the
    /// trimmed name inside the delimiters (`first_name`).
    Placeholder {
        /// As written.
        raw: String,
        /// Trimmed name.
        key: String,
    },
}

/// The formatting a run carries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineStyle {
    /// `*bold*`.
    pub bold: bool,
    /// `_italic_`.
    pub italic: bool,
    /// `~strike~`.
    pub strike: bool,
    /// Triple-backtick monospace.
    pub mono: bool,
}

/// What a run holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunContent {
    /// Literal text.
    Text(String),
    /// A placeholder; see [`Piece::Placeholder`].
    Placeholder {
        /// As written.
        raw: String,
        /// Trimmed name.
        key: String,
    },
    /// An auto-linked URL or email address.
    Link {
        /// The text as written.
        text: String,
        /// The link target (`https://` or `mailto:` added where needed).
        href: String,
    },
}

/// A maximal stretch of one style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// The run's formatting.
    pub style: InlineStyle,
    /// The run's content.
    pub content: RunContent,
}

/// Longest placeholder name accepted, so a stray `{{` cannot swallow a
/// paragraph.
const MAX_PLACEHOLDER_KEY: usize = 64;

fn valid_key(inner: &str) -> Option<String> {
    let key = inner.trim();
    let ok = !key.is_empty()
        && key.chars().count() <= MAX_PLACEHOLDER_KEY
        && !key.contains(['{', '}', '\n', '\r']);
    ok.then(|| key.to_string())
}

/// Splits `{{name}}`, `{{1}}` and `${Contacts.First Name}` out of `text`.
/// A delimiter with no valid close is left as text.
pub fn split_placeholders(text: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut literal = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        let found = if let Some(after) = rest.strip_prefix("{{") {
            after
                .find("}}")
                .and_then(|end| valid_key(&after[..end]).map(|key| (key, 2 + end + 2)))
        } else if let Some(after) = rest.strip_prefix("${") {
            after
                .find('}')
                .and_then(|end| valid_key(&after[..end]).map(|key| (key, 2 + end + 1)))
        } else {
            None
        };
        match found {
            Some((key, len)) => {
                if !literal.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut literal)));
                }
                pieces.push(Piece::Placeholder {
                    raw: rest[..len].to_string(),
                    key,
                });
                rest = &rest[len..];
            }
            None => {
                let c = rest.chars().next().expect("rest is not empty");
                literal.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    if !literal.is_empty() {
        pieces.push(Piece::Text(literal));
    }
    pieces
}

#[derive(Clone, Copy)]
enum Atom {
    Ch(char),
    /// Index into the placeholder list.
    Ph(usize),
}

fn is_space(atom: Option<&Atom>) -> bool {
    matches!(atom, Some(Atom::Ch(c)) if c.is_whitespace())
}

/// A placeholder counts as a word character.
fn is_word(atom: Option<&Atom>) -> bool {
    match atom {
        Some(Atom::Ch(c)) => c.is_alphanumeric(),
        Some(Atom::Ph(_)) => true,
        None => false,
    }
}

fn is_char(atom: Option<&Atom>, want: char) -> bool {
    matches!(atom, Some(Atom::Ch(c)) if *c == want)
}

struct Formatter<'a> {
    atoms: Vec<Atom>,
    placeholders: Vec<(String, String)>,
    runs: &'a mut Vec<Run>,
}

impl Formatter<'_> {
    fn push_text(&mut self, text: &mut String, style: InlineStyle) {
        if !text.is_empty() {
            self.runs.push(Run {
                style,
                content: RunContent::Text(std::mem::take(text)),
            });
        }
    }

    fn push_placeholder(&mut self, index: usize, style: InlineStyle) {
        let (raw, key) = self.placeholders[index].clone();
        self.runs.push(Run {
            style,
            content: RunContent::Placeholder { raw, key },
        });
    }

    fn triple_at(&self, i: usize, end: usize) -> bool {
        i + 3 <= end && (i..i + 3).all(|k| is_char(self.atoms.get(k), '`'))
    }

    /// A `*`, `_` or `~` at `i` that may open a run.
    fn opens(&self, i: usize, end: usize, marker: char) -> bool {
        let before = i.checked_sub(1).and_then(|p| self.atoms.get(p));
        let after = (i + 1 < end).then(|| self.atoms.get(i + 1)).flatten();
        !is_word(before) && after.is_some() && !is_space(after) && !is_char(after, marker)
    }

    /// The first marker after `open` that closes it on the same line.
    fn closing(&self, open: usize, end: usize, marker: char) -> Option<usize> {
        for k in open + 2..end {
            match self.atoms[k] {
                Atom::Ch('\n') => return None,
                Atom::Ch(c)
                    if c == marker
                        && !is_space(self.atoms.get(k - 1))
                        && !is_word(self.atoms.get(k + 1)) =>
                {
                    return Some(k);
                }
                _ => {}
            }
        }
        None
    }

    fn parse(&mut self, start: usize, end: usize, style: InlineStyle) {
        let mut text = String::new();
        let mut i = start;
        while i < end {
            match self.atoms[i] {
                Atom::Ph(index) => {
                    self.push_text(&mut text, style);
                    self.push_placeholder(index, style);
                    i += 1;
                }
                Atom::Ch('`') if self.triple_at(i, end) => {
                    let close = (i + 4..end).find(|&k| self.triple_at(k, end));
                    match close {
                        Some(close) => {
                            self.push_text(&mut text, style);
                            let mono = InlineStyle {
                                mono: true,
                                ..style
                            };
                            let mut inner = String::new();
                            for k in i + 3..close {
                                match self.atoms[k] {
                                    Atom::Ch(c) => inner.push(c),
                                    Atom::Ph(index) => {
                                        self.push_text(&mut inner, mono);
                                        self.push_placeholder(index, mono);
                                    }
                                }
                            }
                            self.push_text(&mut inner, mono);
                            i = close + 3;
                        }
                        None => {
                            text.push_str("```");
                            i += 3;
                        }
                    }
                }
                Atom::Ch(c @ ('*' | '_' | '~')) => {
                    let close = self
                        .opens(i, end, c)
                        .then(|| self.closing(i, end, c))
                        .flatten();
                    match close {
                        Some(close) => {
                            self.push_text(&mut text, style);
                            let inner = match c {
                                '*' => InlineStyle {
                                    bold: true,
                                    ..style
                                },
                                '_' => InlineStyle {
                                    italic: true,
                                    ..style
                                },
                                _ => InlineStyle {
                                    strike: true,
                                    ..style
                                },
                            };
                            self.parse(i + 1, close, inner);
                            i = close + 1;
                        }
                        None => {
                            text.push(c);
                            i += 1;
                        }
                    }
                }
                Atom::Ch(c) => {
                    text.push(c);
                    i += 1;
                }
            }
        }
        self.push_text(&mut text, style);
    }
}

/// Applies WhatsApp formatting to already-split pieces.
pub fn whatsapp_runs(pieces: &[Piece]) -> Vec<Run> {
    let mut atoms = Vec::new();
    let mut placeholders = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Text(text) => atoms.extend(text.chars().map(Atom::Ch)),
            Piece::Placeholder { raw, key } => {
                atoms.push(Atom::Ph(placeholders.len()));
                placeholders.push((raw.clone(), key.clone()));
            }
        }
    }
    let mut runs = Vec::new();
    let end = atoms.len();
    Formatter {
        atoms,
        placeholders,
        runs: &mut runs,
    }
    .parse(0, end, InlineStyle::default());
    runs
}

fn plain_runs(pieces: Vec<Piece>) -> Vec<Run> {
    pieces
        .into_iter()
        .map(|piece| Run {
            style: InlineStyle::default(),
            content: match piece {
                Piece::Text(text) => RunContent::Text(text),
                Piece::Placeholder { raw, key } => RunContent::Placeholder { raw, key },
            },
        })
        .collect()
}

/// The link a word stands for: an `http(s)://` URL, a `www.` host, or an
/// email address.
fn link_href(word: &str) -> Option<String> {
    if word.contains(['"', '\'', '<', '>', '`']) {
        return None;
    }
    let lower = word.to_ascii_lowercase();
    if (lower.starts_with("https://") || lower.starts_with("http://"))
        && word
            .split_once("://")
            .is_some_and(|(_, rest)| !rest.is_empty())
    {
        return Some(word.to_string());
    }
    if lower.starts_with("www.") && word[4..].contains('.') && !word.ends_with('.') {
        return Some(format!("https://{word}"));
    }
    let (local, domain) = word.split_once('@')?;
    let local_ok = !local.is_empty()
        && local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));
    let domain_ok = domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
    (local_ok && domain_ok).then(|| format!("mailto:{word}"))
}

const LEADING_PUNCTUATION: &[char] = &['(', '<', '[', '"', '\''];
const TRAILING_PUNCTUATION: &[char] = &['.', ',', ';', ':', '!', '?', ')', '>', ']', '"', '\''];

/// Splits one text run into text and links, keeping every character.
fn linkify_text(text: &str, style: InlineStyle, out: &mut Vec<Run>) {
    let mut plain = String::new();
    let flush = |plain: &mut String, out: &mut Vec<Run>| {
        if !plain.is_empty() {
            out.push(Run {
                style,
                content: RunContent::Text(std::mem::take(plain)),
            });
        }
    };
    for token in text.split_inclusive(char::is_whitespace) {
        let word_end = token.trim_end_matches(char::is_whitespace).len();
        let (word, space) = token.split_at(word_end);
        let core_start = word.len() - word.trim_start_matches(LEADING_PUNCTUATION).len();
        let core_end = word
            .trim_end_matches(TRAILING_PUNCTUATION)
            .len()
            .max(core_start);
        let core = &word[core_start..core_end];
        match link_href(core) {
            Some(href) => {
                plain.push_str(&word[..core_start]);
                flush(&mut plain, out);
                out.push(Run {
                    style,
                    content: RunContent::Link {
                        text: core.to_string(),
                        href,
                    },
                });
                plain.push_str(&word[core_end..]);
            }
            None => plain.push_str(word),
        }
        plain.push_str(space);
    }
    flush(&mut plain, out);
}

/// The full pipeline: placeholders (when `placeholders`), WhatsApp
/// formatting (when `whatsapp`), then links outside monospace (when
/// `linkify`).
pub fn inline_runs(body: &str, whatsapp: bool, placeholders: bool, linkify: bool) -> Vec<Run> {
    let pieces = if placeholders {
        split_placeholders(body)
    } else if body.is_empty() {
        Vec::new()
    } else {
        vec![Piece::Text(body.to_string())]
    };
    let runs = if whatsapp {
        whatsapp_runs(&pieces)
    } else {
        plain_runs(pieces)
    };
    if !linkify {
        return runs;
    }
    let mut linked = Vec::with_capacity(runs.len());
    for run in runs {
        match run.content {
            RunContent::Text(text) if !run.style.mono => {
                linkify_text(&text, run.style, &mut linked)
            }
            content => linked.push(Run {
                style: run.style,
                content,
            }),
        }
    }
    linked
}
