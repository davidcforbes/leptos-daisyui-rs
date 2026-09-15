//! Helpdesk model: what a ticket, a detail view, a new request and the
//! backend's metadata look like. Pure data, serde-round-trippable so a host
//! can decode straight into it.

use serde::{Deserialize, Serialize};

/// A Jira issue key such as `OF-14`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketKey(pub String);

impl std::fmt::Display for TicketKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a ticket is. `Other` carries a Jira issue type the composite does
/// not name itself (a `Story` filed from Jira directly, say).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketKind {
    #[default]
    Bug,
    Request,
    Other(String),
}

/// Jira's `statusCategory.key`, the stable axis the buckets derive from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusCategory {
    New,
    InProgress,
    Done,
}

impl StatusCategory {
    /// Jira keys are `new`, `indeterminate` and `done`. An unknown key is
    /// treated as `New` so a ticket is never hidden from every bucket.
    pub fn from_jira_key(key: &str) -> Self {
        match key {
            "indeterminate" => Self::InProgress,
            "done" => Self::Done,
            _ => Self::New,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketStatus {
    pub id: String,
    pub name: String,
    pub category: StatusCategory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketPriority {
    Highest,
    High,
    Medium,
    Low,
    Lowest,
    Unset,
}

impl TicketPriority {
    /// Sort rank, most urgent first.
    pub fn rank(self) -> u8 {
        match self {
            Self::Highest => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
            Self::Lowest => 4,
            Self::Unset => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub display_name: String,
    pub initials: String,
}

impl Person {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        let display_name = display_name.into();
        let initials = initials_of(&display_name);
        Self {
            id: id.into(),
            display_name,
            initials,
        }
    }
}

fn initials_of(name: &str) -> String {
    let s: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect();
    if s.is_empty() { "?".to_owned() } else { s }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskTicket {
    pub key: TicketKey,
    pub summary: String,
    pub kind: TicketKind,
    pub status: TicketStatus,
    pub priority: TicketPriority,
    pub assignee: Option<Person>,
    pub requester: Option<Person>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub comment_count: u32,
    pub attachment_count: u32,
    /// Jira browse link, opened in a new tab from the drawer.
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketComment {
    pub id: String,
    pub author: Person,
    pub body: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketDetail {
    pub ticket: HelpdeskTicket,
    pub description: String,
    pub comments: Vec<TicketComment>,
    pub attachments: Vec<TicketAttachment>,
}

/// Where a request was filed from; the dialog shows it read-only.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    pub route: String,
    pub build: String,
}

/// An image the user attached. Bytes are already read; `content_type` is
/// the sniffed type, never the file's declared one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub filename: String,
    pub content_type: String,
    #[serde(with = "serde_bytes_vec")]
    pub bytes: Vec<u8>,
}

mod serde_bytes_vec {
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(b))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTicket {
    pub kind: TicketKind,
    pub summary: String,
    pub description: String,
    pub context: RequestContext,
    pub images: Vec<ImageAttachment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketScope {
    Mine,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityOption {
    pub id: String,
    pub priority: TicketPriority,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskMeta {
    pub configured: bool,
    /// Host-supplied, already localised.
    pub unavailable_reason: Option<String>,
    /// Workflow order.
    pub statuses: Vec<TicketStatus>,
    pub priorities: Vec<PriorityOption>,
    pub assignable: Vec<Person>,
    /// What the New Request dialog offers.
    pub kinds: Vec<TicketKind>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpdeskRole {
    #[default]
    Requester,
    Support,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpdeskErrorKind {
    NotConfigured,
    RateLimited { retry_after_s: u32 },
    Unauthorized,
    NotFound,
    Network,
    Upstream,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskError {
    pub kind: HelpdeskErrorKind,
    pub message: String,
}

impl HelpdeskError {
    pub fn new(kind: HelpdeskErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HelpdeskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HelpdeskError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_category_parses_jira_keys() {
        assert_eq!(StatusCategory::from_jira_key("new"), StatusCategory::New);
        assert_eq!(
            StatusCategory::from_jira_key("indeterminate"),
            StatusCategory::InProgress
        );
        assert_eq!(StatusCategory::from_jira_key("done"), StatusCategory::Done);
        assert_eq!(StatusCategory::from_jira_key("weird"), StatusCategory::New);
    }

    #[test]
    fn ticket_round_trips_through_serde() {
        let t = HelpdeskTicket {
            key: TicketKey("OF-14".into()),
            summary: "Dashboard blank".into(),
            kind: TicketKind::Bug,
            status: TicketStatus {
                id: "1".into(),
                name: "Backlog".into(),
                category: StatusCategory::New,
            },
            priority: TicketPriority::High,
            assignee: None,
            requester: Some(Person::new("w1", "Chris Forbes")),
            created_at_ms: 1_000,
            updated_at_ms: 2_000,
            comment_count: 0,
            attachment_count: 1,
            url: Some("https://x/browse/OF-14".into()),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: HelpdeskTicket = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn image_bytes_travel_as_base64() {
        let img = ImageAttachment {
            filename: "a.png".into(),
            content_type: "image/png".into(),
            bytes: vec![1, 2, 3],
        };
        let v = serde_json::to_value(&img).unwrap();
        assert_eq!(v["bytes"], "AQID");
        let back: ImageAttachment = serde_json::from_value(v).unwrap();
        assert_eq!(back, img);
    }

    #[test]
    fn person_initials_come_from_first_letters() {
        assert_eq!(Person::new("x", "Chris Forbes").initials, "CF");
        assert_eq!(Person::new("x", "Cher").initials, "C");
        assert_eq!(Person::new("x", "").initials, "?");
    }

    #[test]
    fn priority_orders_highest_first() {
        assert!(TicketPriority::Highest.rank() < TicketPriority::Lowest.rank());
        assert!(TicketPriority::Lowest.rank() < TicketPriority::Unset.rank());
    }
}
