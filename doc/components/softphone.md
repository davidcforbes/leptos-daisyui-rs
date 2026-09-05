# Softphone

`Softphone` is LDUI's opinionated client calling component: identity, number selection, elapsed time, call controls and feedback on one bounded surface. It is a controlled UI composition, like EntityTable. The application supplies state and receives typed requests. It does not load a telephony SDK, request microphone access, place calls, capture audio, persist notes or choose a provider.

The interactive showcase is `/components/softphone`. Its [complete simulated host](../../demo/src/demos/softphone.rs) accepts or rejects commands without making calls. [Research](../research/2026-09-04-client-call-workspace.md) provides product context; this document defines the implemented library contract.

## Usage

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
fn ClientPhone() -> impl IntoView {
    let state = RwSignal::new(SoftphoneState {
        context_id: "client-42/interaction-1".into(),
        client: SoftphoneClient {
            name: "Elena Martinez".into(),
            subtitle: "Client · Account review".into(),
            phones: vec![SoftphoneNumber {
                id: "mobile".into(),
                label: "Mobile".into(),
                number: "+1 (415) 555-0142".into(),
            }],
        },
        capabilities: SoftphoneCapabilities {
            hold: true,
            voicemail: true,
            recording: true,
            transcription: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let on_command = Callback::new(move |command: SoftphoneCommand| {
        if command.context_id != state.get_untracked().context_id {
            return;
        }
        // This minimal example declines calls. Replace with your host dispatcher.
        match command.action {
            SoftphoneAction::SelectNumber(id) => {
                state.update(|s| s.selected_phone_id = Some(id));
            }
            _ => state.update(|s| {
                s.error = Some("Connect the application's calling service first.".into());
            }),
        }
    });
    view! { <Softphone id="client-phone" state=state on_command=on_command /> }
}
```

Import through `components::*` or `components::softphone::*`. Include library Rust sources in the application's Tailwind source scan, as for other LDUI components. The component uses daisyUI 5 semantic colors and existing Button, Select and Persona components, with `text-2xl` tabular duration typography.

| Prop | Contract |
| --- | --- |
| `id` | Required unique DOM identity; also prefixes the number and keypad IDs. |
| `state` | Required `Signal<SoftphoneState>`; an `RwSignal` converts directly. Replace related fields atomically. |
| `on_command` | Required `Callback<SoftphoneCommand>`. Requests carry the current `context_id` and typed action. |
| `texts` | Optional reactive `Signal<SoftphoneTexts>`; English defaults. All component labels and status text are configurable. |
| `now_ms` | Optional `Signal<i64>` containing epoch milliseconds. Default browser clock refreshes once per second and cleans up on unmount. Native rendering defaults to zero; supply a clock for deterministic native output. |
| `class` | Optional static class string appended to the outer surface. |

## State and acknowledgment model

The host owns the client, selected number, lifecycle phase, timer, supported capabilities, confirmed toggle values, pending action and error. The component owns only keypad visibility. Opening the keypad does not dispatch a telephone digit. Changing context, leaving Active or removing keypad capability closes it.

For an asynchronous operation, the host should synchronously set `pending = Some(action.kind())` before returning from the callback. On success, update confirmed fields and clear pending/error together. On rejection, preserve confirmed fields, clear pending and provide a useful localized error. Until acknowledgment, Record does not become Stop recording and no recording badge appears. The same rule applies to mute, hold and transcription.

Use a fresh `context_id` for every interaction, including a second call to the same client. Check it before processing a request and again before applying an asynchronous response. The callback payload supplies context, not a unique request token: the host must correlate its own operation IDs, particularly when End call supersedes another pending operation. Discard late acknowledgments after an interaction ends or is replaced. The UI guard is not authorization or an asynchronous command queue.

| Action | Eligible state and result request |
| --- | --- |
| Select number | Ready or Ended, valid unique nonblank ID and display number, no pending request. Emits `SelectNumber(id)`. |
| Call | Ready or Ended, valid selected number, no pending request. Emits `Call { phone_id }`; host resolves the destination. |
| End call | Any live phase. Remains available during another pending action; blocked while EndCall itself is pending. |
| Mute / Unmute | Active or Held with mute capability; emits the desired boolean. |
| Hold / Resume | Active to Held or Held to Active with hold capability; emits the desired boolean. |
| Route to voicemail | Ringing, Active or Held with voicemail capability. The host defines the destination and routing semantics. |
| Record / Stop recording | Active or Held with recording capability; emits desired recording state. |
| Transcribe / Stop transcribing | Active or Held with transcription capability; emits desired transcription state. |
| Keypad | Active with keypad capability. Buttons emit only `0123456789*#`. No global keyboard interception. |

Every command requires a nonblank context ID. Except End call, all commands are blocked while any request is pending. Unsupported optional actions are hidden; supported but currently unavailable actions are disabled. The callback guard repeats eligibility checks so synthetic events cannot bypass native disabled controls. Mute and keypad capabilities default to true; the other capabilities default to false. Enable only operations your host supports.

`Ringing` describes lifecycle state; this version is an outgoing client call console and does not implement an incoming-call accept/reject workflow. Routing to voicemail does not imply that arbitrary providers support that operation. The host controls consent, authorization, errors and which controls it exposes.

## Number selection and duration

One valid number displays directly and is selected implicitly when `selected_phone_id` is absent. Multiple numbers use a native, keyboard-accessible dropdown showing label and formatted number. With multiple numbers and no valid selection, the placeholder remains visible and Call is disabled. An empty list displays the localized no-number message. Unknown, duplicated selected IDs and blank numbers cannot be called. The host owns number formatting and normalization. Selection is locked for a live interaction; keep its identity and destination stable until the call ends.

`SoftphoneTimer::NotStarted` displays `--:--` by default. On connection, set `Running { connected_at_ms }`. The display derives elapsed seconds from the supplied clock, so throttled timer callbacks do not accumulate tick drift. It includes time on hold and reconnecting. It is wall-clock elapsed time, not billable duration or provider-confirmed talk time; a system clock adjustment can change it. Future timestamps clamp to zero.

On completion, the host explicitly supplies `Stopped { seconds }`; changing phase alone does not freeze a Running timer. Formatting is `MM:SS` below one hour and `H:MM:SS` thereafter. For tests, pass a controlled clock and advance it directly. For example, connection at 1,000,000 ms and now at 1,065,000 ms renders `01:05`.

## Visual and accessibility model

The console is full-width up to `max-w-md`, with a client header, contrasting duration band, two-column action grid and full-width primary call/end action. It fits an application sidebar, drawer or detail page. Keep it mounted above route changes if the application needs a persistent call surface. Client text wraps within the available width. Semantic colors respond to the host theme; labels accompany icons, and recording/transcription badges describe confirmed state.

The dropdown has an accessible label. Toggle buttons expose `aria-pressed`, the keypad exposes expanded state and a named group, and each digit has a localized accessible label. Status and pending feedback are polite live regions; errors use an alert. The timer has an accessible name and `aria-live="off"` to avoid announcing every second. Normal Tab, Enter and Space behavior remains available, and typing notes elsewhere does not send DTMF.

Supply translated `SoftphoneTexts` reactively. The digit label is a template containing `{digit}`. Client data and host error messages are already display text and must be localized by the host. The showcase's Change labels control intentionally demonstrates partial translation, not a complete French locale.

## Testing model

Run `cargo test -p leptos-daisyui-rs --lib softphone` for pure state/eligibility and duration boundary tests. Run `cargo xtask test-softphone` for the self-hosted release browser suite. The lane is also included in `cargo xtask verify-full`.

The [browser tests](../../tests/softphone_smoke.rs) interact with the real dropdown and buttons and read separate simulated host receipts. They exercise pending and rejected requests, duplicate guards, confirmed toggles, hold/resume, voicemail, elapsed/frozen duration, no-number and single-number states, capability omissions, label updates, keypad isolation, long text and compact layout. Scoped axe checks cover the console. Screenshots are written to `target/softphone-active.png` and `target/softphone-compact.png` for visual inspection.

In consuming applications, test the host's context and operation correlation, response rejection, end-call supersession and state projection independently. Then test the component with deterministic acknowledgments and clock updates. Actual audio, provider routing, microphone permissions and call reliability belong to the consuming application's integration tests; this library's simulation does not verify those behaviors.

## Maintenance guidance

Keep the number selector mounted while the call projection changes; updating a confirmed toggle should not recreate the selector and discard focus. Selection handlers re-read the current state and restore the accepted value when a request is declined. Native disabled markup and command eligibility are separate protections, and the browser suite exercises both.

Keypad cleanup depends on context identity, phase and capability. Read those reactive dependencies on every effect run before combining conditions. A short-circuit expression whose context-change branch returns true can skip the phase/capability reads, leaving the effect unsubscribed from later changes within that same context. Preserve the open-keypad → Hold and open-keypad → capability-removal checks when refactoring.

Verify typography on the rendered element. A plausible class name can supply no CSS rule while its inherited font size still passes a generic style audit. Softphone's timer uses `text-2xl`; its browser test checks a computed size of 24px in the default showcase environment. See [unapplied typography class](../visual-quality/unapplied-typography-class.md). Check actual text bounds for long client names as well as the panel's scroll width, because clipping can conceal overflow.

Finish source changes before starting the combined release gate. The Trunk watcher can queue further builds while the current compile/optimization is still running; those intermediate artifacts are not final evidence. Use focused checks during implementation, then one final `cargo xtask verify-full` and review its refreshed screenshots.

## Verified implementation snapshot

The approved implementation is commit `f730e66` (Bead `ldui-xmhn`, 4 September 2026, America/Los_Angeles). Its verification record is:

- `cargo test -p leptos-daisyui-rs --lib softphone`: 9 tests passed. A copied-source negative control made the pending guard accept competing actions; the intended test failed, then all 9 passed after restoration.
- `cargo xtask verify-full`: 40/40 steps passed, including 3,251 library tests and both Softphone browser tests. The pre-push native gate also passed 16/16.
- Refreshed 1280px and 375px screenshots were inspected after the timer typography correction. The browser suite verified computed timer size, long-text bounds, keyboard selection, confirmed actions and scoped accessibility.

These are historical results for that commit. Re-run the relevant commands after changes; use the runner's current summary for counts. Screenshot files under `target/` are regenerated local artifacts, not committed baselines.
