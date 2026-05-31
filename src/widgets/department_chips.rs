//! Reusable department-chip cluster (UT2-21 EUC-mz0z; reused by UT2-22 Standards).
//!
//! Renders a flex-wrapped row of outline `Badge` chips, one per
//! department, prefixed with a section heading. Designed for read-only
//! display in the right rail of the Policies / Standards screens; the
//! interactive variant lives in `tag_input.rs`.

use leptos::prelude::*;

use crate::components::{Badge, BadgeSize, BadgeStyle};

/// Renders a cluster of department-name chips with an optional heading.
///
/// Pass a non-empty `Vec<String>`. An empty list renders a muted
/// placeholder so the right-rail layout stays stable.
#[component]
pub fn DepartmentChips(
    /// Section heading (defaults to "Affected Departments").
    #[prop(into, default = "Affected Departments".to_string())]
    title: String,
    /// Department display names — render order is preserved.
    departments: Vec<String>,
) -> impl IntoView {
    let body_view = if departments.is_empty() {
        view! {
            <p class="text-xs text-base-content/50 italic">"No departments assigned."</p>
        }
        .into_any()
    } else {
        view! {
            <div class="flex flex-wrap gap-1.5">
                {departments
                    .into_iter()
                    .map(|d| view! {
                        <Badge style=BadgeStyle::Outline size=BadgeSize::Sm>{d}</Badge>
                    })
                    .collect::<Vec<_>>()}
            </div>
        }
        .into_any()
    };

    view! {
        <div class="space-y-2">
            <h4 class="font-semibold text-sm">{title}</h4>
            {body_view}
        </div>
    }
}
