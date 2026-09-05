# Softphone UI composition

User-approved scope: a UI-library component with documented usage and tests.
The earlier provider/CallSession recommendation is background research, not
implementation scope. No SDK, media session, persistence or telephony backend
belongs in this change.

`Softphone` renders one caller-owned `Signal<SoftphoneState>` and emits
`SoftphoneCommand` values through `on_command`. Each command carries the current
opaque `context_id`, allowing the host to reject stale asynchronous work.
The component never claims that an emitted command has succeeded.

The state contains client name/subtitle and stable phone choices, selection,
call phase, clock specification, confirmed mute/record/transcription state,
supported capabilities, pending action and an optional error. A single phone
is text; multiple phones use a named native Select. Choice is locked throughout
an ongoing call. Empty, unknown or duplicate identities cannot initiate a call.

The primary surface is a contained, theme-aware call console. Client identity
leads; the clock uses tabular numerals; labeled action tiles have stable order;
Call or End call anchors the bottom. Semantic daisyUI base/primary/success/error
colors, existing typography and framework focus states govern appearance.
Two columns keep controls usable in a narrow panel; text wraps instead of
truncating important action names. No decorative animation or status-only color.

Actions are Call, End call, Mute/Unmute, Hold/Resume, Route to voicemail,
Record/Stop recording, Transcribe/Stop transcription and a local keypad.
Capabilities omit unsupported controls. Pending actions disable competing
requests but preserve End call. Current phase guards are enforced in handlers
as well as markup. Voicemail is an opaque request; the host defines its routing
semantics. Recording and transcription stay unchanged until host confirmation.

The timer is NotStarted, Running from a caller-provided epoch timestamp, or
Stopped at a caller-provided duration. Running includes hold/reconnect time;
the host supplies Stopped to freeze it. Clock input is injectable for tests;
the default browser clock ticks once per second and cleans up on disposal.
No microphone permission or browser media API is invoked by this UI.

Usage docs define the host's command acknowledgment model, capabilities,
localization, timer semantics, accessibility and distinction between UI evidence
and actual telephony evidence. The showcase uses simulated state transitions,
explicitly labeled, with pending/accept/reject and deterministic clock controls.

Verification: pure native guard/timer tests; a release browser lane for number
selection, typed callback readback, pending/rejected controls, clocks, keypad,
responsive layout and accessible semantics; reviewed screenshots plus existing
style/layout audits. Negative controls must demonstrate that core oracles fail.

Tracking: ldui-xmhn in Beads.
