//! Page-scoped browser-test host for the client-snapshot list pattern.
//!
//! This intentionally links one story instead of the full showcase catalog so
//! a focused component journey does not pay the compile/link cost of every
//! unrelated demo page.

#[path = "demos/client_snapshot_list.rs"]
mod client_snapshot_list;
mod debug;
mod debug_state;
#[path = "demos/snapshot_table_page.rs"]
mod snapshot_table_page;

use client_snapshot_list::ClientSnapshotListDemo;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::test_mode;
use leptos_daisyui_rs::tokens::{UiAnimationsPreamble, UiTokensPreamble};
use snapshot_table_page::{
    EntityTableEmphasisFixture, EntityTableExternalFocusFixture, EntityTableGroupPagingFixture,
    EntityTableGroupingFixture, EntityTableMultiSelectionFixture,
    EntityTablePageSizeIdentityFixture, EntityTablePresentationFixture,
    EntityTableSelectionFixture, EntityTableViewportFitFixture, SnapshotTablePageControlsFixture,
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
        view! {
            <UiTokensPreamble />
            <UiAnimationsPreamble />
            <main class="min-h-screen bg-base-200 p-4 sm:p-6">
                {if external_focus_fixture {
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
                } else if page_size_identity_fixture {
                    view! { <EntityTablePageSizeIdentityFixture /> }.into_any()
                } else if presentation_fixture {
                    view! { <EntityTablePresentationFixture /> }.into_any()
                } else if viewport_fit_fixture {
                    view! { <EntityTableViewportFitFixture /> }.into_any()
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
