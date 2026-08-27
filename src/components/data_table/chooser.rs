//! Column show/hide chooser for [`DataTable`](super::component::DataTable).
//!
//! A gear-icon [`Dropdown`] holding one [`MenuCheckItem`] per column lets the
//! user toggle which columns are visible. The choice persists per-table to
//! `localStorage` (keyed by the caller's `chooser_key`), drift-safe: ids that no
//! longer exist in the column set are dropped on load.
//!
//! The pure helpers below carry the logic (which columns are visible, whether a
//! toggle is allowed, (de)serialization) so it is unit-testable without a DOM;
//! [`DataTableColumnChooser`] is the thin reactive shell over them.

use crate::components::data_table::types::Column;
use crate::components::dropdown::{Dropdown, DropdownContent};
use crate::components::icon::Icon;
use crate::components::menu::{Menu, MenuCheckItem};
use leptos::prelude::*;
use std::collections::HashSet;

/// `localStorage` key prefix for a table's hidden-column set. The caller's
/// `chooser_key` is appended.
pub(crate) const CHOOSER_STORAGE_PREFIX: &str = "ldui-dt-cols:";

/// The normalized state transition for a column-visibility action.
///
/// Both `DataTable` and `EntityTable` use this policy so required columns and
/// the last visible column cannot diverge between their chooser renderers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColumnVisibilityAction {
    /// Reveal a currently hidden column.
    Show,
    /// Hide a currently visible optional column.
    Hide,
    /// Refuse the action because the column is required or is the last one.
    Unchanged,
}

/// Chooses one visibility transition from current state and column policy.
pub(crate) const fn column_visibility_action(
    is_hidden: bool,
    is_required: bool,
    visible_count: usize,
) -> ColumnVisibilityAction {
    if is_hidden {
        ColumnVisibilityAction::Show
    } else if is_required || visible_count <= 1 {
        ColumnVisibilityAction::Unchanged
    } else {
        ColumnVisibilityAction::Hide
    }
}

/// The columns that remain visible: every column whose id is not in `hidden`,
/// in their original order.
pub(crate) fn visible_columns(columns: &[Column], hidden: &HashSet<&'static str>) -> Vec<Column> {
    columns
        .iter()
        .filter(|c| !hidden.contains(c.id))
        .cloned()
        .collect()
}

/// Whether `id` is currently visible AND the only visible column. Its
/// check-item is disabled in that state so the user can never hide every
/// column (which would leave an empty, headerless table).
pub(crate) fn is_only_visible(
    columns: &[Column],
    hidden: &HashSet<&'static str>,
    id: &str,
) -> bool {
    let Some(column) = columns.iter().find(|column| column.id == id) else {
        return false;
    };
    let is_hidden = hidden.contains(column.id);
    let visible_count = columns
        .iter()
        .filter(|candidate| !hidden.contains(candidate.id))
        .count();
    !is_hidden
        && column_visibility_action(is_hidden, false, visible_count)
            == ColumnVisibilityAction::Unchanged
}

/// Applies the shared visibility transition to DataTable's borrowed-id set.
///
/// The transition is guarded in state as well as in the disabled menu item,
/// so a stale or synthetic event cannot hide the last visible column.
pub(crate) fn toggle_hidden_column(
    columns: &[Column],
    hidden: &mut HashSet<&'static str>,
    id: &'static str,
) -> bool {
    let Some(column) = columns.iter().find(|column| column.id == id) else {
        return false;
    };
    let is_hidden = hidden.contains(column.id);
    let visible_count = columns
        .iter()
        .filter(|candidate| !hidden.contains(candidate.id))
        .count();
    match column_visibility_action(is_hidden, false, visible_count) {
        ColumnVisibilityAction::Show => hidden.remove(column.id),
        ColumnVisibilityAction::Hide => hidden.insert(column.id),
        ColumnVisibilityAction::Unchanged => false,
    }
}

/// Serialize a hidden set to a stable, comma-separated string for storage.
/// Ids are sorted so the persisted value is deterministic.
pub(crate) fn serialize_hidden(hidden: &HashSet<&'static str>) -> String {
    let mut ids: Vec<&'static str> = hidden.iter().copied().collect();
    ids.sort_unstable();
    ids.join(",")
}

/// Parse a stored comma-separated string back into a hidden set, keeping only
/// ids that still exist in `valid_ids`. Drops unknown ids so a renamed/removed
/// column can't leave a phantom entry (mirrors the key-resolution guards
/// elsewhere in DataTable).
pub(crate) fn parse_hidden(stored: &str, valid_ids: &[&'static str]) -> HashSet<&'static str> {
    let wanted: HashSet<&str> = stored.split(',').filter(|s| !s.is_empty()).collect();
    valid_ids
        .iter()
        .copied()
        .filter(|id| wanted.contains(id))
        .collect()
}

/// Gear-icon dropdown listing every column with a checkbox to toggle its
/// visibility. Fully controlled: the caller owns the `hidden` set and this
/// component reads `columns` reactively.
#[component]
pub fn DataTableColumnChooser(
    /// Full column list (unfiltered).
    #[prop(into)]
    columns: Signal<Vec<Column>>,
    /// The set of hidden column ids. Toggled in place as the user ticks items.
    hidden: RwSignal<HashSet<&'static str>>,
) -> impl IntoView {
    view! {
        <Dropdown class="dropdown-end" placement=crate::components::dropdown::DropdownPlacement::Bottom>
            <div
                tabindex="0"
                role="button"
                aria-label="Choose columns"
                class="btn btn-ghost btn-sm btn-square"
            >
                <Icon name="settings" />
            </div>
            <DropdownContent class="bg-base-100 rounded-box z-[1] w-56 p-0 shadow-lg border border-base-300">
                <Menu class="w-full">
                    {move || {
                        columns
                            .get()
                            .into_iter()
                            .map(|col| {
                                let id = col.id;
                                let checked = Signal::derive(move || {
                                    !hidden.with(|h| h.contains(id))
                                });
                                let disabled = Signal::derive(move || {
                                    columns.with(|cols| {
                                        hidden.with(|h| is_only_visible(cols, h, id))
                                    })
                                });
                                let on_toggle = Callback::new(move |_now_checked: bool| {
                                    columns.with_untracked(|columns| {
                                        hidden.update(|hidden| {
                                            toggle_hidden_column(columns, hidden, id);
                                        });
                                    });
                                });
                                view! {
                                    <MenuCheckItem
                                        checked=checked
                                        disabled=disabled
                                        on_toggle=on_toggle
                                    >
                                        {col.header.clone()}
                                    </MenuCheckItem>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </Menu>
            </DropdownContent>
        </Dropdown>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_table::types::Column;

    fn cols() -> Vec<Column> {
        vec![
            Column::new("id", "ID"),
            Column::new("name", "Name"),
            Column::new("balance", "Balance"),
        ]
    }

    #[test]
    fn shared_visibility_policy_covers_show_hide_and_guarded_noop() {
        assert_eq!(
            column_visibility_action(true, false, 1),
            ColumnVisibilityAction::Show
        );
        assert_eq!(
            column_visibility_action(false, false, 2),
            ColumnVisibilityAction::Hide
        );
        assert_eq!(
            column_visibility_action(false, false, 1),
            ColumnVisibilityAction::Unchanged
        );
        assert_eq!(
            column_visibility_action(false, true, 3),
            ColumnVisibilityAction::Unchanged
        );
    }

    #[test]
    fn data_table_toggle_uses_shared_policy_even_without_the_disabled_ui_guard() {
        let columns = cols();
        let mut hidden = HashSet::new();

        assert!(toggle_hidden_column(&columns, &mut hidden, "name"));
        assert!(hidden.contains("name"));
        assert!(toggle_hidden_column(&columns, &mut hidden, "name"));
        assert!(!hidden.contains("name"));

        hidden.extend(["name", "balance"]);
        assert!(!toggle_hidden_column(&columns, &mut hidden, "id"));
        assert!(
            !hidden.contains("id"),
            "the last visible column stays visible"
        );
        assert!(!toggle_hidden_column(&columns, &mut hidden, "missing"));
    }

    // ── visible_columns ──

    #[test]
    fn visible_columns_empty_hidden_returns_all() {
        let hidden = HashSet::new();
        let v = visible_columns(&cols(), &hidden);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].id, "id");
        assert_eq!(v[2].id, "balance");
    }

    #[test]
    fn visible_columns_drops_hidden_preserving_order() {
        let hidden: HashSet<&'static str> = ["name"].into_iter().collect();
        let v = visible_columns(&cols(), &hidden);
        assert_eq!(
            v.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec!["id", "balance"]
        );
    }

    #[test]
    fn visible_columns_all_hidden_returns_empty() {
        let hidden: HashSet<&'static str> = ["id", "name", "balance"].into_iter().collect();
        assert!(visible_columns(&cols(), &hidden).is_empty());
    }

    // ── is_only_visible (the "can't hide the last one" guard) ──

    #[test]
    fn is_only_visible_false_when_many_visible() {
        let hidden = HashSet::new();
        assert!(!is_only_visible(&cols(), &hidden, "id"));
    }

    #[test]
    fn is_only_visible_true_for_sole_survivor() {
        let hidden: HashSet<&'static str> = ["id", "balance"].into_iter().collect();
        assert!(is_only_visible(&cols(), &hidden, "name"));
    }

    #[test]
    fn is_only_visible_false_for_already_hidden_column() {
        // "id" is hidden; the sole visible column is "name" — asking about the
        // hidden "id" must be false (it isn't visible at all).
        let hidden: HashSet<&'static str> = ["id", "balance"].into_iter().collect();
        assert!(!is_only_visible(&cols(), &hidden, "id"));
    }

    // ── serialize / parse round-trip ──

    #[test]
    fn serialize_hidden_is_sorted_and_comma_joined() {
        let hidden: HashSet<&'static str> = ["name", "id"].into_iter().collect();
        assert_eq!(serialize_hidden(&hidden), "id,name");
    }

    #[test]
    fn serialize_empty_hidden_is_empty_string() {
        assert_eq!(serialize_hidden(&HashSet::new()), "");
    }

    #[test]
    fn parse_hidden_keeps_only_valid_ids() {
        let valid = ["id", "name", "balance"];
        let got = parse_hidden("name,gone,balance", &valid);
        let expected: HashSet<&'static str> = ["name", "balance"].into_iter().collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn parse_hidden_empty_string_is_empty_set() {
        let valid = ["id", "name"];
        assert!(parse_hidden("", &valid).is_empty());
    }

    #[test]
    fn serialize_parse_round_trips() {
        let valid = ["id", "name", "balance"];
        let hidden: HashSet<&'static str> = ["balance", "id"].into_iter().collect();
        let round = parse_hidden(&serialize_hidden(&hidden), &valid);
        assert_eq!(round, hidden);
    }
}
