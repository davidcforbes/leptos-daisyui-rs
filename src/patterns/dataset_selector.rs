//! Dataset selection that is structurally separate from local filters.

use crate::components::Select;
use leptos::prelude::*;

/// One selectable complete dataset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetOption {
    /// Value sent to the page controller when selected.
    pub value: String,
    /// Human-readable option label.
    pub label: String,
    /// Whether this dataset is currently unavailable.
    pub disabled: bool,
}

impl DatasetOption {
    /// Creates an enabled dataset option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Marks an option unavailable.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Returns the label for the selected dataset value.
pub fn selected_dataset_label<'a>(options: &'a [DatasetOption], selected: &str) -> Option<&'a str> {
    options
        .iter()
        .find(|option| option.value == selected)
        .map(|option| option.label.as_str())
}

/// A busy dataset replacement remains supersedable. Only an explicit caller
/// gate disables the selector; `loading` controls busy semantics and progress
/// presentation without trapping the user in the in-flight choice.
pub(super) const fn selector_disabled(disabled: bool, _loading: bool) -> bool {
    disabled
}

/// Selector whose value determines which complete dataset is downloaded.
///
/// This component deliberately exposes `data-resettable-filter="false"` and
/// lives in the page-header dataset slot, not in [`FilterBar`](super::FilterBar).
#[component]
pub fn DatasetSelector(
    /// Visible and accessible control label.
    #[prop(into)]
    label: Signal<String>,
    /// Current dataset key.
    #[prop(into)]
    selected: Signal<String>,
    /// Available datasets.
    #[prop(into)]
    options: Signal<Vec<DatasetOption>>,
    /// Called when the user requests a different dataset.
    on_change: Callback<String>,
    /// Whether the requested dataset is loading.
    #[prop(optional, into)]
    loading: Signal<bool>,
    /// Whether dataset selection is disabled.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Optional dataset-load error shown without clearing the prior selection.
    #[prop(optional, into)]
    error: Signal<Option<String>>,
    /// Optional live/freshness status beside the selector.
    #[prop(optional)]
    status: Option<Children>,
    /// Additional outer classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    view! {
        <div
            class=format!("rounded-box bg-base-200 px-3 py-2 {class}")
            data-dataset-selector="true"
            data-resettable-filter="false"
            aria-busy=move || loading.get().then_some("true")
        >
            <label class="flex flex-wrap items-center gap-2">
                <span class="text-xs font-semibold uppercase tracking-wide text-base-content/75">
                    {move || label.get()}
                </span>
                <Select
                    class="select-sm min-w-44 bg-base-100"
                    label=Signal::derive(move || Some(label.get()))
                    value=selected
                    disabled=Signal::derive(move || {
                        selector_disabled(disabled.get(), loading.get())
                    })
                    on_change=on_change
                >
                    {move || options.get().into_iter().map(|option| view! {
                        <option value=option.value disabled=option.disabled>{option.label}</option>
                    }).collect_view()}
                </Select>
                {move || loading.get().then(|| view! {
                    <span class="loading loading-spinner loading-sm" aria-label="Loading dataset"></span>
                })}
                {status.map(|status| status())}
            </label>
            {move || error.get().map(|message| view! {
                <p class="mt-1 text-xs text-error" role="alert">{message}</p>
            })}
        </div>
    }
}
