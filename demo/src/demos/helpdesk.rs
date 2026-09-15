//! Showcase and browser-proof fixture for the `Helpdesk` composite.

use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    HelpdeskFault, HelpdeskRole, InMemoryHelpdeskBackend, RequestContext, SEED_ME,
};
use std::rc::Rc;

use crate::core::{ContentLayout, Section};

/// Showcase page: a role switch over one seeded backend.
#[component]
pub fn HelpdeskDemo() -> impl IntoView {
    let role = RwSignal::new(HelpdeskRole::Support);
    // A `StoredValue` handle is Send+Sync regardless of what it holds, which
    // is required here: `ContentLayout`/`Section`'s `Children` slot boxes
    // this whole block as `Box<dyn FnOnce() -> AnyView + Send>`, and the
    // backend's `Rc<dyn HelpdeskBackend>` is neither. Capturing the handle
    // (not the `Rc` itself) keeps the closure's environment Send.
    let backend = StoredValue::new_local(Rc::new(InMemoryHelpdeskBackend::seeded())
        as Rc<dyn leptos_daisyui_rs::patterns::HelpdeskBackend>);
    let ctx = RequestContext {
        route: "/components/helpdesk".into(),
        build: "demo".into(),
    };
    view! {
        <ContentLayout
            title="Helpdesk"
            description="Role-switched Jira ticket board with a New Request dialog, driven by a HelpdeskBackend"
        >
            <Section title="Role">
                <div class="join" data-testid="helpdesk-role-switch">
                    <button
                        class="join-item btn btn-sm"
                        class:btn-active=move || role.get() == HelpdeskRole::Requester
                        on:click=move |_| role.set(HelpdeskRole::Requester)
                    >
                        "Requester"
                    </button>
                    <button
                        class="join-item btn btn-sm"
                        class:btn-active=move || role.get() == HelpdeskRole::Support
                        on:click=move |_| role.set(HelpdeskRole::Support)
                    >
                        "Support"
                    </button>
                </div>
            </Section>
            <Section title="Board">
                <div class="w-full min-w-0">
                    <leptos_daisyui_rs::patterns::Helpdesk
                        backend=backend.get_value()
                        role=role
                        context=ctx.clone()
                        current_user_id=Some(SEED_ME.to_owned())
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}

/// Browser-proof fixture: both roles mounted on one document over one
/// shared backend, plus an optional fault selected by `?fault=`.
#[component]
pub fn HelpdeskFixture() -> impl IntoView {
    let search = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    let fault = if search.contains("fault=not-configured") {
        Some(HelpdeskFault::NotConfigured)
    } else if search.contains("fault=fail-writes") {
        Some(HelpdeskFault::FailWrites)
    } else {
        None
    };
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
    let backend: Rc<dyn leptos_daisyui_rs::patterns::HelpdeskBackend> = Rc::new(backend);
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
