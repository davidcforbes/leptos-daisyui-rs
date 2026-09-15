//! Opinionated Jira-helpdesk composite driven by a host-implemented backend.
//!
//! See `doc/plans/2026-09-14-helpdesk-composite-design.md` for the design
//! and `doc/components/helpdesk.md` for the consumer guide.

mod backend;
mod drawer;
#[cfg(feature = "test-mode")]
mod memory;
mod model;
mod request_dialog;
mod state;
mod texts;

pub use backend::{HelpdeskBackend, HelpdeskFuture};
pub use drawer::{TicketDetailDrawer, TriageAction};
#[cfg(feature = "test-mode")]
pub use memory::{BackendCall, HelpdeskFault, InMemoryHelpdeskBackend, SEED_ME, SEED_NOW_MS};
pub use model::{
    HelpdeskError, HelpdeskErrorKind, HelpdeskMeta, HelpdeskRole, HelpdeskTicket, ImageAttachment,
    NewTicket, Person, PriorityOption, RequestContext, StatusCategory, TicketAttachment,
    TicketComment, TicketDetail, TicketKey, TicketKind, TicketPriority, TicketScope, TicketStatus,
};
pub use request_dialog::NewRequestDialog;
pub use state::{
    Bucket, BucketCounts, NewTicketCaps, NewTicketError, RoleCapabilities, TicketFilter,
    bucket_counts, relative_age, validate_new_ticket,
};
pub use texts::HelpdeskTexts;
