# Client snapshot list

This is the supported opinionated path when one bounded dataset is downloaded
as an atomic snapshot and search, filters, sorting, visibility, resizing, and
paging all run locally. Use `ServerDataTable` when the server owns any of those
query operations.

## One controlled signal flow

```text
DatasetSelector request
  -> consumer transport
  -> SnapshotTableState accepts one complete SnapshotData
  -> source rows
       -> controlled local search and filter signals
       -> filtered rows -> EntityTable

FilterSchema + local filter values + EntityTablePreferences
  -> SnapshotViewDefaults
  -> explicit enabled Save as Default callback only
```

`SnapshotTablePage` owns slot order and injects dataset/generation identity
from its private state view. It does not generate routes, fetch data, infer
permissions, invent columns, or persist anything. The consumer owns those
domain and transport decisions.

## Preferred hybrid filter layout

The utility `FilterBar` above the table contains global search, controls that
do not map to a single column, active chips/count, local result count, one
Reset, one Save as Default, and save feedback. A status/type/owner control that
maps exactly to one column belongs in `EntityColumnFilters`, directly below
that column. Do not render it again in `FilterBar`.

Both regions read and write the same consumer signals. Reorder and visibility
are table preferences, so the second header row automatically follows the
same visible column sequence. Filter events are isolated from sort, resize,
and row activation.

```rust,no_run
let status = RwSignal::new(String::new());
let filters = vec![EntityColumnFilter::new("status", move || {
    view! {
        <label class="block w-full">
            <span class="sr-only">"Status"</span>
            <Select
                label=Signal::stored(Some("Status".to_owned()))
                value=status
                on_change=Callback::new(move |value| status.set(value))
            >
                <option value="">"All statuses"</option>
                <option value="ready">"Ready"</option>
            </Select>
        </label>
    }.into_any()
})];

view! {
    <FilterBar
        search=search_control
        active_filters=active_chips
        on_remove=remove_filter
        on_reset=reset_filters
        result=result_summary
        default_save=save_binding
        texts=filter_texts
    />
    <EntityTable
        data=filtered_rows
        source_data=source_rows
        columns=reactive_columns
        column_filters=filters
        row_key=row_key
        dataset_identity=dataset_identity
        focus_scope=dataset_access_generation
        preference_ownership=controlled_preferences
        texts=table_texts
    />
}
```

## Default-view boundary

Build the save payload only with `FilterSchema::project_defaults`. The result,
`SnapshotViewDefaults`, serializes exactly two top-level members: `filters`
and `table`. Its filter keys are schema-ordered and allowlisted. Dataset
selector identity, free-text search, page number, rows/revision, sessions,
actions, and undeclared consumer fields such as `office_id` have no
representation and are rejected rather than silently dropped.

Changing a filter, table preference, locale, dataset, or rendered feedback
never invokes persistence. Only pointer, Enter, or Space activation of the one
enabled Save as Default button invokes the callback once. The consumer owns
Pending, Saved, Conflict, and Failure transitions and any revision check.

## Localization and focus

Supply reactive `FilterBarTexts`, `ActiveFilterTexts`,
`DatasetSelectorTexts`, and `EntityTableTexts`. Supply reactive
`EntityColumns<T>` when headers, chooser labels, cell/compact copy, comparator,
or sort keys can change. Stable column IDs preserve valid preferences; the
internal semantic generation prevents reuse of stale sorted indices.

Wrap repeatable row controls in `EntityRowAction`. Give `EntityTable` the
unfiltered `source_data` and a dataset/access `focus_scope`. A true source-row
removal recovers to the same action at the actual visible position when it is
eligible. Filter/page hiding, missing actions, or an empty result focus the
named table region; dataset/access changes never cross-focus.

## Release proof

Run the focused native and real-browser lane first, then the repository gate:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
cargo xtask verify-full
```

Browser proof must cover aligned filter cells after reorder/hide, filter-event
isolation, one Reset/Save, explicit pointer/Enter/Space save counts, every save
state, locale updates without preference reset, latest column semantics,
same-scope row-removal focus, hide/scope/user-moved negative cases, compact
copy, axe, and stable table/header geometry.
