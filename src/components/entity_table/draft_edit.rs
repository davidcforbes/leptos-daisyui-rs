//! Pure state machine for `EntityTable`'s exclusive inline-edit mode
//! (`ldui-ff2f`).
//!
//! # Why this is one mode and not per-row editors
//!
//! The owner's interaction spec pins a single constraint that shapes
//! everything here: while a row is being edited, **every other row in the
//! table is disabled**. Creating a row (`+` in the toolbar) and editing an
//! existing one (that row's Edit button) are therefore two entry points into
//! the *same* mode, not two features.
//!
//! That constraint is worth more than it looks. Because at most one row is
//! ever live, there is no set of open editors to reconcile, no editing state
//! keyed by row identity, and no question about what a data refresh does to
//! *other* rows' editors. The table gains one state, not one per row.
//!
//! It also makes the invariant self-enforcing: another row's Edit button must
//! be inert while a row is live, because a second live row is not a degraded
//! experience, it is an unrepresentable state.
//!
//! # Why `Committing` exists
//!
//! The consumer owns persistence and persistence is asynchronous. Without a
//! distinct in-flight state, Save must either drop the edited row before the
//! write is confirmed — losing the user's typing if it fails — or freeze the
//! UI. So Save moves to [`EntityEditPhase::Committing`], which holds the row
//! intact until the consumer resolves it, and a rejection returns to
//! [`EntityEditPhase::Drafting`] with the input still there.
//!
//! # Staleness
//!
//! Each Save mints an [`EntityEditCommit`] carrying a sequence number. A
//! resolution for any sequence other than the current one is ignored, so a
//! slow first attempt cannot resolve a second one — the same discipline
//! `SnapshotTableState` uses for its request and action handles. Cancelling
//! and re-saving therefore cannot be corrupted by the abandoned attempt
//! landing late.
//!
//! This module is deliberately render-free: it is exercised entirely by native
//! tests, so the correctness lives somewhere a browser is not required to
//! observe it.

use std::rc::Rc;

/// Reads a column's editable value out of a row.
///
/// Aliased for the same reason `EntityCellRenderer` and `EntitySortKey` are:
/// the bare `Rc<dyn Fn…>` trips `clippy::type_complexity`, and the name says
/// what the closure is for at every use site.
pub type EntityCellEditorGet<T> = Rc<dyn Fn(&T) -> String>;

/// Writes a user's input back into a row.
pub type EntityCellEditorSet<T> = Rc<dyn Fn(&mut T, String)>;

/// How one column's cell is edited while its row is live.
///
/// An enum rather than a closure pair so later editor kinds (select, number,
/// date) can be added without changing any existing call site. A column
/// without one renders its normal read-only cell **even in the draft row** —
/// a derived column has nothing meaningful to accept for a blank row, and
/// inventing an editor for it would be wrong.
pub enum EntityCellEditor<T> {
    /// A single-line text control.
    Text {
        /// The editable value, which may differ from the displayed text: a
        /// column can render a formatted string and still edit the raw one.
        get: EntityCellEditorGet<T>,
        /// Writes the user's input back into the row the reducer holds.
        set: EntityCellEditorSet<T>,
    },
}

impl<T> EntityCellEditor<T> {
    /// A text editor over one field.
    pub fn text(
        get: impl Fn(&T) -> String + 'static,
        set: impl Fn(&mut T, String) + 'static,
    ) -> Self {
        Self::Text {
            get: Rc::new(get),
            set: Rc::new(set),
        }
    }

    /// Reads the current editable value out of `row`.
    pub fn value(&self, row: &T) -> String {
        match self {
            Self::Text { get, .. } => get(row),
        }
    }

    /// Applies `value` to `row`.
    pub fn apply(&self, row: &mut T, value: String) {
        match self {
            Self::Text { set, .. } => set(row, value),
        }
    }
}

impl<T> Clone for EntityCellEditor<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Text { get, set } => Self::Text {
                get: Rc::clone(get),
                set: Rc::clone(set),
            },
        }
    }
}

impl<T> std::fmt::Debug for EntityCellEditor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text { .. } => f.write_str("EntityCellEditor::Text"),
        }
    }
}

/// Which row an edit session is acting on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityEditTarget {
    /// A new row that does not exist in the data yet.
    Draft,
    /// An existing row, addressed by its stable row key.
    Existing(String),
}

impl EntityEditTarget {
    /// Whether this session is creating a row rather than updating one.
    pub const fn is_draft(&self) -> bool {
        matches!(self, Self::Draft)
    }

    /// The stable row key being edited, or `None` for a new draft row.
    pub fn row_key(&self) -> Option<&str> {
        match self {
            Self::Draft => None,
            Self::Existing(key) => Some(key.as_str()),
        }
    }
}

/// The table's edit phase. `Idle` is the only state an opted-out table has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityEditPhase<R> {
    /// Normal read-only table. Byte-identical to a table that never opted in.
    Idle,
    /// One row is live and editable; every other row is inert.
    Drafting {
        /// The row being edited, carrying the user's input so far.
        row: R,
        /// Which row this is.
        target: EntityEditTarget,
        /// Feedback from a rejected save, if the user has already tried once.
        rejection: Option<String>,
    },
    /// Save has fired and the consumer has not resolved it yet.
    Committing {
        /// The submitted row, held intact until the outcome is known.
        row: R,
        /// Which row this is.
        target: EntityEditTarget,
    },
}

/// How the consumer resolved a commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityEditOutcome {
    /// Persisted. The session ends; the row re-enters through the consumer's
    /// normal data flow rather than being injected by the table.
    Accepted,
    /// Refused. The session returns to editing with the user's input intact
    /// and this message available for display.
    Rejected(String),
}

/// A minted, single-use commit handle.
///
/// Carries the sequence number the reducer will check, so a resolution for a
/// superseded attempt is ignored rather than applied to the current one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityEditCommit<R> {
    sequence: u64,
    row: R,
    target: EntityEditTarget,
}

impl<R> EntityEditCommit<R> {
    /// The row the user submitted. The consumer validates and persists this.
    pub const fn row(&self) -> &R {
        &self.row
    }

    /// Which row was submitted.
    pub const fn target(&self) -> &EntityEditTarget {
        &self.target
    }

    /// Opaque identity of this attempt.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// What a reducer call did, so a caller can tell a real transition from a
/// no-op without comparing states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityEditDisposition {
    /// The state changed.
    Applied,
    /// Refused because a row is already live. Guards the single-live-row
    /// invariant against a second entry point firing.
    IgnoredBusy,
    /// Refused because no session is open.
    IgnoredIdle,
    /// A resolution arrived for a superseded attempt and was discarded.
    IgnoredStale,
}

/// Holds the last accepted input snapshot while an edit is exclusive.
///
/// Input changes remain pending while `editing` is true. A caller publishes
/// the newest one immediately before returning the edit reducer to `Idle`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EntityEditSnapshotGate<S> {
    accepted: S,
    pending: Option<S>,
}

impl<S> EntityEditSnapshotGate<S> {
    /// Starts with the snapshot the table has already accepted for display.
    pub(crate) const fn new(accepted: S) -> Self {
        Self {
            accepted,
            pending: None,
        }
    }

    /// The only snapshot downstream table calculations may read.
    pub(crate) const fn accepted(&self) -> &S {
        &self.accepted
    }

    /// The newest deferred input, exposed only to the native policy tests.
    #[cfg(test)]
    pub(crate) const fn pending(&self) -> Option<&S> {
        self.pending.as_ref()
    }

    /// Accepts an idle input immediately, or coalesces an editing input.
    pub(crate) fn observe(&mut self, next: S, editing: bool) {
        if editing {
            self.pending = Some(next);
        } else {
            self.accepted = next;
            self.pending = None;
        }
    }

    /// Publishes the newest pending input, if one exists.
    pub(crate) fn release(&mut self) -> bool {
        let Some(pending) = self.pending.take() else {
            return false;
        };
        self.accepted = pending;
        true
    }
}

/// The exclusive edit-mode reducer.
///
/// Holds no rendering concerns and no persistence: every transition is a pure
/// function of the current phase and the requested change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityEditState<R> {
    phase: EntityEditPhase<R>,
    next_sequence: u64,
}

impl<R: Clone> Default for EntityEditState<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Clone> EntityEditState<R> {
    /// A table at rest.
    pub const fn new() -> Self {
        Self {
            phase: EntityEditPhase::Idle,
            next_sequence: 1,
        }
    }

    /// The current phase.
    pub const fn phase(&self) -> &EntityEditPhase<R> {
        &self.phase
    }

    /// Whether any row is live. When true the table renders every other row
    /// inert, and locks sort, filter and paging so the live row cannot be
    /// re-ordered or paged out from under the user.
    pub const fn is_editing(&self) -> bool {
        !matches!(self.phase, EntityEditPhase::Idle)
    }

    /// Whether a commit is in flight. Save is disabled here, which is what
    /// prevents a double submit.
    pub const fn is_committing(&self) -> bool {
        matches!(self.phase, EntityEditPhase::Committing { .. })
    }

    /// The row currently live, if any.
    pub const fn editing_row(&self) -> Option<&R> {
        match &self.phase {
            EntityEditPhase::Idle => None,
            EntityEditPhase::Drafting { row, .. } | EntityEditPhase::Committing { row, .. } => {
                Some(row)
            }
        }
    }

    /// Which row is live, if any.
    pub const fn target(&self) -> Option<&EntityEditTarget> {
        match &self.phase {
            EntityEditPhase::Idle => None,
            EntityEditPhase::Drafting { target, .. }
            | EntityEditPhase::Committing { target, .. } => Some(target),
        }
    }

    /// The message from a rejected save, if the user has one pending.
    pub fn rejection(&self) -> Option<&str> {
        match &self.phase {
            EntityEditPhase::Drafting { rejection, .. } => rejection.as_deref(),
            _ => None,
        }
    }

    /// Whether `row_key` is the live row. Every other rendered row is inert.
    pub fn is_row_live(&self, row_key: &str) -> bool {
        self.target()
            .and_then(EntityEditTarget::row_key)
            .is_some_and(|live| live == row_key)
    }

    /// Begins creating a row (`+`).
    pub fn begin_draft(&mut self, blank: R) -> EntityEditDisposition {
        self.begin(blank, EntityEditTarget::Draft)
    }

    /// Begins editing an existing row (that row's Edit button).
    pub fn begin_edit(&mut self, row_key: impl Into<String>, row: R) -> EntityEditDisposition {
        self.begin(row, EntityEditTarget::Existing(row_key.into()))
    }

    fn begin(&mut self, row: R, target: EntityEditTarget) -> EntityEditDisposition {
        // The single-live-row invariant. A second entry point firing while a
        // row is live is refused rather than silently replacing the session,
        // which would discard the user's unsaved typing without asking.
        if self.is_editing() {
            return EntityEditDisposition::IgnoredBusy;
        }
        self.phase = EntityEditPhase::Drafting {
            row,
            target,
            rejection: None,
        };
        EntityEditDisposition::Applied
    }

    /// Applies a field edit to the live row.
    ///
    /// Typing also clears a previous rejection: the message described input
    /// the user has now changed, and leaving it on screen would blame the
    /// current value for a past failure.
    pub fn edit_field(&mut self, apply: impl FnOnce(&mut R)) -> EntityEditDisposition {
        match &mut self.phase {
            EntityEditPhase::Drafting { row, rejection, .. } => {
                apply(row);
                *rejection = None;
                EntityEditDisposition::Applied
            }
            // Deliberately refused mid-commit: the submitted row is what the
            // consumer is persisting, so letting it change underneath would
            // persist something the user never submitted.
            EntityEditPhase::Committing { .. } | EntityEditPhase::Idle => {
                EntityEditDisposition::IgnoredIdle
            }
        }
    }

    /// Escape. Ends the session and discards the row.
    ///
    /// Refused during `Committing`: the write is already in flight, and a
    /// cancel that cannot recall it would only lie about what happened.
    pub fn cancel(&mut self) -> EntityEditDisposition {
        match self.phase {
            EntityEditPhase::Drafting { .. } => {
                self.phase = EntityEditPhase::Idle;
                EntityEditDisposition::Applied
            }
            EntityEditPhase::Committing { .. } => EntityEditDisposition::IgnoredBusy,
            EntityEditPhase::Idle => EntityEditDisposition::IgnoredIdle,
        }
    }

    /// Save. Mints a commit handle and moves the session in flight.
    pub fn commit(&mut self) -> Result<EntityEditCommit<R>, EntityEditDisposition> {
        let EntityEditPhase::Drafting { row, target, .. } = &self.phase else {
            return Err(match self.phase {
                EntityEditPhase::Committing { .. } => EntityEditDisposition::IgnoredBusy,
                _ => EntityEditDisposition::IgnoredIdle,
            });
        };
        let (row, target) = (row.clone(), target.clone());
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.phase = EntityEditPhase::Committing {
            row: row.clone(),
            target: target.clone(),
        };
        Ok(EntityEditCommit {
            sequence,
            row,
            target,
        })
    }

    /// Resolves a commit. A handle from a superseded attempt is ignored.
    pub fn resolve(
        &mut self,
        commit: &EntityEditCommit<R>,
        outcome: EntityEditOutcome,
    ) -> EntityEditDisposition {
        let EntityEditPhase::Committing { row, target } = &self.phase else {
            return EntityEditDisposition::IgnoredStale;
        };
        // `next_sequence` has already advanced past the live attempt, so the
        // live sequence is the one immediately before it.
        if commit.sequence + 1 != self.next_sequence {
            return EntityEditDisposition::IgnoredStale;
        }
        match outcome {
            EntityEditOutcome::Accepted => {
                self.phase = EntityEditPhase::Idle;
            }
            EntityEditOutcome::Rejected(message) => {
                self.phase = EntityEditPhase::Drafting {
                    row: row.clone(),
                    target: target.clone(),
                    rejection: Some(message),
                };
            }
        }
        EntityEditDisposition::Applied
    }
}

#[cfg(test)]
#[path = "draft_edit/tests.rs"]
mod tests;
