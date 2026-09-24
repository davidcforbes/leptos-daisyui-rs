# AiChatWorkspace

`AiChatWorkspace` is the opinionated AI-chat composite: an engine header, the
knowledge rail, the generic [`AiChat`](../../src/components/ai_chat/) panel and
the evidence rail, all driven by one host-implemented backend. It is shaped
like [`Helpdesk`](./helpdesk.md) — the pattern owns the arrangement and the
honesty rules, the host owns the transport.

It owns no transport and no model. Everything it needs comes through a
host-implemented [`ChatWorkspaceBackend`](#the-backend-trait), so the same
composite backs a real `editmark-server` deployment, a scripted in-memory
fixture, and a host whose engine is not configured at all.

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{AiChatWorkspace, ChatWorkspaceBackend};
use std::rc::Rc;

let backend: Rc<dyn ChatWorkspaceBackend> = Rc::new(ServerChatWorkspace::new(api));
view! {
    <AiChatWorkspace
        backend=backend
        initial_engine=Some("claude".to_string())
    />
}
```

## What the composite decides, and what it refuses to

The workspace's job is to say only what the turn actually supports. Three
rules are structural rather than stylistic, and each has a DOM hook so a test
can read the decision rather than the prose:

- **A grounding verdict gates the evidence rail.** Citations and facts render
  because the verdict says the turn was grounded, not because the backend
  happened to return some. An ungrounded turn shows the withheld-answer copy
  (`grounded_not_found`), which is the visible proof the workspace declined to
  guess.
- **Every fact is qualified.** A bare assertion with no qualification is a
  claim the composite is not entitled to make; `data-ai-chat-fact-qualification`
  is what a sweep asserts on.
- **Availability is a reason, not a boolean.** A disabled engine says *why*
  (`data-ai-chat-engine-reason`), because the action that fixes it — a
  credential, a plan, a sign-in — lives in the host application.

## The backend trait

`ChatWorkspaceBackend` is the whole seam. It returns
`WorkspaceFuture`s and reports failure as a typed `ChatWorkspaceError` with a
`ChatWorkspaceErrorKind`, so the composite renders a named failure rather than
a `Debug` string. `WorkspaceRefusal` is the separate, non-error path: a refusal
is a decision the workspace made on purpose (a guardrail hit, an ungrounded
question), and it carries a `RefusalNextAction` the host can act on through
`on_refusal_action`.

### Not configured is a first-class state

A host that has never configured an engine is not an error. The provider card
reports `card_unready_reason`, the header renders the reason, and the composer
stays inert — no spinner, no retry loop, no failure toast.

## The in-memory backend (test-mode only)

`InMemoryChatWorkspaceBackend`, `ScriptedChatTransport`, `TurnScript`,
`ChatWorkspaceFault`, `FixtureClock` and the `SEED_*` constants are behind the
`test-mode` feature. They drive the showcase page and every browser lane, and
they are deliberately absent from a production build: a real surface
implements `ChatWorkspaceBackend` over its own transport.

`BackendCall` is the fixture's ordered call log, published by the fixture
documents so a lane can assert what the composite *asked for*, not merely what
it rendered.

## Live mode (showcase page only)

The showcase page carries a Fixture/Live switch. **Choosing Live connects to
nothing.** It only says where a connection would go: the fixture keeps driving
the page until Connect is pressed, and a failed Connect leaves it driving too,
so the page is never in a half-real state that reads as a product bug.

Live mode belongs to the demo, not to the pattern: the workspace itself has no
URL and no `SseBridgeTransport`. A unit sweep enforces that separation by
scanning the demo sources — a lane-driven fixture document that so much as
mentions the live module fails, because a browser lane must never be one
button-press away from a real server.

| Hook | Where |
|---|---|
| `data-ai-chat-mode` | `fixture` \| `live`; the Backend switch. |
| `data-ai-chat-live-mode-reason` | The "Live connects to nothing" note. |
| `data-ai-chat-live-base-input`, `-live-connect`, `-live-disconnect` | The base-URL field and the two buttons. |
| `data-ai-chat-live-status`, `-live-base`, `-live-session` | Connection state, target and session id. |
| `data-ai-chat-live-notice` | The last connection notice, when there is one. |
| `data-ai-chat-identifier` | The proof rows' record ids (knowledge entries, turns). A declared mono ROLE in the style audit's `demo_profile`, the `EntityColumn::identifier()` precedent: the family check is exempt, the size check still runs. |

## Layout: Workbench or Rail (ldui-d5ss)

`layout: AiChatWorkspaceLayout` is chosen once, at mount; switching it would
remount the conversation.

- **`Workbench`** (the default) is the three-column body: knowledge rail,
  conversation, evidence rail side by side from the `lg` breakpoint. That is a
  VIEWPORT breakpoint, so it is only right when the workspace has the page's
  width. In a 375 px rail on a 1624 px window it still fires, and the grid
  resolves to 256 px | 1.6 px | 256 px -- the 4iiz-Office production defect
  this layout choice exists to fix.
- **`Rail`** is one column at the HOST's width: the conversation first, then
  the turn's evidence and the knowledge/memory controls behind closed
  disclosures (`<details>`, labelled by `rail_evidence_disclosure` and
  `rail_knowledge_disclosure`). Use it for side rails, drawers and any container
  narrower than about 64rem.

`show_knowledge_rail` and `show_evidence_rail` (both `Signal<bool>`, default
`true`) omit a rail entirely, in either layout. Use them for a host with no
corpus or memory store, rather than showing controls that cannot do anything. In
Workbench an omitted rail also drops its grid track, so the conversation always
owns `1fr` (`workbench_grid_class`).

```rust,ignore
<AiChatWorkspace
    backend=backend
    layout=AiChatWorkspaceLayout::Rail
    show_knowledge_rail=false
    show_evidence_rail=false
/>
```

## Copy and localization

All rendered text comes from `AiChatWorkspaceTexts`, supplied as a `Signal` so
a locale change re-renders without a remount; the inner panel takes
`AiChatTexts` the same way. Nothing formats a user-visible string from an
enum's `Debug`. EN and ES tables ship complete, and unit tests assert that
every field is translated, carries real Spanish diacritics, and claims no
capability the actor does not have.

One rule binds copy to the renderer: **copy a browser suite pins byte-exactly
against rendered markdown may carry no character the markdown pipeline
rewrites.** `editmark-core` parses with `ENABLE_SMART_PUNCTUATION`, which
rewrites quotes, apostrophes, `...`, `--` and `---`; markdown's own inline
markup and block prefixes change the rendered text just as completely. The
pinned set is swept from the browser-suite sources rather than listed by hand
(`every_byte_pinned_field_survives_markdown_rendering`), so a newly pinned
field cannot be silently missed.

## DOM hooks

- `data-ai-chat-workspace-layout` on the root: `workbench` or `rail`.
- `data-ai-chat-workspace-body` on the body: the same two values.
- `data-ai-chat-workspace-conversation` on the conversation panel.
- `data-ai-chat-rail-disclosure` on each Rail disclosure: `evidence` or
  `knowledge`.
- `data-ai-chat-composer-label` on the composer's `sr-only` label, whose text
  is the substituted placeholder (ldui-iay0), never the raw template.

Stable `data-*` attributes, for tests and for host CSS. **Query by these, never
by document position** — a positional selector does not fail when the layout
changes, it silently starts describing something else. This table is the test
contract: a lane asserts against these names, so renaming one is a breaking
change.

| Hook | Where |
|---|---|
| `data-ai-chat-workspace`, `-workspace-engine`, `-workspace-locale` | Root; the selected engine and locale are readable from the DOM. |
| `data-ai-chat-workspace-header`, `data-ai-chat-engine-label` | The engine header and its label. |
| `data-ai-chat-engine-availability`, `-engine-reason`, `-engine-reason-for` | Availability verdict and the reason behind it. |
| `data-ai-chat-provider-rows`, `data-ai-chat-rail-label` | Provider settings rows and their control labels. |
| `data-ai-chat-effort-select`, `-temperature`, `-codex-lever`, `-no-mcp` | Per-provider tuning controls. |
| `data-ai-chat-credential-note` | The host-held-key note. |
| `data-ai-chat-turn-status`, `-turn-status-label`, `-turn-receipt` | Turn lifecycle, its label and the receipt id. |
| `data-ai-chat-outcome`, `-failure`, `-truncated`, `-canceled-partial`, `-cancel-discarded` | Terminal outcomes, including the two partial-result cases. |
| `data-ai-chat-honesty`, `-honesty-reason`, `-honesty-text` | The honesty verdict a turn published. |
| `data-ai-chat-limitations`, `-limitation`, `-escalation` | Declared limitations and the escalation line. |
| `data-ai-chat-usage`, `-usage-plan`, `-usage-metered`, `-usage-budget`, `-usage-cost`, `-usage-output`, `-usage-reasoning`, `-usage-tps` | Usage and cost figures. |
| `data-ai-chat-knowledge-rail`, `-corpus`, `-corpus-list`, `-corpus-scope` | Knowledge rail and corpus selection. |
| `data-ai-chat-ingest-phase`, `-ingest-error`, `-reindex` | Ingest lifecycle. |
| `data-ai-chat-query-mode`, `-query-attribution`, `-posture`, `-grounding` | Query mode, posture and the grounding verdict. |
| `data-ai-chat-kb-list`, `-kb-entry`, `-kb-kind`, `-kb-state`, `-kb-scope`, `-kb-confirm`, `-kb-withdraw`, `-kb-awaiting-curation` | Personal knowledge entries and their curation controls. |
| `data-ai-chat-kb-office-list`, `-kb-office-entry` | Office (shared) knowledge entries. |
| `data-ai-chat-memory-enabled`, `-capture-enabled`, `-memory-offline` | Memory toggles and the offline state. |
| `data-ai-chat-remember`, `-remember-input`, `-remember-kind` | The remember-this control. |
| `data-ai-chat-recall`, `-recall-input`, `-recall-no-hits` | Recall search and its empty state. |
| `data-ai-chat-memory-hit`, `-memory-rank`, `-memory-score`, `-memory-receipt`, `-memory-search`, `-memory-attribution` | One recall hit: rank, score and where it came from. |
| `data-ai-chat-guardrail`, `-refusal`, `-refusal-reason`, `-refusal-action` | Guardrail refusals and the next action offered. |
| `data-ai-chat-evidence-rail`, `-evidence-as-of`, `-citation`, `-fact`, `-fact-qualification` | Evidence rail; every fact carries its qualification. |
| `data-ai-chat-quick-actions`, `-quick-action`, `-quick-action-language` | The quick-action bar. |
| `data-ai-chat-notice` | A host-supplied transcript notice. |

The inner panel's own hooks are the `AiChat` contract and apply here too:
`data-ai-chat-state` on the root, `data-chat-role` and `data-chat-index` on
each row, and `data-chat-tool-phase` (`call` \| `result`) on the two halves of
a tool turn — the last is why a consumer can name the tool RESULT without
counting rows. It is absent, not empty, on a message that carries no phase.

## CSS

Class delivery is not automatic across a Rust path dependency. A consuming
app's `input.css` must scan this crate's source and import the generated
tokens:

```css
@import "tailwindcss";
@import "../leptos-daisyui-rs/styles/tokens.css";
@plugin "daisyui";
@source "../src/**/*.rs";
@source "../leptos-daisyui-rs/src/**/*.rs";
@source inline("chat chat-start chat-end chat-bubble chat-bubble-primary chat-bubble-info chat-bubble-ghost chat-bubble-neutral chat-bubble-warning chat-image chat-header avatar avatar-placeholder");
```

The animation and focus classes the panel emits (`ld-aichat-msg-in`,
`ld-eased`, `ld-focus-ring`) come from `UiAnimationsPreamble` /
`UiTokensPreamble`, not from `styles/tokens.css`. A consumer that never mounts
them gets no error — the classes are simply undefined and the effect silently
does nothing.

## Proof

`cargo xtask test-ai-chat` drives the showcase document over the seeded
fixture: engine selection, the settings rows each provider actually supports,
turn lifecycle and its terminal outcomes, the two partial-result cases, and
the transcript's role hooks. `cargo xtask test-ai-chat-knowledge` drives the
knowledge fixture: corpus scope and query mode, ingest phases, the grounded
and not-found postures with their citations, recall ranking and receipts, and
the guardrail refusals.

Both locale routes are also swept by the two visual-quality audits —
`cargo xtask test-layout` (overlap hard-failure, grid and internal-vs-external
ratchets) and `cargo xtask test-style` (typography, shape, depth and daisyUI
component drift) — at `/components/ai-chat` and `/components/ai-chat-es`. The
ES route is swept as its own page rather than assumed to match EN: localized
copy changes line counts, and that is where a layout regression hides.

> **Browser coverage that only COMPILES proves nothing.** Every suite in this
> area failed on its FIRST actual execution, and each failure was a real
> defect rather than a fixture problem. Run the lane before believing the
> coverage.
