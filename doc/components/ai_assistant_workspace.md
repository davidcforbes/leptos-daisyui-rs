# AiAssistantWorkspace

`AiAssistantWorkspace` is a provider-independent controlled UI composition. It
displays host-owned projections and proposes typed interactions; it does not
grant access, assemble business facts, call a provider, persist records, or
perform a business write.

## Boundary

The consuming application owns authorization, evidence assembly, transport,
persistence, provider execution, and host acknowledgments. The library owns
presentation and fail-closed validation. Unknown or malformed host vocabulary
must remain a contract error and must not become an enabled action.

The public model is exported from
`leptos_daisyui_rs::components::ai_assistant_workspace`:

- `AssistantWorkspaceState` contains the accepted page context, independently
  loaded conversation/settings/memory/learning projections, and the host's next
  request allocation.
- `AssistantContext` and `AssistantContextStamp` carry actor, population, data,
  permission, policy, revision, and replacement-epoch identities.
- `AssistantConversation` owns one canonical controlled `AssistantDraft` and
  durable `AssistantAttempt` records. Refusals, uncertainty, failure, and
  cancellation retain the draft and are not transcript answers.
- `AssistantFact`, `AssistantAnswer`, `AssistantArtifact`, and
  `AssistantProposal` keep qualifications, availability, provenance, evidence,
  scope, and insertion policy beside the displayed claim.
- `AssistantSettings` separates accepted preferences from proposed edits.
  `AssistantConnection` distinguishes connection states and sign-in shapes;
  pending device flow carries only a verification URL, user code, and host
  expiry.

## Guarded Ask flow

Create an explicit `AssistantAction` from the current accepted stamp and target,
then ask the library to construct a command:

```rust,no_run
use leptos_daisyui_rs::components::ai_assistant_workspace::*;

let action = AssistantAction {
    context: context.stamp(),
    target: AssistantIntentTarget::Conversation {
        conversation: AssistantItemRevision { id: conversation.id.clone(), revision: conversation.revision },
        draft: AssistantItemRevision { id: draft.id.clone(), revision: draft.revision },
    },
    kind: AssistantActionKind::Ask,
};

if let Some(command) = command_for(&state, action) {
    // Publish the host-owned pending request and await its acknowledgment.
    host_dispatch(command);
}
```

`can_dispatch` and `command_for` require granted page and conversation Ask
capabilities; an unchanged context stamp; matching conversation and draft
identities/revisions; nonblank text within the host's Rust-scalar limit; an idle
submission; an allocated request ID; accepted settings owned by the current
actor; and an enabled, authorized, signed-in (or not-applicable) selected engine.
Missing or uncertain allocation disables the command. Constructing a command
does not mutate state or clear the draft.

Use `validate_workspace` at the host boundary before rendering accepted
projections. Optional Memory and Learning errors remain local to their sections
and do not disable an otherwise authorized Ask.

## Verification

Run from the repository root:

```text
cargo test -p leptos-daisyui-rs --test ai_assistant_workspace_contract
cargo clippy -p leptos-daisyui-rs --all-targets --features test-mode -- -D warnings
cargo xtask verify
cargo xtask verify-full
```

The full qualification uses release Wasm and the registered browser suites. Use
the actual xtask summary as the authority; the 2026-09-10 qualification for
commit `bc27573` recorded 16/16 native steps and 41/41 full steps. The source
handoff to `4iiz-office` is recorded in Beads; this library repository contains
no Office mutations.
