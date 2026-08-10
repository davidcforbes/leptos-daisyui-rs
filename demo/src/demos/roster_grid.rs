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

/// A twelve-worker, seven-day roster: 84 tiles, deliberately big enough that
/// the difference between one tab stop and one-per-tile is impossible to miss.
/// With `tabindex=0` on every tile this section alone would cost 84 Tab presses
/// to cross; with the roving tabindex it costs one.
fn department_roster() -> Vec<RosterRow> {
    const WORKERS: [&str; 12] = [
        "Ada Lovelace",
        "Grace Hopper",
        "Katherine Johnson",
        "Margaret Hamilton",
        "Dorothy Vaughan",
        "Mary Jackson",
        "Radia Perlman",
        "Barbara Liskov",
        "Frances Allen",
        "Jean Bartik",
        "Karen Sparck Jones",
        "Evelyn Boyd Granville",
    ];

    WORKERS
        .iter()
        .enumerate()
        .map(|(i, worker)| {
            let cells = (0..7)
                .map(|day| match (i + day) % 5 {
                    0 => RosterCell::new("09:00-17:00", ShiftState::Full),
                    1 => RosterCell::new("09:00-13:00", ShiftState::Half),
                    2 => RosterCell::off(),
                    3 => RosterCell::new("Leave", ShiftState::Leave),
                    _ => RosterCell::new("22:00-06:00", ShiftState::Full),
                })
                .collect();
            RosterRow::new(*worker, cells)
        })
        .collect()
}

/// Seven-day column headers for [`department_roster`].
fn full_week_columns() -> Vec<String> {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
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

    let (department_selected, set_department_selected) = signal(None::<(usize, usize)>);
    let (department_activated, set_department_activated) = signal(String::from("(none yet)"));
    let on_department_activate = Callback::new(move |(row, col): (usize, usize)| {
        set_department_selected.set(Some((row, col)));
        let worker = department_roster()
            .get(row)
            .map(|r| r.worker.clone())
            .unwrap_or_default();
        let day = full_week_columns().get(col).cloned().unwrap_or_default();
        set_department_activated.set(format!("{worker} - {day}"));
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
                    "Supplying on_cell_activate makes the table a role=grid widget whose cells carry aria-selected and whose tiles are role=button with an accessible name of \"worker, day, label, state\". Tab into the grid, move with the arrow keys, and press Enter or Space. ARIA marks grid as name-required, so this one is named with label - without it a screen reader announces only \"grid, 5 rows, 6 columns\", identical to the roster below."
                </p>
                <RosterGrid
                    rows=rows
                    label="Duty roster, working week"
                    on_cell_activate=on_cell_activate
                    selected_cell=Signal::derive(move || selected.get())
                />
                <p class="text-sm opacity-60 mt-2">
                    "Last activated: " {move || last_activated.get()}
                </p>
            </Section>

            <Section title="Roving focus: 84 tiles, one tab stop" col=true>
                <p class="text-sm opacity-60">
                    "Twelve workers by seven days. With a tabindex on every tile this grid would be 84 sequential Tab presses to cross; the ARIA Data Grid roving tabindex makes the whole thing a single tab stop. Tab in, then: arrow keys move one cell (stopping at the edges rather than wrapping into another worker's week), Home and End jump to the start and end of the row, Ctrl+Home and Ctrl+End (Cmd on macOS) to the first and last cell of the grid, and Enter or Space activates. Tab again leaves the grid entirely. Modifier chords the browser owns are left alone: Alt+Left and Alt+Right still go Back and Forward."
                </p>
                <h3 class="text-base font-medium" id="roster-department-heading">
                    "Department roster, full week"
                </h3>
                <RosterGrid
                    rows=Signal::derive(department_roster)
                    columns=Signal::derive(full_week_columns)
                    density=RosterDensity::Compact
                    labelled_by="roster-department-heading"
                    on_cell_activate=on_department_activate
                    selected_cell=Signal::derive(move || department_selected.get())
                />
                <p class="text-sm opacity-60 mt-2">
                    "Last activated: " {move || department_activated.get()}
                </p>
            </Section>

            <Section title="Display-only highlight: selected_cell with no callback" col=true>
                <p class="text-sm opacity-60">
                    "selected_cell alone is a read-only highlight - today's shift, a search hit - so it does NOT make the grid interactive. The ring renders, but there is no role=grid, no tabindex and no role=button: tabbing through this section skips the grid entirely. Advertising 140 unresponsive buttons on a 20x7 roster would be WCAG 4.1.2."
                </p>
                <RosterGrid rows=rows selected_cell=Signal::derive(|| Some((1, 3))) />
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

            <Section title="Naming the table: label, labelled_by, described_by" col=true>
                <p class="text-sm opacity-60" id="roster-naming-note">
                    "Spread attributes land on the component's root div, so without these props there is no way to name the table itself and two rosters on one page are announced identically. labelled_by points at the visible heading below and suppresses aria-label, exactly as Modal does, so assistive tech hears what sighted users read."
                </p>
                <h3 class="text-base font-medium" id="roster-ward-b-heading">"Ward B, week of 12 May"</h3>
                <RosterGrid
                    rows=rows
                    labelled_by="roster-ward-b-heading"
                    described_by="roster-naming-note"
                />
                <p class="text-sm opacity-60 mt-2">
                    "Below: the same roster named with label instead, for a table with no visible heading."
                </p>
                <RosterGrid rows=rows label="Ward C, week of 12 May" />
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
