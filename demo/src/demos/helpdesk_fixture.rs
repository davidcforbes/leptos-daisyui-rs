//! Browser-proof fixture for the `Helpdesk` composite: both roles mounted
//! on one document over one shared backend, plus an optional fault.
//! Self-contained (no `crate::core` dependency) so the test-host binary can
//! include it without the full showcase chrome.
//!
//! The fault is a plain prop, not a `?fault=` query param: the test host's
//! harness always appends its own `?pp-freeze=1`, and a second `?` is not a
//! delimiter to `location.search`'s consumer here (`test_mode::is_enabled`
//! splits on `&` only), so `?fault=x?pp-freeze=1` parses as ONE param named
//! `fault` and test/freeze mode never activates. The test host instead
//! selects the fault by pathname suffix, matching every other fixture here.

use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    HelpdeskBackend, HelpdeskFault, HelpdeskRole, InMemoryHelpdeskBackend, RequestContext, SEED_ME,
};
use std::rc::Rc;

/// Both roles mounted on one document over one shared backend.
#[component]
pub fn HelpdeskFixture(#[prop(optional)] fault: Option<HelpdeskFault>) -> impl IntoView {
    let mut backend = InMemoryHelpdeskBackend::seeded();
    if let Some(f) = fault {
        backend = backend.with_fault(f);
    }
    let calls_backend = backend.clone();
    crate::debug::register_signal("helpdesk_calls", move || {
        serde_json::to_value(
            calls_backend
                .calls()
                .iter()
                .map(|c| format!("{c:?}"))
                .collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null)
    });
    let backend: Rc<dyn HelpdeskBackend> = Rc::new(backend);
    let ctx = RequestContext {
        route: "/fixture".into(),
        build: "test".into(),
    };
    let fixed_now = Signal::stored(1_800_000_000_000i64);
    view! {
        <div class="flex flex-col gap-8">
            <section id="helpdesk-support" data-testid="helpdesk-support">
                <leptos_daisyui_rs::patterns::Helpdesk
                    backend=backend.clone()
                    role=HelpdeskRole::Support
                    context=ctx.clone()
                    current_user_id=Some(SEED_ME.to_owned())
                    now_ms=fixed_now
                />
            </section>
            <section id="helpdesk-requester" data-testid="helpdesk-requester">
                <leptos_daisyui_rs::patterns::Helpdesk
                    backend=backend.clone()
                    role=HelpdeskRole::Requester
                    context=ctx.clone()
                    current_user_id=Some(SEED_ME.to_owned())
                    now_ms=fixed_now
                />
            </section>
        </div>
    }
}
