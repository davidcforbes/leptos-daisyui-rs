//! Opinionated person roster: search, collapse rail, a single-select
//! listbox that can be deselected, a selected badge and a footer.
//! See `doc/plans/2026-09-21-person-picker-design.md`.

mod listbox;
mod model;
mod panel;
mod texts;

pub use listbox::PersonListbox;
pub use model::{
    PersonPresence, PickerPerson, STALE_PRESENCE_DOT_CLASS, matches_search, nameable_selection,
    next_selection,
};
pub use panel::PersonPicker;
pub use texts::PersonPickerTexts;

#[cfg(test)]
mod tests;
