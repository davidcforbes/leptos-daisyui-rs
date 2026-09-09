use crate::components::entity_table::EntityPageSizeIntent;
use serde::{Deserialize, Serialize};

/// User intent for server paging, independent of the accepted server query.
///
/// Retain this in a caller-owned `RwSignal` through `page_size_preference`.
/// Measurements never replace the explicit numeric preference. A requested
/// size is not accepted data: the table labels counts from the supplied query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerTablePageSizePreference {
    intent: EntityPageSizeIntent,
    fixed_rows: i64,
}

impl ServerTablePageSizePreference {
    /// Start in Auto, retaining a positive explicit numeric preference.
    pub const fn auto(fixed_rows: i64) -> Self {
        Self {
            intent: EntityPageSizeIntent::Auto,
            fixed_rows: if fixed_rows < 1 { 1 } else { fixed_rows },
        }
    }

    /// Request a positive fixed size that survives resize and refetch.
    pub const fn fixed(rows: i64) -> Self {
        Self {
            intent: EntityPageSizeIntent::Fixed,
            ..Self::auto(rows)
        }
    }

    /// The saved Auto/fixed intent, using the same vocabulary as EntityTable.
    pub const fn intent(self) -> EntityPageSizeIntent {
        self.intent
    }

    /// Explicit numeric choice, never a transient viewport measurement.
    pub fn fixed_rows(self) -> i64 {
        self.fixed_rows.max(1)
    }

    pub(crate) fn is_auto(self) -> bool {
        self.intent == EntityPageSizeIntent::Auto
    }

    pub(crate) fn choose(self, value: &str, auto_available: bool, choices: &[i64]) -> Option<Self> {
        if value == "auto" {
            return auto_available.then(|| Self::auto(self.fixed_rows()));
        }
        let rows = value.parse::<i64>().ok()?;
        (rows > 0 && choices.contains(&rows)).then(|| Self::fixed(rows))
    }

    pub(crate) fn control_value(self, accepted: i64, auto_available: bool) -> String {
        if self.is_auto() && auto_available {
            "auto".to_owned()
        } else {
            accepted.max(1).to_string()
        }
    }
}

pub(crate) fn server_page_size_choices(options: Vec<i64>, accepted: i64, fixed: i64) -> Vec<i64> {
    let mut choices: Vec<_> = options.into_iter().filter(|value| *value > 0).collect();
    choices.extend([accepted.max(1), fixed.max(1)]);
    choices.sort_unstable();
    choices.dedup();
    choices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_retains_numeric_preference_and_labels_only_accepted_size() {
        let pref = ServerTablePageSizePreference::fixed(25)
            .choose("auto", true, &[10, 25])
            .unwrap();
        assert_eq!(pref.fixed_rows(), 25);
        assert_eq!(pref.control_value(7, true), "auto");
        let fixed = pref.choose("10", true, &[10, 25]).unwrap();
        assert_eq!(
            fixed.control_value(7, true),
            "7",
            "a declined request must not claim ten accepted rows"
        );
        assert_eq!(fixed.fixed_rows(), 10);
    }

    #[test]
    fn choices_preserve_nonstandard_accepted_sizes_and_reject_invalid_changes() {
        let choices = server_page_size_choices(vec![0, -1, 10, 10, 25], 7, 11);
        assert_eq!(choices, vec![7, 10, 11, 25]);
        let pref = ServerTablePageSizePreference::fixed(11);
        for value in ["0", "-1", "99", "invalid", "auto"] {
            assert!(pref.choose(value, false, &choices).is_none());
        }
        assert_eq!(ServerTablePageSizePreference::fixed(-2).fixed_rows(), 1);
    }
}
