use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, ButtonSize, StatDeltaTrend};
use leptos_daisyui_rs::patterns::{KpiItem, KpiStatus, KpiStrip, KpiStripTexts, KpiTrend};

fn office_kpis() -> Vec<KpiItem> {
    vec![
        KpiItem::new("open-matters", "Open matters", "128")
            .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
        KpiItem::new("overdue-tasks", "Overdue tasks", "6")
            .status(KpiStatus::Warning)
            .help("Tasks past their due date, across every assignee."),
        KpiItem::new("new-leads", "New leads", "23")
            .status(KpiStatus::Info)
            .description("Last 7 days"),
        KpiItem::new("revenue-booked", "Revenue booked", "$18,400")
            .status(KpiStatus::Success)
            .trend(KpiTrend::new(12.5, StatDeltaTrend::Positive).label("vs last month")),
        KpiItem::new("sla-breaches", "SLA breaches", "2")
            .status(KpiStatus::Error)
            .trend(KpiTrend::new(1.0, StatDeltaTrend::Negative)),
        KpiItem::new("avg-response", "Avg. response time", "3h 12m"),
        KpiItem::new("client-satisfaction", "Client satisfaction", "94%")
            .status(KpiStatus::Success),
        KpiItem::new("last-sync", "Last sync", "").unavailable(),
    ]
}

#[component]
pub fn KpiStripDemo() -> impl IntoView {
    let (french, set_french) = signal(false);

    let localized_items = Signal::derive(move || {
        if french.get() {
            vec![
                KpiItem::new("open-matters", "Dossiers ouverts", "128")
                    .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("cette semaine")),
                KpiItem::new("overdue-tasks", "Taches en retard", "6")
                    .status(KpiStatus::Warning)
                    .help("Taches dont l'echeance est depassee."),
                KpiItem::new("revenue-booked", "Revenu enregistre", "18 400 $")
                    .status(KpiStatus::Success),
            ]
        } else {
            vec![
                KpiItem::new("open-matters", "Open matters", "128")
                    .trend(KpiTrend::new(4.0, StatDeltaTrend::Positive).label("this week")),
                KpiItem::new("overdue-tasks", "Overdue tasks", "6")
                    .status(KpiStatus::Warning)
                    .help("Tasks past their due date, across every assignee."),
                KpiItem::new("revenue-booked", "Revenue booked", "$18,400")
                    .status(KpiStatus::Success),
            ]
        }
    });

    let localized_texts = Signal::derive(move || {
        if french.get() {
            KpiStripTexts {
                unavailable: "Indisponible".to_string(),
                trend_up: "en hausse".to_string(),
                trend_down: "en baisse".to_string(),
                trend_steady: "stable".to_string(),
            }
        } else {
            KpiStripTexts::default()
        }
    });

    view! {
        <ContentLayout
            title="KPI Strip"
            description="A responsive row of independent stat cards -- the opinionated replacement for composing Stat children inside daisyUI's low-level, visually-joined Stats container."
        >
            <Section title="Eight KPIs, wide-to-narrow responsive">
                <div data-testid="kpi-strip-wide">
                    <KpiStrip items=Signal::derive(office_kpis) />
                </div>
            </Section>

            <Section title="Compact">
                <div data-testid="kpi-strip-compact">
                    <KpiStrip items=Signal::derive(office_kpis) compact=true />
                </div>
            </Section>

            <Section title="Long copy stays legible">
                <div class="max-w-2xl" data-testid="kpi-strip-long-copy">
                    <KpiStrip items=Signal::derive(|| {
                        vec![
                            KpiItem::new(
                                "outstanding-invoices",
                                "Outstanding invoices across every active matter this quarter",
                                "$482,900",
                            )
                                .description(
                                    "Includes invoices issued in the last ninety days across every \
                                     practice group, net of write-offs and pending trust transfers.",
                                )
                                .status(KpiStatus::Warning),
                            KpiItem::new("disputed-line-items", "Disputed line items still under review", "14"),
                        ]
                    }) />
                </div>
            </Section>

            <Section title="Localized (reactive)">
                <div class="flex flex-col gap-2" data-testid="kpi-strip-localized">
                    <Button
                        size=ButtonSize::Sm
                        class="self-start"
                        attr:data-testid="kpi-strip-localized-toggle"
                        on:click=move |_| set_french.update(|v| *v = !*v)
                    >
                        "Toggle language"
                    </Button>
                    <KpiStrip items=localized_items texts=localized_texts />
                </div>
            </Section>
        </ContentLayout>
    }
}
