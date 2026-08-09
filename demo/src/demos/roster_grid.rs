use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// A five-day roster exercising every [`ShiftState`], plus two deliberately
/// ragged rows so the pad/truncate behaviour is visible on screen rather than
/// only in the unit tests.
fn week_roster() -> Vec<RosterRow> {
    vec![
        RosterRow::new(
            "Ada Lovelace",
            vec![
                RosterCell::new("09:00-17:00", ShiftState::Full),
                RosterCell::new("09:00-17:00", ShiftState::Full),
                RosterCell::new("09:00-13:00", ShiftState::Half),
                RosterCell::off(),
                RosterCell::new("09:00-17:00", ShiftState::Full),
            ],
        ),
        RosterRow::new(
            "Grace Hopper",
            vec![
                RosterCell::new("Bank holiday", ShiftState::Holiday),
                RosterCell::new("22:00-06:00", ShiftState::Full),
                RosterCell::new("22:00-06:00", ShiftState::Full),
                RosterCell::new("Annual leave", ShiftState::Leave),
                RosterCell::new("Annual leave", ShiftState::Leave),
            ],
        ),
        // Deliberately SHORT: only two of the five columns are supplied, so
        // Wed/Thu/Fri pad with `Off` and stay under their own headers.
        RosterRow::new(
            "Katherine Johnson (short row)",
            vec![
                RosterCell::new("13:00-21:00", ShiftState::Full),
                RosterCell::new("13:00-17:00", ShiftState::Half),
            ],
        ),
        // Deliberately LONG: eight cells for five columns, so the trailing
        // three are truncated rather than pushing the row out of alignment.
        RosterRow::new(
            "Margaret Hamilton (long row)",
            vec![
                RosterCell::new("07:00-15:00", ShiftState::Full),
                RosterCell::new("07:00-11:00", ShiftState::Half),
                RosterCell::new("07:00-15:00", ShiftState::Full),
                RosterCell::new("Training", ShiftState::Leave),
                RosterCell::new("07:00-15:00", ShiftState::Full),
                RosterCell::new("(Sat - truncated)", ShiftState::Full),
                RosterCell::new("(Sun - truncated)", ShiftState::Full),
                RosterCell::new("(overflow - truncated)", ShiftState::Full),
            ],
        ),
    ]
}

/// One row per state, so a reviewer can see all five tints and the
/// solid-vs-dashed accent bar side by side.
fn state_legend() -> Vec<RosterRow> {
    ShiftState::ALL
        .iter()
        .map(|state| {
            RosterRow::new(
                state.as_label(),
                vec![RosterCell::new(state.as_label(), *state)],
            )
        })
        .collect()
}

#[component]
pub fn RosterGridDemo() -> impl IntoView {
    let rows = Signal::derive(week_roster);
    let legend = Signal::derive(state_legend);
    let empty_rows: Signal<Vec<RosterRow>> = Signal::derive(Vec::new);

    let (selected, set_selected) = signal(None::<(usize, usize)>);
    let (last_activated, set_last_activated) = signal(String::from("(none yet)"));

    let on_cell_activate = Callback::new(move |(row, col): (usize, usize)| {
        set_selected.set(Some((row, col)));
        let worker = week_roster()
            .get(row)
            .map(|r| r.worker.clone())
            .unwrap_or_default();
        let day = default_roster_columns()
            .get(col)
            .cloned()
            .unwrap_or_default();
        set_last_activated.set(format!("{worker} - {day}"));
    });

    view! {
        <ContentLayout
            title="Roster Grid (Schedule Matrix)"
            description="A staffing roster: worker rows by weekday columns, where each cell carries a shift value plus a semantic state. Rendered as a real table with scope=col weekday headers and scope=row worker names, so screen readers keep the header association. Colour is never the only channel - every tile shows its own label, carries visually-hidden state text, and gets a solid accent bar when working versus a dashed one when not."
        >
            <Section title="Comfortable density (default), with two ragged rows" col=true>
                <p class="text-sm opacity-60">
                    "Row 3 supplies only two cells for five columns and pads with Off; row 4 supplies eight and truncates. Neither row slips out of alignment with its headers."
                </p>
                <RosterGrid rows=rows />
            </Section>

            <Section title="Compact density" col=true>
                <p class="text-sm opacity-60">
                    "32px rows instead of 40px. Only the height and text size change - tile padding stays at 8px so it never exceeds the 8px gap between tiles."
                </p>
                <RosterGrid rows=rows density=RosterDensity::Compact />
            </Section>

            <Section title="Every shift state" col=true>
                <p class="text-sm opacity-60">
                    "Full and Half are working states (solid accent bar); Off, Holiday and Leave are not (dashed). The distinction survives greyscale."
                </p>
                <RosterGrid
                    rows=legend
                    columns=vec!["State".to_string()]
                    worker_header="Name"
                />
            </Section>

            <Section title="Interactive: activate and select a cell" col=true>
                <p class="text-sm opacity-60">
                    "Supplying on_cell_activate or selected_cell makes tiles focusable with role=button and an accessible name of \"worker, day, label, state\". A display-only roster gains no tab stops at all."
                </p>
                <RosterGrid
                    rows=rows
                    on_cell_activate=on_cell_activate
                    selected_cell=Signal::derive(move || selected.get())
                />
                <p class="text-sm opacity-60 mt-2">
                    "Last activated: " {move || last_activated.get()}
                </p>
            </Section>

            <Section title="Caller-supplied columns and localised state names" col=true>
                <p class="text-sm opacity-60">
                    "Columns are a prop, not a hardcoding, so a seven-day week or another locale needs no change to the component. state_label overrides the announced state names the same way hour_label does for the scheduler components."
                </p>
                <RosterGrid
                    rows=rows
                    worker_header="Personal"
                    columns=vec![
                        "Lun".to_string(),
                        "Mar".to_string(),
                        "Mie".to_string(),
                        "Jue".to_string(),
                        "Vie".to_string(),
                        "Sab".to_string(),
                        "Dom".to_string(),
                    ]
                    state_label=Callback::new(|state: ShiftState| {
                        match state {
                            ShiftState::Full => "Jornada completa",
                            ShiftState::Half => "Media jornada",
                            ShiftState::Off => "Libre",
                            ShiftState::Holiday => "Festivo",
                            ShiftState::Leave => "Vacaciones",
                        }
                            .to_string()
                    })
                />
            </Section>

            <Section title="Empty roster" col=true>
                <p class="text-sm opacity-60">
                    "No rows (or no columns) renders an empty state, never a zero-column table or a header with no body."
                </p>
                <RosterGrid
                    rows=empty_rows
                    empty_title="No one is rostered"
                    empty_subtitle="Add a worker to start building the week."
                />
            </Section>
        </ContentLayout>
    }
}
