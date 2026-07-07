use super::types::{
    EventLayout, HourFormat, SchedulerEvent, compute_event_layout, effective_height_px,
    minute_to_percent,
};
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Day Scheduler Component
///
/// A single-day vertical time grid: an hour gutter (12h/24h labels), hour
/// and half-hour gridlines, absolutely-timed event blocks laid into
/// side-by-side lanes when they overlap, and an optional "now" line. Ported
/// from d2d-ui's owner-drawn `DayScheduler` control -- the Direct2D
/// `rect`/brush/`draw()` painting is replaced by CSS absolute positioning
/// (percent-based `top`/`height`/`left`/`width`), and the overlap-lane
/// layout math in [`compute_event_layout`](super::types::compute_event_layout)
/// is carried over near-verbatim. This component has no internal clock --
/// pass a ticking `now_min` (e.g. derived from
/// [`use_sla_now`](crate::components::use_sla_now)) for a live "now" line.
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::components::{DayScheduler, SchedulerEvent, SchedulerEventColor};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let events = Signal::derive(|| {
///         vec![
///             SchedulerEvent::new("Standup", 9 * 60, 9 * 60 + 15, SchedulerEventColor::Primary),
///             SchedulerEvent::new("Design review", 9 * 60, 10 * 60, SchedulerEventColor::Accent),
///             SchedulerEvent::new("Lunch", 12 * 60, 13 * 60, SchedulerEventColor::Neutral),
///         ]
///     });
///
///     view! {
///         <DayScheduler
///             start_hour=8
///             end_hour=18
///             events=events
///             now_min=Some(10 * 60 + 30)
///             now_label="Now"
///         />
///     }
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex w-full relative w-12 shrink-0 border-r border-t border-base-300 border-base-300/40 flex-1 overflow-hidden");
/// @source inline("absolute inset-x-0 right-2 -translate-y-1/2 whitespace-nowrap z-10 items-center pointer-events-none");
/// @source inline("m-px rounded-sm border-l-4 p-1 truncate font-medium text-xs opacity-60");
/// @source inline("-ml-1 h-2 w-2 rounded-full h-px ml-1 shrink-0 font-medium text-error bg-error");
/// @source inline("bg-neutral/15 bg-primary/15 bg-secondary/15 bg-accent/15 bg-info/15 bg-success/15 bg-warning/15 bg-error/15");
/// @source inline("border-neutral border-primary border-secondary border-accent border-info border-success border-warning border-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the wrapping `div` element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn DayScheduler(
    /// First hour shown (0-23). Defaults to `0` (midnight).
    #[prop(optional, into, default = Signal::derive(|| 0))]
    start_hour: Signal<u32>,

    /// Last hour shown (1-24; coerced to at least `start_hour + 1`).
    /// Defaults to `24` (midnight).
    #[prop(optional, into, default = Signal::derive(|| 24))]
    end_hour: Signal<u32>,

    /// Scheduled events for the day.
    #[prop(optional, into)]
    events: Signal<Vec<SchedulerEvent>>,

    /// Hour-label clock format (24h default).
    #[prop(optional, into)]
    hour_format: Signal<HourFormat>,

    /// Optional "now" marker, in minutes from midnight. This component
    /// keeps no internal timer -- pair it with a ticking `Signal`, e.g.
    /// [`use_sla_now`](crate::components::use_sla_now) converted to minutes,
    /// or any other externally-driven clock signal.
    #[prop(optional, into)]
    now_min: Signal<Option<u32>>,

    /// Optional inline label drawn beside the now-line (e.g. `"Now"`).
    /// Shown only when `now_min` is `Some` and this is non-empty.
    #[prop(optional, into)]
    now_label: Signal<String>,

    /// Height of the grid, in pixels. `0.0` (the default) auto-computes 60px
    /// per displayed hour.
    #[prop(optional, into, default = Signal::derive(|| 0.0))]
    height_px: Signal<f64>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the wrapping div element
    #[prop(optional)]
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let hours = Signal::derive(move || {
        let s = start_hour.get();
        let e = end_hour.get().max(s + 1);
        (s..=e).collect::<Vec<u32>>()
    });

    // Hours that get an additional (fainter) half-hour line -- every
    // displayed hour except the last.
    let half_hours = Signal::derive(move || {
        let s = start_hour.get();
        let e = end_hour.get().max(s + 1);
        (s..e).collect::<Vec<u32>>()
    });

    let grid_height =
        move || effective_height_px(height_px.get(), start_hour.get(), end_hour.get());

    // Index-tagged (event, layout) pairs so <For> has a stable key even
    // though the layout values themselves are plain (non-reactive) data
    // recomputed whenever `events`/`start_hour`/`end_hour` change.
    let paired_events = Signal::derive(move || {
        let evs = events.get();
        let layouts = compute_event_layout(&evs, start_hour.get(), end_hour.get());
        evs.into_iter()
            .zip(layouts)
            .enumerate()
            .collect::<Vec<(usize, (SchedulerEvent, EventLayout))>>()
    });

    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex w-full", class)
        >
            // Hour gutter.
            <div
                class="relative w-12 shrink-0 border-r border-base-300"
                style:height=move || format!("{}px", grid_height())
            >
                <For
                    each=move || hours.get()
                    key=|h| *h
                    children=move |hour| {
                        view! {
                            <div
                                class="absolute right-2 -translate-y-1/2 whitespace-nowrap text-xs opacity-60"
                                style:top=move || {
                                    format!(
                                        "{}%",
                                        minute_to_percent(hour as f64 * 60.0, start_hour.get(), end_hour.get()),
                                    )
                                }
                            >
                                {move || hour_format.get().label(hour)}
                            </div>
                        }
                    }
                />
            </div>

            // Content column: gridlines, event blocks, now-line.
            <div
                class="relative flex-1 overflow-hidden"
                style:height=move || format!("{}px", grid_height())
            >
                // Hour gridlines.
                <For
                    each=move || hours.get()
                    key=|h| *h
                    children=move |hour| {
                        view! {
                            <div
                                class="absolute inset-x-0 border-t border-base-300"
                                style:top=move || {
                                    format!(
                                        "{}%",
                                        minute_to_percent(hour as f64 * 60.0, start_hour.get(), end_hour.get()),
                                    )
                                }
                            ></div>
                        }
                    }
                />

                // Half-hour gridlines (fainter).
                <For
                    each=move || half_hours.get()
                    key=|h| *h
                    children=move |hour| {
                        view! {
                            <div
                                class="absolute inset-x-0 border-t border-base-300/40"
                                style:top=move || {
                                    format!(
                                        "{}%",
                                        minute_to_percent(
                                            hour as f64 * 60.0 + 30.0,
                                            start_hour.get(),
                                            end_hour.get(),
                                        ),
                                    )
                                }
                            ></div>
                        }
                    }
                />

                // Event blocks.
                <For
                    each=move || paired_events.get()
                    key=|(idx, (ev, layout))| {
                        (
                            *idx,
                            ev.start_min,
                            ev.end_min,
                            ev.title.clone(),
                            format!("{:?}", ev.color),
                            layout.left_pct.to_bits(),
                            layout.width_pct.to_bits(),
                        )
                    }
                    children=move |(_, (ev, layout))| {
                        view! {
                            <div
                                class=merge_classes!(
                                    "absolute m-px overflow-hidden rounded-sm border-l-4 p-1 text-xs",
                                    ev.color.bg_class(),
                                    ev.color.border_class()
                                )
                                style:top=format!("{}%", layout.top_pct)
                                style:height=format!("{}%", layout.height_pct)
                                style:left=format!("{}%", layout.left_pct)
                                style:width=format!("{}%", layout.width_pct)
                                title=ev.title.clone()
                            >
                                <span class="truncate font-medium">{ev.title.clone()}</span>
                            </div>
                        }
                    }
                />

                // "Now" line -- rule + marker dot + optional inline label.
                <Show when=move || now_min.get().is_some()>
                    <div
                        class="pointer-events-none absolute inset-x-0 z-10 flex items-center"
                        style:top=move || {
                            format!(
                                "{}%",
                                minute_to_percent(
                                    now_min.get().unwrap_or(0) as f64,
                                    start_hour.get(),
                                    end_hour.get(),
                                ),
                            )
                        }
                    >
                        <span class="-ml-1 h-2 w-2 shrink-0 rounded-full bg-error"></span>
                        <span class="h-px flex-1 bg-error"></span>
                        <Show when=move || !now_label.get().is_empty()>
                            <span class="ml-1 shrink-0 text-xs font-medium text-error">
                                {move || now_label.get()}
                            </span>
                        </Show>
                    </div>
                </Show>
            </div>
        </div>
    }
}
