# Helpdesk

`Helpdesk` is the opinionated Jira-backed support composite: a bucket strip, a
filtered ticket table, a detail drawer with triage and comments, and a New
Request dialog that accepts pasted, dropped or picked screenshots. One
component renders both sides of the desk — the person filing a request and the
person working the queue — and the difference between them is a role, not a
second page.

It owns no transport. Everything it needs comes through a host-implemented
[`HelpdeskBackend`](#the-backend-trait), so the same composite backs a real
Jira deployment, an in-memory fixture, and a host that has never configured a
helpdesk at all.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{Helpdesk, HelpdeskBackend, HelpdeskRole, RequestContext};
use std::rc::Rc;

let backend: Rc<dyn HelpdeskBackend> = Rc::new(JiraHelpdesk::new(api));
view! {
    <Helpdesk
        backend=backend
        role=HelpdeskRole::Support
        context=RequestContext { route: "/queue".into(), build: "2026.9.1".into() }
        current_user_id=Some(me.id.clone())
    />
}
```

## Roles are a capability table, not a permission check

`RoleCapabilities::for_role` is the whole difference between the two modes, and
it is a pure function you can assert on:

| Capability | `Requester` | `Support` |
|---|---|---|
| `scope` | `TicketScope::Mine` | `TicketScope::All` |
| `assignee_filter` | no | yes |
| `mine_only_toggle` | no | yes |
| `triage` (drawer status/assignee/priority selects) | no | yes |
| `comment` | yes | yes |

A requester's list call asks the backend for `Mine`; the composite does not
fetch every ticket and hide the rest. **The role is a rendering contract, not a
security boundary** — the backend still owns authorization, because a browser
signal is not a permission.

## The backend trait

Eight calls, each returning a `HelpdeskFuture<T>` (a boxed local future, since
the composite is CSR and its data is not `Send`). Every write returns the
updated ticket so the list replaces one row without a refetch.

| Method | Purpose |
|---|---|
| `meta()` | Whether the helpdesk is configured, plus the status, priority, assignable and kind vocabularies. |
| `list(scope)` | The caller's own tickets, or every ticket. |
| `detail(key)` | One ticket with its description, comments and attachments. |
| `create(ticket)` | File a `NewTicket`, including its images. |
| `transition(key, to_status_id)` | Move a ticket to a new status. |
| `assign(key, assignee_id)` | Set or clear the assignee. |
| `set_priority(key, priority_id)` | Set the priority. |
| `comment(key, body)` | Append a comment. |

Failures are typed, not stringly: `HelpdeskError { kind, message }` with
`HelpdeskErrorKind::{NotConfigured, RateLimited { retry_after_s }, Unauthorized,
NotFound, Network, Upstream}`. The composite renders each kind differently, so
returning `Upstream` for a rate limit costs the user the retry hint. `message`
is rendered to the user — scrub it host-side; never pass an upstream Jira error
body through.

### Not configured is a first-class state

`meta()` returning `configured: false` is not an error path. The board renders
the host's `unavailable_reason`, and the New Request dialog renders a
`data-helpdesk-not-configured` panel **with no submit button at all** — the
control is absent, not disabled, so there is nothing to click and no request to
refuse. A host that has not wired Jira gets a coherent screen rather than a
spinner that never resolves.

## The in-memory backend (test-mode only)

`InMemoryHelpdeskBackend` lives behind the `test-mode` feature and exists so
proofs and the showcase can run without a Jira. `seeded()` builds twelve
tickets (`OF-1`..`OF-12`, six of them requested by `SEED_ME`) against the fixed
clock `SEED_NOW_MS`, so relative ages are deterministic.

- `with_fault(HelpdeskFault::NotConfigured | RateLimited(s) | FailWrites)`
  injects one failure mode. `FailWrites` keeps reads working, which is the
  shape that proves a refused write **reverts** rather than leaving the board
  showing a change the server rejected.
- `calls()` returns the recorded `BackendCall` log — `Create { summary, images }`
  carries the image count, which is how a proof shows the attachments actually
  reached the transport instead of only rendering as thumbnails.

Pair `now_ms` with `SEED_NOW_MS` in any test that reads an age string.

## Screenshots: `ImageAttachmentField`

The dialog composes `ImageAttachmentField`, which accepts a paste, a drop or a
picked file and admits it by **sniffing the bytes, never trusting the declared
type**. `notes.txt` renamed to `.png` is refused; real JPEG bytes under a `.png`
name are admitted as `image/jpeg`. Defaults are five images of five MB each
(`ImageAttachmentCaps`), and every refusal is announced in a live region and
reported through `on_reject` as a typed `ImageRejection`.

`capture_document_paste` extends the paste handler to the whole document, for
dialog use where the user pastes without first focusing the drop zone. A paste
aimed at the drop zone is taken by the zone's own handler and still bubbles to
the window, so the document listener skips any event whose target lies inside
the zone — **without that guard one pasted screenshot becomes two attachments on
the filed ticket.** It decides by where the event landed rather than only by
`defaultPrevented`, because an event dispatched with `cancelable: false`
silently ignores `preventDefault()`; that is exactly how a synthetic test paste
slipped past a flag-only guard. Neither check stops propagation, so a host's own
paste listeners still run.

## Copy and localization

All rendered text comes from `HelpdeskTexts`, supplied as a `Signal` so a
locale change re-renders without a remount. Nothing in the composite formats a
user-visible string from an enum's `Debug`.

## DOM hooks

Stable `data-*` attributes, for tests and for host CSS. **Query by these, never
by document position** — a positional selector does not fail when the layout
changes, it silently starts describing something else.

| Hook | Where |
|---|---|
| `data-helpdesk`, `data-helpdesk-role` | Root; the role is readable from the DOM. |
| `data-helpdesk-state` | `loading` \| `error` \| `empty` \| `ready`. |
| `data-helpdesk-buckets`, `data-helpdesk-bucket` | The bucket strip and each card. |
| `data-helpdesk-table` | The ticket table (rows are `tbody tr`). |
| `data-helpdesk-search`, `data-helpdesk-filter-priority`, `data-helpdesk-filter-assignee`, `data-helpdesk-mine-only`, `data-helpdesk-refresh` | Filter row. |
| `data-helpdesk-drawer`, `-drawer-open`, `-drawer-header`, `-drawer-close` | Detail drawer. |
| `data-helpdesk-triage`, `-transition`, `-assign`, `-priority` | Support-only triage controls. |
| `data-helpdesk-comments`, `-comment`, `-comment-input`, `-comment-submit` | Comment thread. |
| `data-helpdesk-action-feedback`, `-action-feedback-state` | `idle` \| `pending` \| `success` \| `error`. |
| `data-helpdesk-new-request`, `-dialog`, `-kind`, `-summary`, `-description`, `-context`, `-images`, `-submit`, `-submit-error`, `-cancel`, `-toast` | New Request dialog. |
| `data-helpdesk-not-configured` | The unconfigured panel. |
| `data-image-attachment-field`, `-dropzone`, `-input`, `-list`, `-item`, `-remove`, `-status` | Attachment field. |

## CSS

Class delivery is not automatic across a Rust path dependency. A consuming
app's `input.css` must scan this crate's source and import the generated tokens:

```css
@import "tailwindcss";
@import "../leptos-daisyui-rs/styles/tokens.css";
@plugin "daisyui";
@source "../src/**/*.rs";
@source "../leptos-daisyui-rs/src/**/*.rs";
@source inline("join join-item btn btn-active drawer drawer-end drawer-side drawer-overlay alert alert-success");
```

## Proof

`cargo xtask test-helpdesk` mounts both roles on one document over one shared
backend, so every positive assertion carries its negative control on the same
run: the support board shows an assignee filter *and* the requester board does
not; a support transition writes *and* the requester drawer has no status
select. It also covers the two faults by pathname
(`/helpdesk-fixture-not-configured`, `/helpdesk-fixture-fail-writes`) and runs
vendored axe-core with the drawer and the dialog open.
