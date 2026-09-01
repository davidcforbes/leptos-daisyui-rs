//! Typed, controlled focus requests for mutations an `EntityTable` never sees
//! (`ldui-o0iw`).
//!
//! # The gap this closes
//!
//! [`EntityFocusRecord`](super::EntityFocusRecord) recovers focus for a
//! mutation the table itself observed: it begins with a `focusin` on a marked
//! row action inside the table's own region, so deleting a row from that row's
//! own menu lands focus somewhere sensible. It cannot serve a mutation started
//! **outside** the table — an editor panel beside it whose Delete button
//! removes the selected row. That button is destroyed with the row it deleted,
//! focus falls to `<body>`, and the table has no record to recover from.
//!
//! A consumer cannot fix that from its own layer without querying and focusing
//! DOM this crate owns, which is exactly the cross-layer focus implementation
//! `ldui-ifj.3` moved *into* `EntityTable`. So the table exposes a typed
//! request instead.
//!
//! # It resolves against the presentation, never the source order
//!
//! A request names a **stable row key**, and the table answers it against the
//! rows it is actually painting — after filtering, sorting, paging, grouping
//! and collapse. A row that is filtered or paged away is not "row 3 of the
//! source"; it is not on screen, and the honest answer is the documented
//! table-region fallback rather than a positional guess that would focus an
//! unrelated entity.
//!
//! # Surviving the element being destroyed and recreated
//!
//! The requested row is, by construction, a row that has just been re-rendered:
//! the old element is gone and the new one may not exist yet when the request
//! is observed. The table therefore re-queries by stable key inside a
//! [`request_animation_frame`](leptos::prelude::request_animation_frame), and
//! once more on the following frame, before falling back — the same pattern the
//! internal row-action recovery and the column-move focus restore already use.
//! It never holds a reference to an element across the replacement.
//!
//! # A request states that a mutation was ACCEPTED
//!
//! One request id is applied at most once, and a request is only ever an
//! instruction to move focus. Do not issue one for a mutation that failed or
//! was declined: there is nothing to move focus *to*, and the editor that
//! still owns focus should keep it. The table additionally refuses to steal
//! focus that the user moved somewhere else between the request being observed
//! and the page painting.

use std::fmt;

/// What a focus request aims at inside the row it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityFocusRequestTarget {
    /// The row itself.
    ///
    /// Only rows the table already made focusable can take focus — a table is
    /// row-interactive when `on_row_activate` or `selection` is supplied. A
    /// display-only table has no focusable rows, so a `Row` request there
    /// resolves to the table-region fallback rather than silently inventing a
    /// tab stop the keyboard model never had.
    Row,
    /// A named row action inside that row, identified exactly as
    /// [`EntityRowAction`](super::EntityRowAction) marks it.
    RowAction(String),
}

/// One caller-issued request to move focus to a stable row after an accepted
/// data replacement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFocusRequest {
    /// Request identity. The table applies each id at most once, so a signal
    /// that keeps reporting the same request after it was honored cannot steal
    /// focus back from the user. Bump it for every new request.
    pub id: u64,
    /// The dataset/access scope the request was computed against, compared
    /// against the table's own `focus_scope`. A request stamped with a scope
    /// the table has since left is rejected rather than applied to whatever
    /// row now happens to carry that key.
    pub scope: String,
    /// Stable row identity, from the table's mandatory `row_key`.
    pub row_key: String,
    /// Row, or a named action within it.
    pub target: EntityFocusRequestTarget,
}

impl EntityFocusRequest {
    /// Requests focus on the row itself.
    pub fn row(id: u64, scope: impl Into<String>, row_key: impl Into<String>) -> Self {
        Self {
            id,
            scope: scope.into(),
            row_key: row_key.into(),
            target: EntityFocusRequestTarget::Row,
        }
    }

    /// Requests focus on one named action inside the row.
    pub fn row_action(
        id: u64,
        scope: impl Into<String>,
        row_key: impl Into<String>,
        action_id: impl Into<String>,
    ) -> Self {
        Self {
            id,
            scope: scope.into(),
            row_key: row_key.into(),
            target: EntityFocusRequestTarget::RowAction(action_id.into()),
        }
    }
}

/// What the table did with a request.
///
/// Reported after the fact, so a consumer can log or announce the real result
/// instead of assuming the request succeeded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityFocusRequestOutcome {
    /// The named row was visible and focusable, and now has focus.
    Row {
        /// The stable row key that took focus.
        row_key: String,
    },
    /// The named action inside the named row was visible and enabled, and now
    /// has focus.
    RowAction {
        /// The stable row key holding the focused action.
        row_key: String,
        /// The stable action identity that took focus.
        action_id: String,
    },
    /// The documented fallback: the row is filtered away, paged away, removed,
    /// collapsed, not focusable, or its action is absent or disabled, so focus
    /// went to the table region instead of guessing a row.
    TableRegion,
    /// The request carried a scope the table has left. Nothing was focused.
    StaleScope,
    /// The user moved focus to another meaningful target between the request
    /// being observed and the page painting. Nothing was focused.
    Declined,
}

impl EntityFocusRequestOutcome {
    /// Stable `data-entity-focus-outcome` value, for tests and consumers that
    /// want a non-localized hook.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Row { .. } => "row",
            Self::RowAction { .. } => "row-action",
            Self::TableRegion => "table-region",
            Self::StaleScope => "stale-scope",
            Self::Declined => "declined",
        }
    }

    /// Whether the outcome actually moved focus.
    #[must_use]
    pub const fn moved_focus(&self) -> bool {
        matches!(
            self,
            Self::Row { .. } | Self::RowAction { .. } | Self::TableRegion
        )
    }
}

/// The applied result of one request, echoing the id it answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFocusRequestResolution {
    /// The [`EntityFocusRequest::id`] this answers.
    pub request_id: u64,
    /// What the table actually did.
    pub outcome: EntityFocusRequestOutcome,
}

impl fmt::Display for EntityFocusRequestResolution {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}|{}", self.request_id, self.outcome.as_str())
    }
}

/// Resolves a request against the rows the table is actually painting.
///
/// `visible_row_keys` is the table's own current page, in rendered order —
/// after filtering, sorting, paging, grouping and collapse — never the source
/// snapshot. The DOM may still decline the resulting intent (an unfocusable
/// row, a disabled action), in which case the table downgrades to
/// [`EntityFocusRequestOutcome::TableRegion`]; this function decides everything
/// that does not require the DOM.
#[must_use]
pub fn entity_focus_request_outcome(
    request: &EntityFocusRequest,
    current_scope: &str,
    visible_row_keys: &[String],
) -> EntityFocusRequestOutcome {
    if request.scope != current_scope {
        return EntityFocusRequestOutcome::StaleScope;
    }
    if !visible_row_keys.iter().any(|key| key == &request.row_key) {
        return EntityFocusRequestOutcome::TableRegion;
    }
    match &request.target {
        EntityFocusRequestTarget::Row => EntityFocusRequestOutcome::Row {
            row_key: request.row_key.clone(),
        },
        EntityFocusRequestTarget::RowAction(action_id) => EntityFocusRequestOutcome::RowAction {
            row_key: request.row_key.clone(),
            action_id: action_id.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn a_visible_row_takes_focus_for_either_target() {
        let visible = keys(&["ON-1001", "ON-1002", "ON-1004"]);
        assert_eq!(
            entity_focus_request_outcome(
                &EntityFocusRequest::row(1, "gen-7", "ON-1002"),
                "gen-7",
                &visible
            ),
            EntityFocusRequestOutcome::Row {
                row_key: "ON-1002".to_owned()
            }
        );
        assert_eq!(
            entity_focus_request_outcome(
                &EntityFocusRequest::row_action(2, "gen-7", "ON-1002", "open"),
                "gen-7",
                &visible
            ),
            EntityFocusRequestOutcome::RowAction {
                row_key: "ON-1002".to_owned(),
                action_id: "open".to_owned()
            }
        );
    }

    #[test]
    fn a_stale_scope_is_rejected_before_anything_else() {
        // Even for a row that IS visible: the key may be the same string while
        // naming a different entity in the dataset the table now holds.
        let visible = keys(&["ON-1002"]);
        assert_eq!(
            entity_focus_request_outcome(
                &EntityFocusRequest::row(1, "gen-6", "ON-1002"),
                "gen-7",
                &visible
            ),
            EntityFocusRequestOutcome::StaleScope
        );
    }

    #[test]
    fn a_row_that_is_not_on_the_page_falls_back_and_never_guesses_a_position() {
        // Filtered away, paged away, removed, or collapsed all look the same
        // from here, and all take the documented fallback. Critically, the
        // fallback is NOT "whatever row now sits where that one used to".
        let visible = keys(&["ON-1001", "ON-1004"]);
        for request in [
            EntityFocusRequest::row(1, "gen-7", "ON-1002"),
            EntityFocusRequest::row_action(2, "gen-7", "ON-1002", "open"),
        ] {
            assert_eq!(
                entity_focus_request_outcome(&request, "gen-7", &visible),
                EntityFocusRequestOutcome::TableRegion
            );
        }
        assert_eq!(
            entity_focus_request_outcome(
                &EntityFocusRequest::row(3, "gen-7", "ON-1002"),
                "gen-7",
                &[]
            ),
            EntityFocusRequestOutcome::TableRegion
        );
    }

    #[test]
    fn the_successor_is_named_by_key_not_by_the_deleted_rows_position() {
        // The Office case: ON-1003 was deleted and the page supplied ON-1002 as
        // the successor. Sorting or filtering may have moved it anywhere on the
        // page; the outcome must follow the key.
        for visible in [
            keys(&["ON-1002", "ON-1001", "ON-1004"]),
            keys(&["ON-1004", "ON-1002"]),
            keys(&["ON-1001", "ON-1004", "ON-1002"]),
        ] {
            assert_eq!(
                entity_focus_request_outcome(
                    &EntityFocusRequest::row(9, "gen-7", "ON-1002"),
                    "gen-7",
                    &visible
                ),
                EntityFocusRequestOutcome::Row {
                    row_key: "ON-1002".to_owned()
                }
            );
        }
    }

    #[test]
    fn outcomes_carry_stable_hooks_and_report_whether_focus_moved() {
        assert_eq!(
            EntityFocusRequestOutcome::Row {
                row_key: "a".to_owned()
            }
            .as_str(),
            "row"
        );
        assert_eq!(
            EntityFocusRequestOutcome::TableRegion.as_str(),
            "table-region"
        );
        assert_eq!(
            EntityFocusRequestOutcome::StaleScope.as_str(),
            "stale-scope"
        );
        assert_eq!(EntityFocusRequestOutcome::Declined.as_str(), "declined");
        assert!(EntityFocusRequestOutcome::TableRegion.moved_focus());
        // The two refusals must not read as "focus moved", or a consumer that
        // announces the move would announce one that never happened.
        assert!(!EntityFocusRequestOutcome::StaleScope.moved_focus());
        assert!(!EntityFocusRequestOutcome::Declined.moved_focus());
        assert_eq!(
            EntityFocusRequestResolution {
                request_id: 4,
                outcome: EntityFocusRequestOutcome::Declined,
            }
            .to_string(),
            "4|declined"
        );
    }
}
