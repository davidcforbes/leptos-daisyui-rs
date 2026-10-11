use super::style::{AbsenceKind, RosterZoom};
use super::types::{DayMark, EntryKind, RosterDay, Staffing, civil, clock, day_counts};
use crate::components::{ShiftState, Weekday};

/// Framework-owned copy of [`MonthRoster`](super::MonthRoster): every word it
/// writes, the month and weekday names, and the date templates. Pass it as a
/// `Signal` so a language that arrives after mount propagates instead of
/// freezing; the roster renames its nodes in place (they are keyed by date
/// and content, never by the texts).
///
/// Templates use `{name}` placeholders. The English defaults follow the
/// 4iiz-kit catalogue ("Wednesday, Oct 14, 2026", "October 2026"), and
/// [`es`](Self::es) its Spanish ("miércoles, 14 oct 2026", "octubre de
/// 2026"), so a roster here reads like the kit's in 4iiz-People.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthRosterTexts {
    /// Month names, January first.
    pub month_names: [String; 12],
    /// Short month names, January first.
    pub month_short: [String; 12],
    /// Weekday names, Monday first.
    pub weekday_names: [String; 7],
    /// Short weekday names, Monday first.
    pub weekday_short: [String; 7],
    /// The month title: `{month}`, `{year}`.
    pub month_title: String,
    /// A day's name, as a date box and a chip speak it: `{weekday}`,
    /// `{month_short}`, `{day}`, `{year}`.
    pub date_long: String,
    /// A day in full, the Day zoom's title: `{weekday}`, `{month}`, `{day}`,
    /// `{year}`.
    pub date_full: String,
    /// A week inside one month: `{month}`, `{start}`, `{end}`, `{year}`.
    pub week_same_month: String,
    /// A week across two months of one year: `{start_month}`, `{start}`,
    /// `{end_month}`, `{end}`, `{year}`.
    pub week_same_year: String,
    /// A week across two years: `{start_month}`, `{start}`, `{start_year}`,
    /// `{end_month}`, `{end}`, `{end_year}`.
    pub week_cross_year: String,
    /// "Today": the button, and the word on today's date box.
    pub today: String,
    /// The previous-month arrow's name.
    pub previous_month: String,
    /// The next-month arrow's name.
    pub next_month: String,
    /// The previous-week arrow's name.
    pub previous_week: String,
    /// The next-week arrow's name.
    pub next_week: String,
    /// The previous-day arrow's name.
    pub previous_day: String,
    /// The next-day arrow's name.
    pub next_day: String,
    /// The zoom switch's group name.
    pub zoom_label: String,
    /// The zoom switch's Month button.
    pub zoom_month: String,
    /// The zoom switch's Week button.
    pub zoom_week: String,
    /// The zoom switch's Day button.
    pub zoom_day: String,
    /// The week buttons' column header (read by screen readers only).
    pub week_column: String,
    /// A row's week button: `{date}` (the week's first day, long form).
    pub show_week: String,
    /// The week button's visible ISO week number: `{week}`.
    pub week_number: String,
    /// A day number's button: `{date}` (long form).
    pub show_day: String,
    /// A day that is not in the host's data.
    pub no_schedule: String,
    /// A day that is in the host's data with no one on it.
    pub no_one: String,
    /// [`no_schedule`](Self::no_schedule) inside a day's spoken name.
    pub no_schedule_spoken: String,
    /// [`no_one`](Self::no_one) inside a day's spoken name.
    pub no_one_spoken: String,
    /// The line above a month with no one in it: `{month}` (the month title).
    pub month_empty: String,
    /// A holiday mark: `{name}`.
    pub holiday_mark: String,
    /// A half-day holiday mark: `{name}`.
    pub half_day_holiday_mark: String,
    /// A closed mark: `{reason}`.
    pub closed_mark: String,
    /// The staffing line: `{scheduled}`, `{needed}`.
    pub staffing: String,
    /// The visible overflow toggle: `{count}`.
    pub more_shown: String,
    /// The overflow toggle's spoken name for one person: `{count}`.
    pub more_one: String,
    /// The overflow toggle's spoken name for several people: `{count}`.
    pub more_other: String,
    /// A day's working count: `{count}`.
    pub working_count: String,
    /// A day's absent count: `{count}`.
    pub out_count: String,
    /// A day's off / holiday count: `{count}`.
    pub off_count: String,
    /// The state word of a full shift.
    pub full_shift: String,
    /// The state word of a half shift.
    pub half_shift: String,
    /// The state word of a vacation.
    pub vacation: String,
    /// The state word of an out-of-office day.
    pub out_of_office: String,
    /// The state word of sick leave.
    pub sick: String,
    /// The state word of any other absence.
    pub other_absence: String,
    /// The state word of a holiday entry.
    pub holiday: String,
    /// The state word of a day off.
    pub off: String,
    /// The word a pending (requested, not yet approved) absence carries.
    pub requested: String,
    /// A partial-day absence's hours: `{hours}`.
    pub hours: String,
    /// A visible time range: `{start}`, `{end}`.
    pub time_range: String,
    /// A working row's name in the Day timeline: `{person}`, `{start}`,
    /// `{end}`.
    pub working_spoken: String,
    /// The Week view's absences heading: `{count}`.
    pub week_out: String,
    /// The Day view's absences heading: `{count}`.
    pub out_today: String,
    /// The Day view's group of working entries without times.
    pub all_day: String,
    /// The Day view's coverage timeline name.
    pub coverage: String,
    /// The Day view's line when no one works.
    pub no_one_working: String,
    /// The Day view's holiday line: `{names}`.
    pub holiday_line: String,
    /// The Day view's off line: `{names}`.
    pub off_line: String,
}

fn strings<const N: usize>(words: [&str; N]) -> [String; N] {
    words.map(str::to_owned)
}

impl Default for MonthRosterTexts {
    fn default() -> Self {
        Self {
            month_names: strings([
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]),
            month_short: strings([
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ]),
            weekday_names: strings([
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ]),
            weekday_short: strings(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]),
            month_title: "{month} {year}".to_owned(),
            date_long: "{weekday}, {month_short} {day}, {year}".to_owned(),
            date_full: "{weekday}, {month} {day}, {year}".to_owned(),
            week_same_month: "{month} {start} - {end}, {year}".to_owned(),
            week_same_year: "{start_month} {start} - {end_month} {end}, {year}".to_owned(),
            week_cross_year: "{start_month} {start}, {start_year} - {end_month} {end}, {end_year}"
                .to_owned(),
            today: "Today".to_owned(),
            previous_month: "Previous month".to_owned(),
            next_month: "Next month".to_owned(),
            previous_week: "Previous week".to_owned(),
            next_week: "Next week".to_owned(),
            previous_day: "Previous day".to_owned(),
            next_day: "Next day".to_owned(),
            zoom_label: "View".to_owned(),
            zoom_month: "Month".to_owned(),
            zoom_week: "Week".to_owned(),
            zoom_day: "Day".to_owned(),
            week_column: "Week".to_owned(),
            show_week: "Show the week of {date}".to_owned(),
            week_number: "W{week}".to_owned(),
            show_day: "Show {date}".to_owned(),
            no_schedule: "No schedule published".to_owned(),
            no_one: "No one scheduled".to_owned(),
            no_schedule_spoken: "no schedule published".to_owned(),
            no_one_spoken: "no one scheduled".to_owned(),
            month_empty: "No one is scheduled in {month}".to_owned(),
            holiday_mark: "Holiday: {name}".to_owned(),
            half_day_holiday_mark: "Holiday (half day): {name}".to_owned(),
            closed_mark: "Closed: {reason}".to_owned(),
            staffing: "{scheduled} of {needed} needed".to_owned(),
            more_shown: "+{count}".to_owned(),
            more_one: "{count} more person".to_owned(),
            more_other: "{count} more people".to_owned(),
            working_count: "{count} working".to_owned(),
            out_count: "{count} out".to_owned(),
            off_count: "{count} off".to_owned(),
            full_shift: "Full shift".to_owned(),
            half_shift: "Half shift".to_owned(),
            vacation: "Vacation".to_owned(),
            out_of_office: "Out of office".to_owned(),
            sick: "Sick".to_owned(),
            other_absence: "Out".to_owned(),
            holiday: "Holiday".to_owned(),
            off: "Off".to_owned(),
            requested: "Requested".to_owned(),
            hours: "{hours} h".to_owned(),
            time_range: "{start}-{end}".to_owned(),
            working_spoken: "{person}, working {start} to {end}".to_owned(),
            week_out: "Out ({count})".to_owned(),
            out_today: "Out today ({count})".to_owned(),
            all_day: "All day".to_owned(),
            coverage: "Shift coverage".to_owned(),
            no_one_working: "No one is working".to_owned(),
            holiday_line: "Holiday: {names}".to_owned(),
            off_line: "Off: {names}".to_owned(),
        }
    }
}

impl MonthRosterTexts {
    /// Spanish.
    pub fn es() -> Self {
        Self {
            month_names: strings([
                "enero",
                "febrero",
                "marzo",
                "abril",
                "mayo",
                "junio",
                "julio",
                "agosto",
                "septiembre",
                "octubre",
                "noviembre",
                "diciembre",
            ]),
            month_short: strings([
                "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic",
            ]),
            weekday_names: strings([
                "lunes",
                "martes",
                "miércoles",
                "jueves",
                "viernes",
                "sábado",
                "domingo",
            ]),
            weekday_short: strings(["lun", "mar", "mié", "jue", "vie", "sáb", "dom"]),
            month_title: "{month} de {year}".to_owned(),
            date_long: "{weekday}, {day} {month_short} {year}".to_owned(),
            date_full: "{weekday}, {day} de {month} de {year}".to_owned(),
            week_same_month: "{start} - {end} {month} {year}".to_owned(),
            week_same_year: "{start} {start_month} - {end} {end_month} {year}".to_owned(),
            week_cross_year: "{start} {start_month} {start_year} - {end} {end_month} {end_year}"
                .to_owned(),
            today: "Hoy".to_owned(),
            previous_month: "Mes anterior".to_owned(),
            next_month: "Mes siguiente".to_owned(),
            previous_week: "Semana anterior".to_owned(),
            next_week: "Semana siguiente".to_owned(),
            previous_day: "Día anterior".to_owned(),
            next_day: "Día siguiente".to_owned(),
            zoom_label: "Vista".to_owned(),
            zoom_month: "Mes".to_owned(),
            zoom_week: "Semana".to_owned(),
            zoom_day: "Día".to_owned(),
            week_column: "Semana".to_owned(),
            show_week: "Mostrar la semana del {date}".to_owned(),
            week_number: "S{week}".to_owned(),
            show_day: "Mostrar el {date}".to_owned(),
            no_schedule: "Horario no publicado".to_owned(),
            no_one: "Nadie programado".to_owned(),
            no_schedule_spoken: "horario no publicado".to_owned(),
            no_one_spoken: "nadie programado".to_owned(),
            month_empty: "No hay nadie programado en {month}".to_owned(),
            holiday_mark: "Festivo: {name}".to_owned(),
            half_day_holiday_mark: "Festivo (medio día): {name}".to_owned(),
            closed_mark: "Cerrado: {reason}".to_owned(),
            staffing: "{scheduled} de {needed} necesarios".to_owned(),
            more_shown: "+{count}".to_owned(),
            more_one: "{count} persona más".to_owned(),
            more_other: "{count} personas más".to_owned(),
            working_count: "{count} trabajando".to_owned(),
            out_count: "{count} ausentes".to_owned(),
            off_count: "{count} libres".to_owned(),
            full_shift: "Jornada completa".to_owned(),
            half_shift: "Media jornada".to_owned(),
            vacation: "Vacaciones".to_owned(),
            out_of_office: "Fuera de la oficina".to_owned(),
            sick: "Enfermedad".to_owned(),
            other_absence: "Ausencia".to_owned(),
            holiday: "Festivo".to_owned(),
            off: "Libre".to_owned(),
            requested: "Solicitado".to_owned(),
            hours: "{hours} h".to_owned(),
            time_range: "{start}-{end}".to_owned(),
            working_spoken: "{person}, trabaja de {start} a {end}".to_owned(),
            week_out: "Ausentes ({count})".to_owned(),
            out_today: "Ausentes hoy ({count})".to_owned(),
            all_day: "Todo el día".to_owned(),
            coverage: "Cobertura de turnos".to_owned(),
            no_one_working: "Nadie trabaja".to_owned(),
            holiday_line: "Festivo: {names}".to_owned(),
            off_line: "Libres: {names}".to_owned(),
        }
    }

    /// The month title of the month holding `day`: "October 2026".
    pub fn month_title_of(&self, day: i64) -> String {
        let (year, month, _) = civil(day);
        fill(
            &self.month_title,
            &[
                ("month", &self.month_names[(month - 1) as usize]),
                ("year", &year.to_string()),
            ],
        )
    }

    /// A day's name: "Wednesday, Oct 14, 2026".
    pub fn date_long_of(&self, day: i64) -> String {
        self.date_with(&self.date_long, day)
    }

    /// A day in full: "Wednesday, October 14, 2026".
    pub fn date_full_of(&self, day: i64) -> String {
        self.date_with(&self.date_full, day)
    }

    fn date_with(&self, template: &str, day: i64) -> String {
        let (year, month, dom) = civil(day);
        let weekday = Weekday::from_epoch_day(day).index_from_monday();
        fill(
            template,
            &[
                ("weekday", &self.weekday_names[weekday]),
                ("month_short", &self.month_short[(month - 1) as usize]),
                ("month", &self.month_names[(month - 1) as usize]),
                ("day", &dom.to_string()),
                ("year", &year.to_string()),
            ],
        )
    }

    /// The seven days from `start`: "Oct 12 - 18, 2026", "Sep 28 - Oct 4,
    /// 2026", "Dec 28, 2026 - Jan 3, 2027". The English output is exactly
    /// `WeekView`'s `week_range_label`.
    pub fn week_title_of(&self, start: i64) -> String {
        let (sy, sm, sd) = civil(start);
        let (ey, em, ed) = civil(start + 6);
        let start_month = &self.month_short[(sm - 1) as usize];
        let end_month = &self.month_short[(em - 1) as usize];
        let (sd, ed, sy, ey) = (
            sd.to_string(),
            ed.to_string(),
            sy.to_string(),
            ey.to_string(),
        );
        if sy == ey && sm == em {
            fill(
                &self.week_same_month,
                &[
                    ("month", start_month),
                    ("start", &sd),
                    ("end", &ed),
                    ("year", &sy),
                ],
            )
        } else if sy == ey {
            fill(
                &self.week_same_year,
                &[
                    ("start_month", start_month),
                    ("start", &sd),
                    ("end_month", end_month),
                    ("end", &ed),
                    ("year", &sy),
                ],
            )
        } else {
            fill(
                &self.week_cross_year,
                &[
                    ("start_month", start_month),
                    ("start", &sd),
                    ("start_year", &sy),
                    ("end_month", end_month),
                    ("end", &ed),
                    ("end_year", &ey),
                ],
            )
        }
    }

    /// The header title for `zoom` at `anchor` with weeks starting on
    /// `week_start`.
    pub fn title(&self, zoom: RosterZoom, anchor: i64, week_start: Weekday) -> String {
        match zoom {
            RosterZoom::Month => self.month_title_of(anchor),
            RosterZoom::Week => self.week_title_of(crate::components::week_start_for_weekday(
                anchor, week_start,
            )),
            RosterZoom::Day => self.date_full_of(anchor),
        }
    }

    /// The previous / next arrows' names for `zoom`.
    pub fn arrows(&self, zoom: RosterZoom) -> (&str, &str) {
        match zoom {
            RosterZoom::Month => (&self.previous_month, &self.next_month),
            RosterZoom::Week => (&self.previous_week, &self.next_week),
            RosterZoom::Day => (&self.previous_day, &self.next_day),
        }
    }

    /// The zoom switch's word for `zoom`.
    pub fn zoom_word(&self, zoom: RosterZoom) -> &str {
        match zoom {
            RosterZoom::Month => &self.zoom_month,
            RosterZoom::Week => &self.zoom_week,
            RosterZoom::Day => &self.zoom_day,
        }
    }

    /// The word for an absence kind.
    pub fn absence_word(&self, kind: AbsenceKind) -> &str {
        match kind {
            AbsenceKind::Vacation => &self.vacation,
            AbsenceKind::OutOfOffice => &self.out_of_office,
            AbsenceKind::Sick => &self.sick,
            AbsenceKind::Other => &self.other_absence,
        }
    }

    /// An entry's state in words: "Full shift", "Vacation, Requested",
    /// "Sick, 4 h", "Off". Every chip shows this, so colour is never the only
    /// channel.
    pub fn state_words(&self, kind: EntryKind) -> String {
        match kind.normalized() {
            EntryKind::Working(ShiftState::Half) => self.half_shift.clone(),
            EntryKind::Working(_) => self.full_shift.clone(),
            EntryKind::Absent {
                kind,
                pending,
                partial_hours,
            } => {
                let mut words = vec![self.absence_word(kind).to_owned()];
                if pending {
                    words.push(self.requested.clone());
                }
                if let Some(hours) = partial_hours {
                    words.push(self.hours_of(hours));
                }
                words.join(", ")
            }
            EntryKind::Holiday => self.holiday.clone(),
            EntryKind::Off => self.off.clone(),
        }
    }

    /// "4 h", "2.5 h".
    pub fn hours_of(&self, hours: f32) -> String {
        let text = if hours.fract() == 0.0 {
            format!("{hours:.0}")
        } else {
            format!("{hours:.1}")
        };
        fill(&self.hours, &[("hours", &text)])
    }

    /// "08:00-16:30".
    pub fn time_range_of(&self, start: u16, end: u16) -> String {
        fill(
            &self.time_range,
            &[("start", &clock(start)), ("end", &clock(end))],
        )
    }

    /// The overflow toggle's spoken name: "5 more people".
    pub fn more_spoken(&self, count: usize) -> String {
        let template = if count == 1 {
            &self.more_one
        } else {
            &self.more_other
        };
        fill(template, &[("count", &count.to_string())])
    }

    /// A holiday mark: "Holiday: Columbus Day", or the half-day form.
    pub fn holiday_mark_of(&self, mark: &DayMark) -> String {
        let template = if mark.half_day {
            &self.half_day_holiday_mark
        } else {
            &self.holiday_mark
        };
        fill(template, &[("name", &mark.label)])
    }

    /// A closed mark: "Closed: Building maintenance".
    pub fn closed_mark_of(&self, reason: &str) -> String {
        fill(&self.closed_mark, &[("reason", reason)])
    }

    /// The staffing line: "3 of 4 needed".
    pub fn staffing_of(&self, staffing: Staffing) -> String {
        fill(
            &self.staffing,
            &[
                ("scheduled", &staffing.scheduled.to_string()),
                ("needed", &staffing.needed.to_string()),
            ],
        )
    }

    /// A day's spoken name: its date, "Today", what it holds and its marks.
    /// "Wednesday, Oct 14, 2026, Today, 5 working, 4 out, 5 of 6 needed";
    /// "..., no one scheduled" for a known empty day; "..., no schedule
    /// published" for a day absent from the host's data. Never "0 people":
    /// a zero count is left out, and a day with no one says so in words.
    pub fn day_name(&self, day: i64, is_today: bool, data: Option<&RosterDay>) -> String {
        let mut parts = vec![self.date_long_of(day)];
        if is_today {
            parts.push(self.today.clone());
        }
        match data {
            None => parts.push(self.no_schedule_spoken.clone()),
            Some(data) => {
                let counts = day_counts(&data.entries);
                if data.entries.is_empty() {
                    parts.push(self.no_one_spoken.clone());
                }
                for (count, template) in [
                    (counts.working, &self.working_count),
                    (counts.out, &self.out_count),
                    (counts.off, &self.off_count),
                ] {
                    if count > 0 {
                        parts.push(fill(template, &[("count", &count.to_string())]));
                    }
                }
                if let Some(mark) = &data.holiday {
                    parts.push(self.holiday_mark_of(mark));
                }
                if let Some(reason) = &data.closed {
                    parts.push(self.closed_mark_of(reason));
                }
                if let Some(staffing) = data.staffing {
                    parts.push(self.staffing_of(staffing));
                }
            }
        }
        parts.join(", ")
    }

    /// The overflow toggle's visible text: "+5".
    pub fn more_visible(&self, count: usize) -> String {
        fill(&self.more_shown, &[("count", &count.to_string())])
    }
}

/// Replace each `{key}` in `template` with its value.
pub(crate) fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(template.to_owned(), |text, (key, value)| {
            text.replace(&format!("{{{key}}}"), value)
        })
}
