use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ToastDemo() -> impl IntoView {
    // Install the imperative toast service once for this page; ToastHost
    // renders whatever ToastService::push(...) queues up.
    provide_toast();

    view! {
        <ContentLayout
            title="Toast"
            description="Toast notifications are used to show brief messages to users"
        >

            <Section title="Toast Container (markup only)">
                <div class="w-full h-64 relative border border-base-300">
                    <Toast class="absolute">
                        <div class="alert alert-info">
                            <span>New message arrived.</span>
                        </div>
                    </Toast>
                </div>
            </Section>

            <Section title="Imperative Service">
                <p class="text-base-content/70 mb-4">
                    "Fire toasts from anywhere via " <code>"use_toast()"</code>
                    ". Each button below pushes one variant; the sticky toast (duration 0) stays until you dismiss it."
                </p>
                <ToastButtons />
                <ToastHost position=ToastPosition::TopEnd />
            </Section>
        </ContentLayout>
    }
}

#[component]
fn ToastButtons() -> impl IntoView {
    let toast = use_toast();

    view! {
        <div class="flex flex-wrap gap-2">
            <button
                class="btn btn-info btn-sm"
                on:click=move |_| { toast.push("Here's some info.", ToastVariant::Info); }
            >
                "Info toast"
            </button>
            <button
                class="btn btn-success btn-sm"
                on:click=move |_| { toast.push("Saved successfully!", ToastVariant::Success); }
            >
                "Success toast"
            </button>
            <button
                class="btn btn-warning btn-sm"
                on:click=move |_| { toast.push("Check your input.", ToastVariant::Warning); }
            >
                "Warning toast"
            </button>
            <button
                class="btn btn-error btn-sm"
                on:click=move |_| { toast.push("Something went wrong.", ToastVariant::Error); }
            >
                "Error toast"
            </button>
            <button
                class="btn btn-neutral btn-sm"
                on:click=move |_| {
                    toast
                        .push_with_duration(
                            "Sticky — dismiss me manually.",
                            ToastVariant::Default,
                            0,
                        );
                }
            >
                "Sticky toast"
            </button>
            <button class="btn btn-ghost btn-sm" on:click=move |_| toast.clear()>
                "Clear all"
            </button>
        </div>
    }
}
