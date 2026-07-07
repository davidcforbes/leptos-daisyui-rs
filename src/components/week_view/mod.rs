mod component;

/// Types for WeekView including `CalEvent` and the dependency-free UTC date
/// math (`civil_from_days`, `week_start_for`, `week_range_label`,
/// `weekday_abbrev`, `day_of_month`) ported from d2d-ui.
mod types;

pub use component::*;
pub use types::*;

#[cfg(test)]
mod tests;
