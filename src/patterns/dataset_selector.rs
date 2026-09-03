//! Dataset selection that is structurally separate from local filters.

use crate::components::{Button, Select};
use leptos::prelude::*;

/// Reactive framework-owned copy for dataset replacement presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetSelectorTexts {
    /// Accessible progress label while a replacement is pending.
    pub loading: String,
    /// Template for a displayed-dataset status; `{dataset}` is replaced by consumers.
    pub displayed: String,
    /// Template for a requested destination; `{dataset}` is replaced by consumers.
    pub requested: String,
    /// Prefix used when an older dataset is retained after replacement failure.
    pub retained_error: String,
    /// Retry action label.
    pub retry: String,
}

impl Default for DatasetSelectorTexts {
    fn default() -> Self {
        Self {
            loading: "Loading dataset".to_owned(),
            displayed: "Showing {dataset}".to_owned(),
            requested: "Loading {dataset}".to_owned(),
            retained_error: "Could not replace dataset".to_owned(),
            retry: "Retry".to_owned(),
        }
    }
}

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
    /// Stable, caller-owned DOM identity for the underlying select.
    #[prop(optional, into)]
    control_id: MaybeProp<String>,
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
    /// Reactive framework-owned loading/error/retry copy.
    #[prop(into, default = Signal::stored(DatasetSelectorTexts::default()))]
    texts: Signal<DatasetSelectorTexts>,
    /// Optional explicit retry intent for a retained load failure.
    #[prop(optional)]
    on_retry: Option<Callback<()>>,
    /// Optional live/freshness status beside the selector.
    #[prop(optional)]
    status: Option<Children>,
    /// Additional outer classes.
    #[prop(optional, into)]
    class: &'static str,
) -> impl IntoView {
    let has_custom_status = status.is_some();
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
                    id=control_id
                    label=Signal::derive(move || Some(label.get()))
                    value=selected
                    // `ldui-uxdw`: this pattern owns BOTH signals, so it can
                    // tell `Select` when the option set changed and no caller
                    // has to know the hazard exists. Without it, options that
                    // arrive asynchronously leave the browser's index-0 choice
                    // in place while `selected` still says otherwise -- two
                    // Office pages showed "Charlotte" beside Raleigh's rows.
                    options_revision=Signal::derive(move || {
                        options.with(|options| dataset_options_revision(options))
                    })
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
                    <span
                        class="loading loading-spinner loading-sm"
                        aria-label=move || texts.with(|texts| texts.loading.clone())
                    ></span>
                })}
                {status.map(|status| status())}
                {(!has_custom_status).then(|| view! {
                    <span class="text-xs text-base-content/70" data-dataset-selector-status="true">
                        {move || {
                            let selected = selected.get();
                            let label = options.with(|options| {
                                selected_dataset_label(options, &selected)
                                    .unwrap_or(selected.as_str())
                                    .to_owned()
                            });
                            texts.with(|texts| {
                                if loading.get() {
                                    texts.requested.replace("{dataset}", &label)
                                } else {
                                    texts.displayed.replace("{dataset}", &label)
                                }
                            })
                        }}
                    </span>
                })}
            </label>
            {move || error.get().map(|message| view! {
                <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-error" role="alert">
                    <span>{move || texts.with(|texts| {
                        format!("{}: {message}", texts.retained_error)
                    })}</span>
                    {on_retry.map(|callback| view! {
                        <Button
                            class="btn-ghost btn-xs"
                            on_click=Callback::new(move |_| callback.run(()))
                        >
                            {move || texts.with(|texts| texts.retry.clone())}
                        </Button>
                    })}
                </div>
            })}
        </div>
    }
}

/// An opaque revision of a dataset selector's option set (`ldui-uxdw`).
///
/// Fed to [`Select`]'s `options_revision` so the bound value is re-asserted
/// after the browser rebuilds the option list. Any value that changes with the
/// option set would do; this one is derived from the option VALUES because
/// those are what `<option value=…>` matches on, so it changes exactly when
/// the set the browser can select from changes.
///
/// Uses a unit separator rather than a comma so two option sets cannot collide
/// by containing a separator inside a value — `["a,b"]` and `["a", "b"]` are
/// different sets and must produce different revisions.
fn dataset_options_revision(options: &[DatasetOption]) -> String {
    let mut revision = String::with_capacity(options.len() * 8);
    for option in options {
        revision.push_str(&option.value);
        revision.push('\u{1f}');
    }
    revision
}

#[cfg(test)]
mod revision_tests {
    use super::{DatasetOption, dataset_options_revision};

    fn option(value: &str) -> DatasetOption {
        DatasetOption {
            value: value.to_owned(),
            label: value.to_owned(),
            disabled: false,
        }
    }

    /// The property the whole `ldui-uxdw` fix rests on: the revision must
    /// change whenever the option set does, or the re-assert effect never
    /// re-runs and the browser's index-0 choice silently wins.
    #[test]
    fn the_revision_changes_whenever_the_option_set_does() {
        let empty = dataset_options_revision(&[]);
        let loaded = dataset_options_revision(&[option("raleigh"), option("charlotte")]);

        // The exact transition that failed in production: options arrive after
        // first render, so empty -> loaded MUST be observable.
        assert_ne!(
            empty, loaded,
            "options arriving asynchronously must change the revision"
        );

        // Order matters: the browser's index-0 differs, so the same members in
        // a different order is a different set for this purpose.
        let reordered = dataset_options_revision(&[option("charlotte"), option("raleigh")]);
        assert_ne!(loaded, reordered);

        // Stable for an unchanged set, so this does not thrash the effect on
        // every unrelated render.
        assert_eq!(
            loaded,
            dataset_options_revision(&[option("raleigh"), option("charlotte")])
        );
    }

    /// A separator inside a value must not let two different sets collide.
    #[test]
    fn values_containing_a_separator_cannot_collide() {
        assert_ne!(
            dataset_options_revision(&[option("a,b")]),
            dataset_options_revision(&[option("a"), option("b")]),
        );
    }
}
