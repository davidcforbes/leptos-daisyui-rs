use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
// The SVG chart family lives in `charts`, not `components`. Imported by name
// rather than glob because `charts` also exports a `Sparkline`.
use leptos_daisyui_rs::charts::BarChart;
use leptos_daisyui_rs::components::*;

#[component]
pub fn SparklineDemo() -> impl IntoView {
    // Static sample series shared by the "basic" and "colors" sections. Kept
    // as `RwSignal`s (which are `Copy`) so they can be read from multiple
    // `move` closures below without fighting the borrow checker over a
    // shared `Vec<f32>`.
    let throughput = RwSignal::new(vec![
        12.0, 18.0, 15.0, 22.0, 19.0, 27.0, 24.0, 31.0, 28.0, 35.0,
    ]);
    let cpu = RwSignal::new(vec![40.0, 55.0, 48.0, 62.0, 58.0, 45.0, 50.0, 47.0]);
    let flat = RwSignal::new(vec![0.0, 0.0, 0.0, 0.0, 0.0]);
    let no_data = RwSignal::new(Vec::<f32>::new());

    // Live-updating series to demonstrate the Signal-based `samples` prop --
    // a new sample is appended (and the oldest dropped) once per second.
    let live_samples = RwSignal::new(vec![5.0, 8.0, 6.0, 9.0]);
    Effect::new(move |_| {
        let handle = leptos::leptos_dom::helpers::set_interval_with_handle(
            move || {
                live_samples.update(|s| {
                    let next = (js_sys::Math::random() * 40.0) as f32 + 5.0;
                    s.push(next);
                    if s.len() > 20 {
                        s.remove(0);
                    }
                });
            },
            std::time::Duration::from_millis(1000),
        );

        if let Ok(h) = handle {
            on_cleanup(move || {
                h.clear();
            });
        }
    });

    view! {
        <ContentLayout
            title="Sparkline"
            description="A small time-series line chart -- an inline SVG polyline over a baseline, with an optional framed card and current/peak readout row."
        >
            <Section title="Basic (framed, with readout)" row=true>
                <Sparkline
                    samples=Signal::derive(move || throughput.get())
                    title="Throughput"
                    unit="KB/s"
                    color=SparklineColor::Primary
                    class="w-56"
                />
                <Sparkline
                    samples=Signal::derive(move || cpu.get())
                    title="CPU"
                    unit="%"
                    color=SparklineColor::Accent
                    class="w-56"
                />
            </Section>

            <Section title="Colors" row=true>
                <Sparkline samples=Signal::derive(move || throughput.get()) title="Default" class="w-48" />
                <Sparkline
                    samples=Signal::derive(move || throughput.get())
                    title="Success"
                    color=SparklineColor::Success
                    class="w-48"
                />
                <Sparkline
                    samples=Signal::derive(move || throughput.get())
                    title="Warning"
                    color=SparklineColor::Warning
                    class="w-48"
                />
                <Sparkline
                    samples=Signal::derive(move || throughput.get())
                    title="Error"
                    color=SparklineColor::Error
                    class="w-48"
                />
            </Section>

            <Section title="Flat / empty series sit on the baseline" row=true>
                <Sparkline samples=Signal::derive(move || flat.get()) title="Idle" unit="req/s" class="w-48" />
                <Sparkline samples=Signal::derive(move || no_data.get()) title="No data" class="w-48" />
            </Section>

            <Section title="Inline / unframed (e.g. inside a table cell)">
                <div class="overflow-x-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Host"</th>
                                <th>"Trend"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>"web-01"</td>
                                <td class="w-32">
                                    <Sparkline
                                        samples=Signal::derive(move || throughput.get())
                                        framed=false
                                        color=SparklineColor::Info
                                        width=80.0
                                        height=24.0
                                    />
                                </td>
                            </tr>
                            <tr>
                                <td>"web-02"</td>
                                <td class="w-32">
                                    <Sparkline
                                        samples=Signal::derive(move || cpu.get())
                                        framed=false
                                        color=SparklineColor::Secondary
                                        width=80.0
                                        height=24.0
                                    />
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </Section>

            <Section title="Live updating">
                <Sparkline
                    samples=Signal::derive(move || live_samples.get())
                    title="Live Metric"
                    unit="ms"
                    color=SparklineColor::Primary
                    class="w-72"
                />
            </Section>

            <Section title="BarChart — per-bar color (ldui-jm6)" col=true>
                <p class="text-sm opacity-70">
                    "bar_colors is an optional list positionally parallel to data, so each bar can carry its own judgement — weeks at or above target in success, weeks behind in error — instead of painting the whole chart by the series' majority state. The list need not match data in length: short lists fall back to color, surplus entries are ignored, and the bar count always comes from data."
                </p>
                <div class="w-full max-w-xl">
                    <BarChart
                        data=closed_by_week()
                        bar_colors=closed_by_week_colors()
                        height=180
                    />
                </div>

                <p class="text-sm opacity-70">
                    "Two overrides against a four-bar series — bars 3 and 4 fall back to the chart-wide color rather than panicking or vanishing:"
                </p>
                <div class="w-full max-w-xl">
                    <BarChart
                        data=vec![
                            ("Mon".to_string(), 4.0),
                            ("Tue".to_string(), 7.0),
                            ("Wed".to_string(), 5.0),
                            ("Thu".to_string(), 9.0),
                        ]
                        bar_colors=vec![
                            "var(--color-error)".to_string(),
                            "var(--color-success)".to_string(),
                        ]
                        height=180
                    />
                </div>
            </Section>

        </ContentLayout>
    }
}

/// Weekly closed-work counts for the per-bar-color example.
fn closed_by_week() -> Vec<(String, f64)> {
    vec![
        ("W31".to_string(), 18.0),
        ("W32".to_string(), 24.0),
        ("W33".to_string(), 11.0),
        ("W34".to_string(), 27.0),
        ("W35".to_string(), 9.0),
    ]
}

/// Per-bar judgement for [`closed_by_week`]: at or above the target of 20 is
/// favourable, below it is not. One colour per bar, derived from the same
/// comparison a consumer would already be making.
fn closed_by_week_colors() -> Vec<String> {
    closed_by_week()
        .iter()
        .map(|(_, v)| {
            if *v >= 20.0 {
                "var(--color-success)".to_string()
            } else {
                "var(--color-error)".to_string()
            }
        })
        .collect()
}
