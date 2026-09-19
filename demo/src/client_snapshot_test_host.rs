//! Page-scoped browser-test host for the client-snapshot list pattern.
//!
//! This intentionally links one story instead of the full showcase catalog so
//! a focused component journey does not pay the compile/link cost of every
//! unrelated demo page.

#[path = "demos/ai_chat_fixture.rs"]
mod ai_chat_fixture;
#[path = "demos/client_snapshot_list.rs"]
mod client_snapshot_list;
mod debug;
mod debug_state;
#[path = "demos/helpdesk_fixture.rs"]
mod helpdesk_fixture;
mod office_delta_fixture;
mod office_entity_defaults_fixture;
#[path = "demos/snapshot_table_page.rs"]
mod snapshot_table_page;

use client_snapshot_list::ClientSnapshotListDemo;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    AvailabilityReasonCode, ChatWorkspaceFault, FixtureClock, SEED_CHAT_NOW_MS,
};
use leptos_daisyui_rs::test_mode;
use leptos_daisyui_rs::tokens::{UiAnimationsPreamble, UiTokensPreamble};
use snapshot_table_page::{
    EntityTableDraftRowFixture, EntityTableEmphasisFixture, EntityTableExternalFocusFixture,
    EntityTableGroupPagingFixture, EntityTableGroupingFixture, EntityTableMultiSelectionFixture,
    EntityTablePageSizeIdentityFixture, EntityTablePresentationFixture,
    EntityTableSavedFiltersFixture, EntityTableSelectionFixture, EntityTableViewportFitFixture,
    SnapshotTablePageControlsFixture, SnapshotTablePageFilterActionsFixture,
    SnapshotTablePageFixture,
};

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    if test_mode::is_test_mode() {
        test_mode::install_style_kill_switch();
        debug::register_signal("route", || {
            serde_json::Value::String(
                web_sys::window()
                    .and_then(|window| window.location().pathname().ok())
                    .unwrap_or_default(),
            )
        });
        debug::register_signal("theme", || serde_json::Value::String("light".to_owned()));
        debug::register_signal("state", debug_state::get_all);
        debug::install_debug_bridge();
    }

    mount_to_body(|| {
        let snapshot_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/snapshot-table-page"));
        let snapshot_controls_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/snapshot-table-page-controls"));
        let snapshot_filter_actions_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/snapshot-table-page-filter-actions"));
        let draft_row_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-draft-row"));
        let viewport_fit_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-viewport-fit"));
        let presentation_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-presentation"));
        let page_size_identity_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-page-size-identity"));
        let selection_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-selection"));
        let saved_filters_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-saved-filters"));
        let multi_selection_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-multi-selection"));
        let emphasis_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-emphasis"));
        let grouping_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-grouping"));
        let group_paging_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-group-paging"));
        let external_focus_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/entity-table-external-focus"));
        // The BASE `/ai-chat-fixture` suffix is matched LAST of the ai-chat
        // family, so every `/ai-chat-fixture-<fault>` document below is
        // tested BEFORE it — `ends_with` would otherwise swallow them all,
        // since each of their suffixes contains the base one.
        let ai_chat_path = || {
            web_sys::window()
                .and_then(|window| window.location().pathname().ok())
                .unwrap_or_default()
        };
        let ai_chat_failures = ai_chat_path().ends_with("/ai-chat-fixture-failures");
        let ai_chat_groq_no_key = ai_chat_path().ends_with("/ai-chat-fixture-groq-no-key");
        let ai_chat_tier_disabled = ai_chat_path().ends_with("/ai-chat-fixture-tier-disabled");
        let ai_chat_ollama = ai_chat_path().ends_with("/ai-chat-fixture-ollama-unreachable");
        let ai_chat_watchdog = ai_chat_path().ends_with("/ai-chat-fixture-watchdog");
        let ai_chat_budget = ai_chat_path().ends_with("/ai-chat-fixture-budget-exhausted");
        let ai_chat_cancel = ai_chat_path().ends_with("/ai-chat-fixture-cancel");
        let ai_chat_knowledge = ai_chat_path().ends_with("/ai-chat-fixture-knowledge");
        let ai_chat_fixture = ai_chat_path().ends_with("/ai-chat-fixture");
        let helpdesk_fixture = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/helpdesk-fixture"));
        let helpdesk_fixture_not_configured = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/helpdesk-fixture-not-configured"));
        let helpdesk_fixture_fail_writes = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/helpdesk-fixture-fail-writes"));
        let helpdesk_fixture_no_attachments = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/helpdesk-fixture-no-attachments"));
        let helpdesk_fixture_read_all = web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .is_some_and(|path| path.ends_with("/helpdesk-fixture-read-all"));
        view! {
            <UiTokensPreamble />
            <UiAnimationsPreamble />
            <main class="min-h-screen bg-base-200 p-4 sm:p-6">
                {if web_sys::window()
                    .and_then(|window| window.location().pathname().ok())
                    .is_some_and(|path| path.ends_with("/office-select-regressions")) {
                    view! { <office_delta_fixture::OfficeSelectFixture /> }.into_any()
                } else if web_sys::window()
                    .and_then(|window| window.location().pathname().ok())
                    .is_some_and(|path| path.ends_with("/office-entity-defaults")) {
                    view! { <office_entity_defaults_fixture::OfficeEntityDefaultsFixture /> }.into_any()
                } else if helpdesk_fixture_not_configured {
                    view! {
                        <helpdesk_fixture::HelpdeskFixture fault=leptos_daisyui_rs::patterns::HelpdeskFault::NotConfigured />
                    }
                        .into_any()
                } else if helpdesk_fixture_fail_writes {
                    view! {
                        <helpdesk_fixture::HelpdeskFixture fault=leptos_daisyui_rs::patterns::HelpdeskFault::FailWrites />
                    }
                        .into_any()
                } else if helpdesk_fixture_read_all {
                    // ldui-ftdy: the host owns entitlements; the Requester
                    // section reads every ticket and may reply, but triage
                    // stays gated. Built by struct update over the role's
                    // default, which is how a consumer would write it.
                    let table = leptos_daisyui_rs::patterns::RoleCapabilities {
                        scope: leptos_daisyui_rs::patterns::TicketScope::All,
                        triage: false,
                        comment: true,
                        ..leptos_daisyui_rs::patterns::RoleCapabilities::for_role(
                            leptos_daisyui_rs::patterns::HelpdeskRole::Requester,
                        )
                    };
                    view! { <helpdesk_fixture::HelpdeskFixture requester_capabilities=table /> }
                        .into_any()
                } else if helpdesk_fixture_no_attachments {
                    view! { <helpdesk_fixture::HelpdeskFixture attachments=false /> }.into_any()
                } else if ai_chat_failures {
                    view! { <ai_chat_fixture::AiChatFixture scripted=true /> }.into_any()
                } else if ai_chat_groq_no_key {
                    view! {
                        <ai_chat_fixture::AiChatFixture
                            scripted=true
                            fault=ChatWorkspaceFault::KeyMissing {
                                engine_id: "groq-gpt-oss-120b".to_owned(),
                            }
                        />
                    }
                        .into_any()
                } else if ai_chat_tier_disabled {
                    view! {
                        <ai_chat_fixture::AiChatFixture fault=ChatWorkspaceFault::TierDisabled />
                    }
                        .into_any()
                } else if ai_chat_ollama {
                    // TWO workspaces, because one backend carries one fault
                    // and these two reasons only mean something beside each
                    // other: a runtime that is not running is a different
                    // problem, with a different fix, from a model that is
                    // not installed. Only the first owns the oracle.
                    view! {
                        <ai_chat_fixture::AiChatFixture
                            case="not_running"
                            fault=ChatWorkspaceFault::EngineUnavailable {
                                engine_id: "ollama".to_owned(),
                                code: AvailabilityReasonCode::EngineProcessNotRunning,
                            }
                        />
                        // This one also declines the quick-action bar. The
                        // two roots are therefore each other's control for
                        // `AiChatWorkspace`'s `show_quick_actions` opt-out:
                        // "the bar is absent here" only means the prop works
                        // if the workspace beside it, on the same document
                        // and the same build, still has one.
                        <ai_chat_fixture::AiChatFixture
                            case="model_missing"
                            oracle=false
                            quick_actions=false
                            fault=ChatWorkspaceFault::EngineUnavailable {
                                engine_id: "ollama".to_owned(),
                                code: AvailabilityReasonCode::ModelNotInstalled,
                            }
                        />
                    }
                        .into_any()
                } else if ai_chat_watchdog {
                    // A clock that jumps 20 s per transport poll crosses the
                    // 120 000 ms watchdog in seven polls instead of twelve
                    // hundred; its neighbour's clock never advances at all,
                    // so the same stalled script must NOT trip. That pair is
                    // the proof that the threshold is read from the clock.
                    view! {
                        <ai_chat_fixture::AiChatFixture
                            case="fast"
                            scripted=true
                            clock=FixtureClock::new(SEED_CHAT_NOW_MS, 20_000)
                        />
                        <ai_chat_fixture::AiChatFixture
                            case="frozen"
                            oracle=false
                            scripted=true
                            clock=FixtureClock::new(SEED_CHAT_NOW_MS, 0)
                        />
                    }
                        .into_any()
                } else if ai_chat_budget {
                    view! {
                        <ai_chat_fixture::AiChatFixture
                            fault=ChatWorkspaceFault::BudgetExhausted
                        />
                    }
                        .into_any()
                } else if ai_chat_cancel {
                    // Same shape as the ollama document: one backend can
                    // only discard OR keep, so the two cancels are two
                    // workspaces on one page.
                    view! {
                        <ai_chat_fixture::AiChatFixture case="kept" scripted=true />
                        <ai_chat_fixture::AiChatFixture
                            case="discarded"
                            oracle=false
                            scripted=true
                            fault=ChatWorkspaceFault::CancelDiscards
                        />
                    }
                        .into_any()
                } else if ai_chat_knowledge {
                    // TWO workspaces. A healthy memory store and an offline
                    // one are each other's negative control: "a receipt with
                    // four attribution lanes" is only evidence of a real
                    // search if an unreachable store on the same document
                    // renders a typed honesty row and NO receipt at all.
                    //
                    // Neither instance is `scripted`, so both keep
                    // `KnowledgeSelection::default()`'s GROUNDED posture --
                    // which is what test 4 measures and what a scripted
                    // document cannot have (a grounded prompt matching no
                    // seeded document has its script replaced by the
                    // not-found sentence).
                    view! {
                        <ai_chat_fixture::AiChatFixture case="knowledge" />
                        <ai_chat_fixture::AiChatFixture
                            case="offline"
                            oracle=false
                            fault=ChatWorkspaceFault::MemoryStoreOffline
                        />
                    }
                        .into_any()
                } else if ai_chat_fixture {
                    view! { <ai_chat_fixture::AiChatFixture /> }.into_any()
                } else if helpdesk_fixture {
                    view! { <helpdesk_fixture::HelpdeskFixture /> }.into_any()
                } else if external_focus_fixture {
                    view! { <EntityTableExternalFocusFixture /> }.into_any()
                } else if group_paging_fixture {
                    view! { <EntityTableGroupPagingFixture /> }.into_any()
                } else if grouping_fixture {
                    view! { <EntityTableGroupingFixture /> }.into_any()
                } else if multi_selection_fixture {
                    view! { <EntityTableMultiSelectionFixture /> }.into_any()
                } else if emphasis_fixture {
                    view! { <EntityTableEmphasisFixture /> }.into_any()
                } else if selection_fixture {
                    view! { <EntityTableSelectionFixture /> }.into_any()
                } else if saved_filters_fixture {
                    view! { <EntityTableSavedFiltersFixture /> }.into_any()
                } else if page_size_identity_fixture {
                    view! { <EntityTablePageSizeIdentityFixture /> }.into_any()
                } else if presentation_fixture {
                    view! { <EntityTablePresentationFixture /> }.into_any()
                } else if viewport_fit_fixture {
                    view! { <EntityTableViewportFitFixture /> }.into_any()
                } else if draft_row_fixture {
                    view! { <EntityTableDraftRowFixture /> }.into_any()
                } else if snapshot_filter_actions_fixture {
                    view! { <SnapshotTablePageFilterActionsFixture /> }.into_any()
                } else if snapshot_controls_fixture {
                    view! { <SnapshotTablePageControlsFixture /> }.into_any()
                } else if snapshot_fixture {
                    view! { <SnapshotTablePageFixture /> }.into_any()
                } else {
                    view! { <ClientSnapshotListDemo /> }.into_any()
                }}
            </main>
        }
    });
}
