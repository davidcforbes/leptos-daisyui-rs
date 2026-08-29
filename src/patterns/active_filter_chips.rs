//! Removable summary chips for active local filters.

use crate::components::Button;
use leptos::prelude::*;

/// One active local filter rendered as a removable chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveFilterChip {
    /// Stable filter key passed to the removal callback.
    pub id: String,
    /// Human-readable filter label.
    pub label: String,
    /// Current filter value.
    pub value: String,
}

impl ActiveFilterChip {
    /// Creates a removable local-filter chip.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Reactive framework-owned copy for active-filter summaries and actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveFilterTexts {
    /// Summary shown when no filters are active.
    pub none: String,
    /// Summary shown when exactly one filter is active.
    pub one: String,
    /// Summary template for multiple filters; `{count}` is replaced.
    pub many: String,
    /// Accessible-name template for a chip removal; `{label}` is replaced.
    pub remove: String,
    /// Compatibility clear-all action label.
    pub clear: String,
}

impl Default for ActiveFilterTexts {
    fn default() -> Self {
        Self {
            none: "No active filters".to_owned(),
            one: "1 active filter".to_owned(),
            many: "{count} active filters".to_owned(),
            remove: "Remove {label} filter".to_owned(),
            clear: "Clear filters".to_owned(),
        }
    }
}

/// Produces the canonical active-filter count label.
pub fn active_filter_summary(count: usize) -> String {
    active_filter_summary_with(count, &ActiveFilterTexts::default())
}

/// Produces a localized active-filter summary.
pub fn active_filter_summary_with(count: usize, texts: &ActiveFilterTexts) -> String {
    match count {
        0 => texts.none.clone(),
        1 => texts.one.clone(),
        count => texts.many.replace("{count}", &count.to_string()),
    }
}

/// Renders individually removable local filters and an optional clear-all action.
#[component]
pub fn ActiveFilterChips(
    /// Current active local filters.
    #[prop(into)]
    chips: Signal<Vec<ActiveFilterChip>>,
    /// Called with the stable filter key when a chip is removed.
    on_remove: Callback<String>,
    /// Optional clear-all callback. It never affects the dataset selector.
    #[prop(optional)]
    on_clear: Option<Callback<()>>,
    /// Reactive framework-owned summary, removal, and compatibility copy.
    #[prop(into, default = Signal::stored(ActiveFilterTexts::default()))]
    texts: Signal<ActiveFilterTexts>,
    /// Clear-all action label.
    ///
    /// An empty value selects `texts.clear`; a nonempty value preserves the
    /// historical per-call override.
    #[prop(into, default = Signal::stored(String::new()))]
    clear_label: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            class="flex min-h-7 flex-wrap items-center gap-2"
            data-active-filters="true"
            data-resets-dataset="false"
        >
            <span class="text-xs text-base-content/75">
                {move || texts.with(|texts| {
                    active_filter_summary_with(chips.with(|chips| chips.len()), texts)
                })}
            </span>
            {move || chips.get().into_iter().map(|chip| {
                let id = chip.id.clone();
                let chip_label = chip.label.clone();
                let remove_label = move || texts.with(|texts| {
                    texts.remove.replace("{label}", &chip_label)
                });
                view! {
                    <span class="badge badge-outline gap-1 py-3">
                        <span class="font-medium">{chip.label}</span>
                        <span>{chip.value}</span>
                        <Button
                            class="btn-ghost btn-xs btn-circle h-5 min-h-5 w-5"
                            attr:aria-label=remove_label
                            on_click=Callback::new(move |_| on_remove.run(id.clone()))
                        >
                            "×"
                        </Button>
                    </span>
                }
            }).collect_view()}
            {move || {
                let has_filters = chips.with(|chips| !chips.is_empty());
                (has_filters && on_clear.is_some()).then(|| view! {
                    <Button
                        class="btn-ghost btn-xs"
                        on_click=Callback::new(move |_| {
                            if let Some(callback) = on_clear {
                                callback.run(());
                            }
                        })
                    >
                        {move || {
                            let override_label = clear_label.get();
                            if override_label.is_empty() {
                                texts.with(|texts| texts.clear.clone())
                            } else {
                                override_label
                            }
                        }}
                    </Button>
                })
            }}
        </div>
    }
}
