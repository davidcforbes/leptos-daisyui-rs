//! `MonthRoster`'s Day zoom (ldui-5lmt.5): a shift coverage timeline (one
//! bar per working person on a shared hour axis) and who is out, grouped by
//! kind. Bars are positioned in percent of the axis, so the pane can resize.

use super::component::{Chip, Ctx, DayHead, DayStatus};
use super::style::{ODD_HOUR, RosterZoom, TIMELINE_ROW};
use super::texts::fill;
use super::types::{
    EntryGroup, RosterEntry, absence_groups, bar_geometry, clock, iso_date, sorted_entries,
    timeline_axis,
};
use crate::merge_classes;
use leptos::prelude::*;

/// The anchor day, rebuilt when the anchor moves (prev / next by one day).
#[component]
pub(super) fn DayZoom(ctx: Ctx) -> impl IntoView {
    move || {
        let day = ctx.anchor.get();
        view! { <DayBody ctx=ctx day=day /> }
    }
}

#[component]
fn DayBody(ctx: Ctx, day: i64) -> impl IntoView {
    let data = Memo::new(move |_| ctx.day_data(day));
    let sorted = Memo::new(move |_| {
        data.with(|d| {
            d.as_ref()
                .map(|d| sorted_entries(&d.entries))
                .unwrap_or_default()
        })
    });
    let working = move |timed: bool| {
        move || {
            sorted.with(|entries| {
                entries
                    .iter()
                    .filter(|e| e.kind.group() == EntryGroup::Working)
                    .filter(|e| e.time_span().is_some() == timed)
                    .cloned()
                    .collect::<Vec<_>>()
            })
        }
    };
    let timed = working(true);
    let untimed = working(false);
    let has_working = move || {
        sorted.with(|entries| {
            entries
                .iter()
                .any(|e| e.kind.group() == EntryGroup::Working)
        })
    };
    let axis = Memo::new(move |_| sorted.with(|entries| timeline_axis(entries)));
    let hours = move || {
        let (lo, hi) = axis.get();
        (lo / 60..=hi / 60).collect::<Vec<u16>>()
    };
    let groups = Memo::new(move |_| sorted.with(|entries| absence_groups(entries)));
    let out_total =
        move || groups.with(|g| g.iter().map(|(_, people)| people.len()).sum::<usize>());
    let names_of = move |group: EntryGroup| {
        sorted.with(|entries| {
            entries
                .iter()
                .filter(|e| e.kind.group() == group)
                .map(|e| e.person.clone())
                .collect::<Vec<_>>()
                .join(", ")
        })
    };
    let is_today = move || ctx.today.get() == day;
    // The now line: only on today, only with a time, only inside the axis.
    let now_pct = Memo::new(move |_| {
        let (lo, hi) = axis.get();
        ctx.now_min
            .get()
            .filter(|now| is_today() && (lo..=hi).contains(now))
            .map(|now| bar_geometry(now, now, (lo, hi)).0)
    });
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" && ctx.navigable() {
            ev.prevent_default();
            ctx.navigate(RosterZoom::Week, day);
        }
    };
    let coverage_id = ctx.id("coverage");
    let out_id = ctx.id("out-today");
    let always = Signal::stored(true);
    let chip = move |entry: RosterEntry| view! { <Chip ctx=ctx entry=entry in_tab_order=always prefer_times=true /> };
    let last_hour = move || axis.get().1 / 60;

    view! {
        <div
            data-day-view=""
            data-day=iso_date(day)
            data-today=move || is_today().then_some("")
            class="flex flex-col gap-4"
            on:keydown=on_keydown
        >
            <div class="flex flex-col gap-1">
                <DayHead ctx=ctx day=day data=data tabbable=false no_number=true />
                <DayStatus ctx=ctx data=data />
            </div>
            <Show
                when=has_working
                fallback=move || {
                    sorted
                        .with(|entries| !entries.is_empty())
                        .then(|| {
                            view! {
                                <p data-day-no-working="" class="text-sm">
                                    {move || ctx.texts.with(|t| t.no_one_working.clone())}
                                </p>
                            }
                        })
                }
            >
                <section
                    data-day-coverage=""
                    aria-labelledby=coverage_id.clone()
                    class="flex flex-col gap-2"
                >
                    <h3 id=coverage_id.clone() class="text-sm font-semibold">
                        {move || ctx.texts.with(|t| t.coverage.clone())}
                    </h3>
                    <Show when=move || !untimed().is_empty()>
                        <div data-day-all-day="" class="flex flex-col gap-1">
                            <h4 class="text-xs font-semibold">
                                {move || ctx.texts.with(|t| t.all_day.clone())}
                            </h4>
                            <ul class="flex flex-wrap gap-1">
                                <For each=untimed key=|entry| entry.content_key() children=chip />
                            </ul>
                        </div>
                    </Show>
                    <div
                        data-day-timeline=""
                        class=move || merge_classes!("grid gap-x-2", ctx.layout_classes(TIMELINE_ROW))
                    >
                        <span aria-hidden="true"></span>
                        <div data-day-axis="" aria-hidden="true" class="relative h-6">
                            <For
                                each=hours
                                key=|hour| *hour
                                children=move |hour| {
                                    let left = move || {
                                        let pct = bar_geometry(hour * 60, hour * 60, axis.get()).0;
                                        format!("left: {pct}%")
                                    };
                                    let label_class = move || {
                                        let (lo, _) = axis.get();
                                        let anchor = if hour * 60 == lo {
                                            "translate-x-0"
                                        } else if hour == last_hour() {
                                            "-translate-x-full"
                                        } else {
                                            "-translate-x-1/2"
                                        };
                                        let odd = if hour % 2 == 1 { ctx.layout_classes(ODD_HOUR) } else { "" };
                                        format!(
                                            "absolute bottom-2 left-0 whitespace-nowrap text-xs text-base-content/75 {anchor} {odd}"
                                        )
                                    };
                                    view! {
                                        <span
                                            data-hour=hour.to_string()
                                            class="absolute bottom-0 h-2 w-0 border-l border-base-300"
                                            style=left
                                        >
                                            <span class=label_class>{clock(hour * 60)}</span>
                                        </span>
                                    }
                                }
                            />
                        </div>
                        <For
                            each=timed
                            key=|entry| entry.content_key()
                            children=move |entry| {
                                view! { <TimelineRow ctx=ctx entry=entry axis=axis now_pct=now_pct /> }
                            }
                        />
                    </div>
                </section>
            </Show>
            <Show when=move || { out_total() > 0 }>
                <section data-day-out="" aria-labelledby=out_id.clone() class="flex flex-col gap-2">
                    <h3 id=out_id.clone() class="text-sm font-semibold">
                        {move || {
                            ctx.texts
                                .with(|t| fill(&t.out_today, &[("count", &out_total().to_string())]))
                        }}
                    </h3>
                    <For
                        each=move || groups.get()
                        key=|(kind, people)| {
                            (*kind, people.iter().map(RosterEntry::content_key).collect::<Vec<_>>())
                        }
                        children=move |(kind, people)| {
                            let count = people.len();
                            view! {
                                <div data-out-group=kind.as_str() class="flex flex-col gap-1">
                                    <h4 class="text-xs font-semibold">
                                        {move || {
                                            ctx.texts
                                                .with(|t| format!("{} ({count})", t.absence_word(kind)))
                                        }}
                                    </h4>
                                    <ul class="flex flex-wrap gap-1">
                                        {people.into_iter().map(chip).collect_view()}
                                    </ul>
                                </div>
                            }
                        }
                    />
                </section>
            </Show>
            <Show when=move || !names_of(EntryGroup::Holiday).is_empty()>
                <p data-day-holiday-line="" class="text-sm">
                    {move || {
                        ctx.texts
                            .with(|t| fill(&t.holiday_line, &[("names", &names_of(EntryGroup::Holiday))]))
                    }}
                </p>
            </Show>
            <Show when=move || !names_of(EntryGroup::Off).is_empty()>
                <p data-day-off-line="" class="text-sm">
                    {move || {
                        ctx.texts.with(|t| fill(&t.off_line, &[("names", &names_of(EntryGroup::Off))]))
                    }}
                </p>
            </Show>
        </div>
    }
}

/// One working person: the name, then a track holding their bar (and a
/// segment of the now line; the rows have no gap, so the segments join).
#[component]
fn TimelineRow(
    ctx: Ctx,
    entry: RosterEntry,
    axis: Memo<(u16, u16)>,
    now_pct: Memo<Option<f64>>,
) -> impl IntoView {
    let (start, end) = entry.time_span().unwrap_or((0, 1));
    let geometry = move || {
        let (left, width) = bar_geometry(start, end, axis.get());
        format!("left: {left}%; width: {width}%")
    };
    let range = move || ctx.texts.with(|t| t.time_range_of(start, end));
    let spoken = {
        let person = entry.person.clone();
        move || {
            ctx.texts.with(|t| {
                fill(
                    &t.working_spoken,
                    &[
                        ("person", &person),
                        ("start", &clock(start)),
                        ("end", &clock(end)),
                    ],
                )
            })
        }
    };
    let activate = ctx.on_entry_activate;
    let id = entry.id.clone();
    let bar_class = format!(
        "absolute inset-y-1 flex items-center overflow-hidden rounded-sm border px-1 text-xs {}",
        entry.kind.colour_classes()
    );
    view! {
        <span translate="no" data-day-row-name="" class="truncate text-sm leading-7">
            {entry.person.clone()}
        </span>
        <div data-day-track="" class="relative h-7">
            <div
                data-day-bar=entry.id.clone()
                data-entry-state=entry.kind.slug()
                class=bar_class
                style=geometry
            >
                <span class="truncate" aria-hidden=activate.is_some().then_some("true")>
                    {range}
                </span>
                {activate
                    .map(|activate| {
                        view! {
                            <button
                                type="button"
                                data-entry-open=""
                                data-pressable="true"
                                aria-label=spoken
                                class="ld-focus-ring absolute inset-0 cursor-pointer rounded-sm"
                                on:click=move |_| activate.run(id.clone())
                            ></button>
                        }
                    })}
            </div>
            {move || {
                now_pct
                    .get()
                    .map(|pct| {
                        view! {
                            <div
                                data-now-line=""
                                aria-hidden="true"
                                class="pointer-events-none absolute inset-y-0 border-l-2 border-error"
                                style=format!("left: {pct}%")
                            ></div>
                        }
                    })
            }}
        </div>
    }
}
