# EntityTable

`EntityTable<T>` is the opinionated table for a complete, typed client
snapshot. The caller downloads the whole selected dataset, supplies stable row
keys and typed column functions, and the component owns transient paging while
sorting, resizing, and column visibility stay local. When the snapshot needs
filtering, the caller supplies the locally filtered snapshot and changes
`page_reset_key` to return paging to the first page.

## Choose the table from data ownership

| Data ownership | Component | Observable mode | Rule |
|---|---|---|---|
| Complete typed snapshot is already in the browser | `EntityTable<T>` | `client-snapshot` | Preferred for new contracted snapshot pages. |
| Server owns filtering, sorting, paging, and total count | `ServerDataTable` | `server-query` | Pass only the current slice and round-trip every query change. |
| Existing client table uses dynamic `HashMap` rows or DataTable-only features | `components::DataTable` | `compatibility-client` | Compatibility path; do not choose it for a new contracted snapshot by habit. |
| Existing simple table needs automatic link/badge columns or bulk selection | `widgets::DataTable` | n/a | Retained legacy widget with a different row model. |

Do not pass a server page to `EntityTable` and let it sort or filter that slice.
That silently changes a server query into page-local behavior. Do not download a
complete dataset merely to satisfy `EntityTable` when the server must own the
query. The component roots expose `data-table-data-mode` so browser audits can
detect an ownership mismatch on the running page.

## Shared mechanics, separate data models

`EntityTable` and the DataTable family intentionally keep different row and
column types, but they do not carry separate behavior for common mechanics:

| Mechanic | Shared owner |
|---|---|
| Page count, clamping, bounds, numbered window, and row-range caption | `components::data_table::pagination` |
| Resize minimum, maximum, and drag-delta bounds | `components::data_table::resize` |
| Show/hide transition, required-column guard, and last-visible guard | `components::data_table::chooser` |

The renderers remain separate because `EntityColumn<T>` resolves typed rows,
whereas `Column` resolves dynamic `TableRow` maps. Sharing those renderers would
erase the compile-time distinction the snapshot component exists to provide.

## Core inputs

| Prop | Purpose |
|---|---|
| `data: Signal<Rc<Vec<T>>, LocalStorage>` | The complete selected snapshot. |
| `columns: Vec<EntityColumn<T>>` | Typed cell, sort-key, visibility, and resize declarations. |
| `row_key: EntityRowKey<T>` | Stable identity used for keyed DOM rows and activation. |
| `dataset_identity: Signal<String>` | Identifies the downloaded dataset; a change resets only the current page. |
| `page_reset_key` | Optional identity for local view-state changes that should reset only paging. |
| `compact_row` | Optional single-cell renderer used at compact breakpoints without duplicating rows. |
| `preference_ownership` | Controlled or uncontrolled preference policy. |
| `storage_key` | Legacy local-storage compatibility prop; mutually exclusive with `preference_ownership`. |

Page number, free-text search, selected dataset, row data, and snapshot revision
are transient state and do not belong in `EntityTablePreferences`.

## Preference ownership

### Controlled: recommended for governed persistence

Controlled mode makes the consumer the only source of truth. Every table UI
operation emits one normalized, complete `EntityTablePreferences` replacement.
The component performs no browser-storage I/O and keeps rendering the supplied
signal until the consumer accepts a replacement.

```rust,no_run
use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferences,
};

let preferences = RwSignal::new(EntityTablePreferences::new(3));
let ownership = EntityTablePreferenceOwnership::controlled(
    preferences.into(),
    Callback::new(move |replacement| {
        // Validate/save through the page's governed persistence path, then
        // accept the returned value. This in-memory assignment illustrates
        // the acceptance step.
        preferences.set(replacement);
    }),
);

// Pass `ownership` through the EntityTable `preference_ownership` prop and
// keep `preference_version=3` aligned with the DTO schema.
```

Normalization is pure and deterministic. A schema-version mismatch resets to
defaults; unknown or required hidden-column ids are removed; unsupported sort
columns reset to system order; widths use the shared DataTable bounds.

### Uncontrolled without persistence

This is the default when neither ownership nor `storage_key` is supplied. The
component owns an in-memory signal for its lifetime and never reads or writes
`localStorage`.

```rust,no_run
use leptos_daisyui_rs::components::{
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence,
};

let ownership = EntityTablePreferenceOwnership::uncontrolled(
    EntityTablePreferencePersistence::Disabled,
);
```

### Legacy local storage

Existing callers can keep the historical prop unchanged:

```rust,ignore
<EntityTable
    // required snapshot props omitted
    storage_key="no-hires"
    preference_version=2
/>
```

It resolves to uncontrolled `LegacyLocalStorage` with the existing
`ldui-entity-table:<storage_key>` key. Supplying both `storage_key` and
`preference_ownership` fails closed instead of silently allowing a controlled
table to perform browser I/O.

## Migration path

For an existing `EntityTable` using `storage_key`, preserve that prop until the
consumer has a governed persistence endpoint. Then load the saved preference
DTO into a signal, pass controlled ownership, handle each full replacement in
the page, and remove `storage_key`. Keep `preference_version` stable unless the
preference schema actually changes.

Existing `components::DataTable` call sites remain compatible. Migrate a call
site to `EntityTable<T>` only when it represents a complete snapshot and its
required DataTable-only features have typed equivalents. Existing server-owned
pages migrate directly to `ServerDataTable`; they must not pass one fetched page
through either client table.

## Verification

The focused inner and browser lanes are:

```powershell
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser
```

The browser lane checks real sorting, paging, chooser behavior, legacy
persistence, resize restoration, compact rendering, row/action activation,
accessibility/style oracles, and the `client-snapshot` ownership marker.
