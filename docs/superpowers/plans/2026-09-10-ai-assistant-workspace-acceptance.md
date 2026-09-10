# AiAssistantWorkspace acceptance mapping

These are proof obligations, not reported results or a progress checklist.
The synthetic host in `demo/src/demos/ai_assistant_workspace.rs` owns state and
an independent command/receipt journal. Browser tests live in
`tests/ai_assistant_workspace_smoke.rs`; pure counterparts live under the
component's `tests` module. All provider calls are fake, counted and bounded.

| Named browser test / fixture states | A/B/C/D evidence and negative controls |
|---|---|
| `composer_requires_explicit_intent`: ready, suggestion, blank, over-limit, preparation-only | Suggestion fills controlled draft with zero Ask; mount/tab/resize zero Ask; Enter exactly one request, Shift+Enter newline, IME no Ask; Rust chars not bytes/UTF-16; denied capability means no composer |
| `admission_reserves_and_retains_revision`: submitting, refused, uncertain, admitted, newer-edit | Exact request/context/draft revision; synchronous host reservation prevents double-click; no clear before correlated admission; refusal/uncertainty retains; stale admission does not clear newer edit; no reservable ID disables async action |
| `refusal_actions_are_distinct`: three refusal reasons, unknown reason | Distinct retry-later/settings/new-conversation action; unknown localized contract error and retained draft; New Conversation does not Ask, erase history or forget memory |
| `recovery_never_reasks`: disconnect, gap, duplicate conflict, stream error, unreadable journal | Durable lifecycle stays separate; same request/attempt/sequence in recovery; no duplicate Ask. Inject an extra Ask into fake-host journal, assert detector fails, remove it and assert clean |
| `terminal_truth_and_cancellation`: running, cancel-requested, canceled-in-time, canceled-discarded, interrupted, failed, missing answer, conflicting late terminal | Cancel request not terminal; late completion never re-enables output; both canceled wordings and Activity records; failed attempt has no answer/message/copy/feedback; interrupted mount never restarts |
| `facts_and_declines_preserve_qualification`: zero, unavailable, baseline, answered, declined | Every fact value/qualification/availability/basis stays adjacent in DOM and screenshots; zero is not unavailable; declined body is limitations, only flagged/discarded feedback; no capture/share/insert |
| `scope_and_actor_boundaries`: page population, expanded authorized scope, viewed-subject change, actor change, stale data/policy, scope mismatch | Explicit page versus expanded labels; prompt wording cannot alter command scope; historical accepted scope retained but actions locked; memory owner stays actor; wrong answered_scope withholds answer/proposal; memory withdrawal before release withholds derived output |
| `proposals_require_current_review`: preview, expired, edited template, stale source/destination, empty-only, unchanged-suggestion, human-edited | Review exact source/target/revisions/expiry; only current reviewed proposal dispatches; Insert not Send; newer destination text survives; uncertain write blocks repeats; stale receipt never says Inserted. Inject stale dispatch and preview Insert into command journal, catch separately, restore clean |
| `call_preparation_preserves_rich_content`: ready, preparing, failed, missing facts, unconfirmed language, edited script | Ordered intent/fact/say/avoid/missing-detail, hazards, close options, rehearsal/worked example, SMS/email suggestion, rubric/prompt/freshness provenance; Save/Reset/Regenerate independent exact receipts; preparation failure does not disable unrelated manual host work |
| `settings_are_proposals_until_read_back`: edit, pending, write-echo, read-back, refused, unknown, stale receipt | Accepted and proposed engine/use/capture are distinct; write echo does not accept; failure retains both; use does not enable capture; header selected preference differs from actual per-answer provenance/cache reuse |
| `connection_matrix_and_tier_precedence`: all 8×3 combinations plus unknown/missing/expired device payload | EN/ES labeled states and device code/expiry; tier denial outranks signed-in engine; disabled CLI reason preserved; explicit opaque host OpenVerification/operator/paste handoffs; no token/password input, direct navigation or provider launch |
| `memory_ownership_and_retention_receipts`: unavailable, malformed, denied, candidate, confirmed, withdrawn, hold | Optional error does not block Ask; denied/wrong-actor bodies absent; separate create/confirm/correct/forget/export grants; pending/refused/confirmed operations; withdrawal not purge/erased-everywhere; sign-out worker cleanup separate |
| `knowledge_review_never_optimistically_publishes`: draft, pending, published, stale, metadata-only, withdrawn foundation | Title/question/EN-ES aliases/audience/source/lineage/current comparison in review; independent curator grants; no review body means no approve; pending not published. Inject published state without confirmed receipt in fake-host oracle, catch and restore |
| `history_loading_and_usage_are_truthful`: unavailable, genuinely empty, retained, restricted, unknown tokens, zero tokens, each charge variant, canceled-discarded | LoadHistory never Ask or reuse old facts as current; denied body absent; exact scope/time/actual provenance; absent usage not zero; API-equivalent never added to subscription spending; no invented budget. Inject absent-as-zero view and detect/revert |
| `learning_partial_is_not_all_clear`: complete, partial, missed, unconfigured, unassessable, capacity-paused | Interval/cutoff/timezone/late catch-up/missing partitions, each stage and attributed assessment, oldest/owner/recovery, full improvement/disposition/release/verification/monitoring; zero opportunities never fixed; pause/resume only exact allowed host intents |
| `plain_text_and_privacy_boundaries`: markup-shaped answer/source/error, denied-content sentinel, raw diagnostic sentinel | HTML/script/image/URL strings remain escaped text; script sentinel untouched, zero external resource requests, no auto-navigation. Capture browser console/errors and verify forbidden sentinels absent; Copy only permitted visible content with acknowledged success/failure, no hidden evidence |
| `keyboard_focus_and_scroll_survive_updates`: narrow/wide, EN/ES, long content, scrolled history, new output, detail | Native Select Space/Home/Arrow/Enter; one active interactive tree, no duplicate Ask/Save; detail Back/Escape focus return; Escape never Cancel; scroll-up holds position/focus and offers New output; concise live status not token spam. Inject hidden interactive duplicate, catch/revert |
| `visual_and_disposal_contract`: light/dark, narrow/wide, EN/ES, 200% text, reduced motion, keyboard-height viewport, repeated mount/unmount | Reviewed screenshots of meaningful states; readable scope/evidence/long identifiers/errors, contrast/non-color status/focus indicator; current source/browser hash; no leaked listeners/observers and no callbacks after disposal; zero browser errors |

Native tests cover every model/guard branch before browser integration, including
unknown variants and invalid construction. Test names may be split for bounded
runtime, but each mapped obligation must remain explicit and independently
asserted. Do not widen timing tolerances to hide races or replace effect evidence
with a source-string check. Each negative control must visibly fail the intended
oracle on injection and pass again after restoration.

The shared-host proof certifies the library boundary only. Office still must
qualify authorization, event/replay reconciliation, actual idempotency, provider
isolation, memory/evidence invalidation, vendor revision/digest and legacy-surface
migration. Preserve the reviewed Tachys attribute-budget test on integration;
ordinary library tags stay below 26 attributes and do not dump diagnostics on main.
