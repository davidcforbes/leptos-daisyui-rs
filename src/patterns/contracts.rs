//! Typed contracts shared by opinionated page patterns.
//!
//! The Leptos-free page-contract vocabulary lives in `ldui_design::contracts`
//! (ldui-3u3p) and is re-exported here unchanged. The filter-schema
//! projection below stays in this crate because its default-view payload
//! carries the table's `EntityTablePreferences`.

pub use ldui_design::contracts::*;

use std::{collections::BTreeMap, marker::PhantomData};

use crate::components::EntityTablePreferences;
use serde::{Serialize, ser::SerializeMap};
use serde_json::Value;

/// Typed schema for the local filters associated with a page contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterSchema<T> {
    dataset_selector: &'static str,
    fields: &'static [&'static str],
    filter_state: PhantomData<fn() -> T>,
}

/// A rejected attempt to project consumer filter state into persisted defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterProjectionError {
    /// The schema itself is invalid and cannot authorize any payload.
    InvalidSchema(Vec<ContractError>),
    /// The dataset selector is transient and never belongs in view defaults.
    DatasetSelector(String),
    /// The consumer supplied a key absent from the schema allowlist.
    Undeclared(String),
    /// The consumer supplied the same key more than once.
    Duplicate(String),
}

/// Schema-ordered local values approved for default-view persistence.
///
/// The fields are private and this type has no public constructor. A consumer
/// obtains it only as part of [`SnapshotViewDefaults`] returned by
/// [`FilterSchema::project_defaults`].
#[derive(Clone, Debug, PartialEq)]
pub struct LocalFilterDefaults {
    values: Vec<(String, Value)>,
}

impl LocalFilterDefaults {
    /// Returns one projected value by stable filter key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values
            .iter()
            .find_map(|(candidate, value)| (candidate == key).then_some(value))
    }

    /// Iterates projected values in schema declaration order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&str, &Value)> {
        self.values.iter().map(|(key, value)| (key.as_str(), value))
    }
}

impl Serialize for LocalFilterDefaults {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (key, value) in &self.values {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

/// Persistence-neutral default-view payload for a snapshot table.
///
/// Serialization intentionally contains only `filters` and `table`. Dataset
/// identity, free-text search, the current page, rows/revision, sessions, and
/// action state have no representation in this type.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SnapshotViewDefaults {
    filters: LocalFilterDefaults,
    table: EntityTablePreferences,
}

impl SnapshotViewDefaults {
    /// Returns the schema-projected local filter defaults.
    pub const fn filters(&self) -> &LocalFilterDefaults {
        &self.filters
    }

    /// Returns the complete versioned table preferences.
    pub const fn table(&self) -> &EntityTablePreferences {
        &self.table
    }
}

impl<T> FilterSchema<T> {
    /// Creates a schema for a dataset selector and its independent local filters.
    pub const fn new(dataset_selector: &'static str, fields: &'static [&'static str]) -> Self {
        Self {
            dataset_selector,
            fields,
            filter_state: PhantomData,
        }
    }

    /// Returns local filter keys in their canonical display order.
    pub const fn fields(&self) -> &'static [&'static str] {
        self.fields
    }

    /// Returns the dataset selector key, which is intentionally not a filter.
    pub const fn dataset_selector(&self) -> &'static str {
        self.dataset_selector
    }

    /// Returns every schema violation.
    pub fn validate(&self) -> Result<(), Vec<ContractError>> {
        let mut errors = Vec::new();
        if self.dataset_selector.trim().is_empty() {
            errors.push(ContractError::EmptyDatasetSelector);
        }
        for (index, field) in self.fields.iter().enumerate() {
            if field.trim().is_empty() {
                errors.push(ContractError::EmptyFilter);
            }
            if self.fields[..index].contains(field) {
                errors.push(ContractError::DuplicateFilter(field));
            }
            if field == &self.dataset_selector {
                errors.push(ContractError::DatasetSelectorIsFilter(field));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Projects consumer values through this schema's persistence allowlist.
    ///
    /// Input order is irrelevant: the serialized result always follows the
    /// schema's field order. Any dataset selector, undeclared key, duplicate,
    /// or invalid schema rejects the whole payload rather than silently
    /// dropping authority-bearing state.
    pub fn project_defaults<I, K>(
        &self,
        values: I,
        table: EntityTablePreferences,
    ) -> Result<SnapshotViewDefaults, FilterProjectionError>
    where
        I: IntoIterator<Item = (K, Value)>,
        K: AsRef<str>,
    {
        self.validate()
            .map_err(FilterProjectionError::InvalidSchema)?;

        let mut supplied = BTreeMap::<String, Value>::new();
        for (key, value) in values {
            let key = key.as_ref().to_owned();
            if key == self.dataset_selector {
                return Err(FilterProjectionError::DatasetSelector(key));
            }
            if !self.fields.contains(&key.as_str()) {
                return Err(FilterProjectionError::Undeclared(key));
            }
            if supplied.insert(key.clone(), value).is_some() {
                return Err(FilterProjectionError::Duplicate(key));
            }
        }

        let values = self
            .fields
            .iter()
            .filter_map(|field| {
                supplied
                    .remove(*field)
                    .map(|value| ((*field).to_owned(), value))
            })
            .collect();
        Ok(SnapshotViewDefaults {
            filters: LocalFilterDefaults { values },
            table,
        })
    }
}
