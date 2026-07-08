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
members** — they live outside this repo under `C:\dev`. Because they are not
workspace members, workspace-wide commands (`--all`, `--workspace`) never reach
into them. This is why `cargo fmt --all` is safe here, unlike Rust-DeskApp where
a sibling *was* a member and `--all` had to be avoided.

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
| `fmt-check` | `cargo fmt --all -- --check` | `--all` = the three members only (siblings aren't members), so this never touches sibling repos. |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | Host target; covers lib + xtask + demo. |
| `build` | `cargo build -p leptos-daisyui-rs` | **Library only.** The CSR demo is not natively built here — a native `cargo build` of a wasm/CSR binary can link-fail on `web-sys` host stubs; the demo is *checked* instead (next row) and *really* built by `trunk` (see `verify-full`). |
| `check-demo` | `cargo check -p leptos-daisyui-showcase` | Fast native check of the demo — catches ~all compile breakage without npm/trunk. |
| `test` | `cargo test -p leptos-daisyui-rs --lib` + `cargo test -p xtask` | The library's unit-test suite (~1766 tests) plus the xtask's own pure-function tests (SemVer bump, etc.). Non-`#[ignore]`d tests only. |

### `cargo xtask verify-full` — with the real wasm build

`verify-full` runs `verify` and then `trunk build --release` in `demo/`, which
performs the real `wasm32-unknown-unknown` compile plus the Tailwind CSS build.
It is a **separate task**, not part of the default gate, because it needs `npm`
+ `trunk` + `tailwindcss` installed and takes minutes — keeping `verify` fast and
zero-tooling. Run `verify-full` before a release or when touching wasm-only /
CSS-affecting code.

## Testing policy — screenshot vs. no-screenshot

The dividing line for what the gate runs automatically is **screenshot vs. no
screenshot**, not headed-vs-headless (the same rule Rust-DeskApp uses):

- **No screenshot → auto-gated.** The library's `cargo test --lib` suite is pure
  logic (enum/`as_str` mappings, layout/date math, pagination windowing, class
  building, queue behavior) and runs headlessly in `verify`.
- **Screenshot / live-browser → manual.** Visual verification of the demo (the
  daisyUI components rendering in a real browser) is DPI-, theme-, and
  browser-specific and is **not** in `cargo test`. Today it is done ad-hoc via
  the `run` / `visual-ui-testing` flow against a live `trunk serve` + Chrome
  DevTools MCP, exactly as during component work. If a committed browser
  DOM-oracle suite is ever added, gate it automatically; keep any
  SSIM/baseline-screenshot suite `#[ignore]`d and manual.

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
