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

The repo is a Cargo **workspace** whose members are exactly the three crates it
owns:

```toml
# root Cargo.toml
[package]
name = "leptos-daisyui-rs"   # the library, also the workspace root
publish = false

[workspace]
members = [".", "xtask", "demo"]
resolver = "2"
```

- `.` — the `leptos-daisyui-rs` library (the product).
- `xtask` — the pipeline logic binary.
- `demo` — the `leptos-daisyui-showcase` CSR app (built for real via `trunk`).

The six sibling path-deps (`table-rs`, `ui-tokens`, `ai-chat-core`,
`editmark-mermaid`, `editmark-core`, `pixelproof-web`) are **dependencies, not
members** — they live outside this repo under `C:\dev`.

Two workspace-wide commands turned out **not** to be safe here (verified
empirically when the workspace was created), so the gate scopes explicitly:

- **`cargo fmt --all` reaches into sibling repos** (e.g. `aws-update/...`, a
  transitive local path-dep) — 300+ diffs in code this repo doesn't own. So
  `fmt` is run **per-package** (`-p leptos-daisyui-rs -p leptos-daisyui-showcase
  -p xtask`), never `--all`. (Same hazard Rust-DeskApp's doc warns about.)
- **`cargo clippy --workspace` fails on feature unification** — co-building the
  demo enables `leptos`'s `csr` feature on the library, surfacing csr-only lints
  in the lib that don't exist when it's built standalone (as `cargo test --lib`
  builds it). So `clippy` is run **per-crate** (lib and demo separately).

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
| `fmt-check` | `cargo fmt -p leptos-daisyui-rs -p leptos-daisyui-showcase -p xtask -- --check` | Per-package, **not `--all`** — `--all` reaches into sibling repos (see above). |
| `clippy` | `cargo clippy -p leptos-daisyui-rs --all-targets -- -D warnings` **then** `-p leptos-daisyui-showcase` | Two per-crate runs — **not `--workspace`**, which fails on csr feature unification (see above). Host target. |
| `build` | `cargo build -p leptos-daisyui-rs` | **Library only.** The CSR demo is not natively built here — a native `cargo build` of a wasm/CSR binary can link-fail on `web-sys` host stubs; the demo is *checked* instead (next row) and *really* built by `trunk` (see `verify-full`). |
| `check-demo` | `cargo check -p leptos-daisyui-showcase` | Fast native check of the demo — catches ~all compile breakage without npm/trunk. |
| `test` | `cargo test -p leptos-daisyui-rs --lib` + `cargo test -p xtask` | The library's unit-test suite (~1766 tests) plus the xtask's own pure-function tests (SemVer bump, etc.). Non-`#[ignore]`d tests only. |

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
(`test-reactivity`), then the layout audit (`test-layout`, below), then
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
