//! Caller-controlled selected-key model for
//! [`KeyedResultList`](super::KeyedResultList) (`ldui-bf8c`).
//!
//! # Accepted truth is caller-owned
//!
//! `KeyedResultList` never holds selection state of its own when a
//! [`KeyedResultListSelection`] is supplied. The caller owns the accepted
//! selected key as a `Signal<Option<String>>`, and every pointer or keyboard
//! gesture emits one [`KeyedResultListSelectionProposal`] — a complete
//! replacement value, never a delta and never applied locally. A rejected or
//! delayed proposal leaves the rendered `aria-selected`,
//! `aria-activedescendant`, and highlight styling aligned with the caller's
//! signal, because nothing optimistic was written. This mirrors the
//! established controlled shape used by
//! [`EntityTableSelectionProposal`](crate::components::EntityTableSelectionProposal)
//! and
//! [`ServerTableSelectionProposal`](crate::components::ServerTableSelectionProposal).
//!
//! # A controlled key absent from the current results
//!
//! The list remains the owner of keyboard, hover, scroll-into-view, ARIA, and
//! activation behavior; only the *accepted selected key* is caller-owned. If
//! the caller's accepted key does not match any row in the current `items`
//! (a filter narrowed the results, a replacement dropped that row, the
//! caller has not loaded a matching row yet), the list renders **no** row as
//! falsely selected — `aria-selected` is `false` on every option and
//! `aria-activedescendant` is absent — but it never proposes clearing or
//! otherwise mutates the caller's key. When a row with that key reappears
//! (the filter relaxes, the replacement arrives), the highlight and scroll
//! position are restored automatically, because the displayed selection is
//! computed fresh from `(accepted key, current items)` on every render
//! rather than cached.
//!
//! # Hover preview stays separate
//!
//! Hover preview is never part of the caller-controlled model: it is
//! transient pointer feedback, not a statement about which row is selected,
//! and remains entirely internal to the list in both the controlled and
//! uncontrolled configurations.

use super::types::ResultListItem;
use leptos::prelude::*;

/// Combining controlled `selection` with the uncontrolled
/// `on_selection_change` notification is rejected, not silently resolved:
/// `on_selection_change` exists to report a change the list itself decided,
/// which has no meaning once the caller owns the accepted key.
pub(crate) const CONTROLLED_AND_UNCONTROLLED_SELECTION_CONFIGURATION: &str = "KeyedResultList accepts either selection (controlled) or on_selection_change (uncontrolled), not both";

/// What produced a [`KeyedResultListSelectionProposal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedResultListSelectionCause {
    /// A row was clicked.
    Click,
    /// `ArrowUp`/`ArrowDown`/`Home`/`End` keyboard navigation moved the
    /// highlight.
    Keyboard,
}

/// One user-proposed replacement for the caller's accepted selected key.
///
/// `key` is the complete proposed value — `None` means "no row highlighted"
/// — never a delta against the previous key. Nothing is applied until the
/// caller's own signal changes; see the module documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedResultListSelectionProposal {
    /// The proposed accepted key, or `None` to propose clearing the
    /// selection.
    pub key: Option<String>,
    /// The gesture that produced this proposal.
    pub cause: KeyedResultListSelectionCause,
}

/// Opt-in caller-controlled selected-key model for
/// [`KeyedResultList`](super::KeyedResultList).
///
/// ```rust,no_run
/// # use leptos::prelude::*;
/// # use leptos_daisyui_rs::components::*;
/// # fn demo() {
/// let accepted_key = RwSignal::new(Some("case-a".to_string()));
/// let selection = KeyedResultListSelection::controlled(
///     accepted_key.into(),
///     Callback::new(move |proposal: KeyedResultListSelectionProposal| {
///         // Accepted truth stays caller-owned: apply, or decline.
///         accepted_key.set(proposal.key);
///     }),
/// );
/// # let _ = selection;
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct KeyedResultListSelection {
    pub(crate) selected_key: Signal<Option<String>>,
    pub(crate) on_change: Callback<KeyedResultListSelectionProposal>,
}

impl KeyedResultListSelection {
    /// Creates controlled selection ownership over a single accepted key.
    /// `on_change` receives complete replacement proposals; the caller
    /// decides whether/how to apply them to `selected_key`.
    pub fn controlled(
        selected_key: Signal<Option<String>>,
        on_change: Callback<KeyedResultListSelectionProposal>,
    ) -> Self {
        Self {
            selected_key,
            on_change,
        }
    }

    /// The caller-owned accepted key.
    pub fn selected_key(self) -> Signal<Option<String>> {
        self.selected_key
    }
}

/// Rejects combining controlled `selection` with the uncontrolled
/// `on_selection_change` notification instead of silently picking one.
pub(crate) fn resolve_result_list_selection_mode(
    has_controlled: bool,
    has_uncontrolled_change: bool,
) -> Result<(), &'static str> {
    if has_controlled && has_uncontrolled_change {
        Err(CONTROLLED_AND_UNCONTROLLED_SELECTION_CONFIGURATION)
    } else {
        Ok(())
    }
}

/// The key to actually render as selected/active: the caller's accepted key,
/// filtered to rows that exist in the current `items`. An accepted key with
/// no matching row renders no false highlight without mutating or proposing
/// against the caller's signal; the filter is recomputed on every call, so
/// the key resumes rendering the instant a matching row reappears.
pub(crate) fn displayed_controlled_key<T>(
    accepted_key: Option<&str>,
    items: &[ResultListItem<T>],
) -> Option<String> {
    accepted_key
        .filter(|key| items.iter().any(|item| item.key == *key))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::result_list::types::ResultRow;

    fn item(key: &str) -> ResultListItem<()> {
        ResultListItem::new(key, ResultRow::new(key), ())
    }

    #[test]
    fn accepted_key_present_in_items_is_displayed() {
        let items = vec![item("case-a"), item("case-b")];
        assert_eq!(
            displayed_controlled_key(Some("case-b"), &items),
            Some("case-b".to_owned())
        );
    }

    #[test]
    fn accepted_key_absent_from_items_renders_no_highlight() {
        let items = vec![item("case-a"), item("case-b")];
        assert_eq!(displayed_controlled_key(Some("case-x"), &items), None);
    }

    #[test]
    fn no_accepted_key_renders_no_highlight() {
        let items = vec![item("case-a")];
        assert_eq!(displayed_controlled_key(None, &items), None);
    }

    #[test]
    fn accepted_key_reappearing_after_a_replacement_is_displayed_again() {
        let items_without = vec![item("case-a")];
        let items_with = vec![item("case-a"), item("case-b")];
        assert_eq!(
            displayed_controlled_key(Some("case-b"), &items_without),
            None
        );
        assert_eq!(
            displayed_controlled_key(Some("case-b"), &items_with),
            Some("case-b".to_owned())
        );
    }

    #[test]
    fn empty_items_never_display_a_highlight() {
        assert_eq!(
            displayed_controlled_key(Some("case-a"), &Vec::<ResultListItem<()>>::new()),
            None
        );
    }

    #[test]
    fn controlled_and_uncontrolled_together_is_rejected() {
        assert_eq!(
            resolve_result_list_selection_mode(true, true),
            Err(CONTROLLED_AND_UNCONTROLLED_SELECTION_CONFIGURATION)
        );
    }

    #[test]
    fn controlled_alone_is_accepted() {
        assert_eq!(resolve_result_list_selection_mode(true, false), Ok(()));
    }

    #[test]
    fn uncontrolled_alone_is_accepted() {
        assert_eq!(resolve_result_list_selection_mode(false, true), Ok(()));
    }

    #[test]
    fn neither_is_accepted() {
        assert_eq!(resolve_result_list_selection_mode(false, false), Ok(()));
    }
}
