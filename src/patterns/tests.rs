use super::*;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct PilotFilters {
    search: String,
    status: String,
    case_type: String,
}

page_contract! {
    pub PILOT_PAGE {
        id: "pilot",
        route: "/pilot",
        pattern: PagePattern::ClientSnapshotList,
        dataset: DatasetBehavior::SelectorTriggersLoad { key: "office" },
        local_state: ["search", "status", "case_type", "sort", "page_size", "columns"],
        required_states: [
            InitialLoading, Ready, Revalidating, InitialError, RefreshError,
            NeverLoaded, Empty, FilteredEmpty, Stale, Claiming, ClaimSucceeded,
            ClaimConflict, ClaimFailed, LiveInterrupted
        ],
        breakpoints: [Compact, Wide],
    }
}

filter_schema! {
    pub PILOT_FILTERS: PilotFilters {
        dataset_selector: "office",
        filters: [search, status, case_type],
    }
}

#[test]
fn a_complete_client_snapshot_contract_is_valid() {
    assert_eq!(PILOT_PAGE.validate(), Ok(()));
    assert_eq!(PILOT_FILTERS.validate(), Ok(()));
    assert_eq!(PILOT_FILTERS.fields(), ["search", "status", "case_type"]);
}

#[test]
fn page_contract_rejects_empty_and_relative_routes() {
    let empty = PageContract {
        route: "",
        ..PILOT_PAGE
    };
    assert!(
        empty
            .validate()
            .unwrap_err()
            .contains(&ContractError::EmptyRoute)
    );

    let relative = PageContract {
        route: "pilot",
        ..PILOT_PAGE
    };
    assert!(
        relative
            .validate()
            .unwrap_err()
            .contains(&ContractError::RouteMustBeAbsolute("pilot"))
    );
}

#[test]
fn page_contract_rejects_duplicate_states_and_local_keys() {
    const DUPLICATE_STATES: &[PageState] = &[PageState::Ready, PageState::Ready];
    const DUPLICATE_LOCAL: &[&str] = &["search", "search"];
    let contract = PageContract {
        required_states: DUPLICATE_STATES,
        local_state: DUPLICATE_LOCAL,
        ..PILOT_PAGE
    };
    let errors = contract.validate().unwrap_err();
    assert!(errors.contains(&ContractError::DuplicateState(PageState::Ready)));
    assert!(errors.contains(&ContractError::DuplicateLocalState("search")));
}

#[test]
fn dataset_selector_cannot_be_a_resettable_local_filter() {
    const WRONG_LOCAL: &[&str] = &["search", "office"];
    let contract = PageContract {
        local_state: WRONG_LOCAL,
        ..PILOT_PAGE
    };
    assert!(
        contract
            .validate()
            .unwrap_err()
            .contains(&ContractError::DatasetSelectorIsLocalState("office"))
    );

    let schema = FilterSchema::<PilotFilters>::new("office", &["search", "office"]);
    assert!(
        schema
            .validate()
            .unwrap_err()
            .contains(&ContractError::DatasetSelectorIsFilter("office"))
    );
}

#[test]
fn client_snapshot_pages_require_the_full_state_contract() {
    const TOO_FEW: &[PageState] = &[PageState::Ready, PageState::Empty];
    let contract = PageContract {
        required_states: TOO_FEW,
        ..PILOT_PAGE
    };
    let errors = contract.validate().unwrap_err();
    assert!(errors.contains(&ContractError::MissingState(PageState::InitialLoading)));
    assert!(errors.contains(&ContractError::MissingState(PageState::FilteredEmpty)));
    assert!(errors.contains(&ContractError::MissingState(PageState::LiveInterrupted)));
}

#[test]
fn contract_rejects_duplicate_breakpoints_and_empty_selector_keys() {
    const DUPLICATE_BREAKPOINTS: &[PageBreakpoint] =
        &[PageBreakpoint::Compact, PageBreakpoint::Compact];
    let duplicate = PageContract {
        breakpoints: DUPLICATE_BREAKPOINTS,
        ..PILOT_PAGE
    };
    assert!(
        duplicate
            .validate()
            .unwrap_err()
            .contains(&ContractError::DuplicateBreakpoint(PageBreakpoint::Compact))
    );

    let empty_selector = PageContract {
        dataset: DatasetBehavior::SelectorTriggersLoad { key: "" },
        ..PILOT_PAGE
    };
    assert!(
        empty_selector
            .validate()
            .unwrap_err()
            .contains(&ContractError::EmptyDatasetSelector)
    );
}

#[derive(Clone)]
struct PilotSnapshot;

impl ClientSnapshotContract for PilotSnapshot {
    type Row = String;
    type DatasetKey = String;
    type FilterState = PilotFilters;

    const SCHEMA_VERSION: u16 = 1;
    const STORAGE_KEY: &'static str = "pilot.filters.v1";

    fn row_key(row: &Self::Row) -> &str {
        row
    }

    fn matches(row: &Self::Row, filters: &Self::FilterState) -> bool {
        row.contains(&filters.search)
    }
}

#[test]
fn client_snapshot_contract_keeps_row_identity_and_filtering_typed() {
    let filters = PilotFilters {
        search: "Needle".into(),
        ..PilotFilters::default()
    };
    assert_eq!(PilotSnapshot::row_key(&"Needle row".into()), "Needle row");
    assert!(PilotSnapshot::matches(&"Needle row".into(), &filters));
    assert!(!PilotSnapshot::matches(&"Other".into(), &filters));
    assert_eq!(PilotSnapshot::SCHEMA_VERSION, 1);
}
