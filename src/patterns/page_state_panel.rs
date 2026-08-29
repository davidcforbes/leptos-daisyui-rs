//! Consistent runtime state presentation for snapshot-table pages.

use super::PageStatePanelKind;
use crate::components::{Alert, AlertColor, AlertStyle, Button, ButtonSize, ButtonStyle};
use leptos::prelude::*;

/// Complete localizable copy owned by [`PageStatePanel`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageStatePanelTexts {
    /// Copy shown before any dataset request.
    pub never_loaded: String,
    /// Copy shown while the first request is pending.
    pub initial_loading: String,
    /// Copy shown when the first request fails.
    pub initial_error: String,
    /// Copy shown for an authoritative empty snapshot.
    pub empty_dataset: String,
    /// Copy shown when local filters match no rows.
    pub no_local_results: String,
    /// Copy shown when the session expired.
    pub expired: String,
    /// Copy shown when access is forbidden.
    pub forbidden: String,
    /// Copy shown while retained rows are being replaced.
    pub replacing: String,
    /// Copy shown when a replacement failed but rows are retained.
    pub retained_error: String,
    /// Retry action label.
    pub retry: String,
}

impl Default for PageStatePanelTexts {
    fn default() -> Self {
        Self {
            never_loaded: "Choose a dataset to begin.".to_owned(),
            initial_loading: "Loading dataset.".to_owned(),
            initial_error: "The dataset could not be loaded.".to_owned(),
            empty_dataset: "This dataset contains no rows.".to_owned(),
            no_local_results: "No rows match the current filters.".to_owned(),
            expired: "Your session has expired.".to_owned(),
            forbidden: "You do not have access to this dataset.".to_owned(),
            replacing: "Loading the requested dataset while current rows remain visible."
                .to_owned(),
            retained_error: "The requested dataset could not be loaded; current rows are retained."
                .to_owned(),
            retry: "Retry".to_owned(),
        }
    }
}

fn panel_slug(kind: PageStatePanelKind) -> &'static str {
    match kind {
        PageStatePanelKind::NeverLoaded => "never-loaded",
        PageStatePanelKind::InitialLoading => "initial-loading",
        PageStatePanelKind::InitialError => "initial-error",
        PageStatePanelKind::EmptyDataset => "empty-dataset",
        PageStatePanelKind::NoLocalResults => "no-local-results",
        PageStatePanelKind::Expired => "expired",
        PageStatePanelKind::Forbidden => "forbidden",
        PageStatePanelKind::Replacing => "replacing",
        PageStatePanelKind::RetainedError => "retained-error",
    }
}

fn panel_text(texts: &PageStatePanelTexts, kind: PageStatePanelKind) -> String {
    match kind {
        PageStatePanelKind::NeverLoaded => texts.never_loaded.clone(),
        PageStatePanelKind::InitialLoading => texts.initial_loading.clone(),
        PageStatePanelKind::InitialError => texts.initial_error.clone(),
        PageStatePanelKind::EmptyDataset => texts.empty_dataset.clone(),
        PageStatePanelKind::NoLocalResults => texts.no_local_results.clone(),
        PageStatePanelKind::Expired => texts.expired.clone(),
        PageStatePanelKind::Forbidden => texts.forbidden.clone(),
        PageStatePanelKind::Replacing => texts.replacing.clone(),
        PageStatePanelKind::RetainedError => texts.retained_error.clone(),
    }
}

fn is_error(kind: PageStatePanelKind) -> bool {
    matches!(
        kind,
        PageStatePanelKind::InitialError
            | PageStatePanelKind::Expired
            | PageStatePanelKind::Forbidden
            | PageStatePanelKind::RetainedError
    )
}

fn is_busy(kind: PageStatePanelKind) -> bool {
    matches!(
        kind,
        PageStatePanelKind::InitialLoading | PageStatePanelKind::Replacing
    )
}

fn allows_retry(kind: PageStatePanelKind) -> bool {
    matches!(
        kind,
        PageStatePanelKind::InitialError | PageStatePanelKind::RetainedError
    )
}

/// Renders one precedence-selected page state. Replacement-vs-retained
/// mounting remains the responsibility of [`SnapshotTablePage`](super::SnapshotTablePage).
#[component]
pub fn PageStatePanel(
    /// State selected by the pure snapshot render decision.
    kind: PageStatePanelKind,
    /// Reactive complete framework-owned copy.
    #[prop(into, default = Signal::stored(PageStatePanelTexts::default()))]
    texts: Signal<PageStatePanelTexts>,
    /// Optional typed retry intent for load failures.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
    /// Optional caller-provided error detail.
    #[prop(optional, into)]
    detail: Signal<Option<String>>,
) -> impl IntoView {
    let content = move || panel_text(&texts.get(), kind);
    let retry = move || {
        (allows_retry(kind) && on_retry.is_some()).then(|| {
            view! {
                <Button
                    style=ButtonStyle::Outline
                    size=ButtonSize::Sm
                    nostrip:on_click=on_retry
                        .map(|callback| Callback::new(move |_| callback.run(())))
                >
                    {move || texts.with(|texts| texts.retry.clone())}
                </Button>
            }
        })
    };

    if is_error(kind) {
        view! {
            <div data-page-state-panel=panel_slug(kind) aria-busy=is_busy(kind).then_some("true")>
                <Alert color=AlertColor::Error style=AlertStyle::Soft>
                    <div class="min-w-0 flex-1">
                        <p>{content}</p>
                        {move || detail.get().map(|detail| view! {
                            <p class="mt-1 text-sm opacity-80">{detail}</p>
                        })}
                    </div>
                    {retry}
                </Alert>
            </div>
        }
        .into_any()
    } else {
        view! {
            <section
                class="alert alert-info alert-soft"
                role="status"
                aria-live="polite"
                aria-busy=is_busy(kind).then_some("true")
                data-page-state-panel=panel_slug(kind)
            >
                {is_busy(kind).then(|| view! {
                    <span class="loading loading-spinner loading-sm" aria-hidden="true"></span>
                })}
                <p class="min-w-0 flex-1">{content}</p>
                {retry}
            </section>
        }
        .into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_panel_kind_has_nonempty_default_copy_and_expected_policy() {
        let texts = PageStatePanelTexts::default();
        let kinds = [
            PageStatePanelKind::NeverLoaded,
            PageStatePanelKind::InitialLoading,
            PageStatePanelKind::InitialError,
            PageStatePanelKind::EmptyDataset,
            PageStatePanelKind::NoLocalResults,
            PageStatePanelKind::Expired,
            PageStatePanelKind::Forbidden,
            PageStatePanelKind::Replacing,
            PageStatePanelKind::RetainedError,
        ];
        for kind in kinds {
            assert!(!panel_slug(kind).is_empty());
            assert!(!panel_text(&texts, kind).trim().is_empty());
        }
        assert!(is_busy(PageStatePanelKind::InitialLoading));
        assert!(is_busy(PageStatePanelKind::Replacing));
        assert!(allows_retry(PageStatePanelKind::InitialError));
        assert!(allows_retry(PageStatePanelKind::RetainedError));
        assert!(!allows_retry(PageStatePanelKind::Forbidden));
    }
}
