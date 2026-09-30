# Day Planner Week view: design

**Date:** 2026-09-29.
**Requested by:** the owner, in a leptos-daisyui-rs session. The owner chose the "lean week API" path.
**Upstream:** leptos-daisyui-rs `ldui-9sip` adds WeekView's interactive planner contract.
**Office bead:** `op-lcvat`. On 2026-09-30 the office-builder granted the Office half to an ldui session, on the lane branch `op-lcvat-week-view`. The week_view re-vendor rides that lane as its first commit.
**Status (2026-09-30):** built. `op-lcvat-week-view` = `0476c197` on Office origin, cut from `df6ba2d71` (0930h). It is compiled locally only, so the builder's `cargo make stage-test` on integration remains the gate. See "As built" at the end for where the implementation differs from this plan.
**Do not inherit `op-aa4eo`:** the day grid's builder plots carried-forward rows by minutes of day, with no check on their date. The week must place every block by its actual date, and the narrowing below guarantees that.

## Goal

The Day Planner gets a **Week** tab next to **Plan**. It shows the subject worker's seven days side by side and supports the same planning gestures as the day grid:

- selecting a block;
- opening the edit dialog;
- dragging a rail card onto a day and a time;
- moving a block in time (Up/Down) or across days (Left/Right) with the keyboard;
- clicking a day header to open that day on the Plan tab.

All seven days are working days: this is a seven-day workweek, and none are shaded. The week starts on Monday, matching the firm's reporting weeks (`current_week` is a Monday).

## Non-goals

- Week-level completion, reopen or checklist actions. Those stay on the Plan tab.
- A month view.
- Changing the day bundle or `GET /api/page`.

## Data: one read-only typed projection

The existing precedent is `GET api/day-planner/views/completed`: the `DayPlannerCompletedViewRead` marker in `endpoints_reports.rs` and the handler in `routes_day_planner_views.rs`. The week projection mirrors it.

- **Route:** `GET api/day-planner/views/week?start=YYYY-MM-DD[&worker=]`.
  - `start` is the first day of the week. The server serves exactly seven consecutive dates from it and does not normalise to a Monday.
  - If `worker=` is absent, the worker resolves from the session identity, as the completed view does.
- **Marker:** `DayPlannerWeekViewRead`, with `Response = DayPlannerWeekViewV1`.
- **Snapshot:** `surface_bundles::with_snapshot` gives one row-cache connection. On it the handler looks up the office zone once (`office_zone_local`, falling back to the tier clock), then runs the bundle's `load_day_local` and `load_worker_schedule_dto_local` once per date, seven times each. Every day is therefore read from the same committed WAL snapshot, which keeps the page contract's "one atomic snapshot" promise.
- **Narrowing:** `load_day_local` carries earlier-scheduled open work forward. The week keeps only the assignments whose `scheduled_start` falls in that day's half-open `[00:00, 24:00)` window in the office zone, using `office_day_bounds`. Otherwise a carried row would repeat in every column.
- **Zone:** the week uses one zone source, `office_zone_local`. That is the source the bundle's day and the completed view use. It is not `AppState::load_day`'s bare tier clock.
- **Offsets:** each day carries `utc_offset_minutes`, the zone's offset at local noon on that date. A week that crosses a DST change converts both ways correctly, which the page's single remembered offset cannot do.
- **Schedule range:** the calendar mirror covers today ± 7 days. A date outside that returns its schedule section as unavailable, day by day, rather than as empty.

```rust
// crates/office-perf-dto/src/day_planner_views.rs
// Plain structs only. KNOWN_REFUSALS stays at 85: no ViewSectionV1 and no
// internally tagged enum.
pub struct DayPlannerWeekViewV1 {
    pub worker_ref: String,
    /// The requested first day, `YYYY-MM-DD`.
    pub start: String,
    /// The IANA zone every day was cut in. `None` means no clock is
    /// configured: assignments are unavailable, and the page refuses to
    /// schedule, exactly as it does today.
    #[serde(default)] pub zone: Option<String>,
    /// Exactly seven days, in date order.
    pub days: Vec<DayPlannerWeekDayV1>,
}
pub struct DayPlannerWeekDayV1 {
    pub date: String,
    #[serde(default)] pub utc_offset_minutes: Option<i32>,
    /// Assignments SCHEDULED inside this day's window.
    #[serde(default)] pub assignments: Vec<AssignmentDto>,
    /// `Some(detail)` when the mirror could not answer for this day.
    #[serde(default)] pub assignments_unavailable: Option<String>,
    /// Venue-local appointments from the worker schedule.
    #[serde(default)] pub events: Vec<DayEventDto>,
    #[serde(default)] pub events_unavailable: Option<String>,
}
```

## Registration checklist

This mirrors the completed view. A read-only GET needs no `mutation_contract` entry, cache-policy entry or doorbell.

1. **DTO:** add the types, the `wire_snapshot/registry.rs` lines, then regenerate with `cargo make wire-snapshots-update` followed by a plain run.
2. **Endpoint:** add the marker to `endpoints_reports.rs` and its `endpoint.rs` check line.
3. **Route:** add the handler to `routes_day_planner_views.rs`. Because it is the same file, it needs no new `pg_boundary` corpus line. Then add the `registered_at` assert and the `typed_get` route in `app_router.rs`.
4. **Allowlist:** add `Operation::endpoint::<dto::DayPlannerWeekViewRead>()` to `DAY_PLANNER.operations` in `surface_portfolio.rs`.
5. **Harness:** add a mock of the route to `surfaces/day-planner/harness/server.mjs` and put it in `knownApiPaths`.
6. **Tests:** handler unit tests in the same file, covering:
   - exactly 7 days, in order;
   - narrowing drops a carried-forward row;
   - the DST offset differs across a changeover week;
   - a date outside the calendar range reports events as unavailable, not empty.

## Surface

- **Tabs:** add `<Tab tab_key="week">` to `TabSet id="day-planner-views"`. The source-text tests allow a new `tab_key`; `plan` stays and `completed` stays absent. Its `<TabPanel tab_key="week">` hosts ldui `WeekView`.
- **Fetching:**
  - The week is the Monday-start week containing the displayed `business_date`.
  - The page fetches it through `endpoint::<DayPlannerWeekViewRead>("week").query("start", ..).query("worker", ..).read()`.
  - Its generation counter is separate from `refresh_scope`, so a day refresh never discards the week response or the reverse.
  - While the Week tab is active, ‹ › step the business date by ±7.
- **Events:** each day's scheduled assignments and appointments become `CalEvent`s.
  - The day index is the day's offset from `start`.
  - Minutes are the wall clock, converted with that day's own `utc_offset_minutes`.
  - A side table maps each index to its row kind (Assignment id or Appointment). The day grid does the same with `scheduler_projection`.
- **Interaction**, reusing the day grid's gates (`schedule_writes_available`, `actions_blocked`, `edit_open`, self-service only):
  - Clicking or pressing Enter selects the block. Enter or a double-click on an assignment opens `AssignmentEditDialog` through the existing `open_edit`.
  - **Up/Down** requests the same date with `start ± 30` minutes. **Left/Right** requests `date ± 1` at the same start. The move is refused past the week's first or last day.
  - A rail card or scheduled block dropped on `[data-week-columns]` goes through `week_slot_at(x, y, band)`, then snaps to 30 minutes to give `(day, minute)`.
  - Clicking a day header does `reload_date(date)` and activates the Plan tab.
- **Writes:** a dated reschedule. `requested_span` already takes the date, so it gains an `offset_minutes` parameter and the day grid passes `office_offset_minutes()`. The week reconciles by re-reading the week view, and also refreshes the day scope when the move touched the displayed date.
- **Strings:** EN and ES for the tab name, the week range label, the Previous/Next week labels, and the week section's unavailable notices. Weekday names are ES `lun … dom`. Accessible names use the ES form `"{title}, {weekday} {d}, de HH:MM a HH:MM"`.
- **Vendor:** the lane's first commit re-vendors `src/components/week_view/*` from the pushed ldui commit, following the op-4v27r precedent (Office `4ac0f0889`). It bumps the pin in every surface `Cargo.lock` and the root lock, and runs `scripts/vendor_provenance.py --update` in the same commit. Declared deltas stay untouched.

## Validation

- `cargo make iterate-day-planner`, then `cargo make stage-test`.
- After deploy: the DOM/axe pass on `/day-planner/`, with the Week tab both open and closed.
- Live check of all four gestures: a keyboard day-move, a rail drop onto another day, a header drill-down, and the EN/ES toggle.

## As built (2026-09-30)

The branch has three commits:

- `98ead3b4`: the re-vendor, pinned at ldui `5f7158c`.
- `11792f55`: the feature.
- `0476c197`: placement through `DayAssignment::scheduled_on`.

Where the build differs from the plan above:

- **Offsets come from the SCHEDULE clock, not the office zone.** Each day's
  `utc_offset_minutes` is `configured_timezone()`'s offset at local noon. That
  is the clock the page's read edge (`from_dto`) and write edge
  (`wall_clock_to_utc`) convert under, so week writes agree with the Plan tab.
  The office zone is only where each day's window is cut, and that is
  `office_zone_local`, the bundle's rule. The wire names both:
  `day_zone` and `schedule_zone`.
- **Section states are a plain struct plus a unit enum**
  (`DayPlannerWeekSectionV1`, `DayPlannerWeekSectionStateV1`: fresh, stale,
  never_loaded, unavailable) instead of `*_unavailable: Option<String>`. The
  fixture still builds every type, so `KNOWN_REFUSALS` stays 85.
- **Blocks are placed by the row's actual wall date.** The column is the week
  day where `DayAssignment::scheduled_on(date)` holds. That predicate is 0930h's,
  the Plan tab's own op-aa4eo rule, so the two tabs share one definition. A row
  whose wall clock falls outside the week is counted in a notice, never drawn
  elsewhere.
- **Blocks are ordered by assignment id, never by time.** `WeekView` keys nodes
  by index, so a re-read that reordered them would hand a moved block's focus to
  another assignment.
- **The unplanned rail moved beside both tab panels.** The workspace grid holds
  the panels in its first column, so a rail card drops onto the week too.
- **A move is confirmed on the target date's own offset.** The pending move
  lives in a signal, so whichever week read lands last settles it.
- **`requested_span` delegates to `requested_span_at(.., offset)`**, and
  `from_dto` and `day_event` became `pub(crate)`. The day grid builder was not
  touched; op-aa4eo was the builder's, and shipped in 0930h.
- **ldui gained `all_day_label`** (ldui-ms76). `WeekView` had one English
  literal left, and the Spanish tab passes "Todo el día".
- **The wire snapshots, fingerprint and vendor provenance were hand-written**
  with self-checking scripts, because `cargo` and `vendor_provenance.py
  --update` could not run in the lane.
- **Local compile, on the owner's word.** `office-perf-dto` clippy + 427 tests.
  Day-planner clippy (native and wasm32) + 72 tests. `office-perf-api` clippy
  `--bins` + 220 filtered tests. The xtask digest pin.
