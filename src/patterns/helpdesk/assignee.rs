//! The drawer's assignee picker model (`ldui-purt`): the assignable
//! directory sorted by name, narrowed by a typed query, and keyed so a
//! duplicate display name can never assign the wrong person.
//!
//! Pure functions only; the view lives in `drawer.rs`.

use super::model::Person;
use crate::components::{ResultListItem, ResultRow};

/// The result key of the "no assignee" choice. Person keys are prefixed
/// (`person:<id>`), so no host id can collide with it.
pub const UNASSIGNED_KEY: &str = "unassigned";

/// The result key of one assignable person.
pub fn assignee_key(id: &str) -> String {
    format!("person:{id}")
}

/// What confirming the assignee picker writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssigneeChoice {
    /// Clear the assignee.
    Unassigned,
    /// Assign this person.
    Person(Person),
}

impl AssigneeChoice {
    /// The `assignee_id` a [`TriageAction::Assign`](super::TriageAction)
    /// carries for this choice.
    pub fn assignee_id(&self) -> Option<String> {
        match self {
            Self::Unassigned => None,
            Self::Person(p) => Some(p.id.clone()),
        }
    }
}

/// The directory in display order: by display name, case-insensitively,
/// then by id so two people with the same name keep a stable order. A host
/// directory arrives in whatever order its source returns (4iiz-Office's is
/// ~300 people, unsorted); nobody wants that order.
pub fn sorted_assignable(people: &[Person]) -> Vec<Person> {
    let mut sorted = people.to_vec();
    sorted.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    sorted
}

/// Whether `text` matches the typed `query`: case-insensitive substring,
/// surrounding whitespace ignored, and an empty query matches everything.
pub fn assignee_query_matches(text: &str, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || text.to_lowercase().contains(&q)
}

/// The picker's rows for `query`: the "unassigned" choice first (while it
/// matches), then every matching person in [`sorted_assignable`] order.
pub fn assignee_items(
    people: &[Person],
    query: &str,
    unassigned_label: &str,
) -> Vec<ResultListItem<AssigneeChoice>> {
    let mut items = Vec::new();
    if assignee_query_matches(unassigned_label, query) {
        items.push(ResultListItem::new(
            UNASSIGNED_KEY,
            ResultRow::new(unassigned_label),
            AssigneeChoice::Unassigned,
        ));
    }
    items.extend(
        sorted_assignable(people)
            .into_iter()
            .filter(|p| assignee_query_matches(&p.display_name, query))
            .map(|p| {
                ResultListItem::new(
                    assignee_key(&p.id),
                    ResultRow::new(p.display_name.clone()),
                    AssigneeChoice::Person(p),
                )
            }),
    );
    items
}

/// The number of *people* the current query matches, for the live count
/// announcement. The "unassigned" row is a choice, not a person, so it is
/// not counted.
pub fn assignee_match_count(people: &[Person], query: &str) -> usize {
    people
        .iter()
        .filter(|p| assignee_query_matches(&p.display_name, query))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory() -> Vec<Person> {
        vec![
            Person::new("w-chris", "Chris Forbes"),
            Person::new("w-zoe", "Zoe Park"),
            Person::new("w-ana", "ana Ruiz"),
            Person::new("w-ben", "Ben Adler"),
        ]
    }

    fn names(people: &[Person]) -> Vec<&str> {
        people.iter().map(|p| p.display_name.as_str()).collect()
    }

    #[test]
    fn the_directory_sorts_by_name_ignoring_case() {
        let sorted = sorted_assignable(&directory());
        assert_eq!(
            names(&sorted),
            ["ana Ruiz", "Ben Adler", "Chris Forbes", "Zoe Park"]
        );
    }

    #[test]
    fn duplicate_names_sort_stably_by_id_and_stay_distinct() {
        let people = vec![
            Person::new("w-2", "Daniela Talavera"),
            Person::new("w-1", "Daniela Talavera"),
        ];
        let items = assignee_items(&people, "daniela", "Unassigned");
        let keys: Vec<&str> = items.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(keys, ["person:w-1", "person:w-2"]);
    }

    #[test]
    fn a_query_narrows_case_insensitively_and_trims() {
        let items = assignee_items(&directory(), "  AR ", "Unassigned");
        let titles: Vec<&str> = items.iter().map(|i| i.row.title.as_str()).collect();
        assert_eq!(titles, ["Zoe Park"], "unassigned does not match 'ar'");
        assert_eq!(assignee_match_count(&directory(), "  AR "), 1);
    }

    #[test]
    fn an_empty_query_offers_unassigned_first_then_everyone() {
        let items = assignee_items(&directory(), "", "Unassigned");
        assert_eq!(items[0].key, UNASSIGNED_KEY);
        assert_eq!(items[0].payload.assignee_id(), None);
        assert_eq!(items.len(), 5);
        assert_eq!(
            assignee_match_count(&directory(), ""),
            4,
            "only people count"
        );
        assert_eq!(
            items[1].payload.assignee_id().as_deref(),
            Some("w-ana"),
            "people follow in sorted order"
        );
    }

    #[test]
    fn unassigned_is_itself_searchable() {
        let items = assignee_items(&directory(), "unass", "Unassigned");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].payload, AssigneeChoice::Unassigned);
    }

    #[test]
    fn no_person_key_collides_with_unassigned() {
        assert_ne!(assignee_key(UNASSIGNED_KEY), UNASSIGNED_KEY);
    }
}
