//! Opinionated, optional presentation tools for [`ServerDataTable`](super::ServerDataTable)
//! (ldui-9j16): a compact gear column chooser, a caller toolbar-actions slot
//! placed beside it, and stable column visibility/order preferences. Every
//! piece of this module reuses `EntityTable`'s preference model and pure
//! column-order/visibility functions -- the types genuinely match, since
//! neither the preference payload nor the ordering/visibility algorithms
//! depend on a table's row type.
//!
//! Query ownership (paging, search, sort, filters, cursor tokens) stays
//! entirely with `ServerDataTable`/its caller; nothing here ever reads or
//! writes a `TableQuery`, `ServerCursorQuery`, or fetches a row.
//!
//! # The one-page-CSV mistake this module makes impossible
//!
//! [`ServerTableDisplayedSlice`] is deliberately NOT [`EntityTableDisplayProjection`](crate::components::entity_table::EntityTableDisplayProjection):
//! it carries no `AllFiltered` scope, no dataset identity, and no way to ask
//! for "every row" -- because a server-paginated table never holds every row.
//! Its doc comment says so explicitly, and there is no code path that could
//! silently promote a page to a population.

use crate::components::data_table::types::{Column, TableRow};
use crate::components::entity_table::{
    EntityColumn, EntityColumnChooserTrigger, EntityColumnMove, EntityTablePreferenceOwnership,
    EntityTablePreferencePersistence, EntityTablePreferences, decode_preferences,
    encode_preferences, move_column, normalize_preferences, ordered_columns, reset_columns,
    toggle_hidden_column,
};
use leptos::prelude::*;

/// `localStorage` key prefix for [`ServerTableColumnTools`]'s uncontrolled
/// persistence. Deliberately distinct from `EntityTable`'s own prefix so the
/// two families never collide under one caller-chosen namespace.
const SERVER_COLUMN_TOOLS_STORAGE_PREFIX: &str = "ldui-server-table-cols:";

/// Localizable copy for [`ServerTableColumnTools`]'s chooser panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableColumnToolsTexts {
    /// Accessible label for the column-chooser trigger.
    pub choose_columns: String,
    /// Visible and accessible label for the column-order list.
    pub column_order: String,
    /// Move-earlier template with `{column}`, `{position}`, and `{total}`.
    pub move_earlier: String,
    /// Move-later template with `{column}`, `{position}`, and `{total}`.
    pub move_later: String,
    /// Action label that restores default column visibility and order.
    pub reset_columns: String,
}

impl Default for ServerTableColumnToolsTexts {
    fn default() -> Self {
        Self {
            choose_columns: "Choose columns".to_owned(),
            column_order: "Column order".to_owned(),
            move_earlier: "Move {column} earlier from position {position} of {total}".to_owned(),
            move_later: "Move {column} later from position {position} of {total}".to_owned(),
            reset_columns: "Reset columns".to_owned(),
        }
    }
}

/// Opt-in [`ServerDataTable`](super::ServerDataTable) presentation tools:
/// the compact gear column chooser, a toolbar-actions slot rendered beside
/// it (the natural home for a caller's Export action), and stable
/// visibility/order preferences.
///
/// Omitting this from `ServerDataTable` keeps the historical behavior
/// exactly: no chooser, no toolbar slot, no preference storage, and no
/// [`ServerTableDisplayedSlice`] projection.
pub struct ServerTableColumnTools {
    pub(crate) preference_ownership: EntityTablePreferenceOwnership,
    pub(crate) schema_version: u16,
    pub(crate) chooser_trigger: Signal<EntityColumnChooserTrigger>,
    pub(crate) texts: Signal<ServerTableColumnToolsTexts>,
    pub(crate) toolbar_actions: Option<Children>,
}

impl ServerTableColumnTools {
    /// Creates column tools under explicit preference ownership and a
    /// caller-chosen schema version used to invalidate incompatible stored
    /// payloads (mirrors `EntityTable`'s `preference_version`).
    pub fn new(preference_ownership: EntityTablePreferenceOwnership, schema_version: u16) -> Self {
        Self {
            preference_ownership,
            schema_version,
            chooser_trigger: Signal::stored(EntityColumnChooserTrigger::default()),
            texts: Signal::stored(ServerTableColumnToolsTexts::default()),
            toolbar_actions: None,
        }
    }

    /// Replaces the chooser trigger's visible presentation.
    pub fn with_chooser_trigger(
        mut self,
        trigger: impl Into<Signal<EntityColumnChooserTrigger>>,
    ) -> Self {
        self.chooser_trigger = trigger.into();
        self
    }

    /// Replaces the chooser panel's localized copy.
    pub fn with_texts(mut self, texts: impl Into<Signal<ServerTableColumnToolsTexts>>) -> Self {
        self.texts = texts.into();
        self
    }

    /// Supplies caller-rendered actions (typically an Export button) placed
    /// immediately beside the chooser trigger. The table owns placement and
    /// wrapping; the caller owns all behavior and authorization -- the
    /// renderer receives no rows, dataset, or projection of its own (read
    /// the displayed slice from `ServerDataTable`'s `on_displayed_slice`
    /// instead). Mirrors `SnapshotTablePage::with_toolbar_actions`'s
    /// ergonomic plain-closure call style.
    pub fn with_toolbar_actions(
        mut self,
        render: impl FnOnce() -> AnyView + Send + 'static,
    ) -> Self {
        self.toolbar_actions = Some(Box::new(render));
        self
    }
}

/// Builds metadata-only `EntityColumn` values purely to drive the reused
/// `EntityTable` preference/ordering/visibility functions -- `text`/render
/// closures are never invoked because `ServerDataTable` renders its own
/// `Column`-based cells and never local-sorts or renders through this type.
pub(crate) fn server_column_tools_entity_columns(
    columns: &[Column],
) -> Vec<EntityColumn<TableRow>> {
    columns
        .iter()
        .map(|column| {
            let mut entity_column = EntityColumn::<TableRow>::new(
                column.id,
                column.header.clone(),
                |_row: &TableRow| String::new(),
            );
            if column.required {
                entity_column = entity_column.required();
            }
            entity_column
        })
        .collect()
}

/// Reorders and hides columns per normalized preferences. Required columns
/// can never be hidden -- enforced by `normalize_preferences` itself, which
/// strips any required id out of `hidden_columns` before this ever runs.
pub(crate) fn apply_column_tools_presentation(
    columns: Vec<Column>,
    preferences: &EntityTablePreferences,
) -> Vec<Column> {
    let entity_columns = server_column_tools_entity_columns(&columns);
    ordered_columns(preferences, &entity_columns)
        .into_iter()
        .filter(|entity_column| !preferences.hidden_columns.contains(entity_column.id))
        .filter_map(|entity_column| {
            columns
                .iter()
                .find(|column| column.id == entity_column.id)
                .cloned()
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_storage() -> Option<web_sys::Storage> {
    None
}

pub(crate) fn load_column_tools_preferences(
    persistence: EntityTablePreferencePersistence,
    schema_version: u16,
    columns: &[EntityColumn<TableRow>],
) -> EntityTablePreferences {
    let EntityTablePreferencePersistence::LegacyLocalStorage { storage_key } = persistence else {
        return EntityTablePreferences::new(schema_version);
    };
    let key = format!("{SERVER_COLUMN_TOOLS_STORAGE_PREFIX}{storage_key}");
    browser_storage()
        .and_then(|storage| storage.get_item(&key).ok().flatten())
        .map(|payload| decode_preferences(&payload, schema_version, columns))
        .unwrap_or_else(|| EntityTablePreferences::new(schema_version))
}

pub(crate) fn save_column_tools_preferences(
    persistence: EntityTablePreferencePersistence,
    preferences: &EntityTablePreferences,
) {
    let EntityTablePreferencePersistence::LegacyLocalStorage { storage_key } = persistence else {
        return;
    };
    let Ok(payload) = encode_preferences(preferences) else {
        return;
    };
    let key = format!("{SERVER_COLUMN_TOOLS_STORAGE_PREFIX}{storage_key}");
    if let Some(storage) = browser_storage() {
        let _ = storage.set_item(&key, &payload);
    }
}

/// Who supplies [`ServerTableColumnTools`]'s current preference value.
#[derive(Clone, Copy)]
pub(crate) enum ServerColumnToolsSource {
    Controlled {
        current: Signal<EntityTablePreferences>,
        on_change: Callback<EntityTablePreferences>,
    },
    Uncontrolled {
        current: RwSignal<EntityTablePreferences>,
        persistence: EntityTablePreferencePersistence,
    },
}

/// Normalized read/update access over one `ServerTableColumnTools`
/// preference value, keyed against the server table's own reactive columns.
#[derive(Clone, Copy)]
pub(crate) struct ServerColumnToolsState {
    pub(crate) source: ServerColumnToolsSource,
    schema_version: u16,
    columns: Signal<Vec<Column>>,
}

impl ServerColumnToolsState {
    pub(crate) fn new(
        ownership: EntityTablePreferenceOwnership,
        schema_version: u16,
        columns: Signal<Vec<Column>>,
    ) -> Self {
        let source = match ownership {
            EntityTablePreferenceOwnership::Controlled { current, on_change } => {
                ServerColumnToolsSource::Controlled { current, on_change }
            }
            EntityTablePreferenceOwnership::Uncontrolled { persistence } => {
                let entity_columns = server_column_tools_entity_columns(&columns.get_untracked());
                let initial =
                    load_column_tools_preferences(persistence, schema_version, &entity_columns);
                ServerColumnToolsSource::Uncontrolled {
                    current: RwSignal::new(initial),
                    persistence,
                }
            }
        };
        Self {
            source,
            schema_version,
            columns,
        }
    }

    pub(crate) fn entity_columns(self) -> Vec<EntityColumn<TableRow>> {
        server_column_tools_entity_columns(&self.columns.get())
    }

    fn entity_columns_untracked(self) -> Vec<EntityColumn<TableRow>> {
        server_column_tools_entity_columns(&self.columns.get_untracked())
    }

    /// Column ids in canonical preference order (every declared column,
    /// visible or hidden) -- what the chooser's reorder list iterates.
    pub(crate) fn ordered_column_ids(self) -> Vec<String> {
        let preferences = self.get();
        let entity_columns = self.entity_columns();
        ordered_columns(&preferences, &entity_columns)
            .into_iter()
            .map(|column| column.id.to_owned())
            .collect()
    }

    /// Current reactive header text for one column id, or an empty string
    /// if the id is no longer declared.
    pub(crate) fn header_for(self, column_id: &str) -> String {
        self.columns.with(|columns| {
            columns
                .iter()
                .find(|column| column.id == column_id)
                .map(|column| column.header.clone())
                .unwrap_or_default()
        })
    }

    pub(crate) fn get(self) -> EntityTablePreferences {
        let raw = match self.source {
            ServerColumnToolsSource::Controlled { current, .. } => current.get(),
            ServerColumnToolsSource::Uncontrolled { current, .. } => current.get(),
        };
        normalize_preferences(&raw, self.schema_version, &self.entity_columns())
    }

    pub(crate) fn get_untracked(self) -> EntityTablePreferences {
        let raw = match self.source {
            ServerColumnToolsSource::Controlled { current, .. } => current.get_untracked(),
            ServerColumnToolsSource::Uncontrolled { current, .. } => current.get_untracked(),
        };
        normalize_preferences(&raw, self.schema_version, &self.entity_columns_untracked())
    }

    pub(crate) fn update(self, mutate: impl FnOnce(&mut EntityTablePreferences)) {
        let mut next = self.get_untracked();
        mutate(&mut next);
        let normalized =
            normalize_preferences(&next, self.schema_version, &self.entity_columns_untracked());
        match self.source {
            ServerColumnToolsSource::Controlled { on_change, .. } => on_change.run(normalized),
            ServerColumnToolsSource::Uncontrolled { current, .. } => current.set(normalized),
        }
    }

    pub(crate) fn toggle_column(self, column_id: &str) {
        let entity_columns = self.entity_columns_untracked();
        self.update(|preferences| {
            toggle_hidden_column(preferences, &entity_columns, column_id);
        });
    }

    pub(crate) fn move_column(self, column_id: &str, direction: EntityColumnMove) {
        let entity_columns = self.entity_columns_untracked();
        self.update(|preferences| {
            move_column(preferences, &entity_columns, column_id, direction);
        });
    }

    pub(crate) fn reset(self) {
        self.update(|preferences| {
            reset_columns(preferences);
        });
    }
}

/// A boxed row-key extractor, used only to bridge `ServerDataTable`'s
/// `Callback<TableRow, String>` row_key prop into
/// [`server_table_displayed_slice`]'s borrowed-closure parameter.
pub(crate) type ServerRowKeyFn = Box<dyn Fn(&TableRow) -> String>;

/// Fills a move-earlier/move-later template's `{column}`, `{position}`, and
/// `{total}` placeholders.
pub(crate) fn format_server_move_label(
    template: &str,
    column: &str,
    position: usize,
    total: usize,
) -> String {
    template
        .replace("{column}", column)
        .replace("{position}", &position.to_string())
        .replace("{total}", &total.to_string())
}

/// One column in a [`ServerTableDisplayedSlice`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableDisplayColumn {
    /// Stable column identity.
    pub id: String,
    /// Current reactive/localized column label.
    pub label: String,
}

/// One row in a [`ServerTableDisplayedSlice`], with cells in the same order
/// as [`ServerTableDisplayedSlice::columns`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerTableDisplayRow {
    /// Stable row key from `ServerDataTable`'s `row_key`, when supplied.
    pub key: Option<String>,
    /// Canonical cell text aligned with the projection's ordered columns.
    pub cells: Vec<String>,
}

/// Atomic, read-only snapshot of exactly the server rows
/// [`ServerDataTable`](super::ServerDataTable) is currently displaying, in
/// currently visible column order.
///
/// # This is the current accepted server slice, never the complete result set
///
/// A server-paginated table holds only the one page or cursor slice its
/// caller fetched into `rows` -- there is no unseen population behind this
/// value to walk, count, or download, and `ServerDataTable` never fetches or
/// synthesizes a row to fill one in. Unlike
/// [`EntityTableDisplayProjection`](crate::components::entity_table::EntityTableDisplayProjection),
/// which offers an explicit `AllFiltered` scope over a complete client-side
/// snapshot, this type has no such scope and intentionally cannot grow one:
/// adding it would require inventing rows nobody supplied. Label any UI or
/// export built from this value against the visible page/slice (for example
/// "Export this page" or "Export current view"), never "Export all" or
/// "Export filtered" -- those claims are only true of a complete result set,
/// which this is not.
///
/// While a request is loading or has failed and `ServerDataTable` is
/// retaining the previously accepted rows (see `ServerCursorSliceState`),
/// this snapshot mirrors that retained data: `ServerDataTable` never mutates
/// `rows` itself, so the projection always reflects whatever the caller
/// currently has displayed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerTableDisplayedSlice {
    /// Ordered visible columns used by every row below.
    pub columns: Vec<ServerTableDisplayColumn>,
    /// Displayed rows in their current on-screen order.
    pub rows: Vec<ServerTableDisplayRow>,
}

/// Builds one atomic [`ServerTableDisplayedSlice`] from exactly what
/// `ServerDataTable` is currently rendering -- never anything else.
pub(crate) fn server_table_displayed_slice(
    columns: &[Column],
    rows: &[TableRow],
    row_key: Option<&dyn Fn(&TableRow) -> String>,
) -> ServerTableDisplayedSlice {
    let display_columns = columns
        .iter()
        .map(|column| ServerTableDisplayColumn {
            id: column.id.to_owned(),
            label: column.header.clone(),
        })
        .collect::<Vec<_>>();
    let display_rows = rows
        .iter()
        .map(|row| ServerTableDisplayRow {
            key: row_key.map(|key_of| key_of(row)),
            cells: columns
                .iter()
                .map(|column| row.get(column.id).cloned().unwrap_or_default())
                .collect(),
        })
        .collect();
    ServerTableDisplayedSlice {
        columns: display_columns,
        rows: display_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn columns() -> Vec<Column> {
        vec![
            Column::new("id", "ID").required(),
            Column::new("name", "Name"),
            Column::new("balance", "Balance"),
        ]
    }

    fn preferences() -> EntityTablePreferences {
        EntityTablePreferences::new(1)
    }

    // ── apply_column_tools_presentation ──

    #[test]
    fn default_preferences_keep_every_column_in_declared_order() {
        let result = apply_column_tools_presentation(columns(), &preferences());
        assert_eq!(
            result.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec!["id", "name", "balance"]
        );
    }

    #[test]
    fn hidden_optional_column_is_removed_from_presentation() {
        let mut prefs = preferences();
        prefs.hidden_columns.insert("name".to_owned());
        let result = apply_column_tools_presentation(columns(), &prefs);
        assert_eq!(
            result.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec!["id", "balance"]
        );
    }

    #[test]
    fn required_column_cannot_be_removed_even_via_a_crafted_preference_payload() {
        // A hand-crafted or foreign-schema payload naming a required column
        // in `hidden_columns` must never actually hide it: normalization is
        // the single point of truth (mirrors EntityTable's own guarantee).
        let mut prefs = preferences();
        prefs.hidden_columns.insert("id".to_owned());
        let entity_columns = server_column_tools_entity_columns(&columns());
        let normalized = normalize_preferences(&prefs, 1, &entity_columns);
        let result = apply_column_tools_presentation(columns(), &normalized);
        assert!(result.iter().any(|c| c.id == "id"));
    }

    #[test]
    fn column_order_preference_reorders_the_presentation() {
        let mut prefs = preferences();
        prefs.column_order = vec!["balance".to_owned(), "id".to_owned(), "name".to_owned()];
        let result = apply_column_tools_presentation(columns(), &prefs);
        assert_eq!(
            result.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec!["balance", "id", "name"]
        );
    }

    // ── ServerColumnToolsState pure behavior over toggle/move/reset ──

    #[test]
    fn toggle_column_hides_and_shows_an_optional_column() {
        let entity_columns = server_column_tools_entity_columns(&columns());
        let mut prefs = preferences();
        assert!(toggle_hidden_column(&mut prefs, &entity_columns, "name"));
        assert!(prefs.hidden_columns.contains("name"));
        assert!(toggle_hidden_column(&mut prefs, &entity_columns, "name"));
        assert!(!prefs.hidden_columns.contains("name"));
    }

    #[test]
    fn toggle_column_refuses_to_hide_a_required_column() {
        let entity_columns = server_column_tools_entity_columns(&columns());
        let mut prefs = preferences();
        assert!(!toggle_hidden_column(&mut prefs, &entity_columns, "id"));
        assert!(!prefs.hidden_columns.contains("id"));
    }

    #[test]
    fn move_column_reorders_by_one_position() {
        let entity_columns = server_column_tools_entity_columns(&columns());
        let mut prefs = preferences();
        assert!(move_column(
            &mut prefs,
            &entity_columns,
            "balance",
            EntityColumnMove::Earlier
        ));
        assert_eq!(
            apply_column_tools_presentation(columns(), &prefs)
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec!["id", "balance", "name"]
        );
    }

    #[test]
    fn reset_columns_clears_hidden_and_order_state() {
        let mut prefs = preferences();
        prefs.hidden_columns.insert("name".to_owned());
        prefs.column_order = vec!["balance".to_owned(), "id".to_owned(), "name".to_owned()];
        assert!(reset_columns(&mut prefs));
        assert!(prefs.hidden_columns.is_empty());
        assert!(prefs.column_order.is_empty());
    }

    // ── format_server_move_label ──

    #[test]
    fn format_server_move_label_fills_every_placeholder() {
        let label = format_server_move_label(
            "Move {column} earlier from position {position} of {total}",
            "Balance",
            2,
            3,
        );
        assert_eq!(label, "Move Balance earlier from position 2 of 3");
    }

    // ── server_table_displayed_slice ──

    fn row(pairs: &[(&'static str, &str)]) -> TableRow {
        pairs
            .iter()
            .map(|(k, v)| (*k, (*v).to_owned()))
            .collect::<HashMap<_, _>>()
    }

    #[test]
    fn displayed_slice_carries_only_the_supplied_rows_in_column_order() {
        let cols = vec![
            Column::new("name", "Name"),
            Column::new("balance", "Balance"),
        ];
        let rows = vec![
            row(&[("name", "Alice"), ("balance", "10")]),
            row(&[("name", "Bob"), ("balance", "20")]),
        ];
        let slice = server_table_displayed_slice(&cols, &rows, None);
        assert_eq!(slice.columns.len(), 2);
        assert_eq!(slice.columns[0].id, "name");
        assert_eq!(slice.rows.len(), 2);
        assert_eq!(
            slice.rows[0].cells,
            vec!["Alice".to_owned(), "10".to_owned()]
        );
        assert!(slice.rows[0].key.is_none());
    }

    #[test]
    fn displayed_slice_never_exceeds_the_rows_it_was_given() {
        // The defining contract: no row beyond the supplied slice can ever
        // appear, however many "columns" or "pages" exist server-side.
        let cols = vec![Column::new("name", "Name")];
        let rows = vec![row(&[("name", "Only One")])];
        let slice = server_table_displayed_slice(&cols, &rows, None);
        assert_eq!(slice.rows.len(), 1);
    }

    #[test]
    fn displayed_slice_uses_supplied_row_key() {
        let cols = vec![Column::new("id", "ID")];
        let rows = vec![row(&[("id", "42")])];
        let key_of: &dyn Fn(&TableRow) -> String = &|r: &TableRow| r["id"].clone();
        let slice = server_table_displayed_slice(&cols, &rows, Some(key_of));
        assert_eq!(slice.rows[0].key, Some("42".to_owned()));
    }

    #[test]
    fn displayed_slice_missing_cell_renders_as_empty_string_not_a_panic() {
        let cols = vec![Column::new("missing_key", "Missing")];
        let rows = vec![row(&[("other", "value")])];
        let slice = server_table_displayed_slice(&cols, &rows, None);
        assert_eq!(slice.rows[0].cells, vec![String::new()]);
    }
}
