use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{
    AbsenceKind, EntryGroup, MonthRoster, MonthRosterTexts, RosterDay, RosterEntry, RosterZoom,
};

/// Wednesday 2026-10-14: the fixture's "today" and first anchor.
const TODAY: i64 = 20_740;

/// The fixture office's 24 people.
const PEOPLE: [&str; 24] = [
    "Ana Lopez",
    "Luis Garcia",
    "Mia Chen",
    "Sam Patel",
    "Priya Shah",
    "Tom Baker",
    "Lena Novak",
    "Omar Haddad",
    "Grace Kim",
    "Diego Ruiz",
    "Hannah Lee",
    "Ivan Petrov",
    "Julia Rossi",
    "Kofi Mensah",
    "Laura Silva",
    "Marco Bianchi",
    "Nora Jensen",
    "Oscar Diaz",
    "Paula Moreno",
    "Quinn Avery",
    "Rosa Ortiz",
    "Sean Murphy",
    "Tara Singh",
    "Victor Hale",
];

const EARLY: (u16, u16) = (7 * 60, 15 * 60);
const DAY: (u16, u16) = (8 * 60, 16 * 60 + 30);
const LATE: (u16, u16) = (12 * 60, 20 * 60);
const HALF: (u16, u16) = (8 * 60, 12 * 60);

fn date(day_of_october: u32) -> i64 {
    days_from_civil(2026, 10, day_of_october)
}

fn entry_id(person: usize, day: i64) -> String {
    format!("p{person}-{}", iso(day))
}

fn iso(day: i64) -> String {
    let (year, month, dom) = civil_from_days(day);
    format!("{year:04}-{month:02}-{dom:02}")
}

fn working(person: usize, day: i64, (start, end): (u16, u16), half: bool) -> RosterEntry {
    let shift = if half {
        ShiftState::Half
    } else {
        ShiftState::Full
    };
    RosterEntry::working(entry_id(person, day), PEOPLE[person], shift).times(start, end)
}

fn absent(person: usize, day: i64, kind: AbsenceKind) -> RosterEntry {
    RosterEntry::absent(
        format!("{}-out", entry_id(person, day)),
        PEOPLE[person],
        kind,
    )
}

/// People 0..20 each work one weekday (`i % 5`, Monday first) on a shift
/// chosen by `i / 5`: one Early, one Day, one Late and one Half on every
/// weekday. 20 and 21 work Saturday mornings; 22 and 23 float.
fn regulars(weekday: usize) -> Vec<(usize, (u16, u16), bool)> {
    (0..20)
        .filter(|person| person % 5 == weekday)
        .map(|person| match person / 5 {
            0 => (person, EARLY, false),
            1 => (person, DAY, false),
            2 => (person, LATE, false),
            _ => (person, HALF, true),
        })
        .collect()
}

/// October 2026 for the fixture office. The holiday and closure labels are
/// the HOST's data, so they follow the locale; every entry is the same in
/// both languages (its words come from `MonthRosterTexts`), which is what
/// lets an EN/ES switch rename the chips in place.
fn october(es: bool) -> Vec<RosterDay> {
    let mut days = Vec::new();
    for dom in 1..=31 {
        let day = date(dom);
        // 28 and 29 October are not published yet: absent from the data.
        if dom == 28 || dom == 29 {
            continue;
        }
        let weekday = Weekday::from_epoch_day(day).index_from_monday();
        let mut roster = RosterDay::new(day);
        let mut entries: Vec<RosterEntry> = Vec::new();
        match weekday {
            0..=4 => {
                for (person, shift, half) in regulars(weekday) {
                    entries.push(working(person, day, shift, half));
                }
            }
            5 => {
                entries.push(working(20, day, (9 * 60, 13 * 60), true));
                entries.push(working(21, day, (9 * 60, 13 * 60), true));
            }
            // Sundays are published with no one on them: "No one scheduled".
            _ => {}
        }
        match dom {
            // A US holiday: the Monday people are on holiday, not working.
            12 => {
                roster = roster.holiday(if es {
                    "Día de la Raza"
                } else {
                    "Columbus Day"
                });
                entries = regulars(0)
                    .into_iter()
                    .map(|(person, _, _)| {
                        RosterEntry::holiday(
                            format!("{}-hol", entry_id(person, day)),
                            PEOPLE[person],
                        )
                    })
                    .collect();
            }
            // The busiest day: a training day with five working, four out.
            14 => {
                entries = vec![
                    working(2, day, EARLY, false),
                    working(12, day, LATE, false),
                    working(22, day, DAY, false),
                    working(23, day, (9 * 60, 17 * 60), false),
                    working(21, day, HALF, true),
                    absent(7, day, AbsenceKind::Vacation),
                    absent(20, day, AbsenceKind::Vacation).pending(),
                    absent(13, day, AbsenceKind::OutOfOffice),
                    absent(17, day, AbsenceKind::Sick),
                ];
            }
            6 => entries[1] = absent(6, day, AbsenceKind::Other),
            9 => entries[2] = absent(14, day, AbsenceKind::Vacation).pending(),
            16 => entries[1] = absent(9, day, AbsenceKind::Vacation),
            19 => entries[2] = absent(10, day, AbsenceKind::Sick).partial_hours(4.0),
            20 => entries[2] = absent(11, day, AbsenceKind::OutOfOffice),
            // A half-day holiday: the Friday people work the morning only.
            23 => {
                roster = roster.half_day_holiday(if es {
                    "Tarde de agradecimiento"
                } else {
                    "Staff appreciation afternoon"
                });
                entries = regulars(4)
                    .into_iter()
                    .map(|(person, _, _)| working(person, day, HALF, true))
                    .collect();
            }
            // Closed: known, and no one on it.
            30 => {
                roster = roster.closed(if es {
                    "Mantenimiento del edificio"
                } else {
                    "Building maintenance"
                });
                entries.clear();
            }
            _ => {}
        }
        let scheduled = entries
            .iter()
            .filter(|e| e.kind.group() == EntryGroup::Working)
            .count() as u32;
        roster = match (weekday, dom) {
            (_, 12 | 30) => roster,
            (0..=4, 14) => roster.staffing(scheduled, 6),
            (0..=4, _) => roster.staffing(scheduled, 4),
            (5, _) => roster.staffing(scheduled, 2),
            _ => roster,
        };
        days.push(roster.entries(entries));
    }
    days
}

/// The editable fixture's data (ldui-k145): the same October plus the last
/// days of September and the first of November, which the month grid shows
/// as an adjacent month's days. Like 4iiz-Office, the host lets only days
/// AFTER today change: those are addable and their working entries
/// removable. It keeps its own edits (removed ids, people added per day)
/// and answers with new `days`.
fn editable(es: bool, removed: &[String], added: &[i64]) -> Vec<RosterDay> {
    let mut days = october(es);
    for day in [
        days_from_civil(2026, 9, 28),
        days_from_civil(2026, 9, 29),
        days_from_civil(2026, 9, 30),
        days_from_civil(2026, 11, 1),
    ] {
        let weekday = Weekday::from_epoch_day(day).index_from_monday();
        let entries: Vec<RosterEntry> = regulars(weekday)
            .into_iter()
            .map(|(person, shift, half)| working(person, day, shift, half))
            .collect();
        days.push(RosterDay::new(day).entries(entries));
    }
    days.into_iter()
        .map(|mut roster| {
            let day = roster.day;
            let future = day > TODAY;
            roster.entries.retain(|e| !removed.contains(&e.id));
            for (n, _) in added.iter().enumerate().filter(|(_, d)| **d == day) {
                roster.entries.push(
                    RosterEntry::working(
                        format!("new-{n}"),
                        format!("New person {}", n + 1),
                        ShiftState::Full,
                    )
                    .times(DAY.0, DAY.1),
                );
            }
            if future {
                roster.entries = roster
                    .entries
                    .into_iter()
                    .map(|e| {
                        if e.kind.group() == EntryGroup::Working {
                            e.removable()
                        } else {
                            e
                        }
                    })
                    .collect();
                roster = roster.addable();
            }
            roster
        })
        .collect()
}

#[component]
pub fn MonthRosterDemo() -> impl IntoView {
    let zoom = RwSignal::new(RosterZoom::Month);
    let anchor = RwSignal::new(TODAY);
    let locale = RwSignal::new("en".to_string());
    let last_nav = RwSignal::new(String::new());
    let activated = RwSignal::new(String::new());
    let es = Signal::derive(move || locale.get() == "es");
    let days = Signal::derive(move || october(es.get()));
    let texts = Signal::derive(move || {
        if es.get() {
            MonthRosterTexts::es()
        } else {
            MonthRosterTexts::default()
        }
    });
    let label = Signal::derive(move || {
        if es.get() {
            "Horario de la oficina de Raleigh".to_string()
        } else {
            "Raleigh office schedule".to_string()
        }
    });
    let navigate = Callback::new(move |(to, day): (RosterZoom, i64)| {
        last_nav.set(format!("{}:{}", to.as_str(), iso(day)));
        zoom.set(to);
        anchor.set(day);
    });

    // The editable fixture (ldui-k145): its own zoom and anchor, and the
    // host's edits. A real host opens a dialog on each proposal.
    let edit_zoom = RwSignal::new(RosterZoom::Month);
    let edit_anchor = RwSignal::new(TODAY);
    let removed = RwSignal::new(Vec::<String>::new());
    let added = RwSignal::new(Vec::<i64>::new());
    let opened = RwSignal::new(String::new());
    let opened_count = RwSignal::new(0_u32);
    let last_add = RwSignal::new(String::new());
    let last_remove = RwSignal::new(String::new());
    let edit_days =
        Signal::derive(move || removed.with(|r| added.with(|a| editable(es.get(), r, a))));
    let edit_navigate = Callback::new(move |(to, day): (RosterZoom, i64)| {
        edit_zoom.set(to);
        edit_anchor.set(day);
    });

    view! {
        // A host prints the roster on one landscape page (ldui-5lmt.7).
        <style>"@page { size: landscape; }"</style>
        <ContentLayout
            title="Month Roster"
            description="An opinionated staff calendar: who works each day and who is out, at Month, Week and Day zoom over one controlled data set."
        >
            <Section title="Fixture office, October 2026">
                <p class="text-sm text-base-content/75">
                    "24 people. 12 Oct is a holiday, 23 Oct a half-day holiday, 30 Oct closed, "
                    "and 28-29 Oct are not published (absent from the data). 14 Oct is today: "
                    "a training day with nine entries, so its box shows four and \"+5\". "
                    "The host owns the zoom and the anchor and answers every "
                    <code>"on_navigate"</code> " proposal."
                </p>
                <div class="flex flex-wrap items-center gap-4 text-sm">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="roster-locale-toggle"
                        on:click=move |_| {
                            locale.update(|l| *l = if l == "en" { "es".into() } else { "en".into() })
                        }
                    >
                        {move || if es.get() { "English" } else { "Español" }}
                    </Button>
                    <span>"Zoom: " <code data-testid="roster-zoom">{move || zoom.get().as_str()}</code></span>
                    <span>"Anchor: " <code data-testid="roster-anchor">{move || iso(anchor.get())}</code></span>
                    <span>"Epoch day: " <code data-testid="roster-anchor-day">{move || anchor.get().to_string()}</code></span>
                    <span>"Last navigation: " <code data-testid="roster-last-nav">{move || last_nav.get()}</code></span>
                    <span>"Activated: " <code data-testid="roster-activated">{move || activated.get()}</code></span>
                </div>
                <div id="month-roster-host" class="w-full">
                    <MonthRoster
                        days=days
                        anchor_day=anchor
                        today=Signal::stored(TODAY)
                        zoom=zoom
                        now_min=Signal::stored(Some(10 * 60 + 30))
                        texts=texts
                        label=label
                        on_navigate=navigate
                        on_entry_activate=Callback::new(move |id: String| activated.set(id))
                        attr:id="fixture-roster"
                    />
                </div>
            </Section>
            <Section title="Editing hooks">
                <p class="text-sm text-base-content/75">
                    "The same office with the host's editing hooks. Double-click a day, double-tap it, "
                    "or press Enter on it to open it; the day number still zooms to the Day view. "
                    "Days after today are addable (+) and their working people removable (x). "
                    "The roster only proposes; this host applies the change to its data."
                </p>
                <div class="flex flex-wrap items-center gap-4 text-sm">
                    <span>"Opened: " <code data-testid="editable-opened">{move || opened.get()}</code></span>
                    <span>"Times opened: " <code data-testid="editable-opened-count">{move || opened_count.get()}</code></span>
                    <span>"Add: " <code data-testid="editable-added">{move || last_add.get()}</code></span>
                    <span>"Remove: " <code data-testid="editable-removed">{move || last_remove.get()}</code></span>
                </div>
                <div id="editable-roster-host" class="w-full">
                    <MonthRoster
                        days=edit_days
                        anchor_day=edit_anchor
                        today=Signal::stored(TODAY)
                        zoom=edit_zoom
                        now_min=Signal::stored(Some(10 * 60 + 30))
                        texts=texts
                        label=Signal::derive(move || {
                            if es.get() {
                                "Horario editable de la oficina de Raleigh".to_string()
                            } else {
                                "Raleigh office schedule, editable".to_string()
                            }
                        })
                        on_navigate=edit_navigate
                        on_day_activate=Callback::new(move |day: i64| {
                            opened.set(iso(day));
                            opened_count.update(|n| *n += 1);
                        })
                        on_day_add=Callback::new(move |day: i64| {
                            last_add.set(iso(day));
                            added.update(|a| a.push(day));
                        })
                        on_entry_remove=Callback::new(move |id: String| {
                            last_remove.set(id.clone());
                            removed.update(|r| r.push(id));
                        })
                        attr:id="editable-roster"
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
