# AI chat showcase epic (ldui-iilm) — branch status and resume guide

Last updated: 2026-09-16 (session hand-off). Branch: `feature/ai-chat-showcase`,
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

| Phase | Bead | State | Commits |
|---|---|---|---|
| P0 prereqs + orphan contract gate | ldui-iilm.1 | complete, reviewed | 1af0472 |
| P1 pure model + texts (`patterns::ai_chat_workspace`) | ldui-iilm.2 | complete, reviewed (1 fix round) | 0bd77bc, cfbaee0, 7f736c2 |
| P2 generic `AiChat` upgrades | ldui-iilm.3 | complete, reviewed (1 fix round, re-review clean) | e5ed0f2, 7000c6a |
| P3 shipped fixture (`memory.rs`, test-mode) | ldui-iilm.4 | committed; review not approved; **fix round 1 brief ready, not dispatched** | b6db0a3 |
| P4 composite + first page + `test-ai-chat` lane | ldui-iilm.5 | brief drafted (`task-P4-brief.md`), reconcile against P2/P3 reports first | — |
| P5 honesty + lifecycle (+12 tests) | ldui-iilm.6 | brief drafted | — |
| P6 knowledge (+ `test-ai-chat-knowledge` lane) | ldui-iilm.7 | brief drafted, carries the postgres-agent real-shape rulings | — |
| P7 usage, quick actions, ES route | ldui-iilm.8 | brief drafted | — |
| P8 Ollama backend (Rust-DeskApp + editmark-server) | ldui-iilm.9 | complete, reviewed | Rust-DeskApp `feature/ollama-backend` @445ef36, editmark `feature/ollama-backend` @0f742e1 (both pushed; both repos checked back to `master`) |
| P9 Live mode | ldui-iilm.10 | not started | — |
| P10 audits, docs, full gate | ldui-iilm.11 | not started | — |
| P11 desktop picker (optional) | ldui-iilm.12 | not started | — |

Follow-ups filed: ldui-t2zq (cargo doc gate + 18 warnings), ldui-qob6 (Live-mode
AI Memory bridge over MCP stdio).

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

1. Open the ledger; the last lines name the running or held fix round.
2. `bd ready` from the worktree (it redirects to the main checkout's tracker).
3. Dispatch P3's fix round (`task-P3-fix1-brief.md`), re-review it, close
   ldui-iilm.4, then reconcile and dispatch P4. One writer per worktree at a time. Use PowerShell for git in
   the worktree; commit by path; never `cargo fmt` or commit while a verify
   run or browser lane is reading the tree.
4. Before compiling editmark-server for Live mode (P9), re-checkout
   `feature/ollama-backend` in Rust-DeskApp and editmark and tell the
   4iiz-etl-cd session first (its deploy refuses unpublished branches).
5. Merging to `main` is the owner's call (finishing-a-development-branch),
   after P10's full gate.
