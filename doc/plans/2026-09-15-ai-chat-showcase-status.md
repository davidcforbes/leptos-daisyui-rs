# AI chat showcase epic (ldui-iilm) — branch status and resume guide

Last updated: 2026-09-16 (second session hand-off). Branch: `feature/ai-chat-showcase`,
worktree `C:\dev\ldui-aichat` (a sibling of the main checkout because the Cargo
path deps are `../Rust-DeskApp`, `../table-rs`, `../editmark`, `../PixelProof`).
Approved plan: `~/.claude/plans/cozy-chasing-boot.md`. Execution ledger with
every brief, report, review and diff package:
`.superpowers/sdd/cozy-chasing-boot/progress.md` (gitignored; the ledger, not
`git log`, says which commit is reviewed).

## What this epic builds

The demo's `/components/ai-chat` page becomes an opinionated showcase of every
AI feature already built across the editmark desktop chat and the 4iiz-Office
assistants, and is the **reference implementation Office copies**: four
providers (Claude Code, Codex CLI/Spark, Groq, Ollama) with per-provider model
choice and tuning, four knowledge sources (local corpus scopes, postgres-agent
AI Memory, Office knowledgebase + personal memory, grounded facts), typed
honesty/lifecycle states, usage that never sums plan and metered, bilingual
texts, an in-browser fixture by default and an opt-in Live mode against a
local `editmark-server`. Decisions that are not re-litigated: no API-key input
anywhere; `AiChat` stays generic, all opinion lives in
`patterns::ai_chat_workspace`; Ollama advertises `supports_tool_calls: false`;
attachments are out of scope (follow-up bead).

## Phase status

Eight of eleven phases are complete and their beads closed. Every phase went
through a task review; the ledger records each fix round and its scoped
re-review.

| Phase | Bead | State | Commits |
|---|---|---|---|
| P0 prereqs + orphan contract gate | ldui-iilm.1 | complete, reviewed | 1af0472 |
| P1 pure model + texts (`patterns::ai_chat_workspace`) | ldui-iilm.2 | complete, reviewed (1 fix round) | 0bd77bc, cfbaee0, 7f736c2 |
| P2 generic `AiChat` upgrades | ldui-iilm.3 | complete, reviewed (1 fix round) | e5ed0f2, 7000c6a |
| P3 shipped fixture (`memory.rs`, test-mode) | ldui-iilm.4 | complete, reviewed (1 fix round) | b6db0a3, 061338b |
| P4 composite + first page + `test-ai-chat` lane | ldui-iilm.5 | complete, reviewed (1 fix round) | c3753f2, 5c5c11d |
| P5 honesty + lifecycle | ldui-iilm.6 | complete, reviewed (1 fix round) | 3d9c617, 366511a, e9b736c |
| P6 knowledge + `test-ai-chat-knowledge` lane | ldui-iilm.7 | complete, reviewed (1 fix round) | a53c88c, 741d2ee |
| P7 usage, quick actions, ES route | ldui-iilm.8 | complete, reviewed (1 fix round) | 24ca8b5, 3ab28fc |
| P8 Ollama backend (Rust-DeskApp + editmark-server) | ldui-iilm.9 | complete, reviewed | Rust-DeskApp `feature/ollama-backend` @445ef36, editmark @0f742e1 (both pushed; both repos back on `master`) |
| P9 Live mode | ldui-iilm.10 | **committed and reviewed; fix round 1 PARTIAL (2 of 4 Important fixed)** | e82a9b0, c43202c |
| P10 audits, docs, dev-log, full gate | ldui-iilm.11 | not started; brief written (`task-P10-brief.md`) | — |
| P11 desktop picker (optional) | ldui-iilm.12 | not started; brief written (`task-P11-brief.md`) | — |

Suites at hand-off: `cargo xtask verify` 18/18, `test-ai-chat` 25,
`test-ai-chat-knowledge` 8, library 3519 native tests.

### P9's open findings (the one thing that is not finished)

Its review returned SPEC ✅ and four Important findings. Fix round 1 was cut
short deliberately at a priority order when the session ended, and stopped
cleanly — nothing is uncommitted. Full detail in `task-P9-report.md`.

**Fixed in `c43202c`:**

1. *(was I4)* `connected.set(true)` fired on the backend list rather than the
   session, so a Connect that failed at `open_session` unmounted the fixture.
   The page now opens the session itself via `LiveChatWorkspaceBackend::connect()`
   and promotes only on `Ready`; the composite's first `open_session` **adopts**
   that session instead of opening a second. Covered by
   `a_failed_connect_never_promotes_the_live_workspace`.
2. *(was I1)* Selecting the Fixture radio while connected left the live backend
   driving the page with Disconnect hidden and `data-ai-chat-mode="fixture"`
   published over a live session. The radio now runs the same teardown as
   Disconnect and the mount reads `drives_live(mode, connected)`, so that
   attribute can never lie. Covered by
   `switching_to_fixture_tears_a_live_session_down`.
3. The unused `Request` web-sys feature was removed.

**Still open — and the premises matter more than the findings:**

4. *(I3)* The review asked for the "zero requests on mode select" proof to be
   pointed at the real switch rather than the fixture page's inert decoy radio.
   **That is not currently possible, and the reviewer did not know it:**
   `DemoServer::start(html_target)` serves exactly one binary, and `test-ai-chat`
   serves `client-snapshot-test-host.html`, which never compiles
   `demos/ai_chat.rs` — so `/components/ai-chat` is **not reachable in that
   lane at all**. The two workable routes are a separate lane step with
   `html_target: None`, or `reactivity_smoke` with its pinned count re-pinned.
   Until one is taken, the showcase page has **no automated request oracle**.
5. *(I2)* `is_lane_driven` misses real lane-driven sources
   (`snapshot_table_page.rs`, `client_snapshot_list.rs`, `debug.rs`, and
   `demo/src/main.rs`, the router), and its positive control asserts presence,
   never completeness, so the blind spot cannot be observed. The fixer's finding:
   a computed module closure over `main.rs` is the **wrong** shape — it would
   sweep in `ai_chat.rs` and make the showcase page fail `LIVE_WIRE_MARKERS`. The
   working design is two tiers: compute the closure from
   `client_snapshot_test_host.rs`'s own `#[path]`/`mod` declarations, and
   separately forbid `/components/ai-chat` from the two audit `PAGES` arrays.

So the invariant "no automated lane can reach a live server" **holds today**, by
construction plus the unstated fact that no lane loads the showcase page — but
nothing asserts that second leg, and P10 adds `PAGES` entries for exactly that
page. P10 must close it.

**P9's end-to-end runbook was never executed** (bead ldui-9ics). It needs
`editmark-server --mode mock`, built from the siblings' `feature/ollama-backend`
branches; the verified build recipe and the peer-notification protocol are in
that bead.

## Facts learned mid-epic that bind later phases

- **postgres-agent AI Memory, observed in production** (contract file in the
  ledger directory; durable copy in Rust-DeskApp
  `docs/superpowers/specs/2026-09-15-ai-chat-desktop-web-split-design.md`):
  recall returns curated items only, so a freshly written item is invisible
  until a curator approves it; results over ~8 KB arrive as a stored envelope
  unless `inline: true`; `remember` takes `body` and refuses `lesson` and
  `principle` kinds; the meaning-lane match lands a moment after the write;
  showcase copy must never imply the extraction jobs run. P3 models the
  curation gap (`Candidate` entries are not recallable until confirmed); P6
  renders and tests it.
- **Escalation and truncation have no `StreamEvent`.** They cross the backend
  trait as `TurnRecord.notices` (ruled for P3's fix round); the composite maps
  them to transcript annotations. The fixture's inherent `annotations()` is not
  reachable through `dyn ChatWorkspaceBackend`.
- **Seed the permission mode** in the composite: `AiChat` with `None` shows the
  browser's first option while the host believes nothing is chosen (P2 finding).

## How to resume

1. **Open the ledger first** (`.superpowers/sdd/cozy-chasing-boot/progress.md`).
   It records every ruling and names which commits are reviewed. `git log` shows
   eleven green-looking commits and cannot distinguish a reviewed one from one
   mid-fix-loop.
2. Finish P9's fix round (the four findings above), scope-re-review it, and
   close ldui-iilm.10.
3. Dispatch P10 from `task-P10-brief.md`. **Its first step is to merge `main`
   into this branch** — `doc/dev-log/2026.md` exists on `main` and not here, and
   P10 appends to it. P10 also owns the `cargo doc` gate (ldui-t2zq) and must add
   `/components/ai-chat` and `/components/ai-chat-es` to BOTH audit suites'
   `PAGES` *and* their per-page `audit_test!` invocations — an entry alone sweeps
   nothing.
4. One writer in this worktree at a time; reviews are read-only and may run in
   parallel. Use PowerShell for git here (Bash git is refused by the isolation
   guard). Run `bd` only from `C:\dev\leptos-daisyui-rs`. Commit by path.
   Never `cargo fmt` or commit while a verify run or browser lane is reading the
   tree — it rewrites `demo/.ldui-css-stamp` and fails the suite on the
   `ldui-hun` guard.
5. Merging to `main` is the owner's call (finishing-a-development-branch), after
   P10's full gate and a whole-branch final review.

## What the browser lanes found (why they are worth their runtime)

Every lane in this epic failed on its first real execution, and each failure was
a genuine defect no native test could see. The recurring shape is **the UI
stating something the data does not support** — which a native test cannot
catch, because it asserts on the data and agrees with itself.

- A completed turn was **structurally unobservable**: the composite resolved each
  tick's record from `current_turn_id()`, which by contract excludes a turn the
  moment it finishes, so the header would have read "Validating" forever and the
  whole honesty surface stayed dark on healthy turns.
- A host-pinned engine offered all seven catalogue models.
- A thinking-capable engine streamed a transcript the session had already
  stripped.
- `DropdownContent` renders a `<ul>`, so **every settings popover in this
  library** is an invalid list (axe `list`, blocking) — fixed in the two AI-chat
  popovers, remaining sites filed as ldui-afan.
- An assistant-only turn cited sources, rendering facts whose caveat claims "the
  memo itself is the only source" — filed as ldui-zlgl, because the guarantee is
  currently true only of the fixture.
- A byte-exact copy comparison could never match, because the transcript renders
  through a smart-punctuation pass (ldui-zlu0).
- A phase ladder was unobservable: a 500 ms blind settle sat inside the 900 ms
  window being measured.

## Cloud agent progress (2026-09-16)

- Closed on main / this branch: `ldui-afan`, `ldui-t2zq`, `ldui-zlgl`, `ldui-zlu0`, `ldui-iilm.10` (I2 fixed; I3 → `ldui-4bi8`), `ldui-4klx` (web-sys features).
- Blocked on private sibling checkouts (`ldui-3uzu`): `ldui-iilm.11`, `ldui-iilm.12`, `ldui-9ics`, `ldui-ikvx`, `ldui-qob6`.
- Toolchain pinned via `rust-toolchain.toml` (1.98.1 + wasm32).
