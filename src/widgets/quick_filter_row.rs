//! Canonical Quick Filter row per DESIGN_SYSTEM.md §3.1.
//!
//! The canonical admin-screen filter pattern is a row of dropdown selects
//! (Status / Date Range / Department / per-screen-domain dropdowns) plus an
//! optional search input — NOT tab pills. Per-row category/status badges in
//! the data table carry lifecycle state separately.
//!
//! Replaces the legacy `FilterTabs` (filter pill bar) on 15 admin screens.
//! See FR-11-002, FR-12-002, FR-13-002, FR-14-002, FR-15-003, FR-16-003,
//! FR-17-003, FR-18-002, FR-19-006, FR-20-016, FR-21-007, FR-22-006 spec
//! reframings.
//!
//! Bead: EUC-dpj5

use crate::components::{Button, ButtonSize};
use leptos::prelude::*;

/// One dropdown filter (e.g., Status, Department, Severity).
#[derive(Clone)]
pub struct QuickFilterDropdown {
    pub label: &'static str,
    /// (value, display_label) pairs. The first option is the "no filter"
    /// catch-all (e.g., ("", "All Statuses")).
    pub options: Vec<(&'static str, &'static str)>,
    pub selected: RwSignal<String>,
}

/// One action button in the right-aligned cluster.
///
/// EUC-hh3j.44: filter-row audit found Registry / AI Requests / Sandbox
/// need an action pair (e.g., `+ New` plus a `Clear filters` icon)
/// rather than the single CTA the original `cta_label` prop supports.
/// `actions` complements (does NOT replace) `cta_label` so existing
/// callers keep working.
#[derive(Clone)]
pub struct QuickFilterAction {
    /// Visible button text. Short labels work best — `+`, `Clear`,
    /// `Export`, etc. For icon-style buttons the tooltip carries the
    /// full label.
    pub label: String,
    /// Optional `title=` attribute for hover tooltip. When the label
    /// is short or symbolic (`+`, `x`), set this for a11y + UX.
    pub tooltip: Option<String>,
    /// Click handler.
    pub on_click: Callback<()>,
    /// Optional Tailwind class override (defaults to a neutral outline).
    /// For the canonical green CTA prefer `cta_label` instead of this.
    pub class: Option<String>,
}

/// Canonical Quick Filter row: 1+ dropdowns, optional search input, optional
/// CTA button on the right.
///
/// The component renders a horizontal flex row at the top of the table
/// surface. Dropdown changes update each `selected` signal directly — the
/// page is responsible for re-running its list query when those signals
/// change (typically via a `LocalResource` that depends on the signal).
#[component]
pub fn QuickFilterRow(
    /// 1 or more dropdown filters. Order matters — leftmost is rendered first.
    dropdowns: Vec<QuickFilterDropdown>,
    /// Optional search input. The signal updates on every keystroke.
    #[prop(optional)]
    search: Option<RwSignal<String>>,
    /// Placeholder text for the search input.
    #[prop(optional, into)]
    search_placeholder: Option<String>,
    /// Optional CTA button label on the right (e.g., "+ New Rule").
    #[prop(optional, into)]
    cta_label: Option<String>,
    /// CTA click handler.
    #[prop(optional)]
    on_cta: Option<Callback<()>>,
    /// CTA color class override (defaults to emerald primary).
    #[prop(optional, into)]
    cta_class: Option<String>,
    /// Optional cluster of additional action buttons rendered to the
    /// LEFT of the primary `cta_label` button. Each gets its own
    /// `title=` tooltip and click handler. EUC-hh3j.44.
    #[prop(optional)]
    actions: Option<Vec<QuickFilterAction>>,
) -> impl IntoView {
    let cta_view = cta_label.map(|label| {
        let cls: &'static str = match cta_class.as_deref() {
            Some(c) if !c.is_empty() => Box::leak(c.to_string().into_boxed_str()),
            _ => "bg-emerald-500 hover:bg-emerald-600 text-white border-none",
        };
        view! {
            <Button
                size=ButtonSize::Sm
                class=cls
                on:click=move |_| {
                    if let Some(cb) = &on_cta {
                        cb.run(());
                    }
                }
            >
                {label}
            </Button>
        }
    });

    let actions_view = actions.map(|list| {
        list.into_iter()
            .map(|a| {
                let cls: &'static str = match a.class.as_deref() {
                    Some(c) if !c.is_empty() => Box::leak(c.to_string().into_boxed_str()),
                    _ => "btn-outline",
                };
                let tooltip = a.tooltip.unwrap_or_default();
                let cb = a.on_click;
                view! {
                    <Button
                        size=ButtonSize::Sm
                        class=cls
                        attr:title=tooltip
                        on:click=move |_| { cb.run(()); }
                    >
                        {a.label}
                    </Button>
                }
            })
            .collect::<Vec<_>>()
    });

    let search_view = search.map(|sig| {
        let placeholder = search_placeholder.unwrap_or_else(|| "Search\u{2026}".to_string());
        view! {
            <input
                type="text"
                class="input input-bordered input-sm w-64"
                placeholder=placeholder
                prop:value=move || sig.get()
                on:input=move |ev| sig.set(event_target_value(&ev))
            />
        }
    });

    view! {
        <div class="flex flex-wrap items-center gap-3">
            {dropdowns.into_iter().map(|d| {
                let selected = d.selected;
                view! {
                    <label class="flex flex-col gap-2">
                        <span class="text-xs text-base-content/60 mb-0.5">{d.label}</span>
                        <select
                            class="select select-bordered select-sm"
                            prop:value=move || selected.get()
                            on:change=move |ev| selected.set(event_target_value(&ev))
                        >
                            {d.options.into_iter().map(|(value, label)| {
                                view! {
                                    <option value=value>{label}</option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </label>
                }
            }).collect::<Vec<_>>()}
            {search_view}
            <div class="ml-auto flex items-center gap-2">
                {actions_view}
                {cta_view}
            </div>
        </div>
    }
}
