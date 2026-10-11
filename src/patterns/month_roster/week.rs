//! `MonthRoster`'s Week zoom (ldui-5lmt.4): seven day columns, each listing
//! EVERY entry of its day. Deliberately not an hour grid: a staffing week
//! has 20+ people, and an hour grid with that many lanes is unreadable. It
//! reuses WeekView's date math, not its grid.

use super::component::{Chip, Ctx, DayHead, DayStatus};
use super::style::{DAY_CLOSED, DAY_HOLIDAY, DAY_TODAY, DAY_WEEKEND, RosterZoom, WEEK_COLUMNS};
use super::texts::fill;
use super::types::{EntryGroup, RosterEntry, iso_date, sorted_entries};
use crate::components::{Weekday, week_start_for_weekday};
use crate::merge_classes;
use leptos::prelude::*;

/// The seven columns of the anchor's week. Escape anywhere inside returns
/// to the month.
#[component]
pub(super) fn WeekZoom(ctx: Ctx) -> impl IntoView {
    let start = Memo::new(move |_| week_start_for_weekday(ctx.anchor.get(), ctx.week_start.get()));
    let days = move || (0..7).map(|i| start.get() + i).collect::<Vec<i64>>();
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" && ctx.navigable() {
            ev.prevent_default();
            ctx.navigate(RosterZoom::Month, ctx.anchor.get_untracked());
        }
    };
    view! {
        <div
            data-week-view=""
            class=move || merge_classes!("grid gap-2", ctx.layout_classes(WEEK_COLUMNS))
            on:keydown=on_keydown
        >
            <For
                each=days
                key=|day| *day
                children=move |day| view! { <WeekColumn ctx=ctx day=day /> }
            />
        </div>
    }
}

/// One day of the week: its head (sticky, so the columns scroll together
/// under one row of heads), then every entry: working, "Out (N)", then
/// holiday and off.
#[component]
fn WeekColumn(ctx: Ctx, day: i64) -> impl IntoView {
    let data = Memo::new(move |_| ctx.day_data(day));
    let sorted = Memo::new(move |_| {
        data.with(|d| {
            d.as_ref()
                .map(|d| sorted_entries(&d.entries))
                .unwrap_or_default()
        })
    });
    let group = move |groups: &'static [EntryGroup]| {
        move || {
            sorted.with(|entries| {
                entries
                    .iter()
                    .filter(|e| groups.contains(&e.kind.group()))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        }
    };
    let working = group(&[EntryGroup::Working]);
    let out = group(&[EntryGroup::Out]);
    let rest = group(&[EntryGroup::Holiday, EntryGroup::Off]);
    let out_count = move || {
        sorted.with(|s| {
            s.iter()
                .filter(|e| e.kind.group() == EntryGroup::Out)
                .count()
        })
    };
    let is_today = move || ctx.today.get() == day;
    let weekday = Weekday::from_epoch_day(day);
    let head_id = ctx.id(&format!("week-head-{day}"));
    let class = move || {
        let mut classes =
            vec!["flex min-w-0 flex-col gap-2 rounded-box border border-base-300 p-2"];
        data.with(|d| {
            if d.as_ref().is_some_and(|d| d.closed.is_some()) {
                classes.push(DAY_CLOSED);
            } else if d.as_ref().is_some_and(|d| d.holiday.is_some()) {
                classes.push(DAY_HOLIDAY);
            } else if ctx.weekend.get().contains(weekday) {
                classes.push(DAY_WEEKEND);
            }
        });
        if is_today() {
            classes.push(DAY_TODAY);
        }
        classes.join(" ")
    };
    let always = Signal::stored(true);
    let chip = move |entry: RosterEntry| view! { <Chip ctx=ctx entry=entry in_tab_order=always prefer_times=true /> };

    view! {
        <section
            data-week-column=""
            data-day=iso_date(day)
            data-today=move || is_today().then_some("")
            aria-labelledby=head_id.clone()
            class=class
        >
            <header id=head_id.clone() class="sticky top-0 z-10 flex flex-col gap-1 bg-base-100">
                <h3 class="sr-only">{move || ctx.texts.with(|t| t.date_long_of(day))}</h3>
                <DayHead ctx=ctx day=day data=data tabbable=true with_weekday=true />
            </header>
            <DayStatus ctx=ctx data=data />
            <ul data-week-working="" class="flex flex-col gap-1">
                <For each=working key=|entry| entry.content_key() children=chip />
            </ul>
            <Show when=move || { out_count() > 0 }>
                <h4 data-week-out="" class="text-xs font-semibold">
                    {move || ctx.texts.with(|t| fill(&t.week_out, &[("count", &out_count().to_string())]))}
                </h4>
                <ul data-week-absent="" class="flex flex-col gap-1">
                    <For each=out key=|entry| entry.content_key() children=chip />
                </ul>
            </Show>
            <ul data-week-off="" class="flex flex-col gap-1">
                <For each=rest key=|entry| entry.content_key() children=chip />
            </ul>
        </section>
    }
}
