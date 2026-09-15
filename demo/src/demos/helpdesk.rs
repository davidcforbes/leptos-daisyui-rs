//! Showcase page for the `Helpdesk` composite. The browser-proof fixture
//! lives in `helpdesk_fixture.rs` (self-contained, no `crate::core`
//! dependency, shared with the test-host binary).

use leptos::prelude::*;
use leptos_daisyui_rs::patterns::{
    HelpdeskBackend, HelpdeskRole, InMemoryHelpdeskBackend, RequestContext, SEED_ME,
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
    let backend = StoredValue::new_local(
        Rc::new(InMemoryHelpdeskBackend::seeded()) as Rc<dyn HelpdeskBackend>
    );
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
