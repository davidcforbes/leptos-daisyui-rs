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
    fn meta(&self) -> HelpdeskFuture<HelpdeskMeta>;
    fn list(&self, scope: TicketScope) -> HelpdeskFuture<Vec<HelpdeskTicket>>;
    fn detail(&self, key: &TicketKey) -> HelpdeskFuture<TicketDetail>;
    fn create(&self, ticket: NewTicket) -> HelpdeskFuture<HelpdeskTicket>;
    fn transition(&self, key: &TicketKey, to_status_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    fn assign(&self, key: &TicketKey, assignee_id: Option<&str>) -> HelpdeskFuture<HelpdeskTicket>;
    fn set_priority(&self, key: &TicketKey, priority_id: &str) -> HelpdeskFuture<HelpdeskTicket>;
    fn comment(&self, key: &TicketKey, body: &str) -> HelpdeskFuture<TicketComment>;
}
