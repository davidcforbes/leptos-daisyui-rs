//! Compliance Checklist sidebar (UT2-16 / EUC-s35j).
//!
//! Renders a list of compliance controls with status icons (pass / warn /
//! fail / na) plus an overall percentage progress bar.
//!
//! Reusable across:
//!   - /requests/security  — current home (UT2-16)
//!   - /standards          — per-standard control breakdown (future)
//!   - /policies           — same pattern for policy attestation status

use leptos::prelude::*;

use crate::components::{Card, CardBody, Progress, ProgressColor};

/// One row of the checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplianceChecklistItem {
    pub label: String,
    /// "pass" | "warn" | "fail" | "na"
    pub status: String,
}

fn status_glyph(status: &str) -> (&'static str, &'static str) {
    match status {
        "pass" => ("\u{2713}", "text-emerald-500"),
        "warn" => ("\u{25B3}", "text-amber-500"),
        "fail" => ("\u{2717}", "text-red-500"),
        _ => ("\u{2014}", "text-base-content/40"),
    }
}

/// Picks the daisyUI Progress colour from the overall percentage.
fn progress_color(pct: f64) -> ProgressColor {
    if pct >= 90.0 {
        ProgressColor::Success
    } else if pct >= 70.0 {
        ProgressColor::Warning
    } else {
        ProgressColor::Error
    }
}

#[component]
pub fn ComplianceChecklist(
    #[prop(into, default = "Compliance Checklist".into())] title: String,
    items: Vec<ComplianceChecklistItem>,
    /// Overall pass percentage (0-100). The component clamps to this range.
    overall_pct: f64,
) -> impl IntoView {
    let pct = overall_pct.clamp(0.0, 100.0);
    let passed = items.iter().filter(|i| i.status == "pass").count();
    let total = items.len();
    let color = progress_color(pct);

    view! {
        <Card class="shadow-sm">
            <CardBody class="p-4 space-y-3">
                <div class="flex items-center justify-between">
                    <h3 class="font-semibold text-sm">{title}</h3>
                    <span class="text-xs text-base-content/50">{format!("{passed}/{total} passed")}</span>
                </div>

                <div class="space-y-2">
                    {items.into_iter().map(|item| {
                        let (icon, color) = status_glyph(&item.status);
                        view! {
                            <div class="flex items-center gap-3 py-1">
                                <span class=format!("text-lg font-bold {color} w-5 text-center")>{icon}</span>
                                <span class="text-sm text-base-content/80">{item.label}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>

                <div class="space-y-1">
                    <div class="flex items-center justify-between text-xs text-base-content/60">
                        <span>"Overall"</span>
                        <span class="font-mono">{format!("{:.1}%", pct)}</span>
                    </div>
                    <Progress color=color value=pct max=100.0 />
                </div>
            </CardBody>
        </Card>
    }
}
