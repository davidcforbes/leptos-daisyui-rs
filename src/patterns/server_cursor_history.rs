//! Transport-neutral cursor-history reducer for next-cursor-only server APIs.
//!
//! [`ServerCursorPagination`](crate::components::ServerCursorPagination) lets
//! a table display an opaque previous/next cursor pair, but it does not mint
//! one: a caller whose backend returns only a `next` cursor has historically
//! hand-rolled a page-local stack to derive a truthful `Previous`, reject
//! stale completions, and reset that stack on every search/sort/filter
//! change. [`ServerCursorHistory`] is that state machine, extracted once so
//! every consumer stops re-deriving it.
//!
//! The reducer never fetches, never parses a cursor's contents, never owns
//! rows, and never invents a total. It tracks exactly enough opaque history
//! to answer three questions a next-cursor-only API cannot answer on its
//! own: what request reaches the previous slice, what request reaches the
//! next slice, and whether a completion is still the one the caller is
//! waiting for. An API that already returns both cursors can skip this
//! reducer entirely and drive
//! [`ServerCursorPagination::controlled`](crate::components::ServerCursorPagination::controlled)
//! directly.
//!
//! The commit protocol reuses the idiom from
//! [`SnapshotTableState`](super::SnapshotTableState) and
//! [`ViewportFitEpoch`](crate::components::ViewportFitEpoch): a proposal
//! mints an unmintable-elsewhere handle that supersedes any earlier active
//! one, and only a [`ServerCursorHistory::complete`]/[`ServerCursorHistory::fail`]
//! call carrying the still-active handle can change state. A stale or
//! duplicate completion is therefore structurally unable to move
//! navigation.
//!
//! ## Composing with `ServerDataTable`'s cursor mode
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::components::*;
//! use leptos_daisyui_rs::patterns::ServerCursorHistory;
//!
//! /// Stands in for the caller-owned transport call: sends the derived
//! /// request to a real next-cursor-only API and returns its rows plus its
//! /// own next cursor. The reducer never does this itself.
//! fn fetch_next_cursor_page(_request: &ServerCursorRequest) -> Result<(Vec<TableRow>, Option<ServerCursorToken>), ()> {
//!     Ok((Vec::new(), None))
//! }
//!
//! #[component]
//! fn NextCursorOnlyTable() -> impl IntoView {
//!     let history = RwSignal::new(ServerCursorHistory::new(20));
//!     let rows = RwSignal::new(Vec::<TableRow>::new());
//!     let loading = RwSignal::new(false);
//!     let columns = vec![Column::new("name", "Name")];
//!
//!     let on_change = Callback::new(move |query: ServerCursorQuery| {
//!         let mut minted = None;
//!         history.update(|state| minted = Some(state.propose_query(query)));
//!         let Some(Ok(handle)) = minted else {
//!             return;
//!         };
//!         loading.set(true);
//!         // A real consumer performs this over `spawn_local`; the reducer
//!         // is transport-neutral, so any owned fetch works the same way.
//!         match fetch_next_cursor_page(handle.request()) {
//!             Ok((page_rows, next)) => {
//!                 rows.set(page_rows);
//!                 history.update(|state| {
//!                     state.complete(handle, next);
//!                 });
//!             }
//!             Err(()) => {
//!                 history.update(|state| {
//!                     state.fail(handle);
//!                 });
//!             }
//!         }
//!         loading.set(false);
//!     });
//!
//!     let current = Signal::derive(move || history.with(ServerCursorHistory::current_query));
//!     let page = Signal::derive(move || history.with(ServerCursorHistory::current_page));
//!
//!     view! {
//!         <ServerDataTable
//!             rows=rows
//!             columns=Signal::derive(move || columns.clone())
//!             loading=loading
//!             pagination=ServerTablePagination::cursor(ServerCursorPagination::controlled(
//!                 current,
//!                 page,
//!                 on_change,
//!             ))
//!         />
//!     }
//! }
//! ```
//!
//! `history.current_query()`/`current_page()` derive exactly the two values
//! [`ServerCursorPagination::controlled`](crate::components::ServerCursorPagination::controlled)
//! needs; `on_change` translates its `ServerCursorQuery` proposal into a
//! handle and hands it to the caller-owned fetch, which commits or fails the
//! same handle when it settles.

use crate::components::{
    ColumnFilters, ServerCursorPage, ServerCursorQuery, ServerCursorRequest,
    ServerCursorSliceState, ServerCursorToken, SortOrder,
};

/// Opaque generation bumped by every query-shape or dataset/access reset.
///
/// A generation change invalidates every handle minted before it: the
/// reset immediately mints its own first-slice handle in the new
/// generation, so an in-flight completion from the prior generation always
/// finds a mismatched active handle and is rejected as `IgnoredStale`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServerCursorHistoryGeneration(u64);

impl ServerCursorHistoryGeneration {
    /// Stable diagnostic marker suitable for `data-*` attributes.
    pub fn marker(self) -> String {
        self.0.to_string()
    }
}

/// Failure to mint a navigation handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerCursorHistoryError {
    /// The accepted slice is already the first one; there is no history
    /// entry to step back into.
    PreviousUnavailable,
    /// Neither the accepted slice nor any prior visit recorded a next
    /// cursor, so there is nothing truthful to request.
    NextUnavailable,
    /// The checked proposal sequence cannot advance without wrapping.
    SequenceExhausted,
}

/// Whether a completion/failure changed the reducer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerCursorHistoryDisposition {
    /// The active handle was consumed and state changed.
    Applied,
    /// A superseded handle was supplied.
    IgnoredStale,
    /// The handle was already consumed by a prior completion or failure, or
    /// no proposal is active at all.
    IgnoredConsumed,
}

/// Opaque request handle minted only by [`ServerCursorHistory::propose_query`]
/// or [`ServerCursorHistory::propose_navigation`].
///
/// [`Self::request`] is the exact [`ServerCursorRequest`] transport-owned
/// code must send. For a `Previous` proposal this is never a literal
/// `Previous` cursor -- a next-cursor-only API has no such thing -- it is
/// whichever `First`/`Next` request originally reached that earlier slice,
/// replayed from history.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerCursorHistoryHandle {
    generation: ServerCursorHistoryGeneration,
    sequence: u64,
    target_index: usize,
    request: ServerCursorRequest,
}

impl ServerCursorHistoryHandle {
    /// Dataset/access generation this handle was minted in.
    pub const fn generation(&self) -> ServerCursorHistoryGeneration {
        self.generation
    }

    /// Monotonic proposal sequence used for diagnostics.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// The exact request transport-owned code must send.
    pub const fn request(&self) -> &ServerCursorRequest {
        &self.request
    }
}

/// Pure reducer translating typed cursor-navigation intent into opaque
/// request handles for a next-cursor-only server API.
///
/// The reducer owns the query shape (search/sort/filters/page size) so it
/// can derive a complete [`ServerCursorQuery`]/[`ServerCursorPage`] pair for
/// [`ServerCursorPagination`](crate::components::ServerCursorPagination),
/// and owns just enough opaque cursor history -- one token per confirmed
/// forward transition, never a page of rows -- to answer `Previous`
/// truthfully.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerCursorHistory {
    generation: ServerCursorHistoryGeneration,
    next_sequence: u64,
    /// `forward_tokens[i]` is the request token that carries position `i`
    /// to position `i + 1`. Length equals the deepest position ever
    /// confirmed reachable.
    forward_tokens: Vec<ServerCursorToken>,
    /// Index of the currently accepted (or retained) slice.
    accepted_index: usize,
    /// Next cursor observed for the deepest confirmed position, when that
    /// position has one. `None` there means the API reported the true end.
    frontier_next: Option<ServerCursorToken>,
    slice_state: ServerCursorSliceState,
    active: Option<ServerCursorHistoryHandle>,
    search: String,
    sort: Option<(&'static str, SortOrder)>,
    filters: ColumnFilters,
    page_size: i64,
}

impl ServerCursorHistory {
    /// Creates a reducer at the server-defined first slice with no history.
    pub fn new(page_size: i64) -> Self {
        Self {
            generation: ServerCursorHistoryGeneration::default(),
            next_sequence: 0,
            forward_tokens: Vec::new(),
            accepted_index: 0,
            frontier_next: None,
            slice_state: ServerCursorSliceState::Current,
            active: None,
            search: String::new(),
            sort: None,
            filters: ColumnFilters::new(),
            page_size: page_size.max(1),
        }
    }

    /// Current opaque dataset/access/query-shape generation.
    pub const fn generation(&self) -> ServerCursorHistoryGeneration {
        self.generation
    }

    /// Derives the complete query the accepted slice represents, suitable
    /// for [`ServerCursorPagination::controlled`](crate::components::ServerCursorPagination::controlled)'s
    /// `current` signal.
    pub fn current_query(&self) -> ServerCursorQuery {
        ServerCursorQuery {
            request: self.request_to_reach(self.accepted_index),
            page_size: self.page_size,
            search: self.search.clone(),
            sort: self.sort,
            filters: self.filters.clone(),
        }
    }

    /// Derives navigation metadata for the accepted slice, suitable for
    /// [`ServerCursorPagination::controlled`](crate::components::ServerCursorPagination::controlled)'s
    /// `page` signal. `previous` is a self-minted opaque token when the
    /// accepted slice is not the first (its content is never sent to a
    /// transport as-is; a round trip through [`Self::propose_query`] or
    /// [`Self::propose_navigation`] re-derives the real request from
    /// history). `next` is the transport's own opaque cursor, when known.
    pub fn current_page(&self) -> ServerCursorPage {
        let previous = (self.accepted_index > 0).then(|| self.mint_previous_token());
        let next = self.next_at(self.accepted_index);
        let page = ServerCursorPage::new(previous, next);
        match self.slice_state {
            ServerCursorSliceState::Current => page,
            ServerCursorSliceState::RetainedWhileLoading => page.retained_while_loading(),
            ServerCursorSliceState::RetainedAfterFailure => page.retained_after_failure(),
        }
    }

    /// Translates one full [`ServerCursorQuery`] proposal -- exactly the
    /// shape [`ServerCursorPagination::controlled`](crate::components::ServerCursorPagination::controlled)'s
    /// `on_change` callback receives -- into an opaque request handle. A
    /// change to search, sort, filters, or page size starts a new
    /// first-slice generation regardless of the carried `request`; an
    /// unchanged shape is forwarded to [`Self::propose_navigation`].
    pub fn propose_query(
        &mut self,
        query: ServerCursorQuery,
    ) -> Result<ServerCursorHistoryHandle, ServerCursorHistoryError> {
        let shape_changed = query.search != self.search
            || query.sort != self.sort
            || query.filters != self.filters
            || query.page_size != self.page_size;
        if shape_changed {
            self.start_new_generation(query.search, query.sort, query.filters, query.page_size)
        } else {
            self.propose_navigation(query.request)
        }
    }

    /// Translates one typed navigation intent (`First`/`Previous`/`Next`)
    /// into an opaque request handle without touching the query shape. Any
    /// opaque token carried on the incoming `Previous`/`Next` variant is
    /// ignored -- the real request is always derived from history, never
    /// trusted from the caller's round trip.
    pub fn propose_navigation(
        &mut self,
        request: ServerCursorRequest,
    ) -> Result<ServerCursorHistoryHandle, ServerCursorHistoryError> {
        let (target_index, transport_request) = match request {
            ServerCursorRequest::First => (0, ServerCursorRequest::First),
            ServerCursorRequest::Previous(_) => {
                if self.accepted_index == 0 {
                    return Err(ServerCursorHistoryError::PreviousUnavailable);
                }
                let target = self.accepted_index - 1;
                (target, self.request_to_reach(target))
            }
            ServerCursorRequest::Next(_) => {
                let token = self
                    .next_at(self.accepted_index)
                    .ok_or(ServerCursorHistoryError::NextUnavailable)?;
                (self.accepted_index + 1, ServerCursorRequest::Next(token))
            }
        };
        self.mint(target_index, transport_request)
    }

    /// Starts a brand-new first-slice generation for a dataset/access
    /// change unrelated to [`ServerCursorQuery`]'s own fields (a
    /// `query_reset_key`-style identity change, an access replacement).
    /// Clears search/sort/filters and adopts `page_size`.
    pub fn reset(
        &mut self,
        page_size: i64,
    ) -> Result<ServerCursorHistoryHandle, ServerCursorHistoryError> {
        self.start_new_generation(String::new(), None, ColumnFilters::new(), page_size)
    }

    /// Applies a completed fetch only for the still-active matching handle.
    /// `next` is the opaque next cursor the transport reported for the
    /// newly accepted slice, or `None` at the true end. Only the latest
    /// matching success may commit; every other completion is rejected
    /// before touching any field.
    pub fn complete(
        &mut self,
        handle: ServerCursorHistoryHandle,
        next: Option<ServerCursorToken>,
    ) -> ServerCursorHistoryDisposition {
        if let Err(disposition) = self.matches_active(&handle) {
            return disposition;
        }
        self.active = None;
        let target_index = handle.target_index;
        if target_index > self.forward_tokens.len() {
            let ServerCursorRequest::Next(token) = &handle.request else {
                unreachable!("a frontier extension is only ever reached through a Next request")
            };
            self.forward_tokens.push(token.clone());
        }
        if target_index == self.forward_tokens.len() {
            self.frontier_next = next;
        }
        self.accepted_index = target_index;
        self.slice_state = ServerCursorSliceState::Current;
        ServerCursorHistoryDisposition::Applied
    }

    /// Applies a failure only for the still-active matching handle. The
    /// accepted slice and every recorded history entry are left untouched;
    /// only the retained-status caption changes.
    pub fn fail(&mut self, handle: ServerCursorHistoryHandle) -> ServerCursorHistoryDisposition {
        if let Err(disposition) = self.matches_active(&handle) {
            return disposition;
        }
        self.active = None;
        self.slice_state = ServerCursorSliceState::RetainedAfterFailure;
        ServerCursorHistoryDisposition::Applied
    }

    fn start_new_generation(
        &mut self,
        search: String,
        sort: Option<(&'static str, SortOrder)>,
        filters: ColumnFilters,
        page_size: i64,
    ) -> Result<ServerCursorHistoryHandle, ServerCursorHistoryError> {
        self.generation.0 = self.generation.0.wrapping_add(1);
        self.forward_tokens.clear();
        self.accepted_index = 0;
        self.frontier_next = None;
        self.active = None;
        self.search = search;
        self.sort = sort;
        self.filters = filters;
        self.page_size = page_size.max(1);
        self.mint(0, ServerCursorRequest::First)
    }

    fn mint(
        &mut self,
        target_index: usize,
        request: ServerCursorRequest,
    ) -> Result<ServerCursorHistoryHandle, ServerCursorHistoryError> {
        let sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(ServerCursorHistoryError::SequenceExhausted)?;
        self.next_sequence = sequence;
        let handle = ServerCursorHistoryHandle {
            generation: self.generation,
            sequence,
            target_index,
            request,
        };
        self.active = Some(handle.clone());
        self.slice_state = ServerCursorSliceState::RetainedWhileLoading;
        Ok(handle)
    }

    fn matches_active(
        &self,
        handle: &ServerCursorHistoryHandle,
    ) -> Result<(), ServerCursorHistoryDisposition> {
        let Some(active) = &self.active else {
            return Err(ServerCursorHistoryDisposition::IgnoredConsumed);
        };
        if active.generation != handle.generation || active.sequence != handle.sequence {
            return Err(ServerCursorHistoryDisposition::IgnoredStale);
        }
        Ok(())
    }

    /// The request that, replayed, reaches `position` -- `First` for the
    /// first slice, otherwise a replay of the recorded forward token.
    fn request_to_reach(&self, position: usize) -> ServerCursorRequest {
        if position == 0 {
            ServerCursorRequest::First
        } else {
            ServerCursorRequest::Next(self.forward_tokens[position - 1].clone())
        }
    }

    /// The next cursor known for `position`, whether from a recorded
    /// forward transition or, at the frontier, the most recent completion.
    fn next_at(&self, position: usize) -> Option<ServerCursorToken> {
        if position < self.forward_tokens.len() {
            Some(self.forward_tokens[position].clone())
        } else {
            self.frontier_next.clone()
        }
    }

    /// Mints a self-referential opaque token identifying "one step back
    /// from the accepted slice". Its content is never sent to a real
    /// transport; a round trip through [`Self::propose_navigation`] ignores
    /// it and re-derives the actual request from history.
    fn mint_previous_token(&self) -> ServerCursorToken {
        ServerCursorToken::new(format!(
            "ldui-cursor-history:{}:{}",
            self.generation.0, self.accepted_index
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> ServerCursorHistory {
        ServerCursorHistory::new(10)
    }

    #[test]
    fn first_ever_load_captures_the_frontier_next_cursor() {
        let mut state = history();
        assert_eq!(state.current_query().request, ServerCursorRequest::First);
        assert_eq!(state.current_page().previous, None);
        assert_eq!(state.current_page().next, None);

        let handle = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        assert_eq!(
            state.current_page().state,
            ServerCursorSliceState::RetainedWhileLoading
        );
        assert_eq!(
            state.complete(handle, Some(ServerCursorToken::new("cursor-a"))),
            ServerCursorHistoryDisposition::Applied
        );
        assert_eq!(state.current_page().state, ServerCursorSliceState::Current);
        assert_eq!(
            state.current_page().next,
            Some(ServerCursorToken::new("cursor-a"))
        );
        assert_eq!(state.current_page().previous, None);
    }

    #[test]
    fn forward_navigation_derives_a_next_request_from_the_known_frontier_cursor() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));

        let handle = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::new(
                "stale-caller-supplied-ignored",
            )))
            .unwrap();
        assert_eq!(
            handle.request(),
            &ServerCursorRequest::Next(ServerCursorToken::new("cursor-a")),
            "the caller's echoed token is ignored; the real frontier cursor is used"
        );

        assert_eq!(
            state.complete(handle, Some(ServerCursorToken::new("cursor-b"))),
            ServerCursorHistoryDisposition::Applied
        );
        assert_eq!(
            state.current_query().request,
            ServerCursorRequest::Next(ServerCursorToken::new("cursor-a"))
        );
        assert_eq!(
            state.current_page().next,
            Some(ServerCursorToken::new("cursor-b"))
        );
        assert!(state.current_page().previous.is_some());
    }

    #[test]
    fn backward_navigation_replays_the_original_request_without_a_page_local_stack() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));
        let next1 = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        state.complete(next1, Some(ServerCursorToken::new("cursor-b")));
        let next2 = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        state.complete(next2, Some(ServerCursorToken::new("cursor-c")));

        // Now at slice 2 (reached via First -> Next(cursor-a) -> Next(cursor-b)).
        let previous_token = state.current_page().previous.expect("not the first slice");
        let back_one = state
            .propose_navigation(ServerCursorRequest::Previous(previous_token))
            .unwrap();
        assert_eq!(
            back_one.request(),
            &ServerCursorRequest::Next(ServerCursorToken::new("cursor-a")),
            "Previous replays the request that originally reached slice 1"
        );
        state.complete(back_one, Some(ServerCursorToken::new("cursor-b-again")));
        assert_eq!(
            state.current_query().request,
            ServerCursorRequest::Next(ServerCursorToken::new("cursor-a"))
        );

        let previous_token = state.current_page().previous.expect("not the first slice");
        let back_two = state
            .propose_navigation(ServerCursorRequest::Previous(previous_token))
            .unwrap();
        assert_eq!(back_two.request(), &ServerCursorRequest::First);
        state.complete(back_two, Some(ServerCursorToken::new("cursor-a-again")));
        assert_eq!(state.current_query().request, ServerCursorRequest::First);
        assert_eq!(state.current_page().previous, None);

        assert_eq!(
            state.propose_navigation(ServerCursorRequest::Previous(ServerCursorToken::default())),
            Err(ServerCursorHistoryError::PreviousUnavailable)
        );
    }

    #[test]
    fn redoing_forward_navigation_reuses_the_recorded_token_and_leaves_the_frontier_intact() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));
        let next1 = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        state.complete(next1, Some(ServerCursorToken::new("cursor-b")));

        // Back to slice 0, then forward again: the redo must reuse cursor-a
        // rather than requiring a fresh frontier value.
        let previous_token = state.current_page().previous.unwrap();
        let back = state
            .propose_navigation(ServerCursorRequest::Previous(previous_token))
            .unwrap();
        state.complete(back, Some(ServerCursorToken::new("cursor-a-again")));

        let redo = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        assert_eq!(
            redo.request(),
            &ServerCursorRequest::Next(ServerCursorToken::new("cursor-a"))
        );
        assert_eq!(
            state.complete(redo, Some(ServerCursorToken::new("cursor-b-redo"))),
            ServerCursorHistoryDisposition::Applied
        );

        // A brand-new forward move from here must still reach real new
        // territory using the never-lost frontier cursor.
        let next2 = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        assert_eq!(
            next2.request(),
            &ServerCursorRequest::Next(ServerCursorToken::new("cursor-b-redo"))
        );
    }

    #[test]
    fn next_is_unavailable_at_the_true_end() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, None);
        assert_eq!(state.current_page().next, None);
        assert_eq!(
            state.propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default())),
            Err(ServerCursorHistoryError::NextUnavailable)
        );
    }

    #[test]
    fn failure_retains_the_accepted_slice_and_every_recorded_history_entry() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));
        let before_query = state.current_query();
        let before_page = state.current_page();

        let failing_next = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        assert_eq!(
            state.fail(failing_next),
            ServerCursorHistoryDisposition::Applied
        );

        assert_eq!(state.current_query(), before_query);
        assert_eq!(state.current_page().previous, before_page.previous);
        assert_eq!(state.current_page().next, before_page.next);
        assert_eq!(
            state.current_page().state,
            ServerCursorSliceState::RetainedAfterFailure
        );

        // The accepted slice remains navigable after a failure.
        let retry = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        assert_eq!(
            state.complete(retry, Some(ServerCursorToken::new("cursor-b"))),
            ServerCursorHistoryDisposition::Applied
        );
    }

    #[test]
    fn stale_and_duplicate_completions_cannot_move_navigation() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));

        let stale = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        // A second, later proposal supersedes the first before it resolves
        // (a debounced re-click, or a fast double navigation).
        let latest = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        assert!(latest.sequence() > stale.sequence());

        // The stale handle's eventual completion is out-of-order and must
        // not move the accepted slice.
        assert_eq!(
            state.complete(stale, Some(ServerCursorToken::new("wrong"))),
            ServerCursorHistoryDisposition::IgnoredStale
        );

        assert_eq!(
            state.complete(latest.clone(), Some(ServerCursorToken::new("cursor-b"))),
            ServerCursorHistoryDisposition::Applied
        );
        assert_eq!(
            state.current_query().request,
            ServerCursorRequest::Next(ServerCursorToken::new("cursor-a"))
        );

        // A duplicate completion of the now-consumed handle is a no-op.
        assert_eq!(
            state.complete(latest, Some(ServerCursorToken::new("cursor-c"))),
            ServerCursorHistoryDisposition::IgnoredConsumed
        );
        assert_eq!(
            state.current_query().request,
            ServerCursorRequest::Next(ServerCursorToken::new("cursor-a"))
        );
    }

    #[test]
    fn query_shape_change_starts_a_coherent_first_slice_generation() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));
        let next = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();
        state.complete(next, Some(ServerCursorToken::new("cursor-b")));
        let before = state.generation();

        let mut reshaped_query = state.current_query();
        reshaped_query.search = "alice".to_owned();
        let handle = state.propose_query(reshaped_query).unwrap();

        assert!(state.generation() != before);
        assert_eq!(handle.request(), &ServerCursorRequest::First);
        assert_eq!(
            state.complete(handle, Some(ServerCursorToken::new("cursor-x"))),
            ServerCursorHistoryDisposition::Applied
        );
        assert_eq!(state.current_query().search, "alice");
        assert_eq!(state.current_query().request, ServerCursorRequest::First);
        assert_eq!(state.current_page().previous, None);

        // The old generation's history is gone: Previous is unavailable and
        // a stale forward token from before the reset cannot be replayed.
        assert_eq!(
            state.propose_navigation(ServerCursorRequest::Previous(ServerCursorToken::default())),
            Err(ServerCursorHistoryError::PreviousUnavailable)
        );
    }

    #[test]
    fn dataset_reset_clears_search_sort_and_filters() {
        let mut state = history();
        let mut query = state.current_query();
        query.search = "alice".to_owned();
        let shaped = state.propose_query(query).unwrap();
        state.complete(shaped, Some(ServerCursorToken::new("cursor-a")));
        assert_eq!(state.current_query().search, "alice");

        let handle = state.reset(25).unwrap();
        assert_eq!(handle.request(), &ServerCursorRequest::First);
        state.complete(handle, None);
        assert_eq!(state.current_query().search, "");
        assert_eq!(state.current_query().page_size, 25);
        assert_eq!(state.current_query().request, ServerCursorRequest::First);
    }

    #[test]
    fn a_generation_reset_supersedes_any_in_flight_proposal() {
        let mut state = history();
        let first = state
            .propose_navigation(ServerCursorRequest::First)
            .unwrap();
        state.complete(first, Some(ServerCursorToken::new("cursor-a")));
        let in_flight = state
            .propose_navigation(ServerCursorRequest::Next(ServerCursorToken::default()))
            .unwrap();

        state.reset(10).unwrap();

        // The reset immediately mints its own first-slice handle in the new
        // generation, so the superseded in-flight handle is rejected as
        // stale (a different active handle exists) rather than consumed
        // (no active handle at all).
        assert_eq!(
            state.complete(in_flight, Some(ServerCursorToken::new("cursor-b"))),
            ServerCursorHistoryDisposition::IgnoredStale
        );
    }
}
