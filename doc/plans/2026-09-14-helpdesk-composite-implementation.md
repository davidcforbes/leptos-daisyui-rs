# Helpdesk Composite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `patterns::Helpdesk`, a role-switched Jira-ticket composite driven by a host-implemented `HelpdeskBackend` trait, plus the generic `ImageAttachmentField`, with a demo page and a registered browser proof lane.

**Architecture:** Pure model and derivations (`model.rs`, `state.rs`) are native-tested; the trait (`backend.rs`) uses `'static` non-`Send` boxed futures like `markdown::asset_upload::UploadFuture`; an in-memory backend under `test-mode` feeds the demo and the proof; the composite composes existing patterns (`PageHeader`, `SelectableSummaryGroup`, `FilterBar`, `EntityTable`, `Drawer`, `RecordHeader`, `ActionFeedback`, `Modal`, `PageStatePanel`). Image intake is a new component sharing one `image_files` helper with the Markdown editor.

**Tech Stack:** Rust 2024, Leptos 0.8 CSR, daisyUI 5, web-sys, wasm-bindgen-futures, serde; proofs via pixelproof-web + vendored axe-core through `cargo xtask`.

**Spec:** `doc/plans/2026-09-14-helpdesk-composite-design.md`

## Global Constraints

- Spacing on the canonical scale only: Tailwind `1, 2, 3, 4, 6, 8, 12, 16, 24`; no `gap-5`, `p-5`, `mt-1.5`.
- Sizes use the enums (`IconSize`, `AvatarBadgeSize`), never `w-5 h-5`.
- daisyUI 5: never `.form-control`, `.label-text`, `.label-text-alt`.
- Every inline doc code span stays on ONE `///` line (clippy 1.95 ICE otherwise).
- Every framework-owned element carries a `data-helpdesk-*` / `data-image-attachment-*` hook; tests never select by position.
- Texts structs: all fields `pub`, built by struct literal, `Default` = English.
- Muted text is `text-base-content/75`, never `/50` or `opacity-60` (axe AA).
- fmt per package only: `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask -p ldui-audit`. Never `cargo fmt --all`.
- Library clippy/test run with `--features test-mode`.
- Do not edit the tree while a browser lane is running (the ldui-hun stamp guard fails).
- Commit after each task with the trailer `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.

---

## File Structure

| File | Responsibility |
|---|---|
| `src/patterns/helpdesk/mod.rs` | module wiring, re-exports |
| `src/patterns/helpdesk/model.rs` | wire/model types, `HelpdeskRole`, `HelpdeskError` |
| `src/patterns/helpdesk/state.rs` | pure derivations: buckets, filter, age, validation, role capabilities |
| `src/patterns/helpdesk/backend.rs` | `HelpdeskBackend` trait, `HelpdeskFuture` |
| `src/patterns/helpdesk/memory.rs` | `InMemoryHelpdeskBackend` (`test-mode`) with faults + call log |
| `src/patterns/helpdesk/texts.rs` | `HelpdeskTexts` |
| `src/patterns/helpdesk/request_dialog.rs` | `NewRequestDialog` |
| `src/patterns/helpdesk/drawer.rs` | `TicketDetailDrawer` |
| `src/patterns/helpdesk/component.rs` | `Helpdesk` composite |
| `src/utils/image_files.rs` | `sniff_content_type`, `image_files_in`, `first_image_file` |
| `src/components/image_attachment_field/{mod.rs,admission.rs,component.rs,texts.rs}` | generic paste/drop/pick image intake |
| `src/markdown/editor.rs` | paste/drop refactored onto `image_files` |
| `demo/src/demos/helpdesk.rs` | showcase page + test fixture |
| `demo/src/client_snapshot_test_host.rs`, `demo/src/main.rs`, `demo/src/demos/mod.rs`, `demo/src/core/layout.rs` | registration |
| `tests/helpdesk_smoke.rs` | browser proof lane |
| `xtask/src/main.rs` | `test-helpdesk` step, CLI arm, usage, `full_steps`, fingerprint inputs |
| `tests/layout_audit_smoke.rs`, `tests/style_audit_smoke.rs` | audit page entries + invocations |
| `doc/components/helpdesk.md`, `CLAUDE.md` | docs |

---

### Task 1: Model types

**Files:**
- Create: `src/patterns/helpdesk/mod.rs`
- Create: `src/patterns/helpdesk/model.rs`
- Modify: `src/patterns/mod.rs:3-21` (add `mod helpdesk;`) and the `pub use` block

**Interfaces:**
- Produces: every type in the code below, re-exported from `crate::patterns`.

- [ ] **Step 1: Write the failing test** at the bottom of `model.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_category_parses_jira_keys() {
        assert_eq!(StatusCategory::from_jira_key("new"), StatusCategory::New);
        assert_eq!(StatusCategory::from_jira_key("indeterminate"), StatusCategory::InProgress);
        assert_eq!(StatusCategory::from_jira_key("done"), StatusCategory::Done);
        assert_eq!(StatusCategory::from_jira_key("weird"), StatusCategory::New);
    }

    #[test]
    fn ticket_round_trips_through_serde() {
        let t = HelpdeskTicket {
            key: TicketKey("OF-14".into()),
            summary: "Dashboard blank".into(),
            kind: TicketKind::Bug,
            status: TicketStatus { id: "1".into(), name: "Backlog".into(), category: StatusCategory::New },
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::model`
Expected: compile error, module does not exist.

- [ ] **Step 3: Write `model.rs`**

```rust
//! Helpdesk model: what a ticket, a detail view, a new request and the
//! backend's metadata look like. Pure data, serde-round-trippable so a host
//! can decode straight into it.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketKey(pub String);

impl std::fmt::Display for TicketKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketKind {
    Bug,
    Request,
    Other(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusCategory {
    New,
    InProgress,
    Done,
}

impl StatusCategory {
    /// Jira `statusCategory.key`: `new`, `indeterminate`, `done`. Unknown keys
    /// are treated as `New` so a ticket is never hidden from every bucket.
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
        Self { id: id.into(), display_name, initials }
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
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(b: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64_encode(b))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64_decode(&s).map_err(serde::de::Error::custom)
    }
    fn base64_encode(b: &[u8]) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(b)
    }
    fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|e| e.to_string())
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
    pub unavailable_reason: Option<String>,
    pub statuses: Vec<TicketStatus>,
    pub priorities: Vec<PriorityOption>,
    pub assignable: Vec<Person>,
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
        Self { kind, message: message.into() }
    }
}

impl std::fmt::Display for HelpdeskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
```

`mod.rs`:

```rust
//! Opinionated Jira-helpdesk composite driven by a host-implemented backend.
//! See `doc/plans/2026-09-14-helpdesk-composite-design.md`.

mod model;

pub use model::{
    HelpdeskError, HelpdeskErrorKind, HelpdeskMeta, HelpdeskRole, HelpdeskTicket,
    ImageAttachment, NewTicket, Person, PriorityOption, RequestContext, StatusCategory,
    TicketAttachment, TicketComment, TicketDetail, TicketKey, TicketKind, TicketPriority,
    TicketScope, TicketStatus,
};
```

In `src/patterns/mod.rs` add `mod helpdesk;` to the `mod` list (alphabetical, after `filter_bar`) and `pub use helpdesk::*;` after the `filter_bar` re-export block. (This module is the one glob re-export in `patterns`; it keeps the surface in one place as the composite grows across tasks.)

- [ ] **Step 4: Run tests**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::model`
Expected: 4 passed.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk src/patterns/mod.rs
git commit -m "feat(helpdesk): model types for the helpdesk composite

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 2: Pure derivations (`state.rs`)

**Files:**
- Create: `src/patterns/helpdesk/state.rs`
- Modify: `src/patterns/helpdesk/mod.rs` (add `mod state;` + re-exports)

**Interfaces:**
- Consumes: Task 1 types.
- Produces: `Bucket`, `BucketCounts`, `bucket_counts(&[HelpdeskTicket]) -> BucketCounts`, `TicketFilter { bucket, search, priority, assignee_id, mine_only }` with `TicketFilter::matches(&self, &HelpdeskTicket, me: Option<&str>) -> bool`, `relative_age(now_ms: i64, then_ms: i64) -> String`, `NewTicketCaps { summary_max: 200, description_max: 4000, max_images: 5, max_image_bytes: 5 MiB }`, `NewTicketError`, `validate_new_ticket(&NewTicket, &NewTicketCaps) -> Result<(), Vec<NewTicketError>>`, `RoleCapabilities::for_role(HelpdeskRole)`.

- [ ] **Step 1: Write the failing tests** (bottom of `state.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::helpdesk::model::*;

    fn t(key: &str, cat: StatusCategory, pri: TicketPriority, req: &str, asg: Option<&str>) -> HelpdeskTicket {
        HelpdeskTicket {
            key: TicketKey(key.into()),
            summary: format!("Summary {key}"),
            kind: TicketKind::Bug,
            status: TicketStatus { id: "s".into(), name: "S".into(), category: cat },
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
            t("A-2", StatusCategory::InProgress, TicketPriority::Low, "me", None),
            t("A-3", StatusCategory::Done, TicketPriority::Low, "me", None),
            t("A-4", StatusCategory::New, TicketPriority::Low, "me", None),
        ];
        let c = bucket_counts(&v);
        assert_eq!((c.open, c.in_progress, c.done, c.total()), (2, 1, 1, 4));
    }

    #[test]
    fn filter_matches_bucket_search_priority_assignee_and_mine() {
        let a = t("OF-1", StatusCategory::New, TicketPriority::High, "me", Some("me"));
        let b = t("OF-2", StatusCategory::Done, TicketPriority::Low, "you", Some("you"));
        let f = TicketFilter { bucket: Some(Bucket::Open), ..TicketFilter::default() };
        assert!(f.matches(&a, Some("me")) && !f.matches(&b, Some("me")));
        let f = TicketFilter { search: "of-2".into(), ..TicketFilter::default() };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter { search: "summary".into(), ..TicketFilter::default() };
        assert!(f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter { priority: Some(TicketPriority::Low), ..TicketFilter::default() };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter { assignee_id: Some("you".into()), ..TicketFilter::default() };
        assert!(!f.matches(&a, None) && f.matches(&b, None));
        let f = TicketFilter { mine_only: true, ..TicketFilter::default() };
        assert!(f.matches(&a, Some("me")) && !f.matches(&b, Some("me")));
        assert!(!f.matches(&a, None), "mine-only with no identity matches nothing");
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
        let big = ImageAttachment { filename: "a.png".into(), content_type: "image/png".into(), bytes: vec![0; caps.max_image_bytes + 1] };
        let nt = NewTicket {
            kind: TicketKind::Bug,
            summary: "   ".into(),
            description: "x".repeat(caps.description_max + 1),
            context: RequestContext::default(),
            images: vec![big; caps.max_images + 1],
        };
        let errs = validate_new_ticket(&nt, &caps).unwrap_err();
        assert!(errs.contains(&NewTicketError::SummaryBlank));
        assert!(errs.contains(&NewTicketError::DescriptionTooLong { max: caps.description_max }));
        assert!(errs.contains(&NewTicketError::TooManyImages { max: caps.max_images }));
        assert!(errs.iter().any(|e| matches!(e, NewTicketError::ImageTooLarge { .. })));
        let ok = NewTicket { summary: "Broken".into(), description: String::new(), images: vec![], ..nt };
        assert_eq!(validate_new_ticket(&ok, &caps), Ok(()));
        let long = NewTicket { summary: "x".repeat(caps.summary_max + 1), ..ok };
        assert_eq!(validate_new_ticket(&long, &caps), Err(vec![NewTicketError::SummaryTooLong { max: caps.summary_max }]));
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::state`
Expected: compile error.

- [ ] **Step 3: Write `state.rs`**

```rust
//! Pure derivations for the helpdesk composite: buckets, filtering, age
//! formatting, validation and role capabilities. Nothing here touches the DOM.

use super::model::{
    HelpdeskRole, HelpdeskTicket, NewTicket, StatusCategory, TicketPriority, TicketScope,
};

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

    /// `me` is the current user's id; "mine" means assigned to me.
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
        if let Some(a) = &self.assignee_id
            && t.assignee.as_ref().map(|x| x.id.as_str()) != Some(a.as_str())
        {
            return false;
        }
        if self.mine_only {
            let Some(me) = me else { return false };
            if t.assignee.as_ref().map(|x| x.id.as_str()) != Some(me) {
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
        Self { summary_max: 200, description_max: 4000, max_images: 5, max_image_bytes: 5 * 1024 * 1024 }
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

pub fn validate_new_ticket(t: &NewTicket, caps: &NewTicketCaps) -> Result<(), Vec<NewTicketError>> {
    let mut errs = Vec::new();
    if t.summary.trim().is_empty() {
        errs.push(NewTicketError::SummaryBlank);
    } else if t.summary.chars().count() > caps.summary_max {
        errs.push(NewTicketError::SummaryTooLong { max: caps.summary_max });
    }
    if t.description.chars().count() > caps.description_max {
        errs.push(NewTicketError::DescriptionTooLong { max: caps.description_max });
    }
    if t.images.len() > caps.max_images {
        errs.push(NewTicketError::TooManyImages { max: caps.max_images });
    }
    for img in &t.images {
        if img.bytes.len() > caps.max_image_bytes {
            errs.push(NewTicketError::ImageTooLarge { filename: img.filename.clone(), max: caps.max_image_bytes });
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

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
```

Add to `mod.rs`: `mod state;` and `pub use state::{Bucket, BucketCounts, NewTicketCaps, NewTicketError, RoleCapabilities, TicketFilter, bucket_counts, relative_age, validate_new_ticket};`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::state`
Expected: 5 passed.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): pure buckets, filter, age, validation and role capabilities

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 3: Backend trait and in-memory backend

**Files:**
- Create: `src/patterns/helpdesk/backend.rs`
- Create: `src/patterns/helpdesk/memory.rs`
- Modify: `src/patterns/helpdesk/mod.rs`

**Interfaces:**
- Produces: `HelpdeskFuture<T>`, `trait HelpdeskBackend` (8 methods), `InMemoryHelpdeskBackend::seeded()`, `.with_fault(HelpdeskFault)`, `.with_current_user(id)`, `.calls() -> Vec<BackendCall>`, `HelpdeskFault::{NotConfigured, RateLimited(u32), FailWrites}`, `BackendCall` enum.

- [ ] **Step 1: Write the failing tests** (bottom of `memory.rs`; `#[tokio::test]` default current-thread flavor accepts `!Send` futures):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::helpdesk::model::*;

    #[tokio::test]
    async fn seeded_lists_all_and_mine() {
        let b = InMemoryHelpdeskBackend::seeded();
        let all = b.list(TicketScope::All).await.unwrap();
        assert_eq!(all.len(), 12);
        let mine = b.list(TicketScope::Mine).await.unwrap();
        assert!(mine.iter().all(|t| t.requester.as_ref().map(|p| p.id.as_str()) == Some(SEED_ME)));
        assert!(mine.len() < all.len() && !mine.is_empty());
        let meta = b.meta().await.unwrap();
        assert!(meta.configured && meta.statuses.len() == 5 && meta.kinds.len() == 2);
    }

    #[tokio::test]
    async fn create_prepends_with_next_key_and_logs() {
        let b = InMemoryHelpdeskBackend::seeded();
        let made = b
            .create(NewTicket {
                kind: TicketKind::Request,
                summary: "New thing".into(),
                description: "please".into(),
                context: RequestContext { route: "/x".into(), build: "b1".into() },
                images: vec![ImageAttachment { filename: "a.png".into(), content_type: "image/png".into(), bytes: vec![1, 2] }],
            })
            .await
            .unwrap();
        assert_eq!(made.key.0, "OF-13");
        assert_eq!(made.attachment_count, 1);
        assert_eq!(b.list(TicketScope::All).await.unwrap()[0].key, made.key);
        assert!(matches!(b.calls().last(), Some(BackendCall::Create { summary, images: 1 }) if summary == "New thing"));
    }

    #[tokio::test]
    async fn writes_update_the_ticket_and_detail() {
        let b = InMemoryHelpdeskBackend::seeded();
        let key = TicketKey("OF-1".into());
        let done_id = b.meta().await.unwrap().statuses.iter().find(|s| s.category == StatusCategory::Done).unwrap().id.clone();
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
    }

    #[tokio::test]
    async fn faults_shape_errors() {
        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::NotConfigured);
        let m = b.meta().await.unwrap();
        assert!(!m.configured && m.unavailable_reason.is_some());
        let e = b.create(sample()).await.unwrap_err();
        assert_eq!(e.kind, HelpdeskErrorKind::NotConfigured);

        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::RateLimited(42));
        assert_eq!(b.create(sample()).await.unwrap_err().kind, HelpdeskErrorKind::RateLimited { retry_after_s: 42 });

        let b = InMemoryHelpdeskBackend::seeded().with_fault(HelpdeskFault::FailWrites);
        assert_eq!(b.transition(&TicketKey("OF-1".into()), "x").await.unwrap_err().kind, HelpdeskErrorKind::Upstream);
        assert!(b.list(TicketScope::All).await.is_ok(), "reads still work");
        assert_eq!(b.detail(&TicketKey("OF-999".into())).await.unwrap_err().kind, HelpdeskErrorKind::NotFound);
    }

    fn sample() -> NewTicket {
        NewTicket { kind: TicketKind::Bug, summary: "s".into(), description: String::new(), context: RequestContext::default(), images: vec![] }
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::memory`
Expected: compile error.

- [ ] **Step 3: Write `backend.rs`**

```rust
//! The transport seam. The composite calls these; the host implements them
//! over its own HTTP client. Futures are `'static` and not `Send`, exactly
//! like `crate::markdown::asset_upload::UploadFuture`: wasm is single-threaded
//! and every call is awaited from `spawn_local`.

use super::model::{
    HelpdeskError, HelpdeskMeta, HelpdeskTicket, NewTicket, TicketComment, TicketDetail,
    TicketKey, TicketScope,
};
use std::future::Future;
use std::pin::Pin;

pub type HelpdeskFuture<T> = Pin<Box<dyn Future<Output = Result<T, HelpdeskError>> + 'static>>;

pub trait HelpdeskBackend {
    fn meta(&self) -> HelpdeskFuture<HelpdeskMeta>;
    fn list(&self, scope: TicketScope) -> HelpdeskFuture<Vec<HelpdeskTicket>>;
    fn detail(&self, key: &TicketKey) -> HelpdeskFuture<TicketDetail>;
    fn create(&self, ticket: NewTicket) -> HelpdeskFuture<HelpdeskTicket>;
    fn transition(&self, key: &TicketKey, to_status_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    fn assign(&self, key: &TicketKey, assignee_id: Option<&str>) -> HelpdeskFuture<HelpdeskTicket>;
    fn set_priority(&self, key: &TicketKey, priority_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    fn comment(&self, key: &TicketKey, body: &str) -> HelpdeskFuture<TicketComment>;
}
```

- [ ] **Step 4: Write `memory.rs`**

```rust
//! In-memory `HelpdeskBackend` for the demo page and the browser proof.
//! Deterministic seed, injectable faults, and a call log the fixture exposes.

use super::backend::{HelpdeskBackend, HelpdeskFuture};
use super::model::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const SEED_ME: &str = "w-chris";
const SEED_YOU: &str = "w-dana";
const SEED_NOW_MS: i64 = 1_800_000_000_000;

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
    Create { summary: String, images: usize },
    Transition { key: String, to_status_id: String },
    Assign { key: String, assignee_id: Option<String> },
    SetPriority { key: String, priority_id: String },
    Comment { key: String, body: String },
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
    .map(|(id, name, category)| TicketStatus { id: id.into(), name: name.into(), category })
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
    .map(|(id, priority, name)| PriorityOption { id: id.into(), priority, name: name.into() })
    .collect()
}

fn seed_detail(n: u32, summary: &str, kind: TicketKind, status: usize, pri: usize, requester: &str, assignee: Option<&str>) -> TicketDetail {
    let st = statuses();
    let pr = priorities();
    let hours_ago = i64::from(n) * 7;
    let ticket = HelpdeskTicket {
        key: TicketKey(format!("OF-{n}")),
        summary: summary.into(),
        kind,
        status: st[status].clone(),
        priority: pr[pri].priority,
        assignee: assignee.map(|a| Person::new(a, if a == SEED_ME { "Chris Forbes" } else { "Dana Torres" })),
        requester: Some(Person::new(requester, if requester == SEED_ME { "Chris Forbes" } else { "Dana Torres" })),
        created_at_ms: SEED_NOW_MS - hours_ago * 3_600_000,
        updated_at_ms: SEED_NOW_MS - hours_ago * 1_800_000,
        comment_count: 1,
        attachment_count: u32::from(n % 3 == 0),
        url: Some(format!("https://example.atlassian.net/browse/OF-{n}")),
    };
    TicketDetail {
        description: format!("Page /dashboard · Build demo\n\n{summary} happens when the page loads."),
        comments: vec![TicketComment {
            id: format!("c-{n}"),
            author: Person::new(SEED_YOU, "Dana Torres"),
            body: "Thanks, looking into it.".into(),
            created_at_ms: ticket.created_at_ms + 600_000,
        }],
        attachments: if n % 3 == 0 {
            vec![TicketAttachment {
                id: format!("a-{n}"),
                filename: "screenshot.png".into(),
                content_type: "image/png".into(),
                url: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=".into(),
                thumbnail_url: None,
            }]
        } else {
            vec![]
        },
        ticket,
    }
}

impl InMemoryHelpdeskBackend {
    /// Twelve tickets across all five statuses, two requesters, two assignees.
    pub fn seeded() -> Self {
        use TicketKind::{Bug, Request};
        let details = vec![
            seed_detail(1, "Dashboard blank after login", Bug, 0, 1, SEED_ME, None),
            seed_detail(2, "Export to CSV fails", Bug, 1, 2, SEED_ME, Some(SEED_YOU)),
            seed_detail(3, "Add WhatsApp filter to inbox", Request, 2, 3, SEED_YOU, Some(SEED_YOU)),
            seed_detail(4, "Softphone mutes on transfer", Bug, 3, 0, SEED_ME, Some(SEED_ME)),
            seed_detail(5, "Coordinator board scrolls sideways", Bug, 3, 1, SEED_YOU, Some(SEED_ME)),
            seed_detail(6, "Dark theme for reports", Request, 4, 4, SEED_ME, Some(SEED_YOU)),
            seed_detail(7, "Login loop on Safari", Bug, 4, 2, SEED_YOU, Some(SEED_YOU)),
            seed_detail(8, "Rename Matters tab", Request, 0, 3, SEED_YOU, None),
            seed_detail(9, "KPI strip cards collapse", Bug, 4, 1, SEED_ME, Some(SEED_ME)),
            seed_detail(10, "Keyboard shortcut list", Request, 1, 4, SEED_YOU, None),
            seed_detail(11, "Timezone wrong in call log", Bug, 2, 0, SEED_ME, Some(SEED_YOU)),
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
                    assignable: vec![Person::new(SEED_ME, "Chris Forbes"), Person::new(SEED_YOU, "Dana Torres")],
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
            Some(HelpdeskFault::NotConfigured) => Some(HelpdeskError::new(HelpdeskErrorKind::NotConfigured, "Helpdesk is not configured")),
            Some(HelpdeskFault::RateLimited(s)) => Some(HelpdeskError::new(HelpdeskErrorKind::RateLimited { retry_after_s: s }, "Too many requests")),
            Some(HelpdeskFault::FailWrites) => Some(HelpdeskError::new(HelpdeskErrorKind::Upstream, "Jira refused the change")),
            None => None,
        }
    }

    fn ready<T: 'static>(v: Result<T, HelpdeskError>) -> HelpdeskFuture<T> {
        Box::pin(std::future::ready(v))
    }

    fn not_found(key: &TicketKey) -> HelpdeskError {
        HelpdeskError::new(HelpdeskErrorKind::NotFound, format!("{key} was not found"))
    }

    fn update(&self, key: &TicketKey, f: impl FnOnce(&mut TicketDetail, &Inner) -> Result<(), HelpdeskError>) -> Result<HelpdeskTicket, HelpdeskError> {
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
        inner.details[idx] = d.clone();
        Ok(d.ticket)
    }
}

impl HelpdeskBackend for InMemoryHelpdeskBackend {
    fn meta(&self) -> HelpdeskFuture<HelpdeskMeta> {
        self.log(BackendCall::Meta);
        let mut meta = self.inner.borrow().meta.clone();
        if self.inner.borrow().fault == Some(HelpdeskFault::NotConfigured) {
            meta.configured = false;
            meta.unavailable_reason = Some("The Jira project for this helpdesk has not been set.".into());
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
                TicketScope::Mine => t.requester.as_ref().map(|p| p.id.as_str()) == Some(inner.me.as_str()),
            })
            .collect();
        Self::ready(Ok(v))
    }

    fn detail(&self, key: &TicketKey) -> HelpdeskFuture<TicketDetail> {
        self.log(BackendCall::Detail(key.0.clone()));
        let inner = self.inner.borrow();
        let v = inner.details.iter().find(|d| &d.ticket.key == key).cloned().ok_or_else(|| Self::not_found(key));
        Self::ready(v)
    }

    fn create(&self, ticket: NewTicket) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Create { summary: ticket.summary.clone(), images: ticket.images.len() });
        if let Some(e) = self.write_fault() {
            return Self::ready(Err(e));
        }
        let mut inner = self.inner.borrow_mut();
        let n = inner.next_key;
        inner.next_key += 1;
        let me = Person::new(inner.me.clone(), "Chris Forbes");
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
                description: format!("Page {} · Build {}\n\n{}", ticket.context.route, ticket.context.build, ticket.description),
                comments: vec![],
                attachments,
            },
        );
        Self::ready(Ok(made))
    }

    fn transition(&self, key: &TicketKey, to_status_id: &str) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Transition { key: key.0.clone(), to_status_id: to_status_id.into() });
        let to = to_status_id.to_owned();
        Self::ready(self.update(key, |d, inner| {
            let Some(s) = inner.meta.statuses.iter().find(|s| s.id == to) else {
                return Err(HelpdeskError::new(HelpdeskErrorKind::Upstream, "no such status"));
            };
            d.ticket.status = s.clone();
            Ok(())
        }))
    }

    fn assign(&self, key: &TicketKey, assignee_id: Option<&str>) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::Assign { key: key.0.clone(), assignee_id: assignee_id.map(str::to_owned) });
        let who = assignee_id.map(str::to_owned);
        Self::ready(self.update(key, |d, inner| {
            d.ticket.assignee = match who {
                None => None,
                Some(id) => Some(
                    inner.meta.assignable.iter().find(|p| p.id == id).cloned()
                        .ok_or_else(|| HelpdeskError::new(HelpdeskErrorKind::Upstream, "not assignable"))?,
                ),
            };
            Ok(())
        }))
    }

    fn set_priority(&self, key: &TicketKey, priority_id: &str) -> HelpdeskFuture<HelpdeskTicket> {
        self.log(BackendCall::SetPriority { key: key.0.clone(), priority_id: priority_id.into() });
        let pid = priority_id.to_owned();
        Self::ready(self.update(key, |d, inner| {
            let Some(p) = inner.meta.priorities.iter().find(|p| p.id == pid) else {
                return Err(HelpdeskError::new(HelpdeskErrorKind::Upstream, "no such priority"));
            };
            d.ticket.priority = p.priority;
            Ok(())
        }))
    }

    fn comment(&self, key: &TicketKey, body: &str) -> HelpdeskFuture<TicketComment> {
        self.log(BackendCall::Comment { key: key.0.clone(), body: body.into() });
        let body = body.trim().to_owned();
        let id = {
            let mut inner = self.inner.borrow_mut();
            inner.next_comment += 1;
            format!("c-{}", inner.next_comment)
        };
        let comment = TicketComment { id, author: Person::new(SEED_ME, "Chris Forbes"), body, created_at_ms: SEED_NOW_MS };
        let c2 = comment.clone();
        Self::ready(self.update(key, move |d, _| {
            d.comments.push(c2);
            Ok(())
        })
        .map(|_| comment))
    }
}
```

`mod.rs` additions:

```rust
mod backend;
#[cfg(feature = "test-mode")]
mod memory;

pub use backend::{HelpdeskBackend, HelpdeskFuture};
#[cfg(feature = "test-mode")]
pub use memory::{BackendCall, HelpdeskFault, InMemoryHelpdeskBackend, SEED_ME};
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::`
Expected: 13 passed (4 + 5 + 4).

- [ ] **Step 6: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): HelpdeskBackend trait and in-memory test-mode backend

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 4: Shared image-file helper and Markdown editor refactor

**Files:**
- Create: `src/utils/image_files.rs`
- Modify: `src/utils/mod.rs` (add `mod image_files; pub use image_files::*;`)
- Modify: `src/markdown/editor.rs:312-364` (paste/drop loops)

**Interfaces:**
- Produces: `sniff_content_type(&[u8]) -> Option<&'static str>` (native), `image_files_in(&web_sys::DataTransfer) -> Vec<web_sys::File>` and `first_image_file(&web_sys::DataTransfer) -> Option<web_sys::File>` (wasm only, `#[cfg(target_arch = "wasm32")]`).

- [ ] **Step 1: Write the failing test** (bottom of `image_files.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::sniff_content_type;

    #[test]
    fn sniffs_png_jpeg_webp_and_rejects_others() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut webp = *b"RIFF\0\0\0\0WEBPVP8 ";
        webp[4] = 1;
        let gif = *b"GIF89a\0\0\0\0\0\0";
        assert_eq!(sniff_content_type(&png), Some("image/png"));
        assert_eq!(sniff_content_type(&jpeg), Some("image/jpeg"));
        assert_eq!(sniff_content_type(&webp), Some("image/webp"));
        assert_eq!(sniff_content_type(&gif), None, "GIF is not an accepted type");
        assert_eq!(sniff_content_type(b"hello"), None);
        assert_eq!(sniff_content_type(&[]), None);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p leptos-daisyui-rs --features test-mode image_files`
Expected: compile error.

- [ ] **Step 3: Write `image_files.rs`**

```rust
//! One definition of "an image in a DataTransfer" shared by the Markdown
//! editor's paste/drop and `ImageAttachmentField`, plus magic-byte sniffing
//! so a declared MIME type is never trusted.

/// Returns the accepted content type by magic bytes: PNG, JPEG or WebP.
/// Anything else (including GIF, SVG, BMP) is `None`.
pub fn sniff_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        Some("image/png")
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Every `File` in the transfer whose declared type starts with `image/`.
/// Declared type is a pre-filter only; callers sniff the bytes.
#[cfg(target_arch = "wasm32")]
pub fn image_files_in(data: &web_sys::DataTransfer) -> Vec<web_sys::File> {
    let mut out = Vec::new();
    let Some(files) = data.files() else { return out };
    for i in 0..files.length() {
        if let Some(file) = files.get(i)
            && file.type_().starts_with("image/")
        {
            out.push(file);
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
pub fn first_image_file(data: &web_sys::DataTransfer) -> Option<web_sys::File> {
    image_files_in(data).into_iter().next()
}
```

- [ ] **Step 4: Refactor `src/markdown/editor.rs`**

Replace the paste handler body (`editor.rs:312-327`) with:

```rust
    let onpaste = move |ev: ev::ClipboardEvent| {
        let Some(handle) = uploader_handle else {
            return;
        };
        let Some(data) = ev.clipboard_data() else {
            return;
        };
        if let Some(file) = crate::utils::first_image_file(&data) {
            ev.prevent_default();
            let uploader = handle.get_value();
            upload_and_insert_at_cursor(file, uploader, source, textarea, Some(error_toast));
        }
    };
```

Replace the drop handler's file loop (`editor.rs:344-364`) with:

```rust
    let ondrop = move |ev: ev::DragEvent| {
        ev.prevent_default();
        drag_depth.set(0);
        let Some(handle) = uploader_handle else {
            show_error_toast(error_toast, "Drop ignored: no uploader configured.");
            return;
        };
        let Some(data) = ev.data_transfer() else {
            return;
        };
        match crate::utils::first_image_file(&data) {
            Some(file) => {
                let uploader = handle.get_value();
                upload_and_insert_at_cursor(file, uploader, source, textarea, Some(error_toast));
            }
            None => show_error_toast(error_toast, "Drop ignored: no image found."),
        }
    };
```

- [ ] **Step 5: Run tests and the wasm check**

Run: `cargo test -p leptos-daisyui-rs --features test-mode image_files`
Expected: 1 passed.
Run: `cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown`
Expected: clean (the editor compiles on wasm with the helper).

- [ ] **Step 6: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/utils src/markdown/editor.rs
git commit -m "refactor(markdown): share image-file detection via utils::image_files

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 5: `ImageAttachmentField`

**Files:**
- Create: `src/components/image_attachment_field/mod.rs`
- Create: `src/components/image_attachment_field/admission.rs`
- Create: `src/components/image_attachment_field/texts.rs`
- Create: `src/components/image_attachment_field/component.rs`
- Modify: `src/components/mod.rs` (add `mod image_attachment_field;` and `pub use image_attachment_field::*;` in the alphabetical blocks, between `icon_tile` and `indicator`)

**Interfaces:**
- Consumes: `crate::patterns::ImageAttachment` (Task 1), `crate::utils::{sniff_content_type, image_files_in}` (Task 4), `crate::markdown::file_io::read_file_bytes` (make it `pub use`d from `src/markdown/mod.rs` if it is not already public there).
- Produces: `ImageAttachmentField` component, `ImageAttachmentCaps { max_count, max_bytes }`, `ImageRejection`, `ImageAttachmentTexts`, `admit(existing: usize, filename: &str, bytes: Vec<u8>, caps: &ImageAttachmentCaps) -> Result<ImageAttachment, ImageRejection>`.

- [ ] **Step 1: Write the failing test** (`admission.rs` bottom):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const PNG: [u8; 12] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

    #[test]
    fn admits_a_png_and_uses_the_sniffed_type() {
        let caps = ImageAttachmentCaps::default();
        let a = admit(0, "shot.jpg", PNG.to_vec(), &caps).unwrap();
        assert_eq!(a.content_type, "image/png");
        assert_eq!(a.filename, "shot.jpg");
    }

    #[test]
    fn rejects_type_size_and_count() {
        let caps = ImageAttachmentCaps { max_count: 1, max_bytes: 16 };
        assert_eq!(admit(0, "x.txt", b"hello".to_vec(), &caps), Err(ImageRejection::UnsupportedType { filename: "x.txt".into() }));
        let big = [PNG.to_vec(), vec![0; 16]].concat();
        assert_eq!(admit(0, "big.png", big, &caps), Err(ImageRejection::TooLarge { filename: "big.png".into(), max_bytes: 16 }));
        assert_eq!(admit(1, "y.png", PNG.to_vec(), &caps), Err(ImageRejection::TooMany { max_count: 1 }));
    }

    #[test]
    fn blank_filenames_get_a_default() {
        let a = admit(0, "", PNG.to_vec(), &ImageAttachmentCaps::default()).unwrap();
        assert_eq!(a.filename, "pasted-image.png");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p leptos-daisyui-rs --features test-mode image_attachment_field`
Expected: compile error.

- [ ] **Step 3: Write `admission.rs`**

```rust
//! Pure admission rules: what may enter the attachment list.

use crate::patterns::ImageAttachment;
use crate::utils::sniff_content_type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageAttachmentCaps {
    pub max_count: usize,
    pub max_bytes: usize,
}

impl Default for ImageAttachmentCaps {
    fn default() -> Self {
        Self { max_count: 5, max_bytes: 5 * 1024 * 1024 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageRejection {
    UnsupportedType { filename: String },
    TooLarge { filename: String, max_bytes: usize },
    TooMany { max_count: usize },
}

pub fn admit(existing: usize, filename: &str, bytes: Vec<u8>, caps: &ImageAttachmentCaps) -> Result<ImageAttachment, ImageRejection> {
    if existing >= caps.max_count {
        return Err(ImageRejection::TooMany { max_count: caps.max_count });
    }
    let Some(content_type) = sniff_content_type(&bytes) else {
        return Err(ImageRejection::UnsupportedType { filename: filename.to_owned() });
    };
    if bytes.len() > caps.max_bytes {
        return Err(ImageRejection::TooLarge { filename: filename.to_owned(), max_bytes: caps.max_bytes });
    }
    let filename = if filename.trim().is_empty() {
        format!("pasted-image.{}", &content_type["image/".len()..])
    } else {
        filename.to_owned()
    };
    Ok(ImageAttachment { filename, content_type: content_type.to_owned(), bytes })
}
```

- [ ] **Step 4: Write `texts.rs`**

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageAttachmentTexts {
    pub label: String,
    pub drop_hint: String,
    pub browse: String,
    pub remove: String,
    pub rejected_type: String,
    pub rejected_size: String,
    pub rejected_count: String,
    pub read_failed: String,
}

impl Default for ImageAttachmentTexts {
    fn default() -> Self {
        Self {
            label: "Screenshots".into(),
            drop_hint: "Paste, drop or choose PNG, JPEG or WebP images".into(),
            browse: "Choose files".into(),
            remove: "Remove {filename}".into(),
            rejected_type: "{filename} is not a PNG, JPEG or WebP image".into(),
            rejected_size: "{filename} is larger than {max_mb} MB".into(),
            rejected_count: "At most {max} images".into(),
            read_failed: "Could not read {filename}".into(),
        }
    }
}
```

- [ ] **Step 5: Write `component.rs`**

```rust
use super::admission::{ImageAttachmentCaps, ImageRejection, admit};
use super::texts::ImageAttachmentTexts;
use crate::patterns::ImageAttachment;
use leptos::html::Input;
use leptos::prelude::*;
use leptos::{ev, html};
use wasm_bindgen_futures::spawn_local;

/// Generic image intake: a keyboard-operable drop zone that opens a hidden
/// file picker, accepts drag-drop and clipboard paste, sniffs bytes, and
/// renders removable previews. Emits `ImageAttachment`s into `images`.
#[component]
pub fn ImageAttachmentField(
    #[prop(into)] images: RwSignal<Vec<ImageAttachment>>,
    #[prop(optional, into, default = Signal::stored(ImageAttachmentTexts::default()))]
    texts: Signal<ImageAttachmentTexts>,
    #[prop(optional)] caps: ImageAttachmentCaps,
    /// Also accept Ctrl+V anywhere in the document while mounted.
    #[prop(optional)] capture_document_paste: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_reject: Option<Callback<ImageRejection>>,
    #[prop(optional, into)] class: &'static str,
) -> impl IntoView {
    let input_ref = NodeRef::<Input>::new();
    let dragging = RwSignal::new(0i32);
    let announcement = RwSignal::new(String::new());
    let previews: RwSignal<Vec<(String, String)>> = RwSignal::new(Vec::new()); // (filename, object url)

    let announce_rejection = move |r: ImageRejection| {
        let t = texts.get_untracked();
        let msg = match &r {
            ImageRejection::UnsupportedType { filename } => t.rejected_type.replace("{filename}", filename),
            ImageRejection::TooLarge { filename, max_bytes } => t
                .rejected_size
                .replace("{filename}", filename)
                .replace("{max_mb}", &(max_bytes / (1024 * 1024)).to_string()),
            ImageRejection::TooMany { max_count } => t.rejected_count.replace("{max}", &max_count.to_string()),
        };
        announcement.set(msg);
        if let Some(cb) = on_reject {
            cb.run(r);
        }
    };

    // Reads each file, admits it, pushes it. Sequential so `existing` is right.
    let ingest = move |files: Vec<web_sys::File>| {
        if disabled.get_untracked() {
            return;
        }
        spawn_local(async move {
            for file in files {
                let name = file.name();
                let bytes = match crate::markdown::file_io::read_file_bytes(&file).await {
                    Ok(b) => b,
                    Err(_) => {
                        announcement.set(texts.get_untracked().read_failed.replace("{filename}", &name));
                        continue;
                    }
                };
                let existing = images.try_get_untracked().map(|v| v.len()).unwrap_or(0);
                match admit(existing, &name, bytes, &caps) {
                    Ok(att) => {
                        if let Some(url) = object_url(&att) {
                            previews.try_update(|p| p.push((att.filename.clone(), url)));
                        }
                        images.try_update(|v| v.push(att));
                        announcement.try_set(String::new());
                    }
                    Err(r) => announce_rejection(r),
                }
            }
        });
    };

    let on_paste = move |ev: ev::ClipboardEvent| {
        let Some(data) = ev.clipboard_data() else { return };
        let files = crate::utils::image_files_in(&data);
        if !files.is_empty() {
            ev.prevent_default();
            ingest(files);
        }
    };

    if capture_document_paste {
        let handle = window_event_listener(ev::paste, move |ev| {
            let Some(data) = ev.clipboard_data() else { return };
            let files = crate::utils::image_files_in(&data);
            if !files.is_empty() {
                ev.prevent_default();
                ingest(files);
            }
        });
        on_cleanup(move || handle.remove());
    }

    on_cleanup(move || {
        for (_, url) in previews.get_untracked() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    });

    let remove = move |idx: usize| {
        previews.update(|p| {
            if idx < p.len() {
                let (_, url) = p.remove(idx);
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        });
        images.update(|v| {
            if idx < v.len() {
                v.remove(idx);
            }
        });
    };

    view! {
        <div class=format!("flex flex-col gap-2 {class}") data-image-attachment-field="">
            <button
                type="button"
                class="btn btn-ghost h-auto min-h-24 w-full flex-col gap-2 rounded-box border border-dashed border-base-content/30 bg-base-200 p-4 font-normal"
                class:border-primary=move || dragging.get() > 0
                data-image-attachment-dropzone=""
                data-dragging=move || (dragging.get() > 0).to_string()
                aria-label=move || texts.get().label
                disabled=move || disabled.get()
                on:click=move |_| { if let Some(i) = input_ref.get() { i.click(); } }
                on:paste=on_paste
                on:dragover=move |ev: ev::DragEvent| ev.prevent_default()
                on:dragenter=move |ev: ev::DragEvent| { ev.prevent_default(); dragging.update(|n| *n += 1); }
                on:dragleave=move |_| dragging.update(|n| *n = (*n - 1).max(0))
                on:drop=move |ev: ev::DragEvent| {
                    ev.prevent_default();
                    dragging.set(0);
                    if let Some(data) = ev.data_transfer() { ingest(crate::utils::image_files_in(&data)); }
                }
            >
                <span class="text-sm text-base-content/75">{move || texts.get().drop_hint}</span>
                <span class="btn btn-sm btn-outline" aria-hidden="true">{move || texts.get().browse}</span>
            </button>
            <input
                type="file"
                class="hidden"
                accept="image/png,image/jpeg,image/webp"
                multiple=true
                tabindex="-1"
                aria-hidden="true"
                node_ref=input_ref
                data-image-attachment-input=""
                on:change=move |ev| {
                    let input = event_target::<web_sys::HtmlInputElement>(&ev);
                    let mut files = Vec::new();
                    if let Some(list) = input.files() {
                        for i in 0..list.length() { if let Some(f) = list.get(i) { files.push(f); } }
                    }
                    input.set_value("");
                    ingest(files);
                }
            />
            <ul class="flex flex-wrap gap-2" data-image-attachment-list="">
                <For each=move || previews.get().into_iter().enumerate() key=|(i, (_, url))| format!("{i}-{url}") let:entry>
                    {
                        let (idx, (name, url)) = entry;
                        let remove_label = move || texts.get().remove.replace("{filename}", &name);
                        view! {
                            <li class="flex items-center gap-2 rounded-box border border-base-300 bg-base-100 p-2" data-image-attachment-item="">
                                <img src=url.clone() alt="" class="h-12 w-12 rounded object-cover" />
                                <span class="max-w-32 truncate text-sm">{name.clone()}</span>
                                <button type="button" class="btn btn-ghost btn-xs" aria-label=remove_label data-image-attachment-remove="" on:click=move |_| remove(idx)>"×"</button>
                            </li>
                        }
                    }
                </For>
            </ul>
            <p class="text-sm text-error" role="status" aria-live="polite" data-image-attachment-status="">{move || announcement.get()}</p>
        </div>
    }
}

fn object_url(att: &ImageAttachment) -> Option<String> {
    let array = js_sys::Uint8Array::from(att.bytes.as_slice());
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(&att.content_type);
    let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &opts).ok()?;
    web_sys::Url::create_object_url_with_blob(&blob).ok()
}
```

`mod.rs`:

```rust
mod admission;
mod component;
mod texts;

pub use admission::{ImageAttachmentCaps, ImageRejection, admit};
pub use component::*;
pub use texts::ImageAttachmentTexts;
```

Note: `component.rs` is DOM-bound; guard the module with `#[cfg(target_arch = "wasm32")]` around `mod component;` and its `pub use` if the native build complains about `web_sys::Url::revoke_object_url` (web-sys stubs compile natively, so this is usually unnecessary; keep the file unguarded unless `cargo check` says otherwise).

- [ ] **Step 6: Run tests + wasm check**

Run: `cargo test -p leptos-daisyui-rs --features test-mode image_attachment_field`
Expected: 3 passed.
Run: `cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown`
Expected: clean.

- [ ] **Step 7: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/components
git commit -m "feat(components): ImageAttachmentField with paste, drop and pick

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 6: `HelpdeskTexts`

**Files:**
- Create: `src/patterns/helpdesk/texts.rs`
- Modify: `src/patterns/helpdesk/mod.rs`

**Interfaces:**
- Produces: `HelpdeskTexts` (all `pub String` fields below), `HelpdeskTexts::kind_name(&TicketKind) -> String`, `HelpdeskTexts::priority_name(TicketPriority) -> String`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::helpdesk::model::{TicketKind, TicketPriority};

    #[test]
    fn names_resolve() {
        let t = HelpdeskTexts::default();
        assert_eq!(t.kind_name(&TicketKind::Bug), "Bug");
        assert_eq!(t.kind_name(&TicketKind::Other("Story".into())), "Story");
        assert_eq!(t.priority_name(TicketPriority::Unset), "—");
        assert!(t.rate_limited.contains("{seconds}"));
    }
}
```

- [ ] **Step 2: Run to verify it fails**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::texts` → compile error.

- [ ] **Step 3: Write `texts.rs`**

```rust
use super::model::{TicketKind, TicketPriority};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpdeskTexts {
    pub title: String,
    pub new_request: String,
    pub refresh: String,
    pub buckets_label: String,
    pub bucket_open: String,
    pub bucket_in_progress: String,
    pub bucket_done: String,
    pub bucket_all: String,
    pub search_placeholder: String,
    pub filter_priority: String,
    pub filter_assignee: String,
    pub any_priority: String,
    pub any_assignee: String,
    pub mine_only: String,
    pub reset: String,
    pub col_key: String,
    pub col_summary: String,
    pub col_priority: String,
    pub col_status: String,
    pub col_assignee: String,
    pub col_age: String,
    pub unassigned: String,
    pub kind_bug: String,
    pub kind_request: String,
    pub priority_highest: String,
    pub priority_high: String,
    pub priority_medium: String,
    pub priority_low: String,
    pub priority_lowest: String,
    pub priority_unset: String,
    pub drawer_label: String,
    pub open_in_jira: String,
    pub requester: String,
    pub created: String,
    pub updated: String,
    pub description: String,
    pub attachments: String,
    pub comments: String,
    pub no_comments: String,
    pub comment_placeholder: String,
    pub add_comment: String,
    pub status: String,
    pub assignee: String,
    pub priority: String,
    pub close: String,
    pub dialog_title: String,
    pub kind: String,
    pub summary: String,
    pub summary_required: String,
    pub summary_too_long: String,
    pub description_placeholder: String,
    pub bug_placeholder: String,
    pub context_line: String,
    pub cancel: String,
    pub submit: String,
    pub filed: String,
    pub rate_limited: String,
    pub not_configured_title: String,
    pub submit_failed: String,
    pub action_failed: String,
}

impl Default for HelpdeskTexts {
    fn default() -> Self {
        Self {
            title: "Helpdesk".into(),
            new_request: "New request".into(),
            refresh: "Refresh".into(),
            buckets_label: "Ticket status".into(),
            bucket_open: "Open".into(),
            bucket_in_progress: "In progress".into(),
            bucket_done: "Done".into(),
            bucket_all: "All".into(),
            search_placeholder: "Search key or summary".into(),
            filter_priority: "Priority".into(),
            filter_assignee: "Assignee".into(),
            any_priority: "Any priority".into(),
            any_assignee: "Anyone".into(),
            mine_only: "Mine only".into(),
            reset: "Reset".into(),
            col_key: "Key".into(),
            col_summary: "Summary".into(),
            col_priority: "Priority".into(),
            col_status: "Status".into(),
            col_assignee: "Assignee".into(),
            col_age: "Age".into(),
            unassigned: "—".into(),
            kind_bug: "Bug".into(),
            kind_request: "Request".into(),
            priority_highest: "Highest".into(),
            priority_high: "High".into(),
            priority_medium: "Medium".into(),
            priority_low: "Low".into(),
            priority_lowest: "Lowest".into(),
            priority_unset: "—".into(),
            drawer_label: "Ticket detail".into(),
            open_in_jira: "Open in Jira".into(),
            requester: "Requester".into(),
            created: "Created".into(),
            updated: "Updated".into(),
            description: "Description".into(),
            attachments: "Attachments".into(),
            comments: "Comments".into(),
            no_comments: "No comments yet".into(),
            comment_placeholder: "Add a comment".into(),
            add_comment: "Comment".into(),
            status: "Status".into(),
            assignee: "Assignee".into(),
            priority: "Priority".into(),
            close: "Close".into(),
            dialog_title: "New request".into(),
            kind: "What is this?".into(),
            summary: "Summary".into(),
            summary_required: "A summary is required".into(),
            summary_too_long: "Keep the summary under {max} characters".into(),
            description_placeholder: "What would you like?".into(),
            bug_placeholder: "What I expected / What happened".into(),
            context_line: "Page {route} · Build {build}".into(),
            cancel: "Cancel".into(),
            submit: "Submit".into(),
            filed: "Filed {key}".into(),
            rate_limited: "Too many requests. Try again in {seconds} seconds.".into(),
            not_configured_title: "Helpdesk is not available".into(),
            submit_failed: "Could not file the request".into(),
            action_failed: "Change not saved".into(),
        }
    }
}

impl HelpdeskTexts {
    pub fn kind_name(&self, kind: &TicketKind) -> String {
        match kind {
            TicketKind::Bug => self.kind_bug.clone(),
            TicketKind::Request => self.kind_request.clone(),
            TicketKind::Other(s) => s.clone(),
        }
    }

    pub fn priority_name(&self, p: TicketPriority) -> String {
        match p {
            TicketPriority::Highest => self.priority_highest.clone(),
            TicketPriority::High => self.priority_high.clone(),
            TicketPriority::Medium => self.priority_medium.clone(),
            TicketPriority::Low => self.priority_low.clone(),
            TicketPriority::Lowest => self.priority_lowest.clone(),
            TicketPriority::Unset => self.priority_unset.clone(),
        }
    }
}
```

Add `mod texts; pub use texts::HelpdeskTexts;` to `mod.rs`.

- [ ] **Step 4: Run**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::texts` → 1 passed.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): HelpdeskTexts

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 7: `NewRequestDialog`

**Files:**
- Create: `src/patterns/helpdesk/request_dialog.rs`
- Modify: `src/patterns/helpdesk/mod.rs` (`mod request_dialog; pub use request_dialog::NewRequestDialog;`)

**Interfaces:**
- Consumes: `Modal`, `ModalBox`, `ModalAction`, `Field`, `Input`, `Textarea`, `Button`, `PageStatePanel` + `PageStatePanelKind::Forbidden`, `ImageAttachmentField`, `validate_new_ticket`, `NewTicketCaps`, `HelpdeskTexts`.
- Produces:

```rust
#[component]
pub fn NewRequestDialog(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into)] meta: Signal<Option<HelpdeskMeta>>,
    #[prop(into)] context: Signal<RequestContext>,
    #[prop(into)] texts: Signal<HelpdeskTexts>,
    /// Host submits; resolves with the created ticket or an error.
    on_submit: Callback<(NewTicket, Callback<Result<HelpdeskTicket, HelpdeskError>>)>,
) -> impl IntoView
```

- [ ] **Step 1: Write the failing native test** (bottom of `request_dialog.rs`) for the pure draft helper the view uses:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_builds_a_ticket_and_reports_summary_errors() {
        let mut d = Draft::default();
        d.kind = TicketKind::Bug;
        d.summary = "  ".into();
        let ctx = RequestContext { route: "/r".into(), build: "b".into() };
        let t = HelpdeskTexts::default();
        assert_eq!(d.summary_error(&t, &NewTicketCaps::default()), Some(t.summary_required.clone()));
        d.summary = "Broken".into();
        assert_eq!(d.summary_error(&t, &NewTicketCaps::default()), None);
        let nt = d.to_ticket(&ctx, vec![]);
        assert_eq!((nt.kind, nt.summary.as_str(), nt.context.route.as_str()), (TicketKind::Bug, "Broken", "/r"));
    }
}
```

- [ ] **Step 2: Run to verify it fails**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::request_dialog` → compile error.

- [ ] **Step 3: Write `request_dialog.rs`**

```rust
use super::model::*;
use super::state::{NewTicketCaps, NewTicketError, validate_new_ticket};
use super::texts::HelpdeskTexts;
use crate::components::{
    Button, ButtonColor, ButtonStyle, Field, ImageAttachmentField, Input, Modal, ModalAction,
    ModalBox, Textarea,
};
use crate::patterns::{PageStatePanel, PageStatePanelKind, PageStatePanelTexts};
use leptos::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Draft {
    pub kind: TicketKind,
    pub summary: String,
    pub description: String,
}

impl Default for TicketKind {
    fn default() -> Self {
        TicketKind::Bug
    }
}

impl Draft {
    pub(crate) fn summary_error(&self, t: &HelpdeskTexts, caps: &NewTicketCaps) -> Option<String> {
        let probe = NewTicket { kind: self.kind.clone(), summary: self.summary.clone(), description: String::new(), context: RequestContext::default(), images: vec![] };
        match validate_new_ticket(&probe, caps) {
            Ok(()) => None,
            Err(errs) => errs.iter().find_map(|e| match e {
                NewTicketError::SummaryBlank => Some(t.summary_required.clone()),
                NewTicketError::SummaryTooLong { max } => Some(t.summary_too_long.replace("{max}", &max.to_string())),
                _ => None,
            }),
        }
    }

    pub(crate) fn to_ticket(&self, ctx: &RequestContext, images: Vec<ImageAttachment>) -> NewTicket {
        NewTicket {
            kind: self.kind.clone(),
            summary: self.summary.trim().to_owned(),
            description: self.description.trim().to_owned(),
            context: ctx.clone(),
            images,
        }
    }
}

#[component]
pub fn NewRequestDialog(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into)] meta: Signal<Option<HelpdeskMeta>>,
    #[prop(into)] context: Signal<RequestContext>,
    #[prop(into)] texts: Signal<HelpdeskTexts>,
    on_submit: Callback<(NewTicket, Callback<Result<HelpdeskTicket, HelpdeskError>>)>,
) -> impl IntoView {
    let caps = NewTicketCaps::default();
    let draft = RwSignal::new(Draft::default());
    let images: RwSignal<Vec<ImageAttachment>> = RwSignal::new(Vec::new());
    let touched = RwSignal::new(false);
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let configured = Signal::derive(move || meta.get().is_some_and(|m| m.configured));
    let reason = Signal::derive(move || meta.get().and_then(|m| m.unavailable_reason));
    let kinds = Signal::derive(move || meta.get().map(|m| m.kinds).unwrap_or_else(|| vec![TicketKind::Bug, TicketKind::Request]));
    let summary_error = Signal::derive(move || if touched.get() { draft.get().summary_error(&texts.get(), &caps) } else { None });
    let can_submit = Signal::derive(move || configured.get() && !pending.get() && draft.get().summary_error(&texts.get(), &caps).is_none());

    let reset = move || {
        draft.set(Draft::default());
        images.set(Vec::new());
        touched.set(false);
        error.set(None);
        pending.set(false);
    };
    let close = move || {
        open.set(false);
        reset();
    };

    let submit = move || {
        touched.set(true);
        if !can_submit.get_untracked() {
            return;
        }
        pending.set(true);
        error.set(None);
        let ticket = draft.get_untracked().to_ticket(&context.get_untracked(), images.get_untracked());
        let done = Callback::new(move |r: Result<HelpdeskTicket, HelpdeskError>| match r {
            Ok(_) => close(),
            Err(e) => {
                pending.set(false);
                let t = texts.get_untracked();
                error.set(Some(match e.kind {
                    HelpdeskErrorKind::RateLimited { retry_after_s } => t.rate_limited.replace("{seconds}", &retry_after_s.to_string()),
                    _ => format!("{}: {}", t.submit_failed, e.message),
                }));
            }
        });
        on_submit.run((ticket, done));
    };

    let panel_texts = Signal::derive(move || PageStatePanelTexts { forbidden: texts.get().not_configured_title, ..PageStatePanelTexts::default() });
    let context_line = move || {
        let c = context.get();
        texts.get().context_line.replace("{route}", &c.route).replace("{build}", &c.build)
    };

    view! {
        <Modal
            open=Signal::derive(move || open.get())
            backdrop=true
            label=Signal::derive(move || texts.get().dialog_title)
            on_close_request=Callback::new(move |_| close())
            attr:data-helpdesk-dialog=""
        >
            <ModalBox class="max-w-2xl">
                <h2 class="ld-text-title mb-4">{move || texts.get().dialog_title}</h2>
                <Show when=move || !configured.get()>
                    <div data-helpdesk-not-configured="">
                        <PageStatePanel kind=PageStatePanelKind::Forbidden texts=panel_texts detail=reason />
                    </div>
                </Show>
                <Show when=move || configured.get()>
                    <form class="flex flex-col gap-4" on:submit=move |ev| { ev.prevent_default(); submit(); }>
                        <fieldset class="flex flex-col gap-2" data-helpdesk-kind="">
                            <legend class="text-sm font-medium">{move || texts.get().kind}</legend>
                            <div class="join">
                                <For each=move || kinds.get() key=|k| format!("{k:?}") let:k>
                                    {
                                        let k2 = k.clone();
                                        let k3 = k.clone();
                                        view! {
                                            <input
                                                type="radio"
                                                name="helpdesk-kind"
                                                class="join-item btn btn-sm"
                                                aria-label=move || texts.get().kind_name(&k2)
                                                prop:checked=move || draft.get().kind == k3
                                                on:change=move |_| draft.update(|d| d.kind = k.clone())
                                            />
                                        }
                                    }
                                </For>
                            </div>
                        </fieldset>
                        <Field label=Signal::derive(move || Some(texts.get().summary)) error=summary_error required=true>
                            <Input
                                value=Signal::derive(move || draft.get().summary)
                                maxlength=Some(caps.summary_max as u32)
                                on_input=Callback::new(move |v: String| { touched.set(true); draft.update(|d| d.summary = v); })
                                attr:data-helpdesk-summary=""
                            />
                        </Field>
                        <Field label=Signal::derive(move || Some(texts.get().description))>
                            <Textarea
                                value=Signal::derive(move || draft.get().description)
                                rows=Some(5)
                                maxlength=Some(caps.description_max as u32)
                                placeholder=Signal::derive(move || {
                                    let t = texts.get();
                                    if draft.get().kind == TicketKind::Bug { t.bug_placeholder } else { t.description_placeholder }
                                })
                                on_input=Callback::new(move |v: String| draft.update(|d| d.description = v))
                                attr:data-helpdesk-description=""
                            />
                        </Field>
                        <div data-helpdesk-images="">
                            <ImageAttachmentField images=images capture_document_paste=true disabled=pending />
                        </div>
                        <p class="text-sm text-base-content/75" data-helpdesk-context="">{context_line}</p>
                        <p class="text-sm text-error" role="alert" data-helpdesk-submit-error="">{move || error.get().unwrap_or_default()}</p>
                        <ModalAction>
                            <Button style=ButtonStyle::Ghost on_click=Callback::new(move |_| close()) attr:data-helpdesk-cancel="">{move || texts.get().cancel}</Button>
                            <Button
                                color=ButtonColor::Primary
                                button_type=crate::components::ButtonType::Submit
                                disabled=Signal::derive(move || !can_submit.get())
                                loading=pending
                                attr:data-helpdesk-submit=""
                            >{move || texts.get().submit}</Button>
                        </ModalAction>
                    </form>
                </Show>
            </ModalBox>
        </Modal>
    }
}
```

If `impl Default for TicketKind` conflicts with Task 1 (it does not define one), keep it here or move it to `model.rs`; one definition only. `ButtonType::Submit` is the variant name in `src/components/button/style.rs` (check with `grep -n "enum ButtonType" -A6 src/components/button/style.rs`; the existing default is `Button`).

- [ ] **Step 4: Run**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::request_dialog` → 1 passed; `cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown` clean.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): NewRequestDialog with kind, summary, description and screenshots

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 8: `TicketDetailDrawer`

**Files:**
- Create: `src/patterns/helpdesk/drawer.rs`
- Modify: `src/patterns/helpdesk/mod.rs` (`mod drawer; pub use drawer::{TicketDetailDrawer, TriageAction};`)

**Interfaces:**
- Consumes: `Drawer`, `DrawerContent`, `DrawerSide`, `DrawerOverlay`, `RecordHeader` family, `Select`, `SelectOption`, `Textarea`, `Button`, `Badge`, `RoleCapabilities`, `HelpdeskTexts`, `relative_age`.
- Produces:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriageAction {
    Transition { to_status_id: String },
    Assign { assignee_id: Option<String> },
    SetPriority { priority_id: String },
    Comment { body: String },
}

#[component]
pub fn TicketDetailDrawer(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] detail: Signal<Option<TicketDetail>>,
    #[prop(into)] meta: Signal<Option<HelpdeskMeta>>,
    #[prop(into)] capabilities: Signal<RoleCapabilities>,
    #[prop(into)] texts: Signal<HelpdeskTexts>,
    #[prop(into)] now_ms: Signal<i64>,
    #[prop(into)] feedback: Signal<Option<(bool, String)>>,   // (is_error, message) for this ticket
    on_close: Callback<()>,
    on_action: Callback<TriageAction>,
) -> impl IntoView
```

- [ ] **Step 1: Write the failing native test** (bottom of `drawer.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_status_tone_follows_category() {
        assert_eq!(status_tone(StatusCategory::New), RecordStatusTone::Info);
        assert_eq!(status_tone(StatusCategory::InProgress), RecordStatusTone::Warning);
        assert_eq!(status_tone(StatusCategory::Done), RecordStatusTone::Success);
    }

    #[test]
    fn meta_items_list_requester_created_updated() {
        let t = HelpdeskTexts::default();
        let ticket = HelpdeskTicket {
            key: TicketKey("OF-1".into()), summary: "s".into(), kind: TicketKind::Bug,
            status: TicketStatus { id: "1".into(), name: "Backlog".into(), category: StatusCategory::New },
            priority: TicketPriority::High, assignee: None, requester: Some(Person::new("w", "Chris Forbes")),
            created_at_ms: 0, updated_at_ms: 60_000, comment_count: 0, attachment_count: 0, url: None,
        };
        let items = meta_items(&ticket, &t, 120_000);
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, ["requester", "created", "updated"]);
        assert_eq!(items[0].value, "Chris Forbes");
        assert_eq!(items[1].value, "2m");
        assert_eq!(items[2].value, "1m");
    }
}
```

- [ ] **Step 2: Run to verify it fails**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::drawer` → compile error.

- [ ] **Step 3: Write `drawer.rs`**

```rust
use super::model::*;
use super::state::{RoleCapabilities, relative_age};
use super::texts::HelpdeskTexts;
use crate::components::{
    Badge, BadgeColor, Button, ButtonColor, ButtonSize, ButtonStyle, Drawer, DrawerContent,
    DrawerOverlay, DrawerPlacement, DrawerSide, Select, SelectOption, Textarea,
};
use crate::patterns::{
    RecordBadge, RecordHeader, RecordMetaItem, RecordQuickAction, RecordStatus, RecordStatusTone,
};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriageAction {
    Transition { to_status_id: String },
    Assign { assignee_id: Option<String> },
    SetPriority { priority_id: String },
    Comment { body: String },
}

pub(crate) fn status_tone(c: StatusCategory) -> RecordStatusTone {
    match c {
        StatusCategory::New => RecordStatusTone::Info,
        StatusCategory::InProgress => RecordStatusTone::Warning,
        StatusCategory::Done => RecordStatusTone::Success,
    }
}

pub(crate) fn priority_color(p: TicketPriority) -> BadgeColor {
    match p {
        TicketPriority::Highest | TicketPriority::High => BadgeColor::Error,
        TicketPriority::Medium => BadgeColor::Warning,
        TicketPriority::Low | TicketPriority::Lowest => BadgeColor::Neutral,
        TicketPriority::Unset => BadgeColor::Default,
    }
}

pub(crate) fn meta_items(t: &HelpdeskTicket, texts: &HelpdeskTexts, now_ms: i64) -> Vec<RecordMetaItem> {
    vec![
        RecordMetaItem::new("requester", texts.requester.clone(), t.requester.as_ref().map(|p| p.display_name.clone()).unwrap_or_else(|| texts.unassigned.clone())),
        RecordMetaItem::new("created", texts.created.clone(), relative_age(now_ms, t.created_at_ms)),
        RecordMetaItem::new("updated", texts.updated.clone(), relative_age(now_ms, t.updated_at_ms)),
    ]
}

#[component]
pub fn TicketDetailDrawer(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] detail: Signal<Option<TicketDetail>>,
    #[prop(into)] meta: Signal<Option<HelpdeskMeta>>,
    #[prop(into)] capabilities: Signal<RoleCapabilities>,
    #[prop(into)] texts: Signal<HelpdeskTexts>,
    #[prop(into)] now_ms: Signal<i64>,
    #[prop(into)] feedback: Signal<Option<(bool, String)>>,
    on_close: Callback<()>,
    on_action: Callback<TriageAction>,
) -> impl IntoView {
    let comment_draft = RwSignal::new(String::new());
    let ticket = Signal::derive(move || detail.get().map(|d| d.ticket));
    let triage = Signal::derive(move || capabilities.get().triage);

    let header = move || {
        let t = ticket.get()?;
        let tx = texts.get();
        let mut actions = Vec::new();
        if let Some(url) = &t.url {
            actions.push(RecordQuickAction::new("open-jira", "external-link", tx.open_in_jira.clone()).link(url.clone()).external());
        }
        Some(view! {
            <RecordHeader
                title=t.summary.clone()
                metadata=meta_items(&t, &tx, now_ms.get())
                status=Some(RecordStatus::new(t.status.name.clone()).tone(status_tone(t.status.category)))
                badges=vec![
                    RecordBadge::new("key", t.key.0.clone()),
                    RecordBadge::new("kind", tx.kind_name(&t.kind)).tone(RecordStatusTone::Neutral),
                ]
                actions=actions
                attr:data-helpdesk-drawer-header=""
            />
        })
    };

    let submit_comment = move || {
        let body = comment_draft.get_untracked();
        if body.trim().is_empty() {
            return;
        }
        on_action.run(TriageAction::Comment { body });
        comment_draft.set(String::new());
    };

    view! {
        <Drawer placement=DrawerPlacement::End open=open attr:data-helpdesk-drawer="" attr:data-helpdesk-drawer-open=move || open.get().to_string()>
            <DrawerContent>{()}</DrawerContent>
            <DrawerSide class="z-40">
                <DrawerOverlay attr:on:click=move |_| on_close.run(()) />
                <aside
                    class="flex h-full w-full max-w-xl flex-col gap-4 overflow-y-auto bg-base-100 p-6"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || texts.get().drawer_label
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| { if ev.key() == "Escape" { on_close.run(()); } }
                >
                    <div class="flex justify-end">
                        <Button size=ButtonSize::Sm style=ButtonStyle::Ghost on_click=Callback::new(move |_| on_close.run(())) attr:data-helpdesk-drawer-close="">{move || texts.get().close}</Button>
                    </div>
                    {header}
                    <Show when=move || triage.get()>
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3" data-helpdesk-triage="">
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().status}</span>
                                <Select
                                    value=Signal::derive(move || ticket.get().map(|t| t.status.id).unwrap_or_default())
                                    on_change=Callback::new(move |id: String| on_action.run(TriageAction::Transition { to_status_id: id }))
                                    attr:data-helpdesk-transition=""
                                >
                                    <For each=move || meta.get().map(|m| m.statuses).unwrap_or_default() key=|s| s.id.clone() let:s>
                                        <SelectOption attr:value=s.id.clone()>{s.name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().assignee}</span>
                                <Select
                                    value=Signal::derive(move || ticket.get().and_then(|t| t.assignee.map(|p| p.id)).unwrap_or_default())
                                    on_change=Callback::new(move |id: String| on_action.run(TriageAction::Assign { assignee_id: (!id.is_empty()).then_some(id) }))
                                    attr:data-helpdesk-assign=""
                                >
                                    <SelectOption attr:value="">{move || texts.get().unassigned}</SelectOption>
                                    <For each=move || meta.get().map(|m| m.assignable).unwrap_or_default() key=|p| p.id.clone() let:p>
                                        <SelectOption attr:value=p.id.clone()>{p.display_name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().priority}</span>
                                <Select
                                    value=Signal::derive(move || {
                                        let t = ticket.get();
                                        meta.get().and_then(|m| m.priorities.into_iter().find(|p| Some(p.priority) == t.as_ref().map(|t| t.priority)).map(|p| p.id)).unwrap_or_default()
                                    })
                                    on_change=Callback::new(move |id: String| on_action.run(TriageAction::SetPriority { priority_id: id }))
                                    attr:data-helpdesk-priority=""
                                >
                                    <For each=move || meta.get().map(|m| m.priorities).unwrap_or_default() key=|p| p.id.clone() let:p>
                                        <SelectOption attr:value=p.id.clone()>{p.name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                        </div>
                    </Show>
                    <Show when=move || !triage.get()>
                        <dl class="grid grid-cols-2 gap-2 text-sm" data-helpdesk-readonly-meta="">
                            <dt class="text-base-content/75">{move || texts.get().priority}</dt>
                            <dd><Badge color=Signal::derive(move || priority_color(ticket.get().map(|t| t.priority).unwrap_or(TicketPriority::Unset)))>{move || texts.get().priority_name(ticket.get().map(|t| t.priority).unwrap_or(TicketPriority::Unset))}</Badge></dd>
                            <dt class="text-base-content/75">{move || texts.get().assignee}</dt>
                            <dd>{move || ticket.get().and_then(|t| t.assignee.map(|p| p.display_name)).unwrap_or_else(|| texts.get().unassigned)}</dd>
                        </dl>
                    </Show>
                    <p class="text-sm" role="status" aria-live="polite"
                        class:text-error=move || feedback.get().is_some_and(|f| f.0)
                        class:text-success=move || feedback.get().is_some_and(|f| !f.0)
                        data-helpdesk-action-feedback=""
                        data-helpdesk-action-feedback-state=move || feedback.get().map(|f| if f.0 { "error" } else { "success" }).unwrap_or("idle")
                    >{move || feedback.get().map(|f| f.1).unwrap_or_default()}</p>
                    <section class="flex flex-col gap-2">
                        <h3 class="ld-text-subtitle">{move || texts.get().description}</h3>
                        <p class="whitespace-pre-wrap text-sm" data-helpdesk-description-text="">{move || detail.get().map(|d| d.description).unwrap_or_default()}</p>
                    </section>
                    <Show when=move || detail.get().is_some_and(|d| !d.attachments.is_empty())>
                        <section class="flex flex-col gap-2">
                            <h3 class="ld-text-subtitle">{move || texts.get().attachments}</h3>
                            <ul class="flex flex-wrap gap-2">
                                <For each=move || detail.get().map(|d| d.attachments).unwrap_or_default() key=|a| a.id.clone() let:a>
                                    <li data-helpdesk-attachment="">
                                        <a href=a.url.clone() target="_blank" rel="noopener" class="link text-sm">
                                            <img src=a.thumbnail_url.clone().unwrap_or_else(|| a.url.clone()) alt=a.filename.clone() class="h-16 w-16 rounded object-cover" />
                                            <span class="block max-w-16 truncate">{a.filename.clone()}</span>
                                        </a>
                                    </li>
                                </For>
                            </ul>
                        </section>
                    </Show>
                    <section class="flex flex-col gap-2">
                        <h3 class="ld-text-subtitle">{move || texts.get().comments}</h3>
                        <ol class="flex flex-col gap-2" data-helpdesk-comments="">
                            <For each=move || detail.get().map(|d| d.comments).unwrap_or_default() key=|c| c.id.clone() let:c>
                                <li class="rounded-box bg-base-200 p-3 text-sm" data-helpdesk-comment="">
                                    <div class="mb-1 flex gap-2 text-base-content/75"><span class="font-medium">{c.author.display_name.clone()}</span><span>{move || relative_age(now_ms.get(), c.created_at_ms)}</span></div>
                                    <p class="whitespace-pre-wrap">{c.body.clone()}</p>
                                </li>
                            </For>
                        </ol>
                        <Show when=move || detail.get().is_some_and(|d| d.comments.is_empty())>
                            <p class="text-sm text-base-content/75">{move || texts.get().no_comments}</p>
                        </Show>
                        <Show when=move || capabilities.get().comment>
                            <div class="flex flex-col gap-2">
                                <Textarea value=comment_draft rows=Some(3) placeholder=Signal::derive(move || texts.get().comment_placeholder) on_input=Callback::new(move |v: String| comment_draft.set(v)) attr:data-helpdesk-comment-input="" />
                                <div class="flex justify-end">
                                    <Button color=ButtonColor::Primary size=ButtonSize::Sm disabled=Signal::derive(move || comment_draft.get().trim().is_empty()) on_click=Callback::new(move |_| submit_comment()) attr:data-helpdesk-comment-submit="">{move || texts.get().add_comment}</Button>
                                </div>
                            </div>
                        </Show>
                    </section>
                </aside>
            </DrawerSide>
        </Drawer>
    }
}
```

`DrawerPlacement::End` is the placement enum in `src/components/drawer/style.rs` (confirm the variant name with `grep -n "enum DrawerPlacement" -A4 src/components/drawer/style.rs`). Where `attr:on:click` is not accepted on `DrawerOverlay`, wrap the overlay in a `<div on:click=…>`.

- [ ] **Step 4: Run**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::drawer` → 2 passed; wasm check clean.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): TicketDetailDrawer with triage selects and comments

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 9: `Helpdesk` composite

**Files:**
- Create: `src/patterns/helpdesk/component.rs`
- Modify: `src/patterns/helpdesk/mod.rs` (`mod component; pub use component::Helpdesk;`)

**Interfaces:**
- Consumes: everything above plus `PageHeader`, `SelectableSummaryGroup`, `SelectableSummaryItem`, `FilterBar`, `FilterResultSummary`, `EntityTable`, `EntityColumn`, `PageStatePanel`, `Toast`, `AvatarBadge`, `AvatarBadgeSize`, `Toggle`.
- Produces:

```rust
#[component]
pub fn Helpdesk(
    backend: Rc<dyn HelpdeskBackend>,
    #[prop(into)] role: Signal<HelpdeskRole>,
    #[prop(into)] context: Signal<RequestContext>,
    #[prop(optional, into)] current_user_id: Signal<Option<String>>,
    #[prop(optional, into, default = Signal::stored(HelpdeskTexts::default()))] texts: Signal<HelpdeskTexts>,
    #[prop(optional)] open_request: Option<RwSignal<bool>>,
    #[prop(optional, into)] now_ms: Option<Signal<i64>>,
    #[prop(optional, into)] class: &'static str,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView
```

- [ ] **Step 1: Write the failing native test** for the pure list-merge helper:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn t(key: &str, updated: i64) -> HelpdeskTicket {
        HelpdeskTicket {
            key: TicketKey(key.into()), summary: key.into(), kind: TicketKind::Bug,
            status: TicketStatus { id: "1".into(), name: "B".into(), category: StatusCategory::New },
            priority: TicketPriority::Medium, assignee: None, requester: None,
            created_at_ms: 0, updated_at_ms: updated, comment_count: 0, attachment_count: 0, url: None,
        }
    }

    #[test]
    fn replace_by_key_keeps_order_and_upserts() {
        let mut v = vec![t("A", 1), t("B", 2)];
        replace_by_key(&mut v, t("B", 9));
        assert_eq!(v[1].updated_at_ms, 9);
        replace_by_key(&mut v, t("C", 3));
        assert_eq!(v[0].key.0, "C", "unknown keys are prepended");
    }

    #[test]
    fn sorted_view_is_updated_desc() {
        let v = vec![t("A", 1), t("B", 3), t("C", 2)];
        let s = sorted_desc(&v);
        assert_eq!(s.iter().map(|t| t.key.0.as_str()).collect::<Vec<_>>(), ["B", "C", "A"]);
    }
}
```

- [ ] **Step 2: Run to verify it fails**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::component` → compile error.

- [ ] **Step 3: Write `component.rs`**

```rust
use super::backend::HelpdeskBackend;
use super::drawer::{TicketDetailDrawer, TriageAction, priority_color, status_tone};
use super::model::*;
use super::request_dialog::NewRequestDialog;
use super::state::{Bucket, RoleCapabilities, TicketFilter, bucket_counts, relative_age};
use super::texts::HelpdeskTexts;
use crate::components::{
    Badge, BadgeColor, BadgeSize, Button, ButtonColor, ButtonSize, EntityColumn, EntityTable, Input,
    Select, SelectOption, Toast, Toggle,
};
use crate::patterns::{
    FilterBar, FilterResultSummary, LIST_PAGE_BASE_CLASS, PageHeader, PageStatePanel,
    PageStatePanelKind, SelectableSummaryGroup, SelectableSummaryItem,
};
use crate::widgets::{AvatarBadge, AvatarBadgeSize};
use leptos::prelude::*;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

pub(crate) fn replace_by_key(v: &mut Vec<HelpdeskTicket>, t: HelpdeskTicket) {
    match v.iter().position(|x| x.key == t.key) {
        Some(i) => v[i] = t,
        None => v.insert(0, t),
    }
}

pub(crate) fn sorted_desc(v: &[HelpdeskTicket]) -> Vec<HelpdeskTicket> {
    let mut s = v.to_vec();
    s.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));
    s
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListPhase {
    Loading,
    Ready,
    Error,
}

fn js_now_ms() -> i64 {
    js_sys::Date::now() as i64
}

#[component]
pub fn Helpdesk(
    backend: Rc<dyn HelpdeskBackend>,
    #[prop(into)] role: Signal<HelpdeskRole>,
    #[prop(into)] context: Signal<RequestContext>,
    #[prop(optional, into)] current_user_id: Signal<Option<String>>,
    #[prop(optional, into, default = Signal::stored(HelpdeskTexts::default()))] texts: Signal<HelpdeskTexts>,
    #[prop(optional)] open_request: Option<RwSignal<bool>>,
    #[prop(optional, into)] now_ms: Option<Signal<i64>>,
    #[prop(optional, into)] class: &'static str,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let backend = StoredValue::new_local(backend);
    let caps = Signal::derive(move || RoleCapabilities::for_role(role.get()));
    let now = now_ms.unwrap_or_else(|| Signal::derive(js_now_ms));
    let dialog_open = open_request.unwrap_or_else(|| RwSignal::new(false));

    let meta: RwSignal<Option<HelpdeskMeta>> = RwSignal::new(None);
    let tickets: RwSignal<Vec<HelpdeskTicket>> = RwSignal::new(Vec::new());
    let phase = RwSignal::new(ListPhase::Loading);
    let list_error = RwSignal::new(Option::<String>::None);
    let filter = RwSignal::new(TicketFilter::default());
    let selected: RwSignal<Option<TicketKey>> = RwSignal::new(None);
    let detail: RwSignal<Option<TicketDetail>> = RwSignal::new(None);
    let feedback: RwSignal<Option<(bool, String)>> = RwSignal::new(None);
    let toast: RwSignal<Option<HelpdeskTicket>> = RwSignal::new(None);
    let generation = RwSignal::new(0u32);

    let load_meta = move || {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.meta());
            if let Ok(m) = fut.await {
                meta.try_set(Some(m));
            }
        });
    };

    let load_list = move || {
        let scope = caps.get_untracked().scope;
        let gen = generation.get_untracked() + 1;
        generation.set(gen);
        phase.set(ListPhase::Loading);
        spawn_local(async move {
            let fut = backend.with_value(|b| b.list(scope));
            let r = fut.await;
            if generation.try_get_untracked() != Some(gen) {
                return;
            }
            match r {
                Ok(v) => {
                    tickets.try_set(v);
                    phase.try_set(ListPhase::Ready);
                }
                Err(e) => {
                    list_error.try_set(Some(e.message));
                    phase.try_set(ListPhase::Error);
                }
            }
        });
    };

    load_meta();
    Effect::new(move |_| {
        let _ = role.get();
        load_list();
    });

    let open_detail = move |key: TicketKey| {
        selected.set(Some(key.clone()));
        feedback.set(None);
        detail.set(None);
        spawn_local(async move {
            let fut = backend.with_value(|b| b.detail(&key));
            if let Ok(d) = fut.await
                && selected.try_get_untracked().flatten().as_ref() == Some(&key)
            {
                detail.try_set(Some(d));
            }
        });
    };
    let close_detail = move || {
        selected.set(None);
        detail.set(None);
    };

    let apply_ticket = move |t: HelpdeskTicket| {
        tickets.update(|v| replace_by_key(v, t.clone()));
        detail.update(|d| {
            if let Some(d) = d.as_mut()
                && d.ticket.key == t.key
            {
                d.ticket = t;
            }
        });
    };

    let on_action = Callback::new(move |a: TriageAction| {
        let Some(key) = selected.get_untracked() else { return };
        let before = detail.get_untracked();
        feedback.set(None);
        spawn_local(async move {
            let r: Result<(), HelpdeskError> = match a {
                TriageAction::Transition { to_status_id } => backend.with_value(|b| b.transition(&key, &to_status_id)).await.map(apply_ticket),
                TriageAction::Assign { assignee_id } => backend.with_value(|b| b.assign(&key, assignee_id.as_deref())).await.map(apply_ticket),
                TriageAction::SetPriority { priority_id } => backend.with_value(|b| b.set_priority(&key, &priority_id)).await.map(apply_ticket),
                TriageAction::Comment { body } => backend.with_value(|b| b.comment(&key, &body)).await.map(|c| {
                    detail.update(|d| if let Some(d) = d.as_mut() { d.comments.push(c); d.ticket.comment_count += 1; });
                    if let Some(d) = detail.get_untracked() { apply_ticket(d.ticket); }
                }),
            };
            match r {
                Ok(()) => feedback.try_set(Some((false, texts.get_untracked().priority_name(TicketPriority::Unset).replace('—', "").trim().to_owned()))),
                Err(e) => {
                    detail.try_set(before);
                    feedback.try_set(Some((true, format!("{}: {}", texts.get_untracked().action_failed, e.message))));
                }
            }
        });
    });

    let on_submit = Callback::new(move |(nt, done): (NewTicket, Callback<Result<HelpdeskTicket, HelpdeskError>>)| {
        spawn_local(async move {
            let r = backend.with_value(|b| b.create(nt)).await;
            if let Ok(t) = &r {
                tickets.update(|v| replace_by_key(v, t.clone()));
                toast.set(Some(t.clone()));
            }
            done.run(r);
        });
    });

    let counts = Signal::derive(move || bucket_counts(&tickets.get()));
    let bucket_items = Signal::derive(move || {
        let t = texts.get();
        let c = counts.get();
        vec![
            SelectableSummaryItem::new(Bucket::Open.id(), t.bucket_open, c.open),
            SelectableSummaryItem::new(Bucket::InProgress.id(), t.bucket_in_progress, c.in_progress),
            SelectableSummaryItem::new(Bucket::Done.id(), t.bucket_done, c.done),
        ]
    });
    let visible = Signal::derive(move || {
        let f = filter.get();
        let me = current_user_id.get();
        sorted_desc(&tickets.get()).into_iter().filter(|t| f.matches(t, me.as_deref())).collect::<Vec<_>>()
    });
    let table_data: Signal<Rc<Vec<HelpdeskTicket>>, LocalStorage> = Signal::derive_local(move || Rc::new(visible.get()));
    let result = Signal::derive(move || FilterResultSummary { visible: visible.get().len(), total: tickets.get().len() });

    let columns = {
        let tx = texts.get_untracked();
        vec![
            EntityColumn::<HelpdeskTicket>::new("key", tx.col_key.clone(), |t| t.key.0.clone()).identifier().with_width(96).required(),
            EntityColumn::new("summary", tx.col_summary.clone(), |t| t.summary.clone())
                .render_with(move |t| {
                    let k = texts.get_untracked().kind_name(&t.kind);
                    view! { <span class="flex items-center gap-2"><Badge size=BadgeSize::Xs color=BadgeColor::Neutral>{k}</Badge><span class="truncate">{t.summary.clone()}</span></span> }.into_any()
                })
                .ellipsis()
                .required(),
            EntityColumn::new("priority", tx.col_priority.clone(), move |t| texts.get_untracked().priority_name(t.priority))
                .render_with(move |t| view! { <Badge size=BadgeSize::Sm color=priority_color(t.priority)>{texts.get_untracked().priority_name(t.priority)}</Badge> }.into_any())
                .sortable_by_key(|t| t.priority.rank())
                .with_width(112),
            EntityColumn::new("status", tx.col_status.clone(), |t| t.status.name.clone())
                .render_with(move |t| {
                    let color = match status_tone(t.status.category) { crate::patterns::RecordStatusTone::Success => BadgeColor::Success, crate::patterns::RecordStatusTone::Warning => BadgeColor::Warning, _ => BadgeColor::Info };
                    view! { <Badge size=BadgeSize::Sm color=color>{t.status.name.clone()}</Badge> }.into_any()
                })
                .with_width(160),
            EntityColumn::new("assignee", tx.col_assignee.clone(), move |t| t.assignee.as_ref().map(|p| p.display_name.clone()).unwrap_or_else(|| texts.get_untracked().unassigned))
                .render_with(move |t| match &t.assignee {
                    Some(p) => view! { <AvatarBadge initials=p.initials.clone() name=p.display_name.clone() size=AvatarBadgeSize::Sm /> }.into_any(),
                    None => view! { <span class="text-base-content/75">{texts.get_untracked().unassigned}</span> }.into_any(),
                })
                .with_width(96),
            EntityColumn::new("age", tx.col_age.clone(), move |t| relative_age(now.get_untracked(), t.created_at_ms))
                .render_with(move |t| view! { <time datetime=t.created_at_ms.to_string() class="tabular-nums">{relative_age(now.get_untracked(), t.created_at_ms)}</time> }.into_any())
                .sortable_by_key(|t| std::cmp::Reverse(t.created_at_ms))
                .with_width(72),
        ]
    };

    let on_bucket = Callback::new(move |id: String| {
        filter.update(|f| f.bucket = if f.bucket == Bucket::from_id(&id) { None } else { Bucket::from_id(&id) });
    });
    let selected_bucket = Signal::derive(move || filter.get().bucket.map(|b| b.id().to_owned()));

    view! {
        <div class=format!("{LIST_PAGE_BASE_CLASS} {class}") data-helpdesk="" data-helpdesk-role=move || match role.get() { HelpdeskRole::Requester => "requester", HelpdeskRole::Support => "support" } node_ref=node_ref>
            <PageHeader title=Signal::derive(move || texts.get().title) actions=Box::new(move || view! {
                <Button size=ButtonSize::Sm on_click=Callback::new(move |_| load_list()) attr:data-helpdesk-refresh="">{move || texts.get().refresh}</Button>
                <Button color=ButtonColor::Primary size=ButtonSize::Sm on_click=Callback::new(move |_| dialog_open.set(true)) attr:data-helpdesk-new-request="">{move || texts.get().new_request}</Button>
            }.into_any()) />
            <SelectableSummaryGroup label=Signal::derive(move || texts.get().buckets_label) items=bucket_items selected=selected_bucket on_select=on_bucket attr:data-helpdesk-buckets="" />
            <FilterBar
                result=result
                on_reset=Callback::new(move |_| filter.set(TicketFilter::default()))
                search=Box::new(move || view! {
                    <Input value=Signal::derive(move || filter.get().search) placeholder=Signal::derive(move || texts.get().search_placeholder) on_input=Callback::new(move |v: String| filter.update(|f| f.search = v)) attr:data-helpdesk-search="" />
                }.into_any())
            >
                <Select value=Signal::derive(move || filter.get().priority.map(|p| format!("{p:?}")).unwrap_or_default()) on_change=Callback::new(move |v: String| filter.update(|f| f.priority = [TicketPriority::Highest, TicketPriority::High, TicketPriority::Medium, TicketPriority::Low, TicketPriority::Lowest].into_iter().find(|p| format!("{p:?}") == v))) attr:data-helpdesk-filter-priority="">
                    <SelectOption attr:value="">{move || texts.get().any_priority}</SelectOption>
                    {[TicketPriority::Highest, TicketPriority::High, TicketPriority::Medium, TicketPriority::Low, TicketPriority::Lowest].into_iter().map(|p| view! { <SelectOption attr:value=format!("{p:?}")>{move || texts.get().priority_name(p)}</SelectOption> }).collect_view()}
                </Select>
                <Show when=move || caps.get().assignee_filter>
                    <Select value=Signal::derive(move || filter.get().assignee_id.unwrap_or_default()) on_change=Callback::new(move |v: String| filter.update(|f| f.assignee_id = (!v.is_empty()).then_some(v))) attr:data-helpdesk-filter-assignee="">
                        <SelectOption attr:value="">{move || texts.get().any_assignee}</SelectOption>
                        <For each=move || meta.get().map(|m| m.assignable).unwrap_or_default() key=|p| p.id.clone() let:p>
                            <SelectOption attr:value=p.id.clone()>{p.display_name.clone()}</SelectOption>
                        </For>
                    </Select>
                </Show>
                <Show when=move || caps.get().mine_only_toggle>
                    <label class="flex items-center gap-2 text-sm">
                        <Toggle attr:data-helpdesk-mine-only="" attr:checked=move || filter.get().mine_only on:change=move |_| filter.update(|f| f.mine_only = !f.mine_only) />
                        {move || texts.get().mine_only}
                    </label>
                </Show>
            </FilterBar>
            {move || match phase.get() {
                ListPhase::Loading if tickets.get().is_empty() => view! { <div data-helpdesk-state="loading"><PageStatePanel kind=PageStatePanelKind::InitialLoading /></div> }.into_any(),
                ListPhase::Error => view! { <div data-helpdesk-state="error"><PageStatePanel kind=PageStatePanelKind::InitialError detail=list_error on_retry=Callback::new(move |_| load_list()) /></div> }.into_any(),
                _ if tickets.get().is_empty() => view! { <div data-helpdesk-state="empty"><PageStatePanel kind=PageStatePanelKind::EmptyDataset /></div> }.into_any(),
                _ => view! {
                    <div data-helpdesk-state="ready" data-helpdesk-table="">
                        <EntityTable
                            data=table_data
                            columns=columns.clone()
                            row_key=Rc::new(|t: &HelpdeskTicket| t.key.0.clone())
                            dataset_identity=Signal::derive(move || format!("helpdesk-{}", generation.get()))
                            on_row_activate=Callback::new(move |k: String| open_detail(TicketKey(k)))
                        />
                    </div>
                }.into_any(),
            }}
            <TicketDetailDrawer
                open=Signal::derive(move || selected.get().is_some())
                detail=detail
                meta=meta
                capabilities=caps
                texts=texts
                now_ms=now
                feedback=feedback
                on_close=Callback::new(move |_| close_detail())
                on_action=on_action
            />
            <NewRequestDialog open=dialog_open meta=meta context=context texts=texts on_submit=on_submit />
            <Show when=move || toast.get().is_some()>
                <Toast attr:data-helpdesk-toast="">
                    <div class="alert alert-success">
                        {move || { let t = toast.get(); let tx = texts.get(); t.map(|t| { let msg = tx.filed.replace("{key}", &t.key.0); match t.url { Some(u) => view! { <a class="link" href=u target="_blank" rel="noopener">{msg}</a> }.into_any(), None => view! { <span>{msg}</span> }.into_any() } }) }}
                        <Button size=ButtonSize::Xs on_click=Callback::new(move |_| toast.set(None))>"×"</Button>
                    </div>
                </Toast>
            </Show>
        </div>
    }
}
```

Fix the success-feedback line: replace the odd `priority_name(...).replace(...)` expression with a dedicated text. Add `pub saved: String` ("Saved") to `HelpdeskTexts` in Task 6 (do it now, in this task) and use `feedback.try_set(Some((false, texts.get_untracked().saved)))`.

`Signal::derive_local` exists for `LocalStorage` signals in Leptos 0.8; if the compiler disagrees, hold `table_data` as `RwSignal::new_local(Rc::new(Vec::new()))` updated by an `Effect` from `visible`.

- [ ] **Step 4: Run**: `cargo test -p leptos-daisyui-rs --features test-mode helpdesk::` → 18 passed; `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings` clean; `cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown` clean.

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-rs
git add src/patterns/helpdesk
git commit -m "feat(helpdesk): Helpdesk composite (buckets, filters, table, drawer, dialog)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 10: Demo page, sidebar entry, audit registration

**Files:**
- Create: `demo/src/demos/helpdesk.rs`
- Modify: `demo/src/demos/mod.rs` (add `pub mod helpdesk; pub use helpdesk::*;`)
- Modify: `demo/src/main.rs:100-135` (add `<Route path=path!("/helpdesk") view=HelpdeskDemo />` after `/dropdown`)
- Modify: `demo/src/core/layout.rs:335-342` (add a `ComponentItem { name: "Helpdesk", href: "/components/helpdesk", value: "helpdesk" }` after Client Snapshot List)
- Modify: `tests/layout_audit_smoke.rs:91-105,185-196`, `tests/style_audit_smoke.rs:177-196,400-416`

- [ ] **Step 1: Write `demo/src/demos/helpdesk.rs`**

```rust
use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    Helpdesk, HelpdeskFault, HelpdeskRole, InMemoryHelpdeskBackend, RequestContext, SEED_ME,
};
use std::rc::Rc;

#[component]
pub fn HelpdeskDemo() -> impl IntoView {
    let role = RwSignal::new(HelpdeskRole::Support);
    let backend = Rc::new(InMemoryHelpdeskBackend::seeded());
    let ctx = RequestContext { route: "/components/helpdesk".into(), build: "demo".into() };
    view! {
        <ContentLayout title="Helpdesk" description="Role-switched Jira ticket board with a New Request dialog, driven by a HelpdeskBackend">
            <Section title="Role">
                <div class="join" data-testid="helpdesk-role-switch">
                    <button class="join-item btn btn-sm" class:btn-active=move || role.get() == HelpdeskRole::Requester on:click=move |_| role.set(HelpdeskRole::Requester)>"Requester"</button>
                    <button class="join-item btn btn-sm" class:btn-active=move || role.get() == HelpdeskRole::Support on:click=move |_| role.set(HelpdeskRole::Support)>"Support"</button>
                </div>
            </Section>
            <Section title="Board">
                <div class="w-full min-w-0">
                    <Helpdesk backend=backend.clone() role=role context=ctx.clone() current_user_id=Some(SEED_ME.to_owned()) />
                </div>
            </Section>
        </ContentLayout>
    }
}

/// Browser-proof fixture: both roles on one document, plus a not-configured
/// instance. Fault selection by `?fault=not-configured|fail-writes`.
#[component]
pub fn HelpdeskFixture() -> impl IntoView {
    let search = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
    let fault = if search.contains("fault=not-configured") {
        Some(HelpdeskFault::NotConfigured)
    } else if search.contains("fault=fail-writes") {
        Some(HelpdeskFault::FailWrites)
    } else {
        None
    };
    let mut backend = InMemoryHelpdeskBackend::seeded();
    if let Some(f) = fault {
        backend = backend.with_fault(f);
    }
    let calls_backend = backend.clone();
    crate::debug::register_signal("helpdesk_calls", move || {
        serde_json::to_value(
            calls_backend.calls().iter().map(|c| format!("{c:?}")).collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null)
    });
    let backend = Rc::new(backend);
    let ctx = RequestContext { route: "/fixture".into(), build: "test".into() };
    let fixed_now = Signal::stored(1_800_000_000_000i64);
    view! {
        <div class="flex flex-col gap-8">
            <section id="helpdesk-support" data-testid="helpdesk-support">
                <Helpdesk backend=backend.clone() role=HelpdeskRole::Support context=ctx.clone() current_user_id=Some(SEED_ME.to_owned()) now_ms=fixed_now />
            </section>
            <section id="helpdesk-requester" data-testid="helpdesk-requester">
                <Helpdesk backend=backend.clone() role=HelpdeskRole::Requester context=ctx.clone() current_user_id=Some(SEED_ME.to_owned()) now_ms=fixed_now />
            </section>
        </div>
    }
}
```

`crate::debug::register_signal` is the existing demo helper (`demo/src/debug.rs`); the fixture host calls `debug::install_debug_bridge()` so the value is readable as `window.__ldui_debug.helpdesk_calls` — check the exact bridge global name in `demo/src/debug.rs` and use it verbatim in the test.

- [ ] **Step 2: Register the route and sidebar entry** as listed under Files.

- [ ] **Step 3: Audit registration.** In `tests/layout_audit_smoke.rs` append `("/components/helpdesk", 0, 0),` to `PAGES` and `audit_test!(helpdesk_layout_is_clean, 13);`. In `tests/style_audit_smoke.rs` append

```rust
    (
        "/components/helpdesk",
        &[
            (family::TYPOGRAPHY, 0),
            (family::SHAPE, 0),
            (family::DEPTH, 0),
        ],
    ),
```

and `style_audit_test!(helpdesk_style_is_within_ceiling, 11);`. Ceilings start at zero; if the first run reports findings, fix the markup rather than raising a ceiling (see `doc/visual-quality/`).

- [ ] **Step 4: Build the demo**

Run: `cargo check -p leptos-daisyui-showcase --target wasm32-unknown-unknown`
Expected: clean. Then `cargo xtask check-demo` (from the repo root).

- [ ] **Step 5: fmt and commit**

```bash
cargo fmt -p leptos-daisyui-showcase
git add demo/src tests/layout_audit_smoke.rs tests/style_audit_smoke.rs
git commit -m "feat(demo): Helpdesk showcase page and audit registration

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 11: Test-host fixture, browser lane, xtask registration

**Files:**
- Modify: `demo/src/client_snapshot_test_host.rs` (add `#[path = "demos/helpdesk.rs"] mod helpdesk;`, `use helpdesk::HelpdeskFixture;`, a `helpdesk_fixture` pathname check `ends_with("/helpdesk-fixture")`, and an `else if helpdesk_fixture { view! { <HelpdeskFixture /> }.into_any() }` branch before the existing ones)
- Create: `tests/helpdesk_smoke.rs`
- Modify: `xtask/src/main.rs`: add `helpdesk_step()` after `snapshot_table_page_filter_actions_step()` (line 410); CLI arm `"test-helpdesk" => run_steps(&[helpdesk_step()]),` after the `test-snapshot-table-page-filter-actions` arm (line 2344); `test-helpdesk` token in the usage string (line 2364); `steps.push(helpdesk_step());` after `snapshot_table_page_filter_actions_step()` in `full_steps()` (line 698); `"demo/src/demos/helpdesk.rs",` in `CLIENT_SNAPSHOT_SOURCE_INPUTS` (line 787).
- Modify: `CLAUDE.md` build-commands list (add `cargo xtask test-helpdesk`).

**Interfaces:**
- Consumes: `HelpdeskFixture` (Task 10), `tests/common` helpers, `pixelproof_web::a11y::Axe`.

- [ ] **Step 1: xtask step**

```rust
/// Browser proof for the `Helpdesk` composite: both roles on one document
/// against the in-memory backend, so every positive assertion (buckets,
/// triage selects, paste intake, filing) has its negative control.
fn helpdesk_step() -> Step {
    Step {
        name: "test-helpdesk",
        run: Run::BrowserSuite {
            test: "helpdesk_smoke",
            html_target: Some("client-snapshot-test-host.html"),
        },
    }
}
```

- [ ] **Step 2: Write `tests/helpdesk_smoke.rs`**

```rust
//! Real-browser proof for `patterns::Helpdesk` (both roles on one document,
//! in-memory backend). See doc/plans/2026-09-14-helpdesk-composite-design.md §8.

mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector};
use serde_json::{Value, json};

const PAGE: &str = "/helpdesk-fixture";
const SUPPORT: &str = "#helpdesk-support";
const REQUESTER: &str = "#helpdesk-requester";

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page().evaluate(expr).await.expect("evaluate helpdesk fixture").into_value().expect("helpdesk expression returns JSON")
}

async fn snapshot(h: &pixelproof_web::Harness, root: &str) -> Value {
    eval_json(h, &format!(r#"(() => {{
        const root = document.querySelector('{root}');
        const bucket = id => root.querySelector('[data-helpdesk-buckets] [data-selectable-summary-item="' + id + '"] [data-selectable-summary-count]')?.textContent?.trim() ?? null;
        return {{
            role: root.querySelector('[data-helpdesk]').getAttribute('data-helpdesk-role'),
            open: bucket('open'), inProgress: bucket('in-progress'), done: bucket('done'),
            rows: root.querySelectorAll('[data-helpdesk-table] tbody tr').length,
            keys: Array.from(root.querySelectorAll('[data-helpdesk-table] tbody tr')).map(r => r.textContent.match(/OF-\d+/)?.[0] ?? null),
            assigneeFilter: root.querySelector('[data-helpdesk-filter-assignee]') !== null,
            mineOnly: root.querySelector('[data-helpdesk-mine-only]') !== null,
            drawerOpen: root.querySelector('[data-helpdesk-drawer]')?.getAttribute('data-helpdesk-drawer-open') ?? null,
            drawerKey: root.querySelector('[data-helpdesk-drawer-header]')?.textContent?.match(/OF-\d+/)?.[0] ?? null,
            transition: root.querySelector('[data-helpdesk-transition]') !== null,
            feedbackState: root.querySelector('[data-helpdesk-action-feedback]')?.getAttribute('data-helpdesk-action-feedback-state') ?? null,
            comments: root.querySelectorAll('[data-helpdesk-comment]').length,
            dialogOpen: root.querySelector('[data-helpdesk-dialog]')?.hasAttribute('open') ?? false,
            notConfigured: root.querySelector('[data-helpdesk-not-configured]') !== null,
            submitPresent: root.querySelector('[data-helpdesk-submit]') !== null,
            submitDisabled: root.querySelector('[data-helpdesk-submit]')?.disabled ?? null,
            images: root.querySelectorAll('[data-image-attachment-item]').length,
            imageStatus: root.querySelector('[data-image-attachment-status]')?.textContent?.trim() ?? null,
        }};
    }})()"#)).await
}

/// The selectable-summary hooks above must match `src/patterns/selectable_summary.rs`;
/// run `grep -n "data-selectable-summary" src/patterns/selectable_summary.rs` and
/// adjust the two attribute names before the first run.

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn buckets_count_the_seed_and_filter_rows() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!((s["open"].as_str(), s["inProgress"].as_str(), s["done"].as_str()), (Some("5"), Some("3"), Some("4")), "{s}");
    assert_eq!(s["rows"], json!(12));
    click(&h, &format!("{SUPPORT} [data-selectable-summary-item=\"open\"]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["rows"], json!(5), "open bucket filters to New-category rows: {s}");
    click(&h, &format!("{SUPPORT} [data-selectable-summary-item=\"open\"]")).await;
    assert_eq!(snapshot(&h, SUPPORT).await["rows"], json!(12), "second click clears the bucket");
    assert_no_browser_errors(&h, "buckets").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn requester_sees_only_own_tickets_and_no_support_controls() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-state=\"ready\"]")).await;
    let r = snapshot(&h, REQUESTER).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(r["role"], json!("requester"));
    assert_eq!(r["rows"], json!(6), "seed has six tickets requested by SEED_ME: {r}");
    assert!(!r["assigneeFilter"].as_bool().unwrap() && !r["mineOnly"].as_bool().unwrap(), "{r}");
    assert!(s["assigneeFilter"].as_bool().unwrap() && s["mineOnly"].as_bool().unwrap(), "negative control: {s}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn row_activation_opens_the_drawer_and_escape_closes_it() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-drawer-header]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["drawerOpen"], json!("true"));
    assert!(s["drawerKey"].as_str().unwrap().starts_with("OF-"), "{s}");
    assert_eq!(snapshot(&h, REQUESTER).await["drawerOpen"], json!("false"), "negative control");
    eval_json(&h, &format!("(() => {{ const a = document.querySelector('{SUPPORT} [data-helpdesk-drawer] aside'); a.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Escape', bubbles: true }})); return true; }})()")).await;
    assert_eq!(snapshot(&h, SUPPORT).await["drawerOpen"], json!("false"));
    assert_no_browser_errors(&h, "drawer").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn support_transition_writes_and_requester_has_no_selects() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-transition]")).await;
    eval_json(&h, &format!("(() => {{ const s = document.querySelector('{SUPPORT} [data-helpdesk-transition]'); s.value = '10003'; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }})()")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-action-feedback-state=\"success\"]")).await;
    let calls = eval_json(&h, "JSON.stringify(window.__ldui_debug ? window.__ldui_debug.helpdesk_calls() : [])").await;
    assert!(calls.as_str().unwrap().contains("Transition"), "backend saw the transition: {calls}");
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["done"], json!("5"), "Done bucket grew by one: {s}");

    click(&h, &format!("{REQUESTER} [data-helpdesk-table] tbody tr:first-child")).await;
    wait_for_selector(&h, &format!("{REQUESTER} [data-helpdesk-drawer-header]")).await;
    assert_eq!(snapshot(&h, REQUESTER).await["transition"], json!(false), "requester drawer has no status select");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn failed_write_reverts_and_reports() {
    let h = harness_at(&format!("{PAGE}?fault=fail-writes")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-transition]")).await;
    let before = snapshot(&h, SUPPORT).await;
    eval_json(&h, &format!("(() => {{ const s = document.querySelector('{SUPPORT} [data-helpdesk-transition]'); s.value = '10003'; s.dispatchEvent(new Event('change', {{ bubbles: true }})); return true; }})()")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-action-feedback-state=\"error\"]")).await;
    let after = snapshot(&h, SUPPORT).await;
    assert_eq!(after["done"], before["done"], "bucket counts unchanged after a refused write");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn comments_append_in_both_roles() {
    let h = harness_at(PAGE).await;
    for root in [SUPPORT, REQUESTER] {
        wait_for_selector(&h, &format!("{root} [data-helpdesk-state=\"ready\"]")).await;
        click(&h, &format!("{root} [data-helpdesk-table] tbody tr:first-child")).await;
        wait_for_selector(&h, &format!("{root} [data-helpdesk-comment-input]")).await;
        let before = snapshot(&h, root).await["comments"].as_u64().unwrap();
        eval_json(&h, &format!("(() => {{ const t = document.querySelector('{root} [data-helpdesk-comment-input]'); t.value = 'hello from {root}'; t.dispatchEvent(new Event('input', {{ bubbles: true }})); return true; }})()")).await;
        click(&h, &format!("{root} [data-helpdesk-comment-submit]")).await;
        wait_for_selector(&h, &format!("{root} [data-helpdesk-action-feedback-state=\"success\"]")).await;
        assert_eq!(snapshot(&h, root).await["comments"].as_u64().unwrap(), before + 1, "{root}");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn new_request_validates_pastes_and_files() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["submitDisabled"], json!(true), "blank summary disables submit: {s}");

    let paste = |root: &str, name: &str, bytes: &str, mime: &str| format!(r#"(() => {{
        const bytes = new Uint8Array([{bytes}]);
        const file = new File([bytes], '{name}', {{ type: '{mime}' }});
        const dt = new DataTransfer(); dt.items.add(file);
        const zone = document.querySelector('{root} [data-image-attachment-dropzone]');
        zone.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: dt, bubbles: true }}));
        return true;
    }})()"#);
    const PNG: &str = "0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A,0,0,0,0";
    const JPEG: &str = "0xFF,0xD8,0xFF,0xE0,0,0,0,0,0,0,0,0";
    eval_json(&h, &paste(SUPPORT, "shot.png", PNG, "image/png")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-image-attachment-item]")).await;
    assert_eq!(snapshot(&h, SUPPORT).await["images"], json!(1));
    eval_json(&h, &paste(SUPPORT, "notes.txt", "104,105", "image/png")).await; // declared image, bytes are text
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["images"], json!(1), "sniffing rejects non-image bytes: {s}");
    assert!(s["imageStatus"].as_str().unwrap().contains("notes.txt"), "{s}");
    eval_json(&h, &paste(SUPPORT, "photo.png", JPEG, "image/png")).await; // .png name, JPEG bytes
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(snapshot(&h, SUPPORT).await["images"], json!(2), "JPEG bytes under a .png name are admitted");

    eval_json(&h, &format!("(() => {{ const i = document.querySelector('{SUPPORT} [data-helpdesk-summary]'); i.value = 'Board is blank'; i.dispatchEvent(new Event('input', {{ bubbles: true }})); return true; }})()")).await;
    assert_eq!(snapshot(&h, SUPPORT).await["submitDisabled"], json!(false));
    click(&h, &format!("{SUPPORT} [data-helpdesk-submit]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-toast]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert_eq!(s["keys"][0], json!("OF-13"), "new ticket is first: {s}");
    assert_eq!(s["dialogOpen"], json!(false));
    let calls = eval_json(&h, "JSON.stringify(window.__ldui_debug ? window.__ldui_debug.helpdesk_calls() : [])").await;
    assert!(calls.as_str().unwrap().contains("images: 2"), "create carried both images: {calls}");
    assert_no_browser_errors(&h, "new request").await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn not_configured_shows_panel_and_no_submit() {
    let h = harness_at(&format!("{PAGE}?fault=not-configured")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-not-configured]")).await;
    let s = snapshot(&h, SUPPORT).await;
    assert!(s["notConfigured"].as_bool().unwrap() && !s["submitPresent"].as_bool().unwrap(), "{s}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-helpdesk)"]
async fn axe_clean_with_drawer_and_dialog_open() {
    let h = harness_at(PAGE).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-state=\"ready\"]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-table] tbody tr:first-child")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-drawer-header]")).await;
    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js").expect("load vendored axe-core");
    let report = axe.run(h.page()).await.expect("run axe-core");
    report.assert_no_blocking("helpdesk drawer open").unwrap_or_else(|e| panic!("{e}; {}\nviolations: {:#?}", report.summary(), report.violations));
    click(&h, &format!("{SUPPORT} [data-helpdesk-drawer-close]")).await;
    click(&h, &format!("{SUPPORT} [data-helpdesk-new-request]")).await;
    wait_for_selector(&h, &format!("{SUPPORT} [data-helpdesk-summary]")).await;
    let report = axe.run(h.page()).await.expect("run axe-core");
    report.assert_no_blocking("helpdesk dialog open").unwrap_or_else(|e| panic!("{e}; {}\nviolations: {:#?}", report.summary(), report.violations));
}
```

The `?fault=` query is appended by the test; `harness_at` adds `?pp-freeze=1` after it. If `ldui_web_config`'s `query_suffix` is applied as a plain `?` append, change the fixture to read the fault from the pathname instead (`/helpdesk-fixture-not-configured`, `/helpdesk-fixture-fail-writes`) and add those two suffix checks in the host; either way, the test constants change in one place.

- [ ] **Step 3: Run the lane** (background; first run builds the demo to wasm, ~8 minutes):

Run: `cargo xtask test-helpdesk > .review/test-helpdesk.log 2>&1; echo "lane-exit=$?" >> .review/test-helpdesk.log`
Expected in the log: `test result: ok. 9 passed` (the count proves registration). Fix whatever fails; do not touch the tree while the lane runs.

- [ ] **Step 4: Run the native gate**

Run: `cargo xtask verify`
Expected: all steps PASS (including `test-ld-class-coverage`, `test-daisyui5`, clippy per crate, fmt-check).

- [ ] **Step 5: Commit**

```bash
git add demo/src/client_snapshot_test_host.rs tests/helpdesk_smoke.rs xtask/src/main.rs CLAUDE.md
git commit -m "test(helpdesk): browser proof lane test-helpdesk with both roles and faults

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 12: Docs and full gate

**Files:**
- Create: `doc/components/helpdesk.md`
- Modify: `CLAUDE.md` (a "Recent additions (2026-09-14, Helpdesk composite)" paragraph after the SnapshotTablePage filter-actions section)

- [ ] **Step 1: Write `doc/components/helpdesk.md`** covering: purpose and roles; the `HelpdeskBackend` trait with the eight signatures; the `Helpdesk` props; the in-memory backend and its faults; the `ImageAttachmentField` contract (sniffing, caps, `capture_document_paste`); the `data-helpdesk-*` hook list; CSS: the consumer must scan this crate's `src/**/*.rs` and import `styles/tokens.css`; `@source inline("join join-item btn btn-active drawer drawer-end drawer-side drawer-overlay alert alert-success")`.

- [ ] **Step 2: CLAUDE.md paragraph** (five to eight lines): what landed, the trait-owned transport decision, the `SegmentedBar` correction (it is a proportion bar; the kind picker is radio-as-button), the fixture-by-pathname rule, and the `test-helpdesk` lane.

- [ ] **Step 3: Run the audits on the new page** (background):

Run: `cargo xtask test-layout` then `cargo xtask test-style`
Expected: `helpdesk_layout_is_clean` and `helpdesk_style_is_within_ceiling` pass at zero ceilings.

- [ ] **Step 4: Commit**

```bash
git add doc/components/helpdesk.md CLAUDE.md
git commit -m "docs(helpdesk): component guide and CLAUDE.md note

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

## Self-review

- Spec §3 model → Task 1. §4 trait + in-memory → Task 3. §5 composite → Tasks 7, 8, 9. §6 field + editor refactor → Tasks 4, 5. §7 texts → Task 6 (+`saved` added in Task 9). §8 demo, fixture, lane, audits → Tasks 10, 11; docs → Task 12.
- Spec deviations recorded: `SegmentedBar` replaced by radio-as-button (the component is a proportion bar); `RecordTone` is `RecordStatusTone`; fixture selected by pathname suffix; `current_user_id` and `now_ms` added as composite props for "mine only" and deterministic ages.
- Type consistency: `HelpdeskFuture`, `TicketKey`, `TriageAction`, `RoleCapabilities`, `ImageAttachmentCaps`, `admit`, `replace_by_key`, `BackendCall::Create { summary, images }` are used with the same shapes in every task.
- Known compile-risk points, each with the fallback stated inline: `Signal::derive_local`, `attr:on:click` on `DrawerOverlay`, `ButtonType::Submit` name, `DrawerPlacement::End` name, the selectable-summary `data-*` hook names, the debug bridge global name.
