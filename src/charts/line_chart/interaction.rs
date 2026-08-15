use super::{
    normalize::NormalizedChart,
    types::{
        LineChartActivation, LineChartActivationSource, LineChartActivationValue,
        LineChartModifiers,
    },
};

/// The selected category and its preferred finite series.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActivePoint {
    pub category_index: usize,
    pub preferred_series_index: Option<usize>,
}

/// A keyboard movement understood by the categorical chart's roving target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NavigationKey {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

/// Ephemeral hover, focus, and roving-tab-stop state for a categorical chart.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct InteractionState {
    pub hovered: Option<ActivePoint>,
    pub focused: Option<ActivePoint>,
    pub roving_category_key: Option<String>,
    pub dismissed_category_key: Option<String>,
}

/// An input event reduced into [`InteractionState`].
#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionAction {
    PointerEntered,
    PointerMoved(ActivePoint),
    PointerLeft,
    Focused(ActivePoint),
    Blurred,
    MoveFocus(NavigationKey),
    Dismiss,
    ReconcileData,
}

/// Applies an interaction event without performing DOM or callback side effects.
pub(super) fn reduce(
    state: &InteractionState,
    action: InteractionAction,
    previous: &NormalizedChart,
    next: &NormalizedChart,
) -> InteractionState {
    match action {
        InteractionAction::PointerEntered => InteractionState {
            dismissed_category_key: None,
            ..state.clone()
        },
        InteractionAction::PointerMoved(active) => InteractionState {
            hovered: normalized_active(next, active),
            ..state.clone()
        },
        InteractionAction::PointerLeft => InteractionState {
            hovered: None,
            ..state.clone()
        },
        InteractionAction::Focused(active) => {
            let Some(focused) = normalized_active(next, active) else {
                return state.clone();
            };
            let focus_changed = state.focused.as_ref() != Some(&focused);
            InteractionState {
                roving_category_key: next
                    .categories
                    .get(focused.category_index)
                    .map(|category| category.key.clone()),
                focused: Some(focused),
                dismissed_category_key: if focus_changed {
                    None
                } else {
                    state.dismissed_category_key.clone()
                },
                ..state.clone()
            }
        }
        InteractionAction::Blurred => InteractionState {
            focused: None,
            ..state.clone()
        },
        InteractionAction::MoveFocus(key) => move_focus(state, key, next),
        InteractionAction::Dismiss => InteractionState {
            dismissed_category_key: raw_active(state)
                .and_then(|active| next.categories.get(active.category_index))
                .map(|category| category.key.clone()),
            ..state.clone()
        },
        InteractionAction::ReconcileData => reconcile(state, previous, next),
    }
}

/// Returns the tooltip's active point, honoring pointer precedence and Escape.
pub(super) fn displayed_active(
    state: &InteractionState,
    chart: &NormalizedChart,
) -> Option<ActivePoint> {
    if state.dismissed_category_key.is_some() {
        return None;
    }
    let active = state.hovered.as_ref().or(state.focused.as_ref())?;
    chart.categories.get(active.category_index)?;
    if !category_has_finite_value(chart, active.category_index) {
        return None;
    }
    Some(active.clone())
}

/// Builds the shared pointer/keyboard activation payload for one category.
pub(super) fn activation_for(
    chart: &NormalizedChart,
    active: ActivePoint,
    source: LineChartActivationSource,
    modifiers: LineChartModifiers,
) -> Option<LineChartActivation> {
    let category = chart.categories.get(active.category_index)?;
    let values = chart
        .series
        .iter()
        .filter_map(|series| {
            let point = series.points.get(active.category_index)?;
            let value = point.value.filter(|value| value.is_finite())?;
            Some(LineChartActivationValue {
                series_id: series.id.clone(),
                series_name: series.name.clone(),
                value,
                display_value: point
                    .display_value
                    .clone()
                    .unwrap_or_else(|| value.to_string()),
            })
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }

    let preferred_series_index = finite_series_indices(chart, active.category_index)
        .into_iter()
        .find(|index| Some(*index) == active.preferred_series_index)
        .or_else(|| first_finite_series_index(chart, active.category_index));

    Some(LineChartActivation {
        category_index: active.category_index,
        category_key: category.key.clone(),
        category_label: category.label.clone(),
        preferred_series_id: preferred_series_index
            .and_then(|index| chart.series.get(index))
            .map(|series| series.id.clone()),
        values,
        source,
        modifiers,
    })
}

fn move_focus(
    state: &InteractionState,
    key: NavigationKey,
    chart: &NormalizedChart,
) -> InteractionState {
    let valid_categories = finite_category_indices(chart);
    let Some(category_index) = navigation_category_index(state, key, chart, &valid_categories)
    else {
        return InteractionState {
            hovered: None,
            focused: None,
            roving_category_key: None,
            dismissed_category_key: None,
            ..state.clone()
        };
    };
    let finite_series = finite_series_indices(chart, category_index);
    let preferred_series_index = match key {
        NavigationKey::Up | NavigationKey::Down => cycle_series(
            &finite_series,
            focused_series_at(state, category_index),
            key,
        ),
        _ => focused_series_at(state, category_index)
            .filter(|index| finite_series.contains(index))
            .or_else(|| finite_series.first().copied()),
    };

    let focused = ActivePoint {
        category_index,
        preferred_series_index,
    };
    let focus_changed = state.focused.as_ref() != Some(&focused);
    InteractionState {
        focused: Some(focused),
        roving_category_key: chart
            .categories
            .get(category_index)
            .map(|category| category.key.clone()),
        dismissed_category_key: if focus_changed {
            None
        } else {
            state.dismissed_category_key.clone()
        },
        ..state.clone()
    }
}

fn navigation_category_index(
    state: &InteractionState,
    key: NavigationKey,
    chart: &NormalizedChart,
    valid_categories: &[usize],
) -> Option<usize> {
    let first = *valid_categories.first()?;
    match key {
        NavigationKey::Home => Some(first),
        NavigationKey::End => valid_categories.last().copied(),
        NavigationKey::Up | NavigationKey::Down => {
            current_category_index(state, chart, valid_categories).or(Some(first))
        }
        NavigationKey::Left | NavigationKey::Right => {
            let Some(current) = current_category_index(state, chart, valid_categories) else {
                return Some(first);
            };
            let position = valid_categories
                .iter()
                .position(|index| *index == current)
                .unwrap_or(0);
            let target = match key {
                NavigationKey::Left => position.saturating_sub(1),
                NavigationKey::Right => (position + 1).min(valid_categories.len() - 1),
                _ => unreachable!(),
            };
            valid_categories.get(target).copied()
        }
    }
}

fn current_category_index(
    state: &InteractionState,
    chart: &NormalizedChart,
    valid_categories: &[usize],
) -> Option<usize> {
    state
        .roving_category_key
        .as_deref()
        .and_then(|key| category_index_for_key(chart, key))
        .filter(|index| valid_categories.contains(index))
        .or_else(|| {
            state
                .focused
                .as_ref()
                .map(|active| active.category_index)
                .filter(|index| valid_categories.contains(index))
        })
        .or_else(|| {
            state
                .hovered
                .as_ref()
                .map(|active| active.category_index)
                .filter(|index| valid_categories.contains(index))
        })
}

fn focused_series_at(state: &InteractionState, category_index: usize) -> Option<usize> {
    state
        .focused
        .as_ref()
        .filter(|active| active.category_index == category_index)
        .and_then(|active| active.preferred_series_index)
}

fn cycle_series(
    finite_series: &[usize],
    current: Option<usize>,
    key: NavigationKey,
) -> Option<usize> {
    let first = *finite_series.first()?;
    let position = current
        .and_then(|current| finite_series.iter().position(|index| *index == current))
        .unwrap_or(0);
    match key {
        NavigationKey::Down => finite_series
            .get((position + 1) % finite_series.len())
            .copied(),
        NavigationKey::Up => finite_series
            .get((position + finite_series.len() - 1) % finite_series.len())
            .copied(),
        _ => Some(first),
    }
}

fn reconcile(
    state: &InteractionState,
    previous: &NormalizedChart,
    next: &NormalizedChart,
) -> InteractionState {
    let old_index = old_category_index(state, previous);
    let dismissed_index = state
        .dismissed_category_key
        .as_deref()
        .and_then(|key| category_index_for_key(previous, key));
    let dismissed_is_valid = state
        .dismissed_category_key
        .as_deref()
        .and_then(|key| category_index_for_key(next, key))
        .is_some_and(|index| category_has_finite_value(next, index));
    if state.dismissed_category_key.is_some() && !dismissed_is_valid {
        return InteractionState {
            hovered: None,
            focused: None,
            roving_category_key: nearest_valid_category_index(next, dismissed_index.or(old_index))
                .and_then(|index| next.categories.get(index))
                .map(|category| category.key.clone()),
            dismissed_category_key: None,
        };
    }
    let primary = raw_active(state);
    let reconciled_primary = primary.and_then(|active| reconcile_active(active, previous, next));
    if primary.is_some() && reconciled_primary.is_none() {
        return InteractionState {
            hovered: None,
            focused: None,
            roving_category_key: nearest_valid_category_index(next, old_index)
                .and_then(|index| next.categories.get(index))
                .map(|category| category.key.clone()),
            dismissed_category_key: None,
        };
    }
    let hovered = state
        .hovered
        .as_ref()
        .and_then(|_| reconciled_primary.clone());
    let focused = if state.hovered.is_some() {
        state
            .focused
            .as_ref()
            .and_then(|active| reconcile_active(active, previous, next))
    } else {
        reconciled_primary
    };
    let roving_category_key = state
        .roving_category_key
        .as_deref()
        .and_then(|key| category_index_for_key(next, key))
        .filter(|index| category_has_finite_value(next, *index))
        .or_else(|| nearest_valid_category_index(next, old_index))
        .and_then(|index| next.categories.get(index))
        .map(|category| category.key.clone());
    let dismissed_category_key = state
        .dismissed_category_key
        .as_deref()
        .and_then(|key| category_index_for_key(next, key))
        .filter(|index| category_has_finite_value(next, *index))
        .and_then(|index| next.categories.get(index))
        .map(|category| category.key.clone());

    InteractionState {
        hovered,
        focused,
        roving_category_key,
        dismissed_category_key,
    }
}

fn reconcile_active(
    active: &ActivePoint,
    previous: &NormalizedChart,
    next: &NormalizedChart,
) -> Option<ActivePoint> {
    let previous_category = previous.categories.get(active.category_index)?;
    let category_index = category_index_for_key(next, &previous_category.key)?;
    if !category_has_finite_value(next, category_index) {
        return None;
    }
    let previous_series_id = active
        .preferred_series_index
        .and_then(|index| previous.series.get(index))
        .map(|series| series.id.as_str());
    let preferred_series_index = previous_series_id
        .and_then(|id| {
            next.series.iter().position(|series| {
                series.id == id
                    && series
                        .points
                        .get(category_index)
                        .and_then(|point| point.value)
                        .is_some_and(f64::is_finite)
            })
        })
        .or_else(|| first_finite_series_index(next, category_index));
    Some(ActivePoint {
        category_index,
        preferred_series_index,
    })
}

fn old_category_index(state: &InteractionState, previous: &NormalizedChart) -> Option<usize> {
    raw_active(state)
        .map(|active| active.category_index)
        .filter(|index| *index < previous.categories.len())
        .or_else(|| {
            state
                .roving_category_key
                .as_deref()
                .and_then(|key| category_index_for_key(previous, key))
        })
}

fn raw_active(state: &InteractionState) -> Option<&ActivePoint> {
    state.hovered.as_ref().or(state.focused.as_ref())
}

fn normalized_active(chart: &NormalizedChart, active: ActivePoint) -> Option<ActivePoint> {
    category_has_finite_value(chart, active.category_index).then(|| ActivePoint {
        category_index: active.category_index,
        preferred_series_index: active
            .preferred_series_index
            .filter(|index| finite_series_indices(chart, active.category_index).contains(index))
            .or_else(|| first_finite_series_index(chart, active.category_index)),
    })
}

fn finite_category_indices(chart: &NormalizedChart) -> Vec<usize> {
    chart
        .categories
        .iter()
        .enumerate()
        .filter_map(|(index, _)| category_has_finite_value(chart, index).then_some(index))
        .collect()
}

fn nearest_valid_category_index(
    chart: &NormalizedChart,
    old_index: Option<usize>,
) -> Option<usize> {
    let valid = finite_category_indices(chart);
    let old_index = old_index.unwrap_or(0);
    valid
        .into_iter()
        .min_by_key(|index| index.abs_diff(old_index))
}

fn category_index_for_key(chart: &NormalizedChart, key: &str) -> Option<usize> {
    chart
        .categories
        .iter()
        .position(|category| category.key == key)
}

fn category_has_finite_value(chart: &NormalizedChart, category_index: usize) -> bool {
    first_finite_series_index(chart, category_index).is_some()
}

fn first_finite_series_index(chart: &NormalizedChart, category_index: usize) -> Option<usize> {
    finite_series_indices(chart, category_index)
        .into_iter()
        .next()
}

fn finite_series_indices(chart: &NormalizedChart, category_index: usize) -> Vec<usize> {
    chart
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            series
                .points
                .get(category_index)
                .and_then(|point| point.value)
                .filter(|value| value.is_finite())
                .map(|_| index)
        })
        .collect()
}

/// Focuses an SVG element after roving navigation on the browser target.
#[cfg(target_arch = "wasm32")]
pub(super) fn focus_svg_element(id: &str) {
    use wasm_bindgen::JsCast;

    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    let Ok(value) =
        js_sys::Reflect::get(element.as_ref(), &wasm_bindgen::JsValue::from_str("focus"))
    else {
        return;
    };
    let Ok(focus) = value.dyn_into::<js_sys::Function>() else {
        return;
    };
    let _ = focus.call0(element.as_ref());
}

/// Native reducer tests do not have an SVG document to focus.
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(super) fn focus_svg_element(_id: &str) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::line_chart::normalize::{NormalizedChart, normalize_categorical};
    use crate::charts::{
        LineCategory, LineChartActivationSource, LineChartModifiers, LinePoint, LineSeries,
    };

    fn chart(values: &[(Option<f64>, Option<f64>)]) -> NormalizedChart {
        let categories = values
            .iter()
            .enumerate()
            .map(|(index, _)| LineCategory {
                key: format!("category-{index}"),
                label: format!("Category {index}"),
            })
            .collect::<Vec<_>>();
        let first = values
            .iter()
            .map(|(value, _)| point(*value))
            .collect::<Vec<_>>();
        let second = values
            .iter()
            .map(|(_, value)| point(*value))
            .collect::<Vec<_>>();
        normalize_categorical(
            &categories,
            &[
                LineSeries::new("first", "First", "blue", first),
                LineSeries::new("second", "Second", "red", second),
            ],
        )
    }

    fn point(value: Option<f64>) -> LinePoint {
        value.map(LinePoint::new).unwrap_or_else(LinePoint::missing)
    }

    fn active(category_index: usize, preferred_series_index: Option<usize>) -> ActivePoint {
        ActivePoint {
            category_index,
            preferred_series_index,
        }
    }

    #[test]
    fn hover_wins_until_pointer_leaves_then_focus_resumes() {
        let chart = chart(&[(Some(1.0), Some(2.0)), (Some(3.0), Some(4.0))]);
        let focused = reduce(
            &InteractionState::default(),
            InteractionAction::Focused(active(0, Some(0))),
            &chart,
            &chart,
        );
        let hovered = reduce(
            &focused,
            InteractionAction::PointerMoved(active(1, Some(1))),
            &chart,
            &chart,
        );
        assert_eq!(displayed_active(&hovered, &chart), Some(active(1, Some(1))));

        let left = reduce(&hovered, InteractionAction::PointerLeft, &chart, &chart);
        assert_eq!(displayed_active(&left, &chart), Some(active(0, Some(0))));
    }

    #[test]
    fn blur_hides_focus_card_but_preserves_roving_tab_stop() {
        let chart = chart(&[(Some(1.0), None), (Some(2.0), None)]);
        let focused = reduce(
            &InteractionState::default(),
            InteractionAction::Focused(active(1, Some(0))),
            &chart,
            &chart,
        );
        let blurred = reduce(&focused, InteractionAction::Blurred, &chart, &chart);

        assert_eq!(blurred.roving_category_key.as_deref(), Some("category-1"));
        assert_eq!(displayed_active(&blurred, &chart), None);
    }

    #[test]
    fn category_navigation_skips_gaps_and_clamps_at_edges() {
        let chart = chart(&[(Some(1.0), None), (None, None), (Some(3.0), Some(4.0))]);
        let start = InteractionState {
            focused: Some(active(0, Some(0))),
            roving_category_key: Some("category-0".into()),
            ..Default::default()
        };
        let right = reduce(
            &start,
            InteractionAction::MoveFocus(NavigationKey::Right),
            &chart,
            &chart,
        );
        assert_eq!(right.focused, Some(active(2, Some(0))));
        let clamped = reduce(
            &right,
            InteractionAction::MoveFocus(NavigationKey::Right),
            &chart,
            &chart,
        );
        assert_eq!(clamped.focused, Some(active(2, Some(0))));

        let left = reduce(
            &start,
            InteractionAction::MoveFocus(NavigationKey::Left),
            &chart,
            &chart,
        );
        assert_eq!(left.focused, Some(active(0, Some(0))));
    }

    #[test]
    fn home_and_end_jump_to_first_and_last_finite_categories() {
        let chart = chart(&[
            (None, None),
            (Some(2.0), None),
            (None, None),
            (None, Some(4.0)),
        ]);
        let state = InteractionState::default();

        let home = reduce(
            &state,
            InteractionAction::MoveFocus(NavigationKey::Home),
            &chart,
            &chart,
        );
        let end = reduce(
            &state,
            InteractionAction::MoveFocus(NavigationKey::End),
            &chart,
            &chart,
        );
        assert_eq!(home.focused, Some(active(1, Some(0))));
        assert_eq!(end.focused, Some(active(3, Some(1))));
    }

    #[test]
    fn up_and_down_wrap_through_finite_series_at_the_active_category() {
        let chart = chart(&[(Some(1.0), Some(2.0)), (None, Some(4.0))]);
        let state = InteractionState {
            focused: Some(active(0, Some(0))),
            roving_category_key: Some("category-0".into()),
            ..Default::default()
        };
        let down = reduce(
            &state,
            InteractionAction::MoveFocus(NavigationKey::Down),
            &chart,
            &chart,
        );
        assert_eq!(down.focused, Some(active(0, Some(1))));
        let wrapped = reduce(
            &down,
            InteractionAction::MoveFocus(NavigationKey::Down),
            &chart,
            &chart,
        );
        assert_eq!(wrapped.focused, Some(active(0, Some(0))));
        let up = reduce(
            &state,
            InteractionAction::MoveFocus(NavigationKey::Up),
            &chart,
            &chart,
        );
        assert_eq!(up.focused, Some(active(0, Some(1))));
    }

    #[test]
    fn escape_is_idempotent_and_requires_pointer_reentry_or_a_real_focus_move() {
        let chart = chart(&[(Some(1.0), Some(2.0)), (Some(3.0), Some(4.0))]);
        let state = reduce(
            &InteractionState::default(),
            InteractionAction::Focused(active(0, Some(0))),
            &chart,
            &chart,
        );
        let dismissed = reduce(&state, InteractionAction::Dismiss, &chart, &chart);
        assert_eq!(displayed_active(&dismissed, &chart), None);

        let dismissed_again = reduce(&dismissed, InteractionAction::Dismiss, &chart, &chart);
        assert_eq!(displayed_active(&dismissed_again, &chart), None);

        let pointer_moved = reduce(
            &dismissed_again,
            InteractionAction::PointerMoved(active(0, Some(1))),
            &chart,
            &chart,
        );
        assert_eq!(displayed_active(&pointer_moved, &chart), None);

        let pointer_entered = reduce(
            &pointer_moved,
            InteractionAction::PointerEntered,
            &chart,
            &chart,
        );
        assert_eq!(
            displayed_active(&pointer_entered, &chart),
            Some(active(0, Some(1)))
        );

        let dismissed_for_focus = reduce(&state, InteractionAction::Dismiss, &chart, &chart);

        let moved = reduce(
            &dismissed_for_focus,
            InteractionAction::MoveFocus(NavigationKey::Right),
            &chart,
            &chart,
        );
        assert_eq!(displayed_active(&moved, &chart), Some(active(1, Some(0))));
    }

    #[test]
    fn escape_stays_hidden_when_hover_leaves_to_an_older_focused_category() {
        let chart = chart(&[(Some(1.0), Some(2.0)), (Some(3.0), Some(4.0))]);
        let focused = reduce(
            &InteractionState::default(),
            InteractionAction::Focused(active(0, Some(0))),
            &chart,
            &chart,
        );
        let hovered = reduce(
            &focused,
            InteractionAction::PointerMoved(active(1, Some(1))),
            &chart,
            &chart,
        );
        let dismissed = reduce(&hovered, InteractionAction::Dismiss, &chart, &chart);
        let pointer_left = reduce(&dismissed, InteractionAction::PointerLeft, &chart, &chart);

        assert_eq!(pointer_left.focused, Some(active(0, Some(0))));
        assert_eq!(displayed_active(&pointer_left, &chart), None);
    }

    #[test]
    fn replacement_of_a_dismissed_handover_category_clears_active_state_without_reopening_focus() {
        let previous = chart(&[(Some(1.0), Some(2.0)), (Some(3.0), Some(4.0))]);
        let next = normalize_categorical(
            &[LineCategory {
                key: "category-0".into(),
                label: "Category 0".into(),
            }],
            &[LineSeries::new(
                "first",
                "First",
                "blue",
                vec![LinePoint::new(10.0)],
            )],
        );
        let focused = reduce(
            &InteractionState::default(),
            InteractionAction::Focused(active(0, Some(0))),
            &previous,
            &previous,
        );
        let hovered = reduce(
            &focused,
            InteractionAction::PointerMoved(active(1, Some(1))),
            &previous,
            &previous,
        );
        let dismissed = reduce(&hovered, InteractionAction::Dismiss, &previous, &previous);
        let pointer_left = reduce(
            &dismissed,
            InteractionAction::PointerLeft,
            &previous,
            &previous,
        );

        let reconciled = reduce(
            &pointer_left,
            InteractionAction::ReconcileData,
            &previous,
            &next,
        );
        assert_eq!(reconciled.hovered, None);
        assert_eq!(reconciled.focused, None);
        assert_eq!(reconciled.dismissed_category_key, None);
        assert_eq!(displayed_active(&reconciled, &next), None);
    }

    #[test]
    fn stale_focus_actions_are_inert_and_preserve_existing_roving_and_dismissal_state() {
        let chart = chart(&[(Some(1.0), None), (None, None)]);
        let state = InteractionState {
            focused: Some(active(0, Some(0))),
            roving_category_key: Some("category-0".into()),
            dismissed_category_key: Some("category-0".into()),
            ..Default::default()
        };

        for invalid in [active(1, Some(0)), active(99, Some(99))] {
            let after = reduce(&state, InteractionAction::Focused(invalid), &chart, &chart);
            assert_eq!(after, state);
        }
    }

    #[test]
    fn no_op_navigation_does_not_reopen_a_dismissed_card() {
        let chart = chart(&[(Some(1.0), None)]);
        let state = InteractionState {
            focused: Some(active(0, Some(0))),
            roving_category_key: Some("category-0".into()),
            ..Default::default()
        };
        let dismissed = reduce(&state, InteractionAction::Dismiss, &chart, &chart);
        let same_focus = reduce(
            &dismissed,
            InteractionAction::Focused(active(0, Some(0))),
            &chart,
            &chart,
        );
        assert_eq!(displayed_active(&same_focus, &chart), None);
        for key in [
            NavigationKey::Left,
            NavigationKey::Right,
            NavigationKey::Home,
            NavigationKey::End,
            NavigationKey::Up,
            NavigationKey::Down,
        ] {
            let after = reduce(
                &dismissed,
                InteractionAction::MoveFocus(key),
                &chart,
                &chart,
            );
            assert_eq!(
                displayed_active(&after, &chart),
                None,
                "{key:?} must not clear Escape without a real focus move"
            );
        }
    }

    #[test]
    fn activation_preserves_series_order_preferred_series_source_and_modifiers() {
        let chart = chart(&[(Some(1.0), Some(2.0))]);
        let modifiers = LineChartModifiers {
            shift: true,
            ctrl: true,
            alt: false,
            meta: true,
        };

        let activation = activation_for(
            &chart,
            active(0, Some(1)),
            LineChartActivationSource::Pointer,
            modifiers.clone(),
        )
        .expect("category has values");
        assert_eq!(
            activation
                .values
                .iter()
                .map(|value| value.series_id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second"]
        );
        assert_eq!(activation.preferred_series_id.as_deref(), Some("second"));
        assert_eq!(activation.source, LineChartActivationSource::Pointer);
        assert_eq!(activation.modifiers, modifiers);
        assert_eq!(activation.values[0].display_value, "1");
        assert_eq!(activation.values[1].display_value, "2");
    }

    #[test]
    fn keyboard_activation_falls_back_to_first_finite_series_and_empty_category_is_inert() {
        let chart = chart(&[(None, Some(2.0)), (None, None)]);
        let keyboard = activation_for(
            &chart,
            active(0, Some(0)),
            LineChartActivationSource::Keyboard,
            LineChartModifiers::default(),
        )
        .expect("category has a finite value");
        assert_eq!(keyboard.preferred_series_id.as_deref(), Some("second"));
        assert!(
            activation_for(
                &chart,
                active(1, None),
                LineChartActivationSource::Keyboard,
                LineChartModifiers::default(),
            )
            .is_none()
        );
    }

    #[test]
    fn stale_active_points_are_inert_and_do_not_survive_pointer_updates() {
        let chart = chart(&[(Some(1.0), Some(2.0))]);
        assert!(
            activation_for(
                &chart,
                active(99, Some(99)),
                LineChartActivationSource::Pointer,
                LineChartModifiers::default(),
            )
            .is_none()
        );

        let state = reduce(
            &InteractionState::default(),
            InteractionAction::PointerMoved(active(99, Some(99))),
            &chart,
            &chart,
        );
        assert_eq!(state.hovered, None);
    }

    #[test]
    fn reconciliation_uses_first_duplicate_key_and_nearest_valid_fallback_without_activation() {
        let previous = chart(&[(Some(1.0), None), (Some(2.0), None), (Some(3.0), None)]);
        let next = normalize_categorical(
            &[
                LineCategory {
                    key: "category-1".into(),
                    label: "First duplicate".into(),
                },
                LineCategory {
                    key: "category-1".into(),
                    label: "Second duplicate".into(),
                },
                LineCategory {
                    key: "category-2".into(),
                    label: "Category 2".into(),
                },
            ],
            &[LineSeries::new(
                "first",
                "First",
                "blue",
                vec![
                    LinePoint::new(20.0),
                    LinePoint::new(21.0),
                    LinePoint::new(30.0),
                ],
            )],
        );
        let state = InteractionState {
            hovered: Some(active(1, Some(0))),
            focused: Some(active(1, Some(0))),
            roving_category_key: Some("category-1".into()),
            dismissed_category_key: Some("category-1".into()),
        };
        let retained = reduce(&state, InteractionAction::ReconcileData, &previous, &next);
        assert_eq!(retained.hovered, Some(active(0, Some(0))));
        assert_eq!(retained.focused, Some(active(0, Some(0))));
        assert_eq!(retained.roving_category_key.as_deref(), Some("category-1"));
        assert_eq!(
            retained.dismissed_category_key.as_deref(),
            Some("category-1")
        );

        let removed = reduce(
            &state,
            InteractionAction::ReconcileData,
            &previous,
            &chart(&[(None, None), (None, None), (Some(30.0), None)]),
        );
        assert_eq!(removed.hovered, None);
        assert_eq!(removed.focused, None);
        assert_eq!(removed.dismissed_category_key, None);
        assert_eq!(removed.roving_category_key.as_deref(), Some("category-2"));
    }

    #[test]
    fn reconciliation_removes_hidden_focus_when_the_raw_hovered_category_disappears() {
        let previous = chart(&[(Some(1.0), None), (Some(2.0), None), (Some(3.0), None)]);
        let next = normalize_categorical(
            &[
                LineCategory {
                    key: "category-0".into(),
                    label: "Category 0".into(),
                },
                LineCategory {
                    key: "category-2".into(),
                    label: "Category 2".into(),
                },
            ],
            &[LineSeries::new(
                "first",
                "First",
                "blue",
                vec![LinePoint::new(10.0), LinePoint::new(30.0)],
            )],
        );
        let state = InteractionState {
            hovered: Some(active(1, Some(0))),
            focused: Some(active(0, Some(0))),
            roving_category_key: Some("category-0".into()),
            dismissed_category_key: Some("category-1".into()),
        };

        let reconciled = reduce(&state, InteractionAction::ReconcileData, &previous, &next);
        assert_eq!(reconciled.hovered, None);
        assert_eq!(reconciled.focused, None);
        assert_eq!(reconciled.dismissed_category_key, None);
        assert_eq!(
            reconciled.roving_category_key.as_deref(),
            Some("category-2")
        );
        assert_eq!(displayed_active(&reconciled, &next), None);
    }

    #[test]
    fn nearest_reconciliation_ties_choose_the_lower_category_index() {
        let previous = chart(&[
            (Some(1.0), None),
            (Some(2.0), None),
            (Some(3.0), None),
            (Some(4.0), None),
            (Some(5.0), None),
        ]);
        let next = chart(&[
            (Some(10.0), None),
            (None, None),
            (None, None),
            (None, None),
            (Some(50.0), None),
        ]);
        let state = InteractionState {
            focused: Some(active(2, Some(0))),
            roving_category_key: Some("category-2".into()),
            ..Default::default()
        };

        let reconciled = reduce(&state, InteractionAction::ReconcileData, &previous, &next);
        assert_eq!(
            reconciled.roving_category_key.as_deref(),
            Some("category-0")
        );
    }

    #[test]
    fn reconciliation_uses_the_first_matching_duplicate_series_id() {
        let categories = [LineCategory {
            key: "only".into(),
            label: "Only".into(),
        }];
        let previous = normalize_categorical(
            &categories,
            &[
                LineSeries::new("duplicate", "First", "blue", vec![LinePoint::new(1.0)]),
                LineSeries::new("duplicate", "Second", "red", vec![LinePoint::new(2.0)]),
            ],
        );
        let next = normalize_categorical(
            &categories,
            &[
                LineSeries::new("duplicate", "First", "blue", vec![LinePoint::new(10.0)]),
                LineSeries::new("duplicate", "Second", "red", vec![LinePoint::new(20.0)]),
            ],
        );
        let state = InteractionState {
            focused: Some(active(0, Some(1))),
            roving_category_key: Some("only".into()),
            ..Default::default()
        };

        let reconciled = reduce(&state, InteractionAction::ReconcileData, &previous, &next);
        assert_eq!(reconciled.focused, Some(active(0, Some(0))));
    }
}
