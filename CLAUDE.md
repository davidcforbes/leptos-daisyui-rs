# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust crate providing type-safe, reactive daisyUI 5 component wrappers for Leptos framework. The library wraps daisyUI components as Leptos components with proper type safety, leveraging Leptos's spread attributes functionality. Currently designed for CSR (Client-Side Rendering).

**Component Coverage: 109 components** (full daisyUI 5 coverage, plus custom app-shell/data/scheduling/motion components added since the original 96/96 milestone)

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
cargo xtask verify        # advisory gate: tokens/sibling-tokens/fmt/clippy/build/check-demo/test (exit = # failures)
cargo make verify         # same, via cargo-make
cargo xtask verify-full   # + reactivity suite + the real trunk wasm build (needs npm/trunk/Chrome)
cargo xtask test-reactivity          # reactivity/DOM-oracle suite alone (self-spawns a demo server)
cargo xtask test-layout              # layout audit: overlap/grid/internal<=external over the real DOM
cargo xtask gen-tokens [--check]     # regenerate styles/tokens.css from ui-tokens
cargo xtask check-sibling-tokens     # preamble.rs's ui_tokens refs must exist on the sibling's DEFAULT branch
cargo xtask bump patch|minor|major   # bump the library version (human-chosen level)
```

**Repo-specific gotchas the gate encodes (do NOT run these directly):**
`cargo fmt --all` reaches into sibling repos — fmt **per-package**
(`-p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask`). `cargo clippy
--workspace` fails on leptos-`csr` feature unification — clippy **per-crate**.

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
`test-layout` and `verify-full` each build the demo to wasm, which outruns a
10-minute foreground timeout on a cold target dir — run them in the background.
Any edit under `demo/src` invalidates that build, so re-run the suites *after*
the last demo change, not before: `test-layout`'s per-page violation counts are
ratcheted and a new demo section can move them.

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
# Runs on http://127.0.0.1:3000
# Or use: cargo make dev (from root)

# Build for production
trunk build --release
```

The demo automatically:
- Runs Tailwind CSS compilation via pre_build hook: `npx tailwindcss -i input.css -o output.css`
- Watches both `../src` (library) and `./src` (demo) for changes
- Showcases all 109 components with interactive examples

## Architecture

### Core Structure

The crate has two main modules:
- `src/components/` - daisyUI component wrappers (109 components: full daisyUI 5 coverage plus custom additions)
- `src/utils/` - Utility code: `ClassAttributes` for dynamic class management, plus
  reusable framework hooks (`DebouncedSignal`, `use_swr_resource`)
- `src/motion/` - Animation primitives (`Lerp`, `Transition`, `Keyframe`/`Track`, easing, spring, `use_animated` hook)

### Recent additions (2026-08-06, from the office-perf audit)

All driven by 4iiz-Office consumer findings (op-99t7/op-cy77.x/op-rrp9); the
audit files beads into this repo's tracker continuously — check `bd list`.

- **DataTable**: owned `Column.header` + `Signal<DataTableTexts>` (runtime
  localization); `row_key` (selection keyed by row identity, survives data
  replacement and sorts); `Column::action()` (action cells never trigger
  `on_row_activate`); `extra_filter` predicate + `toolbar` ViewFn slot;
  column-scoped free-text search (`Column::searched(false)` opt-out —
  renderer-only metadata never matches); `ServerDataTable` typed
  `TableQuery`/`on_query_change` API (page/size/search/sort/filters) with
  `filter_options` for population-wide dropdowns. Both variants share
  `TABLE_SCROLL_WRAPPER_CLASS` (horizontal overflow).
- **DayScheduler**: opt-in interaction contract — `on_event_activate`,
  `selected_event`, `on_event_move`/`on_event_resize` (keyboard Arrow /
  Shift+Arrow minute-delta requests; consumer owns the events),
  `event_content` renderer. Event blocks are index-keyed so focus survives
  moves.
- **Modal**: `label` / `labelled_by` / `described_by` accessible naming
  (`labelled_by` suppresses `aria-label`).
- **Field**: mints ids + provides `FieldContext`; `Input`/`Select`/`Textarea`
  auto-consume it (`label[for]`, `aria-describedby`, `aria-errormessage` +
  `aria-invalid`).
- **login_screen**: `ProviderLoginScreen` + `LoginProvider` — branded
  server-redirect OAuth landing (no credential state).
- **widgets**: `name_color_class`/`NAME_PALETTE` deterministic avatar
  palette (`AvatarBadge name=`, `Persona palette=true`); assignments pinned
  by test — changing the hash is a breaking visual change.
- **utils**: `use_event_source` (owned-lifecycle SSE, no `Closure::forget`)
  and `use_event_source_fetch` (authenticated SSE over fetch: headers
  callback per (re)connect, pure `SseParser`, `retry:`/`Last-Event-ID`).
- **LinkButton**: `href` is `MaybeProp<String>` (per-row routes).

### Recent additions (2026-07)
- **Spacing & vertical-rhythm system (2026-07-26).** `styles/tokens.css` is
  generated from `ui-tokens` and imported by `demo/input.css`, so Tailwind's
  numeric spacing scale and type ramp are derived from the same tokens the
  desktop face uses rather than merely agreeing with them. `src/tokens/preamble.rs`
  additionally emits `--ld-space-*`, `--ld-stroke-*`, `--ld-radius-*` and
  `--ld-text-*`/`--ld-line-*` at runtime, plus a `.ld-text-<step>` class per
  ramp step pinning both size and line height. `tests/layout_audit_smoke.rs`
  asserts overlap / grid / internal≤external over the rendered DOM. See the
  spacing rules under Component Guidelines and
  [`doc/plans/2026-07-26-spacing-audit.md`](./doc/plans/2026-07-26-spacing-audit.md).
- `sparkline/`, `empty_state/`, `icon_tile/`, `metric_row/`, `capacity_bar/`, `sla_chip/`, `nav_rail/`, `result_list/`, `day_scheduler/`, `toolbar/`, `tree/`, `week_view/` - new app-shell/data/scheduling components
- `day_scheduler` / `week_view` `hour_label` - optional
  `Callback<u32, String>` formatting the hour-gutter labels, overriding
  `hour_format` when supplied. `hour_format` (`HourFormat::TwentyFour` by
  default, `Twelve` on request) still covers the common case and mirrors
  d2d-ui's own two-variant enum; `hour_label` is the escape hatch for a locale
  neither format expresses. A localised consumer usually wants the *former* —
  driving `hour_format` off the active locale — so reach for the closure only
  when 24h and English AM/PM are both wrong.
- `vertical_steps/` - extended with additional layout options
- `src/motion/` - new animation module (see above)
- `src/utils/swr.rs` - stale-while-revalidate keyed resource cache
  (`use_swr_resource` / `SwrCache` / `provide_swr_cache`). Renders the cached value
  for a key instantly while a background fetch revalidates, so back-navigation
  doesn't spinner over data the user just saw. Demo: `/components/swr`.
- `data_table/auto_page.rs` + the `auto_page_size` prop - responsive paging: rows
  per page derived from the table's rendered height via a `ResizeObserver`.
- `data_table/filter.rs` + `Column::filterable()` - opt-in per-column filter row
  of dropdowns, ANDed with each other and with the existing `searchable` box.
- `data_table` `on_row_activate` + `row_click_kind()`/`RowClickKind` - opt-in row
  activation: a plain click activates (navigate/act on the row), a Ctrl/Shift
  click still feeds the selection state machine. With no callback registered,
  every click selects exactly as before.
- `data_table` keyboard operability + `row_is_interactive()` - when a table is
  interactive (`selected_rows` or `on_row_activate` supplied), rows are
  focusable (`tabindex=0`) with `aria-selected`, and Enter/Space (plus
  Ctrl/Shift) mirror a click. Plain display tables gain no tab stops. The
  internal `on_row_click` callback carries `(idx, ctrl, shift)` bools rather
  than a `MouseEvent` so mouse and keyboard share one path.

### Framework-purity rule (why `utils/` keeps growing)

When a host app (EUC, inventory-web, Rust-DeskApp) hand-rolls a **generic**
reactive pattern, promote it here rather than letting each screen re-derive it.
`utils/debounce.rs` and `utils/swr.rs` were both extracted after a host app
re-implemented them for the second/third time. App-specific logic stays in the app;
framework primitives live here.

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
/* In demo/input.css or your project's input.css */
@import "tailwindcss";
@plugin "daisyui";
@source "../src/**/*.rs";

/* Example: Button component classes */
@source inline("btn btn-neutral btn-primary ... btn-circle");
```

See `demo/input.css` for the complete list or `stytles/daisyui-components.css` to include everything.

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

### Publishing: internal path-dep fork only (`publish = false`)

This is an **internal fork** of `noshishiRust/leptos-daisyui-rs` and is **not**
published to crates.io (`publish = false` in `Cargo.toml`). It is consumed as a
**path dependency** by sibling repos in the portfolio (EUC, Rust-DeskApp). Two
reasons it cannot be published: the crate name `leptos-daisyui-rs` is owned
upstream, and it depends on five sibling path crates with no crates.io release
(`table-rs`, `ui-tokens`, `ai-chat-core`, `editmark-mermaid`, `editmark-core`),
which makes `cargo publish` impossible. Do not attempt to add version pins or a
docs.rs setup — the decision is to keep this a path-dep-only internal library.

### Development Dependencies

**table-rs (Temporary Fork)**: This project currently uses a local fork of table-rs with security and stability fixes.

- **Location**: `../table-rs` (path dependency)
- **Reason**: Applying security/stability fixes and enhancements pending upstream contribution
- **Status**: Temporary - will submit PRs to upstream table-rs project
- **Long-term Goal**: Switch back to official crates.io version once PRs are merged

**Setup Requirements**:
- Clone the table-rs fork to `C:\Development\table-rs` (sibling directory)
- The fork includes security fixes, stability improvements, and functionality enhancements
- Changes in table-rs will be immediately reflected when rebuilding leptos-daisyui-rs

## Known Limitations

- Currently assumes CSR usage only
- CSS classes must be manually added to `input.css` (no automatic purge-safe class inclusion yet)
- Some components cannot be used with alternative HTML elements (e.g., button styles on `<a>` tags) - workaround is creating wrapper components that add classes to children

## Documentation

Component documentation lives in `doc/components/` with markdown files for major components. Reference the daisyUI docs at https://daisyui.com/components/ for design specifications.
