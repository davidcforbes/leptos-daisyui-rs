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
    /// A defect report.
    #[default]
    Bug,
    /// A feature or change request.
    Request,
    /// A Jira issue type the composite does not offer in the New Request
    /// dialog, named as Jira names it.
    Other(String),
}

/// Jira's `statusCategory.key`, the stable axis the buckets derive from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusCategory {
    /// Not yet started (Jira's `new`).
    New,
    /// Underway (Jira's `indeterminate`).
    InProgress,
    /// Complete (Jira's `done`).
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

/// One Jira workflow status: its id, display name, and category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketStatus {
    /// Jira status id, used when transitioning.
    pub id: String,
    /// Display name, e.g. "Backlog".
    pub name: String,
    /// The bucket-deriving category.
    pub category: StatusCategory,
}

/// A ticket's urgency, ordered by [`TicketPriority::rank`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketPriority {
    /// Highest urgency.
    Highest,
    /// High urgency.
    High,
    /// Medium urgency.
    Medium,
    /// Low urgency.
    Low,
    /// Lowest urgency.
    Lowest,
    /// No priority named by Jira, or one this composite does not recognise.
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

/// Someone the composite renders: a requester, an assignee, a comment author.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    /// Stable identifier (a worker ref, or a Jira account id).
    pub id: String,
    /// Full display name.
    pub display_name: String,
    /// Up to two initials, derived from `display_name` by [`Person::new`].
    pub initials: String,
}

impl Person {
    /// Build a `Person`, deriving `initials` from `display_name`.
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

/// One row of the board: everything the list and the buckets need, without
/// the description, comments or attachments (see [`TicketDetail`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskTicket {
    /// The Jira issue key.
    pub key: TicketKey,
    /// The issue summary.
    pub summary: String,
    /// Bug or Request (or an unrecognised Jira issue type).
    pub kind: TicketKind,
    /// Current workflow status.
    pub status: TicketStatus,
    /// Current priority.
    pub priority: TicketPriority,
    /// Who it is assigned to, if anyone.
    pub assignee: Option<Person>,
    /// Who filed it, from the host's requester ledger (never Jira's own
    /// reporter, which is the service account).
    pub requester: Option<Person>,
    /// Creation time, in epoch milliseconds.
    pub created_at_ms: i64,
    /// Last-updated time, in epoch milliseconds.
    pub updated_at_ms: i64,
    /// How many comments the issue holds.
    pub comment_count: u32,
    /// How many attachments the issue holds.
    pub attachment_count: u32,
    /// Jira browse link, opened in a new tab from the drawer.
    pub url: Option<String>,
}

/// One comment on a ticket, already flattened from Jira's ADF to plain text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketComment {
    /// Jira comment id.
    pub id: String,
    /// Who wrote it.
    pub author: Person,
    /// Plain-text body.
    pub body: String,
    /// When it was posted, in epoch milliseconds.
    pub created_at_ms: i64,
}

/// One file attached to a ticket. `url` and `thumbnail_url` are host-owned
/// (never Jira's own attachment URLs, which need the service credential).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketAttachment {
    /// Jira attachment id.
    pub id: String,
    /// Original filename.
    pub filename: String,
    /// MIME type Jira recorded.
    pub content_type: String,
    /// Where the composite fetches the full image from.
    pub url: String,
    /// Where the composite fetches a smaller preview from, if the host
    /// offers one.
    pub thumbnail_url: Option<String>,
}

/// A ticket plus the fields only the detail drawer needs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketDetail {
    /// The row-level fields.
    pub ticket: HelpdeskTicket,
    /// Plain-text description.
    pub description: String,
    /// Comments, oldest first.
    pub comments: Vec<TicketComment>,
    /// Attachments.
    pub attachments: Vec<TicketAttachment>,
}

/// Where a request was filed from; the dialog shows it read-only.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    /// The host's current route (an SPA pathname).
    pub route: String,
    /// The host's build label, for triage.
    pub build: String,
}

/// An image the user attached. Bytes are already read; `content_type` is
/// the sniffed type, never the file's declared one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAttachment {
    /// The name to attach it under.
    pub filename: String,
    /// The sniffed MIME type (`image/png`, `image/jpeg` or `image/webp`).
    pub content_type: String,
    /// Raw bytes, base64-encoded on the wire.
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

/// The New Request dialog's submission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTicket {
    /// Bug or Request.
    pub kind: TicketKind,
    /// The issue summary.
    pub summary: String,
    /// The issue description.
    pub description: String,
    /// Where the request was filed from.
    pub context: RequestContext,
    /// Attached screenshots.
    pub images: Vec<ImageAttachment>,
}

/// Which tickets a list call should return.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketScope {
    /// Only the caller's own tickets.
    Mine,
    /// Every ticket (support only).
    All,
}

/// A priority the host offers, with Jira's id alongside the display enum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityOption {
    /// Jira priority id, used when setting it.
    pub id: String,
    /// The display/sort variant.
    pub priority: TicketPriority,
    /// Display name.
    pub name: String,
}

/// What the host's backend can tell the composite about itself: whether it
/// is set up, and the vocabularies the board and dialog need.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskMeta {
    /// Whether the helpdesk is reachable at all.
    pub configured: bool,
    /// Host-supplied, already localised.
    pub unavailable_reason: Option<String>,
    /// Workflow order.
    pub statuses: Vec<TicketStatus>,
    /// Offered priorities, in the host's preferred order.
    pub priorities: Vec<PriorityOption>,
    /// Who support may assign a ticket to.
    pub assignable: Vec<Person>,
    /// What the New Request dialog offers.
    pub kinds: Vec<TicketKind>,
}

/// Which mode the composite renders in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpdeskRole {
    /// Sees and files only their own tickets.
    #[default]
    Requester,
    /// Sees every ticket and may triage it.
    Support,
}

/// What kind of problem a [`HelpdeskError`] reports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpdeskErrorKind {
    /// The helpdesk has not been set up on this deployment.
    NotConfigured,
    /// Too many requests; retry after the given number of seconds.
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_s: u32,
    },
    /// The caller is not allowed to do this.
    Unauthorized,
    /// The ticket does not exist, or the caller may not see it.
    NotFound,
    /// The request never reached an answer.
    Network,
    /// The host or Jira refused the request for some other reason.
    Upstream,
}

/// An error the composite can render without inspecting a string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpdeskError {
    /// The typed category.
    pub kind: HelpdeskErrorKind,
    /// A scrubbed, display-safe message.
    pub message: String,
}

impl HelpdeskError {
    /// Build an error from its kind and a display-safe message.
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
