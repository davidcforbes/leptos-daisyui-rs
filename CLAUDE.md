# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust crate providing type-safe, reactive daisyUI 5 component wrappers for Leptos framework. The library wraps daisyUI components as Leptos components with proper type safety, leveraging Leptos's spread attributes functionality. Currently designed for CSR (Client-Side Rendering).

**Component Coverage: full daisyUI 5 coverage plus custom app-shell/data/scheduling/motion components** — the count shifts often (120 component modules as of 2026-09); `ls src/components/` is the source of truth.

## Build Commands

### Using the Launcher Script (Easiest)

The project includes a comprehensive PowerShell launcher with an interactive menu:

```powershell
# Interactive menu
.\launcher.ps1

# Direct task execution
.\launcher.ps1 -Task dev      # Launch demo app
.\launcher.ps1 -Task check    # Quick validation
.\launcher.ps1 -Task fix      # Fix and verify
.\launcher.ps1 -Task build    # Build library
.\launcher.ps1 -Task test     # Run tests
.\launcher.ps1 -Task docs     # Build and open docs
```

**Features:**
- 🎨 Color-coded output for easy reading
- ⚡ Parallel workflows (dev + docs in separate windows)
- ✅ Prerequisite checking
- 📊 Task timing and success/failure reporting
- 🔍 25+ pre-configured tasks organized by category

See [`LAUNCHER.md`](./LAUNCHER.md) for complete guide.

### Local CI gate (the primary check — see [`doc/ci-cd.md`](./doc/ci-cd.md))

CI/CD is **local-only, two-layer**: logic lives in the `xtask/` crate; cargo-make
just delegates. Run the gate before committing:

```bash
cargo xtask verify        # native-only gate; no Wasm/Chrome/reactivity lane
cargo make verify         # same, via cargo-make
cargo xtask verify-full   # verify + the browser/Wasm lanes (needs npm/trunk/Chrome)
cargo xtask test-reactivity          # opt-in 69-check DOM/interaction lane; never implicit in verify
                                     # ⚠️ its check count is PINNED in xtask; adding a
                                     # #[tokio::test] there fails the gate until re-pinned
cargo xtask test-layout              # layout audit: overlap/grid/internal<=external over the real DOM
cargo xtask test-style               # style audit: typography/shape/depth + daisyUI component-drift, ratcheted per page
cargo xtask test-keyed-result-list   # focused KeyedResultList browser proof: stable-key selection across duplicate labels/reorders/removals
cargo xtask test-modal-close-proposal  # Modal Escape/backdrop close proposals (ldui-e0fw)
cargo xtask test-selectable-summary    # SelectableSummaryGroup radiogroup contract (ldui-l5cw)
cargo xtask test-bar-chart-divergence  # BarChart signed divergence + a11y (ldui-y2ed)
cargo xtask test-heatmap-matrix        # Heatmap two-axis grid + hidden matrix (ldui-8d94)
cargo xtask test-collapse-naming       # Collapse toggle id/name + title-derived accessible name (ldui-3k00)
cargo xtask test-data-table-fit        # DataTable undeclared columns fit w-full; declared tracks scroll (ldui-qsqz)
cargo xtask test-snapshot-table-page-filter-actions  # SnapshotTablePage's opt-in count/Reset/Save row (ldui-nj3q)
cargo xtask test-entity-draft-row      # EntityTable inline draft-row editing (ldui-ff2f)
cargo xtask test-app-shell             # AppShell region contract (ldui-a8an)
cargo xtask test-field-context-scoping # Field id minting / no inheritance (ldui-a8an)
cargo xtask test-helpdesk              # Helpdesk composite: board, drawer, New Request dialog, host-owned capability table (ldui-b5mu, ldui-ftdy)
cargo xtask test-ai-chat               # AiChatWorkspace showcase over the seeded fixture (ldui-iilm)
cargo xtask test-ai-chat-knowledge     # knowledge rail: corpora, ingest, grounding, recall, guardrails (ldui-iilm.7)
cargo xtask test-admin-workbench       # KpiStrip ladders + the measured help-bubble side (ldui-k3ip, ldui-rzvv)
cargo xtask test-focus-ring            # .ld-focus-ring colour present from frame 0; only the offset animates (ldui-reod)
cargo xtask test-row-action-presets    # row-action + Export presets: glyphs, names, tooltips, disabled_reason (ldui-bmqj, ldui-e6x8, ldui-p82h)
cargo xtask test-server-cursor-footer  # ServerDataTable cursor mode: standard three-region footer with total+position, fallback without (ldui-q14c)
cargo xtask gen-tokens [--check]     # regenerate styles/tokens.css from ui-tokens
cargo xtask check-sibling-tokens     # preamble.rs's ui_tokens refs must exist on the sibling's DEFAULT branch
cargo xtask test-person-picker       # PersonPicker listbox contract, wrapping, Unknown-vs-Offline, contrast
cargo xtask clean-cache              # drop incremental/doc/rust-analyzer caches; keeps deps/ warm (~100 GB)
# (list non-exhaustive and rot-prone — the dispatch match in
#  xtask/src/main.rs is the self-updating source of truth; newer lanes include
#  test-softphone, test-section-heading, test-search-picker-dialog, verify-pattern, check-demo, ...)
cargo xtask bump patch|minor|major   # bump the library version (human-chosen level)
```

Two testing-structure rules: test-host fixtures are selected by **pathname
suffix**, never `?query` (the harness appends its own `?pp-freeze=1`, and a
`/fixture?fault=…` navigation never mounts the freeze stylesheet); and a
consumer that checks a *vendored* copy of this crate with `--all-targets` never
compiles this crate's `cfg(test)` module — a vendored test can be broken for
weeks. `verify` carries a `clippy-lib-wasm` step because clippy only lints what
its target compiles (`#[cfg(target_arch = "wasm32")]` code was never linted
here, hiding real lints).

**Browser coverage that only COMPILES proves nothing.** Agents hand off
browser suites compile-only (`--no-run`), because only the operator can safely
rebuild the shared demo. Every such suite, on its FIRST actual execution, has
failed — and several failures were real product defects invisible to 3000+
native tests: `Heatmap` and `BarChart` were both unreachable by keyboard (their
reconcile effect returns early on its first run, so the roving tab stop was
never seeded and no element carried `tabindex=0`), a `method="dialog"` close
button did nothing (this crate's `Button` defaults to `type="button"`, unlike
daisyUI's documented raw-`<button>` idiom), and focus was lost whenever a row
group collapsed. Run the lane before believing the coverage.

**Do not touch the tree while a browser lane is running -- `cargo fmt`
counts.** xtask builds the stylesheet, stamps `demo/.ldui-css-stamp`, then
starts `trunk serve`; trunk *watches* `../src` and `demo/src`, so any write
there re-runs the `build-css.mjs` pre-build hook and rewrites that stamp while
the already-served `dist` stylesheet still carries the old one. The suite then
fails on the `ldui-hun` guard:

```
STALE STYLESHEET (ldui-hun): the CSS loaded by <route> is not the CSS that was
last built.
  expected marker: ldui-css-stamp-<B>   (from demo/.ldui-css-stamp)
  markers found in the served stylesheets: ldui-css-stamp-<A>
```

That is the guard working, not a fixture defect: **the marker mismatch in the
message is the whole diagnosis**, so re-run on a quiescent tree rather than
investigating the test (the re-run is fast -- everything is already compiled).
Finish every edit, including the per-package `cargo fmt` this repo requires,
*before* launching a lane. The same message with the stamps *matching* is a
different failure -- a tailwind build that never ran (missing
`demo/node_modules`).

**A suite registered in no xtask lane runs nowhere.** It compiles, never
executes, and the evidence it exists to produce is structurally unobtainable.
Seven registration-or-pin gaps occurred in a single session. Adding a
`#[tokio::test]` to `reactivity_smoke.rs` also breaks its pinned count. Verify
a new check by the TEST COUNT changing, never by the suite passing — the same
trap hit the audits themselves: adding pages to `PAGES` swept nothing, because
coverage comes from per-page `audit_test!(name, index)` invocations, and both
suites stayed green while appearing to cover six more pages.

**Never identify an element by document position.** A positional selector does
not fail when the layout changes; it silently starts describing something else.
`'[data-entity-table] label select'` meant the page-size control only while it
was the first label-select in the table, and after ldui-z0n1 moved that control
the query returned the *status filter* while still passing. Use stable data
hooks (`data-entity-page-size-control`, `data-entity-row-range`,
`data-kpi-strip-layout`, `data-heatmap-cell`).

**A rect is not "rendered".** daisyUI's closed `.modal` is `display: grid;
position: fixed; inset: 0; visibility: hidden` -- it overrides the native
`dialog:not([open]) { display: none }` so the close can animate -- and
`getBoundingClientRect()` ignores `visibility`. So every control in a CLOSED
`Modal` reports a real, viewport-anchored rect (the stretched backdrop button's
`bottom` equals the viewport height, scrolled or not). Any "visible controls"
query filtered by `width > 0 && height > 0` finds them; use
`el.checkVisibility({ visibilityProperty: true })`. Same shape as EntityTable's
hidden compact `<td>`. A test that mirrors an `ldui-audit` rule must READ the
rule's constants from `audit/src/*.js` (see `audit_exempt_closest` in
`tests/entity_table_smoke.rs`), never hand-copy them: a copied rule omitted an
exemption and failed on correct markup.

**Geometry assertions: compare centers in an `items-center` row, and size a
negative control from measured slack.** Two probes that passed on correct
markup or proved nothing (ldui-5oce): a "same row" check comparing `top`s
failed by ~6px because a text span is shorter than a button group and the row
centers them; a clipping negative control that expanded a heading by a fixed
64px kept passing after the footer lost a row, because the region had gained
~28px of slack and the constant no longer reached its bottom. Measure
`(top+bottom)/2`, and expand by `regionBottom - lastRowBottom + margin`, then
assert the expansion actually happened. The `EntityTable` footer itself is an
owner ruling, not a per-page option: rows-per-page left, pager centered (an
`auto` grid track between two `minmax(0,1fr)` flanks), row range right, with
the pager on its own centered row below `@2xl`. `ServerDataTable` renders the
same grid in both offset and cursor mode (ldui-q14c); a cursor page shows a
range and a current-page slot only when the caller supplies
`with_total_rows` AND `with_position`. The quick-action row is a ruling too
(ldui-q85o): LEFT Save Filter (always enabled; an empty click is a no-op) then
up to five saved-filter badges with their `x` and no "Filters:" caption;
RIGHT `+ New`, the consumer's actions (Export = the download icon), the gear.

**A disabled action says why (ldui-p82h).** `Button::disabled_reason` (a
`Signal<String>`; blank = enabled) is the one mechanism: it disables, wires
`aria-describedby` to a hidden hint, and sets `title`. The audit's
`disabled-without-reason` rule fails any disabled control without a resolving
description; only STATE-disabled controls are exempt (a pager's current page
and boundary arrows under `[data-pagination]`, `.loading`/`aria-busy`, the
selected option of a single-select group) -- see
`doc/visual-quality/disabled-without-reason.md`. Never fix a finding by
raising a ceiling. The hint is an `aria-hidden` span INSIDE the `<button>`, so
a test reading a disabled button's raw `textContent` gets the label with the
reason glued on (`"ResetNo active filters"`); strip
`[data-button-disabled-reason]` from a clone first, and assert the reason via
`aria-describedby`.

**A viewport breakpoint says nothing about the space a component has.**
`lg:` fires on the WINDOW, so a composite mounted in a 375px side rail on a
wide screen still takes its three-column layout -- `AiChatWorkspace` resolved to
256 | 1.6 | 256 px in 4iiz-Office production (ldui-d5ss). A composite that can
be hosted narrow needs a host-chosen layout (`AiChatWorkspaceLayout::Rail`) or a
container query on its own root. Prove it the way that bead's lane does: mount
it in a narrow host on a WIDE viewport, and keep the old layout in the same
host as the negative control.

**daisyUI 5.5 `.select` is `appearance: base-select` in Chrome, and that changes
clipping (ldui-tfx7).** The selected label ignores `text-overflow`, and
`overflow: hidden` clips at the PADDING box, so a label wider than the column
paints under the arrow. `Select` clips at the content box for that reason; a
raw `<select class="select">` does not, and a hand-copied daisyUI rule without
the `@supports (appearance: base-select)` block will not reproduce the bug.

**Pixel comparisons need a tolerance; byte equality is flaky.** The same arrow
on two identical selects differed by 1/255 of anti-aliasing and failed an exact
PNG comparison at a different size each run. Decode and compare per pixel
against a threshold far below the signal (48 per channel, where a glyph on the
fill is 100+) -- never a MEAN score, which lets a few pixels of a letter vanish.
`image` is a PNG-only dev-dependency for this.

**`@container` collapses a content-sized parent.** `container-type: inline-size`
makes an element's inline size independent of its contents, so a parent that
sizes to content — a bare `<div>` in a `flex` row, or `max-w-*` with no width —
measures it as contributing nothing and collapses to zero. `KpiStrip` cards
rendered ~2px wide with no error, correct markup and every class present. Give
the strip's PARENT a width; the component cannot supply it.

**Repo-specific gotchas the gate encodes (do NOT run these directly):**
`cargo fmt --all` reaches into sibling repos — fmt **per-package**
(`-p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask -p ldui-audit`).
`cargo clippy --workspace` fails on leptos-`csr` feature unification — clippy
**per-crate**. The library's clippy/test steps pass `--features test-mode`;
without it `src/test_mode.rs` is neither linted nor tested.

**Windows `web-sys` + sccache is `os error 206`, not a rustc bug.** sccache
0.17+ expands rustc `@argfile`s, so `web-sys`'s feature `--cfg` list exceeds
CreateProcess's 32,767-character limit (`could not compile web-sys`). rustc
1.98+ is fine. `cargo xtask`, `.\launcher.ps1`, and `cargo make` unset
`RUSTC_WRAPPER` before spawning cargo. A raw `cargo check` in a shell that
has sccache must `Remove-Item Env:RUSTC_WRAPPER` first.

**The visual-quality rulebook is [`doc/visual-quality/`](./doc/visual-quality/)** —
one page per defect pattern the `test-style`/`test-layout` audits detect, with
the fix rather than the ratchet.

**Doc comments: keep every inline code span on ONE `///` line.** A backtick span
that wraps across two `///` lines ICEs clippy 1.95 (panic in
`doc/include_in_doc_without_cfg.rs`). The failure is silent-by-omission: clippy
reports only "the compiler unexpectedly panicked" and lints *nothing* for that
crate, so the gate's clippy step is effectively off while real lints pile up
behind it. If `cargo xtask verify` shows clippy FAIL with no lint output, suspect
this and run `cargo clippy -p <crate> --all-targets -- -D warnings` to see the panic.

**Running the demo needs `npm install` in `demo/` first** (`node_modules` is not
committed). Without it, trunk's Tailwind pre-build hook fails and trunk serves a
**stale** build — the page loads fine but your changes aren't in it.

**Run `cargo xtask` from the repo root.** It resolves `demo/` and `styles/`
against the *current* directory, not the workspace root, so invoking it from
`demo/` (easy to do — a shell's cwd persists between commands) fails as
`xtask: failed to launch npm: The directory name is invalid. (os error 267)`,
the same for `trunk`, plus a spurious `tokens-fresh` FAIL. npm and trunk are
fine; the cwd handed to the child is `demo/demo`.

**Budget ~8 minutes for the first browser-suite run.** `test-reactivity`,
`test-layout`, `test-style` and `verify-full` each build the demo to wasm, which
outruns a 10-minute foreground timeout on a cold target dir — run them in the
background. Any edit under `demo/src` invalidates that build, so re-run the
suites *after* the last demo change, not before: `test-layout`'s and
`test-style`'s per-page violation counts are ratcheted and a new demo section
can move them.

**`styles/tokens.css` is GENERATED — never hand-edit it.** It is the Tailwind
`@theme` block, produced from the shared `ui-tokens` crate by `cargo xtask
gen-tokens` and imported by `demo/input.css`. The gate's first step
(`tokens-fresh`) re-runs the generator with `--check` and fails if the committed
file has drifted, which is how the desktop and web faces are kept from silently
forking. Change a token upstream in `../Rust-DeskApp/crates/ui-tokens`, then
re-run the generator and commit the result. Two rules the generator encodes:

- **DIP → rem, never px.** Tokens are DIPs because Direct2D has no rem; emitting
  them as px on the web pins font sizes and gaps against the user's browser
  font-size preference (WCAG 1.4.4). Only border widths stay px.
- **No named `--spacing-*` keys.** Tailwind resolves `w-*`/`max-w-*` against
  `--spacing-*` *before* `--container-*`, so a `--spacing-xs` key silently
  redefines `max-w-xs` from 20rem to 0.5rem. A unit test forbids them.

**Never reference a `ui_tokens` item that is not on the sibling's DEFAULT
branch.** `ui-tokens` is a *path* dependency, so cargo resolves it to whatever
`../Rust-DeskApp` has checked out. A branch-only item compiles here and the
whole gate goes green, while `main` is unbuildable for everyone whose sibling
sits on `master` — and the break surfaces in a downstream consumer, where it
reads as *their* fault. On 2026-07-29 that cost a 4iiz-office session hours.
The `sibling-tokens` gate step now catches it; it skips (never fails) when the
sibling is absent. If it fires, land the upstream change before committing the
reference.

**After landing anything in `../Rust-DeskApp/crates/ui-tokens`, run `cargo fmt
-p ui-tokens` in *that* repo.** `ui-tokens` is a path dep, not a workspace
member, so `fmt-check` here structurally cannot see it — and editmark's
`cargo fmt --all` *does* reach path deps, so unformatted sibling tokens turn
**editmark's** release gate red over a repo it never compiles. Cost a red gate
twice on 2026-07-30; see [`doc/ci-cd.md`](./doc/ci-cd.md).

### Using cargo-make (Recommended for CI/Scripts)

The project includes a comprehensive `Makefile.toml` with automated workflows:

```bash
# Quick verification before commit
cargo make quick-check        # Format + clippy-fix + test

# Fix all auto-fixable issues
cargo make fix-all           # Run rustfmt and clippy --fix

# Verify everything passes
cargo make verify-all        # Format check + build + test + clippy

# Fix then verify
cargo make fix-and-verify    # Fix all issues, then verify

# CI workflow
cargo make ci                # Full CI: format + build + test + lint

# Demo development
cargo make dev               # Start demo dev server (alias for demo-serve)
cargo make demo-serve        # Run trunk serve in demo/
cargo make demo-build        # Build demo for production

# Documentation
cargo make doc               # Build library documentation
cargo make doc-open          # Build and open docs in browser

# Clean everything
cargo make clean-all         # Clean library and demo artifacts
```

**Available cargo-make tasks**: Run `cargo make --list-all-steps` to see all available tasks.

### Library Development (Direct cargo commands)
```bash
# Build the library
cargo build

# Run tests
cargo test

# Check without building
cargo check

# Build with release optimizations
cargo build --release
```

### Demo Application
```bash
cd demo

# Development server with hot-reload (Trunk watches ../src and ./src automatically)
trunk serve
# Runs on http://127.0.0.1:3010 (demo/Trunk.toml)
# Or use: cargo make dev (from root)

# Build for production
trunk build --release
```

The demo automatically:
- Runs Tailwind CSS compilation via pre_build hook: `npx tailwindcss -i input.css -o output.css`
- Watches both `../src` (library) and `./src` (demo) for changes
- Showcases every component with interactive examples

## Architecture

### Core Structure

- `src/components/` - daisyUI component wrappers plus custom app-shell/data/scheduling components
- `src/utils/` - Utility code: `ClassAttributes` for dynamic class management, plus
  reusable framework hooks (`DebouncedSignal`, `use_swr_resource`)
- `src/motion/` - Animation primitives (`Lerp`, `Transition`, `Keyframe`/`Track`, easing, spring, `use_animated` hook)
- `src/widgets/` - simple widget-level composites (the `Vec<Vec<String>>` DataTable among them)
- `src/patterns/` - opinionated page-level patterns (`patterns::Helpdesk`)
- `src/charts/` - shared SVG paint plumbing (`charts::paint`)

**Choose a table from data ownership before capability (`ldui-aqo`):**
`EntityTable<T>` is the preferred typed complete-client-snapshot table;
`ServerDataTable` is the server-query/current-slice table. Their roots expose
`data-table-data-mode="client-snapshot"` and `"server-query"` respectively.
`components::DataTable` (`src/components/data_table/`) remains the
`"compatibility-client"` path for dynamic `HashMap` rows and its advanced
existing feature surface (`cell_renderers`, `typed_cells`, `row_class_fn`,
`Column::action()`, selection, filters, toolbar composition, and auto paging).
`widgets::DataTable` (`src/widgets/data_table.rs`) remains the simple
`Vec<Vec<String>>` table and the only one with `badge_column_keys`,
`link_column_keys`, and `bulk_select`. Never feed a server page to either client
table. See `doc/components/entity_table.md` for migration and preference
ownership.

`SnapshotTablePage` accepts only state-minted
`SnapshotLocalRowProjection<T>` values for controlled filtered rows. It
validates generation/revision, keeps the complete snapshot in
`EntityTable::source_data`, and derives dataset/page-size select IDs from the
page contract. Standalone `DatasetSelector`/`EntityTable` calls supply
`control_id`/`page_size_control_id`; do not patch IDs into the DOM.

Dated feature waves (Helpdesk, SnapshotTablePage filter actions, the
bead-wave fixes, LineChart/Gauge/PhaseProgress, RosterGrid, swr, motion)
and the findings behind them live in
[`doc/dev-log/2026.md`](./doc/dev-log/2026.md). Promote a still-relevant
rule into this file; do not re-introduce dated narrative.

### Framework-purity rule (why `utils/` keeps growing)

When a host app (EUC, inventory-web, Rust-DeskApp) hand-rolls a **generic**
reactive pattern, promote it here rather than letting each screen re-derive it.
`utils/debounce.rs` and `utils/swr.rs` were both extracted after a host app
re-implemented them for the second/third time. App-specific logic stays in the app;
framework primitives live here.

Timer discipline follows the same rule: if you add a `set_timeout_with_handle`
whose closure touches component signals, store the handle, clear it in
`on_cleanup`, and use try-forms in the closure so a late fire degrades to a
no-op instead of panicking the wasm app (pinned by
`data_table_timers_survive_unmount`).

### Component Pattern

All components follow a consistent wrapper pattern:

1. **Type-Safe Props**: Each component has enums for styling options (e.g., `ButtonColor`, `ButtonSize`, `ButtonShape`, `ButtonStyle`) that map to daisyUI CSS classes
2. **Spread Attributes**: Components accept Leptos spread attributes (`attr:`, `class:`, `style:`, `on:`, `prop:`) to extend underlying HTML elements
3. **Signal-Based Reactivity**: Props accept `Signal<T>` for reactive updates
4. **NodeRef Support**: Components expose `node_ref` prop for direct DOM access

Example component structure:
```
components/
└── button/
    ├── mod.rs           # Public exports
    ├── component.rs     # Button component implementation
    └── style.rs         # ButtonColor, ButtonSize, etc. enums
```

### Class Management

The `utils/class_attribute.rs` module provides `ClassAttributes` for building dynamic class strings:
- `ClassAttribute::Static(&'static str)` - Compile-time class names
- `ClassAttribute::Dynamic(String)` - Runtime class names
- `merge_classes!` macro - Combines base classes with user-provided classes

This ensures daisyUI classes from style enums properly merge with user-provided `class` prop.

## CSS Configuration

daisyUI class names must be explicitly included in Tailwind CSS input for proper compilation. Each component documents required classes:

```css
/* In a sibling consuming project's input.css.
   demo/input.css uses ../styles/tokens.css and scans ../src/**/*.rs. */
@import "tailwindcss";
@import "../leptos-daisyui-rs/styles/tokens.css";
@plugin "daisyui";
@source "../src/**/*.rs";
@source "../leptos-daisyui-rs/src/**/*.rs";

/* Example: Button component classes */
@source inline("btn btn-neutral btn-primary ... btn-circle");
```

Adjust consumer paths relative to `input.css`. See `demo/input.css` for the
showcase's complete source configuration and each component guide for dynamic
inline classes.

### `--ld-*` design tokens: two delivery paths, easy to conflate (ldui-h7tw)

`ui-tokens` values reach the web face through **two independent
mechanisms**, and a consumer that only wires up one gets a silent partial
result rather than an error:

- **Generated into `styles/tokens.css`** by `cargo xtask gen-tokens` (see
  `xtask/src/main.rs`'s `tokens_css()`). This is the `@theme` block (Tailwind
  numeric spacing scale, `--border-width-*`, `--radius-*`, `--color-table-*`,
  `--text-*`) plus, as of ldui-h7tw, an `@layer components` block defining
  the six `.ld-text-display/title/subtitle/body/caption/small` classes that
  `SectionHeading`, `KpiStrip` and `PageHeader` emit. **Importing this file
  (as shown above) is enough on its own** — no component needs to be mounted
  for these to work.
- **Injected at runtime** by mounting `UiTokensPreamble` /
  `UiAnimationsPreamble` (`leptos_daisyui_rs::tokens`) near your app's root,
  which render a `<style>` element from `ui_tokens_css()` /
  `ui_animations_css()`. This is the *only* source for the `--ld-duration-*`,
  `--ld-ease-*`, `--ld-elevation-*`, `--ld-space-*`, `--ld-stroke-*`,
  `--ld-radius-*` custom properties, and for classes documented as requiring
  it: `.ld-eased`, `.ld-pressable`, `.ld-focus-ring`, `.ld-elevated`,
  `.ld-ripple-host`/`.ld-ripple-element`, `.ld-aichat-msg-in`,
  `.ld-vstep-rail`/`.ld-vstep-flow-dash`, and the `@keyframes` names motion
  primitives reference by name. **A consumer that never mounts these
  components gets no error — the classes are simply undefined**, so weight
  and colour classes on the same element still apply while the effect the
  `ld-*` class was for (animation, focus ring, elevation) silently does
  nothing.

Before ldui-h7tw, the six `.ld-text-*` classes were runtime-only too, which
is exactly the shape of bug above: a consumer using `SectionHeading` without
mounting `UiTokensPreamble` got the heading's weight and colour but no size
step, and with Tailwind preflight resetting heading sizes, an `<h2>` rendered
at body size. They are now emitted both ways (same `ui_tokens::typography`
source, so they cannot drift apart) specifically because nothing in
`SectionHeading`'s or `KpiStrip`'s own documentation said mounting the
preamble was required — unlike the animation classes above, which do say so.
`tests/ld_class_stylesheet_coverage.rs` guards both properties: that the type
ramp works from `styles/tokens.css` alone, and that every literal `ld-*`
class any component emits is defined by *some* stylesheet this crate ships
(a lower bar, intentionally — see that file's module doc for what it does
and does not cover, including the currently-unreferenced `--ld-space-*`/
`--ld-stroke-*`/`--ld-radius-*` custom-property family).

### Component shell classes must compose with Tailwind

`.ld-card-depth` sets `--tw-shadow` plus the five-part shadow list like
Tailwind's own `shadow-*`; a bare `box-shadow:` on a component class silently
discards a layered `ring-*` (ldui-xr7i). Selection rings are `outline`s — a
box-shadow ring counts as a rogue DEPTH finding in the style audit — and
keyboard focus is the framework's `ld-focus-ring`, which outranks any
`focus-visible:` utility you add.

### Tooltip bubbles occupy layout at opacity 0 — measure edges, never index them

daisyUI's tooltip bubble is a `::before` pseudo-element: absolutely positioned,
`width: max-content`, `max-width: 20rem`, and present at `opacity: 0` before any
hover. An opacity-0 box still extends its containing block's scrollable
overflow, so a centred `tooltip-top` bubble on a trigger near a container's
right edge adds up to ~160px of horizontal scroll **at rest** — a phantom
scrollbar, and exactly what the `ldui-k3ip` admin-workbench contract measures.

Which trigger sits at an edge is a question of *rendered* geometry. A card's
list index cannot know where a wrapped row ends (the columns are container
queries Rust never sees), and a rung-gated CSS variant cannot either — both
were tried for `KpiStrip` (ldui-rzvv) and both failed the contract. The crate's
answer is `components::tooltip::geometry`, promoted from `RecordHeader`: the
container owns a `ResizeObserver` that bumps a generation; each trigger
measures itself against the container on mount, on every bump, and on
hover/focus, then writes the `Tooltip`'s `position`. A declared edge is only
the first-paint seed. Prove it in a lane by reading `data-tooltip-placement`
*and* the bubble's computed `right` inset (`auto` means `tooltip-top`; a length
means `tooltip-left` really applied), at two container widths.

## Component Guidelines

When adding or modifying components:

1. **Style Enums**: Define enums for all daisyUI variants (color, size, style) with `as_str()` method returning CSS class
2. **Props**: Use `#[prop(optional, into)]` for optional reactive props that accept `Signal<T>`
3. **Spread Attributes**: Do not declare a `#[prop(attrs)] attributes: Vec<(&'static str, Attribute)>` prop — that is a stale Leptos 0.6 idiom and no component in this repo uses it. Leptos 0.8 forwards spread attributes (`attr:`, `class:`, `style:`, `on:`, `prop:`) from call sites onto a component's root HTML element automatically, as long as the component's view resolves to a single root element. Declare an explicit `#[prop(optional, into)] class: &'static str` prop for user-supplied classes and merge it with the component's own daisyUI classes via the `merge_classes!` macro (see `src/components/button/component.rs`, `src/components/checkbox/component.rs`)
4. **Documentation**: Include:
   - Component-level doc comment with usage example
   - CSS classes needed in `input.css` via `@source inline(...)`
   - Node reference documentation with MDN link

5. **Module Structure**:
   ```rust
   // component.rs
   #[component]
   pub fn ComponentName(...) -> impl IntoView { ... }

   // style.rs
   #[derive(Clone, Debug, Default)]
   pub enum ComponentColor { ... }

   // mod.rs
   pub use component::*;
   pub use style::*;
   ```

6. **Chart/Gantt SVG paint routes through `src/charts/paint.rs`.** `var()`
   inside an SVG *presentation attribute* is not specified to substitute, so a
   theme token there degrades to `fill: black` or `stroke: none` silently.
   `tests/svg_paint_routing.rs` scans all of `src/` and is a gate step.

### Spacing rules (enforced, not advisory)

Every spacing value must be a member of the canonical scale — **4, 8, 12, 16,
24, 32, 48, 64, 96 px** — which is `ui_tokens::spacing::SCALE`, shared with the
Direct2D desktop face. In Tailwind terms that is `1, 2, 3, 4, 6, 8, 12, 16, 24`.

- **Sub-4px values are strokes, not spacing.** A 1px divider and a 1px gap are
  different decisions that happen to share a number. Borders, dividers, rules,
  indicator bars and hit-target widths use the stroke family
  (`--border-width-hairline/thin/accent/emphasis`, from `ui_tokens::stroke`) and
  are excluded from the spacing checker. `2px` is **not** a sanctioned spacing
  step — see `ldui-mai.2` for the reasoning.
- **Internal ≤ external.** A container's padding must not exceed the gap
  separating it from its neighbours, and a child's padding must not exceed the
  gap between children — otherwise the two read as one group (Gestalt
  proximity). This is the bug class behind the Kanban card fix.
- **Never hardcode a dimension the tokens already name.** The nav rail's active
  bar is `w-(--border-width-accent)`, not `w-1`, because the desktop draws 3px
  there and a literal silently drifts.
- **Sizes are a third family, not spacing.** Spacing answers "how far apart?"
  and must land on the canonical scale. A *size* ramp answers "how big?" and
  follows its own roughly-geometric progression — `IconSize` is 16/20/24/32/48
  and `IconTileSize` is 24/32/40/48/64. The 20 and 40 steps are on the 4px grid
  but deliberately off the 9-step scale: snapping them would collide with their
  neighbours and collapse a 5-step ramp to 4. The shared token crate takes the
  same position (`TABLE_ROW_HEIGHT` is 40). Use the enums — don't write
  `w-5 h-5` for an icon.

`cargo xtask test-layout` asserts all of this against the rendered DOM
(`tests/layout_audit_smoke.rs`). Overlap is a hard failure; grid and
internal-vs-external are ratcheted per page and may only be lowered. Full
findings: [`doc/plans/2026-07-26-spacing-audit.md`](./doc/plans/2026-07-26-spacing-audit.md).

### Muted text: `/75` or darker

On the light theme `text-base-content/60` is 3.37:1 and `/70` is 4.36:1 --
both fail WCAG AA for 14px normal text. `/75` (5.03:1) is the smallest step
that passes and is already the crate's dominant muted step. A consumer's axe
found the saved-filter caption at `/60` (3c35912). Run axe in EVERY state a
component has: a caption that renders only after a save is invisible to an
at-rest axe pass.

### daisyUI 5, not 4

`.form-control`, `.label-text` and `.label-text-alt` were **removed in
daisyUI 5** and do nothing. They are not harmless leftovers: `.form-control`
supplied `display:flex; flex-direction:column`, so without it a
`<label class="form-control w-full">` falls back to `display:inline`, `w-full`
goes inert, and the label and its input flow inline instead of stacking. Use
`fieldset` + `label`, or plain `flex flex-col gap-2`. The `test-daisyui5` gate
step (`tests/no_dead_daisyui4_classes.rs`) fails if any of the three reappears.

### Wrapper elements inside a daisyUI `menu` pick up the item grid

daisyUI styles a menu item's content box as `display: grid;
grid-auto-flow: column; align-items: center`, so an icon, label and badge share
one line. Its selector excludes a direct `ul` and `.menu-title` — **but not a
wrapper you insert**. `MenuItem`'s structural (non-interactive) `is_submenu`
branch wraps children in a `<span>` to keep an id/role/tabindex for roving
focus, and that span collected the item grid: it laid a `MenuTitle` out
*beside* its `SubMenu` instead of above it, turning the showcase sidebar's
group headings into labels floating left of their own link columns (`ldui-1n3`).

The wrapper therefore carries `contents` (`display: contents`) so it generates
no box and its children rejoin the `<li>`'s flow — daisyUI's documented
`<li><h2 class="menu-title"><ul>` structure. If you add any wrapper inside a
menu `<li>`, expect the same and do the same.

## Development Notes

- **Edition**: Uses Rust Edition 2024
- **Leptos Version**: 0.8 with CSR features
- **daisyUI Version**: Targets daisyUI 5
- **Tailwind CSS**: Version 4 compatibility
- **Upstream a consumer's vendored deltas by three-way merge against THAT
  CONSUMER'S re-vendor base** (the vendored tree as it stood at its last sync),
  never by copying its files or merging against the ldui commit it synced
  from — the latter replays deltas this repo has since re-implemented
  differently.

### Publishing: internal path-dep fork only (`publish = false`)

This is an **internal fork** of `noshishiRust/leptos-daisyui-rs` and is **not**
published to crates.io (`publish = false` in `Cargo.toml`). It is consumed as a
**path dependency** by sibling repos in the portfolio (EUC, Rust-DeskApp). Two
reasons it cannot be published: the crate name `leptos-daisyui-rs` is owned
upstream, and it depends on seven sibling path crates with no crates.io release
(`table-rs`, `ui-tokens`, `ai-chat-core`, `editmark-mermaid`, `editmark-core`,
`pixelproof-web`, `pixelproof-style-audit`), which makes `cargo publish`
impossible. Do not attempt to add version pins or a docs.rs setup — the decision
is to keep this a path-dep-only internal library.

### Development Dependencies

**table-rs (Temporary Fork)**: This project currently uses a local fork of table-rs with security and stability fixes.

- **Location**: `../table-rs` (path dependency)
- **Reason**: Applying security/stability fixes and enhancements pending upstream contribution
- **Status**: Temporary - will submit PRs to upstream table-rs project
- **Long-term Goal**: Switch back to official crates.io version once PRs are merged

**Setup Requirements**:
- Clone the table-rs fork to `C:\dev\table-rs` (sibling directory)
- The fork includes security fixes, stability improvements, and functionality enhancements
- Changes in table-rs will be immediately reflected when rebuilding leptos-daisyui-rs

## Known Limitations

- Currently assumes CSR usage only
- Each consumer must scan this crate's Rust source and import
  `styles/tokens.css` in its own `input.css`; class and token delivery is not
  automatic across a Rust path dependency
- Some components cannot be used with alternative HTML elements (e.g., button styles on `<a>` tags) - workaround is creating wrapper components that add classes to children

## Documentation

Component documentation lives in `doc/components/` with markdown files for major components. Reference the daisyUI docs at https://daisyui.com/components/ for design specifications.
