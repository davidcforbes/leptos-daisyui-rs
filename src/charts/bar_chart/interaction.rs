//! Keyed hover/focus state and the pure reducer that moves it.
//!
//! Every input event — pointer, focus, arrow key, Escape, and a data
//! replacement — funnels through [`reduce`], which is a pure function of the
//! previous state and the bar keys before and after. DOM side effects (moving
//! focus, invoking the host's callback) stay at the call sites, so all of the
//! behaviour the acceptance criteria describe is testable natively.
//!
//! **State is held by key, never by index.** An index re-points at a different
//! office the moment the caller sorts most-dragging-first, filters, or replaces
//! the dataset; a key does not. That is the same identity rule
//! `ldui-nz6d`/`ldui-px06` established for tables and `ldui-9tr` for the line
//! chart's categories.

/// Which bar the reader is hovering, has focused, and would tab to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BarInteraction {
    /// The single tab stop. Always a key that exists and is activatable, once
    /// [`reduce`] has seen any data.
    pub roving_key: Option<String>,
    /// The focused bar, if focus is inside the chart.
    pub focused_key: Option<String>,
    /// The hovered bar, if a pointer is over one.
    pub hovered_key: Option<String>,
}

impl BarInteraction {
    /// The bar a reader is currently attending to: the pointer wins over
    /// focus, matching every hover surface in this crate.
    pub(super) fn active_key(&self) -> Option<&str> {
        self.hovered_key.as_deref().or(self.focused_key.as_deref())
    }
}

/// A keyboard navigation request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Nav {
    Previous,
    Next,
    First,
    Last,
}

/// Something that happened to the chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Focused(String),
    Blurred,
    Hovered(String),
    HoverEnded,
    MoveFocus(Nav),
    /// Escape: give up the hover and the focus highlight without moving the
    /// tab stop, so a reader who dismisses is not sent back to bar one.
    Dismiss,
    /// The data changed. Keys are re-resolved rather than reset.
    ReconcileData,
}

/// Applies `action` to `state`.
///
/// `previous` and `next` are the *activatable* keys before and after the
/// change; they are the same list for every action but [`Action::ReconcileData`].
pub(super) fn reduce(
    state: &BarInteraction,
    action: Action,
    previous: &[String],
    next: &[String],
) -> BarInteraction {
    let mut out = state.clone();
    match action {
        Action::Focused(key) => {
            if contains(next, &key) {
                out.focused_key = Some(key.clone());
                out.roving_key = Some(key);
            }
        }
        Action::Blurred => out.focused_key = None,
        Action::Hovered(key) => {
            if contains(next, &key) {
                out.hovered_key = Some(key);
            }
        }
        Action::HoverEnded => out.hovered_key = None,
        Action::Dismiss => {
            out.hovered_key = None;
            out.focused_key = None;
        }
        Action::MoveFocus(nav) => {
            if let Some(key) = moved(state, nav, next) {
                out.focused_key = Some(key.clone());
                out.roving_key = Some(key);
            }
        }
        Action::ReconcileData => {
            out.roving_key = reconcile(state.roving_key.as_deref(), previous, next);
            out.focused_key = state
                .focused_key
                .as_deref()
                .and_then(|key| reconcile(Some(key), previous, next));
            // A pointer's position after a data replacement is not knowable, so
            // the hover is dropped rather than guessed at.
            out.hovered_key = None;
        }
    }
    // The tab stop must always land somewhere real: the first activatable bar
    // when the chart has one, and nothing at all when it does not.
    if out
        .roving_key
        .as_deref()
        .is_none_or(|key| !contains(next, key))
    {
        out.roving_key = next.first().cloned();
    }
    out
}

fn contains(keys: &[String], key: &str) -> bool {
    keys.iter().any(|candidate| candidate == key)
}

fn index_of(keys: &[String], key: Option<&str>) -> Option<usize> {
    let key = key?;
    keys.iter().position(|candidate| candidate == key)
}

fn moved(state: &BarInteraction, nav: Nav, keys: &[String]) -> Option<String> {
    if keys.is_empty() {
        return None;
    }
    let last = keys.len() - 1;
    let current = index_of(keys, state.focused_key.as_deref())
        .or_else(|| index_of(keys, state.roving_key.as_deref()))
        .unwrap_or(0);
    let index = match nav {
        // Clamped rather than wrapping: a composite widget that silently jumps
        // from the last office back to the first hides the end of the list from
        // a reader who cannot see the bars.
        Nav::Previous => current.saturating_sub(1),
        Nav::Next => (current + 1).min(last),
        Nav::First => 0,
        Nav::Last => last,
    };
    keys.get(index).cloned()
}

/// Follows `key` through a data change.
///
/// A key that still exists keeps its state, wherever it moved to — which is
/// what makes focus survive a sort. A key that was removed hands its state to
/// whatever now occupies its old position, clamped to the end of the list, so
/// focus moves *predictably* instead of vanishing or jumping to the start.
fn reconcile(key: Option<&str>, previous: &[String], next: &[String]) -> Option<String> {
    let key = key?;
    if contains(next, key) {
        return Some(key.to_string());
    }
    if next.is_empty() {
        return None;
    }
    let old_index = index_of(previous, Some(key))?;
    next.get(old_index.min(next.len() - 1)).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn start(all: &[String]) -> BarInteraction {
        reduce(&BarInteraction::default(), Action::ReconcileData, all, all)
    }

    #[test]
    fn the_tab_stop_starts_on_the_first_activatable_bar() {
        let all = keys(&["north", "south", "east"]);
        let state = start(&all);

        assert_eq!(state.roving_key.as_deref(), Some("north"));
        assert_eq!(state.focused_key, None, "no focus is claimed on render");
        assert_eq!(state.hovered_key, None);
        assert_eq!(state.active_key(), None);
    }

    #[test]
    fn a_chart_with_nothing_activatable_has_no_tab_stop_at_all() {
        // The "noninteractive charts gain no tab stops" criterion's data-driven
        // half: an all-missing chart offers nowhere to tab to.
        let state = start(&[]);

        assert_eq!(state.roving_key, None);
    }

    #[test]
    fn focus_and_hover_move_the_state_and_the_pointer_wins_for_active() {
        let all = keys(&["north", "south"]);
        let focused = reduce(
            &start(&all),
            Action::Focused("north".to_string()),
            &all,
            &all,
        );
        assert_eq!(focused.focused_key.as_deref(), Some("north"));
        assert_eq!(focused.roving_key.as_deref(), Some("north"));
        assert_eq!(focused.active_key(), Some("north"));

        let hovered = reduce(&focused, Action::Hovered("south".to_string()), &all, &all);
        assert_eq!(
            hovered.active_key(),
            Some("south"),
            "a pointer over one bar wins over focus on another"
        );
        assert_eq!(
            hovered.focused_key.as_deref(),
            Some("north"),
            "hovering must not steal focus"
        );

        let unhovered = reduce(&hovered, Action::HoverEnded, &all, &all);
        assert_eq!(unhovered.active_key(), Some("north"));
    }

    #[test]
    fn a_key_that_is_not_activatable_is_refused_rather_than_stored() {
        let all = keys(&["north"]);
        let state = reduce(
            &start(&all),
            Action::Focused("gone".to_string()),
            &all,
            &all,
        );

        assert_eq!(state.focused_key, None);
        assert_eq!(state.roving_key.as_deref(), Some("north"));
    }

    #[test]
    fn arrow_navigation_clamps_at_both_ends() {
        let all = keys(&["a", "b", "c"]);
        let mut state = reduce(&start(&all), Action::Focused("a".to_string()), &all, &all);

        state = reduce(&state, Action::MoveFocus(Nav::Previous), &all, &all);
        assert_eq!(
            state.focused_key.as_deref(),
            Some("a"),
            "clamped, not wrapped"
        );

        state = reduce(&state, Action::MoveFocus(Nav::Next), &all, &all);
        assert_eq!(state.focused_key.as_deref(), Some("b"));

        state = reduce(&state, Action::MoveFocus(Nav::Last), &all, &all);
        assert_eq!(state.focused_key.as_deref(), Some("c"));

        state = reduce(&state, Action::MoveFocus(Nav::Next), &all, &all);
        assert_eq!(
            state.focused_key.as_deref(),
            Some("c"),
            "clamped at the end"
        );

        state = reduce(&state, Action::MoveFocus(Nav::First), &all, &all);
        assert_eq!(state.focused_key.as_deref(), Some("a"));
        assert_eq!(state.roving_key.as_deref(), Some("a"));
    }

    #[test]
    fn navigation_skips_bars_that_have_no_value() {
        // Only activatable keys are ever handed to the reducer, so a gap in the
        // data is simply not in the list — Arrow moves over it in one step.
        let activatable = keys(&["a", "c"]);
        let state = reduce(
            &reduce(
                &start(&activatable),
                Action::Focused("a".to_string()),
                &activatable,
                &activatable,
            ),
            Action::MoveFocus(Nav::Next),
            &activatable,
            &activatable,
        );

        assert_eq!(state.focused_key.as_deref(), Some("c"));
    }

    #[test]
    fn escape_drops_the_highlight_without_moving_the_tab_stop() {
        let all = keys(&["a", "b", "c"]);
        let focused = reduce(&start(&all), Action::Focused("c".to_string()), &all, &all);
        let dismissed = reduce(&focused, Action::Dismiss, &all, &all);

        assert_eq!(dismissed.focused_key, None);
        assert_eq!(dismissed.hovered_key, None);
        assert_eq!(
            dismissed.roving_key.as_deref(),
            Some("c"),
            "a reader who dismisses must not be sent back to bar one"
        );
    }

    #[test]
    fn focus_follows_a_key_through_a_reorder() {
        // The reason state is keyed: sorting most-dragging-first is exactly
        // what a consumer does between renders.
        let before = keys(&["north", "south", "east"]);
        let after = keys(&["east", "north", "south"]);
        let focused = reduce(
            &start(&before),
            Action::Focused("south".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&focused, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key.as_deref(), Some("south"));
        assert_eq!(reconciled.roving_key.as_deref(), Some("south"));
    }

    #[test]
    fn removing_the_focused_bar_moves_focus_predictably_to_its_position() {
        let before = keys(&["a", "b", "c"]);
        let after = keys(&["a", "c"]);
        let focused = reduce(
            &start(&before),
            Action::Focused("b".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&focused, Action::ReconcileData, &before, &after);

        assert_eq!(
            reconciled.focused_key.as_deref(),
            Some("c"),
            "the bar that now occupies the removed one's position takes focus"
        );
        assert_eq!(reconciled.roving_key.as_deref(), Some("c"));
    }

    #[test]
    fn removing_the_last_focused_bar_clamps_to_the_new_end() {
        let before = keys(&["a", "b", "c"]);
        let after = keys(&["a", "b"]);
        let focused = reduce(
            &start(&before),
            Action::Focused("c".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&focused, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key.as_deref(), Some("b"));
    }

    #[test]
    fn emptying_the_chart_clears_every_key_rather_than_dangling() {
        let before = keys(&["a", "b"]);
        let focused = reduce(
            &start(&before),
            Action::Focused("a".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&focused, Action::ReconcileData, &before, &[]);

        assert_eq!(reconciled.focused_key, None);
        assert_eq!(reconciled.roving_key, None);
        assert_eq!(reconciled.hovered_key, None);
    }

    #[test]
    fn a_data_change_drops_a_hover_it_cannot_verify() {
        let before = keys(&["a", "b"]);
        let hovered = reduce(
            &start(&before),
            Action::Hovered("a".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&hovered, Action::ReconcileData, &before, &before);

        assert_eq!(
            reconciled.hovered_key, None,
            "where the pointer now sits is not knowable from the data alone"
        );
    }

    #[test]
    fn a_wholesale_replacement_still_leaves_a_reachable_tab_stop() {
        let before = keys(&["a", "b"]);
        let after = keys(&["x", "y"]);
        let focused = reduce(
            &start(&before),
            Action::Focused("a".to_string()),
            &before,
            &before,
        );

        let reconciled = reduce(&focused, Action::ReconcileData, &before, &after);

        assert_eq!(reconciled.focused_key.as_deref(), Some("x"));
        assert!(after.contains(&reconciled.roving_key.clone().unwrap()));
    }

    #[test]
    fn the_tab_stop_is_always_a_key_that_exists() {
        // The invariant the render relies on when deciding which rect gets
        // tabindex=0: swept over every action against a changed dataset.
        let before = keys(&["a", "b", "c"]);
        let after = keys(&["b"]);
        let actions = [
            Action::Focused("a".to_string()),
            Action::Hovered("b".to_string()),
            Action::MoveFocus(Nav::Last),
            Action::Dismiss,
            Action::Blurred,
        ];
        for action in actions {
            let state = reduce(&start(&before), action, &before, &before);
            let reconciled = reduce(&state, Action::ReconcileData, &before, &after);
            let roving = reconciled.roving_key.expect("a nonempty chart has a stop");
            assert!(after.contains(&roving), "{roving} is not in {after:?}");
            if let Some(focused) = reconciled.focused_key {
                assert!(after.contains(&focused), "{focused} is not in {after:?}");
            }
        }
    }
}
