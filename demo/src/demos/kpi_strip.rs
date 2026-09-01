use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, ButtonSize, StatDeltaTrend};
use leptos_daisyui_rs::patterns::{
    KpiAction, KpiBaseline, KpiItem, KpiStatus, KpiStrip, KpiStripTexts, KpiTrend,
};

/// The twelve production KPIs the consumer's Dashboard renders (ldui-ztgo),
/// reproduced with framework props only -- no local card component, no
/// consumer grid, no consumer CSS.
///
/// Deliberately mixed, because a real dashboard is: cards with and without a
/// baseline, above/level/below, a metric with no baseline at all, one whose
/// trailing window is still filling, an unavailable value, one whose current
/// value runs clean past the top of its track, and one whose declared
/// baseline is zero.
fn dashboard_kpis() -> Vec<KpiItem> {
    let detail = || KpiAction::new("View details");
    vec![
        // Above baseline. The bead's own worked example: 112%, 12% above.
        KpiItem::new("intakes", "Intakes", "280")
            .status(KpiStatus::Success)
            .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg / 250"))
            .action(detail()),
        // Below baseline.
        KpiItem::new("closes", "Closes", "5,739")
            .status(KpiStatus::Warning)
            .baseline(KpiBaseline::against(5739.0, 6705.0).label("12-week avg / 6,705"))
            .action(detail()),
        // Exactly on baseline -- "in line with", never "0% above".
        KpiItem::new("signed-retainers", "Signed retainers", "412")
            .baseline(KpiBaseline::against(412.0, 412.0).label("12-week avg / 412"))
            .action(detail()),
        // Well past the top of the track: the bar saturates, the readout
        // does not.
        KpiItem::new("web-leads", "Web leads", "780")
            .status(KpiStatus::Info)
            .baseline(KpiBaseline::against(780.0, 250.0).label("12-week avg / 250"))
            .action(detail()),
        // Activity-neutral: a baseline, no status emphasis, no action.
        KpiItem::new("calls-logged", "Calls logged", "1,204")
            .baseline(KpiBaseline::against(1204.0, 1180.0).label("12-week avg / 1,180")),
        // A brand-new metric: there is no baseline to compare against.
        KpiItem::new("referrals", "Referrals", "12")
            .baseline(KpiBaseline::absent(12.0).label("12-week avg"))
            .action(detail()),
        // The window exists but is not yet full.
        KpiItem::new("retention", "Retention", "88%")
            .baseline(KpiBaseline::settling(88.0).label("12-week avg"))
            .action(detail()),
        // A declared baseline of zero: no bar, no fabricated percentage, and
        // the card says so rather than printing "inf%".
        KpiItem::new("appeals-filed", "Appeals filed", "3")
            .baseline(KpiBaseline::against(3.0, 0.0).label("12-week avg / 0")),
        // Off target, with help copy competing for the label's width.
        KpiItem::new("sla-breaches", "SLA breaches", "9")
            .status(KpiStatus::Error)
            .baseline(KpiBaseline::against(9.0, 4.0).label("12-week avg / 4"))
            .help("Matters that passed their response deadline this week.")
            .action(detail()),
        // No baseline row at all: unchanged from before ldui-ztgo.
        KpiItem::new("avg-response", "Avg. response time", "3h 12m")
            .trend(KpiTrend::new(6.0, StatDeltaTrend::Negative).label("vs last week")),
        // An action a caller has temporarily withdrawn.
        KpiItem::new("payments-collected", "Payments collected", "$92,400")
            .status(KpiStatus::Success)
            .baseline(KpiBaseline::against(92400.0, 81000.0).label("12-week avg / $81,000"))
            .action(KpiAction::new("View details").disabled(true)),
        // Unavailable value, still aligned with its neighbours.
        KpiItem::new("last-sync", "Last sync", "").unavailable(),
    ]
}

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
    // The framework emits the stable `KpiItem::id` and nothing else. A real
    // consumer maps it to a route and a selected scope; the showcase just
    // records it, which is also what the browser proof reads.
    let (activated, set_activated) = signal(String::new());
    let on_activate = Callback::new(move |id: String| set_activated.set(id));

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
                KpiItem::new("intakes", "Admissions", "280")
                    .baseline(KpiBaseline::against(280.0, 250.0).label("moyenne 12 semaines"))
                    .action(KpiAction::new("Voir le detail")),
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
                KpiItem::new("intakes", "Intakes", "280")
                    .baseline(KpiBaseline::against(280.0, 250.0).label("12-week avg"))
                    .action(KpiAction::new("View details")),
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
                // Every comparison sentence is framework-owned copy, so a
                // locale switch reaches it without rebuilding the items.
                baseline_ratio: "{ratio} %".to_string(),
                baseline_above: "{delta} % au-dessus de la {baseline}".to_string(),
                baseline_below: "{delta} % en dessous de la {baseline}".to_string(),
                baseline_level: "Conforme a la {baseline}".to_string(),
                baseline_absent: "Pas encore de reference".to_string(),
                baseline_settling: "Reference en cours de constitution".to_string(),
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
            <Section title="Twelve KPIs with typed baselines and activation">
                <div class="flex flex-col gap-4" data-testid="kpi-strip-dashboard">
                    <p class="ld-text-small text-base-content/75">
                        "Bounded comparison bar, fixed baseline marker at 80% of every track, and a truthful readout that keeps reporting the real ratio after the bar has run out of room. Mixed on purpose: above, level, below, saturated, no baseline, still settling, a zero baseline, a disabled action, and an unavailable value."
                    </p>
                    <KpiStrip
                        items=Signal::derive(dashboard_kpis)
                        on_activate=on_activate
                    />
                    <p class="ld-text-small text-base-content/75">
                        "Last activated: "
                        <span
                            class="font-semibold tabular-nums"
                            data-testid="kpi-strip-dashboard-activated"
                        >
                            {move || activated.get()}
                        </span>
                    </p>
                </div>
            </Section>

            <Section title="Compact, with comparisons">
                <div data-testid="kpi-strip-dashboard-compact">
                    <KpiStrip
                        items=Signal::derive(dashboard_kpis)
                        compact=true
                        on_activate=on_activate
                    />
                </div>
            </Section>

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
