//! Pure derivations for the helpdesk composite: buckets, filtering, age
//! formatting, validation and role capabilities. Nothing here touches the
//! DOM, so every rule is native-testable.

use super::model::{
    HelpdeskRole, HelpdeskTicket, NewTicket, StatusCategory, TicketPriority, TicketScope,
};

/// The three category buckets the board offers. Jira's own five statuses
/// are a column and a drawer select, never a bucket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket {
    Open,
    InProgress,
    Done,
}

impl Bucket {
    pub fn of(category: StatusCategory) -> Self {
        match category {
            StatusCategory::New => Self::Open,
            StatusCategory::InProgress => Self::InProgress,
            StatusCategory::Done => Self::Done,
        }
    }

    /// Stable id used in `data-helpdesk-bucket` and the summary group.
    pub fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in-progress",
            Self::Done => "done",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "open" => Some(Self::Open),
            "in-progress" => Some(Self::InProgress),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BucketCounts {
    pub open: u64,
    pub in_progress: u64,
    pub done: u64,
}

impl BucketCounts {
    pub fn total(self) -> u64 {
        self.open + self.in_progress + self.done
    }
}

pub fn bucket_counts(tickets: &[HelpdeskTicket]) -> BucketCounts {
    tickets.iter().fold(BucketCounts::default(), |mut c, t| {
        match Bucket::of(t.status.category) {
            Bucket::Open => c.open += 1,
            Bucket::InProgress => c.in_progress += 1,
            Bucket::Done => c.done += 1,
        }
        c
    })
}

/// The local filter state. `mine_only` means assigned to the current user.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TicketFilter {
    pub bucket: Option<Bucket>,
    pub search: String,
    pub priority: Option<TicketPriority>,
    pub assignee_id: Option<String>,
    pub mine_only: bool,
}

impl TicketFilter {
    pub fn is_empty(&self) -> bool {
        self.bucket.is_none()
            && self.search.trim().is_empty()
            && self.priority.is_none()
            && self.assignee_id.is_none()
            && !self.mine_only
    }

    /// `me` is the current user's id; "mine" means assigned to me. With no
    /// identity, a mine-only filter matches nothing rather than everything.
    pub fn matches(&self, t: &HelpdeskTicket, me: Option<&str>) -> bool {
        if let Some(b) = self.bucket
            && Bucket::of(t.status.category) != b
        {
            return false;
        }
        let q = self.search.trim().to_lowercase();
        if !q.is_empty()
            && !t.key.0.to_lowercase().contains(&q)
            && !t.summary.to_lowercase().contains(&q)
        {
            return false;
        }
        if let Some(p) = self.priority
            && t.priority != p
        {
            return false;
        }
        let assignee = t.assignee.as_ref().map(|x| x.id.as_str());
        if let Some(a) = &self.assignee_id
            && assignee != Some(a.as_str())
        {
            return false;
        }
        if self.mine_only {
            let Some(me) = me else { return false };
            if assignee != Some(me) {
                return false;
            }
        }
        true
    }
}

/// Compact age: `0m`, `5m`, `3h`, `2d`, `3w`. Future timestamps clamp to `0m`.
pub fn relative_age(now_ms: i64, then_ms: i64) -> String {
    let mins = (now_ms - then_ms).max(0) / 60_000;
    if mins < 60 {
        format!("{mins}m")
    } else if mins < 60 * 24 {
        format!("{}h", mins / 60)
    } else if mins < 60 * 24 * 14 {
        format!("{}d", mins / (60 * 24))
    } else {
        format!("{}w", mins / (60 * 24 * 7))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewTicketCaps {
    pub summary_max: usize,
    pub description_max: usize,
    pub max_images: usize,
    pub max_image_bytes: usize,
}

impl Default for NewTicketCaps {
    fn default() -> Self {
        Self {
            summary_max: 200,
            description_max: 4000,
            max_images: 5,
            max_image_bytes: 5 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewTicketError {
    SummaryBlank,
    SummaryTooLong { max: usize },
    DescriptionTooLong { max: usize },
    TooManyImages { max: usize },
    ImageTooLarge { filename: String, max: usize },
}

/// Every problem at once, so a form can mark each field.
pub fn validate_new_ticket(t: &NewTicket, caps: &NewTicketCaps) -> Result<(), Vec<NewTicketError>> {
    let mut errs = Vec::new();
    if t.summary.trim().is_empty() {
        errs.push(NewTicketError::SummaryBlank);
    } else if t.summary.chars().count() > caps.summary_max {
        errs.push(NewTicketError::SummaryTooLong {
            max: caps.summary_max,
        });
    }
    if t.description.chars().count() > caps.description_max {
        errs.push(NewTicketError::DescriptionTooLong {
            max: caps.description_max,
        });
    }
    if t.images.len() > caps.max_images {
        errs.push(NewTicketError::TooManyImages {
            max: caps.max_images,
        });
    }
    for img in &t.images {
        if img.bytes.len() > caps.max_image_bytes {
            errs.push(NewTicketError::ImageTooLarge {
                filename: img.filename.clone(),
                max: caps.max_image_bytes,
            });
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

/// What a role may see and do. One place, so the composite's branches and
/// the proof's negative controls read the same table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleCapabilities {
    pub scope: TicketScope,
    pub assignee_filter: bool,
    pub mine_only_toggle: bool,
    pub triage: bool,
    pub comment: bool,
}

impl RoleCapabilities {
    pub fn for_role(role: HelpdeskRole) -> Self {
        match role {
            HelpdeskRole::Requester => Self {
                scope: TicketScope::Mine,
                assignee_filter: false,
                mine_only_toggle: false,
                triage: false,
                comment: true,
            },
            HelpdeskRole::Support => Self {
                scope: TicketScope::All,
                assignee_filter: true,
                mine_only_toggle: true,
                triage: true,
                comment: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::helpdesk::model::*;

    fn t(
        key: &str,
        cat: StatusCategory,
        pri: TicketPriority,
        req: &str,
        asg: Option<&str>,
    ) -> HelpdeskTicket {
        HelpdeskTicket {
            key: TicketKey(key.into()),
            summary: format!("Summary {key}"),
            kind: TicketKind::Bug,
            status: TicketStatus {
                id: "s".into(),
                name: "S".into(),
                category: cat,
            },
            priority: pri,
            assignee: asg.map(|a| Person::new(a, a)),
            requester: Some(Person::new(req, req)),
            created_at_ms: 0,
            updated_at_ms: 0,
            comment_count: 0,
            attachment_count: 0,
            url: None,
        }
    }

    #[test]
    fn buckets_count_by_category() {
        let v = vec![
            t("A-1", StatusCategory::New, TicketPriority::High, "me", None),
            t(
                "A-2",
                StatusCategory::InProgress,
                TicketPriority::Low,
                "me",
                None,
            ),
            t("A-3", StatusCategory::Done, TicketPriority::Low, "me", None),
            t("A-4", StatusCategory::New, TicketPriority::Low, "me", None),
        ];
        let c = bucket_counts(&v);
        assert_eq!((c.open, c.in_progress, c.done, c.total()), (2, 1, 1, 4));
        assert_eq!(
            Bucket::from_id(Bucket::InProgress.id()),
            Some(Bucket::InProgress)
        );
        assert_eq!(Bucket::from_id("nope"), None);
    }

    #[test]
    fn filter_matches_bucket_search_priority_assignee_and_mine() {
        let a = t(
            "OF-1",
            StatusCategory::New,
            TicketPriority::High,
            "me",
            Some("me"),
        );
        let b = t(
            "OF-2",
            StatusCategory::Done,
            TicketPriority::Low,
            "you",
            Some("you"),
        );
        let empty = TicketFilter::default();
        assert!(empty.is_empty() && empty.matches(&a, None) && empty.matches(&b, None));
        let f = TicketFilter {
            bucket: Some(Bucket::Open),
            ..TicketFilter::default()
        };
        assert!(f.matches(&a, Some("me")) && !f.matches(&b, Some("me")));
        let f = TicketFilter {
            search: "of-2".into(),
            ..TicketFilter::default()
        };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter {
            search: "summary".into(),
            ..TicketFilter::default()
        };
        assert!(f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter {
            priority: Some(TicketPriority::Low),
            ..TicketFilter::default()
        };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter {
            assignee_id: Some("you".into()),
            ..TicketFilter::default()
        };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter {
            mine_only: true,
            ..TicketFilter::default()
        };
        assert!(f.matches(&a, Some("me")) && !f.matches(&b, Some("me")));
        assert!(
            !f.matches(&a, None),
            "mine-only with no identity matches nothing"
        );
    }

    #[test]
    fn relative_age_steps() {
        let m = 60_000;
        assert_eq!(relative_age(m * 5, 0), "5m");
        assert_eq!(relative_age(0, 0), "0m");
        assert_eq!(relative_age(m * 60 * 3, 0), "3h");
        assert_eq!(relative_age(m * 60 * 24 * 2, 0), "2d");
        assert_eq!(relative_age(m * 60 * 24 * 21, 0), "3w");
        assert_eq!(relative_age(0, m), "0m", "future timestamps clamp to zero");
    }

    #[test]
    fn validation_reports_every_problem() {
        let caps = NewTicketCaps::default();
        let big = ImageAttachment {
            filename: "a.png".into(),
            content_type: "image/png".into(),
            bytes: vec![0; caps.max_image_bytes + 1],
        };
        let nt = NewTicket {
            kind: TicketKind::Bug,
            summary: "   ".into(),
            description: "x".repeat(caps.description_max + 1),
            context: RequestContext::default(),
            images: vec![big; caps.max_images + 1],
        };
        let errs = validate_new_ticket(&nt, &caps).unwrap_err();
        assert!(errs.contains(&NewTicketError::SummaryBlank));
        assert!(errs.contains(&NewTicketError::DescriptionTooLong {
            max: caps.description_max
        }));
        assert!(errs.contains(&NewTicketError::TooManyImages {
            max: caps.max_images
        }));
        assert!(
            errs.iter()
                .any(|e| matches!(e, NewTicketError::ImageTooLarge { .. }))
        );
        let ok = NewTicket {
            summary: "Broken".into(),
            description: String::new(),
            images: vec![],
            ..nt
        };
        assert_eq!(validate_new_ticket(&ok, &caps), Ok(()));
        let long = NewTicket {
            summary: "x".repeat(caps.summary_max + 1),
            ..ok
        };
        assert_eq!(
            validate_new_ticket(&long, &caps),
            Err(vec![NewTicketError::SummaryTooLong {
                max: caps.summary_max
            }])
        );
    }

    #[test]
    fn role_capabilities() {
        let r = RoleCapabilities::for_role(HelpdeskRole::Requester);
        assert_eq!(r.scope, TicketScope::Mine);
        assert!(!r.assignee_filter && !r.mine_only_toggle && !r.triage && r.comment);
        let s = RoleCapabilities::for_role(HelpdeskRole::Support);
        assert_eq!(s.scope, TicketScope::All);
        assert!(s.assignee_filter && s.mine_only_toggle && s.triage && s.comment);
    }
}
