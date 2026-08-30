# CI/CD — local-only, two-layer (xtask + cargo-make)

leptos-daisyui-rs's CI/CD runs **entirely locally**. There is no GitHub Actions,
no hosted CI. This follows the org guiding principle
(`~/.claude/rust-ci-cd-build-strategy.md`): all CI/CD is executed and controlled
locally, so the local runner **is** the pipeline. The model mirrors the sibling
Rust-DeskApp repo's `docs/ci-cd.md`, adapted for this repo's shape — a single
CSR library crate plus a separate demo crate, seven sibling path-deps, and
`publish = false`.

## Two layers, never conflated

| Layer | What | Where |
|---|---|---|
| **Orchestration** | the task list / entry points | [`Makefile.toml`](../Makefile.toml) (cargo-make) + the `cargo xtask` alias in [`.cargo/config.toml`](../.cargo/config.toml) |
| **Logic** | the actual checks (fmt scope, clippy/build/test scoping, advisory summary, version bump) | [`xtask/`](../xtask) — a zero-dependency Rust binary |

The orchestrator holds **no logic** — every task just calls `cargo xtask <sub>`.
`launcher.ps1`'s check/fix/verify entries route through the same tasks.

## Workspace shape

The repo is a Cargo **workspace** whose members are exactly the four crates it
owns:

```toml
# root Cargo.toml
[package]
name = "leptos-daisyui-rs"   # the library, also the workspace root
publish = false

[workspace]
members = [".", "xtask", "demo", "audit"]
resolver = "3"
```

- `.` — the `leptos-daisyui-rs` library (the product).
- `xtask` — the pipeline logic binary.
- `demo` — the `leptos-daisyui-showcase` CSR app (built for real via `trunk`).
- `audit` — `ldui-audit`, the visual-quality audit surface: the
  `pixelproof-style-audit` engine's rule model plus this library's daisyUI
  component-drift rules and `ui-tokens` profile defaults. A dev-dependency of
  the library's browser suites, and of consumer apps that audit their own
  screens.

The seven sibling path-deps (`table-rs`, `ui-tokens`, `ai-chat-core`,
`editmark-mermaid`, `editmark-core`, `pixelproof-web`,
`pixelproof-style-audit`) are **dependencies, not members** — they live outside
this repo under `C:\dev`.

Two workspace-wide commands turned out **not** to be safe here (verified
empirically when the workspace was created), so the gate scopes explicitly:

- **`cargo fmt --all` reaches into sibling repos** (e.g. `aws-update/...`, a
  transitive local path-dep) — 300+ diffs in code this repo doesn't own. So
  `fmt` is run **per-package** (`-p leptos-daisyui-rs -p leptos-daisyui-showcase
  -p xtask -p ldui-audit`), never `--all`. (Same hazard Rust-DeskApp's doc warns
  about.) Every member is named, so adding one means adding it here too.
- **`cargo clippy --workspace` fails on feature unification** — co-building the
  demo enables `leptos`'s `csr` feature on the library, surfacing csr-only lints
  in the lib that don't exist when it's built standalone (as `cargo test --lib`
  builds it). So `clippy` is run **per-crate** (lib, demo and audit separately).

## Running it

```bash
cargo xtask verify      # zero extra tooling — just the Rust toolchain
cargo make verify       # same thing via cargo-make (needs `cargo install cargo-make`)
```

Individual steps: `cargo xtask fmt-check | clippy | build | check-demo | test`
(or the matching `cargo make` tasks).

The gate is **advisory-first**: it runs every step even if an earlier one fails,
then prints a PASS/FAIL summary. The process exit code is the number of failed
steps (0 = all green), so a hook or script can gate on it.

## The gate — `cargo xtask verify`

Each step is scoped per-crate; that scoping **is** the xtask's logic.

| Step | Command | Note |
|---|---|---|
| `tokens-fresh` | `cargo xtask gen-tokens --check` | Fails if `styles/tokens.css` no longer matches what the `ui-tokens` crate produces — i.e. the desktop and web faces have silently forked. First because it is the cheapest and because a stale theme invalidates every downstream visual result. |
| `sibling-tokens` | `cargo xtask check-sibling-tokens` | Fails if `src/tokens/preamble.rs` references a `ui_tokens` item that does not exist on `../Rust-DeskApp`'s **default** branch. See below — this is the one break no other step can see. |
| `fmt-check` | `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask -p ldui-audit -- --check` | Per-package, **not `--all`** — `--all` reaches into sibling repos (see above). Every workspace member is named explicitly, so a new member has to be added here or it goes unformatted. |
| `clippy-lib` | `cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings` | Per-crate — **not `--workspace`**, which fails on csr feature unification (see above). Host target. `--features test-mode` because `src/test_mode.rs` is behind that feature and a default-feature clippy never lints it. |
| `clippy-demo` | `cargo clippy -p leptos-daisyui-showcase --all-targets -- -D warnings` | Same per-crate reason. |
| `clippy-audit` | `cargo clippy -p ldui-audit --all-targets -- -D warnings` | The visual-quality audit crate (`audit/`), which composes the `pixelproof-style-audit` engine with the daisyUI drift rules. |
| `clippy-xtask` | `cargo clippy -p xtask --all-targets -- -D warnings` | The gate must lint the crate that **is** the gate. Its absence let a `needless_borrows_for_generic_args` sit in `xtask/src/main.rs` from 2026-07-26 to 2026-08-10 while `verify` reported a clean 13/13 — `test-xtask` was running its tests all along, so only the lint was missing (ldui-mpm). |
| `build` | `cargo build -p leptos-daisyui-rs` | **Library only.** The CSR demo is not natively built here — a native `cargo build` of a wasm/CSR binary can link-fail on `web-sys` host stubs; the demo is *checked* instead (next row) and *really* built by `trunk` (see `verify-full`). |
| `check-demo` | `cargo check -p leptos-daisyui-showcase` | Fast native check of the demo — catches ~all compile breakage without npm/trunk. |
| `test-lib` | `cargo test -p leptos-daisyui-rs --lib --features test-mode` | The library's 2,382-test suite. Non-`#[ignore]`d tests only. `--features test-mode` for the same reason as `clippy-lib`: without it the 7 `test_mode` tests silently do not run, and that module is what the browser suites' freeze/oracle bridge is built on. |
| `test-xtask` | `cargo test -p xtask` | The xtask's own pure-function tests (SemVer bump, the sibling-token parser, the gate's own argument vectors). |
| `test-audit` | `cargo test -p ldui-audit --lib` | The audit crate's browser-free tests: the generated sweep JS (rule ids, the per-family cap, the percentage-radius conversion) and the drift/engine report merge. |
| `test-daisyui5` | `cargo test -p leptos-daisyui-rs --test no_dead_daisyui4_classes` | Source scan (no browser) guarding against `.form-control` / `.label-text` / `.label-text-alt` coming back — removed in daisyUI 5, so they are silently inert. |
| `test-svg-paint` | `cargo test -p leptos-daisyui-rs --test svg_paint_routing` | Source scan (no browser) over **all of `src/`**: no `fill=`/`stroke=`/`stop-color=`/`flood-color=`/`lighting-color=` may carry a custom property, and any non-literal value must be a `charts::paint` binding. `var()` substitution is not specified to run in a presentation attribute, so a token there degrades to `fill: black` or `stroke: none` **silently, with no console error**. It has to be its own step because `test-lib` runs unit tests only — an integration test not named here never runs in the gate at all. Scoped to `src/charts` originally, which is exactly how it read green over four live defects in `src/components/gantt/` (ldui-1g5, widened in ldui-xxc). |

### Pattern-scoped verification

Opinionated patterns have a checked, fail-closed manifest in
`xtask/src/pattern_checks.rs`. For the client-snapshot list:

```bash
cargo xtask verify-pattern client-snapshot-list --inner
cargo xtask verify-pattern client-snapshot-list --browser

# Thin cargo-make aliases:
cargo make verify-client-snapshot-inner
cargo make verify-client-snapshot-browser
```

`--inner` runs only the page-contract/pattern unit tests and typed
`EntityTable` model/storage tests. `--browser` compiles and serves
`demo/client-snapshot-test-host.html`, whose dedicated binary links the one
pattern family under test rather than the full component catalog, then runs
`tests/entity_table_smoke.rs` and `tests/snapshot_table_smoke.rs` consecutively
against the same verified server. Unknown pattern names and lane flags exit 2;
there is no fallback to a broad suite.

The browser lane records
`target/pattern-checks/client-snapshot-list/browser.fingerprint` only after the
page host starts successfully. The manifest includes aggregate relevant-source
hashes, `Cargo.lock`, generated `styles/tokens.css`, Rust and Trunk versions,
and a command-version marker. A matching run explicitly reuses Cargo/Trunk
artifacts; a changed input delegates safe incremental invalidation to those
tools. The fingerprint is evidence and selection metadata, not a replacement
build cache or a bespoke pipeline runner.

The page-scoped journeys cover the semantic/responsive table contract, the
utility-plus-aligned controlled filter model, schema-projected explicit default
saves, reactive localized column semantics, generation-scoped row-action focus
recovery, local sort/page/column behavior, dataset-selector isolation, refresh
DOM retention, action isolation, and engine-driven style/layout checks at wide
and compact widths. They also prove the typed `SnapshotTablePage` slot order,
generation coherence, atomic replacement, and retained table-node identity;
the injected selector/filter marker swap is the ordering negative control.
The visual ratchet is zero for overlap, typography, shape,
grid, internal spacing, and component drift. Depth is exactly 5 for daisyUI's
`oklab()`/`oklch()` form-control shadows, an existing parser limitation; the
test declares the showcase's authored button shadows instead of spending
ceiling on known values. An inject/catch/remove negative control proves the
new style and layout rules are live.

### `cargo xtask check-sibling-tokens` — a path dep hides an unmerged branch

`ui-tokens` is a **path** dependency, so cargo resolves it to whatever
`../Rust-DeskApp` currently has *checked out*. An item that exists only on an
unmerged branch therefore compiles here perfectly, and every step above stays
green — while `main` is in fact unbuildable for anyone whose sibling sits on
its default branch. The failure surfaces only in a downstream consumer, where
it reads as that consumer's own fault.

That is not hypothetical: on 2026-07-29 `SPACE_HUGE`, `SPACE_XXXL`,
`LINE_DISPLAY` and the whole `stroke` module were branch-only, and it cost a
4iiz-office session hours of chasing a break that was never theirs.

The step resolves the sibling's default branch via `origin/HEAD` and asserts
every `ui_tokens` item the preamble references exists *there*, ignoring the
working tree. It reports which branch each missing item was found on instead —
usually naming the branch that still needs merging. Two things it handles that
a naive grep would not: items reached through a module alias (`ty::LINE_DISPLAY`
never appears in a `use` line), and tokens merely *named in prose* by a doc
comment. If the sibling is absent, or has no `origin/HEAD`, the step **skips**
rather than fails, so clones without it (EUC, CI) still gate cleanly.

#### What this gate still cannot see: the sibling's *formatting*

`ui-tokens` is a path dependency, **not** a workspace member, so
`cargo fmt -p ui-tokens` fails here with "not a member of the workspace". The
per-package `fmt-check` above therefore cannot cover it — by construction, not
by omission. Nothing in this repo's gate will ever tell you that a change you
landed in the sibling is unformatted.

That bit on 2026-07-29. Two `ui-tokens` commits were cherry-picked onto
Rust-DeskApp `master` to unblock this repo; `cargo check --workspace` and the
sibling's tests were run, but not `cargo fmt`, and two unformatted array
literals in `spacing.rs` rode along. This repo stayed 10/10 green. The break
surfaced in **editmark**, whose `cargo fmt --all` *does* reach path deps —
`--all` means "all packages and their local path-based dependencies" — turning
its release gate red over a repo it never compiles. Fixed in Rust-DeskApp
`3804fdd`; editmark has since scoped its own `format` task to explicit `-p`
flags, which is the same workaround this repo uses.

**So: after landing anything in `../Rust-DeskApp/crates/ui-tokens`, run
`cargo fmt -p ui-tokens` in that repo.** No gate on this side will catch it,
and the repo that does catch it is one nobody thinks to look at.

### `cargo xtask gen-tokens` — the Tailwind theme is generated, not written

`styles/tokens.css` is **generated** from the `ui-tokens` crate and imported by
`demo/input.css`. Never hand-edit it; run `cargo xtask gen-tokens` and commit
the result. The gate's `tokens-fresh` step re-runs the generator with `--check`
and fails if the committed file differs.

It emits the spacing base unit, the stroke family, radii, and the type ramp
with its grid-aligned line heights. Two deliberate choices, both load-bearing:

- **rem, not px.** The token crate stores DIPs because Direct2D has no rem. The
  generator converts to rem so the web keeps scaling with the user's browser
  font-size preference — emitting the raw DIPs as px would pin every gap and
  font size against that preference (WCAG 1.4.4). Only border widths stay px: a
  hairline must not grow with the type.
- **No named `--spacing-*` keys.** Tailwind resolves `max-w-*`/`w-*` against the
  `--spacing-*` namespace *before* `--container-*`, so adding a `--spacing-xs`
  key silently redefines `max-w-xs` from 20rem to 0.5rem. The numeric scale is
  already token-derived via `--spacing`; semantic aliases would buy nothing and
  cost a namespace collision. A unit test guards this.

The generated theme is behaviour-preserving by construction: compiling
`demo/input.css` with and without the import produces zero semantically
different theme values.

### `cargo xtask verify-full` — with the browser suites and the real wasm build

`verify-full` runs `verify`, then the client-snapshot and SnapshotTable
page-scoped browser lanes, then builds the full catalog once in release mode and
reuses that same verified server for the 51-check reactivity/DOM-oracle suite
(`test-reactivity`), layout audit (`test-layout`, below), style audit
(`test-style`, below), the focused `KeyedResultList` browser proof
(`test-keyed-result-list`, ldui-r1z), the focused `SectionHeading`
browser proof (`test-section-heading`, ldui-lwu), and the focused
`SearchPickerDialog` browser proof (`test-search-picker-dialog`, ldui-i95p).
That catalog server is the real `wasm32-unknown-unknown` release build, so a
second standalone `trunk build --release` would only repeat the same pipeline
and is intentionally absent. It is a **separate task**, not part of the
default gate, because it needs `npm` + `trunk` + `tailwindcss` + Chrome
installed and takes minutes — keeping `verify` fast and zero-tooling. Run
`verify-full` before a release or when touching wasm-only / CSS-affecting
code.

The page-scoped host and catalog are two distinct HTML/Wasm targets and therefore
require two server builds. Consecutive suites for the same target share one
server. In measured warm runs, Cargo's catalog compile was under one second but
Trunk's Wasm optimization took roughly two minutes per invocation; sharing the
catalog server across its six suites (reactivity, layout, style, the
focused `KeyedResultList` proof, the focused `SectionHeading` proof, and the
focused `SearchPickerDialog` proof) removes five redundant optimization passes
from `verify-full`.

### Gate cadence during a live Beads drain

`cargo xtask verify` is the 14-step native gate listed in the table above.
`cargo xtask verify-full` adds ten browser/Wasm checks and reports 24 steps.
Say which command is running before starting it; "the verification gate" is
ambiguous because the two commands have materially different cost and coverage.

Use this cadence when working through an issue queue:

1. While an issue is active, run the narrowest test or pattern lane that proves
   the changed behavior. This keeps the red/green loop short.
2. When a fresh queue inventory appears empty, run the broad gate required for
   the final candidate tree: `verify` for native-only work, or `verify-full`
   when browser/Wasm/CSS behavior changed or release-level evidence is required.
3. Immediately after any long gate, and again before landing, run
   `bd ready --json` plus `bd list --status open --json`, `in_progress`, and
   `blocked`. `bd ready` is only the runnable subset, and every result is a
   snapshot; audits can add work while the gate is running.
4. If late-arriving work changes the candidate tree, run its focused checks and
   then repeat the affected final gate. A previous pass describes the previous
   tree, not the one being shipped.

## Testing policy — screenshot vs. no-screenshot

The dividing line for what may run in an automated gate is **screenshot vs. no
screenshot**, not headed-vs-headless (the same rule Rust-DeskApp uses). Which
non-screenshot lane runs by default is a separate tooling-cost decision:

- **No screenshot → eligible for an automated gate.** Two suites qualify:
  - The library's `cargo test --lib` suite is pure logic (enum/`as_str`
    mappings, layout/date math, pagination windowing, class building, queue
    behavior) and runs headlessly in `verify`.
  - The **51-check reactivity/DOM-oracle** suite
    (`tests/reactivity_smoke.rs`) drives real
    CDP input at the demo app and asserts internal Leptos state through the
    `window.__APP_DEBUG__` oracle — no pixels, so it is deterministic across
    machines. It runs only when explicitly requested through
    `cargo xtask test-reactivity` or `verify-full`; it is not part of ordinary
    `verify` rebuilds. It lands in `verify-full` rather than `verify` because it
    needs npm/trunk/Chrome and a wasm build; `verify` stays zero-tooling.
- **Screenshot / live-browser → manual.** The screenshot suite
  (`tests/visual_smoke.rs`, baselines under `tests/visual/baselines`) is
  `#[ignore]`d and run on demand via `cargo make test-visual`
  (`scripts/test-visual.ps1`: idempotent `npm install`, `trunk serve` on :3010,
  run the ignored tests, tear down; refresh baselines with
  `VISUAL_TEST_MODE=capture`). SSIM/baseline comparisons are DPI/monitor-specific
  and stay out of every gate. Ad-hoc visual checks also use the `run` /
  `visual-ui-testing` flow against a live `trunk serve` + Chrome DevTools MCP.

#### Recapturing baselines is a claim, not a refresh

`VISUAL_TEST_MODE=capture` overwrites the committed PNGs with whatever is on
screen — which asserts that what is on screen is *correct*. Blessing a
rendering blind also destroys the suite's ability to ever report it, because
the broken state becomes the reference.

This is not hypothetical. On 2026-07-27 all nine baselines failed (SSIM
0.69–0.96 against a 0.98 threshold) after three weeks of unrelated work, and
comparing one render against its baseline is what surfaced `ldui-1n3` — a real
regression where daisyUI's menu-item grid had unstacked every sidebar group
title. A straight recapture would have made it permanent and invisible.

The procedure that worked, and the one to repeat:

1. Run in compare mode and read the SSIM per page.
2. **Fix what the diffs reveal first.** Fixing `ldui-1n3` alone took the
   failures from 9 to 5 — four pages needed no recapture at all.
3. Eyeball each remaining render *against its baseline* and classify every
   difference as intended or regression. An 0.88 on a page that recently
   gained features is exactly as likely to be a broken table as a better one.
4. Capture all baselines in **one run**, so the set is mutually consistent —
   one browser, one build — rather than mixed vintages.
5. Re-run in compare mode to confirm the new set actually passes.

Because the suite is `#[ignore]`d and out of every gate, drift accumulates
silently until someone asks. Run it at feature boundaries; a failure then
means something, where a quarterly run just means "lots changed".

### `cargo xtask test-reactivity` — the self-spawning subset

This is the independently selectable 51-check lane. Run it when reactivity,
browser interaction, or localized state behavior needs proof; an ordinary
native rebuild does not invoke it implicitly.

The step owns the whole server lifecycle in Rust (logic in the xtask; the
PowerShell script stays the manual/screenshot path):

1. `npm install` in `demo/` if `demo/node_modules` is missing (Trunk's Tailwind
   pre-build hook needs it).
2. Reserve a **free port from the OS** (bind `127.0.0.1:0`, read it back, release
   it) and `trunk serve` on it. Each invocation gets its own port rather than
   contending on the shared `:3010` — the shared-port flake documented in
   Rust-DeskApp's `doc/ci-cd.md`.
3. Poll the served HTML and its hashed `output-*.css` until the HTML references
   the requested Wasm binary and the CSS carries the current build's unique
   stamp. A plain `200` is insufficient: Trunk can bind its port and serve a
   previous target from `dist/` while the new pipeline is still running.
   15-minute budget; aborts early if the `trunk` child exits.
4. Run `cargo test -p leptos-daisyui-rs --test reactivity_smoke -- --ignored
   --test-threads=1` with `VISUAL_TEST_BASE_URL` pointed at that port.
   `--test-threads=1` because each test drives its own headless Chrome loading
   the catalog Wasm; parallel instances starve each other past the mount-wait
   budget.
5. Kill the `trunk` process tree on drop (`taskkill /T /F` on Windows — Trunk
   spawns cargo/wasm-bindgen children).

The xtask browser gates always own this server. They intentionally ignore a
caller-supplied `VISUAL_TEST_BASE_URL`, because an external server cannot prove
that it serves the requested page-vs-catalog Wasm target or the stylesheet
generated for the current source. Direct invocation of an ignored browser test
may still use that variable for interactive diagnosis; it is not a verification
gate.

When these catalog suites are adjacent inside `verify-full`, steps 1-3 and 5
wrap the group once: reactivity, layout, style, the focused
`KeyedResultList` proof (`test-keyed-result-list`), the focused
`SectionHeading` proof (`test-section-heading`), and the focused
`SearchPickerDialog` proof (`test-search-picker-dialog`) all use the same
current release server. Their standalone subcommands continue to own an
isolated server so they remain independently reproducible.

The suite keeps its `#[ignore]` attributes (the gate passes `--ignored`
explicitly) so that a bare `cargo test` with no server running still passes.
`#[ignore]` here means "needs a server", not "is manual".

### The stylesheet freshness guard — why a green audit now means something

Every browser suite audits **computed styles**, so all of them are worthless if
the stylesheet the browser loaded is not the stylesheet the repo currently
describes. That was not hypothetical: a `CssSyntaxError` in `demo/input.css`
made `tailwindcss` exit non-zero, **trunk swallowed its `pre_build` hook's
failure**, kept serving the *previous* `output.css`, and both audit suites
passed against it — `test-style` 7/7 and `test-layout` 9/9, exit 0. Every
ceiling and ratchet in the visual-quality system was measuring whatever CSS last
succeeded (ldui-hun). `verify` cannot see it either: it runs `cargo check` and
never invokes trunk.

Checking tailwind's exit code is not sufficient, because `dist/` also goes stale
for reasons tailwind never sees — a concurrent session's `trunk build` rewriting
`demo/dist` underneath a running suite failed nine layout tests on pages it
never touched. So the guard is the general one:

1. Trunk's `pre_build` hook runs [`demo/build-css.mjs`](../demo/build-css.mjs)
   instead of `npx tailwindcss` directly. On success it appends a **run-unique
   marker** to `output.css` and records it in `demo/.ldui-css-stamp`
   (gitignored, like `output.css`); on failure it records `fail` there and
   leaves `output.css` alone, so the evidence survives even though trunk
   discards the exit code.
2. `run_browser_suite`'s `DemoServer::start` runs the same script itself before
   spawning trunk, so a broken stylesheet fails the step in seconds rather than
   after an eight-minute wasm build.
3. Before launching any test, `DemoServer::start` fetches the served HTML and
   hashed stylesheet and requires both the requested Wasm binary and current CSS
   marker. This rejects an old `dist/` even when it already answers `200`.
4. `common::harness_at` — the single funnel `style_audit_smoke`,
   `layout_audit_smoke`, `visual_smoke` and `reactivity_smoke` all navigate
   through — asserts **per navigation** that the CSS the page loaded carries the
   current marker, and refuses to audit otherwise. Per navigation, not once per
   process, so a rebuild *during* a suite is caught too.

The marker is an unmatchable id rule (`#ldui-css-stamp-<token> { … }`), not a
comment: comments do not appear in the CSSOM (so verifying one would need a
network fetch) and are stripped by the CSS minifier in release builds. Nothing
in the demo carries that id, so the rule matches no element and cannot perturb a
computed style — confirmed empirically by both suites' zero-slack ceilings and
by `report_layout_backlog` returning identical per-page counts with and without
it.

Failure names the stylesheet build (`STYLESHEET BUILD FAILED` /
`STALE STYLESHEET`) rather than surfacing as a ceiling breach or a mount
timeout, which is where this defect used to send the reader.

### `cargo xtask test-layout` — the spacing/overlap audit

Same server lifecycle as `test-reactivity` (both go through
`run_browser_suite`), running `tests/layout_audit_smoke.rs`. This is the
article's manual "10-minute spacing audit" made permanent — three checks swept
over the rendered DOM via `getBoundingClientRect`:

1. **Overlap** — no two visible in-flow siblings may intersect. **Hard failure,
   no tolerance.** This is the regression test for the whole class of bug where
   a component grows and its neighbour does not move.
2. **Grid** — vertical gaps between stacked siblings must land on the canonical
   scale.
3. **Internal ≤ external** — a container's padding must not exceed the gap to
   its siblings, or the containers visually merge.

Checks 2 and 3 are **ratcheted, not zeroed**: each page carries a committed
ceiling in `PAGES`, and a ceiling may only be lowered. A rendered gap is the sum
of margins, line boxes, borders and daisyUI's own internal padding, much of
which this library does not control — zeroing that on day one is not
achievable, but letting it grow silently is what this stops.

Exempt from comparison, mirroring the desktop contract: invisible or zero-area
elements, out-of-flow elements (an overlay is *supposed* to cover its siblings),
inline boxes, and anything inside an open dialog/dropdown/modal. Full
containment is skipped rather than reported — that is nesting, not collision.

`cargo xtask test-layout` runs it, and it is a step of `verify-full`.

`sweep_detects_injected_violations` is a **negative control**: it injects a
deliberate 20px overlap and a 7px off-grid gap, asserts both are caught, then
removes them and asserts the counts return to baseline. A detector that reports
zero because it is broken is worse than no detector, because it reads as
evidence.

> Unlike the desktop half, this one measures rather than declares. The desktop
> suite carries a caveat that a semantic rect is a *declaration*, so a clean
> sweep there is only a ratchet against new declared-box collisions.
> `getBoundingClientRect` is a real post-layout measurement, so that caveat does
> not transfer: an overlap reported here is an overlap that really renders.

### `cargo xtask test-style` — the typography/shape/depth + drift audit

Same server lifecycle again (`run_browser_suite`), running
`tests/style_audit_smoke.rs`. `test-layout`'s sibling: the *same* engine sweep
(`ldui_audit::audit_page`) under a fuller profile, reading the families that
suite ignores.

- **typography** — computed `font-family` off the profile's declared family
  (i.e. a silent font fallback), and font sizes off `ui_tokens`' type ramp.
- **shape** — `border-radius` outside the declared radius set.
- **depth** — `box-shadow` outside `ui_tokens::elevation`, compared per
  component with epsilons rather than as strings.
- **component-drift** — the four daisyUI heuristics that are *not* in the
  engine because the engine knows no framework: a raw `<button>` without
  `.btn`, a raw `<table>` without `.table`, a pill-shaped text chip without
  `.badge`, and a text input with no `fieldset`/wrapping `label`/`label[for]`.
  They come from a second small in-page sweep (`audit/src/drift.js`) merged
  into the same report.

The profile is `ldui_audit::from_ui_tokens`, whose defaults are derived from
the shared token crate at compile time, with the font family read from the
running demo (`common::body_font_family`) rather than named — a family that is
merely *declared* proves nothing about what is serving.

**Ceilings are ratcheted per page and per family** in the suite's `PAGES`
table, exactly like `test-layout`: a ceiling may be lowered freely, and raising
one needs a reason in the commit message. daisyUI's own defaults report
non-zero typography and depth out of the box — that is the baseline the ratchet
tracks, not a blocker. Overlap is never ratcheted: gating goes through
`ldui_audit::verify`, which rejects a ceiling entry for it as a
misconfiguration.

Two things the suite asserts beyond the ceilings:

- **Truncation is a failure.** The engine caps each family at 200 violations
  and sets `AuditReport::truncated`, which makes the counts a floor rather than
  a total. Without an explicit assertion, a family saturated at the cap sits
  under any ceiling above it forever and every further regression passes while
  the report reads clean. Both audit suites call
  `common::assert_not_truncated`.
- **A negative control** (`sweep_detects_injected_style_and_drift_violations`)
  injects one deliberate violation per family, asserts each is caught, removes
  them, and asserts every count returns to baseline — including the drift rules,
  so a merge that silently dropped them cannot read as a clean page.

`report_style_backlog` is the reporting-only pass the committed ceilings are
filled from; run it explicitly after a deliberate change.

The rulebook — what each family means, the defect patterns behind them, and how
to fix rather than ratchet — is `doc/visual-quality/`.

## Versioning — the automated bump

Version bumping is release logic, so it lives in the xtask. The **level is a
human call** — we do *not* infer it from commit messages.

```bash
cargo xtask bump patch --dry-run   # preview
cargo xtask bump minor             # apply
# or via cargo-make:
cargo make bump-patch | bump-minor | bump-major
```

SemVer policy (`MAJOR.MINOR.PATCH`):

| Level | When | Effect |
|---|---|---|
| **patch** | bug fixes / corrections, no new functionality | `x.y.Z+1` |
| **minor** | new functionality / features | `x.Y+1.0` |
| **major** | large-scale refactors / generational upgrades | `X+1.0.0` |

`bump` rewrites **only** the `[package] version` of the `leptos-daisyui-rs`
library (not the demo's version, not a dependency's `version`, not `xtask`). The
SemVer arithmetic and the section-aware `Cargo.toml` rewrite are pure,
unit-tested functions (`bump_version`, `set_package_version` in
`xtask/src/main.rs`) — they run under `cargo xtask test`. After a bump: review,
`cargo build` to refresh `Cargo.lock`, commit, then tag `vX.Y.Z`.

Note this crate is `publish = false` and consumed purely by path (sibling repos
ignore its version), so the bump exists for tag/changelog discipline, not for
any consumer-visible effect.

## Opt-in pre-push hook

[`.githooks/pre-push`](../.githooks/pre-push) runs `cargo xtask verify` and
reports the result but **never blocks the push** (advisory). Enable per clone:

```bash
git config core.hooksPath .githooks
```

The repo already ships a bd (beads) `pre-commit` hook under `.git/hooks/`; the
advisory `pre-push` is additive and independent of it.

## Idempotency / resumability

Every gate step is read-only and re-runnable, so re-running `cargo xtask verify`
after a failure simply re-checks everything — resumability is inherent. `bump` is
the only mutating step; it is guarded by `--dry-run` and rewrites deterministic,
unit-tested output. This repo has **no sign / package / publish flow**
(`publish = false`), so there is no mutating release step that would need an
explicit check-before-do guard. If one is ever added, guard it with an
idempotency check (per the guiding-principle doc) rather than building a
checkpoint engine.

## Why no hosted CI

Per the org mandate, CI/CD is local-only: the siblings this repo path-depends on
already sit alongside it under `C:\dev`, so a hosted runner would have to
check out seven sibling repos via PATs to reproduce what a local `cargo xtask
verify` does for free. The local runner is the source of truth; the optional
pre-push hook is the only automation surface, and it is advisory.
