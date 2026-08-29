//! Typed contracts shared by opinionated page patterns.

use std::{collections::BTreeMap, fmt, marker::PhantomData};

use crate::components::EntityTablePreferences;
use serde::{
    Serialize,
    ser::{SerializeMap, SerializeStruct},
};
use serde_json::Value;

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

/// A violation found while validating a frozen PageContract v2 declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageContractV2Error {
    /// The page identifier is empty.
    EmptyId,
    /// The route is empty.
    EmptyRoute,
    /// The route is not application-absolute.
    RouteMustBeAbsolute(&'static str),
    /// A dataset selector was incorrectly declared as a local filter.
    DatasetSelectorIsFilter(&'static str),
    /// Client-snapshot data cannot delegate sorting to the server.
    ClientSnapshotUsesServerSort,
    /// A table archetype declares the wrong client/server data boundary.
    ArchetypeDataModeMismatch {
        /// Table archetype being validated.
        archetype: PageArchetype,
        /// Incoherent declared data mode.
        declared: DataMode,
    },
    /// A table archetype declares the wrong dataset replacement boundary.
    ArchetypeDatasetLoadMismatch {
        /// Table archetype being validated.
        archetype: PageArchetype,
        /// Incoherent declared dataset load.
        declared: DatasetLoad,
    },
    /// A table archetype declares sorting at the wrong execution boundary.
    ArchetypeSortMismatch {
        /// Table archetype being validated.
        archetype: PageArchetype,
        /// Incoherent declared sorting mode.
        declared: SortExecution,
    },
    /// A table archetype has no stable keyed row identity.
    TableArchetypeMissingRowIdentity(PageArchetype),
    /// A transient, data-bearing, or authority field was allowlisted for persistence.
    ForbiddenPersistedState(StateField),
    /// A satellite declared a dependency on core state or the core package.
    SatelliteDependsOnCore(&'static str),
    /// A named mutation omitted one required authoritative outcome.
    MissingMutationOutcome {
        /// Stable mutation name.
        mutation: &'static str,
        /// Required outcome that was not declared.
        outcome: MutationOutcome,
    },
    /// A named mutation omitted both generic and row-removal success outcomes.
    MissingMutationSuccess(&'static str),
    /// A realtime page omitted a disconnect or resynchronization state.
    MissingRealtimeState(RealtimeState),
    /// A visual test lane has no nonempty named baseline.
    VisualLaneMissingBaseline,
    /// One or more visual baselines were declared without selecting the Visual lane.
    BaselinesRequireVisualLane,
    /// A visual baseline has an empty stable name.
    EmptyBaselineName,
    /// The declaration does not use the frozen PageContract v2 schema version.
    UnsupportedContractVersion(u16),
    /// The user-facing page title is empty.
    EmptyTitle,
    /// The product or repository owner identifier is empty.
    EmptyOwner,
    /// The route contains an implementation prefix, query, fragment, or whitespace.
    RouteMustBeStable(&'static str),
    /// No page-owned source glob was declared.
    MissingOwnedGlobs,
    /// No forbidden shared/core source glob was declared.
    MissingForbiddenGlobs,
    /// A page-owned source glob is empty.
    EmptyOwnedGlob,
    /// A forbidden source glob is empty.
    EmptyForbiddenGlob,
    /// A page-owned source glob occurs more than once.
    DuplicateOwnedGlob(&'static str),
    /// A forbidden source glob occurs more than once.
    DuplicateForbiddenGlob(&'static str),
    /// The same exact source glob is both owned and forbidden.
    OwnedGlobIsForbidden(&'static str),
    /// A compile-time dependency occurs more than once.
    DuplicateDependency(PageDependency),
    /// A compatibility identity has no corresponding typed dependency edge.
    MissingCompatibilityDependency(CompatibilityDependencyKind),
    /// More than one dependency edge claims the same compatibility role.
    DuplicateCompatibilityDependency(CompatibilityDependencyKind),
    /// A typed dependency identity disagrees with the exported compatibility identity.
    CompatibilityDependencyMismatch {
        /// Compatibility role being checked.
        kind: CompatibilityDependencyKind,
        /// Identity declared by the dependency edge.
        dependency: &'static str,
        /// Identity declared by the compatibility contract.
        compatibility: &'static str,
    },
    /// A satellite declared a dependency on another satellite package.
    SatelliteDependsOnSatellite(&'static str),
    /// The core surface declared a compile-time dependency on a satellite package.
    CoreDependsOnSatellite(&'static str),
    /// A page control occurs more than once.
    DuplicateControl(PageControl),
    /// A persisted-default field occurs more than once.
    DuplicatePersistedState(StateField),
    /// A transient-state field occurs more than once.
    DuplicateTransientState(StateField),
    /// A field was declared both persisted and transient.
    StateIsPersistedAndTransient(StateField),
    /// A mutation name occurs more than once.
    DuplicateMutation(&'static str),
    /// A mutation outcome occurs more than once.
    DuplicateMutationOutcome {
        /// Stable mutation name.
        mutation: &'static str,
        /// Repeated outcome.
        outcome: MutationOutcome,
    },
    /// A realtime event occurs more than once.
    DuplicateRealtimeEvent(RealtimeEvent),
    /// A realtime recovery state occurs more than once.
    DuplicateRealtimeState(RealtimeState),
    /// More than one capability rule owns the same action.
    DuplicateCapabilityAction(CapabilityAction),
    /// A responsive layout occurs more than once.
    DuplicateResponsiveLayout(ResponsiveLayout),
    /// A responsive behavior occurs more than once.
    DuplicateResponsiveBehavior(ResponsiveBehavior),
    /// An accessibility obligation occurs more than once.
    DuplicateAccessibilityObligation(AccessibilityObligation),
    /// An accessible-label purpose occurs more than once.
    DuplicateAccessibleLabel(&'static str),
    /// A presentation state occurs more than once.
    DuplicatePresentationState(PresentationState),
    /// A test lane occurs more than once.
    DuplicateTestLane(TestLane),
    /// A visual baseline name occurs more than once.
    DuplicateBaseline(&'static str),
    /// The shared-foundation compatibility identity is empty.
    EmptyFoundationCompatibility,
    /// The server-API compatibility identity is empty.
    EmptyServerApiCompatibility,
    /// Durable preference schemas are versioned from one, never zero.
    ZeroPreferenceSchema,
    /// The compressed bundle budget is zero.
    ZeroBundleBudget,
    /// More than one compressed bundle budget was declared.
    DuplicateBundleBudget,
    /// An interaction budget has no stable operation name.
    EmptyInteractionBudgetOperation,
    /// An interaction budget has no fixture row count.
    ZeroInteractionBudgetRows(&'static str),
    /// An interaction p95 budget is zero milliseconds.
    ZeroInteractionBudgetMillis(&'static str),
    /// More than one interaction budget targets the same operation.
    DuplicateInteractionBudget(&'static str),
    /// A lane-duration budget refers to a lane the page does not select.
    BudgetForUndeclaredLane(TestLane),
    /// A lane-duration budget is zero milliseconds.
    ZeroTestLaneBudget(TestLane),
    /// More than one duration budget targets the same lane.
    DuplicateTestLaneBudget(TestLane),
    /// A baseline names a presentation state absent from the contract.
    BaselineStateNotDeclared(PresentationState),
    /// A baseline names a responsive layout absent from the contract.
    BaselineLayoutNotDeclared(ResponsiveLayout),
    /// A typed dependency variant carries an empty compatibility/package identity.
    EmptyDependencyIdentity(PageDependency),
    /// A product-extensible typed variant carries an empty stable name.
    EmptyNamedValue(ContractNameKind),
    /// A mutation has an empty stable name.
    EmptyMutationName,
    /// A mutation has no server-authoritative capability identity.
    EmptyMutationCapability(&'static str),
    /// A capability rule has an empty capability identity.
    EmptyCapabilityName(CapabilityAction),
    /// An accessible-label declaration has no stable purpose key.
    EmptyAccessibleLabelPurpose,
    /// An accessible-label declaration has no user-facing text.
    EmptyAccessibleLabelText(&'static str),
    /// No capability rule declares the named mutation.
    MissingMutationCapabilityRule(&'static str),
    /// The mutation rule and mutation contract name different capabilities.
    MutationCapabilityMismatch {
        /// Stable mutation name.
        mutation: &'static str,
        /// Capability declared by the mutation contract.
        expected: &'static str,
        /// Capability declared by the authorization rule.
        declared: &'static str,
    },
    /// A capability rule refers to a mutation absent from the contract.
    CapabilityForUnknownMutation(&'static str),
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

/// Contract schema version frozen for the `ldui-surface-v1` foundation.
pub const PAGE_CONTRACT_V2_VERSION: u16 = 2;

/// Stable identity emitted before every deterministic v2 contract export.
pub const PAGE_CONTRACT_V2_EXPORT_SCHEMA: &str = "ldui.page-contract.v2";

/// Product-extensible declaration slot whose stable name is being validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContractNameKind {
    /// Product-specific dataset selector.
    DatasetSelector,
    /// Product-specific initial dataset policy.
    DatasetDefault,
    /// Stable row-identity field.
    RowIdentity,
    /// Local-filter field key.
    LocalFilter,
    /// Product-specific page control.
    PageControl,
    /// Product-specific or selector-backed state field.
    StateField,
    /// Product-specific realtime event.
    RealtimeEvent,
    /// Product-specific or mutation capability action.
    CapabilityAction,
    /// Product-specific responsive layout.
    ResponsiveLayout,
    /// Product-specific responsive behavior.
    ResponsiveBehavior,
    /// Product-specific accessibility obligation.
    AccessibilityObligation,
    /// Product-specific presentation state.
    PresentationState,
}

/// Determines whether a page is linked into core or delivered independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageDelivery {
    /// The page is linked into the core surface.
    Core,
    /// The page is an independently compiled and delivered surface.
    Satellite,
}

/// Opinionated composition owned by the shared foundation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageArchetype {
    /// A bounded snapshot with local controls and a full-width table.
    SnapshotTablePage,
    /// A server-queried table for datasets that cannot be loaded as one snapshot.
    ServerTablePage,
    /// A page centered on one record and its related actions.
    RecordDetailPage,
    /// A multi-step or validated form workflow.
    FormWorkflowPage,
    /// A dashboard composed from summary and visualization patterns.
    DashboardPage,
    /// A page for durable user or system settings.
    SettingsPage,
}

/// A compile-time dependency edge declared by a page surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageDependency {
    /// A versioned shared UI foundation contract.
    SharedFoundation(&'static str),
    /// A versioned server API contract.
    ServerApi(&'static str),
    /// State or context owned by the core Wasm application.
    CoreState(&'static str),
    /// A package that links the core surface.
    CorePackage(&'static str),
    /// Another independently delivered page package.
    Satellite(&'static str),
    /// Another explicitly named build-time dependency.
    Named(&'static str),
}

/// Typed compatibility role whose identity is also a build dependency edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityDependencyKind {
    /// Versioned shared UI foundation.
    SharedFoundation,
    /// Versioned server API surface.
    ServerApi,
}

impl PageDependency {
    fn identity(self) -> &'static str {
        match self {
            Self::SharedFoundation(identity)
            | Self::ServerApi(identity)
            | Self::CoreState(identity)
            | Self::CorePackage(identity)
            | Self::Satellite(identity)
            | Self::Named(identity) => identity,
        }
    }
}

/// Source ownership and forbidden-path metadata consumed by CI/CD validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceOwnership {
    /// Globs whose inputs are owned by this page.
    pub owned_globs: &'static [&'static str],
    /// Globs this page worker or package must not change or import.
    pub forbidden_globs: &'static [&'static str],
    /// Explicit compile-time dependency edges.
    pub dependencies: &'static [PageDependency],
}

/// Selector that replaces the page's source dataset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSelector {
    /// The page has no dataset selector.
    None,
    /// The canonical Office selector.
    Office,
    /// A product-specific selector with a stable field key.
    Named(&'static str),
}

impl DatasetSelector {
    fn key(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Office => Some("office"),
            Self::Named(key) => Some(key),
        }
    }
}

/// Initial dataset selection policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetDefault {
    /// No dataset is selected implicitly.
    None,
    /// Select the office from the authenticated user context.
    UserContextOffice,
    /// Use a product-specific named default policy.
    Named(&'static str),
}

/// How selecting a dataset obtains its replacement data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetLoad {
    /// No dataset load occurs.
    None,
    /// Validate a complete replacement and swap it atomically.
    AtomicSnapshot,
    /// Query the server for each requested table view.
    ServerQuery,
}

/// Typed dataset-selector semantics for a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DatasetContract {
    /// Selector that changes the source dataset.
    pub selector: DatasetSelector,
    /// Initial selection policy.
    pub default: DatasetDefault,
    /// Whether the selector exposes an explicit all-datasets choice.
    pub allow_all: bool,
    /// Replacement loading policy.
    pub load: DatasetLoad,
}

/// Boundary at which table search, filtering, sorting, and paging execute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataMode {
    /// The complete bounded dataset is manipulated in browser memory.
    ClientSnapshot,
    /// Each view change is represented in a server query.
    ServerQuery,
}

/// Stable identity contract for rows rendered by a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RowIdentity {
    /// The page does not render identifiable rows.
    None,
    /// Stable domain field used as the keyed row identity.
    Stable(&'static str),
}

/// Where and how the declared sort runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortExecution {
    /// The page exposes no sorting.
    None,
    /// One sort key is evaluated locally.
    LocalSingleColumn,
    /// An ordered list of sort keys is evaluated locally.
    LocalMultiColumn,
    /// Sort changes invoke a server callback.
    ServerCallback,
}

/// User controls promised by a page contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageControl {
    /// Free-text search over the loaded view.
    Search,
    /// A local filter keyed by a stable field name.
    LocalFilter(&'static str),
    /// User-selectable page size.
    PageSize,
    /// Resizable columns.
    ResizeColumns,
    /// Reorderable columns.
    ReorderColumns,
    /// Hideable and showable columns.
    ToggleColumns,
    /// Paged navigation through the derived result.
    Pagination,
    /// A product-specific typed control.
    Named(&'static str),
}

/// Data mode, row identity, sorting, and local control declarations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DataContract {
    /// Client-snapshot or server-query mode.
    pub mode: DataMode,
    /// Stable identity of rendered rows.
    pub row_identity: RowIdentity,
    /// Sort execution boundary.
    pub sort: SortExecution,
    /// Controls exposed by the page.
    pub controls: &'static [PageControl],
}

/// State field whose lifetime is declared by the page contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateField {
    /// Current dataset selector value.
    DatasetSelector(&'static str),
    /// Free-text search input.
    FreeTextSearch,
    /// Current local result page.
    CurrentPage,
    /// Loaded source rows.
    Rows,
    /// Snapshot or event revision identity.
    SnapshotRevision,
    /// Request identities used to reject late responses.
    RequestIds,
    /// Child-session authorization state.
    ChildSession,
    /// Public tab or other tab-local state.
    TabState,
    /// Local filter values.
    Filters,
    /// Sort columns and directions.
    Sort,
    /// Selected page size.
    PageSize,
    /// Column visibility choices.
    ColumnVisibility,
    /// Ordered column keys.
    ColumnOrder,
    /// Column widths.
    ColumnWidths,
    /// A product-specific state field.
    Named(&'static str),
}

/// Explicit durable-default allowlist and transient-state declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StateOwnership {
    /// Fields persisted only by the page's explicit save-default action.
    pub persisted_default: &'static [StateField],
    /// Fields that stay within the page instance or child session.
    pub transient: &'static [StateField],
}

/// Observable outcomes required for a mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOutcome {
    /// The authoritative mutation is in flight.
    Pending,
    /// The authoritative mutation succeeded without prescribing a UI consequence.
    Success,
    /// The authoritative mutation succeeded and its UI consequence is row removal.
    SuccessRemoval,
    /// Another actor or stale revision won the mutation race.
    Conflict,
    /// The mutation failed and can be surfaced or retried safely.
    Failure,
}

/// One named mutation and the complete outcome vocabulary it promises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MutationContract {
    /// Stable mutation name.
    pub name: &'static str,
    /// Server-resolved capability required to perform it.
    pub capability: &'static str,
    /// Observable result states.
    pub outcomes: &'static [MutationOutcome],
}

/// Transport used for scoped server-to-page changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeTransport {
    /// The page has no live update transport.
    None,
    /// The page consumes Server-Sent Events.
    ServerSentEvents,
}

/// A typed live event understood by the page reducer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeEvent {
    /// A product-specific event with a stable name.
    Named(&'static str),
}

/// Recovery states required around a live event transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeState {
    /// The transport is disconnected or retrying.
    Disconnected,
    /// A revision gap or invalidation is being repaired from authoritative data.
    Resynchronizing,
}

/// Live-event declarations for one page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RealtimeContract {
    /// Transport used by the page.
    pub transport: RealtimeTransport,
    /// Event vocabulary applied by the reducer.
    pub events: &'static [RealtimeEvent],
    /// Disconnect and recovery states exposed by the page.
    pub states: &'static [RealtimeState],
}

/// Operation protected by a named server capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAction {
    /// View or load the page and its dataset.
    View,
    /// Invoke a named mutation.
    Mutation(&'static str),
    /// Invoke another named page operation.
    Named(&'static str),
}

/// Server-authoritative capability required for a page operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CapabilityRule {
    /// Protected page operation.
    pub action: CapabilityAction,
    /// Stable server capability name.
    pub capability: &'static str,
}

/// Named responsive layout a page promises to support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsiveLayout {
    /// Standard desktop layout.
    Desktop,
    /// Shared compact layout for mobile and narrow windows.
    CompactMobile,
    /// Product-specific viewport class.
    Named(&'static str),
}

/// Shared responsive behavior required by the page composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsiveBehavior {
    /// Horizontal controls wrap under framework-owned breakpoint rules.
    HorizontalControlsWrap,
    /// A full-width table uses the shared overflow viewport.
    FullWidthTableOverflow,
    /// Rows use the framework's compact presentation when space is constrained.
    CompactRows,
    /// Product-specific responsive behavior.
    Named(&'static str),
}

/// Responsive layouts and shared behaviors promised by the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ResponsiveContract {
    /// Supported named layout classes.
    pub layouts: &'static [ResponsiveLayout],
    /// Framework-owned behavior applied across those layouts.
    pub behaviors: &'static [ResponsiveBehavior],
}

/// Accessibility behavior a page must prove.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityObligation {
    /// Predictable native keyboard navigation and activation.
    KeyboardNavigation,
    /// Visible focus for every interactive control.
    VisibleFocus,
    /// Stable accessible names and correct roles.
    AccessibleNames,
    /// Announcements for asynchronous outcomes not otherwise obvious.
    AsyncAnnouncements,
    /// Deterministic focus recovery when a keyed row is removed.
    FocusAfterRowRemoval,
    /// Keyboard operation for resize, reorder, visibility, sorting, and paging.
    ColumnOperations,
    /// Product-specific accessibility obligation.
    Named(&'static str),
}

/// One stable accessible label required by the contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AccessibleLabel {
    /// Stable purpose key used by tests and implementation.
    pub purpose: &'static str,
    /// User-facing accessible name.
    pub label: &'static str,
}

impl AccessibleLabel {
    /// Creates a stable purpose-to-label declaration.
    pub const fn new(purpose: &'static str, label: &'static str) -> Self {
        Self { purpose, label }
    }
}

/// Accessibility obligations and named control labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AccessibilityContract {
    /// Behavior families the page must prove.
    pub obligations: &'static [AccessibilityObligation],
    /// Stable accessible names used by page tests.
    pub labels: &'static [AccessibleLabel],
}

/// Exhaustive named UI state carried by a v2 contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationState {
    /// Initial load with no usable snapshot.
    InitialLoading,
    /// Ready desktop presentation.
    ReadyDesktop,
    /// Ready compact/mobile presentation.
    ReadyMobile,
    /// Replacement dataset is loading while the old dataset remains visible.
    OfficeSwitching,
    /// Initial snapshot request failed.
    InitialError,
    /// Refresh failed while a usable snapshot remains visible.
    RetainedRefreshError,
    /// No dataset has ever loaded.
    NeverLoaded,
    /// Loaded dataset contains no rows.
    EmptyDataset,
    /// Local controls exclude every loaded row.
    FilteredEmpty,
    /// Displayed data is known to be stale.
    Stale,
    /// Claim mutation is pending.
    ClaimPending,
    /// Claim succeeded and the authoritative row was removed.
    ClaimSuccessRemoval,
    /// Recent-call confirmation is required before claim.
    RecentCallConfirmation,
    /// Claim encountered an authoritative conflict.
    ClaimConflict,
    /// Claim failed and can be retried or explained.
    ClaimFailure,
    /// Claim succeeded remotely but local reconciliation is pending.
    UnsyncedClaim,
    /// Live updates are disconnected or reconnecting.
    StreamInterrupted,
    /// The child session is expired or revoked.
    SessionExpired,
    /// The actor is not permitted to view or act.
    Forbidden,
    /// Durable preferences are being saved.
    PreferenceSaving,
    /// Durable preferences were saved.
    PreferenceSaved,
    /// Durable preference revision conflicted.
    PreferenceConflict,
    /// Durable preference save failed.
    PreferenceFailure,
    /// Product-specific named presentation state.
    Named(&'static str),
}

/// Page-owned verification lane selected by contract metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestLane {
    /// Static and typed contract validation.
    Contract,
    /// Native model and reducer tests.
    Native,
    /// Exact production surface compilation and artifact checks.
    ProductionBuild,
    /// Browser behavior journeys.
    Browser,
    /// Automated and keyboard accessibility checks.
    Accessibility,
    /// Reviewed named-state visual comparisons.
    Visual,
    /// Launch, capability, child-session, and revocation checks.
    SecuritySession,
    /// Bundle, latency, memory, and isolation measurement.
    Measurement,
}

/// Enforced bundle, interaction, or test-lane budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageBudget {
    /// Maximum Brotli-compressed Wasm bytes.
    BundleBrotliBytes {
        /// Inclusive maximum byte count.
        max: u64,
    },
    /// Maximum p95 latency for a named operation and dataset size.
    InteractionP95Millis {
        /// Stable operation name.
        operation: &'static str,
        /// Fixture row count at which the budget applies.
        rows: u32,
        /// Inclusive p95 latency in milliseconds.
        max: u64,
    },
    /// Maximum duration for one selected test lane.
    TestLaneMillis {
        /// Lane constrained by the budget.
        lane: TestLane,
        /// Inclusive duration in milliseconds.
        max: u64,
    },
}

/// Shared-foundation, server-API, and preference compatibility identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CompatibilityContract {
    /// Shared UI foundation compatibility identity.
    pub foundation: &'static str,
    /// Server surface API compatibility identity.
    pub server_api: &'static str,
    /// Durable preference schema version.
    pub preference_schema: u16,
}

/// Reviewed visual baseline bound to one named state and responsive layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NamedBaseline {
    /// Stable baseline identity.
    pub name: &'static str,
    /// Presentation state captured by the baseline.
    pub state: PresentationState,
    /// Responsive layout captured by the baseline.
    pub layout: ResponsiveLayout,
}

impl NamedBaseline {
    /// Creates a named state/layout baseline declaration.
    pub const fn new(
        name: &'static str,
        state: PresentationState,
        layout: ResponsiveLayout,
    ) -> Self {
        Self {
            name,
            state,
            layout,
        }
    }
}

/// Complete compile-time PageContract v2 declaration for one surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageContractV2 {
    /// Contract schema version; must be [`PAGE_CONTRACT_V2_VERSION`].
    pub contract_version: u16,
    /// Stable page and surface identifier.
    pub id: &'static str,
    /// User-facing page title.
    pub title: &'static str,
    /// Product or repository owner identifier.
    pub owner: &'static str,
    /// Core or satellite delivery boundary.
    pub delivery: PageDelivery,
    /// Opinionated page archetype.
    pub archetype: PageArchetype,
    /// Stable user-facing route family.
    pub route: &'static str,
    /// Owned paths, forbidden paths, and dependency edges.
    pub source: SourceOwnership,
    /// Dataset selector and replacement semantics.
    pub dataset: DatasetContract,
    /// Data mode, stable row identity, sort boundary, and controls.
    pub data: DataContract,
    /// Durable-default allowlist and transient-state declaration.
    pub state: StateOwnership,
    /// Page mutations and their complete outcomes.
    pub mutations: &'static [MutationContract],
    /// Realtime transport, events, and recovery states.
    pub realtime: RealtimeContract,
    /// Server-authoritative capability rules.
    pub capabilities: &'static [CapabilityRule],
    /// Responsive layouts and shared behavior.
    pub responsive: ResponsiveContract,
    /// Accessibility obligations and named labels.
    pub accessibility: AccessibilityContract,
    /// Exhaustive user-visible state vocabulary.
    pub presentation_states: &'static [PresentationState],
    /// Required page-level verification lanes.
    pub test_lanes: &'static [TestLane],
    /// Enforced bundle, interaction, and lane budgets.
    pub budgets: &'static [PageBudget],
    /// Shared foundation, API, and preference compatibility.
    pub compatibility: CompatibilityContract,
    /// Reviewed visual baselines keyed by named state and layout.
    pub baselines: &'static [NamedBaseline],
}

/// Stable serializable envelope used by native contract exporters.
#[derive(Debug)]
pub struct PageContractExport<'a> {
    schema: &'static str,
    contract: &'a PageContractV2,
}

/// Failure to validate or serialize a public PageContract v2 export.
#[derive(Debug)]
pub enum PageContractExportError {
    /// The contract contradicted the frozen v2 schema and was not serialized.
    Validation(Vec<PageContractV2Error>),
    /// A validated contract could not be encoded as JSON.
    Serialization(serde_json::Error),
}

impl fmt::Display for PageContractExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(errors) => write!(
                formatter,
                "page contract has {} validation error(s): {errors:?}",
                errors.len(),
            ),
            Self::Serialization(error) => write!(formatter, "page contract export failed: {error}"),
        }
    }
}

impl std::error::Error for PageContractExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Validation(_) => None,
            Self::Serialization(error) => Some(error),
        }
    }
}

impl Serialize for PageContractExport<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut export = serializer.serialize_struct("PageContractExport", 2)?;
        export.serialize_field("schema", self.schema)?;
        export.serialize_field("contract", &SerializablePageContract(self.contract))?;
        export.end()
    }
}

struct SerializablePageContract<'a>(&'a PageContractV2);

impl Serialize for SerializablePageContract<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let contract = self.0;
        let mut descriptor = serializer.serialize_struct("PageContractV2", 21)?;
        descriptor.serialize_field("contract_version", &contract.contract_version)?;
        descriptor.serialize_field("id", contract.id)?;
        descriptor.serialize_field("title", contract.title)?;
        descriptor.serialize_field("owner", contract.owner)?;
        descriptor.serialize_field("delivery", &contract.delivery)?;
        descriptor.serialize_field("archetype", &contract.archetype)?;
        descriptor.serialize_field("route", contract.route)?;
        descriptor.serialize_field("source", &contract.source)?;
        descriptor.serialize_field("dataset", &contract.dataset)?;
        descriptor.serialize_field("data", &contract.data)?;
        descriptor.serialize_field("state", &contract.state)?;
        descriptor.serialize_field("mutations", contract.mutations)?;
        descriptor.serialize_field("realtime", &contract.realtime)?;
        descriptor.serialize_field("capabilities", contract.capabilities)?;
        descriptor.serialize_field("responsive", &contract.responsive)?;
        descriptor.serialize_field("accessibility", &contract.accessibility)?;
        descriptor.serialize_field("presentation_states", contract.presentation_states)?;
        descriptor.serialize_field("test_lanes", contract.test_lanes)?;
        descriptor.serialize_field("budgets", contract.budgets)?;
        descriptor.serialize_field("compatibility", &contract.compatibility)?;
        descriptor.serialize_field("baselines", contract.baselines)?;
        descriptor.end()
    }
}

impl PageContractV2 {
    /// Returns every v2 schema violation without performing I/O.
    pub fn validate(&self) -> Result<(), Vec<PageContractV2Error>> {
        let mut errors = Vec::new();

        if self.contract_version != PAGE_CONTRACT_V2_VERSION {
            errors.push(PageContractV2Error::UnsupportedContractVersion(
                self.contract_version,
            ));
        }
        if self.id.trim().is_empty() {
            errors.push(PageContractV2Error::EmptyId);
        }
        if self.title.trim().is_empty() {
            errors.push(PageContractV2Error::EmptyTitle);
        }
        if self.owner.trim().is_empty() {
            errors.push(PageContractV2Error::EmptyOwner);
        }
        if self.route.is_empty() {
            errors.push(PageContractV2Error::EmptyRoute);
        } else {
            if !self.route.starts_with('/') {
                errors.push(PageContractV2Error::RouteMustBeAbsolute(self.route));
            }
            if self.route.trim() != self.route
                || self.route.chars().any(char::is_whitespace)
                || self.route.contains('?')
                || self.route.contains('#')
                || self.route.contains("/satellites/")
                || self.route == "/satellites"
                || self.route.starts_with("/satellites/")
            {
                errors.push(PageContractV2Error::RouteMustBeStable(self.route));
            }
        }

        if self.source.owned_globs.is_empty() {
            errors.push(PageContractV2Error::MissingOwnedGlobs);
        }
        for (index, glob) in self.source.owned_globs.iter().enumerate() {
            if glob.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyOwnedGlob);
            }
            if self.source.owned_globs[..index].contains(glob) {
                errors.push(PageContractV2Error::DuplicateOwnedGlob(glob));
            }
            if !glob.trim().is_empty() && self.source.forbidden_globs.contains(glob) {
                errors.push(PageContractV2Error::OwnedGlobIsForbidden(glob));
            }
        }
        if self.source.forbidden_globs.is_empty() {
            errors.push(PageContractV2Error::MissingForbiddenGlobs);
        }
        for (index, glob) in self.source.forbidden_globs.iter().enumerate() {
            if glob.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyForbiddenGlob);
            }
            if self.source.forbidden_globs[..index].contains(glob) {
                errors.push(PageContractV2Error::DuplicateForbiddenGlob(glob));
            }
        }
        let mut foundation_dependency = None;
        let mut foundation_dependency_count = 0;
        let mut server_api_dependency = None;
        let mut server_api_dependency_count = 0;
        for (index, dependency) in self.source.dependencies.iter().enumerate() {
            if dependency.identity().trim().is_empty() {
                errors.push(PageContractV2Error::EmptyDependencyIdentity(*dependency));
            }
            match dependency {
                PageDependency::SharedFoundation(identity) => {
                    foundation_dependency_count += 1;
                    foundation_dependency.get_or_insert(*identity);
                }
                PageDependency::ServerApi(identity) => {
                    server_api_dependency_count += 1;
                    server_api_dependency.get_or_insert(*identity);
                }
                PageDependency::CoreState(_)
                | PageDependency::CorePackage(_)
                | PageDependency::Satellite(_)
                | PageDependency::Named(_) => {}
            }
            if self.source.dependencies[..index].contains(dependency) {
                errors.push(PageContractV2Error::DuplicateDependency(*dependency));
            }
        }
        match (foundation_dependency_count, foundation_dependency) {
            (0, _) | (1, None) => errors.push(PageContractV2Error::MissingCompatibilityDependency(
                CompatibilityDependencyKind::SharedFoundation,
            )),
            (1, Some(dependency)) if dependency != self.compatibility.foundation => {
                errors.push(PageContractV2Error::CompatibilityDependencyMismatch {
                    kind: CompatibilityDependencyKind::SharedFoundation,
                    dependency,
                    compatibility: self.compatibility.foundation,
                });
            }
            (1, Some(_)) => {}
            (_, _) => errors.push(PageContractV2Error::DuplicateCompatibilityDependency(
                CompatibilityDependencyKind::SharedFoundation,
            )),
        }
        match (server_api_dependency_count, server_api_dependency) {
            (0, _) | (1, None) => errors.push(PageContractV2Error::MissingCompatibilityDependency(
                CompatibilityDependencyKind::ServerApi,
            )),
            (1, Some(dependency)) if dependency != self.compatibility.server_api => {
                errors.push(PageContractV2Error::CompatibilityDependencyMismatch {
                    kind: CompatibilityDependencyKind::ServerApi,
                    dependency,
                    compatibility: self.compatibility.server_api,
                });
            }
            (1, Some(_)) => {}
            (_, _) => errors.push(PageContractV2Error::DuplicateCompatibilityDependency(
                CompatibilityDependencyKind::ServerApi,
            )),
        }

        if matches!(
            self.dataset.selector,
            DatasetSelector::Named(name) if name.trim().is_empty()
        ) {
            errors.push(PageContractV2Error::EmptyNamedValue(
                ContractNameKind::DatasetSelector,
            ));
        }
        if matches!(
            self.dataset.default,
            DatasetDefault::Named(name) if name.trim().is_empty()
        ) {
            errors.push(PageContractV2Error::EmptyNamedValue(
                ContractNameKind::DatasetDefault,
            ));
        }
        if let Some(selector_key) = self.dataset.selector.key() {
            for control in self.data.controls {
                if let PageControl::LocalFilter(filter_key) = control
                    && filter_key == &selector_key
                {
                    errors.push(PageContractV2Error::DatasetSelectorIsFilter(selector_key));
                }
            }
        }
        if matches!(
            self.data.row_identity,
            RowIdentity::Stable(name) if name.trim().is_empty()
        ) {
            errors.push(PageContractV2Error::EmptyNamedValue(
                ContractNameKind::RowIdentity,
            ));
        }
        for (index, control) in self.data.controls.iter().enumerate() {
            match control {
                PageControl::LocalFilter(name) if name.trim().is_empty() => errors.push(
                    PageContractV2Error::EmptyNamedValue(ContractNameKind::LocalFilter),
                ),
                PageControl::Named(name) if name.trim().is_empty() => errors.push(
                    PageContractV2Error::EmptyNamedValue(ContractNameKind::PageControl),
                ),
                _ => {}
            }
            if self.data.controls[..index].contains(control) {
                errors.push(PageContractV2Error::DuplicateControl(*control));
            }
        }

        if self.data.mode == DataMode::ClientSnapshot
            && self.data.sort == SortExecution::ServerCallback
        {
            errors.push(PageContractV2Error::ClientSnapshotUsesServerSort);
        }
        match self.archetype {
            PageArchetype::SnapshotTablePage => {
                if self.data.mode != DataMode::ClientSnapshot {
                    errors.push(PageContractV2Error::ArchetypeDataModeMismatch {
                        archetype: self.archetype,
                        declared: self.data.mode,
                    });
                }
                if self.dataset.load != DatasetLoad::AtomicSnapshot {
                    errors.push(PageContractV2Error::ArchetypeDatasetLoadMismatch {
                        archetype: self.archetype,
                        declared: self.dataset.load,
                    });
                }
                if self.data.sort == SortExecution::ServerCallback {
                    errors.push(PageContractV2Error::ArchetypeSortMismatch {
                        archetype: self.archetype,
                        declared: self.data.sort,
                    });
                }
                if self.data.row_identity == RowIdentity::None {
                    errors.push(PageContractV2Error::TableArchetypeMissingRowIdentity(
                        self.archetype,
                    ));
                }
            }
            PageArchetype::ServerTablePage => {
                if self.data.mode != DataMode::ServerQuery {
                    errors.push(PageContractV2Error::ArchetypeDataModeMismatch {
                        archetype: self.archetype,
                        declared: self.data.mode,
                    });
                }
                if self.dataset.load != DatasetLoad::ServerQuery {
                    errors.push(PageContractV2Error::ArchetypeDatasetLoadMismatch {
                        archetype: self.archetype,
                        declared: self.dataset.load,
                    });
                }
                if !matches!(
                    self.data.sort,
                    SortExecution::None | SortExecution::ServerCallback
                ) {
                    errors.push(PageContractV2Error::ArchetypeSortMismatch {
                        archetype: self.archetype,
                        declared: self.data.sort,
                    });
                }
                if self.data.row_identity == RowIdentity::None {
                    errors.push(PageContractV2Error::TableArchetypeMissingRowIdentity(
                        self.archetype,
                    ));
                }
            }
            PageArchetype::RecordDetailPage
            | PageArchetype::FormWorkflowPage
            | PageArchetype::DashboardPage
            | PageArchetype::SettingsPage => {}
        }

        for (index, field) in self.state.persisted_default.iter().enumerate() {
            if matches!(
                field,
                StateField::DatasetSelector(name) | StateField::Named(name)
                    if name.trim().is_empty()
            ) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::StateField,
                ));
            }
            if self.state.persisted_default[..index].contains(field) {
                errors.push(PageContractV2Error::DuplicatePersistedState(*field));
            }
            if self.state.transient.contains(field) {
                errors.push(PageContractV2Error::StateIsPersistedAndTransient(*field));
            }
            if matches!(
                field,
                StateField::DatasetSelector(_)
                    | StateField::FreeTextSearch
                    | StateField::CurrentPage
                    | StateField::Rows
                    | StateField::SnapshotRevision
                    | StateField::RequestIds
                    | StateField::ChildSession
                    | StateField::TabState
            ) {
                errors.push(PageContractV2Error::ForbiddenPersistedState(*field));
            }
        }
        for (index, field) in self.state.transient.iter().enumerate() {
            if matches!(
                field,
                StateField::DatasetSelector(name) | StateField::Named(name)
                    if name.trim().is_empty()
            ) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::StateField,
                ));
            }
            if self.state.transient[..index].contains(field) {
                errors.push(PageContractV2Error::DuplicateTransientState(*field));
            }
        }

        if self.delivery == PageDelivery::Satellite {
            for dependency in self.source.dependencies {
                match dependency {
                    PageDependency::CoreState(name) | PageDependency::CorePackage(name) => {
                        errors.push(PageContractV2Error::SatelliteDependsOnCore(name));
                    }
                    PageDependency::Satellite(name) => {
                        errors.push(PageContractV2Error::SatelliteDependsOnSatellite(name));
                    }
                    PageDependency::SharedFoundation(_)
                    | PageDependency::ServerApi(_)
                    | PageDependency::Named(_) => {}
                }
            }
        }
        if self.delivery == PageDelivery::Core {
            for dependency in self.source.dependencies {
                if let PageDependency::Satellite(name) = dependency {
                    errors.push(PageContractV2Error::CoreDependsOnSatellite(name));
                }
            }
        }

        const REQUIRED_MUTATION_OUTCOMES: &[MutationOutcome] = &[
            MutationOutcome::Pending,
            MutationOutcome::Conflict,
            MutationOutcome::Failure,
        ];
        for (mutation_index, mutation) in self.mutations.iter().enumerate() {
            if mutation.name.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyMutationName);
            }
            if mutation.capability.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyMutationCapability(mutation.name));
            }
            if self.mutations[..mutation_index]
                .iter()
                .any(|candidate| candidate.name == mutation.name)
            {
                errors.push(PageContractV2Error::DuplicateMutation(mutation.name));
            }
            for (outcome_index, outcome) in mutation.outcomes.iter().enumerate() {
                if mutation.outcomes[..outcome_index].contains(outcome) {
                    errors.push(PageContractV2Error::DuplicateMutationOutcome {
                        mutation: mutation.name,
                        outcome: *outcome,
                    });
                }
            }
            for outcome in REQUIRED_MUTATION_OUTCOMES {
                if !mutation.outcomes.contains(outcome) {
                    errors.push(PageContractV2Error::MissingMutationOutcome {
                        mutation: mutation.name,
                        outcome: *outcome,
                    });
                }
            }
            if !mutation.outcomes.iter().any(|outcome| {
                matches!(
                    outcome,
                    MutationOutcome::Success | MutationOutcome::SuccessRemoval
                )
            }) {
                errors.push(PageContractV2Error::MissingMutationSuccess(mutation.name));
            }
        }

        if self.realtime.transport != RealtimeTransport::None || !self.realtime.events.is_empty() {
            for state in [RealtimeState::Disconnected, RealtimeState::Resynchronizing] {
                if !self.realtime.states.contains(&state) {
                    errors.push(PageContractV2Error::MissingRealtimeState(state));
                }
            }
        }
        for (index, event) in self.realtime.events.iter().enumerate() {
            if matches!(event, RealtimeEvent::Named(name) if name.trim().is_empty()) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::RealtimeEvent,
                ));
            }
            if self.realtime.events[..index].contains(event) {
                errors.push(PageContractV2Error::DuplicateRealtimeEvent(*event));
            }
        }
        for (index, state) in self.realtime.states.iter().enumerate() {
            if self.realtime.states[..index].contains(state) {
                errors.push(PageContractV2Error::DuplicateRealtimeState(*state));
            }
        }

        for (index, rule) in self.capabilities.iter().enumerate() {
            if rule.capability.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyCapabilityName(rule.action));
            }
            if matches!(
                rule.action,
                CapabilityAction::Mutation(name) | CapabilityAction::Named(name)
                    if name.trim().is_empty()
            ) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::CapabilityAction,
                ));
            }
            if self.capabilities[..index]
                .iter()
                .any(|candidate| candidate.action == rule.action)
            {
                errors.push(PageContractV2Error::DuplicateCapabilityAction(rule.action));
            }
        }
        for mutation in self.mutations {
            match self
                .capabilities
                .iter()
                .find(|rule| rule.action == CapabilityAction::Mutation(mutation.name))
            {
                None => errors.push(PageContractV2Error::MissingMutationCapabilityRule(
                    mutation.name,
                )),
                Some(rule) if rule.capability != mutation.capability => {
                    errors.push(PageContractV2Error::MutationCapabilityMismatch {
                        mutation: mutation.name,
                        expected: mutation.capability,
                        declared: rule.capability,
                    });
                }
                Some(_) => {}
            }
        }
        for rule in self.capabilities {
            if let CapabilityAction::Mutation(name) = rule.action
                && !self.mutations.iter().any(|mutation| mutation.name == name)
            {
                errors.push(PageContractV2Error::CapabilityForUnknownMutation(name));
            }
        }
        for (index, layout) in self.responsive.layouts.iter().enumerate() {
            if matches!(layout, ResponsiveLayout::Named(name) if name.trim().is_empty()) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::ResponsiveLayout,
                ));
            }
            if self.responsive.layouts[..index].contains(layout) {
                errors.push(PageContractV2Error::DuplicateResponsiveLayout(*layout));
            }
        }
        for (index, behavior) in self.responsive.behaviors.iter().enumerate() {
            if matches!(behavior, ResponsiveBehavior::Named(name) if name.trim().is_empty()) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::ResponsiveBehavior,
                ));
            }
            if self.responsive.behaviors[..index].contains(behavior) {
                errors.push(PageContractV2Error::DuplicateResponsiveBehavior(*behavior));
            }
        }
        for (index, obligation) in self.accessibility.obligations.iter().enumerate() {
            if matches!(
                obligation,
                AccessibilityObligation::Named(name) if name.trim().is_empty()
            ) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::AccessibilityObligation,
                ));
            }
            if self.accessibility.obligations[..index].contains(obligation) {
                errors.push(PageContractV2Error::DuplicateAccessibilityObligation(
                    *obligation,
                ));
            }
        }
        for (index, label) in self.accessibility.labels.iter().enumerate() {
            if label.purpose.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyAccessibleLabelPurpose);
            }
            if label.label.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyAccessibleLabelText(label.purpose));
            }
            if self.accessibility.labels[..index]
                .iter()
                .any(|candidate| candidate.purpose == label.purpose)
            {
                errors.push(PageContractV2Error::DuplicateAccessibleLabel(label.purpose));
            }
        }
        for (index, state) in self.presentation_states.iter().enumerate() {
            if matches!(state, PresentationState::Named(name) if name.trim().is_empty()) {
                errors.push(PageContractV2Error::EmptyNamedValue(
                    ContractNameKind::PresentationState,
                ));
            }
            if self.presentation_states[..index].contains(state) {
                errors.push(PageContractV2Error::DuplicatePresentationState(*state));
            }
        }
        for (index, lane) in self.test_lanes.iter().enumerate() {
            if self.test_lanes[..index].contains(lane) {
                errors.push(PageContractV2Error::DuplicateTestLane(*lane));
            }
        }

        if self.compatibility.foundation.trim().is_empty() {
            errors.push(PageContractV2Error::EmptyFoundationCompatibility);
        }
        if self.compatibility.server_api.trim().is_empty() {
            errors.push(PageContractV2Error::EmptyServerApiCompatibility);
        }
        if self.compatibility.preference_schema == 0 {
            errors.push(PageContractV2Error::ZeroPreferenceSchema);
        }

        for (index, budget) in self.budgets.iter().enumerate() {
            match budget {
                PageBudget::BundleBrotliBytes { max } => {
                    if *max == 0 {
                        errors.push(PageContractV2Error::ZeroBundleBudget);
                    }
                    if self.budgets[..index]
                        .iter()
                        .any(|candidate| matches!(candidate, PageBudget::BundleBrotliBytes { .. }))
                    {
                        errors.push(PageContractV2Error::DuplicateBundleBudget);
                    }
                }
                PageBudget::InteractionP95Millis {
                    operation,
                    rows,
                    max,
                } => {
                    if operation.trim().is_empty() {
                        errors.push(PageContractV2Error::EmptyInteractionBudgetOperation);
                    }
                    if *rows == 0 {
                        errors.push(PageContractV2Error::ZeroInteractionBudgetRows(operation));
                    }
                    if *max == 0 {
                        errors.push(PageContractV2Error::ZeroInteractionBudgetMillis(operation));
                    }
                    if self.budgets[..index].iter().any(|candidate| {
                        matches!(
                            candidate,
                            PageBudget::InteractionP95Millis {
                                operation: candidate_operation,
                                ..
                            } if candidate_operation == operation
                        )
                    }) {
                        errors.push(PageContractV2Error::DuplicateInteractionBudget(operation));
                    }
                }
                PageBudget::TestLaneMillis { lane, max } => {
                    if !self.test_lanes.contains(lane) {
                        errors.push(PageContractV2Error::BudgetForUndeclaredLane(*lane));
                    }
                    if *max == 0 {
                        errors.push(PageContractV2Error::ZeroTestLaneBudget(*lane));
                    }
                    if self.budgets[..index].iter().any(|candidate| {
                        matches!(
                            candidate,
                            PageBudget::TestLaneMillis {
                                lane: candidate_lane,
                                ..
                            } if candidate_lane == lane
                        )
                    }) {
                        errors.push(PageContractV2Error::DuplicateTestLaneBudget(*lane));
                    }
                }
            }
        }

        if !self.test_lanes.contains(&TestLane::Visual) && !self.baselines.is_empty() {
            errors.push(PageContractV2Error::BaselinesRequireVisualLane);
        }
        if self.test_lanes.contains(&TestLane::Visual)
            && !self
                .baselines
                .iter()
                .any(|baseline| !baseline.name.trim().is_empty())
        {
            errors.push(PageContractV2Error::VisualLaneMissingBaseline);
        }
        for (index, baseline) in self.baselines.iter().enumerate() {
            if baseline.name.trim().is_empty() {
                errors.push(PageContractV2Error::EmptyBaselineName);
            }
            if self.baselines[..index]
                .iter()
                .any(|candidate| candidate.name == baseline.name)
            {
                errors.push(PageContractV2Error::DuplicateBaseline(baseline.name));
            }
            if !self.presentation_states.contains(&baseline.state) {
                errors.push(PageContractV2Error::BaselineStateNotDeclared(
                    baseline.state,
                ));
            }
            if !self.responsive.layouts.contains(&baseline.layout) {
                errors.push(PageContractV2Error::BaselineLayoutNotDeclared(
                    baseline.layout,
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Returns a typed export envelope with a stable schema identity.
    pub fn export(&self) -> Result<PageContractExport<'_>, PageContractExportError> {
        self.validate()
            .map_err(PageContractExportError::Validation)?;
        Ok(PageContractExport {
            schema: PAGE_CONTRACT_V2_EXPORT_SCHEMA,
            contract: self,
        })
    }

    /// Validates and serializes the typed export in stable declaration order.
    pub fn export_json(&self) -> Result<String, PageContractExportError> {
        let export = self.export()?;
        serde_json::to_string(&export).map_err(PageContractExportError::Serialization)
    }
}

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
