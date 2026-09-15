# Helpdesk composite: design (2026-09-14)

Status: approved design, awaiting implementation plan.
Companion: `C:\dev\4iiz-Office\docs\superpowers\specs\2026-09-14-helpdesk-design.md`
(the consumer half: API proxy, ledger, surface, F2 launcher).

## 1. Purpose

An opinionated composite that lets an application's users see Jira support
tickets and file new ones (bug or request) with screenshots, without the
application re-deriving the list, drawer, form and attachment mechanics per
host. The first consumer is 4iiz-Office, whose Jira project is `OF`
(4iiz-Office). The composite is role-switched: a **Requester** sees and files
their own tickets; **Support** sees every ticket and triages it (status,
assignee, priority, comments).

The composite owns no transport. It drives a `HelpdeskBackend` trait that the
host implements (approach C, chosen 2026-09-14 over data-in/callbacks-out and
over a thin library). An in-memory implementation ships under the `test-mode`
feature so the demo page and the browser proof run without a network.

## 2. Module layout

```
src/patterns/helpdesk/
  mod.rs          public exports
  model.rs        HelpdeskTicket, TicketDetail, NewTicket, HelpdeskMeta, enums, HelpdeskError
  backend.rs      HelpdeskBackend trait + HelpdeskFuture alias
  state.rs        pure derivations: buckets, filter predicate, age formatting, validation
  texts.rs        HelpdeskTexts (all copy), Default = English
  component.rs    Helpdesk composite (buckets + filters + table + drawer + dialog)
  drawer.rs       TicketDetailDrawer
  request_dialog.rs  NewRequestDialog
  memory.rs       InMemoryHelpdeskBackend  (#[cfg(feature = "test-mode")])
src/components/image_attachment_field/
  mod.rs, component.rs, style.rs, tests.rs
src/utils/image_files.rs   read_image_files(&DataTransfer) + read_file_bytes, shared with markdown/editor.rs
```

`src/patterns/mod.rs` re-exports the public items the way it does for
`snapshot_table_page`. `src/components/mod.rs` exports `ImageAttachmentField`.

## 3. Model (`model.rs`)

All types are `Clone + Debug + PartialEq`, `serde` derive behind the crate's
existing serde feature so hosts can deserialise straight into them.

```rust
pub struct TicketKey(pub String);            // "OF-14"

pub enum TicketKind { Bug, Request, Other(String) }

pub enum StatusCategory { New, InProgress, Done }   // Jira statusCategory.key: new / indeterminate / done
pub struct TicketStatus { pub id: String, pub name: String, pub category: StatusCategory }

pub enum TicketPriority { Highest, High, Medium, Low, Lowest, Unset }
// with `id: Option<String>` carried separately in HelpdeskMeta.priorities so the host
// can map back to Jira priority ids; the enum is the display/sort key.

pub struct Person { pub id: String, pub display_name: String, pub initials: String }

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
    pub url: Option<String>,          // Jira browse link, opened in a new tab from the drawer
}

pub struct TicketComment { pub id: String, pub author: Person, pub body: String, pub created_at_ms: i64 }
pub struct TicketAttachment { pub id: String, pub filename: String, pub content_type: String,
                              pub url: String, pub thumbnail_url: Option<String> }
pub struct TicketDetail { pub ticket: HelpdeskTicket, pub description: String,
                          pub comments: Vec<TicketComment>, pub attachments: Vec<TicketAttachment> }

pub struct RequestContext { pub route: String, pub build: String }

pub struct ImageAttachment { pub filename: String, pub content_type: String, pub bytes: Vec<u8> }

pub struct NewTicket { pub kind: TicketKind, pub summary: String, pub description: String,
                       pub context: RequestContext, pub images: Vec<ImageAttachment> }

pub enum TicketScope { Mine, All }

pub struct PriorityOption { pub id: String, pub priority: TicketPriority, pub name: String }

pub struct HelpdeskMeta {
    pub configured: bool,
    pub unavailable_reason: Option<String>,     // host-supplied, already localised
    pub statuses: Vec<TicketStatus>,            // workflow order
    pub priorities: Vec<PriorityOption>,
    pub assignable: Vec<Person>,
    pub kinds: Vec<TicketKind>,                 // what the New Request dialog offers
}

pub enum HelpdeskErrorKind { NotConfigured, RateLimited { retry_after_s: u32 }, Unauthorized, NotFound, Network, Upstream }
pub struct HelpdeskError { pub kind: HelpdeskErrorKind, pub message: String }
```

`HelpdeskRole { Requester, Support }` lives in `model.rs` too.

## 4. Backend trait (`backend.rs`)

```rust
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

Futures are `'static` and not `Send`, matching `markdown::asset_upload::UploadFuture`
(`src/markdown/asset_upload.rs:20`); wasm is single-threaded and the composite
awaits them via `spawn_local`. The composite takes `backend: Rc<dyn HelpdeskBackend>`.
Every write returns the updated ticket so the list can replace the row by key
without a refetch.

`InMemoryHelpdeskBackend` (`memory.rs`, `test-mode` only) holds `RefCell` state
seeded by `InMemoryHelpdeskBackend::seeded()` (twelve tickets across all five
OF statuses, two requesters, two assignees) and exposes
`with_fault(HelpdeskFault)` where `HelpdeskFault::{NotConfigured, RateLimited(u32), FailWrites}`
so proofs cover error paths. It also records a `Vec<BackendCall>` log that the
proof reads back through `window.__ldui_helpdesk_calls` (the same
`debug::register_signal` mechanism `demo/src/client_snapshot_test_host.rs:35` uses).

## 5. Composite (`component.rs`)

```rust
#[component]
pub fn Helpdesk(
    backend: Rc<dyn HelpdeskBackend>,
    #[prop(into)] role: Signal<HelpdeskRole>,
    #[prop(into)] context: Signal<RequestContext>,
    #[prop(optional)] texts: HelpdeskTexts,
    /// Host-controlled: set true to open the New Request dialog (F2 launcher).
    #[prop(optional)] open_request: Option<RwSignal<bool>>,
    #[prop(optional, into)] class: &'static str,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
) -> impl IntoView
```

Root: `<div data-helpdesk data-helpdesk-role="requester|support" class=LIST_PAGE_BASE_CLASS>`.

Structure, top to bottom, reusing the existing patterns by name:

1. **Header**: `PageHeader` with `texts.title` and one quick action, the
   `New request` button (`data-helpdesk-new-request`).
2. **Buckets**: `SelectableSummaryGroup` (`src/patterns/selectable_summary.rs:806`)
   with three cards derived from `StatusCategory`: Open (New), In progress
   (InProgress), Done. A fourth "All" card is the group's cleared state.
   Counts come from `state::buckets(&tickets)`. Cards carry
   `data-helpdesk-bucket="open|in-progress|done"`. Jira's own five statuses
   are *not* buckets; they appear as the Status column and the drawer select.
3. **Filters**: `FilterBar` (`src/patterns/filter_bar.rs:245`) hosting a search
   `Input` (matches key and summary, case-insensitive), a priority `Select`,
   and, for Support only, an assignee `Select` fed from `meta.assignable` and a
   `Toggle` "Mine only" (`data-helpdesk-mine-only`). Requester role has scope
   fixed to `TicketScope::Mine` and renders neither. Reset clears all.
4. **Table**: `EntityTable<HelpdeskTicket>` (`src/components/entity_table/component.rs:746`)
   with `on_row_activate` opening the drawer. Columns, in order:
   Key; Summary (kind icon + text); Priority (`Badge`, tone by priority);
   Status (`Badge`, tone by category); Assignee (`AvatarBadge` initials, or
   an em dash); Age (`<time datetime=…>` with `state::relative_age(now, created_at_ms)`
   → "5m", "3h", "2d", "3w"). Default sort: updated descending. Empty, loading
   and error states render through `PageStatePanel` (`src/patterns/page_state_panel.rs:106`)
   inside `AsyncDataSection` so a refetch keeps the stale rows visible.
5. **Drawer** (`drawer.rs`): `Drawer` placement end, `open` when a key is
   selected, `data-helpdesk-drawer`. Contents: `RecordHeader` (key as eyebrow,
   summary as title, status as `RecordStatus`, kind badge, "Open in Jira" quick
   action when `url` is set); meta rows (requester, created, updated); the
   description; attachments as thumbnails linking to `url`
   (`data-helpdesk-attachment`); the comment thread (`data-helpdesk-comment`);
   an add-comment `Textarea` + button (`data-helpdesk-comment-submit`), both
   roles. Support only: three `Select`s in one row, status (`data-helpdesk-transition`,
   options = `meta.statuses`), assignee (`data-helpdesk-assign`), priority
   (`data-helpdesk-priority`). Each change fires the backend call immediately,
   applies the returned ticket optimistically-then-authoritatively, and
   reports through `ActionFeedback<TicketKey>` (`src/patterns/action_feedback.rs:137`);
   on error the select reverts and the feedback names the cause. The
   Requester drawer renders these fields as read-only meta rows, never as
   disabled selects.
6. **New Request dialog** (`request_dialog.rs`): `Modal` with `label`, opened
   by the header button or `open_request`. Fields: kind as a two-option
   `SegmentedBar` (Bug / Request, from `meta.kinds`; `data-helpdesk-kind`);
   summary `Input` (required, max 200, `data-helpdesk-summary`); description
   `Textarea` (max 4000; Bug placeholder is `texts.bug_placeholder` = "What I
   expected / What happened"); `ImageAttachmentField` (`data-helpdesk-images`);
   a read-only context line "Page {route} · Build {build}"; Cancel and Submit.
   Submit disabled until summary is non-blank. Validation errors render inline
   under the field with `aria-errormessage` through `Field`. On success:
   dialog closes, ticket is prepended to the list, a `Toast` shows
   `texts.filed` with the key linking to `url`. `NotConfigured` from `meta`
   replaces the form body with a `PageStatePanel` showing
   `meta.unavailable_reason` (`data-helpdesk-not-configured`) and hides
   Submit. `RateLimited` after submit shows `texts.rate_limited` with the
   retry seconds and keeps the draft.

**Role rules** (all in one `state::RoleCapabilities::for_role()` so they are
testable natively): scope (Mine / All), show assignee filter, show mine-only,
show triage selects, may comment (both).

**Refetch policy**: `meta` once on mount; `list` on mount, after every
successful create, and on an explicit Refresh quick action. Writes never
trigger a full refetch.

## 6. `ImageAttachmentField` (`src/components/image_attachment_field/`)

```rust
#[component]
pub fn ImageAttachmentField(
    #[prop(into)] images: RwSignal<Vec<ImageAttachment>>,
    #[prop(optional)] texts: ImageAttachmentTexts,
    #[prop(default = 5)] max_count: usize,
    #[prop(default = 5 * 1024 * 1024)] max_bytes: usize,
    #[prop(default = &["image/png", "image/jpeg", "image/webp"])] accept: &'static [&'static str],
    /// Also accept Ctrl+V anywhere in the document while mounted (dialog use).
    #[prop(default = false)] capture_document_paste: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_reject: Option<Callback<ImageRejection>>,
    #[prop(optional, into)] class: &'static str,
) -> impl IntoView
```

Rendering: a drop zone that is a real `<button type="button">`
(`data-image-attachment-dropzone`), keyboard-operable, opening a hidden
`<input type="file" multiple accept=…>`; drag-enter/leave counter for the cue
class (the same counter idiom as `src/markdown/editor.rs:336-352`); an
`on:paste` on the zone plus, when `capture_document_paste`, a document-level
listener registered with `window_event_listener` and removed in `on_cleanup`;
a preview grid of `<li data-image-attachment-item>` with an object URL
`<img>` (URLs revoked when the item is removed and in `on_cleanup`), filename,
size, and a Remove button. Rejections (wrong type, too big, over count) are
announced in an `aria-live="polite"` region and passed to `on_reject`.
Type is checked by magic bytes (`image_files::sniff_content_type`), not by the
file's declared MIME, because pasted clipboard images arrive as
`image/png` regardless of source and dropped files can lie.

`src/utils/image_files.rs` exports `read_image_files(&DataTransfer) -> Vec<File>`
and `read_file_bytes(&File) -> impl Future<Output = Result<Vec<u8>, String>>`
(moved from `src/markdown/editor.rs`), and `sniff_content_type(&[u8]) -> Option<&'static str>`.
`MarkdownEditor`'s paste and drop handlers (`editor.rs:312-364`) are rewritten
over `read_image_files` so the two paths share one definition of "an image in
a DataTransfer"; its behaviour (first image only, upload and insert at cursor)
is unchanged.

CSS: the zone and previews use existing daisyUI classes (`border-dashed`,
`rounded-box`, `bg-base-200`) plus a `data-dragging` attribute for the cue;
any new `ld-*` class is added to the runtime preamble and to
`tests/ld_class_stylesheet_coverage.rs` expectations. Spacing stays on the
canonical scale (gap-2, p-4).

## 7. Texts (`texts.rs`)

`HelpdeskTexts` and `ImageAttachmentTexts` are plain structs with `Default`
English copy, built by struct literal like `EntityTableTexts` (the per-type
compatibility memory: Office builds texts structs literally, so every field
is `pub` and the struct is `#[non_exhaustive]`-free). Includes: title,
new_request, buckets (open, in_progress, done, all), filters (search
placeholder, priority, assignee, mine_only, reset), columns, drawer labels,
kind names, placeholders, filed, rate_limited (with `{seconds}`),
not_configured_title, and the attachment field's copy (drop_hint, browse,
remove, rejected_type, rejected_size, rejected_count).

## 8. Demo and proof

- **Demo page** `demo/src/demos/helpdesk.rs`, route `/components/helpdesk`,
  mounting two `Helpdesk` instances (Requester, Support) over one seeded
  `InMemoryHelpdeskBackend`, with a role switch. Added to `PAGES` in
  `tests/layout_audit_smoke.rs:91` and `tests/style_audit_smoke.rs:177` **and**
  given its own `audit_test!` invocations, since a `PAGES` entry alone sweeps
  nothing.
- **Fixture** in `demo/src/client_snapshot_test_host.rs`: a `helpdesk` query
  flag selecting a fixture that mounts both roles and reads the fault from a
  second query parameter. `demo/src/demos/helpdesk.rs` is added to
  `CLIENT_SNAPSHOT_SOURCE_INPUTS` in `xtask/src/main.rs:787` so the browser
  build fingerprint sees it.
- **Browser lane** `tests/helpdesk_smoke.rs`, registered as
  `fn helpdesk_step() -> Step { name: "test-helpdesk", run: Run::BrowserSuite { test: "helpdesk_smoke", html_target: Some("client-snapshot-test-host.html") } }`
  next to `snapshot_table_page_filter_actions_step` (`xtask/src/main.rs:402`),
  dispatched from the CLI match and listed in the usage string. Checks, each
  with its negative control on the same document:
  1. bucket counts equal the seed's category counts; selecting Open filters the
     rows to New-category statuses; All restores.
  2. Support root has the assignee filter and mine-only toggle; Requester root
     has neither; Requester rows are only the seed's requester-A tickets.
  3. activating a row opens the drawer with that key in the RecordHeader;
     Escape closes it and focus returns to the row.
  4. Support status select → backend call log records `transition(key, id)`,
     the row's Status badge updates, feedback state is `success`; with
     `FailWrites` the select reverts and feedback state is `error`. Requester
     drawer has no `[data-helpdesk-transition]`.
  5. add comment appends a `[data-helpdesk-comment]` in both roles.
  6. New Request: Submit disabled with blank summary; a synthetic
     `ClipboardEvent` carrying a 1x1 PNG `File` in `clipboardData.files`
     produces one `[data-image-attachment-item]`; a second paste of a text/plain
     item produces none; a `File` with a `.png` name but JPEG magic bytes is
     accepted as image/jpeg; a 6 MB file is rejected with the size message;
     Submit prepends `OF-…` as the first row and shows the toast.
  7. with `NotConfigured`, the dialog shows `[data-helpdesk-not-configured]`
     and no Submit; the header button is still present.
  8. vendored axe: zero blocking findings on the page with the drawer open and
     with the dialog open.
  Verified registered by the suite's test count appearing in the lane output.
- **Native tests** (`state.rs`, `memory.rs`, `image_attachment_field/tests.rs`,
  `utils/image_files.rs`): buckets, filter predicate, relative_age boundaries,
  `NewTicket::validate` (summary blank/too long, description too long, image
  caps), `RoleCapabilities`, in-memory backend semantics and faults,
  `sniff_content_type` on PNG/JPEG/WebP/GIF/junk.

## 9. Out of scope

Kanban layout, editing summary/description after filing, attachment upload
from the drawer, Jira-side watchers, and any transport. Screen capture via
`getDisplayMedia` was considered and declined (2026-09-14).
