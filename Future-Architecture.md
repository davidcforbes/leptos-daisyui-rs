# Future Architecture: Opinionated Pages and Independent Wasm Satellites

**Status:** Owner-approved target architecture

**Last updated:** 2026-08-26

**Scope:** `leptos-daisyui-rs`, `4iiz-Office`, and `4iiz-Inventory`

**Audience:** Codex, Claude Code, reviewers, and the migration coordinator

**Tracking:** framework architecture `ldui-pwx`; No-Hires pilot `op-vzoiv`

## 1. Authority and purpose

This document is the canonical contract for building and converting 4iiz web
pages. Give an AI coding agent this document together with one page assignment
and that page's product requirements.

This document supersedes the earlier recommendation to retain one shipping
Office Wasm application. The No-Hires test capsule proved useful isolation, but
it was not a production satellite. The next step is to implement No-Hires as a
real independently built, loaded, authenticated, tested, and deployed
satellite. If the production pilot satisfies the measurements in this
document, pages that do not require in-process interaction with the core will
be converted in parallel using the same contract.

The architecture has two equally important goals:

1. Make the visual and behavioral result explicit enough that agents do not
   invent page structure or component behavior.
2. Make each independent page a build, test, artifact, and deployment unit so
   changing it does not rebuild the core or unrelated pages.

The CI/CD changes described here are prerequisites, not optional follow-up
work. Page isolation is not real if the pipeline still rebuilds and retests the
portfolio for every page edit.

## 2. Decision status

### 2.1 What the existing pilot proved

The No-Hires pilot established that:

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

### 2.2 What the existing pilot did not prove

The current production `/no-hires` route is still part of the monolithic
`office-perf-web` bundle. The test capsule:

- is not served by the production route;
- does not use the production child-session lifecycle;
- is not independently released or rolled back;
- does not prove that a core release can remain byte-for-byte unchanged;
- does not provide an apples-to-apples production startup comparison.

The existing measurements therefore prove test isolation and page behavior.
They do not yet prove production delivery isolation.

### 2.3 Corrected architectural verdict

Implement No-Hires as the first production satellite. Measure it against the
current production route. Do not generalize the result to every page until
that comparison is complete.

The target portfolio is:

- a small core for master authentication, session ownership, navigation, and
  truly shared in-process workflows;
- many independently compiled satellite pages for bounded workflows;
- server-owned manifests and generic launch routing so adding a satellite
  does not rebuild the core;
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
- Prove with artifact hashes that unrelated surfaces did not change.
- Run expensive tests only when their declared inputs or behaviors can change.
- Keep the final full release gate authoritative without paying its cost after
  every isolated page edit.

### 3.2 Non-goals

- A generic low-code page builder or runtime JSON UI renderer.
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
| **Surface manifest** | Server-readable metadata mapping a stable route to an immutable artifact and declaring compatibility and launch policy. |
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

server launch/session/static-asset layer
        ├────────► reads surface manifests
        ├────────► launches core
        └────────► launches each satellite
```

The following dependencies are forbidden:

```text
core ──X──► satellite
satellite ──X──► core
satellite A ──X──► satellite B
```

A satellite MAY depend on a small versioned shared crate. It MUST NOT depend
on the core application's page modules, router, global app state, or compiled
artifact. The core MUST NOT contain a compile-time registry of satellite
pages.

### 5.2 Server-provided page directory

The core's page launcher reads a server-provided list of authorized surfaces.
The list contains labels, icons, launch URLs, and authorization metadata. It
is data, not Rust source linked into the core.

Therefore:

- registering a new satellite does not rebuild core Wasm;
- changing a satellite label or artifact does not rebuild core Wasm;
- removing authorization from a user does not require a browser artifact
  rebuild;
- the server remains authoritative about which launch links a user receives.

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
and content-addressed. A surface manifest atomically points the stable route
to one compatible artifact set.

### 5.4 Independent production artifacts

Each surface produces its own:

- optimized Wasm;
- JavaScript loader;
- CSS needed by that surface;
- immutable asset directory;
- source map and symbol archive, where policy permits;
- surface manifest;
- software bill of materials or dependency record;
- page receipt.

Publishing No-Hires MUST NOT rewrite the core or another satellite's asset
directory. Promotion changes only the No-Hires manifest pointer. Rollback
restores only that pointer.

The server's generic static and manifest layer MUST discover surface
manifests without compiling a central Rust match statement. Suitable
implementations include a manifest directory or immutable object-store
prefix assembled during release.

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
  server/                         # generic launch/API/static-manifest layer
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
      surface.toml
      src/
      tests/
    inventory-aging/
      Cargo.toml
      Cargo.lock
      Trunk.toml
      surface.toml
      src/
      tests/
  receipts/                       # generated release inputs
```

The exact directories may differ, but these properties are mandatory:

- a page build names its own `--manifest-path` and Trunk target;
- a satellite's dependency graph has no core or sibling page package;
- adding a page does not require editing a central Rust module or enumerated
  Cargo workspace member list;
- each independently released surface has a reproducible dependency lock
  boundary, or an equivalently isolated generated lock artifact;
- package discovery reads per-page manifests;
- page output and cache directories are page-specific;
- the final integration task enumerates discovered surfaces and invokes their
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
| `FilterBar` | Horizontal filter row above content; search, filter controls, active-filter summary, Reset, and Save as Default. |
| `EntityTable` | Client-snapshot table with local filter/sort/page, resizable/reorderable/hideable columns, stable row identity, empty/loading/error states, and keyboard/accessibility behavior. |
| `ServerDataTable` | Explicit server-query table for datasets that cannot be safely or efficiently loaded as a snapshot. |
| `PageStatePanel` | Consistent loading, empty, no-results, error, expired-session, and forbidden presentation. |
| `ActionFeedback` | Pending, success, recoverable conflict, stale-row, and failure behavior for row actions. |

Patterns own spacing, alignment, responsive collapse, labels, keyboard
behavior, and state presentation. Page code supplies typed content and domain
callbacks.

`EntityTable` and the existing server-oriented DataTable MUST converge on
shared column, cell, resize, reorder, visibility, pagination, and
accessibility contracts where practical. They MUST remain explicit about
their data mode. Silently switching between client and server filtering is
forbidden. Framework tracking issue `ldui-aqo` owns this convergence.

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
| `leptos-daisyui-rs` | Tokens, primitives, opinionated patterns, archetypes, contract/registry schemas, browser audit rules, reference states, and framework verification. |
| `4iiz-Office` | Office core, Office satellite domain code, snapshot/mutation/event APIs, child sessions, Office preferences, production surface manifests/receipts, and Office delivery orchestration. |
| `4iiz-Inventory` | Inventory core/satellites, Inventory domain APIs and adapters, Inventory manifests/receipts, and Inventory delivery orchestration. |

A consumer needing a generally reusable component opens or claims one central
framework change. It does not copy the component into Office and Inventory.
A product-specific cell, reducer, query adapter, or label remains in the
consumer.

## 9. Page contracts

Every new or converted page MUST declare a contract next to the page. The
exact macro syntax may evolve, but it must express the following information
without relying on prose.

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

### 9.1 Per-page surface manifest

Each satellite owns a manifest in its own directory. Agents do not add entries
to a central source file.

```toml
id = "no-hires"
contract_version = 1
delivery = "satellite"
route = "/no-hires/"
artifact = "dist/no-hires/<build-id>/"
api_compat = "office-page-v1"
foundation_compat = "ldui-page-v1"
preference_schema = 1
launch_policy = "office.no-hires.read"
```

Packaging fills immutable hashes and sizes into generated release metadata.
The server discovers manifests. Duplicate IDs/routes, incompatible versions,
missing artifacts, or mutable asset names fail assembly.

## 10. `SnapshotTablePage` contract

### 10.1 Required layout

From top to bottom:

1. `PageHeader`
2. office `DatasetSelector`
3. optional full-width `KpiStrip`
4. horizontal `FilterBar`
5. active status/error feedback when needed
6. full-width `EntityTable`
7. table pagination integrated with the table pattern

Office is not placed among the filters. The filter row is horizontal at
desktop widths and follows the framework's defined responsive wrapping at
narrow widths. Agents do not invent a sidebar alternative for this archetype.

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

## 11. Registry and mechanical enforcement

The framework publishes machine-readable metadata for components, patterns,
archetypes, tokens, and audit rules. For each approved item, the registry
records:

- stable name and version;
- import path;
- intended use and prohibited use;
- typed properties and variants;
- accessibility contract;
- responsive behavior;
- named states;
- compatible archetypes;
- examples;
- test selectors;
- deprecation/replacement metadata.

CI enforces:

- no raw hex/rgb colors in consumer page code;
- no unapproved DaisyUI component classes in domain pages;
- no arbitrary breakpoint/spacing values outside the foundation;
- no duplicate local reimplementation of registered patterns;
- every page has a valid contract and manifest;
- every contract state has a fixture/story or a documented nonvisual test;
- every visual baseline has review metadata;
- a satellite has no forbidden dependency edge;
- the core has no compile-time satellite registry;
- persisted state is allowlisted;
- source ownership is unambiguous.

Enforcement should produce a direct diagnostic and approved replacement, not
merely fail on a style string.

## 12. CI/CD architecture

### 12.1 Governing principle

Keep orchestration in the existing `cargo-make`/CI runner. Put
project-specific build, manifest, hashing, compatibility, packaging, and
verification logic in the Rust `xtask`. Steps are idempotent and inspect their
required outputs.

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
cargo xtask assemble-surface-manifest <receipt-directory>
cargo make ui-final-integration
cargo make ship-page -- no-hires
```

`verify-page` resolves the contract and ownership metadata, runs the selected
lane, and records timings. `package-page` builds the exact production
satellite, not a test-only substitute. `ship-page` packages, verifies,
publishes immutable assets, performs compatibility checks, and atomically
promotes only that page's manifest.

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
| Frozen shared foundation, root workspace, central CI, generic server routing, or unknown ownership | Fail with an ownership/foundation error. Do **not** silently run the full gate. |

The last row is deliberate. Page agents are not allowed to change shared
inputs during the fan-out. Escalating to a full gate would hide an ownership
violation and recreate the month-long feedback loop.

In steady state, an approved shared-foundation change selects the reverse
dependency set declared by surface manifests. It can select many surfaces if
the shared behavior genuinely affects many surfaces. Unknown ownership still
fails closed.

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
| Integration of one completed page | Receipt/input verification, manifest collision/compatibility checks, cheap repository checks | Full workspace rebuild and cross-page E2E |
| Wave completion | One final integration gate over the fully assembled tree | Nothing is deferred beyond release |

The final full rebuild and end-to-end suite is held until all planned
page-level changes are complete in isolation and integrated. It runs once on
the exact release candidate.

This rule applies to the migration wave. It does not remove the final gate,
and it does not permit release from page receipts alone.

### 12.7 Final integration gate

`cargo make ui-final-integration` must:

1. verify the tree contains only approved integrated work;
2. validate every page contract and unique route/surface ID;
3. recompute and validate every page receipt;
4. validate dependency direction;
5. rebuild the shared foundation from clean declared inputs;
6. rebuild core and every migrated satellite;
7. confirm rebuilt artifact hashes match receipt expectations where inputs
   are identical;
8. prove unrelated surface boundaries and immutable asset names;
9. run full native/workspace checks;
10. run all browser, accessibility, and reviewed visual suites against the
    assembled production artifacts;
11. run cross-page launch, auth, revocation, manifest, compatibility, and
    rollback journeys;
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
7. atomic manifest promotion.

The job MUST verify that core and unrelated surface artifact hashes and
manifest pointers did not change. Core, server, shared-foundation, schema, or
cross-surface protocol changes use their broader declared release gates.

### 12.9 Deployment and rollback

- Publish immutable assets before changing a manifest pointer.
- Verify assets by downloading/hashing or another independent read-back, not
  by trusting an upload success echo.
- Retain the previous compatible manifest and assets for rollback.
- Promote one surface atomically.
- Do not garbage-collect artifacts still referenced by active/recent
  manifests or boot documents.
- Record page ID, build ID, hashes, compatibility versions, and promoter.
- A failed satellite promotion does not roll back an unchanged core.

## 13. Parallel agent protocol

### 13.1 Prerequisites before fan-out

The coordinator completes and freezes:

- shared semantic tokens and opinionated patterns required by the wave;
- archetype and page-contract schema;
- manifest schema and generic server discovery;
- child-session and preference APIs;
- page-level CI/CD commands and receipt validation;
- ownership map and protected shared paths;
- named test fixtures and visual-review process;
- the production No-Hires pilot and its measured acceptance decision.

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
  shared patterns, central translations, core router, generic launch/session
  code, global CI, or release-manifest assembly.
- Do not add the page to a hand-maintained central registry.
- Do not weaken a test, audit ceiling, or budget to make the page pass.
- Do not commit generated/cache output except approved manifests, receipts, or
  reviewed baselines.
- Preserve unrelated work and verify the worktree is not behind `main` before
  asserting that a capability is missing.

The coordinator preallocates workspace/package hooks or uses discovery so
page branches do not contend on shared files.

### 13.5 Agent implementation workflow

1. Read this document, the page assignment, repo instructions, and relevant
   framework examples.
2. Claim the page issue and confirm owned/forbidden paths.
3. Capture the current page's matched baseline where one exists.
4. Add or refine the page contract and named fixtures.
5. Write failing contract/behavior tests for the assigned acceptance criteria.
6. Implement using the selected archetype and registered patterns.
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
- a manifest route/ID collides;
- the dataset exceeds the page's client-snapshot budget;
- auth, accessibility, data integrity, or measurement evidence fails;
- page isolation requires editing core or another satellite.

The agent does not solve a shared problem with page-local duplication.

### 13.7 Coordinator integration workflow

For each page:

1. verify branch/worktree base and changed-file ownership;
2. verify the receipt against merged inputs;
3. check route, package, and manifest uniqueness;
4. integrate without broadening that page's scope;
5. rerun only cheap receipt/manifest checks;
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
| Contract/static | Page-contract validity, ownership, forbidden dependencies/classes, manifest schema | Every page change |
| Native inner | Pure reducer/model/filter/sort/pagination/preferences/event logic | Relevant Rust/model changes |
| Production build | Exact satellite compilation, artifact/size/hash checks | Every shippable page change |
| Browser behavior | Launch, loading, controls, table interaction, claims, events, recovery | Behavior or browser harness changes |
| Accessibility | Keyboard, focus, names, roles, contrast/axe rules | Relevant UI changes; final page gate |
| Visual | Reviewed named-state screenshots and visual audit | Any presentation-affecting change |
| Security/session | Launch, cookie scope, renewal, revocation, office authorization | Auth/session/API changes; required pilot gate |
| Final integration | Core/satellite launch matrix, all artifacts, cross-page compatibility | Once after the migration wave |

Visual comparison uses stable rendering and reviewed similarity thresholds,
not byte-for-byte PNG equality. Dynamic fields are fixed or masked only when
they are not the subject under test.

### 14.4 No-Hires named journeys

The production pilot must cover at least:

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

### 15.1 Existing pilot evidence

The existing controlled pilot recorded:

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

### 15.3 No-Hires production-satellite acceptance budgets

These are initial budgets for proving the architecture. Tighten them after
the matched run; do not silently relax them.

| Measure | Initial budget |
|---|---:|
| Core Brotli Wasm after extraction | No more than 1% above the pre-pilot 3,305,899 B baseline, with the difference explained |
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

## 16. Migration sequence

### Phase 0 — Shared foundation and CI/CD

- finalize the page-contract and manifest schemas;
- implement/finalize the required opinionated patterns;
- add generic manifest discovery and satellite launch/session support;
- add separate `user-state.sqlite` preference storage and APIs;
- implement page-level build/test/package/receipt commands;
- protect shared paths and validate ownership;
- implement final integration assembly and gate;
- freeze versioned inputs for the wave.

### Phase 1 — Real No-Hires production satellite

- move the shipping No-Hires route to an independent production artifact;
- remove its code from core linkage;
- use the real child-session and preference boundaries;
- run the named journeys and A/B/C/D controls;
- capture the matched before/after report;
- verify core and unrelated artifacts remain unchanged on a page-only edit;
- canary, roll back, and promote the page independently;
- review results before authorizing broad fan-out.

### Phase 2 — Freeze and fan out

- incorporate only lessons proven by the No-Hires production pilot;
- version and freeze the shared foundation;
- select bounded pages that fit approved archetypes;
- give each agent one complete assignment packet and worktree;
- generate page receipts from isolated page gates;
- resolve shared gaps centrally, not inside page branches.

### Phase 3 — Integrate without portfolio rebuilds

- merge completed page branches one at a time;
- validate ownership, receipts, compatibility, and manifest uniqueness;
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

- canary satellite manifests independently;
- monitor launch, RUM, auth/session, and event health;
- use page-only delivery for isolated satellite changes;
- use dependency-selected broader gates for shared/protocol changes;
- keep periodic portfolio/full-gate runs as defense in depth.

## 17. Decisions agents must not reopen

Unless the owner explicitly changes this document:

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
- The No-Hires layout uses a horizontal filter row above a full-width table.
- Office-sized snapshots use local search/filter/sort/pagination.
- Satellites do not import core or each other.
- Page contracts and macros validate and document; they are not a generic page
  generator.
- Page agents do not modify frozen shared foundation or central registries.
- Page-level gates run during fan-out.
- The complete rebuild and cross-page end-to-end gate is held until the end of
  the integrated migration wave.
- Exact production artifacts, not test-only capsules, determine bundle and
  launch acceptance.

## 18. Success measures

The architecture succeeds when:

- No-Hires ships as a real independent surface and meets its accepted
  production budgets;
- a No-Hires-only change leaves core and unrelated artifact hashes unchanged;
- a second tab reuses immutable page assets while maintaining independent
  local state;
- logout and timeout reliably invalidate all child tabs;
- agents implement routine pages primarily from approved patterns and one
  archetype;
- at least 99% of routine composition requires no page-local control or layout
  invention;
- page contracts make persistence, state, data mode, and tests mechanically
  discoverable;
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
