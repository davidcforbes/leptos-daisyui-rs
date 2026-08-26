//! Page-scoped browser-test host for the client-snapshot list pattern.
//!
//! This intentionally links one story instead of the full showcase catalog so
//! a focused component journey does not pay the compile/link cost of every
//! unrelated demo page.

#[path = "demos/client_snapshot_list.rs"]
mod client_snapshot_list;

use client_snapshot_list::ClientSnapshotListDemo;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_daisyui_rs::test_mode;
use leptos_daisyui_rs::tokens::{UiAnimationsPreamble, UiTokensPreamble};

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    if test_mode::is_test_mode() {
        test_mode::install_style_kill_switch();
    }

    mount_to_body(|| {
        view! {
            <UiTokensPreamble />
            <UiAnimationsPreamble />
            <main class="min-h-screen bg-base-200 p-4 sm:p-6">
                <ClientSnapshotListDemo />
            </main>
        }
    });
}
