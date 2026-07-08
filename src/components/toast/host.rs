use super::component::Toast;
use super::service::{ToastEntry, use_toast};
use super::style::ToastPosition;
use crate::components::alert::Alert;
use leptos::{html::Div, prelude::*};

/// # Toast Host Component
///
/// Mounts the imperative toast queue (see [`use_toast`]/
/// [`provide_toast`](super::service::provide_toast)) as daisyUI `Alert`s
/// inside the existing [`Toast`] positioning container. Each entry
/// auto-dismisses after its `duration_ms` (`0` = sticky — the user must
/// dismiss it via the close button).
///
/// Render one `ToastHost` near your app root (after calling
/// `provide_toast()`), then call `use_toast().push(...)` from anywhere below
/// it to fire a notification.
///
/// ## Example
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     provide_toast();
///
///     view! {
///         <ToastHost position=ToastPosition::TopEnd />
///         <button on:click=move |_| { use_toast().push("Saved!", ToastVariant::Success); }>
///             "Save"
///         </button>
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("toast toast-start toast-center toast-end toast-top toast-middle toast-bottom");
/// @source inline("alert alert-info alert-success alert-warning alert-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the underlying [`Toast`] container's top `<div>` ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn ToastHost(
    /// Position of the toast container on the page
    #[prop(optional, into)]
    position: Signal<ToastPosition>,

    /// Node reference for the underlying Toast container's top `<div>`
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let toast = use_toast();

    view! {
        <Toast position=position node_ref=node_ref>
            <For
                each=move || toast.entries().get()
                key=|entry| entry.id
                children=move |entry: ToastEntry| {
                    let id = entry.id;
                    if entry.duration_ms > 0 {
                        set_timeout(
                            move || toast.dismiss(id),
                            std::time::Duration::from_millis(entry.duration_ms as u64),
                        );
                    }
                    view! {
                        <Alert color=entry.variant.as_alert_color()>
                            <span>{entry.message.clone()}</span>
                            <button
                                type="button"
                                class="btn btn-sm btn-circle btn-ghost"
                                aria-label="Dismiss notification"
                                on:click=move |_| toast.dismiss(id)
                            >
                                "\u{2715}"
                            </button>
                        </Alert>
                    }
                }
            />
        </Toast>
    }
}
