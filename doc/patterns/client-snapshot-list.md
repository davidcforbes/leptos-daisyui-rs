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
       -> filtered rows
       -> SnapshotTableState::local_row_projection
       -> SnapshotTablePage local_rows -> EntityTable data
       -> authoritative source rows -> EntityTable source_data

FilterSchema + local filter values + EntityTablePreferences
  -> SnapshotViewDefaults
  -> explicit enabled Save as Default callback only
```

`SnapshotTablePage` owns slot order and injects dataset/generation identity
from its private state view. It does not generate routes, fetch data, infer
permissions, invent columns, or persist anything. The consumer owns those
domain and transport decisions.

## Controlled local-row projection

Keep one complete `SnapshotData` in `SnapshotTableState`; do not replace that
authoritative snapshot on every local filter change. Instead, filter its
`Rc<Vec<R>>`, call `state.local_row_projection(filtered_rows)`, and pass the
result through `SnapshotTablePage::local_rows`. The returned
`SnapshotLocalRowProjection<R>` carries private generation/revision binding and
the matching `LocalResultSummary`, so the rendered rows and the no-results
decision cannot drift apart.

`SnapshotTablePage` validates the projection before use. It renders a current
projection as `EntityTable::data`, always supplies the complete displayed rows
as `EntityTable::source_data`, and falls back to the authoritative rows when a
projection is stale. A pending replacement keeps the old projection valid
while those rows remain displayed; completing a replacement or changing access
invalidates it. Consumers cannot supply a separate dataset identity.

The older `local_result` count-only prop remains source-compatible. When a
current `local_rows` value is present, its embedded summary is authoritative;
otherwise the page uses `local_result` as before.

```rust,no_run
let local_rows = RwSignal::new_local(None);

Effect::new(move |_| {
    let query = search.get();
    let projection = state.with(|state| {
        let displayed = state.view(None).displayed()?;
        let rows = displayed
            .rows()
            .iter()
            .filter(|row| matches_query(row, &query))
            .cloned()
            .collect::<Vec<_>>();
        state.local_row_projection(Rc::new(rows))
    });
    local_rows.set(projection);
});

view! {
    <SnapshotTablePage
        contract_id="no-hires"
        state=state.into()
        local_rows=local_rows.into()
        // header, selector, filters, table config, and callbacks omitted
    />
}
```

`SnapshotTablePage` derives collision-safe selector IDs from `contract_id`:
`<contract_id>-dataset-select` and `<contract_id>-rows-per-page`. Standalone
`DatasetSelector` and `EntityTable` callers should set `control_id` and
`page_size_control_id` respectively. The base `Select` also accepts an explicit
typed `id`; when omitted inside `Field`, the field association still supplies
its generated ID.

## Page-header navigation placement

`PageHeader` keeps its historical `PageHeaderNavigationLayout::InlineResponsive`
default: the optional `back` slot shares the title cluster at wide widths and
stacks within that cluster at compact widths. Choose
`PageHeaderNavigationLayout::DedicatedRow` when navigation must remain on its
own row above title, subtitle, freshness, dataset, and actions at every width:

```rust,no_run
<PageHeader
    title="No Hires"
    subtitle="Records requiring follow-up."
    navigation_layout=PageHeaderNavigationLayout::DedicatedRow
    navigation_label=Signal::stored("No Hires navigation".to_owned())
    back=Box::new(|| view! {
        <a href="/">"Back to Office"</a>
    }.into_any())
/>
```

The dedicated row is a labeled navigation landmark. Both layouts instantiate
the back view once, keep exactly one `h1`, preserve DOM/keyboard order, and emit
`data-page-header-navigation-layout` for browser audits. The dedicated mode
uses `min-w-0` and wrapping around both the navigation row and the lower
heading/action row so long labels do not squeeze the title or create page-wide
horizontal overflow.

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
        page_size_control_id="client-list-page-size"
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

## Behavior-only EntityTable passthroughs

`SnapshotEntityTableConfig` owns the internally rendered `EntityTable`'s
identity-critical bindings (rows, `dataset_identity`, `focus_scope`) itself,
sourced from the same private state view as the dataset selector -- a caller
cannot supply those through the config. Everything the underlying table can
express as pure behavior is a typed builder instead (`ldui-myhh` /
`ldui-5ano`), so a consumer that previously had to drop to a page-local raw
`EntityTable` for these can now stay on the canonical page:

| Builder | Forwards to | Purpose |
|---|---|---|
| `with_page_reset_key` | `EntityTable::page_reset_key` | Resets pagination on a caller-owned local-filter identity, distinct from the page's own dataset/access generation. |
| `with_viewport_fit` | `EntityTable::viewport_fit` | Framework-measured adaptive row capacity from a definite parent or CSS height budget. |
| `with_toolbar_actions` | `EntityTable::toolbar_actions` | Caller-rendered table utilities (Export, Refresh) placed after page size and before the column chooser. |
| `on_display_projection` / `with_projection_action_columns` | `EntityTable::on_display_projection` / `projection_action_columns` | Atomic read-only display projection for caller-owned export encoding, plus its action-column policy. |
| `with_column_chooser_trigger` | `EntityTable::column_chooser_trigger` | Text (default) or compact icon presentation of the framework-owned chooser; both keep identical accessible semantics. |

```rust,no_run
let table = SnapshotEntityTableConfig::new(columns(), row_key, preference_ownership)
    .with_page_reset_key(Signal::derive(move || local_filter_hash.get()))
    .with_viewport_fit(EntityTableViewportFit::fill_parent().with_min_rows(5))
    .with_toolbar_actions(move || view! {
        <Button on_click=export_csv>"Export CSV"</Button>
    }.into_any())
    .on_display_projection(Callback::new(move |projection| export_projection.set(projection)))
    .with_column_chooser_trigger(EntityColumnChooserTrigger::Icon);
```

None of these can carry rows, dataset identity, revision, count, or
generation -- their types have no such field, so a caller cannot smuggle
identity through them even by accident. A consumer already using the raw
`EntityTable` escape hatch purely for these behaviors (Office No-Hires: local
filter reset from a later page, fill-parent row capacity, an icon chooser,
and CSV export adjacent to table utilities) should migrate the call site back
onto `SnapshotTablePage` with these builders rather than keep a page-local
duplicate table. See `doc/components/entity_table.md`'s Core inputs table for
each underlying prop's full contract, and
`demo/src/demos/snapshot_table_page.rs::SnapshotTablePageControlsFixture` for
a complete reference fixture.

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
copy, distinct stable selector IDs, controlled projection/no-results behavior,
axe, and stable table/header geometry.

The behavior-only passthroughs above have their own focused fixture and
browser proof: `demo/src/demos/snapshot_table_page.rs`'s
`SnapshotTablePageControlsFixture` (route `/components/snapshot-table-page-controls`)
and `tests/snapshot_table_page_controls_smoke.rs`, covering local-filter page
reset from a later page, adaptive height, the icon chooser opening visibly,
export receiving the authoritative rendered projection, and no storage I/O or
dataset-identity drift across the whole sequence.
