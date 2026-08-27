# Repository Guidelines

<!-- bd-doctor-divergence: ok -->

## Project Structure & Module Organization

This Rust 2024 workspace contains the library (`src/`), reusable browser-audit
crate (`audit/`), Leptos CSR showcase (`demo/`), and build logic (`xtask/`).
Components normally live in `src/components/<name>/` with `component.rs`,
`style.rs`, `tests.rs`, and `mod.rs`; broader primitives live under `charts/`,
`markdown/`, `motion/`, `theme/`, `tokens/`, `utils/`, and `widgets/`.
Integration tests are in `tests/`, PixelProof baselines in
`tests/visual/baselines/`, and visual-quality rules in `doc/visual-quality/`.

## Build, Test, and Development Commands

- `cargo xtask verify` runs the 14-step local gate: generated-token checks,
  scoped formatting/clippy, builds, and native tests.
- `cargo xtask verify-full` adds the browser suites and release Trunk build;
  it requires npm, Trunk, and Chrome. Run `npm install` in `demo/` once.
- `cargo xtask test-style`, `test-layout`, or `test-reactivity` runs one
  self-hosted browser suite. `cargo make test-visual` runs screenshot checks.
- `cargo make dev` serves the showcase at `http://127.0.0.1:3010`.

Run xtask from the repository root. Do not use `cargo fmt --all` (it reaches
path dependencies) or `cargo clippy --workspace` (Leptos CSR feature
unification breaks it); use the scoped xtask commands.

When draining Beads, treat every queue read as a snapshot. Use focused checks
while an issue is active, reserve the broad required gate for the final
candidate tree, and announce the exact command before a long run. After every
long gate and immediately before landing, re-run `bd ready --json` plus the
open, in-progress, and blocked queries; a consumer audit can file new work while
tests are running. See `doc/ci-cd.md` for the gate cadence and step breakdown.

## Coding Style & Testing

Use rustfmt defaults, `snake_case` modules/functions, and `UpperCamelCase`
components and enums. Public APIs require rustdoc; keep an inline backtick span
on one `///` line to avoid the current clippy ICE. Follow existing Leptos 0.8
patterns: reactive props use `Signal<T>`, style variants map through `as_str()`,
and caller classes merge through `merge_classes!`. Target daisyUI 5 only.

Co-locate unit tests in each component and name behavior-focused tests in
`snake_case`. Browser tests are intentionally ignored and run through xtask.
Visual audit ceilings are zero-slack and ratchet down; new rules require an
inject/catch/revert negative control. Never hand-edit `styles/tokens.css`;
regenerate it with `cargo xtask gen-tokens`.

## Commits & Pull Requests

History uses scoped Conventional Commit subjects such as
`fix(audit): ...` and `feat(filter-sidebar): ...`, often ending with a beads ID.
PRs should explain behavior and risk, link beads, list exact verification
commands, and include reviewed screenshots/diffs for visible changes. Note and
justify any new-page ceiling or engine-driven measurement change.


<!-- BEGIN BEADS INTEGRATION v:1 profile:full hash:6c1e3c16 -->
<!-- BEGIN BEADS INTEGRATION -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Dolt-powered version control with native sync
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs via Dolt:

- Each write auto-commits to Dolt history
- Use `bd dolt push`/`bd dolt pull` for remote sync
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- END BEADS INTEGRATION -->
<!-- END BEADS INTEGRATION -->
