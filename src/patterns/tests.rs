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

page_contract! {
    pub NO_HIRES_V2: v2 {
        contract_version: 2,
        id: "no-hires",
        title: "No Hires",
        owner: "office",
        delivery: Satellite,
        archetype: SnapshotTablePage,
        route: "/no-hires/",
        source: SourceOwnership {
            owned_globs: &[
                "surfaces/no-hires/src/**",
                "surfaces/no-hires/tests/**",
                "docs/testing/surfaces/no-hires/baselines/**",
            ],
            forbidden_globs: &[
                "crates/office-perf-web/**",
                "crates/office-perf-api/src/app_router.rs",
            ],
            dependencies: &[
                PageDependency::SharedFoundation("ldui-surface-v1"),
                PageDependency::ServerApi("office-surface-api-v1"),
            ],
        },
        dataset: DatasetContract {
            selector: DatasetSelector::Office,
            default: DatasetDefault::UserContextOffice,
            allow_all: true,
            load: DatasetLoad::AtomicSnapshot,
        },
        data: DataContract {
            mode: DataMode::ClientSnapshot,
            row_identity: RowIdentity::Stable("deal_id"),
            sort: SortExecution::LocalMultiColumn,
            controls: &[
                PageControl::Search,
                PageControl::LocalFilter("status"),
                PageControl::LocalFilter("case_type"),
                PageControl::PageSize,
                PageControl::ResizeColumns,
                PageControl::ReorderColumns,
                PageControl::ToggleColumns,
                PageControl::Pagination,
            ],
        },
        state: StateOwnership {
            persisted_default: &[
                StateField::Filters,
                StateField::Sort,
                StateField::PageSize,
                StateField::ColumnVisibility,
                StateField::ColumnOrder,
                StateField::ColumnWidths,
            ],
            transient: &[
                StateField::DatasetSelector("office"),
                StateField::FreeTextSearch,
                StateField::CurrentPage,
                StateField::Rows,
                StateField::SnapshotRevision,
                StateField::RequestIds,
                StateField::ChildSession,
                StateField::TabState,
            ],
        },
        mutations: &[MutationContract {
            name: "claim",
            capability: "office.no-hires.claim",
            outcomes: &[
                MutationOutcome::Pending,
                MutationOutcome::SuccessRemoval,
                MutationOutcome::Conflict,
                MutationOutcome::Failure,
            ],
        }],
        realtime: RealtimeContract {
            transport: RealtimeTransport::ServerSentEvents,
            events: &[
                RealtimeEvent::Named("claimed"),
                RealtimeEvent::Named("snapshot_invalidated"),
            ],
            states: &[
                RealtimeState::Disconnected,
                RealtimeState::Resynchronizing,
            ],
        },
        capabilities: &[
            CapabilityRule {
                action: CapabilityAction::View,
                capability: "office.no-hires.read",
            },
            CapabilityRule {
                action: CapabilityAction::Mutation("claim"),
                capability: "office.no-hires.claim",
            },
        ],
        responsive: ResponsiveContract {
            layouts: &[ResponsiveLayout::Desktop, ResponsiveLayout::CompactMobile],
            behaviors: &[
                ResponsiveBehavior::HorizontalControlsWrap,
                ResponsiveBehavior::FullWidthTableOverflow,
                ResponsiveBehavior::CompactRows,
            ],
        },
        accessibility: AccessibilityContract {
            obligations: &[
                AccessibilityObligation::KeyboardNavigation,
                AccessibilityObligation::VisibleFocus,
                AccessibilityObligation::AccessibleNames,
                AccessibilityObligation::AsyncAnnouncements,
                AccessibilityObligation::FocusAfterRowRemoval,
                AccessibilityObligation::ColumnOperations,
            ],
            labels: &[
                AccessibleLabel::new("dataset_selector", "Office"),
                AccessibleLabel::new("save_default", "Save as Default"),
                AccessibleLabel::new("claim", "Claim no-hire record"),
            ],
        },
        presentation_states: &[
            PresentationState::InitialLoading,
            PresentationState::ReadyDesktop,
            PresentationState::ReadyMobile,
            PresentationState::OfficeSwitching,
            PresentationState::InitialError,
            PresentationState::RetainedRefreshError,
            PresentationState::NeverLoaded,
            PresentationState::EmptyDataset,
            PresentationState::FilteredEmpty,
            PresentationState::Stale,
            PresentationState::ClaimPending,
            PresentationState::ClaimSuccessRemoval,
            PresentationState::RecentCallConfirmation,
            PresentationState::ClaimConflict,
            PresentationState::ClaimFailure,
            PresentationState::UnsyncedClaim,
            PresentationState::StreamInterrupted,
            PresentationState::SessionExpired,
            PresentationState::Forbidden,
            PresentationState::PreferenceSaving,
            PresentationState::PreferenceSaved,
            PresentationState::PreferenceConflict,
            PresentationState::PreferenceFailure,
        ],
        test_lanes: &[
            TestLane::Contract,
            TestLane::Native,
            TestLane::ProductionBuild,
            TestLane::Browser,
            TestLane::Accessibility,
            TestLane::Visual,
            TestLane::SecuritySession,
            TestLane::Measurement,
        ],
        budgets: &[
            PageBudget::BundleBrotliBytes { max: 512_000 },
            PageBudget::InteractionP95Millis {
                operation: "local_view",
                rows: 2_000,
                max: 100,
            },
            PageBudget::TestLaneMillis {
                lane: TestLane::Native,
                max: 20_000,
            },
            PageBudget::TestLaneMillis {
                lane: TestLane::Visual,
                max: 180_000,
            },
        ],
        compatibility: CompatibilityContract {
            foundation: "ldui-surface-v1",
            server_api: "office-surface-api-v1",
            preference_schema: 1,
        },
        baselines: &[
            NamedBaseline::new(
                "ready-desktop",
                PresentationState::ReadyDesktop,
                ResponsiveLayout::Desktop,
            ),
            NamedBaseline::new(
                "ready-mobile",
                PresentationState::ReadyMobile,
                ResponsiveLayout::CompactMobile,
            ),
        ],
    }
}

#[test]
fn legacy_contract_error_remains_exhaustively_matchable() {
    fn stable_variant_name(error: ContractError) -> &'static str {
        match error {
            ContractError::EmptyId => "empty_id",
            ContractError::EmptyRoute => "empty_route",
            ContractError::RouteMustBeAbsolute(_) => "route_must_be_absolute",
            ContractError::DuplicateState(_) => "duplicate_state",
            ContractError::DuplicateLocalState(_) => "duplicate_local_state",
            ContractError::EmptyLocalState => "empty_local_state",
            ContractError::DuplicateBreakpoint(_) => "duplicate_breakpoint",
            ContractError::MissingState(_) => "missing_state",
            ContractError::MissingBreakpoint(_) => "missing_breakpoint",
            ContractError::EmptyDatasetSelector => "empty_dataset_selector",
            ContractError::DatasetSelectorIsLocalState(_) => "dataset_selector_is_local_state",
            ContractError::DatasetSelectorIsFilter(_) => "dataset_selector_is_filter",
            ContractError::EmptyFilter => "empty_filter",
            ContractError::DuplicateFilter(_) => "duplicate_filter",
        }
    }

    assert_eq!(
        stable_variant_name(ContractError::DatasetSelectorIsFilter("office")),
        "dataset_selector_is_filter"
    );
}

#[test]
fn page_contract_v2_declares_and_exports_the_complete_surface_contract() {
    assert_eq!(NO_HIRES_V2.validate(), Ok(()));

    let first = NO_HIRES_V2
        .export_json()
        .expect("typed page-contract data has a JSON representation");
    let second = NO_HIRES_V2
        .export_json()
        .expect("repeated export remains available");

    assert_eq!(first, second);
    let mut cursor = first
        .find("\"contract\":{")
        .expect("the export envelope contains its contract object");
    for field in [
        "contract_version",
        "id",
        "title",
        "owner",
        "delivery",
        "archetype",
        "route",
        "source",
        "dataset",
        "data",
        "state",
        "mutations",
        "realtime",
        "capabilities",
        "responsive",
        "accessibility",
        "presentation_states",
        "test_lanes",
        "budgets",
        "compatibility",
        "baselines",
    ] {
        let needle = format!("\"{field}\":");
        let offset = first[cursor..]
            .find(&needle)
            .unwrap_or_else(|| panic!("{field} must appear in the frozen export order"));
        cursor += offset + needle.len();
    }
    let descriptor: serde_json::Value =
        serde_json::from_str(&first).expect("validated export is JSON");
    assert_eq!(
        descriptor,
        serde_json::json!({
            "schema": "ldui.page-contract.v2",
            "contract": {
                "contract_version": 2,
                "id": "no-hires",
                "title": "No Hires",
                "owner": "office",
                "delivery": "satellite",
                "archetype": "snapshot_table_page",
                "route": "/no-hires/",
                "source": {
                    "owned_globs": [
                        "surfaces/no-hires/src/**",
                        "surfaces/no-hires/tests/**",
                        "docs/testing/surfaces/no-hires/baselines/**"
                    ],
                    "forbidden_globs": [
                        "crates/office-perf-web/**",
                        "crates/office-perf-api/src/app_router.rs"
                    ],
                    "dependencies": [
                        { "shared_foundation": "ldui-surface-v1" },
                        { "server_api": "office-surface-api-v1" }
                    ]
                },
                "dataset": {
                    "selector": "office",
                    "default": "user_context_office",
                    "allow_all": true,
                    "load": "atomic_snapshot"
                },
                "data": {
                    "mode": "client_snapshot",
                    "row_identity": { "stable": "deal_id" },
                    "sort": "local_multi_column",
                    "controls": [
                        "search",
                        { "local_filter": "status" },
                        { "local_filter": "case_type" },
                        "page_size",
                        "resize_columns",
                        "reorder_columns",
                        "toggle_columns",
                        "pagination"
                    ]
                },
                "state": {
                    "persisted_default": [
                        "filters",
                        "sort",
                        "page_size",
                        "column_visibility",
                        "column_order",
                        "column_widths"
                    ],
                    "transient": [
                        { "dataset_selector": "office" },
                        "free_text_search",
                        "current_page",
                        "rows",
                        "snapshot_revision",
                        "request_ids",
                        "child_session",
                        "tab_state"
                    ]
                },
                "mutations": [{
                    "name": "claim",
                    "capability": "office.no-hires.claim",
                    "outcomes": ["pending", "success_removal", "conflict", "failure"]
                }],
                "realtime": {
                    "transport": "server_sent_events",
                    "events": [
                        { "named": "claimed" },
                        { "named": "snapshot_invalidated" }
                    ],
                    "states": ["disconnected", "resynchronizing"]
                },
                "capabilities": [
                    {
                        "action": "view",
                        "capability": "office.no-hires.read"
                    },
                    {
                        "action": { "mutation": "claim" },
                        "capability": "office.no-hires.claim"
                    }
                ],
                "responsive": {
                    "layouts": ["desktop", "compact_mobile"],
                    "behaviors": [
                        "horizontal_controls_wrap",
                        "full_width_table_overflow",
                        "compact_rows"
                    ]
                },
                "accessibility": {
                    "obligations": [
                        "keyboard_navigation",
                        "visible_focus",
                        "accessible_names",
                        "async_announcements",
                        "focus_after_row_removal",
                        "column_operations"
                    ],
                    "labels": [
                        { "purpose": "dataset_selector", "label": "Office" },
                        { "purpose": "save_default", "label": "Save as Default" },
                        { "purpose": "claim", "label": "Claim no-hire record" }
                    ]
                },
                "presentation_states": [
                    "initial_loading",
                    "ready_desktop",
                    "ready_mobile",
                    "office_switching",
                    "initial_error",
                    "retained_refresh_error",
                    "never_loaded",
                    "empty_dataset",
                    "filtered_empty",
                    "stale",
                    "claim_pending",
                    "claim_success_removal",
                    "recent_call_confirmation",
                    "claim_conflict",
                    "claim_failure",
                    "unsynced_claim",
                    "stream_interrupted",
                    "session_expired",
                    "forbidden",
                    "preference_saving",
                    "preference_saved",
                    "preference_conflict",
                    "preference_failure"
                ],
                "test_lanes": [
                    "contract",
                    "native",
                    "production_build",
                    "browser",
                    "accessibility",
                    "visual",
                    "security_session",
                    "measurement"
                ],
                "budgets": [
                    { "bundle_brotli_bytes": { "max": 512000 } },
                    {
                        "interaction_p95_millis": {
                            "operation": "local_view",
                            "rows": 2000,
                            "max": 100
                        }
                    },
                    {
                        "test_lane_millis": {
                            "lane": "native",
                            "max": 20000
                        }
                    },
                    {
                        "test_lane_millis": {
                            "lane": "visual",
                            "max": 180000
                        }
                    }
                ],
                "compatibility": {
                    "foundation": "ldui-surface-v1",
                    "server_api": "office-surface-api-v1",
                    "preference_schema": 1
                },
                "baselines": [
                    {
                        "name": "ready-desktop",
                        "state": "ready_desktop",
                        "layout": "desktop"
                    },
                    {
                        "name": "ready-mobile",
                        "state": "ready_mobile",
                        "layout": "compact_mobile"
                    }
                ]
            }
        })
    );
}

#[test]
fn page_contract_v2_refuses_to_export_an_invalid_contract() {
    const CONTROLS: &[PageControl] = &[PageControl::LocalFilter("office")];
    let contradictory = PageContractV2 {
        data: DataContract {
            controls: CONTROLS,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    for error in [
        contradictory.export().unwrap_err(),
        contradictory.export_json().unwrap_err(),
    ] {
        match error {
            PageContractExportError::Validation(errors) => {
                assert!(errors.contains(&PageContractV2Error::DatasetSelectorIsFilter("office")))
            }
            PageContractExportError::Serialization(error) => {
                panic!("validation must run before serialization: {error}")
            }
        }
    }
}

#[test]
fn page_contract_v2_export_error_names_validation_failures() {
    const CONTROLS: &[PageControl] = &[PageControl::LocalFilter("office")];
    let contradictory = PageContractV2 {
        data: DataContract {
            controls: CONTROLS,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    let rendered = contradictory.export_json().unwrap_err().to_string();
    assert!(
        rendered.contains("DatasetSelectorIsFilter(\"office\")"),
        "validation display must retain actionable diagnostics: {rendered}"
    );
}

#[test]
fn page_contract_v2_rejects_the_dataset_selector_as_a_local_filter() {
    const CONTROLS: &[PageControl] = &[PageControl::Search, PageControl::LocalFilter("office")];
    let contract = PageContractV2 {
        data: DataContract {
            controls: CONTROLS,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    assert!(
        contract
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::DatasetSelectorIsFilter("office"))
    );
}

#[test]
fn page_contract_v2_rejects_server_sort_callbacks_for_client_snapshots() {
    let contract = PageContractV2 {
        data: DataContract {
            sort: SortExecution::ServerCallback,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    assert!(
        contract
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::ClientSnapshotUsesServerSort)
    );
}

#[test]
fn page_contract_v2_rejects_incoherent_snapshot_table_data_declarations() {
    let contract = PageContractV2 {
        dataset: DatasetContract {
            load: DatasetLoad::ServerQuery,
            ..NO_HIRES_V2.dataset
        },
        data: DataContract {
            mode: DataMode::ServerQuery,
            row_identity: RowIdentity::None,
            sort: SortExecution::ServerCallback,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeDataModeMismatch {
            archetype: PageArchetype::SnapshotTablePage,
            declared: DataMode::ServerQuery,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeDatasetLoadMismatch {
            archetype: PageArchetype::SnapshotTablePage,
            declared: DatasetLoad::ServerQuery,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeSortMismatch {
            archetype: PageArchetype::SnapshotTablePage,
            declared: SortExecution::ServerCallback,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::TableArchetypeMissingRowIdentity(
            PageArchetype::SnapshotTablePage
        ))
    );
}

#[test]
fn page_contract_v2_rejects_incoherent_server_table_data_declarations() {
    let contract = PageContractV2 {
        archetype: PageArchetype::ServerTablePage,
        data: DataContract {
            mode: DataMode::ClientSnapshot,
            row_identity: RowIdentity::None,
            sort: SortExecution::LocalMultiColumn,
            ..NO_HIRES_V2.data
        },
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeDataModeMismatch {
            archetype: PageArchetype::ServerTablePage,
            declared: DataMode::ClientSnapshot,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeDatasetLoadMismatch {
            archetype: PageArchetype::ServerTablePage,
            declared: DatasetLoad::AtomicSnapshot,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::ArchetypeSortMismatch {
            archetype: PageArchetype::ServerTablePage,
            declared: SortExecution::LocalMultiColumn,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::TableArchetypeMissingRowIdentity(
            PageArchetype::ServerTablePage
        ))
    );
}

#[test]
fn page_contract_v2_accepts_sortable_and_unsortable_server_tables() {
    for sort in [SortExecution::None, SortExecution::ServerCallback] {
        let contract = PageContractV2 {
            archetype: PageArchetype::ServerTablePage,
            dataset: DatasetContract {
                load: DatasetLoad::ServerQuery,
                ..NO_HIRES_V2.dataset
            },
            data: DataContract {
                mode: DataMode::ServerQuery,
                sort,
                ..NO_HIRES_V2.data
            },
            ..NO_HIRES_V2
        };

        assert_eq!(contract.validate(), Ok(()), "{sort:?} must be coherent");
    }
}

#[test]
fn page_contract_v2_rejects_transient_or_authority_state_as_a_persisted_default() {
    const FORBIDDEN: &[StateField] = &[
        StateField::DatasetSelector("office"),
        StateField::FreeTextSearch,
        StateField::CurrentPage,
        StateField::Rows,
        StateField::SnapshotRevision,
        StateField::RequestIds,
        StateField::ChildSession,
        StateField::TabState,
    ];
    let contract = PageContractV2 {
        state: StateOwnership {
            persisted_default: FORBIDDEN,
            ..NO_HIRES_V2.state
        },
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    for field in FORBIDDEN {
        assert!(
            errors.contains(&PageContractV2Error::ForbiddenPersistedState(*field)),
            "{field:?} must never be stored as a durable page default"
        );
    }
}

#[test]
fn page_contract_v2_rejects_satellite_dependencies_on_core() {
    const DEPENDENCIES: &[PageDependency] = &[
        PageDependency::CoreState("office_app_context"),
        PageDependency::CorePackage("office-perf-web"),
    ];
    let contract = PageContractV2 {
        source: SourceOwnership {
            dependencies: DEPENDENCIES,
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::SatelliteDependsOnCore(
            "office_app_context"
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::SatelliteDependsOnCore(
            "office-perf-web"
        ))
    );
}

#[test]
fn page_contract_v2_rejects_satellite_dependencies_on_another_satellite() {
    const DEPENDENCIES: &[PageDependency] = &[PageDependency::Satellite("inventory-aging")];
    let contract = PageContractV2 {
        source: SourceOwnership {
            dependencies: DEPENDENCIES,
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };

    assert!(contract.validate().unwrap_err().contains(
        &PageContractV2Error::SatelliteDependsOnSatellite("inventory-aging")
    ));
}

#[test]
fn page_contract_v2_rejects_core_dependencies_on_a_satellite() {
    const DEPENDENCIES: &[PageDependency] = &[
        PageDependency::SharedFoundation("ldui-surface-v1"),
        PageDependency::ServerApi("office-surface-api-v1"),
        PageDependency::Satellite("inventory-aging"),
    ];
    let contract = PageContractV2 {
        delivery: PageDelivery::Core,
        source: SourceOwnership {
            dependencies: DEPENDENCIES,
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };

    assert!(contract.validate().unwrap_err().contains(
        &PageContractV2Error::CoreDependsOnSatellite("inventory-aging")
    ));
}

#[test]
fn page_contract_v2_rejects_mutations_without_every_authoritative_outcome() {
    const MUTATIONS: &[MutationContract] = &[MutationContract {
        name: "claim",
        capability: "office.no-hires.claim",
        outcomes: &[],
    }];
    let contract = PageContractV2 {
        mutations: MUTATIONS,
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    for outcome in [
        MutationOutcome::Pending,
        MutationOutcome::Conflict,
        MutationOutcome::Failure,
    ] {
        assert!(
            errors.contains(&PageContractV2Error::MissingMutationOutcome {
                mutation: "claim",
                outcome,
            })
        );
    }
    assert!(errors.contains(&PageContractV2Error::MissingMutationSuccess("claim")));
}

#[test]
fn page_contract_v2_accepts_generic_success_without_claiming_row_removal() {
    const MUTATIONS: &[MutationContract] = &[MutationContract {
        name: "save",
        capability: "office.settings.write",
        outcomes: &[
            MutationOutcome::Pending,
            MutationOutcome::Success,
            MutationOutcome::Conflict,
            MutationOutcome::Failure,
        ],
    }];
    const CAPABILITIES: &[CapabilityRule] = &[
        CapabilityRule {
            action: CapabilityAction::View,
            capability: "office.no-hires.read",
        },
        CapabilityRule {
            action: CapabilityAction::Mutation("save"),
            capability: "office.settings.write",
        },
    ];
    let contract = PageContractV2 {
        archetype: PageArchetype::SettingsPage,
        mutations: MUTATIONS,
        capabilities: CAPABILITIES,
        ..NO_HIRES_V2
    };

    assert_eq!(contract.validate(), Ok(()));
}

#[test]
fn page_contract_v2_rejects_realtime_without_disconnect_and_resynchronization() {
    let contract = PageContractV2 {
        realtime: RealtimeContract {
            states: &[],
            ..NO_HIRES_V2.realtime
        },
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::MissingRealtimeState(
        RealtimeState::Disconnected
    )));
    assert!(errors.contains(&PageContractV2Error::MissingRealtimeState(
        RealtimeState::Resynchronizing
    )));
}

#[test]
fn page_contract_v2_rejects_visual_lanes_without_a_nonempty_named_baseline() {
    let absent = PageContractV2 {
        baselines: &[],
        ..NO_HIRES_V2
    };
    assert!(
        absent
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::VisualLaneMissingBaseline)
    );

    const UNNAMED: &[NamedBaseline] = &[NamedBaseline::new(
        "",
        PresentationState::ReadyDesktop,
        ResponsiveLayout::Desktop,
    )];
    let unnamed = PageContractV2 {
        baselines: UNNAMED,
        ..NO_HIRES_V2
    };
    assert!(
        unnamed
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::EmptyBaselineName)
    );
}

#[test]
fn page_contract_v2_rejects_baselines_when_the_visual_lane_is_absent() {
    const LANES: &[TestLane] = &[
        TestLane::Contract,
        TestLane::Native,
        TestLane::ProductionBuild,
        TestLane::Browser,
        TestLane::Accessibility,
        TestLane::SecuritySession,
        TestLane::Measurement,
    ];
    let contract = PageContractV2 {
        test_lanes: LANES,
        budgets: &[],
        ..NO_HIRES_V2
    };

    assert!(
        contract
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::BaselinesRequireVisualLane)
    );
}

#[test]
fn page_contract_v2_rejects_malformed_identity_version_and_stable_route() {
    let malformed = PageContractV2 {
        contract_version: 1,
        id: " ",
        title: "",
        owner: " ",
        route: "/satellites/no-hires/?tab=public#state",
        ..NO_HIRES_V2
    };
    let errors = malformed.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::UnsupportedContractVersion(1)));
    assert!(errors.contains(&PageContractV2Error::EmptyId));
    assert!(errors.contains(&PageContractV2Error::EmptyTitle));
    assert!(errors.contains(&PageContractV2Error::EmptyOwner));
    assert!(errors.contains(&PageContractV2Error::RouteMustBeStable(
        "/satellites/no-hires/?tab=public#state"
    )));

    let relative = PageContractV2 {
        route: "no-hires/",
        ..NO_HIRES_V2
    };
    assert!(
        relative
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::RouteMustBeAbsolute("no-hires/"))
    );

    let whitespace = PageContractV2 {
        route: "/no hires/",
        ..NO_HIRES_V2
    };
    assert!(
        whitespace
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::RouteMustBeStable("/no hires/"))
    );
}

#[test]
fn page_contract_v2_rejects_malformed_source_ownership_and_compatibility() {
    const OWNED: &[&str] = &["", "surface/**", "surface/**", "shared/**"];
    const FORBIDDEN: &[&str] = &["", "shared/**", "shared/**"];
    let malformed = PageContractV2 {
        source: SourceOwnership {
            owned_globs: OWNED,
            forbidden_globs: FORBIDDEN,
            ..NO_HIRES_V2.source
        },
        compatibility: CompatibilityContract {
            foundation: "",
            server_api: " ",
            preference_schema: 0,
        },
        ..NO_HIRES_V2
    };
    let errors = malformed.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::EmptyOwnedGlob));
    assert!(errors.contains(&PageContractV2Error::EmptyForbiddenGlob));
    assert!(errors.contains(&PageContractV2Error::DuplicateOwnedGlob("surface/**")));
    assert!(errors.contains(&PageContractV2Error::DuplicateForbiddenGlob("shared/**")));
    assert!(errors.contains(&PageContractV2Error::OwnedGlobIsForbidden("shared/**")));
    assert!(errors.contains(&PageContractV2Error::EmptyFoundationCompatibility));
    assert!(errors.contains(&PageContractV2Error::EmptyServerApiCompatibility));
    assert!(errors.contains(&PageContractV2Error::ZeroPreferenceSchema));

    let missing = PageContractV2 {
        source: SourceOwnership {
            owned_globs: &[],
            forbidden_globs: &[],
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };
    let errors = missing.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::MissingOwnedGlobs));
    assert!(errors.contains(&PageContractV2Error::MissingForbiddenGlobs));
}

#[test]
fn page_contract_v2_requires_exactly_one_matching_compatibility_dependency() {
    let missing = PageContractV2 {
        source: SourceOwnership {
            dependencies: &[],
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };
    let errors = missing.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::MissingCompatibilityDependency(
            CompatibilityDependencyKind::SharedFoundation
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::MissingCompatibilityDependency(
            CompatibilityDependencyKind::ServerApi
        ))
    );

    const DUPLICATED: &[PageDependency] = &[
        PageDependency::SharedFoundation("ldui-surface-v1"),
        PageDependency::SharedFoundation("ldui-surface-v2"),
        PageDependency::ServerApi("office-surface-api-v1"),
        PageDependency::ServerApi("office-surface-api-v2"),
    ];
    let duplicated = PageContractV2 {
        source: SourceOwnership {
            dependencies: DUPLICATED,
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };
    let errors = duplicated.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::DuplicateCompatibilityDependency(
            CompatibilityDependencyKind::SharedFoundation
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateCompatibilityDependency(
            CompatibilityDependencyKind::ServerApi
        ))
    );

    const MISMATCHED: &[PageDependency] = &[
        PageDependency::SharedFoundation("ldui-surface-v2"),
        PageDependency::ServerApi("office-surface-api-v2"),
    ];
    let mismatched = PageContractV2 {
        source: SourceOwnership {
            dependencies: MISMATCHED,
            ..NO_HIRES_V2.source
        },
        ..NO_HIRES_V2
    };
    let errors = mismatched.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::CompatibilityDependencyMismatch {
            kind: CompatibilityDependencyKind::SharedFoundation,
            dependency: "ldui-surface-v2",
            compatibility: "ldui-surface-v1",
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::CompatibilityDependencyMismatch {
            kind: CompatibilityDependencyKind::ServerApi,
            dependency: "office-surface-api-v2",
            compatibility: "office-surface-api-v1",
        })
    );
}

#[test]
fn page_contract_v2_rejects_duplicate_closed_declarations() {
    const DEPENDENCIES: &[PageDependency] = &[
        PageDependency::SharedFoundation("ldui-surface-v1"),
        PageDependency::SharedFoundation("ldui-surface-v1"),
    ];
    const CONTROLS: &[PageControl] = &[PageControl::Search, PageControl::Search];
    const PERSISTED: &[StateField] = &[StateField::Filters, StateField::Filters];
    const TRANSIENT: &[StateField] = &[
        StateField::Filters,
        StateField::CurrentPage,
        StateField::CurrentPage,
    ];
    const OUTCOMES: &[MutationOutcome] = &[
        MutationOutcome::Pending,
        MutationOutcome::Pending,
        MutationOutcome::SuccessRemoval,
        MutationOutcome::Conflict,
        MutationOutcome::Failure,
    ];
    const MUTATIONS: &[MutationContract] = &[
        MutationContract {
            name: "claim",
            capability: "office.no-hires.claim",
            outcomes: OUTCOMES,
        },
        MutationContract {
            name: "claim",
            capability: "office.no-hires.claim",
            outcomes: &[
                MutationOutcome::Pending,
                MutationOutcome::SuccessRemoval,
                MutationOutcome::Conflict,
                MutationOutcome::Failure,
            ],
        },
    ];
    const EVENTS: &[RealtimeEvent] = &[
        RealtimeEvent::Named("claimed"),
        RealtimeEvent::Named("claimed"),
    ];
    const REALTIME_STATES: &[RealtimeState] = &[
        RealtimeState::Disconnected,
        RealtimeState::Disconnected,
        RealtimeState::Resynchronizing,
    ];
    const CAPABILITIES: &[CapabilityRule] = &[
        CapabilityRule {
            action: CapabilityAction::View,
            capability: "office.no-hires.read",
        },
        CapabilityRule {
            action: CapabilityAction::View,
            capability: "office.no-hires.read",
        },
    ];
    const LAYOUTS: &[ResponsiveLayout] = &[
        ResponsiveLayout::Desktop,
        ResponsiveLayout::Desktop,
        ResponsiveLayout::CompactMobile,
    ];
    const BEHAVIORS: &[ResponsiveBehavior] = &[
        ResponsiveBehavior::HorizontalControlsWrap,
        ResponsiveBehavior::HorizontalControlsWrap,
    ];
    const OBLIGATIONS: &[AccessibilityObligation] = &[
        AccessibilityObligation::KeyboardNavigation,
        AccessibilityObligation::KeyboardNavigation,
    ];
    const LABELS: &[AccessibleLabel] = &[
        AccessibleLabel::new("claim", "Claim row"),
        AccessibleLabel::new("claim", "Claim no-hire record"),
    ];
    const PRESENTATION_STATES: &[PresentationState] = &[
        PresentationState::ReadyDesktop,
        PresentationState::ReadyDesktop,
        PresentationState::ReadyMobile,
    ];
    const TEST_LANES: &[TestLane] = &[TestLane::Contract, TestLane::Visual, TestLane::Visual];
    const BASELINES: &[NamedBaseline] = &[
        NamedBaseline::new(
            "ready",
            PresentationState::ReadyDesktop,
            ResponsiveLayout::Desktop,
        ),
        NamedBaseline::new(
            "ready",
            PresentationState::ReadyMobile,
            ResponsiveLayout::CompactMobile,
        ),
    ];
    let contract = PageContractV2 {
        source: SourceOwnership {
            dependencies: DEPENDENCIES,
            ..NO_HIRES_V2.source
        },
        data: DataContract {
            controls: CONTROLS,
            ..NO_HIRES_V2.data
        },
        state: StateOwnership {
            persisted_default: PERSISTED,
            transient: TRANSIENT,
        },
        mutations: MUTATIONS,
        realtime: RealtimeContract {
            events: EVENTS,
            states: REALTIME_STATES,
            ..NO_HIRES_V2.realtime
        },
        capabilities: CAPABILITIES,
        responsive: ResponsiveContract {
            layouts: LAYOUTS,
            behaviors: BEHAVIORS,
        },
        accessibility: AccessibilityContract {
            obligations: OBLIGATIONS,
            labels: LABELS,
        },
        presentation_states: PRESENTATION_STATES,
        test_lanes: TEST_LANES,
        baselines: BASELINES,
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::DuplicateDependency(
        PageDependency::SharedFoundation("ldui-surface-v1")
    )));
    assert!(errors.contains(&PageContractV2Error::DuplicateControl(PageControl::Search)));
    assert!(
        errors.contains(&PageContractV2Error::DuplicatePersistedState(
            StateField::Filters
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateTransientState(
            StateField::CurrentPage
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::StateIsPersistedAndTransient(
            StateField::Filters
        ))
    );
    assert!(errors.contains(&PageContractV2Error::DuplicateMutation("claim")));
    assert!(
        errors.contains(&PageContractV2Error::DuplicateMutationOutcome {
            mutation: "claim",
            outcome: MutationOutcome::Pending,
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateRealtimeEvent(
            RealtimeEvent::Named("claimed")
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateRealtimeState(
            RealtimeState::Disconnected
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateCapabilityAction(
            CapabilityAction::View
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateResponsiveLayout(
            ResponsiveLayout::Desktop
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateResponsiveBehavior(
            ResponsiveBehavior::HorizontalControlsWrap
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::DuplicateAccessibilityObligation(
            AccessibilityObligation::KeyboardNavigation
        ))
    );
    assert!(errors.contains(&PageContractV2Error::DuplicateAccessibleLabel("claim")));
    assert!(
        errors.contains(&PageContractV2Error::DuplicatePresentationState(
            PresentationState::ReadyDesktop
        ))
    );
    assert!(errors.contains(&PageContractV2Error::DuplicateTestLane(TestLane::Visual)));
    assert!(errors.contains(&PageContractV2Error::DuplicateBaseline("ready")));
}

#[test]
fn page_contract_v2_rejects_invalid_or_unselected_lane_budgets() {
    const LANES: &[TestLane] = &[TestLane::Contract, TestLane::Native, TestLane::Native];
    const BUDGETS: &[PageBudget] = &[
        PageBudget::BundleBrotliBytes { max: 0 },
        PageBudget::BundleBrotliBytes { max: 512_000 },
        PageBudget::InteractionP95Millis {
            operation: "",
            rows: 0,
            max: 0,
        },
        PageBudget::InteractionP95Millis {
            operation: "",
            rows: 2_000,
            max: 100,
        },
        PageBudget::TestLaneMillis {
            lane: TestLane::Visual,
            max: 0,
        },
        PageBudget::TestLaneMillis {
            lane: TestLane::Visual,
            max: 180_000,
        },
    ];
    let contract = PageContractV2 {
        test_lanes: LANES,
        budgets: BUDGETS,
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(errors.contains(&PageContractV2Error::DuplicateTestLane(TestLane::Native)));
    assert!(errors.contains(&PageContractV2Error::ZeroBundleBudget));
    assert!(errors.contains(&PageContractV2Error::DuplicateBundleBudget));
    assert!(errors.contains(&PageContractV2Error::EmptyInteractionBudgetOperation));
    assert!(errors.contains(&PageContractV2Error::ZeroInteractionBudgetRows("")));
    assert!(errors.contains(&PageContractV2Error::ZeroInteractionBudgetMillis("")));
    assert!(errors.contains(&PageContractV2Error::DuplicateInteractionBudget("")));
    assert!(
        errors.contains(&PageContractV2Error::BudgetForUndeclaredLane(
            TestLane::Visual
        ))
    );
    assert!(errors.contains(&PageContractV2Error::ZeroTestLaneBudget(TestLane::Visual)));
    assert!(
        errors.contains(&PageContractV2Error::DuplicateTestLaneBudget(
            TestLane::Visual
        ))
    );
}

#[test]
fn page_contract_v2_rejects_baselines_for_undeclared_states_or_layouts() {
    const STATES: &[PresentationState] = &[PresentationState::ReadyDesktop];
    const LAYOUTS: &[ResponsiveLayout] = &[ResponsiveLayout::Desktop];
    const BASELINES: &[NamedBaseline] = &[NamedBaseline::new(
        "ready-mobile",
        PresentationState::ReadyMobile,
        ResponsiveLayout::CompactMobile,
    )];
    let contract = PageContractV2 {
        presentation_states: STATES,
        responsive: ResponsiveContract {
            layouts: LAYOUTS,
            ..NO_HIRES_V2.responsive
        },
        baselines: BASELINES,
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::BaselineStateNotDeclared(
            PresentationState::ReadyMobile
        ))
    );
    assert!(
        errors.contains(&PageContractV2Error::BaselineLayoutNotDeclared(
            ResponsiveLayout::CompactMobile
        ))
    );
}

#[test]
fn page_contract_v2_rejects_empty_extensible_and_descriptor_names() {
    const DEPENDENCIES: &[PageDependency] = &[
        PageDependency::SharedFoundation(""),
        PageDependency::ServerApi(" "),
        PageDependency::Named(""),
    ];
    const CONTROLS: &[PageControl] = &[
        PageControl::Search,
        PageControl::LocalFilter(""),
        PageControl::Named(" "),
    ];
    const PERSISTED: &[StateField] = &[StateField::Named("")];
    const TRANSIENT: &[StateField] = &[StateField::DatasetSelector(" ")];
    const MUTATIONS: &[MutationContract] = &[MutationContract {
        name: "",
        capability: " ",
        outcomes: &[
            MutationOutcome::Pending,
            MutationOutcome::SuccessRemoval,
            MutationOutcome::Conflict,
            MutationOutcome::Failure,
        ],
    }];
    const EVENTS: &[RealtimeEvent] = &[RealtimeEvent::Named("")];
    const CAPABILITIES: &[CapabilityRule] = &[
        CapabilityRule {
            action: CapabilityAction::View,
            capability: "",
        },
        CapabilityRule {
            action: CapabilityAction::Named(" "),
            capability: "named.capability",
        },
        CapabilityRule {
            action: CapabilityAction::Mutation(""),
            capability: "mutation.capability",
        },
    ];
    const LAYOUTS: &[ResponsiveLayout] = &[ResponsiveLayout::Desktop, ResponsiveLayout::Named("")];
    const BEHAVIORS: &[ResponsiveBehavior] = &[
        ResponsiveBehavior::HorizontalControlsWrap,
        ResponsiveBehavior::Named(" "),
    ];
    const OBLIGATIONS: &[AccessibilityObligation] = &[
        AccessibilityObligation::KeyboardNavigation,
        AccessibilityObligation::Named(""),
    ];
    const LABELS: &[AccessibleLabel] = &[
        AccessibleLabel::new("", "Label"),
        AccessibleLabel::new("empty_text", " "),
    ];
    const PRESENTATION_STATES: &[PresentationState] = &[
        PresentationState::ReadyDesktop,
        PresentationState::ReadyMobile,
        PresentationState::Named(""),
    ];
    let contract = PageContractV2 {
        source: SourceOwnership {
            dependencies: DEPENDENCIES,
            ..NO_HIRES_V2.source
        },
        dataset: DatasetContract {
            selector: DatasetSelector::Named(""),
            default: DatasetDefault::Named(" "),
            ..NO_HIRES_V2.dataset
        },
        data: DataContract {
            row_identity: RowIdentity::Stable(" "),
            controls: CONTROLS,
            ..NO_HIRES_V2.data
        },
        state: StateOwnership {
            persisted_default: PERSISTED,
            transient: TRANSIENT,
        },
        mutations: MUTATIONS,
        realtime: RealtimeContract {
            events: EVENTS,
            ..NO_HIRES_V2.realtime
        },
        capabilities: CAPABILITIES,
        responsive: ResponsiveContract {
            layouts: LAYOUTS,
            behaviors: BEHAVIORS,
        },
        accessibility: AccessibilityContract {
            obligations: OBLIGATIONS,
            labels: LABELS,
        },
        presentation_states: PRESENTATION_STATES,
        ..NO_HIRES_V2
    };

    let errors = contract.validate().unwrap_err();
    for dependency in DEPENDENCIES {
        assert!(errors.contains(&PageContractV2Error::EmptyDependencyIdentity(*dependency)));
    }
    assert!(errors.contains(&PageContractV2Error::EmptyMutationName));
    assert!(errors.contains(&PageContractV2Error::EmptyMutationCapability("")));
    assert!(errors.contains(&PageContractV2Error::EmptyCapabilityName(
        CapabilityAction::View
    )));
    assert!(errors.contains(&PageContractV2Error::EmptyAccessibleLabelPurpose));
    assert!(errors.contains(&PageContractV2Error::EmptyAccessibleLabelText("empty_text")));
    for kind in [
        ContractNameKind::DatasetSelector,
        ContractNameKind::DatasetDefault,
        ContractNameKind::RowIdentity,
        ContractNameKind::LocalFilter,
        ContractNameKind::PageControl,
        ContractNameKind::StateField,
        ContractNameKind::RealtimeEvent,
        ContractNameKind::CapabilityAction,
        ContractNameKind::ResponsiveLayout,
        ContractNameKind::ResponsiveBehavior,
        ContractNameKind::AccessibilityObligation,
        ContractNameKind::PresentationState,
    ] {
        assert!(
            errors.contains(&PageContractV2Error::EmptyNamedValue(kind)),
            "{kind:?} must carry a nonempty stable name"
        );
    }
}

#[test]
fn page_contract_v2_requires_mutation_capability_rules_to_match_exactly() {
    const VIEW_ONLY: &[CapabilityRule] = &[CapabilityRule {
        action: CapabilityAction::View,
        capability: "office.no-hires.read",
    }];
    let missing = PageContractV2 {
        capabilities: VIEW_ONLY,
        ..NO_HIRES_V2
    };
    assert!(
        missing
            .validate()
            .unwrap_err()
            .contains(&PageContractV2Error::MissingMutationCapabilityRule("claim"))
    );

    const MISMATCHED: &[CapabilityRule] = &[
        CapabilityRule {
            action: CapabilityAction::View,
            capability: "office.no-hires.read",
        },
        CapabilityRule {
            action: CapabilityAction::Mutation("claim"),
            capability: "office.no-hires.read",
        },
        CapabilityRule {
            action: CapabilityAction::Mutation("archive"),
            capability: "office.no-hires.archive",
        },
    ];
    let mismatched = PageContractV2 {
        capabilities: MISMATCHED,
        ..NO_HIRES_V2
    };
    let errors = mismatched.validate().unwrap_err();
    assert!(
        errors.contains(&PageContractV2Error::MutationCapabilityMismatch {
            mutation: "claim",
            expected: "office.no-hires.claim",
            declared: "office.no-hires.read",
        })
    );
    assert!(
        errors.contains(&PageContractV2Error::CapabilityForUnknownMutation(
            "archive"
        ))
    );
}

filter_schema! {
    pub PILOT_FILTERS: PilotFilters {
        dataset_selector: "office",
        filters: [search, status, case_type],
    }
}

#[derive(Clone)]
struct DeclaredRow {
    id: &'static str,
    name: &'static str,
}

entity_columns! {
    fn declared_columns(prefix: &'static str) -> DeclaredRow => [
        crate::components::EntityColumn::text(
            "client",
            format!("{prefix} Client"),
            |row: &DeclaredRow| row.name.to_owned(),
        ).required(),
        crate::components::EntityColumn::action(
            "actions",
            "Actions",
            |row: &DeclaredRow| row.id.to_owned(),
        ).required(),
    ]
}

#[test]
fn entity_column_declaration_keeps_the_row_type_explicit() {
    let columns = declared_columns("Pilot");
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].header, "Pilot Client");
    assert!(columns[0].required);
    assert!(columns[1].is_action);
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

#[test]
fn list_page_and_filter_bar_defaults_are_full_width_and_wrapping() {
    let list = list_page_class("pilot-page");
    assert!(list.split_ascii_whitespace().any(|class| class == "w-full"));
    assert!(
        list.split_ascii_whitespace()
            .any(|class| class == "min-w-0")
    );
    assert!(
        list.split_ascii_whitespace()
            .any(|class| class == "pilot-page")
    );
    assert!(!list.contains("max-w-"));

    let filters = filter_bar_class("pilot-filters");
    assert!(
        filters
            .split_ascii_whitespace()
            .any(|class| class == "flex-wrap")
    );
    assert!(
        filters
            .split_ascii_whitespace()
            .any(|class| class == "items-end")
    );
    assert!(filters.contains("pilot-filters"));
}

#[test]
fn dataset_selector_resolves_the_selected_dataset_without_filter_semantics() {
    let options = vec![
        DatasetOption::new("mx", "Mexico City"),
        DatasetOption::new("in", "New Delhi"),
    ];
    assert_eq!(selected_dataset_label(&options, "in"), Some("New Delhi"));
    assert_eq!(selected_dataset_label(&options, "missing"), None);
}

#[test]
fn dataset_selector_stays_changeable_while_a_supersedable_load_is_busy() {
    assert!(!super::dataset_selector::selector_disabled(false, true));
    assert!(super::dataset_selector::selector_disabled(true, false));
    assert!(super::dataset_selector::selector_disabled(true, true));
}

#[test]
fn active_filter_summary_is_explicit_and_pluralized() {
    assert_eq!(active_filter_summary(0), "No active filters");
    assert_eq!(active_filter_summary(1), "1 active filter");
    assert_eq!(active_filter_summary(3), "3 active filters");
}

#[test]
fn async_section_retains_usable_snapshots_for_non_initial_states() {
    for state in [
        PageState::Ready,
        PageState::Revalidating,
        PageState::RefreshError,
        PageState::Stale,
        PageState::Claiming,
        PageState::ClaimSucceeded,
        PageState::ClaimConflict,
        PageState::ClaimFailed,
        PageState::LiveInterrupted,
    ] {
        assert!(
            state_shows_content(state),
            "{state:?} should retain content"
        );
    }
    for state in [
        PageState::InitialLoading,
        PageState::InitialError,
        PageState::NeverLoaded,
        PageState::Empty,
        PageState::FilteredEmpty,
    ] {
        assert!(
            !state_shows_content(state),
            "{state:?} should replace content"
        );
    }
}
