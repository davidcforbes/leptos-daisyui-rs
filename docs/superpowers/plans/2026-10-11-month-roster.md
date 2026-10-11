# MonthRoster: implementation plan

Spec: `docs/superpowers/specs/2026-10-11-month-roster-design.md`. Epic `ldui-5lmt`.

Order follows the bead dependencies (`.1` before `.2`; `.3`-`.7` after `.2`; `.5` after `.4`).
Edits come first and compile once at the end (global fix-cycle rule): no cargo while a step is
half-written, one `cargo check` per crate, then fmt, clippy, tests, ONE demo build and the lanes.

## Task 1 (`.1`): types and date math

- `crates/ldui-design/src/components/month_roster.rs`: `RosterZoom`, `AbsenceKind`,
  `RosterLayout` with `as_str()` slugs and the absence tint/border class tables; register the
  module in `components/mod.rs`.
- `src/components/week_view/types.rs`: public `days_in_month`, `days_from_civil`; point
  `entity_table/date_filter.rs` and `gantt/timeline/scale.rs` at them.
- `src/patterns/month_roster/{mod,style,types,texts}.rs`: the data model and builders,
  `month_span`, `visible_range`, `step`, `sort_entries`, `iso_week`, the roving move, the counts,
  the accessible names, the axis and bar geometry, the absence grouping; `MonthRosterTexts` with
  `Default` and `es()`.
- `src/patterns/month_roster/tests.rs`: the native tests named in the spec, section 10.
- Re-export from `src/patterns/mod.rs`.

## Task 2 (`.2`, `.3`, `.6`, `.7`): the month grid

- `src/patterns/month_roster/component.rs`: root, header (title, nav, zoom switch, actions slot),
  the empty line, the month table keyed by row start and date, date boxes, chips, overflow, roving
  focus and keys, container-query and print classes, the `@source inline` block.
- `demo/src/demos/month_roster.rs` + `mod.rs` + route in `demo/src/main.rs` + sidebar entry in
  `demo/src/core/layout.rs`: the October 2026 fixture office, EN/ES toggle, zoom/anchor state in
  testids, a resizable host (`#month-roster-host`), `@page { size: landscape }`.

## Task 3 (`.4`, `.5`): week and day views

- `src/patterns/month_roster/week.rs`, `day.rs`: the two zooms, Escape handling.

## Task 4: browser lane and audits

- `tests/month_roster_smoke.rs`: one test per CHECK (month grid, keyboard, week, day, narrow,
  print, axe light/dark, EN/ES renames the same nodes).
- `xtask/src/main.rs`: `month_roster_step()` (`test-month-roster`), pushed in `full_steps()`,
  dispatched by name, listed in the usage line; `doc/ci-cd.md` and CLAUDE.md lane list.
- `tests/layout_audit_smoke.rs`, `tests/style_audit_smoke.rs`: the page with its measured ceilings
  and its own `audit_test!` / `style_audit_test!` invocation.

## Task 5: docs and close

- `doc/components/month-roster.md`, a `doc/dev-log/2026.md` entry.
- Verify: per-crate check, fmt (per package), clippy (per crate, CI flags), lib tests, the
  lane (count rises), each BREAK proven then reverted, layout/style audits, `cargo xtask verify`.
- Close the children on their CHECKs, then the epic; commit with the beads export; push to
  fork/main; message 4iiz-office with the commit.
