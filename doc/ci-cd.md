# CI/CD — local-only, two-layer (xtask + cargo-make)

leptos-daisyui-rs's CI/CD runs **entirely locally**. There is no GitHub Actions,
no hosted CI. This follows the org guiding principle
(`~/.claude/rust-ci-cd-build-strategy.md`): all CI/CD is executed and controlled
locally, so the local runner **is** the pipeline. The model mirrors the sibling
Rust-DeskApp repo's `docs/ci-cd.md`, adapted for this repo's shape — a single
CSR library crate plus a separate demo crate, six sibling path-deps, and
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
| `build` | `cargo build -p leptos-daisyui-rs` | **Library only.** The CSR demo is not natively built here — a native `cargo build` of a wasm/CSR binary can link-fail on `web-sys` host stubs; the demo is *checked* instead (next row) and *really* built by `trunk` (see `verify-full`). |
| `check-demo` | `cargo check -p leptos-daisyui-showcase` | Fast native check of the demo — catches ~all compile breakage without npm/trunk. |
| `test-lib` | `cargo test -p leptos-daisyui-rs --lib --features test-mode` | The library's unit-test suite (~2045 tests). Non-`#[ignore]`d tests only. `--features test-mode` for the same reason as `clippy-lib`: without it the 7 `test_mode` tests silently do not run, and that module is what the browser suites' freeze/oracle bridge is built on. |
| `test-xtask` | `cargo test -p xtask` | The xtask's own pure-function tests (SemVer bump, the sibling-token parser, the gate's own argument vectors). |
| `test-audit` | `cargo test -p ldui-audit --lib` | The audit crate's browser-free tests: the generated sweep JS (rule ids, the per-family cap, the percentage-radius conversion) and the drift/engine report merge. |
| `test-daisyui5` | `cargo test -p leptos-daisyui-rs --test no_dead_daisyui4_classes` | Source scan (no browser) guarding against `.form-control` / `.label-text` / `.label-text-alt` coming back — removed in daisyUI 5, so they are silently inert. |
| `test-svg-paint` | `cargo test -p leptos-daisyui-rs --test svg_paint_routing` | Source scan (no browser) over **all of `src/`**: no `fill=`/`stroke=`/`stop-color=`/`flood-color=`/`lighting-color=` may carry a custom property, and any non-literal value must be a `charts::paint` binding. `var()` substitution is not specified to run in a presentation attribute, so a token there degrades to `fill: black` or `stroke: none` **silently, with no console error**. It has to be its own step because `test-lib` runs unit tests only — an integration test not named here never runs in the gate at all. Scoped to `src/charts` originally, which is exactly how it read green over four live defects in `src/components/gantt/` (ldui-1g5, widened in ldui-xxc). |

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

`verify-full` runs `verify`, then the reactivity/DOM-oracle suite
(`test-reactivity`), then the layout audit (`test-layout`, below), then the
style audit (`test-style`, below), then
`trunk build --release` in `demo/` — the real
`wasm32-unknown-unknown` compile plus the Tailwind CSS build. It is a **separate
task**, not part of the default gate, because it needs `npm` + `trunk` +
`tailwindcss` + Chrome installed and takes minutes — keeping `verify` fast and
zero-tooling. Run `verify-full` before a release or when touching wasm-only /
CSS-affecting code.

## Testing policy — screenshot vs. no-screenshot

The dividing line for what a gate runs automatically is **screenshot vs. no
screenshot**, not headed-vs-headless (the same rule Rust-DeskApp uses):

- **No screenshot → auto-gated.** Two suites qualify:
  - The library's `cargo test --lib` suite is pure logic (enum/`as_str`
    mappings, layout/date math, pagination windowing, class building, queue
    behavior) and runs headlessly in `verify`.
  - The **reactivity/DOM-oracle** suite (`tests/reactivity_smoke.rs`) drives real
    CDP input at the demo app and asserts internal Leptos state through the
    `window.__APP_DEBUG__` oracle — no pixels, so it is deterministic across
    machines. It is gated by `cargo xtask test-reactivity`, and runs as a step of
    `verify-full`. It lands in `verify-full` rather than `verify` because it
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

The step owns the whole server lifecycle in Rust (logic in the xtask; the
PowerShell script stays the manual/screenshot path):

1. `npm install` in `demo/` if `demo/node_modules` is missing (Trunk's Tailwind
   pre-build hook needs it).
2. Reserve a **free port from the OS** (bind `127.0.0.1:0`, read it back, release
   it) and `trunk serve` on it. Each invocation gets its own port rather than
   contending on the shared `:3010` — the shared-port flake documented in
   Rust-DeskApp's `doc/ci-cd.md`.
3. Poll `GET /` until it answers `200`, which means Trunk finished the first wasm
   build and wrote `index.html` (a stricter signal than "the port is bound",
   which Trunk does *before* building). 15-minute budget; aborts early if the
   `trunk` child exits.
4. Run `cargo test -p leptos-daisyui-rs --test reactivity_smoke -- --ignored
   --test-threads=1` with `VISUAL_TEST_BASE_URL` pointed at that port.
   `--test-threads=1` because each test drives its own headless Chrome loading
   the ~60 MB dev wasm; parallel instances starve each other past the mount-wait
   budget.
5. Kill the `trunk` process tree on drop (`taskkill /T /F` on Windows — Trunk
   spawns cargo/wasm-bindgen children).

Setting `VISUAL_TEST_BASE_URL` yourself skips steps 1-3 and 5 and reuses your
already-running dev server.

The suite keeps its `#[ignore]` attributes (the gate passes `--ignored`
explicitly) so that a bare `cargo test` with no server running still passes.
`#[ignore]` here means "needs a server", not "is manual".

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
check out six sibling repos via PATs to reproduce what a local `cargo xtask
verify` does for free. The local runner is the source of truth; the optional
pre-push hook is the only automation surface, and it is advisory.
