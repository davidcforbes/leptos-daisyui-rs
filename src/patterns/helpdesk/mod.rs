//! Opinionated Jira-helpdesk composite driven by a host-implemented backend.
//!
//! See `doc/plans/2026-09-14-helpdesk-composite-design.md` for the design
//! and `doc/components/helpdesk.md` for the consumer guide.

mod model;
mod state;

pub use model::{
    HelpdeskError, HelpdeskErrorKind, HelpdeskMeta, HelpdeskRole, HelpdeskTicket, ImageAttachment,
    NewTicket, Person, PriorityOption, RequestContext, StatusCategory, TicketAttachment,
    TicketComment, TicketDetail, TicketKey, TicketKind, TicketPriority, TicketScope, TicketStatus,
};
pub use state::{
    Bucket, BucketCounts, NewTicketCaps, NewTicketError, RoleCapabilities, TicketFilter,
    bucket_counts, relative_age, validate_new_ticket,
};
