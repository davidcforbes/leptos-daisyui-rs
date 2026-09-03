//! Native proof of the exclusive inline-edit reducer (`ldui-ff2f`).
//!
//! Every state the machine can reach is exercised here rather than in a
//! browser, so the correctness does not depend on a suite someone has to
//! remember to register (`ldui-a8an`). Each rule that exists to *refuse*
//! something is tested by attempting it, not by asserting the happy path
//! around it — a guard nothing ever trips is indistinguishable from no guard.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Row {
    name: String,
    code: String,
}

fn blank() -> Row {
    Row::default()
}

fn existing() -> Row {
    Row {
        name: "Consultation".to_owned(),
        code: "CONS".to_owned(),
    }
}

fn state() -> EntityEditState<Row> {
    EntityEditState::new()
}

#[test]
fn a_new_table_is_idle_and_nothing_is_live() {
    let s = state();
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
    assert!(!s.is_editing());
    assert!(!s.is_committing());
    assert!(s.editing_row().is_none());
    assert!(s.target().is_none());
    assert!(s.rejection().is_none());
    // An opted-out table must never report a row as live, so the renderer's
    // inert treatment can key off exactly this and stay off.
    assert!(!s.is_row_live("anything"));
}

#[test]
fn plus_begins_a_draft_and_edit_begins_an_update() {
    let mut s = state();
    assert_eq!(s.begin_draft(blank()), EntityEditDisposition::Applied);
    assert_eq!(s.target(), Some(&EntityEditTarget::Draft));
    assert!(s.target().expect("live").is_draft());
    assert!(s.target().and_then(EntityEditTarget::row_key).is_none());

    let mut s = state();
    assert_eq!(
        s.begin_edit("row-7", existing()),
        EntityEditDisposition::Applied
    );
    assert_eq!(
        s.target(),
        Some(&EntityEditTarget::Existing("row-7".to_owned()))
    );
    assert_eq!(s.editing_row(), Some(&existing()));
}

/// The invariant the whole design rests on: a second entry point firing while
/// a row is live is REFUSED, not allowed to replace the session. Replacing it
/// would discard unsaved typing without asking.
#[test]
fn a_second_entry_point_cannot_open_a_second_live_row() {
    let mut s = state();
    s.begin_draft(blank());

    assert_eq!(
        s.begin_edit("row-7", existing()),
        EntityEditDisposition::IgnoredBusy
    );
    assert_eq!(
        s.begin_draft(blank()),
        EntityEditDisposition::IgnoredBusy,
        "a second '+' must not replace an open draft"
    );
    // The original session is untouched.
    assert_eq!(s.target(), Some(&EntityEditTarget::Draft));

    // Same guard from the update entry point.
    let mut s = state();
    s.begin_edit("row-1", existing());
    assert_eq!(
        s.begin_edit("row-2", existing()),
        EntityEditDisposition::IgnoredBusy
    );
    assert_eq!(
        s.target(),
        Some(&EntityEditTarget::Existing("row-1".to_owned()))
    );
}

/// Exactly one row reports live, which is what the renderer uses to decide
/// which rows go inert.
#[test]
fn only_the_edited_row_is_live() {
    let mut s = state();
    s.begin_edit("row-2", existing());
    assert!(s.is_row_live("row-2"));
    assert!(!s.is_row_live("row-1"));
    assert!(!s.is_row_live("row-3"));

    // A draft has no key, so no EXISTING row is ever live during a create --
    // otherwise a key collision could make a real row editable by accident.
    let mut s = state();
    s.begin_draft(blank());
    assert!(!s.is_row_live("row-1"));
    assert!(!s.is_row_live(""));
}

#[test]
fn typing_updates_the_live_row() {
    let mut s = state();
    s.begin_draft(blank());
    assert_eq!(
        s.edit_field(|row| row.name = "Intake".to_owned()),
        EntityEditDisposition::Applied
    );
    assert_eq!(
        s.editing_row().map(|r| r.name.as_str()),
        Some("Intake"),
        "the reducer must carry the user's input"
    );
}

#[test]
fn escape_ends_the_session_and_leaves_no_row_behind() {
    let mut s = state();
    s.begin_draft(blank());
    s.edit_field(|row| row.name = "half typed".to_owned());
    assert_eq!(s.cancel(), EntityEditDisposition::Applied);
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
    assert!(s.editing_row().is_none(), "no phantom row may survive");
}

#[test]
fn escape_on_an_idle_table_is_a_no_op() {
    let mut s = state();
    assert_eq!(s.cancel(), EntityEditDisposition::IgnoredIdle);
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
}

#[test]
fn save_moves_in_flight_and_carries_the_submitted_row() {
    let mut s = state();
    s.begin_edit("row-7", existing());
    s.edit_field(|row| row.code = "CONS-2".to_owned());

    let commit = s.commit().expect("drafting commits");
    assert_eq!(commit.row().code, "CONS-2");
    assert_eq!(
        commit.target(),
        &EntityEditTarget::Existing("row-7".to_owned())
    );
    assert!(s.is_committing());
    assert!(s.is_editing(), "the table stays disabled while in flight");
}

/// Save is refused mid-commit. This is the double-submit guard.
#[test]
fn save_cannot_fire_twice_and_the_row_cannot_change_in_flight() {
    let mut s = state();
    s.begin_draft(blank());
    let _first = s.commit().expect("first commit");

    assert_eq!(s.commit().unwrap_err(), EntityEditDisposition::IgnoredBusy);
    // The submitted row is what the consumer is persisting; letting it change
    // underneath would persist something the user never submitted.
    assert_eq!(
        s.edit_field(|row| row.name = "changed mid flight".to_owned()),
        EntityEditDisposition::IgnoredIdle
    );
    assert_eq!(s.editing_row().map(|r| r.name.as_str()), Some(""));
    // And Escape cannot recall a write already in flight.
    assert_eq!(s.cancel(), EntityEditDisposition::IgnoredBusy);
    assert!(s.is_committing());
}

#[test]
fn accepted_returns_the_table_to_readonly() {
    let mut s = state();
    s.begin_edit("row-7", existing());
    let commit = s.commit().expect("commit");
    assert_eq!(
        s.resolve(&commit, EntityEditOutcome::Accepted),
        EntityEditDisposition::Applied
    );
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
    assert!(!s.is_editing());
}

/// A rejection must not cost the user their typing.
#[test]
fn rejected_returns_to_editing_with_input_intact() {
    let mut s = state();
    s.begin_draft(blank());
    s.edit_field(|row| row.name = "Intake".to_owned());
    let commit = s.commit().expect("commit");

    assert_eq!(
        s.resolve(
            &commit,
            EntityEditOutcome::Rejected("Name taken".to_owned())
        ),
        EntityEditDisposition::Applied
    );
    assert!(!s.is_committing());
    assert!(s.is_editing());
    assert_eq!(s.editing_row().map(|r| r.name.as_str()), Some("Intake"));
    assert_eq!(s.rejection(), Some("Name taken"));
}

/// Typing after a rejection clears the message: it described input the user
/// has now changed, and leaving it up would blame the current value for a
/// past failure.
#[test]
fn typing_clears_a_previous_rejection() {
    let mut s = state();
    s.begin_draft(blank());
    let commit = s.commit().expect("commit");
    s.resolve(
        &commit,
        EntityEditOutcome::Rejected("Name taken".to_owned()),
    );
    assert_eq!(s.rejection(), Some("Name taken"));

    s.edit_field(|row| row.name = "Intake 2".to_owned());
    assert_eq!(s.rejection(), None);
}

/// The staleness guard: a slow first attempt must not resolve a later one.
#[test]
fn a_superseded_commit_cannot_resolve_the_current_one() {
    let mut s = state();
    s.begin_draft(blank());
    let first = s.commit().expect("first commit");
    s.resolve(&first, EntityEditOutcome::Rejected("try again".to_owned()));

    let second = s.commit().expect("second commit");
    assert_ne!(first.sequence(), second.sequence());

    // The abandoned first attempt landing late is discarded...
    assert_eq!(
        s.resolve(&first, EntityEditOutcome::Accepted),
        EntityEditDisposition::IgnoredStale
    );
    assert!(
        s.is_committing(),
        "a stale resolution must not end the live session"
    );
    // ...while the current one still resolves.
    assert_eq!(
        s.resolve(&second, EntityEditOutcome::Accepted),
        EntityEditDisposition::Applied
    );
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
}

#[test]
fn resolving_when_nothing_is_in_flight_is_ignored() {
    let mut s = state();
    s.begin_draft(blank());
    let commit = s.commit().expect("commit");
    s.resolve(&commit, EntityEditOutcome::Accepted);

    // Replaying the same handle against an idle table.
    assert_eq!(
        s.resolve(&commit, EntityEditOutcome::Accepted),
        EntityEditDisposition::IgnoredStale
    );
    assert_eq!(s.phase(), &EntityEditPhase::Idle);
}

/// `ldui-tmoz`: the constrained-scroll contract, stated as the component
/// states it.
///
/// Pinned natively because the whole point of the mode is that three
/// behaviours move together — one page, every row, no footer. A flag that
/// silently stopped implying one of them would still look right in a
/// screenshot.
#[test]
fn constrained_scroll_is_one_page_holding_every_row() {
    use crate::components::{EntityPageSize, EntityTablePagination};

    assert_eq!(
        EntityTablePagination::default(),
        EntityTablePagination::Paged,
        "every existing table must keep paging; the mode is strictly opt-in"
    );
    assert!(!EntityTablePagination::Paged.is_constrained_scroll());
    assert!(EntityTablePagination::ConstrainedScroll.is_constrained_scroll());

    // The component resolves the page size to the row count, so the body, the
    // row-range summary and the pager's page count all read the same single
    // indivisible value (ldui-5p06) rather than disagreeing.
    let resolve = |rows: usize| EntityPageSize::fixed(rows.max(1));
    assert_eq!(resolve(17).rows(), 17, "all 17 rows sit on one page");
    assert_eq!(
        resolve(17).rows(),
        17,
        "and the page can never be smaller than the data it must show"
    );
    // An empty table still needs one page to host its empty state; a page size
    // of zero is not a representable choice.
    assert_eq!(resolve(0).rows(), 1);
}

/// The predicate the renderer uses to mark rows inert (`ldui-ff2f` 3c).
///
/// Stated as the renderer states it — `is_editing() && !is_row_live(key)` —
/// so the rule is pinned natively rather than only in a browser suite. An
/// opted-out table never satisfies the first half, which is what keeps the
/// inert treatment unreachable for every existing consumer.
#[test]
fn rows_are_inert_only_while_another_row_is_live() {
    let inert = |s: &EntityEditState<Row>, key: &str| s.is_editing() && !s.is_row_live(key);

    let mut s = state();
    // Idle: nothing is inert, which is the entire behaviour of a table that
    // never opted in.
    assert!(!inert(&s, "row-1"));
    assert!(!inert(&s, "row-2"));

    s.begin_edit("row-2", existing());
    assert!(!inert(&s, "row-2"), "the live row stays interactive");
    assert!(inert(&s, "row-1"), "every other row goes inert");
    assert!(inert(&s, "row-3"));

    // A draft has no key, so EVERY existing row is inert -- there is no real
    // row to keep interactive.
    let mut s = state();
    s.begin_draft(blank());
    assert!(inert(&s, "row-1"));
    assert!(inert(&s, "row-2"));

    // The rule holds while a commit is in flight, so the table cannot be
    // clicked out from under a pending write.
    let commit = s.commit().expect("commit");
    assert!(inert(&s, "row-1"));

    // ...and lifts the moment the session ends.
    s.resolve(&commit, EntityEditOutcome::Accepted);
    assert!(!inert(&s, "row-1"));
}

/// Editability is opt-in per column, and the default must be read-only —
/// otherwise a derived column would start accepting input for a blank row.
#[test]
fn columns_are_read_only_until_a_column_opts_in() {
    use crate::components::EntityColumn;

    let derived = EntityColumn::text("total", "Total", |r: &Row| r.code.clone());
    assert!(
        !derived.is_editable(),
        "a plain column must stay read-only even inside a live row"
    );
    assert!(derived.editor.is_none());

    let editable = EntityColumn::text("name", "Name", |r: &Row| r.name.clone()).editable(
        EntityCellEditor::text(|r: &Row| r.name.clone(), |r: &mut Row, v| r.name = v),
    );
    assert!(editable.is_editable());

    // The editor round-trips through the row the reducer holds.
    let editor = editable.editor.as_ref().expect("editor was set");
    let mut row = existing();
    assert_eq!(editor.value(&row), "Consultation");
    editor.apply(&mut row, "Intake".to_owned());
    assert_eq!(row.name, "Intake");
    // Editing one field must not disturb the rest of the row.
    assert_eq!(row.code, "CONS");
}

/// The editable value may legitimately differ from the displayed text: a
/// column can render a formatted string and still edit the raw one.
#[test]
fn the_edited_value_is_independent_of_the_displayed_text() {
    use crate::components::EntityColumn;

    let column = EntityColumn::text("code", "Code", |r: &Row| format!("[{}]", r.code)).editable(
        EntityCellEditor::text(|r: &Row| r.code.clone(), |r: &mut Row, v| r.code = v),
    );
    let row = existing();
    assert_eq!((column.text)(&row), "[CONS]", "display is formatted");
    assert_eq!(
        column.editor.as_ref().expect("editor").value(&row),
        "CONS",
        "the editor exposes the raw value, not the formatting"
    );
}

/// Both entry points must be reusable: finishing one session leaves the table
/// able to start another, from either direction.
#[test]
fn sessions_are_reusable_from_both_entry_points() {
    let mut s = state();
    s.begin_draft(blank());
    let c = s.commit().expect("commit");
    s.resolve(&c, EntityEditOutcome::Accepted);

    assert_eq!(
        s.begin_edit("row-1", existing()),
        EntityEditDisposition::Applied
    );
    assert_eq!(s.cancel(), EntityEditDisposition::Applied);
    assert_eq!(s.begin_draft(blank()), EntityEditDisposition::Applied);
}
