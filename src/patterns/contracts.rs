//! Typed contracts shared by opinionated page patterns.

use std::marker::PhantomData;

/// A reusable page structure whose required behavior can be validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PagePattern {
    /// A complete dataset is loaded for one selector value and manipulated locally.
    ClientSnapshotList,
}

/// Describes whether changing a selector loads a different dataset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatasetBehavior {
    /// The page does not have a dataset selector.
    None,
    /// Changing the named selector loads a new complete client snapshot.
    SelectorTriggersLoad {
        /// Stable field key for the selector.
        key: &'static str,
    },
}

/// User-visible states that an opinionated page may be required to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageState {
    /// The first dataset is loading and no usable snapshot exists yet.
    InitialLoading,
    /// A usable snapshot is visible.
    Ready,
    /// A usable snapshot remains visible while a replacement is loading.
    Revalidating,
    /// The first dataset load failed.
    InitialError,
    /// A replacement load failed while an older snapshot remains usable.
    RefreshError,
    /// No dataset has been selected or loaded yet.
    NeverLoaded,
    /// The loaded dataset contains no rows.
    Empty,
    /// Local filters exclude every row in a non-empty dataset.
    FilteredEmpty,
    /// The displayed snapshot is older than the server's current epoch or sequence.
    Stale,
    /// A row claim is in flight.
    Claiming,
    /// A row claim completed successfully.
    ClaimSucceeded,
    /// A row was claimed elsewhere before the local claim completed.
    ClaimConflict,
    /// A row claim failed for a reason other than a conflict.
    ClaimFailed,
    /// Live dataset updates are temporarily unavailable.
    LiveInterrupted,
}

/// Responsive layouts that a page contract can require.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageBreakpoint {
    /// Narrow layout intended for phones and compact windows.
    Compact,
    /// Wide layout intended for desktop work.
    Wide,
}

/// A declarative, machine-checkable description of a page's behavioral surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageContract {
    /// Stable page identifier used by tests and measurements.
    pub id: &'static str,
    /// Absolute application route.
    pub route: &'static str,
    /// Opinionated page pattern used by the route.
    pub pattern: PagePattern,
    /// Dataset-selection behavior.
    pub dataset: DatasetBehavior,
    /// Keys for local state that persists independently of dataset selection.
    pub local_state: &'static [&'static str],
    /// User-visible states the implementation must handle.
    pub required_states: &'static [PageState],
    /// Responsive layouts the implementation must support.
    pub breakpoints: &'static [PageBreakpoint],
}

/// A violation found while validating a page or filter contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError {
    /// The page identifier is empty.
    EmptyId,
    /// The route is empty.
    EmptyRoute,
    /// The route is not application-absolute.
    RouteMustBeAbsolute(&'static str),
    /// A required page state occurs more than once.
    DuplicateState(PageState),
    /// A local-state key occurs more than once.
    DuplicateLocalState(&'static str),
    /// A local-state key is empty.
    EmptyLocalState,
    /// A responsive breakpoint occurs more than once.
    DuplicateBreakpoint(PageBreakpoint),
    /// A state required by the selected pattern is missing.
    MissingState(PageState),
    /// The selected pattern requires a responsive breakpoint that is missing.
    MissingBreakpoint(PageBreakpoint),
    /// The key for a dataset selector is empty.
    EmptyDatasetSelector,
    /// A dataset selector was incorrectly declared as resettable local state.
    DatasetSelectorIsLocalState(&'static str),
    /// A dataset selector was incorrectly declared as a local filter.
    DatasetSelectorIsFilter(&'static str),
    /// A local filter key is empty.
    EmptyFilter,
    /// A local filter key occurs more than once.
    DuplicateFilter(&'static str),
}

const CLIENT_SNAPSHOT_STATES: &[PageState] = &[
    PageState::InitialLoading,
    PageState::Ready,
    PageState::Revalidating,
    PageState::InitialError,
    PageState::RefreshError,
    PageState::NeverLoaded,
    PageState::Empty,
    PageState::FilteredEmpty,
    PageState::Stale,
    PageState::Claiming,
    PageState::ClaimSucceeded,
    PageState::ClaimConflict,
    PageState::ClaimFailed,
    PageState::LiveInterrupted,
];

const CLIENT_SNAPSHOT_BREAKPOINTS: &[PageBreakpoint] =
    &[PageBreakpoint::Compact, PageBreakpoint::Wide];

impl PageContract {
    /// Returns every contract violation so agents can fix a page in one pass.
    pub fn validate(&self) -> Result<(), Vec<ContractError>> {
        let mut errors = Vec::new();

        if self.id.trim().is_empty() {
            errors.push(ContractError::EmptyId);
        }
        if self.route.is_empty() {
            errors.push(ContractError::EmptyRoute);
        } else if !self.route.starts_with('/') {
            errors.push(ContractError::RouteMustBeAbsolute(self.route));
        }

        collect_duplicate_local_state(self.local_state, &mut errors);
        collect_duplicate_states(self.required_states, &mut errors);
        collect_duplicate_breakpoints(self.breakpoints, &mut errors);

        if let DatasetBehavior::SelectorTriggersLoad { key } = self.dataset {
            if key.trim().is_empty() {
                errors.push(ContractError::EmptyDatasetSelector);
            } else if self.local_state.contains(&key) {
                errors.push(ContractError::DatasetSelectorIsLocalState(key));
            }
        }

        match self.pattern {
            PagePattern::ClientSnapshotList => {
                for state in CLIENT_SNAPSHOT_STATES {
                    if !self.required_states.contains(state) {
                        errors.push(ContractError::MissingState(*state));
                    }
                }
                for breakpoint in CLIENT_SNAPSHOT_BREAKPOINTS {
                    if !self.breakpoints.contains(breakpoint) {
                        errors.push(ContractError::MissingBreakpoint(*breakpoint));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn collect_duplicate_local_state(keys: &'static [&'static str], errors: &mut Vec<ContractError>) {
    for (index, key) in keys.iter().enumerate() {
        if key.trim().is_empty() {
            errors.push(ContractError::EmptyLocalState);
        }
        if keys[..index].contains(key) {
            errors.push(ContractError::DuplicateLocalState(key));
        }
    }
}

fn collect_duplicate_states(states: &'static [PageState], errors: &mut Vec<ContractError>) {
    for (index, state) in states.iter().enumerate() {
        if states[..index].contains(state) {
            errors.push(ContractError::DuplicateState(*state));
        }
    }
}

fn collect_duplicate_breakpoints(
    breakpoints: &'static [PageBreakpoint],
    errors: &mut Vec<ContractError>,
) {
    for (index, breakpoint) in breakpoints.iter().enumerate() {
        if breakpoints[..index].contains(breakpoint) {
            errors.push(ContractError::DuplicateBreakpoint(*breakpoint));
        }
    }
}

/// Typed schema for the local filters associated with a page contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterSchema<T> {
    dataset_selector: &'static str,
    fields: &'static [&'static str],
    filter_state: PhantomData<fn() -> T>,
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
}

/// Binds a complete client snapshot to typed row, selector, and filter state.
pub trait ClientSnapshotContract {
    /// A row in the downloaded snapshot.
    type Row: Clone + 'static;
    /// Value selecting which complete dataset is loaded.
    type DatasetKey: Clone + Eq + 'static;
    /// Local filter state retained across dataset changes and resets.
    type FilterState: Clone + Default + PartialEq + 'static;

    /// Version used to invalidate incompatible persisted local state.
    const SCHEMA_VERSION: u16;
    /// Stable storage key for persisted local state.
    const STORAGE_KEY: &'static str;

    /// Returns the stable identity of a row.
    fn row_key(row: &Self::Row) -> &str;

    /// Returns whether a row matches the complete local filter state.
    fn matches(row: &Self::Row, filters: &Self::FilterState) -> bool;
}
