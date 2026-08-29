# Adopted Architecture: Opinionated Pages and Independent Wasm Satellites

**Status:** Adopted — full-scale implementation in progress

**Last updated:** 2026-08-26

**Scope:** `leptos-daisyui-rs`, `4iiz-Office`, and `4iiz-Inventory`

**Audience:** Codex, Claude Code, reviewers, and the migration coordinator

**Tracking:** framework architecture `ldui-pwx`; No-Hires first production
conversion `op-vzoiv`

## 1. Authority and purpose

This document is the canonical contract for building and converting 4iiz web
pages. Give an AI coding agent this document together with one page assignment
and that page's product requirements.

This document supersedes the earlier recommendation to retain one shipping
Office Wasm application. It is an implementation directive, not a proposal or
an experiment. The No-Hires test capsule supplied baseline evidence but was
not a production satellite. No-Hires is now the first production conversion
and reference implementation: it will be independently built, loaded,
authenticated, tested, deployed, and measured. All other eligible pages will
then be converted in parallel using the same contract.

No-Hires measurements tune budgets, implementation details, and operating
limits. They do not decide whether the adopted satellite and page-contract
architecture proceeds.

The architecture has two equally important goals:

1. Make the visual and behavioral result explicit enough that agents do not
   invent page structure or component behavior.
2. Make each independent page a build, test, artifact, and deployment unit so
   changing it does not rebuild the core or unrelated pages.

The CI/CD changes described here are prerequisites, not optional follow-up
work. Page isolation is not real if the pipeline still rebuilds and retests the
portfolio for every page edit.

## 2. Implementation mandate and baseline evidence

### 2.1 What the prototype baseline established

The completed No-Hires prototype work established that:

- an opinionated snapshot-table page can cover the page's real interaction
  model;
- office selection can load a bounded dataset while search, filtering,
  sorting, pagination, and claim removal remain client-side;
- a separately linked test capsule can exclude unrelated Office pages;
- native, browser, accessibility, and visual tests can be selected by semantic
  ownership instead of running the full workspace gate;
- named states, stable fixtures, semantic selectors, and negative controls
  give agents an objective definition of done;
- the dominant warm verification cost is browser and visual work, not the
  native page tests.

### 2.2 What production implementation must now deliver

The current production `/no-hires` route is still part of the monolithic
`office-perf-web` bundle. The test capsule:

- is not served by the production route;
- does not use the production child-session lifecycle;
- is not independently released or rolled back;
- does not demonstrate that a core release can remain byte-for-byte
  unchanged;
- does not provide an apples-to-apples production startup comparison.

The baseline measurements demonstrate test isolation and page behavior.
Full-scale implementation now supplies the production delivery, session,
independent release, rollback, and matched before/after evidence.

### 2.3 Adopted implementation directive

Implement No-Hires as the first production satellite and reference conversion.
Measure it against the current production route, apply the operational
lessons, and continue the scheduled conversion of every eligible page. An
objective security, correctness, data-integrity, or operational failure blocks
the affected implementation until corrected; it does not return the program
to architecture-selection mode.

The required portfolio is:

- a small core for master authentication, session ownership, navigation, and
  truly shared in-process workflows;
- many independently compiled satellite pages for bounded workflows;
- explicitly coded, searchable core navigation, launch actions, and server
  routes;
- CI/CD release descriptors that validate independent artifacts without
  driving runtime navigation or routing;
- a shared opinionated UI framework and page contracts;
- page-level verification throughout a migration wave;
- one full rebuild and cross-page end-to-end gate after all isolated page
  changes in that wave have been integrated.

## 3. Goals and non-goals

### 3.1 Goals

- Cover at least 99% of routine page composition with approved patterns and
  archetypes.
- Make page layout, states, data ownership, persistence, and tests explicit.
- Let an agent implement one page without redesigning foundational controls.
- Preserve responsive behavior on low-bandwidth networks in Mexico and India.
- Keep office-sized snapshots, including roughly 2,000 No-Hires rows,
  interactive without a server round trip for each view change.
- Allow several satellite pages, or several tabs of one satellite, to remain
  open side by side.
- Build, verify, package, deploy, and roll back one satellite independently.
- Verify with artifact hashes that unrelated surfaces did not change.
- Run expensive tests only when their declared inputs or behaviors can change.
- Keep the final full release gate authoritative without paying its cost after
  every isolated page edit.

### 3.2 Non-goals

- A generic low-code page builder or runtime JSON UI renderer.
- Runtime component/page registries or contract/descriptor-driven navigation,
  launch behavior, or server routing.
- One macro that generates an entire business application.
- Cross-page shared mutable browser memory.
- Treating a public tab identifier as an authentication credential.
- Persisting every transient interaction.
- Replacing the server's authorization checks with client-side claims.
- Loading an entire all-office dataset merely because client-side filtering is
  available.
- Eliminating all duplicate dependency bytes across independently compiled
  satellites before measuring whether that is material.
- Skipping verification because a previous run passed.
- Building a custom CI task graph or checkpoint engine in Rust.

## 4. Terminology

| Term | Meaning |
|---|---|
| **Core** | The primary application that owns master authentication, master session state, navigation, and workflows that genuinely require shared in-process state. |
| **Satellite** | A same-origin, independently compiled and deployed Wasm page opened in its own tab. It does not import the core or another satellite. |
| **Surface** | Any independently versioned browser artifact: the core or one satellite. |
| **Page contract** | Typed declaration of a page's archetype, dataset selector, local controls, persisted settings, states, capabilities, and test obligations. |
| **Surface release descriptor** | CI/CD metadata binding an explicitly coded surface ID to immutable artifacts, compatibility versions, and verification evidence. It does not create routes or navigation. |
| **Shared foundation** | Versioned tokens, primitives, opinionated patterns, page archetypes, audit rules, and contract schema consumed by surfaces. |
| **Page receipt** | Machine-verifiable evidence binding page inputs and foundation version to artifact hashes and selected test results. |
| **Page-level gate** | Tests and packaging for one page plus only its real dependencies. |
| **Final integration gate** | The one complete workspace build, all affected shared checks, and cross-page end-to-end release suite after a migration wave is assembled. |
| **Dataset selector** | A control that replaces the source snapshot. Office is a dataset selector, not a filter. |
| **Local view state** | Search, filters, sort, pagination, and table layout applied in the satellite without fetching a replacement dataset. |

Normative words `MUST`, `MUST NOT`, `SHOULD`, and `MAY` carry their usual
requirements meaning.

## 5. Delivery topology

### 5.1 Dependency rule

The required compile-time dependency direction is:

```text
versioned shared foundation
        │
        ├────────► core surface
        ├────────► no-hires surface
        ├────────► inventory-aging surface
        └────────► other satellite surfaces

explicit server launch/session/static-asset code
        ├────────► serves the core
        ├────────► serves each coded satellite route
        └────────► validates every child-session request
```

The following dependencies are forbidden:

```text
core ──X──► satellite
satellite ──X──► core
satellite A ──X──► satellite B
```

A satellite MAY depend on a small versioned shared crate. It MUST NOT depend
on the core application's page modules, router, global app state, or compiled
artifact. Explicit core navigation MAY name a satellite's stable route, but it
MUST NOT import or link that satellite's package.

### 5.2 Explicit navigation, launch, and routes

Core navigation and launch actions are ordinary, explicit Rust source. Server
launch, boot-document, API, and event endpoints are ordinary, explicit router
code. Page labels, icons, stable URLs, launch behavior, and permission checks
must be code-searchable and code-reviewed.

Therefore:

- adding, removing, or renaming a satellite intentionally changes the core
  navigation and relevant server route code and may rebuild those surfaces;
- the integration coordinator owns those shared registration changes so
  parallel page agents do not contend on them;
- changing an existing satellite's UI, domain logic, or artifact does not
  rebuild core Wasm, the server, or unrelated satellites;
- changing only a promoted immutable artifact does not change navigation or
  route code;
- removing authorization from a user does not require a browser artifact
  rebuild;
- the server remains authoritative for authorization, but authorization data
  cannot create a page, route, or launch action.

Page contracts and CI/CD tools MAY scan or index the explicit source to
validate uniqueness, ownership, and test selection. That generated validation
data is never a runtime navigation or routing input.

The page-only independence guarantee applies after a satellite's explicit
navigation and server routes exist. Adding, removing, or renaming that
registration is intentionally a broader core/server integration change.

### 5.3 Stable routes and immutable assets

No implementation detail such as `/satellites/` appears in user-facing URLs.
For No-Hires, use:

```text
POST /no-hires/launch
GET  /no-hires/t/<public-tab-id>/
GET  /no-hires/assets/<build-id>/<content-hashed-file>
GET  /no-hires/t/<public-tab-id>/api/snapshot?office=<office-id>
POST /no-hires/t/<public-tab-id>/api/claims/<row-id>
GET  /no-hires/t/<public-tab-id>/api/events
GET  /no-hires/t/<public-tab-id>/api/preferences
PUT  /no-hires/t/<public-tab-id>/api/preferences
```

The exact API resource names may change, but the tab-scoped route boundary
MUST remain.

HTML boot documents are stable route responses with short or no-cache
semantics. Wasm, JavaScript glue, CSS, and other static assets are immutable
and content-addressed. Explicit server code owns the stable route. CI/CD
promotion changes only the page-specific immutable build reference used by
that coded route.

### 5.4 Independent production artifacts

Each surface produces its own:

- optimized Wasm;
- JavaScript loader;
- CSS needed by that surface;
- immutable asset directory;
- source map and symbol archive, where policy permits;
- surface release descriptor;
- software bill of materials or dependency record;
- page receipt.

Publishing No-Hires MUST NOT rewrite the core or another satellite's asset
directory. Promotion changes only the No-Hires artifact reference. Rollback
restores only that reference.

The surface release descriptor is consumed by CI/CD for hashing,
compatibility, verification, publishing, promotion, and rollback. It cannot
register a route, add a navigation item, generate a page, or make a surface
available at runtime. A route must already exist in explicit reviewed server
code.

### 5.5 Browser-tab model

- Core launches a satellite with `window.open` or an ordinary target-blank
  navigation after a successful launch POST.
- Every launch creates a distinct tab-scoped child session, even for the same
  user and page.
- A satellite is self-contained after boot and keeps transient page state in
  its own Wasm instance.
- Tabs do not use `window.opener` as an authority or shared state channel.
- The same satellite artifact is cacheable across tabs; additional tabs
  SHOULD not transfer the Wasm bytes again while the immutable asset remains
  cached.
- Core logout MAY broadcast a best-effort close-or-lock notification through
  `BroadcastChannel`. Server-side revocation is authoritative because
  browsers cannot reliably force-close unrelated tabs.
- A satellite has no logout action. The user closes its tab.

This model intentionally permits side-by-side work. Separate tabs are an
interaction requirement, not an accidental consequence of routing.

### 5.6 Physical package topology

Every surface is a real Cargo/Trunk build root, not a feature flag or second
`main` function inside the core package. A representative consumer layout is:

```text
app/
  server/                         # explicit launch/API/static-route code
  surfaces/
    core/
      Cargo.toml
      Cargo.lock
      Trunk.toml
      src/
    no-hires/
      Cargo.toml
      Cargo.lock
      Trunk.toml
      src/
      tests/
    inventory-aging/
      Cargo.toml
      Cargo.lock
      Trunk.toml
      src/
      tests/
  release-descriptors/            # generated CI/CD evidence
  receipts/                       # generated CI/CD evidence
```

The exact directories may differ, but these properties are mandatory:

- a page build names its own `--manifest-path` and Trunk target;
- a satellite's dependency graph has no core or sibling page package;
- the page package itself does not require an enumerated Cargo workspace
  member list;
- adding a page still requires explicit coordinator-owned navigation and
  server-route integration;
- each independently released surface has a reproducible dependency lock
  boundary, or an equivalently isolated generated lock artifact;
- CI/CD validation indexes per-page contracts and release descriptors;
- page output and cache directories are page-specific;
- the final integration task enumerates that CI-only index and invokes the
  independent builds.

A shared root workspace MAY be used for development utilities only if adding
or building one surface does not mutate a portfolio-wide lockfile or make
unrelated surfaces inputs to that build. A per-surface package can consume a
pinned framework release, immutable source revision, or a validated local
foundation artifact. Its receipt records the exact choice.

Independent Wasm bundles will duplicate some framework and Leptos bytes across
different pages. Browsers reuse the immutable bundle across tabs of the same
page, but cannot assume byte reuse across separately linked Wasm modules.
Measure this portfolio tradeoff before introducing a more complex shared-Wasm
runtime.

## 6. Authentication and child sessions

### 6.1 Launch protocol

1. The authenticated core sends `POST /<page>/launch` with the master session
   and CSRF protection.
2. The server confirms the user may launch that surface.
3. The server generates:
   - a high-entropy public tab identifier for the URL;
   - a separate high-entropy secret;
   - a database record containing only a hash of the secret.
4. The response installs the secret in a cookie scoped to the exact tab route
   and redirects or returns the tab URL.
5. The satellite loads its own HTML and assets, then calls APIs beneath its
   tab-scoped route.

The cookie MUST use `Secure`, `HttpOnly`, and an appropriate `SameSite`
setting. Its `Path` MUST be `/<page>/t/<public-tab-id>/`. Do not put the secret
in the URL, HTML, JavaScript, local storage, session storage, logs, or
analytics.

### 6.2 Child-session record

The server-side child-session record includes at least:

- child-session ID;
- public tab ID;
- secret hash;
- master-session ID;
- user ID;
- surface ID;
- issued time;
- last-seen time;
- absolute expiry;
- revocation time and reason;
- optional current dataset identity for event routing and diagnostics.

The public tab ID is routing metadata, not authentication.

### 6.3 Lifetime and renewal

- Default renewable inactivity lease: 15 minutes.
- Successful authenticated activity renews the lease.
- Renewal MUST NOT extend beyond the master session's absolute expiry.
- Core logout, master timeout, user disablement, or authorization removal
  invalidates all affected child sessions.
- Each API and event-stream authorization checks both the child record and
  the master-session validity.
- Expired and revoked child records are cleaned up by an idempotent server
  task.
- Losing connectivity does not convert a lease into an unlimited session.

On `401` or `403`, the satellite enters a locked/expired state, stops
mutations and event reconnect loops, and tells the user to return to the core.

### 6.4 Security boundary

Satellites are same-origin trusted first-party code. A compromised same-origin
surface can attack other same-origin content regardless of process
separation. Independent bundles improve delivery and fault isolation; they do
not create a browser security origin.

The server MUST:

- validate every requested office identifier and authorize every mutation;
- permit a No-Hires user to select any canonical office or the explicit
  All Offices dataset; No-Hires has no office-specific access restriction;
- validate `Origin`/CSRF as appropriate;
- bind events to authorized sessions and datasets;
- reject guessed public tab IDs without the secret;
- avoid sensitive data in URLs and client logs;
- rate-limit launch and mutation endpoints;
- audit launch, claim, preference write, revocation, and authorization
  failures.

## 7. State and persistence

### 7.1 State ownership

| State | Owner | Lifetime | Survives refresh? | Survives session? |
|---|---|---:|---:|---:|
| Master identity/session | Core and server | Master session | Yes | No |
| Child page session | Server | Tab lease | Usually | No |
| Loaded office snapshot | Satellite memory | Current page instance | No | No |
| Search text | Satellite memory | Current page instance | No | No |
| Active filters | Satellite memory | Current page instance | No, unless reloaded from saved default | Only when explicitly saved |
| Sort | Satellite memory | Current page instance | No, unless reloaded from saved default | Only when explicitly saved |
| Current page number | Satellite memory | Current page instance | No | No |
| Column width/order/visibility | Satellite memory | Current page instance | No, unless reloaded from saved default | Only when explicitly saved |
| Saved default | Server `user-state.sqlite` | Until changed/deleted | Yes | Yes |
| Cached reporting snapshot/cubes | Server `dashboard.sqlite` | Rebuildable cache lifecycle | N/A | N/A |

### 7.2 Separate SQLite files

Durable user settings live in a physically separate server-side
`user-state.sqlite`. Cached PostgreSQL mirrors and calculated cubes remain in
the rebuildable `dashboard.sqlite`.

This separation is mandatory:

- rebuilding or replacing dashboard cache data cannot erase user settings;
- backup and retention policies can differ;
- migrations of durable preferences are explicit;
- no satellite opens a SQLite file directly; all access is through
  authenticated server APIs.

### 7.3 Save as Default

Filters and sorts apply immediately in memory. Persistence occurs only when
the user selects **Save as Default**.

For a snapshot-table page, save:

- filter values;
- sort column and direction;
- page size;
- column visibility;
- column order;
- column widths.

Do not save:

- selected office;
- free-text search;
- current page number;
- row data;
- snapshot revision;
- child-session or tab identifiers.

The preference key is at least `(user_id, surface_id, contract_version)`.
Preferences MUST have a versioned schema and deterministic migration or reset
behavior. Saving is idempotent. A successful response updates the locally
known preference revision; a conflict or failure is shown without discarding
the active in-memory choices.

## 8. Opinionated UI architecture

The framework is not merely a collection of low-level DaisyUI wrappers.
Routine pages are assembled from five bounded layers.

### 8.1 Layer 0: semantic foundations

This layer owns:

- design tokens and generated CSS;
- typography, spacing, density, radius, elevation, and motion policies;
- semantic color and status vocabulary;
- breakpoint and responsive rules;
- focus, keyboard, reduced-motion, and accessibility rules;
- visual-quality audit rules.

Consumers MUST use semantic tokens. They MUST NOT introduce arbitrary colors,
spacing, shadows, or breakpoints in page code.

### 8.2 Layer 1: primitives

Primitives include Button, Input, Select, Badge, Alert, Dialog, Drawer,
Tooltip, Spinner, Skeleton, Tabs, Pagination controls, and accessible
form-field composition.

Primitives expose typed variants and merge caller classes through the
framework's approved mechanism. A business page SHOULD rarely compose raw
DaisyUI class strings.

### 8.3 Layer 2: opinionated patterns

Patterns encode recurring layout and behavior:

| Pattern | Contract |
|---|---|
| `PageHeader` | Title, concise context, primary actions, status/last-updated slot, responsive action wrapping. |
| `DatasetSelector` | Visually distinct source-snapshot selector with loading, failure, and current-dataset labeling. |
| `KpiStrip` | Responsive row of consistently sized stat cards with value, label, optional trend/help, and semantic status. |
| `FilterBar` | Slim utility row above the table for global search, non-column/domain controls, active-filter summary, result count, Reset, and Save as Default. One-to-one column filters belong in the table's aligned filter row and share this controlled state. |
| `EntityTable` | Client-snapshot table with an aligned column-filter row, local filter/sort/page, stable column tracks, resizable/reorderable/hideable columns, stable row identity, empty/loading/error states, and keyboard/accessibility behavior. |
| `ServerDataTable` | Explicit server-query table for datasets that cannot be safely or efficiently loaded as a snapshot. |
| `PageStatePanel` | Consistent loading, empty, no-results, error, expired-session, and forbidden presentation. |
| `ActionFeedback` | Pending, success, recoverable conflict, stale-row, and failure behavior for row actions. |

Patterns own spacing, alignment, responsive collapse, labels, keyboard
behavior, and state presentation. Page code supplies typed content and domain
callbacks.

`EntityTable` and the DataTable family share pagination, resize bounds, and
column-visibility transitions while retaining typed renderers for their
different row models. Their data modes are explicit and browser-observable:
`client-snapshot` for `EntityTable`, `server-query` for `ServerDataTable`, and
`compatibility-client` for the existing dynamic `components::DataTable` path.
New contracted snapshots use `EntityTable`; existing dynamic client tables
remain compatible until their required feature surface has a typed migration.
Silently switching between client and server filtering is forbidden.

### 8.4 Layer 3: page archetypes

The initial archetype catalog is:

1. `SnapshotTablePage`
2. `ServerTablePage`
3. `RecordDetailPage`
4. `FormWorkflowPage`
5. `DashboardPage`
6. `SettingsPage`

Each archetype owns the broad composition and state model. A page may request
a new archetype only when its information architecture cannot fit an existing
one without violating user needs. Cosmetic preference is not sufficient.

### 8.5 Layer 4: domain composition

Domain pages provide:

- typed row and form models;
- column definitions;
- allowed filters;
- dataset loader;
- mutations and event reducers;
- authorization-derived capabilities;
- copy and domain-specific cells;
- named story fixtures.

Domain pages do not redefine table mechanics, filter layout, responsive
behavior, loading panels, or visual tokens.

### 8.6 No generic page generator

Agents are the code generators. We will not build another general program that
tries to infer a business UI. Page-contract macros exist to declare intent,
provide typed metadata, and validate invariants. They do not replace ordinary
Rust implementation of domain behavior.

### 8.7 Repository responsibilities

| Repository | Owns |
|---|---|
| `leptos-daisyui-rs` | Tokens, primitives, opinionated patterns, archetypes, page-contract schemas, browser audit rules, reference states, and framework verification. |
| `4iiz-Office` | Explicit Office navigation/routes, Office core, satellite domain code, snapshot/mutation/event APIs, child sessions, Office preferences, release descriptors/receipts, and Office delivery orchestration. |
| `4iiz-Inventory` | Explicit Inventory navigation/routes, Inventory core/satellites, domain APIs and adapters, release descriptors/receipts, and Inventory delivery orchestration. |

A consumer needing a generally reusable component opens or claims one central
framework change. It does not copy the component into Office and Inventory.
A product-specific cell, reducer, query adapter, or label remains in the
consumer.

## 9. Page contracts

Every new or converted page MUST declare a contract next to the page. The
exact macro syntax may evolve, but it must express the following information
without relying on prose. The contract validates explicit Rust implementation;
it is not interpreted to render, register, navigate to, or launch the page.

```rust
page_contract! {
    id: "no-hires",
    title: "No Hires",
    owner: "office",
    delivery: Satellite,
    archetype: SnapshotTablePage,
    route: "/no-hires/",
    dataset: {
        selector: Office,
        default: UserContextOffice,
        allow_all: true,
        load: AtomicSnapshot,
    },
    table: {
        mode: ClientSnapshot,
        row_key: NoHireRow::id,
        features: [
            Search,
            LocalFilters,
            MultiColumnSort,
            ResizeColumns,
            ReorderColumns,
            ToggleColumns,
            Pagination,
        ],
    },
    persisted_default: [
        Filters,
        Sort,
        PageSize,
        ColumnVisibility,
        ColumnOrder,
        ColumnWidths,
    ],
    transient: [
        Office,
        Search,
        CurrentPage,
        Rows,
        SnapshotRevision,
    ],
    realtime: {
        transport: ServerSentEvents,
        events: [Claimed, SnapshotInvalidated],
    },
    states: [
        Loading,
        Ready,
        EmptyDataset,
        NoFilterResults,
        SnapshotError,
        ClaimPending,
        ClaimConflict,
        StreamDisconnected,
        SessionExpired,
        Forbidden,
    ],
}
```

The contract schema MUST require:

- stable page/surface ID and product owner;
- core-versus-satellite decision;
- archetype and route;
- dataset scope and selector semantics;
- client-snapshot or server-query data mode;
- row identity;
- controls and capabilities;
- persistence allowlist and transient-state list;
- mutation and real-time event model;
- permissions/capability rules;
- responsive behavior;
- named UI states;
- accessibility obligations;
- page-owned source globs;
- required native, browser, visual, artifact, and security tests;
- performance and bundle budgets;
- compatibility versions for shared foundation, server API, and preference
  schema.

The macro or validator MUST reject contradictory declarations, including:

- `ClientSnapshot` with server-side sort callbacks;
- a dataset selector also listed as a local filter;
- persisted row/snapshot/session data;
- a satellite importing core state;
- a mutation without pending, success/removal, conflict, and failure states;
- a real-time page without disconnect and resynchronization behavior;
- a visual page with no named baselines.

### 9.1 Generated CI/CD surface release descriptor

Packaging generates a surface release descriptor for each satellite from its
page contract, explicit build inputs, and produced artifacts. Agents do not
use this descriptor to implement the page and do not add entries to a runtime
registry.

```toml
id = "no-hires"
contract_version = 1
delivery = "satellite"
expected_route = "/no-hires/"
artifact = "dist/no-hires/<build-id>/"
api_compat = "office-page-v1"
foundation_compat = "ldui-page-v1"
preference_schema = 1
expected_launch_policy = "office.no-hires.read"
```

Packaging fills immutable hashes, sizes, and test-receipt identity into the
generated release metadata. The page-level gate compares the descriptor with
the page contract and package code. During coordinator integration, CI/CD also
compares it with the explicit navigation, server-route, and authorization
code. Duplicate IDs/routes, mismatches, incompatible versions, missing
artifacts, or mutable asset names fail validation.

The descriptor is build and deployment evidence only. Reading it at runtime
MUST NOT create a route, navigation item, launch action, page component, or
permission.

## 10. `SnapshotTablePage` contract

### 10.1 Required layout

From top to bottom:

1. `PageHeader`
2. office `DatasetSelector`
3. optional full-width `KpiStrip`
4. slim `FilterBar` utility row for global/non-column controls and filter state
5. active status/error feedback when needed
6. full-width `EntityTable` with a column-aligned filter row beneath its
   column-header row
7. table pagination integrated with the table pattern

Office is not placed among the filters. A local filter that maps one-to-one to
a visible column is rendered once, in the table's second `thead` row beneath
that column. The utility `FilterBar` does not duplicate those controls; it
retains global search, filters that do not correspond to one column, active
chips/count, Reset, and Save as Default. Header, filter, and body rows share
one horizontally scrolling column-track model at narrow widths. An optional
drawer may mirror the same controlled filter state as a narrow-layout
fallback, but it is not the desktop default and cannot become a second source
of truth.

The opinionated visual contract is a dark-blue column-header band with white
content, a light-blue column-filter band with dark content, and faint neutral
row and column grid lines. These colors are framework semantic tokens, not
consumer literals. Zebra striping is opt-in because the two hierarchy bands
and full grid already provide row/column structure.

Sorting is a body-data operation. It may reorder body rows and update the
sort indicator and announcement, but MUST preserve the outer table bounds,
column widths and x positions, header/filter positions, grid-line positions,
and horizontal scroll origin. Sort indicators reserve space in every sortable
header, and the column-track model never derives from only the currently
visible page of cells.

### 10.2 Client-snapshot data flow

For No-Hires:

1. Load the user's default office snapshot after boot.
2. Normalize and index rows once for local operations.
3. Apply saved default filters, sort, page size, and column layout.
4. Apply search, filter, and sort changes immediately in memory.
5. Paginate the derived local result.
6. On a claim, send the mutation with row/revision identity.
7. On success or an event that another user claimed the row, remove it from
   the local source set and recompute the derived view.
8. Re-request the snapshot only on explicit office selection, authoritative
   invalidation, detected revision gap, or recovery that cannot be applied
   incrementally.

For the expected office-level dataset of approximately 2,000 rows, this
reduces repeated bandwidth and latency. It also keeps interaction responsive
when the user is far from the server. `All Offices` is explicit and MAY have a
different size budget or server strategy if production measurements require
it.

### 10.3 Atomic office changes

Selecting another office starts a new dataset load. During the load:

- retain the old snapshot rather than blanking the table;
- label it with the office it actually represents;
- show the pending destination office;
- prevent an old response from overwriting a newer selection;
- swap rows, revision, counts, and office label atomically only after the new
  snapshot validates;
- preserve filters, sort, page size, and column layout;
- reset current page only as needed to produce a valid page;
- on failure, keep the prior snapshot and provide Retry.

Filters and sorts are local settings. Office changes do not reset them.

The canonical typed controller mints opaque checked request handles rather
than accepting caller sequence numbers. A matching response that names the
wrong office is rejected and consumes that request, preventing a permanent
loading state. Row-action work likewise starts with a framework-issued opaque
handle bound to its stable key and the displayed dataset/access generation.
Atomic office replacement or expired/forbidden access clears the old action
bindings; late action completions are ignored and cannot repopulate the new
surface.

### 10.4 Claims and real-time events

Claim mutations are idempotent or carry an idempotency key. The server remains
authoritative.

- Own successful claim: remove the row locally.
- Another user's claim event: remove the row locally.
- Already-claimed conflict: remove or reconcile the row and explain the
  conflict without treating it as a fatal page error.
- Temporary mutation failure: restore enabled state and permit retry.
- Event disconnect: show non-blocking stale/reconnecting status and reconnect
  with bounded backoff.
- Revision gap or `SnapshotInvalidated`: load one replacement dataset.
- Duplicate or out-of-order event: ignore safely using event/revision identity.
- Office switch: unsubscribe or logically detach from the prior dataset before
  applying events to the new one.

The server signals clients that currently hold the affected dataset. It may
send a row-level event or request a full resnapshot. Full resnapshot should be
rare.

## 11. Page-contract validation and CI/CD enforcement

There is no runtime page registry, component registry, data-driven UI
definition, or manifest-driven navigation system.

Normal Rust source is authoritative:

- components, patterns, and archetypes are typed Rust APIs with Rust
  documentation and code-searchable imports;
- each page is ordinary explicit Rust code;
- each page contract is a compile-time declaration co-located with that code;
- core navigation and launch actions are explicit Rust code;
- server routes and authorization checks are explicit server code.

CI/CD MAY parse page contracts and explicit source, or emit a temporary
machine-readable index, to select tests and validate the assembled release.
That index is generated evidence, not application input, and is not shipped as
a runtime page catalog.

CI enforces:

- no raw hex/rgb colors in consumer page code;
- no unapproved DaisyUI component classes in domain pages;
- no arbitrary breakpoint/spacing values outside the foundation;
- no duplicate local reimplementation of approved framework patterns;
- every page has a valid compile-time contract;
- each generated release descriptor agrees with the explicit page package,
  navigation, server route, and authorization code;
- page IDs and routes are unique;
- every contract state has a fixture/story or a documented nonvisual test;
- every visual baseline has review metadata;
- a satellite has no forbidden dependency edge;
- the core may explicitly navigate to a satellite route but cannot import or
  link the satellite package;
- persisted state is allowlisted;
- source ownership is unambiguous.

Enforcement should produce a direct diagnostic and approved replacement, not
merely fail on a style string. Validation output MUST NOT generate page
implementation, navigation, launch behavior, or server routing.

## 12. CI/CD architecture

### 12.1 Governing principle

Keep orchestration in the existing `cargo-make`/CI runner. Put
project-specific build, release-descriptor, hashing, compatibility, packaging,
and verification logic in the Rust `xtask`. Steps are idempotent and inspect
their required outputs.

Do not create a general Rust task runner, custom DAG, or checkpoint engine.
Use the CI platform's rerun-failed-job support. Cache validated compilation and
browser artifacts, never a test verdict.

Local commands and CI jobs MUST call the same underlying tasks.

### 12.2 Required page commands

The implementation may refine spelling, but must provide these capabilities:

```text
cargo xtask verify-page no-hires --inner
cargo xtask verify-page no-hires --browser
cargo xtask verify-page no-hires --visual
cargo xtask verify-page no-hires --all
cargo xtask package-page no-hires
cargo xtask write-page-receipt no-hires
cargo xtask verify-page-receipt no-hires <receipt>
cargo xtask assemble-release-manifest <receipt-directory>
cargo make ui-final-integration
cargo make ship-page -- no-hires
```

`verify-page` resolves the contract and ownership metadata, runs the selected
lane, and records timings. `package-page` builds the exact production
satellite, not a test-only substitute. `ship-page` packages, verifies,
publishes immutable assets, performs compatibility checks, and atomically
promotes only that page's artifact reference. None of these commands changes
navigation or server routes.

### 12.3 Semantic affected-test selection

Selection is based on declared ownership and behavior, not file extensions
alone.

During a parallel migration wave:

| Changed input | Required action |
|---|---|
| One page's domain source, contract, fixture, or page-owned style | Run that page's declared lanes and package it. |
| That page's browser harness | Run that page's browser and dependent visual lanes. |
| Reviewed visual baseline only | Validate review metadata and run that page's visual lane. |
| Documentation with no executable contract input | Markdown/document checks only. |
| Frozen shared foundation, root workspace, central CI, shared explicit server routing, or unknown ownership | Fail with an ownership/foundation error. Do **not** silently run the full gate. |

The last row is deliberate. Page agents are not allowed to change shared
inputs during the fan-out. Escalating to a full gate would hide an ownership
violation and recreate the month-long feedback loop.

In steady state, an approved shared-foundation change selects the reverse
dependency set declared by compile-time page contracts and generated release
descriptors. It can select many surfaces if the shared behavior genuinely
affects many surfaces. Unknown ownership still fails closed.

### 12.4 Build caching

Cache keys include all real inputs:

- Rust source and relevant `Cargo.toml`/lock data;
- target triple, profile, Rust/Cargo version, and Wasm tools;
- foundation and contract schema versions;
- generated token/schema inputs;
- npm lockfile and browser harness version where applicable;
- environment variables that alter output;
- exact build command and required output list.

Cache entries contain build artifacts only. Selected tests execute on every
run. A missing or malformed expected output invalidates the cache even when
the key matches.

Each parallel worktree uses a distinct `CARGO_TARGET_DIR` and page-owned
browser output directory. Shared mutable target directories are forbidden.

### 12.5 Page receipts

A page receipt makes isolated verification portable across integration
commits. It is not bound solely to a Git commit SHA because merging a page
changes the commit without changing the page bytes.

A receipt includes:

- page ID and contract version;
- digest of every page-owned source/config/fixture input;
- shared-foundation artifact/digest and compatibility version;
- server API and preference schema compatibility;
- toolchain and build-profile identity;
- production artifact hashes and compressed/raw sizes;
- selected test names, timestamps, durations, and outcomes;
- visual baseline IDs and review metadata;
- browser/OS identity for environment-sensitive checks;
- source commit and worktree metadata for traceability;
- receipt schema version and signature or trusted CI attestation.

A receipt is valid after merge only when recomputing its declared input
digests yields the same values. It is invalidated by:

- a changed page-owned input;
- a changed compatible foundation input in its dependency closure;
- a changed contract, API, preference, build-tool, or test-harness input;
- a missing artifact or mismatched hash;
- an expired policy-defined verification window;
- an unverifiable or untrusted attestation.

The integration coordinator verifies receipts; it does not blindly trust a
green page branch.

### 12.6 Migration-wave pipeline

The pipeline has three distinct scopes:

| Stage | What runs | What does not run |
|---|---|---|
| Page implementation | Page contract/native tests, exact satellite build, page browser/accessibility/visual tests, package checks, receipt generation | Core rebuild, unrelated satellite builds/tests, portfolio E2E |
| Integration of one completed page | Receipt/input verification, explicit route/ID and release-descriptor compatibility checks, cheap repository checks | Full workspace rebuild and cross-page E2E |
| Wave completion | One final integration gate over the fully assembled tree | Nothing is deferred beyond release |

The final full rebuild and end-to-end suite is held until all planned
page-level changes are complete in isolation and integrated. It runs once on
the exact release candidate.

This rule applies to the migration wave. It does not remove the final gate,
and it does not permit release from page receipts alone.

### 12.7 Final integration gate

`cargo make ui-final-integration` must:

1. verify the tree contains only approved integrated work;
2. validate every page contract against explicit navigation and server routes,
   including unique route/surface IDs;
3. recompute and validate every page receipt;
4. validate dependency direction;
5. rebuild the shared foundation from clean declared inputs;
6. rebuild core and every migrated satellite;
7. confirm rebuilt artifact hashes match receipt expectations where inputs
   are identical;
8. verify unrelated surface boundaries and immutable asset names;
9. run full native/workspace checks;
10. run all browser, accessibility, and reviewed visual suites against the
    assembled production artifacts;
11. run cross-page launch, auth, revocation, release-descriptor compatibility,
    and rollback journeys;
12. generate one signed release manifest and measurement report.

Failure is fixed at the narrowest owning layer. A failed final gate does not
justify rerunning unchanged page suites repeatedly while diagnosing an
unrelated issue.

### 12.8 Independent steady-state delivery

After the migration wave has passed its final gate and the satellite platform
is established, an isolated satellite release may use:

1. affected-input validation;
2. page-level tests;
3. exact production package and artifact checks;
4. compatibility and security launch tests;
5. immutable publish;
6. page-only canary/health check;
7. atomic artifact-reference promotion.

The job MUST verify that core and unrelated surface artifact hashes and
promoted artifact references did not change. Core, server, shared-foundation,
schema, explicit route, or cross-surface protocol changes use their broader
declared release gates.

### 12.9 Deployment and rollback

- Publish immutable assets before changing a promoted artifact reference.
- Verify assets by downloading/hashing or another independent read-back, not
  by trusting an upload success echo.
- Retain the previous compatible release descriptor and assets for rollback.
- Promote one surface atomically.
- Do not garbage-collect artifacts still referenced by active/recent
  promotions or boot documents.
- Record page ID, build ID, hashes, compatibility versions, and promoter.
- A failed satellite promotion does not roll back an unchanged core.

## 13. Parallel agent protocol

### 13.1 Prerequisites before fan-out

The coordinator completes and freezes:

- shared semantic tokens and opinionated patterns required by the wave;
- archetype and page-contract schema;
- release-descriptor schema and the explicit server launch/route pattern;
- child-session and preference APIs;
- page-level CI/CD commands and receipt validation;
- ownership map and protected shared paths;
- named test fixtures and visual-review process;
- the No-Hires first-conversion requirements, baseline measurements, and
  implementation guardrails.

If a required shared component is missing, add it centrally before assigning
dependent pages. Do not ask every page agent to invent a local version.

### 13.2 Core versus satellite decision

A page remains in core only if it requires ongoing in-process access to core
state or participates in a tightly coupled workflow that cannot be expressed
through authenticated server APIs and normal navigation.

The following are not reasons to keep a page in core:

- it needs the current user;
- it needs authorization;
- it needs an office list;
- it needs saved settings;
- it needs real-time server events;
- users launch it from core;
- it uses the same design system.

Those are supported satellite capabilities.

### 13.3 Agent assignment packet

Every agent receives:

- this document;
- exactly one page/surface ID;
- product acceptance criteria and current-page route;
- selected archetype;
- approved component/pattern list;
- dataset and API contract;
- state/persistence matrix;
- authorization and event rules;
- named stories and journey list;
- performance/bundle budgets;
- owned path globs and forbidden shared paths;
- exact page-level commands;
- expected receipt location.

An assignment is incomplete if the agent must guess layout, data mode,
persistence, or test scope.

### 13.4 Worktree and ownership rules

- One page per agent and one worktree/branch per page.
- Claim the page's bead before editing.
- Use a page-specific `CARGO_TARGET_DIR`.
- Edit only declared page-owned paths.
- Do not edit root lockfiles, root workspace membership, shared tokens,
  shared patterns, central translations, core router, shared explicit
  launch/session code, global CI, or release-manifest assembly.
- Do not edit shared explicit navigation or server-route registration; the
  integration coordinator owns those code changes.
- Do not weaken a test, audit ceiling, or budget to make the page pass.
- Do not commit generated/cache output except approved release descriptors,
  receipts, or reviewed baselines.
- Preserve unrelated work and verify the worktree is not behind `main` before
  asserting that a capability is missing.

The coordinator preallocates workspace/package hooks and batches explicit
navigation/server-route edits so page branches do not contend on shared files.

### 13.5 Agent implementation workflow

1. Read this document, the page assignment, repo instructions, and relevant
   framework examples.
2. Claim the page issue and confirm owned/forbidden paths.
3. Capture the current page's matched baseline where one exists.
4. Add or refine the page contract and named fixtures.
5. Write failing contract/behavior tests for the assigned acceptance criteria.
6. Implement using the selected archetype and approved framework patterns.
7. Run the smallest inner lane while iterating.
8. Run the page browser/accessibility lane when behavior is ready.
9. Run the visual lane for every changed named state and review diffs.
10. Build the exact production satellite and run it through the real launch
    and child-session boundary.
11. Run negative controls for new test/audit rules, then revert the fault.
12. Record after measurements and generate the page receipt.
13. Confirm only owned files changed; commit and push the page branch.
14. Hand the receipt and material risks to the coordinator.

### 13.6 Stop and escalate

An agent stops and reports the blocker when:

- a required shared component or token is missing;
- the archetype cannot express a real product requirement;
- a shared schema/API change is required;
- owned paths overlap another page;
- the page contract or generated release descriptor conflicts with an
  explicitly coded route/ID;
- the dataset exceeds the page's client-snapshot budget;
- auth, accessibility, data integrity, or measurement evidence fails;
- page isolation requires editing core or another satellite.

The agent does not solve a shared problem with page-local duplication.

### 13.7 Coordinator integration workflow

For each page:

1. verify branch/worktree base and changed-file ownership;
2. verify the receipt against merged inputs;
3. check route, package, and release-descriptor uniqueness;
4. integrate without broadening that page's scope;
5. rerun only cheap receipt, explicit-route, and release-descriptor checks;
6. record the integrated surface as pending final gate.

If the shared foundation must change mid-wave:

1. pause dependent page work;
2. make and verify one central foundation change;
3. publish a new foundation digest/version;
4. identify reverse-dependent page receipts;
5. rerun only those pages' required lanes;
6. resume fan-out.

After all pages are integrated, freeze the candidate and run the final
integration gate once.

## 14. Testing contract

### 14.1 Test the production artifact

Browser, visual, launch, and bundle tests MUST exercise the artifact that will
ship. A minimized test capsule is useful for inner-loop diagnostics only. Its
size or startup time is not production-satellite evidence.

### 14.2 A/B/C/D proof

Every new page behavior or audit rule follows:

- **A — Baseline:** demonstrate the relevant test passes on known-good code.
- **B — Break:** inject a targeted fault and show the intended test fails for
  the intended reason.
- **C — Fix:** revert/fix the fault and show the test passes.
- **D — Regression:** keep the test and reviewed fixture/baseline in the
  owning suite.

The injected fault must be reverted. A test that has never detected its
targeted failure mode is not proven.

### 14.3 Standard page lanes

| Lane | Purpose | Typical trigger |
|---|---|---|
| Contract/static | Page-contract validity, ownership, forbidden dependencies/classes, release-descriptor schema, and agreement with explicit routes | Every page change |
| Native inner | Pure reducer/model/filter/sort/pagination/preferences/event logic | Relevant Rust/model changes |
| Production build | Exact satellite compilation, artifact/size/hash checks | Every shippable page change |
| Browser behavior | Launch, loading, controls, table interaction, claims, events, recovery | Behavior or browser harness changes |
| Accessibility | Keyboard, focus, names, roles, contrast/axe rules | Relevant UI changes; final page gate |
| Visual | Reviewed named-state screenshots and visual audit | Any presentation-affecting change |
| Security/session | Launch, cookie scope, renewal, revocation, office authorization | Auth/session/API changes; required first-conversion gate |
| Final integration | Core/satellite launch matrix, all artifacts, cross-page compatibility | Once after the migration wave |

Visual comparison uses stable rendering and reviewed similarity thresholds,
not byte-for-byte PNG equality. Dynamic fields are fixed or masked only when
they are not the subject under test.

### 14.4 No-Hires named journeys

The No-Hires production implementation must cover at least:

1. launch from an authenticated core;
2. direct URL without a valid child secret is rejected;
3. default office snapshot loads;
4. office changes atomically while local view settings remain;
5. All Offices is explicit;
6. search, filters, sort, pagination, resize, reorder, and visibility are
   local and responsive;
7. Reset changes current local settings;
8. Save as Default persists only the allowlisted fields;
9. refresh rebuilds transient state and reloads saved defaults;
10. own claim removes a row;
11. another user's claim event removes a row;
12. conflict and transient failure recover correctly;
13. disconnect/reconnect and revision-gap resnapshot work;
14. two tabs maintain independent local state;
15. two tabs reuse cached immutable assets;
16. core logout/timeout invalidates existing satellite operations;
17. expired tabs stop reconnect/mutation loops;
18. any canonical office and All Offices can be selected, while an invalid or
    noncanonical office identifier is rejected server-side;
19. artifact rollback restores the prior page without changing core;
20. Mexico/India-like latency and constrained bandwidth remain usable.

## 15. Measurements and budgets

### 15.1 Existing prototype baseline

The completed controlled prototype recorded:

| Measure | Result |
|---|---:|
| Deterministic 2,000-row fixture SHA-256 | `79168fd44e01bdd97270a3e014ffa31e27dd8d965a0d745bbed66a00d4a5ef8f` |
| Snapshot payload, Brotli | 9,801 B |
| Test capsule, Brotli | 313,289 B |
| Legacy recorded full app, Brotli | 3,305,899 B |
| Controlled time to rows, capsule | 149.6 ms |
| Controlled time to rows, legacy full app | 1,656.5 ms |
| Search | 27.5 ms |
| Filter | 33.3 ms |
| Sort | 33.2 ms |
| Pagination | 33.2 ms |
| Claim removal | 32.9 ms |
| Server-event removal | 33.3 ms |
| Measured heap, capsule | 2,783,932 B |
| Measured heap, legacy harness | 6,639,949 B |
| Warm native affected lane | 49 tests / 10.746 s |
| Browser lane | 4 tests / 22.089 s |
| Visual lane | 2 tests / 99.466 s |
| Selected total | 55 tests / 132.301 s |
| Named/reviewed UI evidence | 16 named states / 13 reviewed baselines |

The current real production Wasm grew from 3,305,899 Brotli bytes to
3,480,254 bytes, an increase of 174,355 bytes or approximately 5.27%.
That growth occurred because the shipping No-Hires code remained linked into
the monolith. It is evidence of the problem, not the expected result of a
production satellite.

The capsule-versus-full startup comparison is directional evidence only. It
did not hold the production routing, authentication, network, and asset
topology constant.

### 15.2 Required matched production comparison

Measure the current production-shaped monolithic route immediately before
extraction and the production-shaped satellite immediately after extraction
using the same:

- data fixture and snapshot bytes;
- browser/host;
- cold-cache and warm-cache definitions;
- network latency/bandwidth profiles;
- server/API behavior;
- authentication path;
- instrumentation and sample count.

Record at minimum:

- core raw, gzip, and Brotli bytes;
- satellite raw, gzip, and Brotli bytes;
- JS glue and CSS bytes;
- cold and warm bytes transferred;
- cache reuse for a second tab;
- navigation-to-first-row and navigation-to-interactive p50/p95;
- local search/filter/sort/page p50/p95;
- office-switch bytes and time;
- heap after load and after repeated interactions;
- build, package, and each test-lane duration, cold and warm;
- session launch/renew/revoke latency;
- event-to-row-removal latency;
- server CPU/memory and error rate under child-session concurrency.

Store the report next to the page receipt and retain the raw machine-readable
measurements.

### 15.3 Initial No-Hires implementation guardrails

These are enforced implementation and operating guardrails, not an
architecture-adoption vote. Use the matched production run to tighten them and
to optimize the implementation. Any relaxation requires an explicit owner
decision and recorded evidence.

| Measure | Initial budget |
|---|---:|
| Core Brotli Wasm after extraction | No more than 1% above the recorded monolithic 3,305,899 B baseline, with the difference explained |
| No-Hires satellite Brotli Wasm | At most 500 KiB |
| Local search/filter/sort/page p95 on 2,000 rows | At most 100 ms |
| Warm native page lane | At most 20 s |
| Cached browser plus visual page lanes | At most 3 min |
| Second tab Wasm transfer | 0 additional Wasm payload when the immutable artifact is cached |
| Expired child-session cleanup | Within 60 s of policy expiry |
| Core logout to mutation rejection | Within 5 s |
| 1,000 concurrent child sessions | Under 1% request errors, no cross-session events, API p95 under 1 s |
| 4,000 concurrent child sessions | Characterize resource curve and failure behavior before setting a release budget |

All performance comparisons report sample count and variance, not one
best-case run.

### 15.4 Per-page migration measurements

Every converted page records before and after:

- linked core and page bytes;
- first useful render;
- interaction latency;
- page-owned build/test duration;
- full-gate contribution;
- number of local component/style exceptions removed;
- number of named states and reviewed baselines;
- accessibility findings;
- transferred bytes for the normal dataset;
- any new shared-foundation dependency.

### 15.5 Production telemetry

After opt-in/privacy review, collect aggregate real-user measures by broad
region/network class, particularly Mexico and India:

- launch and first-row p75/p95;
- office-switch bytes/time;
- local interaction latency;
- satellite boot failures;
- child-session expiry/reconnect outcomes;
- snapshot/event resync frequency;
- asset cache-hit effectiveness.

Do not collect row content, search text, filter values, or public tab IDs in
analytics.

## 16. Full-scale implementation program

All phases below are approved implementation work. Phase boundaries coordinate
dependencies and keep feedback local; they are not architecture-selection or
authorization gates.

### Phase 0 — Implement shared foundation and CI/CD

- implement the page-contract and release-descriptor schemas;
- implement/finalize the required opinionated patterns;
- implement explicit navigation/server-route conventions and satellite
  launch/session support;
- add separate `user-state.sqlite` preference storage and APIs;
- implement page-level build/test/package/receipt commands;
- protect shared paths and validate ownership;
- implement final integration assembly and gate;
- freeze versioned inputs for the wave.

### Phase 1 — First production conversion: No-Hires

- move the shipping No-Hires route to an independent production artifact;
- remove its code from core linkage;
- use the real child-session and preference boundaries;
- run the named journeys and A/B/C/D controls;
- capture the matched before/after report;
- verify core and unrelated artifacts remain unchanged on a page-only edit;
- canary, roll back, and promote the page independently;
- apply the measured lessons to budgets, tooling, and subsequent conversions.

### Phase 2 — Portfolio conversion wave

- incorporate the No-Hires production lessons without reopening the adopted
  architecture;
- version and freeze the shared foundation;
- classify every remaining page and convert every satellite-eligible page
  using an approved archetype;
- give each agent one complete assignment packet and worktree;
- generate page receipts from isolated page gates;
- resolve shared gaps centrally, not inside page branches.

### Phase 3 — Integrate without portfolio rebuilds

- merge completed page branches one at a time;
- validate ownership, receipts, compatibility, explicit route/ID agreement,
  and release-descriptor uniqueness;
- do not rebuild core, unrelated pages, or run cross-page E2E after each
  integration;
- freeze the final assembled candidate only after all page work is integrated.

### Phase 4 — One final full gate

- run `cargo make ui-final-integration` once on the complete release
  candidate;
- fix failures at their owning layer and invalidate only affected receipts;
- rerun the final gate on the corrected candidate;
- publish the signed release manifest and measurement report.

### Phase 5 — Rollout and steady state

- canary satellite artifacts independently;
- monitor launch, RUM, auth/session, and event health;
- use page-only delivery for isolated satellite changes;
- use dependency-selected broader gates for shared/protocol changes;
- keep periodic portfolio/full-gate runs as defense in depth.

## 17. Decisions agents must not reopen

Unless the owner explicitly changes this document:

- Full-scale implementation is approved and in progress; No-Hires is the first
  production conversion, not an experiment or adoption gate.
- No-Hires is a production satellite, not only a test capsule.
- Satellites open in their own tabs.
- User-facing routes do not contain `/satellites/`.
- The core owns the master session; satellites use minimal tab-scoped child
  sessions.
- The URL contains only a public tab ID; the secret is in an exact-path
  `Secure`/`HttpOnly` cookie.
- Core logout/timeout invalidates child sessions; satellites have no logout.
- Satellite transient state stays inside its Wasm instance.
- Durable user settings use separate server-side `user-state.sqlite`.
- Persistence requires an explicit **Save as Default** action.
- Office is a dataset selector and may select any canonical office without an
  office-specific restriction; it is not a filter.
- Filters and sorts are local and survive office changes.
- The No-Hires layout uses a slim filter utility row above a full-width table,
  with one-to-one column controls aligned in the table's second header row.
- Opinionated tables use the semantic dark-blue header, light-blue filter
  band, faint full grid, opt-in zebra, and sort-stable column-track contract.
- Office-sized snapshots use local search/filter/sort/pagination.
- Satellites do not import core or each other.
- Page contracts and macros validate and document; they are not a generic page
  generator.
- Page implementation, navigation, launch actions, and server routes are
  explicit searchable Rust code; there is no runtime page registry or
  data-driven UI/navigation system.
- Release descriptors, receipts, and generated indexes exist only for CI/CD
  validation, packaging, promotion, and rollback.
- Page agents do not modify frozen shared foundation or coordinator-owned
  navigation/server-route integration files.
- Page-level gates run during fan-out.
- The complete rebuild and cross-page end-to-end gate is held until the end of
  the integrated migration wave.
- Exact production artifacts, not test-only capsules, determine compliance
  with bundle, launch, and operating guardrails.

## 18. Implementation and operating success measures

The implementation is complete and operating successfully when:

- No-Hires ships as the first real independent surface and meets its
  production guardrails;
- every scheduled satellite-eligible page has been converted or has an
  explicit documented reason to remain in core;
- a No-Hires-only change leaves core and unrelated artifact hashes unchanged;
- a second tab reuses immutable page assets while maintaining independent
  local state;
- logout and timeout reliably invalidate all child tabs;
- agents implement routine pages primarily from approved patterns and one
  archetype;
- at least 99% of routine composition requires no page-local control or layout
  invention;
- page contracts make persistence, state, data mode, and tests mechanically
  verifiable by CI;
- isolated page feedback completes in minutes rather than requiring the full
  portfolio gate;
- multiple page agents integrate without shared-file conflicts;
- the final assembled release passes one authoritative full gate;
- low-bandwidth production telemetry confirms useful responsiveness;
- page-only steady-state releases rebuild and deploy only the changed
  satellite.

## 19. Relevant references

### Local sources of truth

- `AGENTS.md` — repository workflow and verification rules
- `doc/visual-quality/` — visual-quality policy and audit rules
- `audit/` — reusable browser-audit implementation
- `xtask/` — build/test/package logic
- `C:\Users\david\.claude\rust-ci-cd-build-strategy.md` — Rust build and
  CI/CD layering
- `C:\dev\4iiz-Office\TEST_STRATEGY.md` — Office test lanes and release-gate
  policy; update it during Phase 0 to match this architecture

### External design and implementation references

- [Leptos components and typed props](https://book.leptos.dev/view/03_components.html)
- [Leptos component children and composition](https://book.leptos.dev/view/09_component_children.html)
- [Leptos Wasm code splitting](https://book.leptos.dev/deployment/binary_size.html#code-splitting)
- [Trunk Rust targets and `data-bin`](https://trunkrs.dev/assets/#rust)
- [daisyUI design model](https://daisyui.com/docs/intro/)
- [daisyUI semantic colors](https://daisyui.com/docs/colors/)
- [GOV.UK Design System patterns](https://design-system.service.gov.uk/patterns/)
- [Storybook stories as captured component states](https://storybook.js.org/docs/get-started/whats-a-story)
- [Cargo test selection](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Cargo build timings](https://doc.rust-lang.org/stable/cargo/reference/timings.html)
- [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)

The external links explain underlying tools and design-system concepts. This
document remains authoritative for 4iiz decisions.
