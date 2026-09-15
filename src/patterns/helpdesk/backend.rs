//! The transport seam. The composite calls these; the host implements them
//! over its own HTTP client. Futures are `'static` and not `Send`, exactly
//! like `crate::markdown::asset_upload::UploadFuture`: wasm is single-threaded
//! and every call is awaited from `spawn_local`.

use super::model::{
    HelpdeskError, HelpdeskMeta, HelpdeskTicket, NewTicket, TicketComment, TicketDetail, TicketKey,
    TicketScope,
};
use std::future::Future;
use std::pin::Pin;

/// A boxed, local future resolving to the backend's typed result.
pub type HelpdeskFuture<T> = Pin<Box<dyn Future<Output = Result<T, HelpdeskError>> + 'static>>;

/// Everything the `Helpdesk` composite asks of its host. Every write returns
/// the updated ticket so the list can replace one row without a refetch.
pub trait HelpdeskBackend {
    /// Whether the helpdesk is set up, and the vocabularies it offers.
    fn meta(&self) -> HelpdeskFuture<HelpdeskMeta>;
    /// The caller's own tickets, or every ticket for support.
    fn list(&self, scope: TicketScope) -> HelpdeskFuture<Vec<HelpdeskTicket>>;
    /// One ticket's full detail, including comments and attachments.
    fn detail(&self, key: &TicketKey) -> HelpdeskFuture<TicketDetail>;
    /// File a new ticket.
    fn create(&self, ticket: NewTicket) -> HelpdeskFuture<HelpdeskTicket>;
    /// Move a ticket to a new status.
    fn transition(&self, key: &TicketKey, to_status_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    /// Set or clear a ticket's assignee.
    fn assign(&self, key: &TicketKey, assignee_id: Option<&str>) -> HelpdeskFuture<HelpdeskTicket>;
    /// Set a ticket's priority.
    fn set_priority(&self, key: &TicketKey, priority_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    /// Add a comment to a ticket.
    fn comment(&self, key: &TicketKey, body: &str) -> HelpdeskFuture<TicketComment>;
}
