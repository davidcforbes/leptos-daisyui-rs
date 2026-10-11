# MonthRoster: design

Epic `ldui-5lmt` (children `.1`-`.7`), requested by the owner on 2026-10-11 through the
4iiz-Office laptop session (Office bead `op-uclmv`). This is a port of the 4iiz-kit
`MonthRoster` so that Office's calendar views look like 4iiz-People's month schedule.

## 1. What it is

`patterns::MonthRoster` is an opinionated layer-2 pattern (`src/patterns/month_roster/`). It
shows who works each day and who is out (vacation, out of office, sick), at three zoom levels of
the same data:

| Zoom | What it draws |
|---|---|
| Month | The kit's grid: whole weeks of seven date boxes, with chips for people, 4 visible and "+N" for the rest |
| Week | Seven day columns, each listing EVERY entry: working first, then "Out (N)" |
| Day | A shift coverage timeline (one bar per working person on an hour axis), then "Out today" grouped by kind |

The pattern owns layout, keyboard and state presentation. The page supplies typed data and
callbacks (Future-Architecture.md:478-533).

## 2. Reference (read, not depended on)

- The kit: `C:/dev/4iiz-kit/crates/kit-ui/src/dates/month_roster.rs` (classes :199-232, EN/ES
  catalogue :65-148, `span()` :961-968, header :1026-1105), the templates
  `templates/kit/dates/month_roster{,_day,_chip}.html`, the keyboard in
  `kit-assets/web/src/behaviors/month_roster.ts`, and `docs/month-roster.md`.
- What the port keeps: the classes (day box, outside/closed/holiday/weekend tints, today ring plus
  the word), the chip (accent bar, three lines, the state in words), the in-place overflow (a native
  peer checkbox revealing the rest of the list), the agenda below a width, and one tab stop with a
  roving active day.
- What the port drops (decision 5): add, edit, move, remove and drag. The kit's move form, its
  dialogs and its htmx swaps have no counterpart. A later bead can add editing if a host asks.

## 3. Decisions

The epic's decisions are defaults. This spec refines some of them; it drops none.

1. **Controlled and data-agnostic, like `WeekView`.** The component never fetches, never reads a
   clock and never mutates `days`. Every prev/next/today/zoom/drill request is a proposal,
   `on_navigate(RosterZoom, i64)`, that the host answers by changing `zoom` and `anchor_day`. The
   pure `visible_range(zoom, anchor, week_start)` tells the host which days to load.
   *Refinement:* epoch days are `i64`, not `i32`, because every date function the crate already
   has (`civil_from_days`, `week_start_for_weekday`, `Weekday::from_epoch_day`) takes `i64`. A host
   holding `i32` writes `.into()`. 2026-10-14 is epoch day 20740 either way.
2. **Unknown differs from empty.** A day ABSENT from `days` renders "No schedule published" (muted).
   A PRESENT day with no entries renders "No one scheduled". The component never says "0 people".
   A month with no entries at all shows one line, "No one is scheduled in October 2026", above a grid
   that still renders.
3. **Entry vocabulary.** `EntryKind` reuses `roster_grid::ShiftState` for the working colours and
   adds the absence kinds:

   | Kind | Tint | Accent bar | State word |
   |---|---|---|---|
   | `Working(Full)` | `bg-success/15` | `border-success` solid | Full shift |
   | `Working(Half)` | `bg-info/15` | `border-info` solid | Half shift |
   | `Absent{Vacation}` | `bg-warning/15` | `border-warning` dashed | Vacation |
   | `Absent{OutOfOffice}` | `bg-secondary/15` | `border-secondary` dashed | Out of office |
   | `Absent{Sick}` | `bg-error/15` | `border-error` dashed | Sick |
   | `Absent{Other}` | `bg-base-200/60` | `border-base-300` dashed | Out |
   | `Holiday` | `bg-accent/15` | `border-accent` dashed | Holiday |
   | `Off` | `bg-base-200/60` | `border-base-300` dashed | Off |

   A PENDING absence keeps its border, has NO fill and adds the word "Requested". Partial-day hours
   add "4 h". Every chip states its state in words, so no legend is needed and colour is never the
   only channel. `Working(Off|Holiday|Leave)` is normalised to `Off`, `Holiday` and
   `Absent{Other}` respectively, so a misuse can never paint work that was not rostered.
   `AbsenceKind::OutOfOffice` stays although 4iiz-People has no such type yet: the use case names it.
4. **Order inside a day:** working (by start time, untimed last, then name), then out (by kind in
   the order Vacation, Out of office, Sick, Other, then name), then holiday, then off (by name).
5. **Read-only in v1.** Activating a chip calls `on_entry_activate(entry id)`. There is no add,
   edit, move, remove or drag.
6. **CSR only**, like the rest of ldui.
7. **Strings.** `MonthRosterTexts` (`Default` = English, plus `es()`), passed as a `Signal`,
   includes the month and weekday names and every template. A switch from EN to ES and back renames
   the SAME nodes: the date boxes are keyed by date and the chips by content, never by the texts.
   Formats follow the kit: "Wednesday, Oct 14, 2026" / "miércoles, 14 oct 2026" for a day's name,
   "October 2026" / "octubre de 2026" for the month.
8. **Layout follows the host container, not the viewport.** Below 42rem (672 px) of the ROOT'S
   width the month is an agenda, the week stacks and the day timeline compresses. The root is an
   `@container`. A host can force a layout with `layout: Signal<Option<RosterLayout>>`.

Further decisions this spec adds:

9. **No `data-ld-pattern`.** The kit declares `data-ld-pattern="MonthRoster"`, but PixelProof's
   catalogue has no MonthRoster and the ldui-wra5 probe reports any name outside the catalogue as
   misplaced. The root carries `data-month-roster` instead. Add the declaration when the catalogue
   gains the pattern.
10. **The week view is not an hour grid.** A staffing week has 20+ people; an hour grid with that
    many lanes is unreadable. Week reuses WeekView's date math, not its grid.
11. **Weekend shading** defaults to Saturday and Sunday (`weekend: Signal<WeekdaySet>`), as the kit
    does. A host whose weekend is Friday-Saturday passes its own set.

## 4. Public API

```rust
// crates/ldui-design/src/components/month_roster.rs (Leptos-free variants)
pub enum RosterZoom { Month, Week, Day }          // Default: Month
pub enum AbsenceKind { Vacation, OutOfOffice, Sick, Other }
pub enum RosterLayout { Grid, Agenda }

// src/patterns/month_roster/types.rs (pure, Leptos-free)
pub struct RosterDay { pub day: i64, pub holiday: Option<DayMark>, pub closed: Option<String>,
                       pub staffing: Option<Staffing>, pub entries: Vec<RosterEntry> }
pub struct DayMark { pub label: String, pub half_day: bool }
pub struct Staffing { pub scheduled: u32, pub needed: u32 }
pub struct RosterEntry { pub id: String, pub person: String, pub detail: Option<String>,
                         pub start_min: Option<u16>, pub end_min: Option<u16>,
                         pub kind: EntryKind, pub link: Option<String> }
pub enum EntryKind { Working(ShiftState),
                     Absent { kind: AbsenceKind, pending: bool, partial_hours: Option<f32> },
                     Holiday, Off }

pub fn month_span(year: i64, month: u32, week_start: Weekday) -> (i64, i64, usize);
pub fn visible_range(zoom: RosterZoom, anchor: i64, week_start: Weekday) -> (i64, i64);
pub fn step(zoom: RosterZoom, anchor: i64, delta: i64) -> i64;
pub fn sort_entries(entries: &mut [RosterEntry]);
pub fn iso_week(day: i64) -> u32;

// src/components/week_view/types.rs (the crate's date math, now public)
pub fn days_in_month(year: i64, month: u32) -> u32;
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64;

// src/patterns/month_roster/component.rs
#[component]
pub fn MonthRoster(
    days: Signal<Vec<RosterDay>>,
    anchor_day: Signal<i64>,
    today: Signal<i64>,
    #[optional] zoom: Signal<RosterZoom>,
    #[optional] now_min: Signal<Option<u16>>,
    #[optional, default Monday] week_start: Signal<Weekday>,
    #[optional, default Sat+Sun] weekend: Signal<WeekdaySet>,
    #[optional] texts: Signal<MonthRosterTexts>,
    #[optional] layout: Signal<Option<RosterLayout>>,
    #[optional] label: MaybeProp<String>,          // the grid's name; default the title
    #[optional] on_navigate: Option<Callback<(RosterZoom, i64)>>,
    #[optional] on_entry_activate: Option<Callback<String>>,
    #[optional] actions: Option<Children>,         // host controls in the header (Office's office select)
    #[optional] class: &'static str,
    #[optional] node_ref: NodeRef<Div>,
) -> impl IntoView
```

`month_span` returns the first and last epoch day of the WHOLE weeks covering the month and the
row count, 4 to 6, never padded to 6. Verified fixtures: October 2026 (a Thursday start) with a
Monday week is 2026-09-28..2026-11-01, 5 rows; with a Sunday week 2026-09-27..2026-10-31, 5 rows;
August 2026 with Monday is 2026-07-27..2026-09-06, 6 rows; February 2027 with Monday is
2027-02-01..2027-02-28, exactly 4 rows (5 with Sunday).

`step(Month, ..)` keeps the day of the month, clamped: Jan 31 + 1 month = Feb 28. `step(Week, ..)`
moves 7 days and `step(Day, ..)` moves 1.

`visible_range`: Month is the `month_span` of the anchor's month; Week is the seven days from
`week_start_for_weekday(anchor, week_start)`; Day is the anchor alone.

Without `on_navigate` the roster is display-only: no prev/next/today/zoom controls, the day numbers
are text and the grid has no roving focus (`role="grid"` only when interactive, as RosterGrid does).

## 5. DOM contract (stable hooks; tests never use position)

| Hook | On |
|---|---|
| `data-month-roster`, `data-zoom="month|week|day"`, `data-layout="auto|grid|agenda"` | the root |
| `data-month-roster-title` | the h2 title |
| `data-month-roster-action="previous|today|next|zoom-month|zoom-week|zoom-day|week"` | every control (print hides them) |
| `data-month-empty` | the "No one is scheduled in..." line |
| `data-month-grid` | the month `<table>` |
| `data-day="YYYY-MM-DD"`, `data-outside`, `data-today`, `data-holiday`, `data-closed`, `data-weekend`, `data-unknown`, `data-empty` | a month date box (`td`), a week column, the day view root |
| `data-day-number` | the day-number button (Month) / date button (Week) |
| `data-staffing`, `data-staffing-short` | the staffing line |
| `data-entry="<id>"`, `data-entry-state="full|half|vacation|out-of-office|sick|other|holiday|off"`, `data-entry-pending` | a chip |
| `data-overflow`, `data-overflow-toggle`, `data-overflow-list` | the "+N" label, its checkbox, the hidden rest |
| `data-week-column`, `data-week-out` | a week column, its "Out (N)" heading |
| `data-day-axis`, `data-hour="8"`, `data-day-bar="<id>"`, `data-day-all-day`, `data-now-line`, `data-out-group="vacation|..."`, `data-day-off-line` | the Day view |

## 6. Month grid (`.2`, `.3`)

- `<table data-month-grid class="w-full table-fixed border-collapse border border-table-grid">`,
  an 8th leading column for the week buttons (`<th scope="row">`, `w-10`), weekday headers
  `<th scope="col" class="bg-table-header text-table-header-content">` with the short name visible
  and the long name `sr-only select-none`.
- A date box: `h-28 border border-table-grid p-1 align-top`; outside `bg-base-200/40
  text-base-content/75`; closed `bg-base-200/60`; holiday `bg-accent/10`; weekend `bg-base-200/30`;
  today `ring-2 ring-inset ring-primary` plus the visible word "Today".
- The head: the day number as a button (`on_navigate(Day, day)`), "Today", "Holiday: X" (with
  "(half day)" when `half_day`), "Closed: reason", then "3 of 4 needed" (`font-semibold` when short).
- Chips: a `<ul>`, one `<li>` each, three lines: the name (`truncate font-semibold`,
  `translate="no"`), the detail (the host's `detail`, else "08:00-16:30" from the times, else
  nothing), the state words. A chip is a `<button>` when `on_entry_activate` is set, a link when
  the entry has `link`, otherwise text.
- Overflow: the first 4 chips show. The rest stay in the DOM in a list that a native peer checkbox
  labelled "+5" (spoken "5 more people") reveals. The list also shows itself while focus is inside
  it, and always in print.
- **Keyboard (interactive roster only):** `table role="grid"`, `td role="gridcell"`. ONE date box
  holds `tabindex=0`, the others `-1`. The active box is the remembered focus when it is still in
  the shown month, else today when it is in the month, else the anchor. Controls inside every
  non-active box are `tabindex=-1`, so Tab enters the grid once, visits only the active box's own
  controls and then leaves. On a box: ArrowLeft/Right = 1 day, ArrowUp/Down = 1 week (clamped to the
  rendered span), Home/End = the ends of the row, Enter = `on_navigate(Day, day)`. On a box or inside
  one: PageUp/PageDown = `on_navigate(Month, step(Month, active, -/+1))`, and the focus lands on the
  same day of the new month once the host has rendered it. Escape inside a box returns focus to the
  box. Alt/Ctrl/Meta chords are left to the browser.
- Each box's `aria-label`: "Wednesday, Oct 14, 2026, Today, 5 working, 4 out, Holiday: X" or
  "..., no one scheduled" / "..., no schedule published". Never "0 people". Today is the ring and
  the word, not `aria-current` alone.
- The data refresh keeps focus: boxes are keyed by date, so a new `days` value restyles them in
  place. Chips are keyed by their whole content (the `<For>` key lesson).

## 7. Zoom and Week view (`.4`)

- A segmented control "Month | Week | Day" (`join`, `aria-pressed`), each a proposal
  `on_navigate(zoom, anchor)`. From Month zoom, Week and Day use the ACTIVE day rather than the
  anchor, so a keyboard user can arrow to a day and then choose its week.
- The title follows the zoom: "October 2026", "Oct 12 - 18, 2026" (the localised
  `week_range_label`), "Wednesday, October 14, 2026". Prev/next step by the zoom unit; Today keeps
  the zoom.
- Each month row's leading header holds a button "Show the week of Oct 12, 2026": the visible text
  is the ISO week ("W42") when the week starts on Monday, and a chevron otherwise. It calls
  `on_navigate(Week, row start)`. It takes no tab stop (keyboard users have the switch).
- Week view: seven columns (`grid grid-cols-7`), each a `<section>` named by its head. The head
  (sticky at the top, so the columns scroll together under it) holds the weekday and the date as a
  button that zooms to Day, the today ring and word, holiday/closed marks and staffing. Then EVERY
  entry (no +N) in the epic's order; working chips show "HH:MM-HH:MM" when times are known; then an
  "Out (N)" sub-heading over the absences; then holiday/off chips.
- Escape in Week = `on_navigate(Month, anchor)`.

## 8. Day view (`.5`)

- The day head (holiday/closed banner, staffing line).
- A coverage timeline: one row per working person with times, sorted by start, the name left and a
  bar from `start_min` to `end_min` on a shared hour axis. The axis is 07:00-20:00 by default,
  widened to whole hours covering every entry. Half bars use the Half colour. Working entries
  without times sit in an "All day" group above the axis. A now line appears only when the day is
  today and `now_min` is given. Bars are tokens only (`bg-*/15 border border-*`), positioned in
  percent of the axis so the pane can resize.
- "Out today (N)", grouped by kind (Vacation, Out of office, Sick, Out), each person with their
  partial-day hours, pending ones marked "Requested".
- One line each for "Holiday: names" and "Off: names".
- Display-only parts take no tab stops. With `on_entry_activate`, a row is a button named
  "Ana Lopez, working 08:00 to 16:30".
- Escape in Day = `on_navigate(Week, anchor)`.

## 9. Narrow container (`.6`) and print (`.7`)

- The root is `@container w-full`. Below `@2xl` (42rem): the month is an agenda (table, body, rows
  and boxes `display:block`; weekday header, week buttons and outside days hidden; each in-month day
  shows its h3 full-date heading, which is `sr-only` in the grid); the week stacks
  (`grid-cols-1`); the day's name column narrows and odd hour labels hide. `layout` forces
  `Grid` or `Agenda` regardless of width (the classes are chosen in Rust from literal tables, so
  Tailwind sees every one).
- A container query, not `md:`: Office hosts the roster in panes, and `md:` would read the window.
  `@container` gives the root an inline size independent of its contents, so its PARENT must have
  a width (a block, or a flex item with a width); documented in the guide.
- Print: `print:hidden` on every `[data-month-roster-action]` element and on the "+N" label; the
  overflow list is `print:not-sr-only print:flex`; date boxes are `print:h-auto`. A host sets
  `@page { size: landscape }` (the demo does, the guide documents it). The state words carry the
  meaning without colour.

## 10. Tests

- Native (`src/patterns/month_roster/tests.rs`, in `cargo xtask verify`): the four verified spans,
  Jan 31 + 1 month = Feb 28, a Sunday-start week range, the in-day order, `iso_week`,
  `visible_range` per zoom, the roving moves and clamps, the accessible names (never "0 people",
  unknown vs empty), the axis widening and bar geometry, the absence grouping, the class tables
  (every literal present in the `@source inline` block), and EN/ES completeness.
- Browser (`tests/month_roster_smoke.rs`, `#[ignore]`d, lane `cargo xtask test-month-roster`
  inside `verify-full`): one test per child's CHECK, each proven by its BREAK before landing, plus
  axe in light and dark. The page joins the layout and style audits.
- Demo: `/components/month-roster`, a fixture office of 24 people in October 2026 (a US holiday on
  12 Oct, a half-day holiday, a closed day, a 9-entry day, approved and pending vacations,
  out of office, sick, partial hours, two unknown days), an EN/ES toggle, and the controlled
  zoom/anchor state shown in testids.

## 11. Consumers

4iiz-Office Staff > Monthly Schedule (`op-uclmv`, today a RosterGrid), then its 30-day leave
calendar and My Schedule month. Office's DTOs already carry these shapes (`PeopleScheduleDayDto`:
shift label/start/end/holiday/half_day/absence_type_code; `StaffLeaveCalendarEntryDto`:
approval_status, partial_day_hours; `PeopleHolidayDto`: label_en/es, half_day).
