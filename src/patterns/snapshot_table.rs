//! Atomic runtime state for opinionated client-snapshot table pages.

use super::{
    ActionFeedbackContent, ActionFeedbackModel, ActionFeedbackState, ActionTransitionError,
};
use std::rc::Rc;

/// Opaque dataset/access generation shared by page, selector, table, focus,
/// and local-result summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotGeneration(u64);

impl SnapshotGeneration {
    /// Stable diagnostic marker suitable for `data-*` attributes.
    pub fn marker(self) -> String {
        self.0.to_string()
    }
}

/// Access replacement that takes precedence over every content phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnapshotAccess {
    /// Snapshot content may be requested and displayed.
    #[default]
    Allowed,
    /// The user's session is no longer valid.
    Expired,
    /// The user is authenticated but not permitted to view the page.
    Forbidden,
}

/// Validation failure while constructing one complete displayed snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotDataError {
    /// Revisions are required so local summaries cannot bind ambiguously.
    EmptyRevision,
    /// A complete snapshot cannot contain more rows than its authoritative
    /// count reports.
    AuthoritativeCountLessThanRows {
        /// Count reported by the authoritative snapshot response.
        authoritative: usize,
        /// Number of rows actually supplied in the response.
        rows: usize,
    },
}

/// One complete, atomically replaceable displayed dataset.
pub struct SnapshotData<R, V, M> {
    dataset: V,
    rows: Rc<Vec<R>>,
    revision: String,
    authoritative_count: usize,
    metadata: Option<M>,
}

impl<R, V, M> SnapshotData<R, V, M> {
    /// Validates and creates a complete displayed snapshot.
    pub fn new(
        dataset: V,
        rows: Rc<Vec<R>>,
        revision: impl Into<String>,
        authoritative_count: usize,
        metadata: Option<M>,
    ) -> Result<Self, SnapshotDataError> {
        let revision = revision.into();
        if revision.trim().is_empty() {
            return Err(SnapshotDataError::EmptyRevision);
        }
        if authoritative_count < rows.len() {
            return Err(SnapshotDataError::AuthoritativeCountLessThanRows {
                authoritative: authoritative_count,
                rows: rows.len(),
            });
        }
        Ok(Self {
            dataset,
            rows,
            revision,
            authoritative_count,
            metadata,
        })
    }

    /// Dataset identity labeling these rows.
    pub const fn dataset(&self) -> &V {
        &self.dataset
    }

    /// Immutable complete snapshot rows.
    pub const fn rows(&self) -> &Rc<Vec<R>> {
        &self.rows
    }

    /// Authoritative snapshot revision.
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Authoritative unfiltered row count.
    pub const fn authoritative_count(&self) -> usize {
        self.authoritative_count
    }

    /// Optional typed snapshot metadata.
    pub const fn metadata(&self) -> Option<&M> {
        self.metadata.as_ref()
    }
}

/// Opaque request token minted only by [`SnapshotTableState::start_request`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotRequestHandle<V> {
    sequence: u64,
    dataset: V,
}

/// Opaque generation-bound token minted only by
/// [`SnapshotTableState::start_action`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotActionHandle<K> {
    generation: SnapshotGeneration,
    sequence: u64,
    key: K,
}

impl<K> SnapshotActionHandle<K> {
    /// Monotonic action-start sequence used for diagnostics.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Dataset/access generation in which the action started.
    pub const fn generation(&self) -> SnapshotGeneration {
        self.generation
    }

    /// Stable action key carried by this token.
    pub const fn key(&self) -> &K {
        &self.key
    }
}

/// Failure to mint a new generation-bound action token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotActionStartError {
    /// An action cannot start while access is replaced.
    AccessUnavailable(SnapshotAccess),
    /// No displayed snapshot exists to bind the action to.
    NoDisplayedSnapshot,
    /// The checked action sequence cannot advance without wrapping.
    SequenceExhausted,
}

/// Whether a generation-bound action completion changed feedback state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotActionDisposition {
    /// The active handle was consumed and its outcome was recorded.
    Applied,
    /// The handle was already consumed or no action remains for its key.
    IgnoredConsumed,
    /// A newer action or dataset/access generation superseded the handle.
    IgnoredStale,
}

impl<V> SnapshotRequestHandle<V> {
    /// Monotonic request sequence for logging and diagnostics.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Destination dataset carried by the token.
    pub const fn dataset(&self) -> &V {
        &self.dataset
    }
}

/// Failure to mint a new request token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotRequestError {
    /// A request cannot start while access is replaced.
    AccessUnavailable(SnapshotAccess),
    /// The checked sequence cannot advance without wrapping.
    SequenceExhausted,
}

/// Whether a completion/failure changed the controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotTransitionDisposition {
    /// The active request was consumed and state changed.
    Applied,
    /// A superseded request token was supplied.
    IgnoredStale,
    /// The token has already been consumed or access cleared it.
    IgnoredConsumed,
    /// The returned snapshot identifies a different dataset than the token.
    IgnoredDatasetMismatch,
}

/// Runtime phase derived from private controller state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotTablePhase {
    /// No request has completed or failed yet.
    NeverLoaded,
    /// First dataset request is in flight.
    InitialLoading,
    /// First dataset request failed and no rows can be retained.
    InitialError,
    /// One complete dataset is displayed with no load transition.
    Displaying,
    /// Another dataset is requested while current rows remain displayed.
    Replacing,
    /// A replacement failed while current rows remain displayed.
    RetainedError,
    /// Session access replaced the complete content surface.
    Expired,
    /// Authorization replaced the complete content surface.
    Forbidden,
}

/// Framework-owned panel presentation selected by runtime precedence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageStatePanelKind {
    /// No dataset has been requested.
    NeverLoaded,
    /// Initial load skeleton/status.
    InitialLoading,
    /// Initial load error.
    InitialError,
    /// The authoritative complete dataset contains no rows.
    EmptyDataset,
    /// Local filters removed every row from a non-empty dataset.
    NoLocalResults,
    /// Expired-session replacement.
    Expired,
    /// Forbidden-access replacement.
    Forbidden,
    /// Retained replacement/refresh notice.
    Replacing,
    /// Retained replacement/refresh failure.
    RetainedError,
}

/// Pure page-render decision. Public code can inspect it but cannot combine a
/// replacement panel with a retained table accidentally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotRenderDecision {
    panel: Option<PageStatePanelKind>,
    table_mounted: bool,
    retained_notice: bool,
}

impl SnapshotRenderDecision {
    /// A panel replaces table content.
    pub(crate) const fn replacement(panel: PageStatePanelKind) -> Self {
        Self {
            panel: Some(panel),
            table_mounted: false,
            retained_notice: false,
        }
    }

    /// A notice is shown while the same table remains mounted.
    pub(crate) const fn retained(panel: PageStatePanelKind) -> Self {
        Self {
            panel: Some(panel),
            table_mounted: true,
            retained_notice: true,
        }
    }

    /// Normal table presentation with no framework panel.
    pub(crate) const fn table_without_panel() -> Self {
        Self {
            panel: None,
            table_mounted: true,
            retained_notice: false,
        }
    }

    /// Panel selected by precedence, if any.
    pub const fn panel(self) -> Option<PageStatePanelKind> {
        self.panel
    }

    /// Whether the same table subtree remains mounted.
    pub const fn table_mounted(self) -> bool {
        self.table_mounted
    }

    /// Whether `panel` is a notice above retained data rather than a replacement.
    pub const fn retained_notice(self) -> bool {
        self.retained_notice
    }
}

/// Filtered-count proof bound to exactly one displayed generation/revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalResultSummary {
    generation: SnapshotGeneration,
    revision: String,
    filtered_count: usize,
}

impl LocalResultSummary {
    /// Number of locally projected rows carried by this identity-bound proof.
    pub const fn filtered_count(&self) -> usize {
        self.filtered_count
    }
}

/// Controlled local rows bound to exactly one displayed generation/revision.
///
/// The private identity proof prevents consumers from pairing filtered rows
/// with a separately supplied dataset key. Create projections only through
/// [`SnapshotTableState::local_row_projection`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotLocalRowProjection<R> {
    summary: LocalResultSummary,
    rows: Rc<Vec<R>>,
}

impl<R> SnapshotLocalRowProjection<R> {
    /// Identity-bound filtered-count proof used by the page render decision.
    pub const fn summary(&self) -> &LocalResultSummary {
        &self.summary
    }

    /// Controlled rows for the generation/revision carried by [`Self::summary`].
    pub const fn rows(&self) -> &Rc<Vec<R>> {
        &self.rows
    }
}

struct SnapshotLoadFailure<V, E> {
    dataset: V,
    error: E,
}

/// Pure reducer for one client-snapshot table page.
pub struct SnapshotTableState<R, V, E, M, K> {
    next_request_sequence: u64,
    active_request: Option<SnapshotRequestHandle<V>>,
    displayed: Option<SnapshotData<R, V, M>>,
    load_failure: Option<SnapshotLoadFailure<V, E>>,
    access: SnapshotAccess,
    generation: SnapshotGeneration,
    actions: ActionFeedbackModel<K>,
    active_actions: Vec<SnapshotActionHandle<K>>,
}

impl<R, V, E, M, K> Default for SnapshotTableState<R, V, E, M, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R, V, E, M, K> SnapshotTableState<R, V, E, M, K> {
    /// Creates an allowed page that has never requested a dataset.
    pub const fn new() -> Self {
        Self {
            next_request_sequence: 0,
            active_request: None,
            displayed: None,
            load_failure: None,
            access: SnapshotAccess::Allowed,
            generation: SnapshotGeneration(0),
            actions: ActionFeedbackModel::new(),
            active_actions: Vec::new(),
        }
    }

    /// Current opaque dataset/access generation.
    pub const fn generation(&self) -> SnapshotGeneration {
        self.generation
    }

    /// Current keyed action collection.
    pub const fn actions(&self) -> &ActionFeedbackModel<K> {
        &self.actions
    }

    /// Mint a filtered-count summary bound to the current displayed snapshot.
    pub fn local_result_summary(&self, filtered_count: usize) -> Option<LocalResultSummary> {
        let displayed = self.displayed.as_ref()?;
        (self.access == SnapshotAccess::Allowed && filtered_count <= displayed.rows.len()).then(
            || LocalResultSummary {
                generation: self.generation,
                revision: displayed.revision.clone(),
                filtered_count,
            },
        )
    }

    /// Mints controlled local rows and their count proof against the current
    /// displayed generation/revision.
    pub fn local_row_projection(&self, rows: Rc<Vec<R>>) -> Option<SnapshotLocalRowProjection<R>> {
        let summary = self.local_result_summary(rows.len())?;
        Some(SnapshotLocalRowProjection { summary, rows })
    }

    /// Returns a projection's rows only while its private generation/revision
    /// still matches the allowed displayed snapshot.
    pub fn validated_local_rows<'a>(
        &self,
        projection: &'a SnapshotLocalRowProjection<R>,
    ) -> Option<&'a Rc<Vec<R>>> {
        (self.validated_local_count(Some(projection.summary())) == Some(projection.rows().len()))
            .then_some(projection.rows())
    }

    fn validated_local_count(&self, local_result: Option<&LocalResultSummary>) -> Option<usize> {
        let displayed = (self.access == SnapshotAccess::Allowed)
            .then_some(self.displayed.as_ref())
            .flatten()?;
        local_result
            .filter(|summary| {
                summary.generation == self.generation
                    && summary.revision == displayed.revision
                    && summary.filtered_count <= displayed.rows.len()
            })
            .map(|summary| summary.filtered_count)
    }

    /// Read the only valid runtime presentation for the current controller.
    pub fn view<'a>(
        &'a self,
        local_result: Option<&LocalResultSummary>,
    ) -> SnapshotTableView<'a, R, V, E, M, K> {
        let displayed = (self.access == SnapshotAccess::Allowed)
            .then_some(self.displayed.as_ref())
            .flatten();
        let local_filtered_count = self.validated_local_count(local_result);
        let phase = match self.access {
            SnapshotAccess::Expired => SnapshotTablePhase::Expired,
            SnapshotAccess::Forbidden => SnapshotTablePhase::Forbidden,
            SnapshotAccess::Allowed => match (
                displayed.is_some(),
                self.active_request.is_some(),
                self.load_failure.is_some(),
            ) {
                (false, true, _) => SnapshotTablePhase::InitialLoading,
                (false, false, true) => SnapshotTablePhase::InitialError,
                (false, false, false) => SnapshotTablePhase::NeverLoaded,
                (true, true, _) => SnapshotTablePhase::Replacing,
                (true, false, true) => SnapshotTablePhase::RetainedError,
                (true, false, false) => SnapshotTablePhase::Displaying,
            },
        };
        SnapshotTableView {
            phase,
            displayed,
            requested_dataset: self
                .active_request
                .as_ref()
                .map(|request| &request.dataset)
                .or_else(|| self.load_failure.as_ref().map(|failure| &failure.dataset)),
            load_error: self.load_failure.as_ref().map(|failure| &failure.error),
            local_filtered_count,
            generation: self.generation,
            actions: &self.actions,
        }
    }

    /// Replaces access atomically, consuming data, requests, and actionable
    /// bindings. Returning to Allowed starts from NeverLoaded.
    pub fn replace_access(&mut self, access: SnapshotAccess) {
        if self.access == access {
            return;
        }
        self.access = access;
        self.generation.0 = self.generation.0.saturating_add(1);
        self.active_request = None;
        self.displayed = None;
        self.load_failure = None;
        self.actions.clear();
        self.active_actions.clear();
    }

    #[cfg(test)]
    fn force_next_request_sequence_for_test(&mut self, sequence: u64) {
        self.next_request_sequence = sequence;
    }
}

impl<R, V: Clone, E, M, K> SnapshotTableState<R, V, E, M, K> {
    /// Mints the next checked opaque request handle and supersedes any older
    /// in-flight destination while retaining displayed rows.
    pub fn start_request(
        &mut self,
        dataset: V,
    ) -> Result<SnapshotRequestHandle<V>, SnapshotRequestError> {
        if self.access != SnapshotAccess::Allowed {
            return Err(SnapshotRequestError::AccessUnavailable(self.access));
        }
        let sequence = self
            .next_request_sequence
            .checked_add(1)
            .ok_or(SnapshotRequestError::SequenceExhausted)?;
        self.next_request_sequence = sequence;
        let handle = SnapshotRequestHandle { sequence, dataset };
        self.active_request = Some(handle.clone());
        self.load_failure = None;
        Ok(handle)
    }
}

impl<R, V: Clone + PartialEq, E, M, K> SnapshotTableState<R, V, E, M, K> {
    fn matching_request(
        &self,
        handle: &SnapshotRequestHandle<V>,
    ) -> Result<(), SnapshotTransitionDisposition> {
        let Some(active) = self.active_request.as_ref() else {
            return Err(SnapshotTransitionDisposition::IgnoredConsumed);
        };
        if active.sequence != handle.sequence || active.dataset != handle.dataset {
            return Err(SnapshotTransitionDisposition::IgnoredStale);
        }
        Ok(())
    }

    /// Applies only the still-active matching token and atomically swaps every
    /// displayed field.
    pub fn complete(
        &mut self,
        handle: SnapshotRequestHandle<V>,
        data: SnapshotData<R, V, M>,
    ) -> SnapshotTransitionDisposition {
        if let Err(disposition) = self.matching_request(&handle) {
            return disposition;
        }
        if data.dataset != handle.dataset {
            self.active_request = None;
            return SnapshotTransitionDisposition::IgnoredDatasetMismatch;
        }
        self.active_request = None;
        self.load_failure = None;
        self.displayed = Some(data);
        self.generation.0 = self.generation.0.saturating_add(1);
        self.actions.clear();
        self.active_actions.clear();
        SnapshotTransitionDisposition::Applied
    }

    /// Applies an error only to the still-active matching token. Existing rows
    /// become a retained-error presentation instead of being discarded.
    pub fn fail(
        &mut self,
        handle: SnapshotRequestHandle<V>,
        error: E,
    ) -> SnapshotTransitionDisposition {
        if let Err(disposition) = self.matching_request(&handle) {
            return disposition;
        }
        self.active_request = None;
        self.load_failure = Some(SnapshotLoadFailure {
            dataset: handle.dataset,
            error,
        });
        SnapshotTransitionDisposition::Applied
    }
}

impl<R, V, E, M, K: Clone + Eq> SnapshotTableState<R, V, E, M, K> {
    /// Starts one action against the current displayed generation, records its
    /// Pending state, and supersedes only an older action with the same key.
    /// Carries no attempt-specific content; see
    /// [`Self::start_action_with_content`] to attach some.
    pub fn start_action(
        &mut self,
        key: K,
    ) -> Result<SnapshotActionHandle<K>, SnapshotActionStartError> {
        self.start_action_with_content(key, ActionFeedbackContent::default())
    }

    /// Starts one action against the current displayed generation, records its
    /// Pending state with optional caller-supplied content, and supersedes
    /// only an older action with the same key. The superseded handle can no
    /// longer complete successfully (see [`Self::finish_action_with_content`]),
    /// so its content can never be attached after this call.
    pub fn start_action_with_content(
        &mut self,
        key: K,
        content: ActionFeedbackContent,
    ) -> Result<SnapshotActionHandle<K>, SnapshotActionStartError> {
        if self.access != SnapshotAccess::Allowed {
            return Err(SnapshotActionStartError::AccessUnavailable(self.access));
        }
        if self.displayed.is_none() {
            return Err(SnapshotActionStartError::NoDisplayedSnapshot);
        }
        let sequence = self
            .actions
            .set_with_content(key.clone(), ActionFeedbackState::Pending, content)
            .map_err(|_| SnapshotActionStartError::SequenceExhausted)?;
        let handle = SnapshotActionHandle {
            generation: self.generation,
            sequence,
            key,
        };
        self.active_actions
            .retain(|active| active.key != handle.key);
        self.active_actions.push(handle.clone());
        Ok(handle)
    }

    /// Records a terminal outcome only for the still-active matching action
    /// handle. Pending transitions must mint a new handle through
    /// [`Self::start_action`]. Carries no attempt-specific content; see
    /// [`Self::finish_action_with_content`] to attach some.
    pub fn finish_action(
        &mut self,
        handle: SnapshotActionHandle<K>,
        state: ActionFeedbackState,
    ) -> Result<SnapshotActionDisposition, ActionTransitionError> {
        self.finish_action_with_content(handle, state, ActionFeedbackContent::default())
    }

    /// Records a terminal outcome, with optional caller-supplied content, only
    /// for the still-active matching action handle. A stale or duplicate
    /// handle (superseded by a newer [`Self::start_action_with_content`] call,
    /// consumed by an earlier completion, or invalidated by a generation
    /// change) is rejected before the content ever reaches the model, so a
    /// stale attempt's text can never be attached over a newer attempt's.
    /// Pending transitions must mint a new handle through
    /// [`Self::start_action_with_content`].
    pub fn finish_action_with_content(
        &mut self,
        handle: SnapshotActionHandle<K>,
        state: ActionFeedbackState,
        content: ActionFeedbackContent,
    ) -> Result<SnapshotActionDisposition, ActionTransitionError> {
        if handle.generation != self.generation
            || self.access != SnapshotAccess::Allowed
            || self.displayed.is_none()
        {
            return Ok(SnapshotActionDisposition::IgnoredStale);
        }
        let Some(index) = self
            .active_actions
            .iter()
            .position(|active| active.key == handle.key)
        else {
            return Ok(SnapshotActionDisposition::IgnoredConsumed);
        };
        if self.active_actions[index].sequence != handle.sequence
            || self.active_actions[index].generation != handle.generation
        {
            return Ok(SnapshotActionDisposition::IgnoredStale);
        }
        if state == ActionFeedbackState::Pending {
            return Err(ActionTransitionError::PendingRequiresStart);
        }
        self.actions.set_with_content(handle.key, state, content)?;
        self.active_actions.remove(index);
        Ok(SnapshotActionDisposition::Applied)
    }

    /// Dismisses one completed action outcome.
    pub fn dismiss_action(&mut self, key: &K) -> bool {
        self.actions.dismiss(key)
    }
}

/// Read-only projection of one valid controller state.
pub struct SnapshotTableView<'a, R, V, E, M, K> {
    phase: SnapshotTablePhase,
    displayed: Option<&'a SnapshotData<R, V, M>>,
    requested_dataset: Option<&'a V>,
    load_error: Option<&'a E>,
    local_filtered_count: Option<usize>,
    generation: SnapshotGeneration,
    actions: &'a ActionFeedbackModel<K>,
}

impl<'a, R, V, E, M, K> SnapshotTableView<'a, R, V, E, M, K> {
    /// Derived runtime phase.
    pub const fn phase(&self) -> SnapshotTablePhase {
        self.phase
    }

    /// Complete displayed binding, absent for replacement-only phases.
    pub const fn displayed(&self) -> Option<&'a SnapshotData<R, V, M>> {
        self.displayed
    }

    /// Active or most recently failed replacement destination.
    pub const fn requested_dataset(&self) -> Option<&'a V> {
        self.requested_dataset
    }

    /// Active initial/retained load failure.
    pub const fn load_error(&self) -> Option<&'a E> {
        self.load_error
    }

    /// Validated local filtered count, absent when no matching summary exists.
    pub const fn local_filtered_count(&self) -> Option<usize> {
        self.local_filtered_count
    }

    /// Opaque generation shared by every identity-critical binding.
    pub const fn generation(&self) -> SnapshotGeneration {
        self.generation
    }

    /// Read-only keyed action feedback.
    pub const fn actions(&self) -> &'a ActionFeedbackModel<K> {
        self.actions
    }

    /// Applies the architecture's access/content/local-result precedence.
    pub fn render_decision(&self) -> SnapshotRenderDecision {
        match self.phase {
            SnapshotTablePhase::Expired => {
                SnapshotRenderDecision::replacement(PageStatePanelKind::Expired)
            }
            SnapshotTablePhase::Forbidden => {
                SnapshotRenderDecision::replacement(PageStatePanelKind::Forbidden)
            }
            SnapshotTablePhase::NeverLoaded => {
                SnapshotRenderDecision::replacement(PageStatePanelKind::NeverLoaded)
            }
            SnapshotTablePhase::InitialLoading => {
                SnapshotRenderDecision::replacement(PageStatePanelKind::InitialLoading)
            }
            SnapshotTablePhase::InitialError => {
                SnapshotRenderDecision::replacement(PageStatePanelKind::InitialError)
            }
            SnapshotTablePhase::Replacing => {
                SnapshotRenderDecision::retained(PageStatePanelKind::Replacing)
            }
            SnapshotTablePhase::RetainedError => {
                SnapshotRenderDecision::retained(PageStatePanelKind::RetainedError)
            }
            SnapshotTablePhase::Displaying => {
                if self.local_filtered_count == Some(0) {
                    let kind = if self
                        .displayed
                        .is_some_and(|snapshot| snapshot.authoritative_count == 0)
                    {
                        PageStatePanelKind::EmptyDataset
                    } else {
                        PageStatePanelKind::NoLocalResults
                    };
                    SnapshotRenderDecision::replacement(kind)
                } else {
                    SnapshotRenderDecision::table_without_panel()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
