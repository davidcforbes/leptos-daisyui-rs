//! In-memory `HelpdeskBackend` for the demo page and the browser proof.
//! Deterministic seed, injectable faults, and a call log the fixture exposes.

use super::backend::{HelpdeskBackend, HelpdeskFuture};
use super::model::*;
use std::cell::RefCell;
use std::rc::Rc;

/// The seeded "current user": requester of six tickets, assignee of three.
pub const SEED_ME: &str = "w-chris";
const SEED_YOU: &str = "w-dana";
/// A fixed "now" so ages are deterministic in proofs.
pub const SEED_NOW_MS: i64 = 1_800_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpdeskFault {
    NotConfigured,
    RateLimited(u32),
    FailWrites,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendCall {
    Meta,
    List(TicketScope),
    Detail(String),
    Create {
        summary: String,
        images: usize,
    },
    Transition {
        key: String,
        to_status_id: String,
    },
    Assign {
        key: String,
        assignee_id: Option<String>,
    },
    SetPriority {
        key: String,
        priority_id: String,
    },
    Comment {
        key: String,
        body: String,
    },
}

struct Inner {
    me: String,
    fault: Option<HelpdeskFault>,
    meta: HelpdeskMeta,
    details: Vec<TicketDetail>,
    calls: Vec<BackendCall>,
    next_key: u32,
    next_comment: u32,
}

#[derive(Clone)]
pub struct InMemoryHelpdeskBackend {
    inner: Rc<RefCell<Inner>>,
}

fn statuses() -> Vec<TicketStatus> {
    [
        ("10000", "Backlog", StatusCategory::New),
        ("10001", "Selected for Development", StatusCategory::New),
        ("10002", "Scoping", StatusCategory::InProgress),
        ("3", "In Progress", StatusCategory::InProgress),
        ("10003", "Done", StatusCategory::Done),
    ]
    .into_iter()
    .map(|(id, name, category)| TicketStatus {
        id: id.into(),
        name: name.into(),
        category,
    })
    .collect()
}

fn priorities() -> Vec<PriorityOption> {
    [
        ("1", TicketPriority::Highest, "Highest"),
        ("2", TicketPriority::High, "High"),
        ("3", TicketPriority::Medium, "Medium"),
        ("4", TicketPriority::Low, "Low"),
        ("5", TicketPriority::Lowest, "Lowest"),
    ]
    .into_iter()
    .map(|(id, priority, name)| PriorityOption {
        id: id.into(),
        priority,
        name: name.into(),
    })
    .collect()
}

fn name_of(id: &str) -> &'static str {
    if id == SEED_ME {
        "Chris Forbes"
    } else {
        "Dana Torres"
    }
}

const PIXEL_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";

fn seed_detail(
    n: u32,
    summary: &str,
    kind: TicketKind,
    status: usize,
    pri: usize,
    requester: &str,
    assignee: Option<&str>,
) -> TicketDetail {
    let st = statuses();
    let pr = priorities();
    let hours_ago = i64::from(n) * 7;
    let ticket = HelpdeskTicket {
        key: TicketKey(format!("OF-{n}")),
        summary: summary.into(),
        kind,
        status: st[status].clone(),
        priority: pr[pri].priority,
        assignee: assignee.map(|a| Person::new(a, name_of(a))),
        requester: Some(Person::new(requester, name_of(requester))),
        created_at_ms: SEED_NOW_MS - hours_ago * 3_600_000,
        updated_at_ms: SEED_NOW_MS - hours_ago * 1_800_000,
        comment_count: 1,
        attachment_count: u32::from(n % 3 == 0),
        url: Some(format!("https://example.atlassian.net/browse/OF-{n}")),
    };
    let comments = vec![TicketComment {
        id: format!("c-{n}"),
        author: Person::new(SEED_YOU, name_of(SEED_YOU)),
        body: "Thanks, looking into it.".into(),
        created_at_ms: ticket.created_at_ms + 600_000,
    }];
    let attachments = if n % 3 == 0 {
        vec![TicketAttachment {
            id: format!("a-{n}"),
            filename: "screenshot.png".into(),
            content_type: "image/png".into(),
            url: PIXEL_PNG.into(),
            thumbnail_url: None,
        }]
    } else {
        vec![]
    };
    TicketDetail {
        description: format!(
            "Page /dashboard · Build demo\n\n{summary} happens when the page loads."
        ),
        comments,
        attachments,
        ticket,
    }
}

impl Default for InMemoryHelpdeskBackend {
    fn default() -> Self {
        Self::seeded()
    }
}

impl InMemoryHelpdeskBackend {
    /// Twelve tickets across all five statuses, two requesters, two assignees.
    pub fn seeded() -> Self {
        use TicketKind::{Bug, Request};
        let details = vec![
            seed_detail(1, "Dashboard blank after login", Bug, 0, 1, SEED_ME, None),
            seed_detail(2, "Export to CSV fails", Bug, 1, 2, SEED_ME, Some(SEED_YOU)),
            seed_detail(
                3,
                "Add WhatsApp filter to inbox",
                Request,
                2,
                3,
                SEED_YOU,
                Some(SEED_YOU),
            ),
            seed_detail(
                4,
                "Softphone mutes on transfer",
                Bug,
                3,
                0,
                SEED_ME,
                Some(SEED_ME),
            ),
            seed_detail(
                5,
                "Coordinator board scrolls sideways",
                Bug,
                3,
                1,
                SEED_YOU,
                Some(SEED_ME),
            ),
            seed_detail(
                6,
                "Dark theme for reports",
                Request,
                4,
                4,
                SEED_ME,
                Some(SEED_YOU),
            ),
            seed_detail(
                7,
                "Login loop on Safari",
                Bug,
                4,
                2,
                SEED_YOU,
                Some(SEED_YOU),
            ),
            seed_detail(8, "Rename Matters tab", Request, 0, 3, SEED_YOU, None),
            seed_detail(
                9,
                "KPI strip cards collapse",
                Bug,
                4,
                1,
                SEED_ME,
                Some(SEED_ME),
            ),
            seed_detail(10, "Keyboard shortcut list", Request, 1, 4, SEED_YOU, None),
            seed_detail(
                11,
                "Timezone wrong in call log",
                Bug,
                2,
                0,
                SEED_ME,
                Some(SEED_YOU),
            ),
            seed_detail(12, "Bulk assign from list", Request, 0, 2, SEED_YOU, None),
        ];
        Self {
            inner: Rc::new(RefCell::new(Inner {
                me: SEED_ME.into(),
                fault: None,
                meta: HelpdeskMeta {
                    configured: true,
                    unavailable_reason: None,
                    statuses: statuses(),
                    priorities: priorities(),
                    assignable: vec![
                        Person::new(SEED_ME, name_of(SEED_ME)),
                        Person::new(SEED_YOU, name_of(SEED_YOU)),
                    ],
                    kinds: vec![TicketKind::Bug, TicketKind::Request],
                },
                details,
                calls: Vec::new(),
                next_key: 13,
                next_comment: 100,
            })),
        }
    }

    pub fn with_fault(self, fault: HelpdeskFault) -> Self {
        self.inner.borrow_mut().fault = Some(fault);
        self
    }

    pub fn with_current_user(self, id: impl Into<String>) -> Self {
        self.inner.borrow_mut().me = id.into();
        self
    }

    pub fn current_user(&self) -> String {
        self.inner.borrow().me.clone()
    }

    pub fn calls(&self) -> Vec<BackendCall> {
        self.inner.borrow().calls.clone()
    }

    fn log(&self, call: BackendCall) {
        self.inner.borrow_mut().calls.push(call);
    }

    fn write_fault(&self) -> Option<HelpdeskError> {
        match self.inner.borrow().fault {
            Some(HelpdeskFault::NotConfigured) => Some(HelpdeskError::new(
                HelpdeskErrorKind::NotConfigured,
                "Helpdesk is not configured",
            )),
            Some(HelpdeskFault::RateLimited(s)) => Some(HelpdeskError::new(
                HelpdeskErrorKind::RateLimited { retry_after_s: s },
                "Too many requests",
            )),
            Some(HelpdeskFault::FailWrites) => Some(HelpdeskError::new(
                HelpdeskErrorKind::Upstream,
                "Jira refused the change",
            )),
            None => None,
        }
    }

    fn ready<T: 'static>(v: Result<T, HelpdeskError>) -> HelpdeskFuture<T> {
        Box::pin(std::future::ready(v))
    }

    fn not_found(key: &TicketKey) -> HelpdeskError {
        HelpdeskError::new(HelpdeskErrorKind::NotFound, format!("{key} was not found"))
    }

    fn update(
        &self,
        key: &TicketKey,
        f: impl FnOnce(&mut TicketDetail, &Inner) -> Result<(), HelpdeskError>,
    ) -> Result<HelpdeskTicket, HelpdeskError> {
        if let Some(e) = self.write_fault() {
            return Err(e);
        }
        let mut inner = self.inner.borrow_mut();
        let Some(idx) = inner.details.iter().position(|d| &d.ticket.key == key) else {
            return Err(Self::not_found(key));
        };
        let mut d = inner.details[idx].clone();
        f(&mut d, &inner)?;
        d.ticket.updated_at_ms = SEED_NOW_MS;
        d.ticket.comment_count = d.comments.len() as u32;
        let ticket = d.ticket.clone();
        inner.details[idx] = d;
        Ok(ticket)
    }
}

impl HelpdeskBackend for InMemoryHelpdeskBackend {
    fn meta(&self) -> HelpdeskFuture<HelpdeskMeta> {
        self.log(BackendCall::Meta);
        let inner = self.inner.borrow();
        let mut meta = inner.meta.clone();
        if inner.fault == Some(HelpdeskFault::NotConfigured) {
            meta.configured = false;
            meta.unavailable_reason =
                Some("The Jira project for this helpdesk has not been set.".into());
        }
        Self::ready(Ok(meta))
    }

    fn list(&self, scope: TicketScope) -> HelpdeskFuture<Vec<HelpdeskTicket>> {
        self.log(BackendCall::List(scope));
        let inner = self.inner.borrow();
        let v = inner
            .details
            .iter()
            .map(|d| d.ticket.clone())
            .filter(|t| match scope {
                TicketScope::All => true,
                TicketScope::Mine => {
                    t.requester.as_ref().map(|p| p.id.as_str()) == Some(inner.me.as_str())
                }
            })
            .collect();
        Self::ready(Ok(v))
    }

    fn detail(&self, key: &TicketKey) -> HelpdeskFuture<TicketDetail> {
        self.log(BackendCall::Detail(key.0.clone()));
        let inner = self.inner.borrow();
        let v = inner
            .details
            .iter()
            .find(|d| &d.ticket.key == key)
            .cloned()
            .ok_or_else(|| Self::not_found(key));
        Self::ready(v)
    }

    fn create(&self, ticket: NewTicket) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Create {
            summary: ticket.summary.clone(),
            images: ticket.images.len(),
        });
        if let Some(e) = self.write_fault() {
            return Self::ready(Err(e));
        }
        let mut inner = self.inner.borrow_mut();
        let n = inner.next_key;
        inner.next_key += 1;
        let me = Person::new(inner.me.clone(), name_of(&inner.me));
        let made = HelpdeskTicket {
            key: TicketKey(format!("OF-{n}")),
            summary: ticket.summary.trim().to_owned(),
            kind: ticket.kind,
            status: inner.meta.statuses[0].clone(),
            priority: TicketPriority::Medium,
            assignee: None,
            requester: Some(me),
            created_at_ms: SEED_NOW_MS,
            updated_at_ms: SEED_NOW_MS,
            comment_count: 0,
            attachment_count: ticket.images.len() as u32,
            url: Some(format!("https://example.atlassian.net/browse/OF-{n}")),
        };
        let attachments = ticket
            .images
            .iter()
            .enumerate()
            .map(|(i, img)| TicketAttachment {
                id: format!("a-{n}-{i}"),
                filename: img.filename.clone(),
                content_type: img.content_type.clone(),
                url: String::new(),
                thumbnail_url: None,
            })
            .collect();
        inner.details.insert(
            0,
            TicketDetail {
                ticket: made.clone(),
                description: format!(
                    "Page {} · Build {}\n\n{}",
                    ticket.context.route, ticket.context.build, ticket.description
                ),
                comments: vec![],
                attachments,
            },
        );
        Self::ready(Ok(made))
    }

    fn transition(&self, key: &TicketKey, to_status_id: &str) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Transition {
            key: key.0.clone(),
            to_status_id: to_status_id.into(),
        });
        let to = to_status_id.to_owned();
        Self::ready(self.update(key, |d, inner| {
            let Some(s) = inner.meta.statuses.iter().find(|s| s.id == to) else {
                return Err(HelpdeskError::new(
                    HelpdeskErrorKind::Upstream,
                    "no such status",
                ));
            };
            d.ticket.status = s.clone();
            Ok(())
        }))
    }

    fn assign(&self, key: &TicketKey, assignee_id: Option<&str>) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Assign {
            key: key.0.clone(),
            assignee_id: assignee_id.map(str::to_owned),
        });
        let who = assignee_id.map(str::to_owned);
        Self::ready(self.update(key, |d, inner| {
            d.ticket.assignee = match who {
                None => None,
                Some(id) => Some(
                    inner
                        .meta
                        .assignable
                        .iter()
                        .find(|p| p.id == id)
                        .cloned()
                        .ok_or_else(|| {
                            HelpdeskError::new(HelpdeskErrorKind::Upstream, "not assignable")
                        })?,
                ),
            };
            Ok(())
        }))
    }

    fn set_priority(&self, key: &TicketKey, priority_id: &str) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::SetPriority {
            key: key.0.clone(),
            priority_id: priority_id.into(),
        });
        let pid = priority_id.to_owned();
        Self::ready(self.update(key, |d, inner| {
            let Some(p) = inner.meta.priorities.iter().find(|p| p.id == pid) else {
                return Err(HelpdeskError::new(
                    HelpdeskErrorKind::Upstream,
                    "no such priority",
                ));
            };
            d.ticket.priority = p.priority;
            Ok(())
        }))
    }

    fn comment(&self, key: &TicketKey, body: &str) -> HelpdeskFuture<TicketComment> {
        self.log(BackendCall::Comment {
            key: key.0.clone(),
            body: body.into(),
        });
        let body = body.trim().to_owned();
        let (id, me) = {
            let mut inner = self.inner.borrow_mut();
            inner.next_comment += 1;
            (format!("c-{}", inner.next_comment), inner.me.clone())
        };
        let comment = TicketComment {
            id,
            author: Person::new(me.clone(), name_of(&me)),
            body,
            created_at_ms: SEED_NOW_MS,
        };
        let pushed = comment.clone();
        Self::ready(
            self.update(key, move |d, _| {
                d.comments.push(pushed);
                Ok(())
            })
            .map(|_| comment),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn seeded_lists_all_and_mine() {
        let b = InMemoryHelpdeskBackend::seeded();
        let all = b.list(TicketScope::All).await.unwrap();
        assert_eq!(all.len(), 12);
        let mine = b.list(TicketScope::Mine).await.unwrap();
        assert!(
            mine.iter()
                .all(|t| t.requester.as_ref().map(|p| p.id.as_str()) == Some(SEED_ME))
        );
        assert_eq!(mine.len(), 6);
        let meta = b.meta().await.unwrap();
        assert!(meta.configured && meta.statuses.len() == 5 && meta.kinds.len() == 2);
        assert_eq!(b.calls().len(), 3);
    }

    #[tokio::test]
    async fn create_prepends_with_next_key_and_logs() {
        let b = InMemoryHelpdeskBackend::seeded();
        let made = b
            .create(NewTicket {
                kind: TicketKind::Request,
                summary: "New thing".into(),
                description: "please".into(),
                context: RequestContext {
                    route: "/x".into(),
                    build: "b1".into(),
                },
                images: vec![ImageAttachment {
                    filename: "a.png".into(),
                    content_type: "image/png".into(),
                    bytes: vec![1, 2],
                }],
            })
            .await
            .unwrap();
        assert_eq!(made.key.0, "OF-13");
        assert_eq!(made.attachment_count, 1);
        assert_eq!(b.list(TicketScope::All).await.unwrap()[0].key, made.key);
        assert!(matches!(
            b.calls().first(),
            Some(BackendCall::Create { summary, images: 1 }) if summary == "New thing"
        ));
        let d = b.detail(&made.key).await.unwrap();
        assert!(d.description.starts_with("Page /x · Build b1"));
    }

    #[tokio::test]
    async fn writes_update_the_ticket_and_detail() {
        let b = InMemoryHelpdeskBackend::seeded();
        let key = TicketKey("OF-1".into());
        let done_id = b
            .meta()
            .await
            .unwrap()
            .statuses
            .iter()
            .find(|s| s.category == StatusCategory::Done)
            .unwrap()
            .id
            .clone();
        let t = b.transition(&key, &done_id).await.unwrap();
        assert_eq!(t.status.category, StatusCategory::Done);
        let t = b.assign(&key, Some("w-dana")).await.unwrap();
        assert_eq!(t.assignee.as_ref().unwrap().id, "w-dana");
        let t = b.assign(&key, None).await.unwrap();
        assert!(t.assignee.is_none());
        let t = b.set_priority(&key, "1").await.unwrap();
        assert_eq!(t.priority, TicketPriority::Highest);
        let c = b.comment(&key, "hello").await.unwrap();
        assert_eq!(c.body, "hello");
        let d = b.detail(&key).await.unwrap();
        assert_eq!(d.comments.last().unwrap().body, "hello");
        assert_eq!(d.ticket.comment_count as usize, d.comments.len());
        assert_eq!(d.ticket.status.category, StatusCategory::Done);
    }

    #[tokio::test]
    async fn faults_shape_errors() {
        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::NotConfigured);
        let m = b.meta().await.unwrap();
        assert!(!m.configured && m.unavailable_reason.is_some());
        let e = b.create(sample()).await.unwrap_err();
        assert_eq!(e.kind, HelpdeskErrorKind::NotConfigured);

        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::RateLimited(42));
        assert_eq!(
            b.create(sample()).await.unwrap_err().kind,
            HelpdeskErrorKind::RateLimited { retry_after_s: 42 }
        );

        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::FailWrites);
        assert_eq!(
            b.transition(&TicketKey("OF-1".into()), "x")
                .await
                .unwrap_err()
                .kind,
            HelpdeskErrorKind::Upstream
        );
        assert!(b.list(TicketScope::All).await.is_ok(), "reads still work");
        assert_eq!(
            b.detail(&TicketKey("OF-999".into()))
                .await
                .unwrap_err()
                .kind,
            HelpdeskErrorKind::NotFound
        );
    }

    fn sample() -> NewTicket {
        NewTicket {
            kind: TicketKind::Bug,
            summary: "s".into(),
            description: String::new(),
            context: RequestContext::default(),
            images: vec![],
        }
    }
}
