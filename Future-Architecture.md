# Future Architecture for leptos-daisyui-rs

**Status:** Proposed

**Date:** 2026-08-25

**Scope:** `leptos-daisyui-rs`, `4iiz-office`, and `4iiz-inventory`

## Executive summary

`leptos-daisyui-rs` has broad component coverage, but it does not yet provide
enough enforced decisions about how complete application pages should be
assembled. AI coding agents therefore make the same design, composition, state,
and testing decisions independently on every screen. The result is duplicated
components, inconsistent layouts, large screen modules, visual drift, slow
feedback, and repeated rework.

The recommended architecture adds an opinionated composition layer between
components and application screens:

```text
semantic tokens
      |
      v
low-level primitives
      |
      v
opinionated patterns
      |
      v
page archetypes/templates
      |
      v
small domain-specific slots
```

A machine-readable UI contract should drive the entire development workflow:

```text
component registry
      +--> generated documentation
      +--> page scaffolding
      +--> agent skills and local instructions
      +--> dependency/impact graph
      +--> state, accessibility, and visual tests
```

The first vertical slice should make the existing full DataTable canonical and
build a `ListPage` template from `PageHeader`, `KpiStrip`, `FilterBar`,
`AsyncDataSection`, and `EntityTable<T>`. Inventory Explorer and Office Queue
should be its first two consumers.

For build performance, the project should combine semantic affected-test
selection with independently linked Wasm test capsules. Leptos lazy-route
splitting helps browser delivery size but happens after the complete Wasm link,
so it does not solve the current link-time bottleneck by itself.

## Goals

- Cover approximately 99% of recurring UI behavior with supported patterns and
  page templates.
- Make page appearance and behavior explicit before implementation.
- Give Codex and Claude Code one unambiguous, versioned implementation path.
- Prevent application-local reinvention of tables, filters, pagers, stat cards,
  section cards, and common page states.
- Select only the tests affected by a change during local development while
  retaining complete release and scheduled gates.
- Prevent a page or component test from linking every unrelated page and heavy
  framework dependency.
- Preserve typed Rust APIs and compile-time validation.

## Non-goals

- Replacing Leptos or daisyUI.
- Creating a general-purpose runtime UI language capable of arbitrary markup.
- Creating a separate crate or Cargo feature for every individual component.
- Eliminating complete test runs from release and scheduled CI.
- Forcing domain-specific charts and operational visualizations into generic
  components.
- Treating screenshot baselines as the source of design intent.

## Research findings

### leptos-daisyui-rs

The framework already contains 116 component directories and substantial
mechanical functionality. The full
[`components::DataTable`](src/components/data_table/component.rs) already
supports:

- resizable columns;
- typed sorting;
- per-column filtering;
- pagination and responsive page sizing;
- search and a column chooser;
- selection, activation, and inspection;
- custom and action cells;
- client-side and server-driven query models.

The server-driven API is represented by
[`TableQuery`](src/components/data_table/server_component.rs), and the column
model is defined in [`types.rs`](src/components/data_table/types.rs).

The problem is therefore not an absence of table mechanics. It is ambiguity and
insufficient policy:

- A second public [`widgets::DataTable`](src/widgets/data_table.rs) accepts
  `Vec<Vec<String>>`, leaving agents to choose between overlapping APIs.
- Other public concepts such as Toolbar and Sparkline also overlap.
- [`QuickFilterRow`](src/widgets/quick_filter_row.rs) includes raw controls,
  hard-coded emerald styling, runtime class leaking, and broad class override
  escape hatches.
- Public `class` props and `DataTableClasses` allow valid but visually unrelated
  screens.
- Component counts in README, CLAUDE.md, and Component_Library.md disagree.
- Only a small fraction of components have dedicated usage documentation.
- Consumer Tailwind source configuration depends on manually copied class
  information.

The visual-quality documentation already records the most important testing
limitation: a page can use legal computed styles while still selecting the
wrong component variant. There is no machine-readable design intent against
which an audit can validate the selection. See
[`default-component-not-specced.md`](doc/visual-quality/default-component-not-specced.md).

### 4iiz-office

Office demonstrates the cost of leaving page composition to each screen:

- The web application consumes a vendored framework copy in
  `vendor/leptos-daisyui-rs`, so framework fixes require an explicit vendor-sync
  and provenance check.
- The application contains approximately 270 local Leptos component functions.
- Several major screen modules are between 1,400 and 2,041 lines.
- Only two actual full `components::DataTable` instances were found, while many
  screens manually assemble tables, filters, pagination, row identity, and row
  actions.
- Application-local StatCard, SectionCard, filter, pager, and pagination
  variants have proliferated.
- The repository's own `screens/queue_analysis.md` describes table behavior
  being rebuilt despite most of it already existing in the framework.

Office has meaningful visual, DOM/model, accessibility, and effect testing.
However, much of it was introduced after screens had already been composed.
Regression baselines consequently protect an implementation but do not prove
that it matches the intended design reference. Ordinary page baselines at
800x600 also provide a materially different design surface from the larger
operational mockups and the 1440x900 style-audit viewport.

### 4iiz-inventory

Inventory is closer to the intended direction:

- It uses the full DataTable on seven screens.
- Inventory Explorer is the strongest existing candidate for a canonical
  `ListPage` reference implementation.
- It has strong in-progress visual-state and model/DOM testing.

The remaining issues are still structural:

- Major screens range from approximately 740 to 3,159 lines.
- Metrics, cards, page sections, and page geometry are frequently composed
  locally.
- Every page is referenced by one router and linked into one Wasm binary.
- The fast verification lane is not automatically scoped from the changed
  files and their semantic consumers.

The checked-in architecture baselines recorded:

- 846.48 seconds for a full verification;
- 113.95 seconds for the engine unit suite;
- 211.14 seconds for the server API suite;
- 377.55 seconds for a warmed full gate;
- desired scoped engine and web commands that had not yet been implemented in
  the recorded baseline.

See `C:\dev\4iiz-inventory\docs\operations\future-architecture-baseline.md`
and
`C:\dev\4iiz-inventory\docs\operations\architecture-compliance-baseline.md`.

## Root-cause model

The month-long UI effort and repeated agent errors are a system-design problem,
not merely a model-quality problem.

The primary causes are:

1. Too many equally valid low-level choices.
2. No typed page-level composition contract.
3. Duplicate and overlapping public components.
4. Broad styling escape hatches.
5. Stale and contradictory documentation.
6. Regression tests without a separate design-intent oracle.
7. Large always-loaded agent instruction files.
8. Slow compilation and monolithic Wasm linking between corrections.
9. Framework version drift between live path dependencies and vendored copies.
10. Reactive promotion of app-local fixes into the framework rather than a
    planned pattern architecture.

## Target framework architecture

### Layer 0: semantic foundations

Extend the token system from raw values to organizational roles:

- page canvas, raised surface, and inset surface;
- primary, secondary, muted, and critical text;
- compact and comfortable density;
- control height and table row height;
- page maximum width and section spacing;
- positive and negative business outcomes;
- increasing and decreasing trends, which are not always equivalent to good
  and bad outcomes;
- loading, stale, incomplete, unavailable, and permission-denied states.

Application screens should not choose raw colors, arbitrary padding, one-off
border radii, or independent typography scales.

### Layer 1: primitives

The existing daisyUI wrappers remain the low-level implementation vocabulary.
They should primarily be used inside the framework's patterns rather than
directly throughout application screen modules.

Primitives should continue to provide:

- typed variants and sizes;
- accessible structure;
- semantic theme colors;
- predictable class merging;
- stable DOM hooks for testing.

### Layer 2: opinionated patterns

The initial pattern catalog should include:

| Pattern | Owned decisions |
|---|---|
| `PageHeader` | Breadcrumbs, title, subtitle, primary/secondary actions, responsive wrapping |
| `Section` | Heading hierarchy, spacing, optional actions, surface treatment |
| `KpiStrip` | Metric order, density, value formatting, context, trend, drill-down, loading |
| `FilterBar` | Filter layout, apply/reset, URL persistence, active count, narrow-screen collapse |
| `ActiveFilterChips` | Active-state summary, individual removal, clear-all behavior |
| `EntityTable<T>` | Typed columns, sorting, resizing, pagination, query ownership, actions, selection |
| `AsyncDataSection` | Never-loaded, loading, loaded, empty, partial, stale, error, and denied states |
| `ActionBar` | Primary, secondary, destructive, and overflow action hierarchy |
| `BulkActionBar` | Selection summary, bulk actions, progress, and clearing selection |
| `EntitySummary` | Description-list/property-grid layout and responsive behavior |
| `FormSection` | Field grouping, help/error placement, save/cancel behavior |
| `MasterDetail` | List/detail sizing, selection, route state, and narrow-screen transition |
| `ChartPanel` | Heading, legend, date range, loading/empty/error states, data table alternative |
| `TimelineSection` | Temporal grouping, density, status semantics, and overflow |

These patterns must own layout, responsiveness, accessibility, keyboard
behavior, state rendering, and semantic styling. A pattern that only wraps a
`div` does not reduce the decision surface sufficiently.

#### KpiStrip contract

Each KPI should describe:

- label;
- value and formatter;
- comparison or context;
- business intent;
- trend direction;
- help text;
- optional drill-down action;
- loading and unavailable representations.

The pattern decides geometry, density, wrapping, typography, and responsive
behavior.

#### FilterBar contract

The pattern should own:

- supported filter control types;
- apply and reset semantics;
- active-filter count and chips;
- URL/query persistence;
- keyboard order;
- responsive collapse into a drawer or disclosure;
- distinction between no data, no matching results, and unavailable data.

#### EntityTable contract

The existing full DataTable should become the implementation foundation rather
than creating another independent table.

The canonical wrapper should provide:

- typed `TableSpec<T>` columns rather than string maps;
- resizing, sorting, and pagination by default;
- one controlled query model for client and server modes;
- standard filtering and column chooser integration;
- standard row actions and bulk actions;
- standard loading, never-loaded, empty, partial, stale, error, and permission
  states;
- narrow, reviewed styling extension points.

The simple widget DataTable should be migrated or deprecated.

### Layer 3: page archetypes

Six templates should cover nearly all recurring page structures:

1. `ListPage` or queue
2. `DashboardPage` or overview
3. `DetailPage`
4. `WorkbenchPage` or master-detail
5. `FormPage` or settings/wizard
6. `TimelinePage` or planner

Templates own:

- page width and outer spacing;
- header and action placement;
- density and section hierarchy;
- allowed pattern slots;
- responsive transformations;
- complete state contracts;
- accessibility landmarks;
- stable visual-test anchors.

The target is approximately 99% coverage of recurring UI behavior, not 99% of
literal markup. Bespoke domain charts and operational visualizations remain
typed slots inside the controlled page skeleton.

### Layer 4: domain composition

Application repositories should normally provide only:

- domain data and query adapters;
- labels and formatting rules;
- typed columns and filters;
- role and permission rules;
- domain actions;
- exceptional custom visualization slots.

They should not recreate page chrome, state containers, filters, tables,
pagers, metric cards, section cards, or responsive policies.

## Page contracts

Every page should have a small, versioned, machine-readable contract. For
example:

```toml
id = "inventory.explorer"
archetype = "list"
density = "compact"
reference = "design/inventory-explorer-1440x900.png"

states = [
  "never-loaded",
  "loading",
  "loaded",
  "empty",
  "partial",
  "stale",
  "error",
  "permission-denied",
  "overflow",
]

viewports = ["1440x900", "1024x768", "390x844"]
roles = ["manager", "analyst"]
```

The complete contract should also declare:

- user job and intended outcome;
- title, subtitle, breadcrumbs, and primary action;
- metric definitions and order;
- filters, defaults, and persistence;
- columns, formats, default sort, and row actions;
- section hierarchy;
- responsive invariants;
- interaction effects;
- expected accessibility behavior;
- named design reference and reviewer;
- acceptance thresholds.

Runtime page implementations should remain typed Rust. The manifest contains
data and policy, not arbitrary markup or classes. It may generate typed Rust or
be validated against Rust-owned archetype schemas.

## Component and pattern registry

Each public framework item should declare:

- stable ID;
- maturity and deprecation state;
- `use_when` and `do_not_use_when` guidance;
- props, variants, and organizational defaults;
- accessibility and keyboard contract;
- responsive contract;
- required states;
- story/demo route and fixture IDs;
- downstream pattern, template, and page consumers;
- permitted overrides;
- replacement for deprecated APIs.

The registry should generate:

- component and pattern documentation;
- usage examples;
- Tailwind source/safelist configuration;
- page scaffolding;
- agent reference material;
- visual and interaction story matrices;
- the test impact graph;
- stale-documentation and duplicate-public-name checks.

## Enforcement

Instructions alone cannot enforce the architecture. Add mechanical rules:

- Screen modules may import templates and patterns. Direct primitive imports
  require an explicit exception.
- Ban raw `button`, `input`, `select`, and `table` elements in application
  screen modules.
- Ban hard-coded colors and arbitrary layout values in screens.
- Ban app-local StatCard, SectionCard, FilterBlock, PagerRow, and equivalent
  primitives after canonical replacements exist.
- Deprecate duplicate public concepts.
- Restrict `class` and `DataTableClasses` overrides to framework internals or a
  reviewed exception registry.
- Give page composition modules a size budget; state, query, and transformation
  logic belong in separate modules.
- Record each exception with a reason, owner, scope, and expiry.
- Fail CI if generated registry outputs, documentation, agent packs, or consumer
  version locks are stale.

## Agent integration

### Instruction structure

Root `AGENTS.md` and `CLAUDE.md` files should be concise routers containing only
facts and rules needed in every session. Multi-step UI procedures should live in
a task-specific skill.

The framework should ship one canonical `ldui-page` agent pack and generate the
tool-specific projections:

```text
agent-pack/ldui-page/             # canonical source
.agents/skills/ldui-page/         # Codex projection
.claude/skills/ldui-page/         # Claude Code projection
```

The skill workflow should require an agent to:

1. Read the page contract.
2. Select the declared archetype.
3. Use registry-approved patterns.
4. Scaffold rather than hand-create the page shell.
5. Avoid primitives, raw controls, and arbitrary classes unless an exception is
   declared.
6. Run `verify-changed --explain`.
7. Inspect the rendered page against its named design reference.
8. Exercise the required state, accessibility, interaction, and effect oracles.

### Consumer synchronization

The framework should publish a contract/version manifest containing:

- framework commit or release ID;
- registry schema version;
- token version;
- agent-pack version;
- generated CSS-source version.

Inventory's sibling path dependency and Office's vendored dependency should
both verify this manifest. Office additionally needs an explicit vendor-sync
command that copies the framework and agent pack together and verifies the
stored commit provenance.

### Scaffolding and diagnostics

Recommended commands:

```text
cargo xtask ui new-page --template list --name inventory.explorer
cargo xtask ui doctor
cargo xtask ui explain-page inventory.explorer
cargo xtask verify-changed --base origin/main --explain
```

`ui new-page` should generate the screen composition, page contract, story
fixtures, route entry, state catalog, and test stubs.

`ui doctor` should identify raw controls, unsupported imports, arbitrary values,
duplicate local patterns, undocumented exceptions, and stale generated files.

## Testing architecture

### Design intent versus regression baselines

A screenshot baseline answers: "Did this implementation change?"

It does not answer: "Did the implementation select the intended hierarchy,
variant, density, or component?"

Each page therefore needs two distinct references:

1. A named design-intent reference or approved page contract.
2. A regression baseline accepted only after comparison with that reference.

No new baseline should be accepted without its design reference, viewport, and
reviewer being recorded.

### Story/state catalog

Each pattern should have Storybook-like Leptos stories for meaningful states:

- default;
- loading;
- never loaded;
- empty;
- partial;
- stale;
- error;
- permission denied;
- overflow and long labels;
- compact and comfortable density;
- narrow and wide viewports;
- keyboard interaction;
- supported themes and contrast modes;
- localization fixtures where applicable.

Stories should be executable test cases, not documentation-only examples. The
story concept is useful because it captures each interesting component state
independently; it does not require adopting Storybook's JavaScript runtime.

### Test layers

Retain the repository's visual-quality methodology:

- Layer A: pixels and computed style.
- Layer B: DOM state, model state, and desynchronization checks.
- Layer C: accessibility and keyboard behavior.
- Layer D1: correlated browser/network traces and errors.
- Layer D2: independent completion or rows-affected evidence.

New test rules should include a break-and-revert negative control proving that
the test catches the defect it claims to guard.

## Affected-test selection

Implement:

```text
cargo xtask verify-changed --base <merge-base> --explain
```

The algorithm should:

1. Read changed paths from Git.
2. Resolve files to registry components, patterns, templates, and page specs.
3. Walk the transitive reverse-consumer graph.
4. Select the required native, story, DOM/model, accessibility, visual, and
   effect tests.
5. Escalate unknown or unmapped changes to the full gate.
6. Print the reason every selected test is required.
7. Persist timings and selection evidence for later optimization.

Suggested selection policy:

| Change | Required checks |
|---|---|
| Documentation only | Registry and documentation generation checks |
| One primitive | Its unit tests and directly consuming stories |
| Pattern | Pattern tests plus every template/page using it |
| Page implementation or spec | That page's model, DOM, accessibility, interaction, and visual states |
| DTO/API contract | Changed package and reverse dependents; UI tests only when rendered behavior changes |
| Token, theme, macro, or base CSS | Full affected UI matrix |
| Cargo features, build logic, or audit infrastructure | Full relevant gate |
| Unknown or unmapped file | Full gate |

Use distinct feedback lanes:

- Inner loop: targeted check and unit tests, with a goal below 60 seconds.
- Pre-push: all affected stories and page contracts.
- Main/release: complete repository and release gate.
- Nightly: all supported themes, roles, states, and viewports.

Periodic full gates are necessary to detect errors in the selector itself.

## Build-time findings

A live `cargo xtask test-style` run on 2026-08-25 took approximately 14.5
minutes. The seven browser tests executed in 19.66 seconds. More than 95% of the
elapsed time was Wasm compilation and linking rather than assertion execution.

During the run:

- `rust-lld` reached approximately 1.85 GB;
- it crashed with Windows status `0xC0000005`;
- the audit freshness marker correctly rejected the stale served bundle;
- no false-green visual result was produced.

Observed local debug Wasm artifacts were:

| Application | Debug Wasm size |
|---|---:|
| Framework showcase | 87.5 MB |
| Office | 95.6 MB |
| Inventory | 22.8 MB |

These are not compressed production transfer sizes. They are nevertheless the
artifacts the local linker and browser audit process must produce and load.

Immediate optimizations should include:

1. Build once and share one running server across style, layout, and reactivity
   suites.
2. Combine style and layout collection where both audit the same route.
3. Content-hash the generated stylesheet and prevent xtask and Trunk from
   rebuilding identical CSS.
4. Split the showcase into smaller independently linked audit targets.
5. Record per-step wall time and Cargo timing reports.
6. Measure a reduced-debug-information development profile such as
   `debug = "line-tables-only"`.
7. Use stable target directories and canonical feature sets to maximize cache
   reuse.
8. Normalize `NO_COLOR=1` to a value accepted by Trunk 0.21.
9. Terminate before browser startup when the Wasm build or link fails.

Test-name filtering alone is insufficient because Cargo may still compile and
link the entire test executable. Test selection must be paired with smaller
build targets.

## Wasm bundle architecture

### Current state

The framework showcase directly references every component demo route in
[`demo/src/main.rs`](demo/src/main.rs). Office and Inventory similarly reference
every application route in one executable. Consequently, all reachable page
code enters one Wasm compilation and link.

The root framework crate also has only the `test-mode` feature. Data-table,
chart, Gantt, AI, Markdown, Mermaid, and other dependency families all belong to
one crate closure.

### Option A: Leptos lazy routes

Leptos 0.8 supports `#[lazy]`, `#[lazy_route]`, `LazyRoute`, and
`cargo leptos --split`. This creates a base Wasm module and route chunks loaded
on demand.

Benefits:

- smaller initial browser payload;
- route code downloads only when needed;
- one router, shell, and reactive application model;
- nested route chunks can load concurrently with route data.

Limitations for the current problem:

- `cargo-leptos` first completes the full Cargo Wasm build and link, then reads
  that linked Wasm and splits it;
- it therefore does not remove unrelated routes from the original linker job;
- splitting adds another post-link processing step;
- current projects use Trunk CSR, while the supported lazy-route example uses
  cargo-leptos and SSR/hydration;
- the tooling is comparatively new and requires a Windows/toolchain validation
  spike.

Conclusion: use lazy routes to improve production delivery after a successful
spike, but do not treat them as the solution to local link time or linker memory.

### Option B: separate Trunk page targets

Trunk can select a particular Cargo binary through `data-bin` or
`data-target-name`. A page or story can therefore be a genuinely independent
Wasm target.

Benefits:

- a selected target does not link unrelated page binaries;
- browser tests build only the requested page or pattern fixture;
- failures are isolated;
- CI can distribute targets across a matrix;
- pages can be opened independently or embedded in a catalog iframe.

Costs:

- each bundle contains its own reachable Leptos/runtime code;
- independent Wasm applications do not naturally share a Rust heap, reactive
  owner, or in-memory state;
- navigation between production bundles normally causes a full page load or
  requires an explicit JavaScript/browser-storage boundary;
- building every one of 1,000 binaries still requires 1,000 link jobs;
- a monolithic shared framework crate can still cause broad compilation even
  when final page linking is isolated.

Conclusion: this is the recommended architecture for story, component, pattern,
and page-level visual-test capsules. It is not the default recommendation for
1,000 production application bundles.

### Option C: bounded framework crates and domain bundles

Split the framework into a small number of stable dependency families:

```text
ldui-core
ldui-patterns
ldui-data
ldui-charts
ldui-planning
ldui-ai
leptos-daisyui-rs      # compatibility facade/re-exports
```

This prevents a table page from compiling and linking Gantt, AI chat, Mermaid,
and unrelated visualization dependencies. Avoid one crate or feature per
component; that would create excessive maintenance and feature combinations.

Applications that remain on Trunk CSR can optionally build a small number of
domain bundles, such as:

- core/authentication;
- queue and tabular operations;
- conversations;
- planning;
- reporting and charts;
- administration/settings.

Five to twenty domain bundles are more manageable than one bundle per page and
still materially bound link size.

### Option D: template runtime plus data-only page specifications

Hundreds or thousands of standard pages should not require hundreds or
thousands of Rust component implementations.

The preferred scale model is:

```text
shared shell and template runtime
              +
validated page specification loaded by route
              +
domain data/query adapter
```

Six compiled page archetypes can render many validated `PageSpec` instances.
Page count then adds mostly data, not new generated control-flow code or a new
Wasm linker target. Exceptional custom pages can remain separately compiled
modules or route chunks.

This should remain a bounded archetype system, not an arbitrary UI DSL. Specs
may choose registered metrics, filters, columns, actions, and slots, but cannot
inject arbitrary markup or CSS.

### Recommended Wasm strategy

Use a hybrid:

1. Data-only page specifications for standard pages.
2. Six typed, compiled page templates.
3. Independently linked Trunk capsules for component, pattern, and page tests.
4. Five to eight bounded framework dependency crates.
5. Build-on-demand locally and a parallel all-capsule CI matrix.
6. A measured `cargo-leptos --split` spike for production route delivery.
7. Coarse domain bundles if applications remain on Trunk CSR and the monolithic
   production link remains unacceptable.

## Proposed repository shape

```text
crates/
  ldui-core/
  ldui-patterns/
  ldui-data/
  ldui-charts/
  ldui-planning/
  ldui-ai/

src/
  registry/
  page_specs/

ui-fixtures/
  primitives/
  patterns/
  templates/
  consumer-pages/

agent-pack/
  ldui-page/

doc/
  generated/
  visual-quality/
```

The existing root package can remain as a compatibility facade while consumers
migrate to narrower packages or feature groups.

## Migration sequence

### Phase 0: control the decision surface

- Create the machine-readable component and pattern registry.
- Generate component counts and documentation from the registry.
- Choose canonical public names and deprecate duplicates.
- Make the full DataTable canonical.
- Add framework/registry/agent-pack version manifests.
- Add build-step timing and `verify-changed --explain` infrastructure.
- Fix the current Trunk environment normalization and failure propagation.

### Phase 1: one golden vertical slice

- Implement `PageHeader`.
- Implement `KpiStrip`.
- Implement `FilterBar` and active chips.
- Implement `AsyncDataSection`.
- Implement `EntityTable<T>` over the full DataTable.
- Compose them into `ListPage`.
- Migrate Inventory Explorer.
- Migrate Office Queue.
- Create independent Wasm story and page capsules.
- Approve references and state matrices for both consumers.

### Phase 2: broaden the template system

- Extract `DashboardPage` from real dashboard/coordinator screens.
- Extract `DetailPage` from account, matter, or stage-detail screens.
- Extract `WorkbenchPage` from coordinator/manager workflows.
- Add `FormPage` and `TimelinePage` only from demonstrated consumer needs.
- Remove the replaced app-local cards, filters, tables, and pagers.

### Phase 3: enforce the architecture

- Enable forbidden-import and raw-control checks.
- Require exception registry entries.
- Generate and distribute the agent skill pack.
- Require page contracts for new pages.
- Require affected tests and design-reference review before baseline acceptance.
- Introduce bounded framework dependency crates and Wasm fixture shards.

### Phase 4: optimize production delivery

- Benchmark production and debug compilation separately.
- Spike `cargo-leptos --split` on one representative nested route.
- Measure full link time, split processing time, initial payload, navigation,
  caching, and Windows reliability.
- Adopt lazy routes only if the measurements and operational model are better.
- Otherwise introduce coarse Trunk domain bundles where justified.

## Success measures

Track outcomes rather than component count:

- At least 90-95% of ordinary screen composition comes from approved patterns
  and templates.
- Approximately 99% of repeated mechanical UI behavior is framework-owned.
- One canonical public table API.
- No application-local StatCard, pager, filter-row, or generic SectionCard
  replacements without an exception.
- No raw interactive elements or hard-coded design values in screen modules.
- Every page declares required states, roles, viewports, and a design reference.
- Every page baseline is approved against that reference.
- Page composition files remain small; query and state logic are separated.
- Median affected inner-loop feedback is below 60 seconds.
- Visual test capsules link only their declared dependency family.
- Full release and nightly gates remain green and periodically validate the
  affected-test selector.
- Agent retries, review corrections, UI defect escape rate, and implementation
  time decline measurably after each template migration.

## External references

- [Leptos components and typed props](https://book.leptos.dev/view/03_components.html)
- [Leptos component children and composition](https://book.leptos.dev/view/09_component_children.html)
- [Leptos Wasm code splitting](https://book.leptos.dev/deployment/binary_size.html#code-splitting)
- [Current cargo-leptos frontend build sequence](https://raw.githubusercontent.com/leptos-rs/cargo-leptos/main/src/compile/front.rs)
- [Trunk Rust targets and `data-bin`](https://github.com/trunk-rs/trunk/blob/main/guide/src/assets/index.md#rust)
- [daisyUI design and flexibility model](https://daisyui.com/docs/intro/)
- [daisyUI semantic colors](https://daisyui.com/docs/colors/)
- [GOV.UK Design System patterns](https://design-system.service.gov.uk/patterns/)
- [Storybook stories as captured component states](https://storybook.js.org/docs/get-started/whats-a-story)
- [Cargo test selection](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Cargo build timings](https://doc.rust-lang.org/stable/cargo/reference/timings.html)
- [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Codex AGENTS.md guidance](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
- [Codex skills](https://learn.chatgpt.com/docs/build-skills)
- [Claude Code project instructions](https://code.claude.com/docs/en/memory)
- [Claude Code skills](https://code.claude.com/docs/en/skills)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks-guide)
