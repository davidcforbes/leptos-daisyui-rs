//! State-complete container that retains usable stale content.

use super::contracts::PageState;
use crate::components::Button;
use leptos::prelude::*;

/// Default copy for every data-section replacement and retention state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsyncDataTexts {
    /// Initial loading title announced to assistive technology.
    pub loading: String,
    /// First-load error title.
    pub initial_error: String,
    /// Prompt shown before any dataset has loaded.
    pub never_loaded: String,
    /// Empty-dataset title.
    pub empty: String,
    /// Locally filtered-empty title.
    pub filtered_empty: String,
    /// Background-refresh status.
    pub revalidating: String,
    /// Refresh failure shown above retained content.
    pub refresh_error: String,
    /// Stale-snapshot warning shown above retained content.
    pub stale: String,
    /// Live-update interruption shown above retained content.
    pub live_interrupted: String,
    /// Row-claim conflict shown above retained content.
    pub claim_conflict: String,
    /// Row-claim failure shown above retained content.
    pub claim_failed: String,
    /// Retry action label.
    pub retry: String,
}

impl Default for AsyncDataTexts {
    fn default() -> Self {
        Self {
            loading: "Loading dataset".to_owned(),
            initial_error: "The dataset could not be loaded.".to_owned(),
            never_loaded: "Choose a dataset to begin.".to_owned(),
            empty: "This dataset has no rows.".to_owned(),
            filtered_empty: "No rows match the current filters.".to_owned(),
            revalidating: "Refreshing dataset…".to_owned(),
            refresh_error: "Refresh failed. The previous snapshot is still shown.".to_owned(),
            stale: "This snapshot may be out of date.".to_owned(),
            live_interrupted: "Live updates are interrupted. The last snapshot is still shown."
                .to_owned(),
            claim_conflict: "This row was claimed by another user.".to_owned(),
            claim_failed: "The row could not be claimed.".to_owned(),
            retry: "Retry".to_owned(),
        }
    }
}

/// Returns whether a state has a usable snapshot that must remain mounted.
pub fn state_shows_content(state: PageState) -> bool {
    matches!(
        state,
        PageState::Ready
            | PageState::Revalidating
            | PageState::RefreshError
            | PageState::Stale
            | PageState::Claiming
            | PageState::ClaimSucceeded
            | PageState::ClaimConflict
            | PageState::ClaimFailed
            | PageState::LiveInterrupted
    )
}

/// Renders all required async states while keeping an existing snapshot mounted.
#[component]
pub fn AsyncDataSection(
    /// Current page state from the declared page contract.
    #[prop(into)]
    state: Signal<PageState>,
    /// State-specific copy.
    #[prop(into, default = Signal::stored(AsyncDataTexts::default()))]
    texts: Signal<AsyncDataTexts>,
    /// Retry callback used by initial and retained refresh errors.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
    /// Usable dataset content. It is mounted once and hidden only for replacement states.
    children: Children,
) -> impl IntoView {
    let content = children();
    view! {
        <section
            class="min-w-0 space-y-3"
            data-async-data-section="true"
            aria-busy=move || matches!(
                state.get(),
                PageState::InitialLoading | PageState::Revalidating
            ).then_some("true")
        >
            {move || render_state_message(state.get(), texts.get(), on_retry)}
            <div
                class:hidden=move || !state_shows_content(state.get())
                data-retained-content="true"
            >
                {content}
            </div>
        </section>
    }
}

fn render_state_message(
    state: PageState,
    texts: AsyncDataTexts,
    on_retry: Option<Callback<()>>,
) -> AnyView {
    match state {
        PageState::InitialLoading => view! {
            <div class="space-y-3 py-4" role="status" aria-label=texts.loading>
                <div class="skeleton h-12 w-full"></div>
                <div class="skeleton h-12 w-full"></div>
                <div class="skeleton h-12 w-full"></div>
            </div>
        }
        .into_any(),
        PageState::InitialError => replacement_message(texts.initial_error, texts.retry, on_retry),
        PageState::NeverLoaded => replacement_message(texts.never_loaded, String::new(), None),
        PageState::Empty => replacement_message(texts.empty, String::new(), None),
        PageState::FilteredEmpty => replacement_message(texts.filtered_empty, String::new(), None),
        PageState::Revalidating => retained_alert("alert-info", texts.revalidating, None, None),
        PageState::RefreshError => retained_alert(
            "alert-warning",
            texts.refresh_error,
            Some(texts.retry),
            on_retry,
        ),
        PageState::Stale => retained_alert("alert-warning", texts.stale, None, None),
        PageState::LiveInterrupted => {
            retained_alert("alert-warning", texts.live_interrupted, None, None)
        }
        PageState::ClaimConflict => {
            retained_alert("alert-warning", texts.claim_conflict, None, None)
        }
        PageState::ClaimFailed => retained_alert("alert-error", texts.claim_failed, None, None),
        PageState::Ready | PageState::Claiming | PageState::ClaimSucceeded => ().into_any(),
    }
}

fn replacement_message(
    message: String,
    retry_label: String,
    on_retry: Option<Callback<()>>,
) -> AnyView {
    view! {
        <div class="flex min-h-48 flex-col items-center justify-center gap-3 rounded-box border border-dashed border-base-300 p-8 text-center">
            <p class="text-base font-medium text-base-content/75">{message}</p>
            {(on_retry.is_some()).then(|| view! {
                <Button
                    class="btn-primary btn-sm"
                    on_click=Callback::new(move |_| {
                        if let Some(callback) = on_retry {
                            callback.run(());
                        }
                    })
                >
                    {retry_label}
                </Button>
            })}
        </div>
    }
    .into_any()
}

fn retained_alert(
    color_class: &'static str,
    message: String,
    retry_label: Option<String>,
    on_retry: Option<Callback<()>>,
) -> AnyView {
    view! {
        <div class=format!("alert {color_class}") role="alert" data-retained-state="true">
            <span>{message}</span>
            {(retry_label.is_some() && on_retry.is_some()).then(|| view! {
                <Button
                    class="btn-ghost btn-sm"
                    on_click=Callback::new(move |_| {
                        if let Some(callback) = on_retry {
                            callback.run(());
                        }
                    })
                >
                    {retry_label.clone().unwrap_or_default()}
                </Button>
            })}
        </div>
    }
    .into_any()
}
