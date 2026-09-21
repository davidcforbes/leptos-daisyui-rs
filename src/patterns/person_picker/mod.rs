//! Opinionated person roster: search, collapse rail, a single-select
//! listbox that can be deselected, a selected badge and a footer.
//! See `doc/plans/2026-09-21-person-picker-design.md`.

mod model;
mod texts;

pub use model::{
    PersonPresence, PickerPerson, matches_search, nameable_selection, next_selection,
};
pub use texts::PersonPickerTexts;

#[cfg(test)]
mod tests;
