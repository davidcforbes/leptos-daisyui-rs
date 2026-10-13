# MonthRoster

`patterns::MonthRoster` (epic `ldui-5lmt`) is an opinionated staff calendar. It shows who is
scheduled to work each day and who is out on vacation, out of the office or sick, at three zoom
levels of one data set. It is a port of the 4iiz-kit `MonthRoster`, so 4iiz-Office and
4iiz-People share one look. Design: `docs/superpowers/specs/2026-10-11-month-roster-design.md`.

| Zoom | What it draws |
|---|---|
| Month | The kit's grid of whole weeks. Each date box shows its marks, staffing and people: 4 chips and "+N" for the rest |
| Week | Seven day columns listing EVERY entry: working first, then "Out (N)", then holiday and off |
| Day | A shift coverage timeline (one bar per working person on an hour axis) and "Out today", grouped by kind |

Demo: `/components/month-roster`. Browser lane: `cargo xtask test-month-roster` (inside
`verify-full`).

## The contract: controlled, data-agnostic, it proposes and never edits

The roster never fetches, never reads a clock and never mutates its data. The host owns the zoom
and the anchor day. Every prev / next / Today / zoom switch / week button / day number / Enter /
PageUp / PageDown / Escape is a proposal, `on_navigate((zoom, epoch_day))`, that the host answers
by setting both signals:

```rust
use leptos::prelude::*;
use leptos_daisyui_rs::components::{ShiftState, Weekday};
use leptos_daisyui_rs::patterns::{
    AbsenceKind, MonthRoster, MonthRosterTexts, RosterDay, RosterEntry, RosterZoom,
    roster_visible_range,
};

let zoom = RwSignal::new(RosterZoom::Month);
let anchor = RwSignal::new(20_740_i64); // 2026-10-14, days since 1970-01-01

// Load exactly the days the roster will draw.
let range = Memo::new(move |_| roster_visible_range(zoom.get(), anchor.get(), Weekday::Monday));

view! {
    <MonthRoster
        days=days                       // Signal<Vec<RosterDay>>
        anchor_day=anchor
        today=today_epoch_day           // from the HOST's clock and time zone
        zoom=zoom
        now_min=now_minutes             // Option<u16>, for the Day view's now line
        texts=texts                     // MonthRosterTexts::default() / ::es()
        label="Raleigh office schedule"
        on_navigate=move |(z, day): (RosterZoom, i64)| { zoom.set(z); anchor.set(day); }
        on_entry_activate=move |id: String| open_person(id)
    >
    </MonthRoster>
}
```

- **Epoch days are `i64`**, like `WeekView` and the crate's date math (`days_from_civil`,
  `civil_from_days`). A host holding `i32` writes `.into()`.
- **Without `on_navigate` the roster is display-only:** no header controls, no week buttons, plain
  day numbers and no roving focus. With only `on_entry_activate` the chips are buttons and the grid
  keeps its single tab stop.
- **The roster never edits.** Opening a day, adding someone and removing an entry are proposals
  the host answers by changing `days` (see [Editing hooks](#editing-hooks-ldui-k145)). There is no
  move or drag. Chip activation goes to the host.
- `actions` is a slot at the end of the header for the host's own controls (Office puts its office
  select there).

## Data

```rust
RosterDay::new(day)
    .holiday("Columbus Day")             // or .half_day_holiday(..)
    .closed("Building maintenance")
    .staffing(5, 6)                      // "5 of 6 needed", bold when short
    .entries([
        RosterEntry::working("a1", "Ana Lopez", ShiftState::Full).times(8 * 60, 16 * 60 + 30),
        RosterEntry::working("a2", "Mia Chen", ShiftState::Half).detail("Early"),
        RosterEntry::absent("v1", "Omar Haddad", AbsenceKind::Vacation),
        RosterEntry::absent("v2", "Rosa Ortiz", AbsenceKind::Vacation).pending(),
        RosterEntry::absent("s1", "Hannah Lee", AbsenceKind::Sick).partial_hours(4.0),
        RosterEntry::holiday("h1", "Tom Baker"),
        RosterEntry::off("o1", "Sam Patel"),
    ])
```

- **A day ABSENT from `days` is unknown** and reads "No schedule published". **A present day with
  no entries is known empty** and reads "No one scheduled". The roster never says "0 people". A
  month whose published days are all empty shows "No one is scheduled in October 2026" above a grid
  that still renders.
- **Order inside a day** is fixed: working (by start time, untimed last, then name), then out
  (Vacation, Out of office, Sick, Other, then name), then holiday, then off.
- **Every chip says its state in words** ("Full shift", "Vacation, Requested", "Sick, 4 h"). The
  colour and the accent bar repeat it: solid for working, dashed for everything else. A pending
  absence keeps its border and has no fill.
- `Working` takes `RosterGrid`'s `ShiftState`; only `Full` and `Half` mean work. `Working(Off)`,
  `Working(Holiday)` and `Working(Leave)` are normalised to `Off`, `Holiday` and an `Other`
  absence, so a misuse never paints work that was not rostered.
- **Let the roster format times.** `detail` is the host's shift label. Without one, the roster
  writes "08:00-16:30" from `times`. Times are locale-free, so a language switch changes no entry,
  and the chips are renamed in place rather than rebuilt.

## Editing hooks (ldui-k145)

4iiz-Office edits People schedules from its Monthly Schedule. Three optional props let a host open
its own dialogs; a host that passes none of them gets exactly the read-only roster.

```rust
RosterDay::new(day).addable()                       // this day takes a "+"
RosterEntry::working(id, name, ShiftState::Full).removable()   // this entry takes an "x"

view! {
    <MonthRoster
        // ...
        on_day_activate=move |day: i64| open_day_dialog(day)
        on_day_add=move |day: i64| open_add_form(day)
        on_entry_remove=move |id: String| ask_reason_then_remove(id)
    />
}
```

- **`on_day_activate`** opens a day from a double-click on a date box (Month) or a day column
  (Week), a touch double-tap, or `Enter` on the focused box or column. A double-click or a
  double-tap on one of the day's own controls (a chip's button or link, the day number, "+", "x",
  "+N") keeps that control's meaning. The double-click's word selection is cleared. With the hook
  set, `Enter` no longer zooms, so the day number joins the active box's Tab order and still zooms
  on a click or `Enter`. In the Week view, each column takes a tab stop. Boxes and columns carry
  `touch-action: manipulation`, so the browser does not zoom on the second tap.
- **`on_day_add`** draws a labelled "+" (`data-day-add="2026-10-21"`, named "Add someone on
  Wednesday, Oct 21, 2026") at the top right of each day the host marks `addable`, in all three
  zooms. The host decides which days: Office marks only the days after today.
- **`on_entry_remove`** draws a labelled "x" (`data-entry-remove="<entry id>"`, named "Remove Mia
  Chen from Wednesday, Oct 21, 2026") beside each entry the host marks `removable`. In a chip it sits
  beside the chip's own `on_entry_activate` button or link, never inside it. In the Day timeline it
  sits beside the person's name.
- Every new control is a real `<button type="button">` with its name from `texts` (`add_to_day`,
  `remove_entry`, both EN and ES). In the month grid each one follows the box's roving Tab order like
  the chips do, and `Escape` returns from it to its box. They are `print:hidden`.
- **Focus after a removal is the host's job.** When the host's answer removes the entry, its "x"
  goes with it. Return focus to the day's box, `td[data-day="<iso date>"]`, or the Week column,
  `[data-week-column][data-day="<iso date>"]`.

## Keyboard (the month grid)

| Key | Effect |
|---|---|
| `Tab` | Enter the grid at the active day (today in the month, else the anchor); the next Tab visits that day's own chips and "+N", then leaves |
| Arrows | One day / one week, within the shown weeks |
| `Home` / `End` | The ends of the week row |
| `PageUp` / `PageDown` | Propose the previous / next month, same day of the month; focus lands on it once rendered |
| `Enter` | Propose the Day zoom of that day; with `on_day_activate`, open the day instead |
| `Escape` | From a chip back to its day; in Week back to Month; in Day back to Week |

Each date box is a `gridcell` named in words: "Wednesday, Oct 14, 2026, Today, 5 working, 4 out,
5 of 6 needed". Today is the outline AND the word. From the month, the Week and Day switch buttons zoom
to the active day, so a keyboard user arrows to a day and then picks its week. The row week buttons
take no tab stop for that reason.

## Layout follows the container, not the window

The root is an `@container`. Below 42rem of the ROOT's width the month becomes an agenda: no
weekday header, no adjacent-month days, and a visible date heading on every day. The week stacks
and the day's name column narrows, with odd hour labels hidden. Office hosts the roster in panes,
where a viewport breakpoint would read the window instead. Force a layout with
`layout=Some(RosterLayout::Agenda)` (or `Grid`).

`container-type: inline-size` makes the root's width independent of its contents. **Give its
parent a width**: a block element, or a flex item with `w-full` / `flex-1`. A bare `<div>` in a
`flex` row collapses the roster to nothing.

## Print

The header controls, the zoom switch, the week buttons and the "+N" labels are `print:hidden`. The
overflow lists are `print:not-sr-only print:flex`, so every entry prints. Set the page orientation
in the host:

```css
@page { size: landscape; }
```

## Styling

The look is the kit's: `table-fixed border-collapse border-table-grid`, header cells
`bg-table-header`, date boxes `h-28 p-1`, adjacent-month days `bg-base-200/40`, closed
`bg-base-200/60`, holiday `bg-accent/10`, weekend `bg-base-200/30`, and today
`outline-2 -outline-offset-2 outline-primary`. The kit draws a `ring-2 ring-inset`; the roster
uses an outline because a box-shadow ring is a DEPTH finding in the style audit. The weekend defaults to Saturday and Sunday (`weekend` prop). The
week-button column's cells carry `text-table-header-content` with the header fill, and so does the
week button itself, which uses no daisyUI `btn` class (`WEEK_BUTTON`, ldui-u0yr): a `btn-ghost`'s rest
colour depends on the daisyUI build, and the build 4iiz-Office serves drew it in base-content on the
navy header (1.06:1). Its hover is a translucent white (`hover:bg-white/15`), never a light fill
under the light text. Adjacent-month days dim their text to `/85`, not the crate's usual `/75`,
because a chip's 12px text sits on a tint over that surface (ldui-on0i, measured by Office at 4.4:1).
The
class list for `input.css` is in the component's rustdoc (`@source inline(...)`). A native test
fails if a class the roster can emit is missing from it.

The roster declares no `data-ld-pattern`: PixelProof's parity catalogue has no MonthRoster (see
ldui-wra5). Its root carries `data-month-roster`, `data-zoom` and `data-layout`. The full hook
table is in the design spec, section 5.
