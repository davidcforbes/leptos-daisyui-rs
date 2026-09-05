use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    BadgeColor, Button, EntityBadgePresentation, EntityCellEditor, EntityColumn,
    EntityColumnChooserTrigger, EntityColumnFilter, EntityColumnFilterOption, EntityDate,
    EntityDateFilter, EntityDateFilterProposal, EntityDraftCommit, EntityDraftRow,
    EntityEditOutcome, EntityFocusRequest, EntityFocusRequestResolution,
    EntityGroupCollapseProposal, EntityIconColor, EntityIconPresentation, EntityNullOrder,
    EntityPageSize, EntityRowAction, EntityRowEmphasis, EntityRowGroup, EntityRowGrouping,
    EntityTable, EntityTableDisplayProjection, EntityTableMultiSelection,
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence, EntityTablePreferences,
    EntityTableProjectionScope, EntityTableSelection, EntityTableSelectionCause,
    EntityTableSelectionProposal, EntityTableTexts, EntityTableViewportFit,
};
use leptos_daisyui_rs::patterns::{
    ActionFeedbackContent, ActionFeedbackState, ActiveFilterChip, FilterBarTexts, FilterSchema,
    PageHeader, PageHeaderNavigationLayout, SnapshotData, SnapshotDatasetOption,
    SnapshotDatasetSelectorConfig, SnapshotDefaultSave, SnapshotDefaultSaveState,
    SnapshotDeltaDisposition, SnapshotDeltaHandle, SnapshotEntityTableConfig,
    SnapshotFilterActionsConfig, SnapshotLocalRowProjection, SnapshotRequestHandle,
    SnapshotTablePage, SnapshotTableState, SnapshotTransitionDisposition, SnapshotViewDefaults,
};
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureRow {
    id: String,
    client: String,
    status: String,
}

fn rows(dataset: &str) -> Rc<Vec<FixtureRow>> {
    let office = if dataset == "office-in" {
        "New Delhi"
    } else {
        "Mexico City"
    };
    Rc::new(
        (1..=3)
            .map(|index| FixtureRow {
                id: format!("{dataset}-{index}"),
                client: format!("{office} Client {index}"),
                status: if index == 1 { "Urgent" } else { "Ready" }.to_owned(),
            })
            .collect(),
    )
}

fn draft_fixture_rows() -> Rc<Vec<FixtureRow>> {
    Rc::new(
        (1..=30)
            .map(|index| FixtureRow {
                id: format!("office-mx-{index}"),
                client: format!("Mexico City Client {index}"),
                status: if index == 1 { "Urgent" } else { "Ready" }.to_owned(),
            })
            .collect(),
    )
}

fn snapshot(dataset: &str, revision: &str) -> SnapshotData<FixtureRow, String, ()> {
    let rows = rows(dataset);
    SnapshotData::new(
        dataset.to_owned(),
        Rc::clone(&rows),
        revision,
        rows.len(),
        Some(()),
    )
    .expect("fixture snapshot is complete")
}

fn columns() -> Vec<EntityColumn<FixtureRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &FixtureRow| row.client.clone())
            .required()
            .with_min_width(220),
        EntityColumn::text("status", "Status", |row: &FixtureRow| row.status.clone())
            .with_min_width(120),
    ]
}

/// Small real-WASM fixture for the typed snapshot composition contract.
#[component]
pub fn SnapshotTablePageFixture() -> impl IntoView {
    type State = SnapshotTableState<FixtureRow, String, String, (), String>;

    let mut initial = State::new();
    let request = initial
        .start_request("office-mx".to_owned())
        .expect("initial fixture request");
    assert_eq!(
        initial.complete(request, snapshot("office-mx", "mx-r1")),
        leptos_daisyui_rs::patterns::SnapshotTransitionDisposition::Applied
    );

    let state = RwSignal::new_local(initial);
    let pending = RwSignal::new_local(Option::<SnapshotRequestHandle<String>>::None);
    let filter_mode = RwSignal::new("all");
    let local_rows = RwSignal::new_local(Option::<SnapshotLocalRowProjection<FixtureRow>>::None);

    Effect::new(move |_| {
        let mode = filter_mode.get();
        let projection = state.with(|state| {
            let displayed = state.view(None).displayed()?;
            let rows = displayed
                .rows()
                .iter()
                .filter(|row| match mode {
                    "urgent" => row.status == "Urgent",
                    "none" => false,
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            state.local_row_projection(Rc::new(rows))
        });
        local_rows.set(projection);
    });

    let request_dataset = Callback::new(move |dataset: String| {
        let mut next = None;
        state.update(|state| next = state.start_request(dataset).ok());
        pending.set(next);
    });
    let selector = SnapshotDatasetSelectorConfig::new(
        "Office",
        Signal::stored(vec![
            SnapshotDatasetOption::new("office-mx".to_owned(), "Mexico City"),
            SnapshotDatasetOption::new("office-in".to_owned(), "New Delhi"),
        ]),
        Arc::new(|value: &String| value.clone()),
        request_dataset,
    );
    let table = SnapshotEntityTableConfig::new(
        columns(),
        Rc::new(|row: &FixtureRow| row.id.clone()),
        EntityTablePreferenceOwnership::uncontrolled(EntityTablePreferencePersistence::Disabled),
    );

    let start_replacement = Callback::new(move |_| request_dataset.run("office-in".to_owned()));
    let fail_replacement = Callback::new(move |_| {
        let Some(handle) = pending.get_untracked() else {
            return;
        };
        state.update(|state| {
            state.fail(handle, "Fixture replacement failed.".to_owned());
        });
        pending.set(None);
    });
    let complete_replacement = Callback::new(move |_| {
        let Some(handle) = pending.get_untracked() else {
            return;
        };
        state.update(|state| {
            state.complete(handle, snapshot("office-in", "in-r1"));
        });
        pending.set(None);
    });
    let retry = Callback::new(move |_| request_dataset.run("office-in".to_owned()));

    // ldui-vn81 / ldui-cb29: generation-bound displayed-snapshot delta
    // fixtures. Each button mints a fresh `start_delta` handle against the
    // currently displayed snapshot and immediately applies it -- exactly how
    // a real consumer reacts to their own claim response or an incoming SSE
    // removal -- proving rows/revision/count change while the dataset/access
    // generation, and any unrelated in-flight replacement, stay untouched.
    let delta_revision = RwSignal::new(0_u32);
    let last_delta_disposition = RwSignal::new(String::from("(none)"));
    let stale_delta_handle = RwSignal::new_local(Option::<SnapshotDeltaHandle<String>>::None);

    let apply_delta_removing = move |remove_id: Option<String>| {
        state.update(|state| {
            let Ok(handle) = state.start_delta() else {
                last_delta_disposition.set("start_delta failed".to_owned());
                return;
            };
            // Keep the very first minted handle around so a later click can
            // replay it after it has already been consumed -- the
            // duplicate/stale-delta negative control.
            stale_delta_handle.update(|stored| {
                if stored.is_none() {
                    *stored = Some(handle.clone());
                }
            });
            let Some(displayed) = state.view(None).displayed() else {
                last_delta_disposition.set("no displayed snapshot".to_owned());
                return;
            };
            let dataset = displayed.dataset().clone();
            let mut rows = displayed.rows().as_ref().clone();
            if let Some(remove_id) = remove_id {
                rows.retain(|row| row.id != remove_id);
            }
            let next_revision = delta_revision.get_untracked() + 1;
            delta_revision.set(next_revision);
            let row_count = rows.len();
            let new_data = SnapshotData::new(
                dataset,
                Rc::new(rows),
                format!("delta-r{next_revision}"),
                row_count,
                Some(()),
            )
            .expect("delta fixture data is complete");
            let disposition = state.apply_delta(handle, new_data);
            last_delta_disposition.set(
                match disposition {
                    SnapshotDeltaDisposition::Applied => "applied",
                    SnapshotDeltaDisposition::IgnoredStale => "ignored-stale",
                    SnapshotDeltaDisposition::IgnoredDatasetMismatch => "ignored-dataset-mismatch",
                }
                .to_owned(),
            );
        });
    };
    let delta_own_claim = Callback::new(move |_| {
        // Removes the fixture's own Urgent row -- the caller's own claim.
        apply_delta_removing(Some("office-mx-1".to_owned()));
    });
    let delta_sse_removal = Callback::new(move |_| {
        // Removes a different row -- another user's SSE-delivered removal.
        apply_delta_removing(Some("office-mx-2".to_owned()));
    });
    let delta_replay_stale = Callback::new(move |_| {
        let Some(handle) = stale_delta_handle.get_untracked() else {
            last_delta_disposition.set("no stored handle".to_owned());
            return;
        };
        state.update(|state| {
            let Some(displayed) = state.view(None).displayed() else {
                return;
            };
            let data = SnapshotData::new(
                displayed.dataset().clone(),
                Rc::clone(displayed.rows()),
                "delta-replay-stale".to_owned(),
                displayed.rows().len(),
                Some(()),
            )
            .expect("replay fixture data is complete");
            let disposition = state.apply_delta(handle, data);
            last_delta_disposition.set(
                match disposition {
                    SnapshotDeltaDisposition::Applied => "applied",
                    SnapshotDeltaDisposition::IgnoredStale => "ignored-stale",
                    SnapshotDeltaDisposition::IgnoredDatasetMismatch => "ignored-dataset-mismatch",
                }
                .to_owned(),
            );
        });
    });

    // ldui-baz4: per-action message detail fixtures. Each button drives the
    // typed `start_action_with_content`/`finish_action_with_content` API
    // directly against the same private-field state the table renders from,
    // so the resulting `ActionFeedback` copy is exactly what a real consumer
    // (conflict reason, partial-success count, retryable transport detail)
    // would see.
    let action_conflict = Callback::new(move |_| {
        state.update(|state| {
            if let Ok(handle) = state.start_action("row-1".to_owned()) {
                let _ = state.finish_action_with_content(
                    handle,
                    ActionFeedbackState::RecoverableConflict,
                    ActionFeedbackContent {
                        primary: None,
                        detail: Some(
                            "Another editor changed this record 2 minutes ago.".to_owned(),
                        ),
                    },
                );
            }
        });
    });
    let action_partial = Callback::new(move |_| {
        state.update(|state| {
            if let Ok(handle) = state.start_action("row-1".to_owned()) {
                let _ = state.finish_action_with_content(
                    handle,
                    ActionFeedbackState::PartialSuccess,
                    ActionFeedbackContent {
                        primary: None,
                        detail: Some("3 of 5 items updated.".to_owned()),
                    },
                );
            }
        });
    });
    let action_retryable = Callback::new(move |_| {
        state.update(|state| {
            if let Ok(handle) = state.start_action("row-1".to_owned()) {
                let _ = state.finish_action_with_content(
                    handle,
                    ActionFeedbackState::RetryableFailure,
                    ActionFeedbackContent {
                        primary: None,
                        detail: Some("Timed out contacting the service; retry.".to_owned()),
                    },
                );
            }
        });
    });
    let action_concurrent = Callback::new(move |_| {
        state.update(|state| {
            if let Ok(handle) = state.start_action_with_content(
                "row-1".to_owned(),
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("Saving row 1…".to_owned()),
                },
            ) {
                let _ = state.finish_action_with_content(
                    handle,
                    ActionFeedbackState::Success,
                    ActionFeedbackContent {
                        primary: None,
                        detail: Some("Row 1 saved.".to_owned()),
                    },
                );
            }
            // row-2 is left Pending so both keys are visible at once, proving
            // concurrent keys retain independent content.
            let _ = state.start_action_with_content(
                "row-2".to_owned(),
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("Saving row 2…".to_owned()),
                },
            );
        });
    });
    let action_stale = Callback::new(move |_| {
        state.update(|state| {
            let Ok(stale_handle) = state.start_action_with_content(
                "row-3".to_owned(),
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("First attempt detail — must never display.".to_owned()),
                },
            ) else {
                return;
            };
            let Ok(fresh_handle) = state.start_action_with_content(
                "row-3".to_owned(),
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("Second attempt in progress.".to_owned()),
                },
            ) else {
                return;
            };
            // The superseded handle's completion must be ignored, and its
            // hostile content must never reach the model.
            let _ = state.finish_action_with_content(
                stale_handle,
                ActionFeedbackState::Success,
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("STALE COMPLETION — must never display.".to_owned()),
                },
            );
            let _ = state.finish_action_with_content(
                fresh_handle,
                ActionFeedbackState::RetryableFailure,
                ActionFeedbackContent {
                    primary: None,
                    detail: Some("Timed out contacting the service; retry.".to_owned()),
                },
            );
        });
    });

    view! {
        <SnapshotTablePage
            contract_id="snapshot-page"
            state=state.into()
            local_rows=local_rows.into()
            header=Box::new(|| view! {
                <PageHeader
                    title="Snapshot table fixture"
                    subtitle="Typed identity, retained mounting, and slot-order proof."
                    navigation_layout=PageHeaderNavigationLayout::DedicatedRow
                    navigation_label=Signal::stored("Snapshot navigation".to_owned())
                    back=Box::new(|| view! {
                        <a
                            class="btn btn-ghost btn-sm"
                            href="#snapshot-page-table"
                            data-testid="snapshot-page-back"
                        >
                            "← Back to reports"
                        </a>
                    }.into_any())
                />
            }.into_any())
            dataset_selector=selector
            kpis=Box::new(|| view! {
                <div class="stats shadow-sm" data-testid="snapshot-kpi">
                    <div class="stat py-2">
                        <div class="stat-title">"Rows"</div>
                        <div class="stat-value text-2xl">"3"</div>
                    </div>
                </div>
            }.into_any())
            filters=Box::new(move || view! {
                <div class="flex flex-wrap gap-2" aria-label="Fixture controls">
                    <Button
                        attr:data-testid="snapshot-filter-all"
                        attr:aria-pressed=move || (filter_mode.get() == "all").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("all"))
                    >
                        "All rows"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-filter-urgent"
                        attr:aria-pressed=move || (filter_mode.get() == "urgent").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("urgent"))
                    >
                        "Urgent only"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-filter-none"
                        attr:aria-pressed=move || (filter_mode.get() == "none").to_string()
                        on_click=Callback::new(move |_| filter_mode.set("none"))
                    >
                        "No matches"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-start-replacement"
                        on_click=start_replacement
                    >
                        "Start replacement"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-fail-replacement"
                        on_click=fail_replacement
                    >
                        "Fail replacement"
                    </Button>
                    <Button
                        attr:data-testid="snapshot-complete-replacement"
                        on_click=complete_replacement
                    >
                        "Complete replacement"
                    </Button>
                    <Button attr:data-testid="action-conflict" on_click=action_conflict>
                        "Trigger conflict"
                    </Button>
                    <Button attr:data-testid="action-partial" on_click=action_partial>
                        "Trigger partial success"
                    </Button>
                    <Button attr:data-testid="action-retryable" on_click=action_retryable>
                        "Trigger retryable failure"
                    </Button>
                    <Button attr:data-testid="action-concurrent" on_click=action_concurrent>
                        "Trigger concurrent actions"
                    </Button>
                    <Button attr:data-testid="action-stale" on_click=action_stale>
                        "Trigger stale completion"
                    </Button>
                    <Button attr:data-testid="delta-own-claim" on_click=delta_own_claim>
                        "Apply own-claim delta"
                    </Button>
                    <Button attr:data-testid="delta-sse-removal" on_click=delta_sse_removal>
                        "Apply SSE-removal delta"
                    </Button>
                    <Button attr:data-testid="delta-replay-stale" on_click=delta_replay_stale>
                        "Replay stale delta"
                    </Button>
                    <span data-testid="delta-last-disposition">
                        {move || last_delta_disposition.get()}
                    </span>
                </div>
            }.into_any())
            entity_table=table
            on_retry=retry
            action_key_label=Rc::new(|key: &String| key.clone())
        />
    }
}

fn controls_fixture_rows(dataset: &str) -> Rc<Vec<FixtureRow>> {
    let office = if dataset == "office-in" {
        "New Delhi"
    } else {
        "Mexico City"
    };
    Rc::new(
        (1..=8)
            .map(|index| FixtureRow {
                id: format!("{dataset}-controls-{index}"),
                client: format!("{office} Client {index}"),
                status: if index == 1 { "Urgent" } else { "Ready" }.to_owned(),
            })
            .collect(),
    )
}

fn controls_fixture_snapshot(
    dataset: &str,
    revision: &str,
) -> SnapshotData<FixtureRow, String, ()> {
    let rows = controls_fixture_rows(dataset);
    SnapshotData::new(
        dataset.to_owned(),
        Rc::clone(&rows),
        revision,
        rows.len(),
        Some(()),
    )
    .expect("controls fixture snapshot is complete")
}

/// Focused browser fixture for the `SnapshotEntityTableConfig` behavior-only
/// passthroughs (`ldui-myhh` / `ldui-5ano`): local-filter page reset from a
/// later page (`page_reset_key`), framework-measured adaptive height
/// (`viewport_fit`), the icon column-chooser presentation
/// (`column_chooser_trigger`), and the display-projection callback driving a
/// caller-owned Export action (`with_toolbar_actions` /
/// `on_display_projection`) -- all forwarded through `SnapshotTablePage`'s
/// internally owned `EntityTable` without granting rows, dataset identity,
/// revision, or generation. Eight rows per dataset keep the table on a
/// second page at the short `viewport_fit` budget below, so the filter-reset
/// proof exercises a real later-page starting point instead of an
/// already-page-one table.
#[component]
pub fn SnapshotTablePageControlsFixture() -> impl IntoView {
    type State = SnapshotTableState<FixtureRow, String, String, (), String>;

    let mut initial = State::new();
    let request = initial
        .start_request("office-mx".to_owned())
        .expect("initial controls-fixture request");
    assert_eq!(
        initial.complete(request, controls_fixture_snapshot("office-mx", "mx-r1")),
        SnapshotTransitionDisposition::Applied
    );

    let state = RwSignal::new_local(initial);
    let filter_mode = RwSignal::new("all");
    let local_rows = RwSignal::new_local(Option::<SnapshotLocalRowProjection<FixtureRow>>::None);

    Effect::new(move |_| {
        let mode = filter_mode.get();
        let projection = state.with(|state| {
            let displayed = state.view(None).displayed()?;
            let rows = displayed
                .rows()
                .iter()
                .filter(|row| match mode {
                    "urgent" => row.status == "Urgent",
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            state.local_row_projection(Rc::new(rows))
        });
        local_rows.set(projection);
    });

    // Single-dataset selector: dataset switching itself is proven by
    // `SnapshotTablePageFixture` above, so this fixture keeps the selector
    // minimal and spends its surface on the new behavior-only passthroughs.
    let request_dataset = Callback::new(move |dataset: String| {
        let mut next = None;
        state.update(|state| next = state.start_request(dataset.clone()).ok());
        if let Some(handle) = next {
            state.update(|state| {
                state.complete(handle, controls_fixture_snapshot(&dataset, "mx-r2"));
            });
        }
    });
    let selector = SnapshotDatasetSelectorConfig::new(
        "Office",
        Signal::stored(vec![SnapshotDatasetOption::new(
            "office-mx".to_owned(),
            "Mexico City",
        )]),
        Arc::new(|value: &String| value.clone()),
        request_dataset,
    );

    let display_projection = RwSignal::new_local(Option::<EntityTableDisplayProjection>::None);
    let export_output = RwSignal::new(String::new());
    let export_clicks = RwSignal::new(0_u32);

    let export_rows = Callback::new(move |_| {
        export_clicks.update(|count| *count += 1);
        let text = display_projection.with(|projection| {
            projection
                .as_ref()
                .map(|projection| {
                    projection
                        .rows(EntityTableProjectionScope::AllFiltered)
                        .iter()
                        .map(|row| row.cells.join(","))
                        .collect::<Vec<_>>()
                        .join(";")
                })
                .unwrap_or_default()
        });
        export_output.set(text);
    });

    let page_reset_key = Signal::derive(move || filter_mode.get().to_owned());
    let table = SnapshotEntityTableConfig::new(
        columns(),
        Rc::new(|row: &FixtureRow| row.id.clone()),
        EntityTablePreferenceOwnership::uncontrolled(EntityTablePreferencePersistence::Disabled),
    )
    .with_page_reset_key(page_reset_key)
    // The containing block below supplies the definite height required by
    // `fill_parent`. The table slot must carry that remaining-height budget
    // through to EntityTable instead of sizing itself from the painted rows.
    .with_viewport_fit(EntityTableViewportFit::fill_parent().with_min_rows(2))
    .with_toolbar_actions(move || {
        view! {
            <Button attr:data-testid="controls-export" on_click=export_rows>
                "Export"
            </Button>
        }
        .into_any()
    })
    .on_display_projection(Callback::new(
        move |projection: EntityTableDisplayProjection| {
            display_projection.set(Some(projection));
        },
    ))
    .with_column_chooser_trigger(EntityColumnChooserTrigger::Icon);

    view! {
        <section id="snapshot-controls-fixture" class="space-y-3">
            <div class="h-[520px] min-h-0" data-testid="controls-height-budget">
                <SnapshotTablePage
                    contract_id="snapshot-controls"
                    state=state.into()
                    local_rows=local_rows.into()
                    header=Box::new(|| view! {
                        <PageHeader
                            title="Snapshot table controls fixture"
                            subtitle="Behavior-only EntityTable passthroughs: page reset, viewport fit, toolbar export, icon chooser."
                        />
                    }.into_any())
                    dataset_selector=selector
                    filters=Box::new(move || view! {
                        <div class="flex flex-wrap gap-2" aria-label="Controls fixture filters">
                            <Button
                                attr:data-testid="controls-filter-all"
                                attr:aria-pressed=move || (filter_mode.get() == "all").to_string()
                                on_click=Callback::new(move |_| filter_mode.set("all"))
                            >
                                "All rows"
                            </Button>
                            <Button
                                attr:data-testid="controls-filter-urgent"
                                attr:aria-pressed=move || (filter_mode.get() == "urgent").to_string()
                                on_click=Callback::new(move |_| filter_mode.set("urgent"))
                            >
                                "Urgent only"
                            </Button>
                        </div>
                    }.into_any())
                    entity_table=table
                    action_key_label=Rc::new(|key: &String| key.clone())
                    class="h-full"
                />
            </div>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Export clicks: "
                    <code data-testid="controls-export-clicks">
                        {move || export_clicks.get().to_string()}
                    </code>
                </span>
            </div>
            <pre data-testid="controls-export-output" class="text-xs">{move || export_output.get()}</pre>
        </section>
    }
}

/// Spanish copy for the framework utility row, proving the count template
/// and both action labels are localizable through one `FilterBarTexts`.
fn spanish_filter_bar_texts() -> FilterBarTexts {
    FilterBarTexts {
        region_label: "Filtros".to_owned(),
        active_none: "Sin filtros activos".to_owned(),
        active_one: "1 filtro activo".to_owned(),
        active_many: "{count} filtros activos".to_owned(),
        remove_filter: "Quitar el filtro {label}".to_owned(),
        result_count: "{visible} de {total} resultados".to_owned(),
        reset: "Restablecer".to_owned(),
        save_default: "Guardar como predeterminado".to_owned(),
        clean_reason: "Los valores predeterminados ya están guardados".to_owned(),
        pending_reason: "Se está guardando la vista predeterminada".to_owned(),
        pending_feedback: "Guardando la vista predeterminada".to_owned(),
        saved_feedback: "Vista predeterminada guardada".to_owned(),
        conflict_feedback: "Conflicto de vista predeterminada: {message}".to_owned(),
        failure_feedback: "No se pudo guardar la vista predeterminada: {message}".to_owned(),
    }
}

/// Focused browser fixture for the opt-in framework utility row
/// (`ldui-nj3q`): the visible/total result count, one Reset, and one explicit
/// Save as Default obtained from `SnapshotTablePage` itself rather than from
/// a consumer-composed `FilterBar`.
///
/// Renders BOTH configurations on one page on purpose. `#snapshot-actions`
/// opts in; `#snapshot-plain` does not and must keep rendering exactly as it
/// does today -- no filter bar, no count, no Reset, no Save as Default. The
/// absent case is the negative control for every assertion about the
/// opted-in case. Only after those same-state assertions does the fixture
/// transition `#snapshot-plain` to an authoritative empty snapshot, proving
/// the reachable no-local-projection footer path for `ldui-r50n`.
#[component]
pub fn SnapshotTablePageFilterActionsFixture() -> impl IntoView {
    type State = SnapshotTableState<FixtureRow, String, String, (), String>;

    fn seeded_state() -> State {
        let mut initial = State::new();
        let request = initial
            .start_request("office-mx".to_owned())
            .expect("initial filter-actions request");
        assert_eq!(
            initial.complete(request, snapshot("office-mx", "mx-r1")),
            SnapshotTransitionDisposition::Applied
        );
        initial
    }

    fn empty_state() -> State {
        let mut initial = State::new();
        let request = initial
            .start_request("office-mx".to_owned())
            .expect("initial empty filter-actions request");
        let empty = SnapshotData::new(
            "office-mx".to_owned(),
            Rc::new(Vec::<FixtureRow>::new()),
            "mx-empty",
            0,
            Some(()),
        )
        .expect("empty fixture snapshot is complete");
        assert_eq!(
            initial.complete(request, empty),
            SnapshotTransitionDisposition::Applied
        );
        initial
    }

    fn selector_config() -> SnapshotDatasetSelectorConfig<String> {
        SnapshotDatasetSelectorConfig::new(
            "Office",
            Signal::stored(vec![SnapshotDatasetOption::new(
                "office-mx".to_owned(),
                "Mexico City",
            )]),
            Arc::new(|value: &String| value.clone()),
            Callback::new(|_: String| {}),
        )
    }

    fn table_config() -> SnapshotEntityTableConfig<FixtureRow> {
        SnapshotEntityTableConfig::new(
            columns(),
            Rc::new(|row: &FixtureRow| row.id.clone()),
            EntityTablePreferenceOwnership::uncontrolled(
                EntityTablePreferencePersistence::Disabled,
            ),
        )
    }

    let state = RwSignal::new_local(seeded_state());
    let plain_state = RwSignal::new_local(seeded_state());
    let filter_mode = RwSignal::new("all");
    let local_rows = RwSignal::new_local(Option::<SnapshotLocalRowProjection<FixtureRow>>::None);

    Effect::new(move |_| {
        let mode = filter_mode.get();
        let projection = state.with(|state| {
            let displayed = state.view(None).displayed()?;
            let rows = displayed
                .rows()
                .iter()
                .filter(|row| match mode {
                    "urgent" => row.status == "Urgent",
                    "none" => false,
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            state.local_row_projection(Rc::new(rows))
        });
        local_rows.set(projection);
    });

    let spanish = RwSignal::new(false);
    let texts = Signal::derive(move || {
        if spanish.get() {
            spanish_filter_bar_texts()
        } else {
            FilterBarTexts::default()
        }
    });
    let empty_row_range = Signal::derive(move || {
        if spanish.get() {
            "Filas de actividad: 0 de {total}".to_owned()
        } else {
            "Activity rows 0 of {total}".to_owned()
        }
    });

    let reset_clicks = RwSignal::new(0_u32);
    let on_reset = Callback::new(move |()| {
        reset_clicks.update(|count| *count += 1);
        filter_mode.set("all");
    });

    let chips = Signal::derive(move || match filter_mode.get() {
        "urgent" => vec![ActiveFilterChip::new("status", "Status", "Urgent")],
        "none" => vec![ActiveFilterChip::new("status", "Status", "No matches")],
        _ => Vec::new(),
    });
    let on_remove = Callback::new(move |_key: String| filter_mode.set("all"));

    let save_state = RwSignal::new(SnapshotDefaultSaveState::Clean);
    let saved_filter_value = RwSignal::new(String::from("(none)"));
    let defaults = Signal::derive(move || {
        FilterSchema::<()>::new("office", &["status"])
            .project_defaults(
                [(
                    "status",
                    serde_json::Value::String(filter_mode.get().to_owned()),
                )],
                EntityTablePreferences::new(1),
            )
            .expect("fixture defaults project through the declared schema")
    });
    let on_save = Callback::new(move |payload: SnapshotViewDefaults| {
        saved_filter_value.set(
            payload
                .filters()
                .get("status")
                .map_or_else(|| "(none)".to_owned(), ToString::to_string),
        );
        save_state.set(SnapshotDefaultSaveState::Saved);
    });

    let filter_actions = SnapshotFilterActionsConfig::new()
        .with_texts(texts)
        .on_reset(on_reset)
        .with_active_filters(chips, on_remove)
        .with_default_save(SnapshotDefaultSave::new(defaults, save_state, on_save));

    let mode_buttons = move || {
        view! {
            <div class="flex flex-wrap gap-2" aria-label="Filter-actions fixture controls">
                <Button
                    attr:data-testid="actions-filter-all"
                    attr:aria-pressed=move || (filter_mode.get() == "all").to_string()
                    on_click=Callback::new(move |_| filter_mode.set("all"))
                >
                    "All rows"
                </Button>
                <Button
                    attr:data-testid="actions-filter-urgent"
                    attr:aria-pressed=move || (filter_mode.get() == "urgent").to_string()
                    on_click=Callback::new(move |_| filter_mode.set("urgent"))
                >
                    "Urgent only"
                </Button>
            </div>
        }
    };

    view! {
        <section id="snapshot-filter-actions-fixture" class="space-y-6">
            <div class="flex flex-wrap gap-2" aria-label="Filter-actions fixture harness">
                <Button
                    attr:data-testid="actions-locale-es"
                    on_click=Callback::new(move |_| spanish.set(true))
                >
                    "Español"
                </Button>
                <Button
                    attr:data-testid="actions-locale-en"
                    on_click=Callback::new(move |_| spanish.set(false))
                >
                    "English"
                </Button>
                <Button
                    attr:data-testid="plain-empty-snapshot"
                    on_click=Callback::new(move |_| plain_state.set(empty_state()))
                >
                    "Empty plain snapshot"
                </Button>
                <Button
                    attr:data-testid="actions-save-dirty"
                    on_click=Callback::new(move |_| {
                        save_state.set(SnapshotDefaultSaveState::Dirty);
                    })
                >
                    "Mark view dirty"
                </Button>
                <Button
                    attr:data-testid="actions-save-conflict"
                    on_click=Callback::new(move |_| {
                        save_state.set(SnapshotDefaultSaveState::Conflict(
                            "A newer default exists.".to_owned(),
                        ));
                    })
                >
                    "Force save conflict"
                </Button>
                <span>
                    "Reset clicks: "
                    <code data-testid="actions-reset-clicks">
                        {move || reset_clicks.get().to_string()}
                    </code>
                </span>
                <span>
                    "Saved status: "
                    <code data-testid="actions-saved-filter">
                        {move || saved_filter_value.get()}
                    </code>
                </span>
            </div>

            <SnapshotTablePage
                contract_id="snapshot-actions"
                state=state.into()
                local_rows=local_rows.into()
                header=Box::new(|| view! {
                    <PageHeader
                        title="Snapshot table filter actions"
                        subtitle="Framework-owned result count, Reset, and Save as Default."
                    />
                }.into_any())
                dataset_selector=selector_config()
                filters=Box::new(move || mode_buttons().into_any())
                filter_actions=filter_actions
                entity_table=table_config()
                action_key_label=Rc::new(|key: &String| key.clone())
            />

            <SnapshotTablePage
                contract_id="snapshot-plain"
                state=plain_state.into()
                header=Box::new(|| view! {
                    <PageHeader
                        title="Snapshot table without filter actions"
                        subtitle="Negative control: the same composite, no opt-in."
                    />
                }.into_any())
                dataset_selector=selector_config()
                filters=Box::new(|| view! {
                    <div class="flex flex-wrap gap-2" aria-label="Plain fixture controls">
                        <Button attr:data-testid="plain-filter-all">"All rows"</Button>
                    </div>
                }.into_any())
                entity_table=table_config().with_empty_row_range(empty_row_range)
                action_key_label=Rc::new(|key: &String| key.clone())
            />
        </section>
    }
}

/// Focused browser fixture for `EntityTable` inline draft-row editing
/// (`ldui-ff2f`).
///
/// Mounts BOTH configurations on one document, the pattern `ldui-nj3q` used:
/// `#draft-optin` opts in, `#draft-plain` does not. The claim that an
/// un-opted table renders unchanged is then proven on the same run rather
/// than asserted — a negative control for every positive assertion.
///
/// The consumer's side of the contract is visible too: Save hands over a row
/// and a `resolve` handle, and this fixture deliberately does NOT resolve it
/// automatically. The Accept / Reject buttons stand in for a real async
/// write, so a test can observe the in-flight `Committing` state that a
/// synchronous fixture would skip straight past.
#[component]
pub fn EntityTableDraftRowFixture() -> impl IntoView {
    let data = RwSignal::new_local(draft_fixture_rows());
    let source_data = RwSignal::new_local(draft_fixture_rows());
    let dataset_identity = RwSignal::new(String::from("draft-optin"));
    let page_reset_key = RwSignal::new(String::from("draft-page-0"));
    let focus_scope = RwSignal::new(String::from("draft-scope-0"));
    let refresh_generation = RwSignal::new(0_u8);
    let plain_data = RwSignal::new_local(rows("office-mx"));
    let commits = RwSignal::new(0_u32);
    let last_committed = RwSignal::new(String::from("(none)"));
    let last_target = RwSignal::new(String::from("(none)"));
    let retired_rows = RwSignal::new(Vec::<String>::new());
    let filter_value = RwSignal::new(String::new());
    let filter_proposals = RwSignal::new(0_u32);
    let toolbar_clicks = RwSignal::new(0_u32);
    let row_activations = RwSignal::new(0_u32);
    let selection_proposals = RwSignal::new(0_u32);
    let selected_key = RwSignal::new(Option::<String>::None);
    let pending_resolve = RwSignal::new(Option::<Callback<EntityEditOutcome>>::None);

    let apply_refresh = move |generation: u8| {
        refresh_generation.set(generation);
        let next = Rc::new(vec![FixtureRow {
            id: format!("refresh-{generation}"),
            client: format!("Refresh {generation}"),
            status: format!("Generation {generation}"),
        }]);
        data.set(Rc::clone(&next));
        source_data.set(next);
        dataset_identity.set(format!("draft-optin-{generation}"));
        page_reset_key.set(format!("refresh-{generation}"));
        focus_scope.set(format!("draft-scope-{generation}"));
    };

    let editable_columns = move || {
        vec![
            EntityColumn::text("client", "Client", |row: &FixtureRow| row.client.clone())
                .required()
                .with_min_width(220)
                .editable(EntityCellEditor::text(
                    |row: &FixtureRow| row.client.clone(),
                    |row: &mut FixtureRow, value| row.client = value,
                )),
            // Deliberately NOT editable: proves a derived column keeps
            // rendering read-only text even inside the live row.
            EntityColumn::text("id", "Id", |row: &FixtureRow| row.id.clone()).with_min_width(140),
            EntityColumn::text("status", "Status", |row: &FixtureRow| row.status.clone())
                .with_min_width(120)
                .editable(EntityCellEditor::select(
                    [
                        "Ready",
                        "Urgent",
                        "Reviewed",
                        "Generation 1",
                        "Generation 2",
                    ]
                    .into_iter()
                    .map(|value| {
                        leptos_daisyui_rs::components::EntityCellSelectOption::new(value, value)
                    })
                    .collect(),
                    |row: &FixtureRow| row.status.clone(),
                    |row: &mut FixtureRow, value| row.status = value,
                )),
            EntityColumn::action("actions", "Actions", |_row: &FixtureRow| {
                "Retire".to_owned()
            })
            .render_with(move |row: &FixtureRow| {
                let key = row.id.clone();
                let retire_key = key.clone();
                view! {
                    <EntityRowAction action_id="retire">
                        <Button
                            class="btn-xs btn-ghost"
                            attr:data-fixture-retire=key
                            on_click=Callback::new(move |_| {
                                retired_rows.update(|rows| rows.push(retire_key.clone()));
                            })
                        >
                            "Retire"
                        </Button>
                    </EntityRowAction>
                }
                .into_any()
            })
            .inline_edit_host(),
        ]
    };

    let on_commit = Callback::new(move |commit: EntityDraftCommit<FixtureRow>| {
        commits.update(|count| *count += 1);
        last_committed.set(format!("{}|{}", commit.row.client, commit.row.status));
        last_target.set(match &commit.target {
            leptos_daisyui_rs::components::EntityEditTarget::Draft => "draft".to_owned(),
            leptos_daisyui_rs::components::EntityEditTarget::Existing(key) => {
                format!("existing:{key}")
            }
        });
        // Held, not resolved: the table stays in flight until a button below
        // answers, which is the whole point of the resolve handle.
        pending_resolve.set(Some(commit.resolve));
    });
    let draft_filters = vec![EntityColumnFilter::text(
        "status",
        "draft-status-filter",
        "Filter status",
        filter_value,
        "Filter status",
        Callback::new(move |_proposal: String| {
            filter_proposals.update(|count| *count += 1);
        }),
    )];

    view! {
        <section id="draft-row-fixture" class="mx-auto max-w-4xl space-y-6 bg-base-100 p-4">
            <h1 class="ld-text-display font-semibold">"Inline draft-row editing"</h1>

            <div class="flex flex-wrap items-center gap-2">
                <Button
                    attr:data-testid="draft-refresh-1"
                    on_click=Callback::new(move |_| apply_refresh(1))
                >
                    "Refresh 1"
                </Button>
                <Button
                    attr:data-testid="draft-refresh-2"
                    on_click=Callback::new(move |_| apply_refresh(2))
                >
                    "Refresh 2"
                </Button>
                <Button
                    attr:data-testid="draft-accept"
                    on_click=Callback::new(move |_| {
                        if let Some(resolve) = pending_resolve.get_untracked() {
                            resolve.run(EntityEditOutcome::Accepted);
                            pending_resolve.set(None);
                        }
                    })
                >
                    "Accept commit"
                </Button>
                <Button
                    attr:data-testid="draft-reject"
                    on_click=Callback::new(move |_| {
                        if let Some(resolve) = pending_resolve.get_untracked() {
                            resolve.run(EntityEditOutcome::Rejected("Name taken".to_owned()));
                            pending_resolve.set(None);
                        }
                    })
                >
                    "Reject commit"
                </Button>
                <span>
                    "Commits: "
                    <code data-testid="draft-commit-count">
                        {move || commits.get().to_string()}
                    </code>
                </span>
                <span>
                    "Last: "
                    <code data-testid="draft-last-committed">{move || last_committed.get()}</code>
                </span>
                <span>
                    "Target: "
                    <code data-testid="draft-last-target">{move || last_target.get()}</code>
                </span>
                <span>
                    "Retired: "
                    <code data-testid="draft-retire-count">
                        {move || retired_rows.with(|rows| rows.len()).to_string()}
                    </code>
                </span>
                <span>
                    "Filter proposals: "
                    <code data-testid="draft-filter-proposals">
                        {move || filter_proposals.get().to_string()}
                    </code>
                </span>
                <span>
                    "Toolbar clicks: "
                    <code data-testid="draft-toolbar-clicks">
                        {move || toolbar_clicks.get().to_string()}
                    </code>
                </span>
                <span>
                    "Row activations: "
                    <code data-testid="draft-row-activations">
                        {move || row_activations.get().to_string()}
                    </code>
                </span>
                <span>
                    "Selection proposals: "
                    <code data-testid="draft-selection-proposals">
                        {move || selection_proposals.get().to_string()}
                    </code>
                </span>
            </div>

            <div data-testid="draft-optin-table" id="draft-optin">
                <EntityTable
                    data=data
                    source_data=source_data.into()
                    columns=editable_columns()
                    column_filters=draft_filters
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity=dataset_identity
                    page_reset_key=page_reset_key
                    focus_scope=focus_scope
                    toolbar_actions=Box::new(move || view! {
                        <Button
                            class="btn-sm btn-ghost"
                            attr:data-testid="draft-toolbar-action"
                            on_click=Callback::new(move |_| {
                                toolbar_clicks.update(|count| *count += 1);
                            })
                        >
                            "Archive"
                        </Button>
                    }.into_any())
                    on_row_activate=Callback::new(move |_key: String| {
                        row_activations.update(|count| *count += 1);
                    })
                    selection=EntityTableSelection::controlled(
                        selected_key.into(),
                        Callback::new(move |_key: Option<String>| {
                            selection_proposals.update(|count| *count += 1);
                        }),
                    )
                    draft_row=EntityDraftRow::new(
                        || FixtureRow {
                            id: "draft-new".to_owned(),
                            client: String::new(),
                            status: "Ready".to_owned(),
                        },
                        on_commit,
                    )
                    .allow_row_edit(true)
                />
            </div>

            // Negative control: same columns, no draft_row. Must render no `+`,
            // no draft row, and no data-entity-edit-phase at all.
            <div data-testid="draft-plain-table" id="draft-plain">
                <EntityTable
                    data=plain_data
                    columns=editable_columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="draft-plain"
                />
            </div>
        </section>
    }
}

/// Real-WASM fixture for framework-owned EntityTable viewport-fit paging.
#[component]
pub fn EntityTableViewportFitFixture() -> impl IntoView {
    let height = RwSignal::new(520_u32);
    let alternate_copy = RwSignal::new(false);
    // ldui-5p06's headline story: 17 rows is the count that used to render
    // five rows under a control reading `25`, advertising four pages.
    let resolved_page_size = RwSignal::new(Option::<EntityPageSize>::None);
    // Held as a stable `Rc` rather than rebuilt per read: the table's
    // sorted-index cache keys on row identity, so a fresh allocation on every
    // `get` would thrash it and re-key every rendered row.
    let build_rows = |count: usize| {
        Rc::new(
            (1..=count)
                .map(|index| FixtureRow {
                    id: format!("fit-{index:02}"),
                    client: format!("Viewport client {index:02}"),
                    status: if index % 3 == 0 { "Urgent" } else { "Ready" }.to_owned(),
                })
                .collect::<Vec<_>>(),
        )
    };
    let rows = RwSignal::new_local(build_rows(60));
    let data: Signal<Rc<Vec<FixtureRow>>, LocalStorage> = rows.into();
    let columns = Signal::derive_local(move || {
        vec![
            EntityColumn::text(
                "client",
                if alternate_copy.get() {
                    "Client account with localized heading"
                } else {
                    "Client"
                },
                |row: &FixtureRow| row.client.clone(),
            )
            .required()
            .with_min_width(220),
            EntityColumn::text(
                "status",
                if alternate_copy.get() {
                    "Localized workflow status"
                } else {
                    "Status"
                },
                |row: &FixtureRow| row.status.clone(),
            )
            .with_min_width(150),
        ]
    });
    let texts = Signal::derive(move || EntityTableTexts {
        rows_per_page: if alternate_copy.get() {
            "Rows per page used when the viewport is too short".to_owned()
        } else {
            "Rows per page".to_owned()
        },
        ..EntityTableTexts::default()
    });
    let filters = vec![EntityColumnFilter::new("status", || {
        view! {
            <span class="block min-h-8 py-1 text-xs" data-testid="viewport-fit-filter-row">
                "Controlled status filter"
            </span>
        }
        .into_any()
    })];

    view! {
        <section id="entity-viewport-fit-fixture" class="space-y-3">
            <div class="flex flex-wrap gap-2" aria-label="Viewport fit controls">
                <Button attr:data-testid="viewport-fit-default" on_click=Callback::new(move |_| height.set(520))>
                    "Default height"
                </Button>
                <Button attr:data-testid="viewport-fit-tall" on_click=Callback::new(move |_| height.set(800))>
                    "Tall height"
                </Button>
                <Button attr:data-testid="viewport-fit-short" on_click=Callback::new(move |_| height.set(180))>
                    "Short height"
                </Button>
                <Button attr:data-testid="viewport-fit-locale" on_click=Callback::new(move |_| alternate_copy.update(|value| *value = !*value))>
                    "Toggle localized copy"
                </Button>
                <Button attr:data-testid="viewport-fit-rows-17" on_click=Callback::new(move |_| rows.set(build_rows(17)))>
                    "17 rows"
                </Button>
                <Button attr:data-testid="viewport-fit-rows-60" on_click=Callback::new(move |_| rows.set(build_rows(60)))>
                    "60 rows"
                </Button>
            </div>
            <p class="text-sm text-base-content/75">
                "Resolved page size: "
                <code data-testid="viewport-fit-resolved">
                    {move || match resolved_page_size.get() {
                        Some(page_size) if page_size.is_auto() => {
                            format!("auto:{}", page_size.rows())
                        }
                        Some(page_size) => format!("fixed:{}", page_size.rows()),
                        None => "none".to_owned(),
                    }}
                </code>
            </p>
            <div
                class="min-h-0 w-full"
                style=move || format!("height: {}px", height.get())
                data-testid="viewport-fit-budget"
            >
                <EntityTable
                    data=data
                    columns=columns
                    column_filters=filters
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="viewport-fit-fixture"
                    viewport_fit=EntityTableViewportFit::fill_parent().with_min_rows(3)
                    preference_ownership=EntityTablePreferenceOwnership::uncontrolled(
                        EntityTablePreferencePersistence::Disabled,
                    )
                    texts=texts
                    page_size_control_id="viewport-fit-page-size"
                    on_page_size_resolved=Callback::new(move |page_size| {
                        resolved_page_size.set(Some(page_size));
                    })
                />
            </div>
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTablePresentationRow {
    id: String,
    reference: String,
    narrative: String,
    rich: String,
    number: i64,
    date_key: (u16, u8, u8),
    optional_rank: Option<i64>,
    currency: String,
    percentage: String,
    status: String,
    icon_label: String,
    contact_name: String,
    contact_role: Option<String>,
}

/// Builds the `contact` column's `primary_secondary` presentation.
/// `localized` reconstructs both the header and the closures that produce
/// the primary/secondary lines, so replacing the whole `columns` list (not
/// just row data) with a fresh call is what changes the rendered text.
fn presentation_contact_column(localized: bool) -> EntityColumn<EntityTablePresentationRow> {
    let header = if localized {
        "Principal y secundario"
    } else {
        "Primary and secondary"
    };
    let role_label = if localized { "Rol" } else { "Role" };
    EntityColumn::text(
        "contact",
        header,
        move |row: &EntityTablePresentationRow| match row
            .contact_role
            .as_deref()
            .map(str::trim)
            .filter(|role| !role.is_empty())
        {
            Some(role) => format!("{} ({role_label}: {role})", row.contact_name),
            None => row.contact_name.clone(),
        },
    )
    .primary_secondary(
        |row: &EntityTablePresentationRow| row.contact_name.clone(),
        move |row: &EntityTablePresentationRow| {
            row.contact_role
                .as_deref()
                .map(str::trim)
                .filter(|role| !role.is_empty())
                .map(|role| format!("{role_label}: {role}"))
        },
    )
    .with_min_width(140)
    .with_width(170)
}

/// Builds every column for [`EntityTablePresentationFixture`]. Only
/// `contact` (see [`presentation_contact_column`]) varies with `localized`;
/// the rest are rebuilt identically on every call, matching how
/// `client_snapshot_list.rs`'s `demo_columns` reconstructs its full column
/// list from one locale flag.
fn presentation_columns(localized: bool) -> Vec<EntityColumn<EntityTablePresentationRow>> {
    vec![
        EntityColumn::text(
            "reference",
            "Reference",
            |row: &EntityTablePresentationRow| row.reference.clone(),
        )
        .identifier()
        .ellipsis()
        .with_min_width(110)
        .with_width(150),
        EntityColumn::text(
            "narrative",
            "Narrative",
            |row: &EntityTablePresentationRow| row.narrative.clone(),
        )
        .line_clamp(2)
        .with_min_width(150)
        .with_width(220),
        EntityColumn::text(
            "rich",
            "Rich precedence",
            |row: &EntityTablePresentationRow| row.rich.clone(),
        )
        .line_clamp(2)
        .with_width(180)
        .numeric()
        .render_with(|row: &EntityTablePresentationRow| {
            let text = row.rich.clone();
            view! { <em data-entity-presentation-rich="true">{text}</em> }.into_any()
        }),
        EntityColumn::text(
            "number",
            "Typed number",
            |row: &EntityTablePresentationRow| row.number.to_string(),
        )
        .sortable_by_key(|row| row.number)
        .numeric()
        .with_width(130),
        EntityColumn::text("date", "Typed date", |row: &EntityTablePresentationRow| {
            format!(
                "{:04}-{:02}-{:02}",
                row.date_key.0, row.date_key.1, row.date_key.2
            )
        })
        .sortable_by_key(|row| row.date_key)
        .align_center()
        .tabular_numbers()
        .with_width(135),
        EntityColumn::text(
            "optional",
            "Optional rank",
            |row: &EntityTablePresentationRow| {
                row.optional_rank
                    .map_or_else(|| "Not ranked".to_owned(), |rank| rank.to_string())
            },
        )
        .sortable_by_optional_key(EntityNullOrder::Last, |row| row.optional_rank)
        .numeric()
        .with_width(140),
        EntityColumn::text(
            "currency",
            "Currency-like",
            |row: &EntityTablePresentationRow| row.currency.clone(),
        )
        .numeric()
        .ellipsis()
        .with_width(165),
        EntityColumn::text(
            "percentage",
            "Percentage-like",
            |row: &EntityTablePresentationRow| row.percentage.clone(),
        )
        .numeric()
        .with_width(145),
        EntityColumn::text(
            "status_badge",
            "Semantic badge",
            |row: &EntityTablePresentationRow| row.status.clone(),
        )
        .badge_with(|row| match row.id.as_str() {
            "presentation-1" => Some(EntityBadgePresentation::new(BadgeColor::Warning)),
            "presentation-2" => Some(EntityBadgePresentation::new(BadgeColor::Success)),
            "presentation-3" => None,
            _ => Some(EntityBadgePresentation::new(BadgeColor::Info)),
        })
        .with_width(155),
        EntityColumn::text(
            "state_icon",
            "Semantic icon",
            |row: &EntityTablePresentationRow| row.icon_label.clone(),
        )
        .align_center()
        .icon_with(|row| match row.id.as_str() {
            "presentation-1" => Some(EntityIconPresentation::new(
                "circle-check",
                EntityIconColor::Success,
            )),
            "presentation-2" => Some(EntityIconPresentation::new(
                "triangle-alert",
                EntityIconColor::Warning,
            )),
            "presentation-3" => None,
            _ => Some(EntityIconPresentation::new("check", EntityIconColor::Info)),
        })
        .with_width(145),
        presentation_contact_column(localized),
    ]
}

/// Focused browser fixture for framework-owned EntityColumn presentation.
#[component]
pub fn EntityTablePresentationFixture() -> impl IntoView {
    let data = RwSignal::new_local(Rc::new(vec![
        EntityTablePresentationRow {
            id: "presentation-1".to_owned(),
            reference: "REFERENCE-WITH-ONE-UNBROKEN-VALUE-0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .to_owned(),
            narrative: "This canonical narrative is intentionally long enough to occupy more than two visual lines while remaining complete in the DOM for assistive technology and export."
                .to_owned(),
            rich: "Canonical rich-renderer fallback text".to_owned(),
            number: 10,
            date_key: (2026, 1, 2),
            optional_rank: None,
            currency: "-$12,345,678,901.25".to_owned(),
            percentage: "100.00%".to_owned(),
            status: "Needs review".to_owned(),
            icon_label: "Enabled".to_owned(),
            contact_name: "Jordan Blake".to_owned(),
            contact_role: Some("Team lead".to_owned()),
        },
        EntityTablePresentationRow {
            id: "presentation-2".to_owned(),
            reference: "SHORT-2".to_owned(),
            narrative: "A short narrative.".to_owned(),
            rich: "Second rich value".to_owned(),
            number: 2,
            date_key: (2025, 12, 31),
            optional_rank: Some(10),
            currency: "$24.00".to_owned(),
            percentage: "20.50%".to_owned(),
            status: "Ready".to_owned(),
            icon_label: "Attention".to_owned(),
            contact_name: "Sam Rivera".to_owned(),
            contact_role: Some(String::new()),
        },
        EntityTablePresentationRow {
            id: "presentation-3".to_owned(),
            reference: "NEGATIVE-3".to_owned(),
            narrative: "Signed numeric ordering fixture.".to_owned(),
            rich: "Third rich value".to_owned(),
            number: -3,
            date_key: (2026, 1, 1),
            optional_rank: Some(2),
            currency: "-$3.00".to_owned(),
            percentage: "-3.00%".to_owned(),
            status: "Unknown status".to_owned(),
            icon_label: "Unknown state".to_owned(),
            contact_name: "Alex Chen".to_owned(),
            contact_role: None,
        },
        EntityTablePresentationRow {
            id: "presentation-4".to_owned(),
            reference: "SECOND-TWO".to_owned(),
            narrative: "Stable equal-key ordering fixture.".to_owned(),
            rich: "Fourth rich value".to_owned(),
            number: 2,
            date_key: (2026, 2, 1),
            optional_rank: Some(10),
            currency: "$2.00".to_owned(),
            percentage: "2.00%".to_owned(),
            status: String::new(),
            icon_label: String::new(),
            contact_name: "Morgan Lee".to_owned(),
            contact_role: Some("Reviewer".to_owned()),
        },
    ]));
    let semantic_localized = RwSignal::new(false);
    let toggle_semantic_locale = Callback::new(move |_| {
        let localized = !semantic_localized.get_untracked();
        semantic_localized.set(localized);
        data.update(|rows| {
            let mut replacement = rows.as_ref().clone();
            replacement[0].status = if localized {
                "Revisión necesaria"
            } else {
                "Needs review"
            }
            .to_owned();
            replacement[0].icon_label = if localized { "Habilitado" } else { "Enabled" }.to_owned();
            *rows = Rc::new(replacement);
        });
    });
    // Distinct from `semantic_localized` above: this toggles the `columns`
    // *Signal itself* -- an entirely new `Vec<EntityColumn<_>>` with new
    // primary/secondary closures for the `contact` column -- rather than
    // mutating row data through the existing static columns. Proves
    // EntityColumn::primary_secondary re-renders both lines when the caller
    // replaces the columns list reactively (ldui-97v), the same reactive
    // primitive `client_snapshot_list.rs`'s locale toggle exercises for
    // plain-text columns.
    let column_locale = RwSignal::new(false);
    let toggle_column_locale = Callback::new(move |_| {
        column_locale.update(|localized| *localized = !*localized);
    });
    let columns = Signal::derive_local(move || presentation_columns(column_locale.get()));

    view! {
        <section
            id="entity-table-presentation-fixture"
            class="mx-auto max-w-3xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Entity column presentation"</h1>
            <p class="text-sm text-base-content/70">
                "Plain canonical values demonstrate ellipsis and two-line clipping; the rich column proves render_with precedence; the contact column demonstrates the opinionated primary/secondary presentation, including a row with no secondary line and a row with an empty one."
            </p>
            <div class="flex flex-wrap gap-2">
                <Button
                    attr:data-testid="entity-presentation-locale"
                    on_click=toggle_semantic_locale
                >
                    "Toggle semantic cell locale"
                </Button>
                <Button
                    attr:data-testid="entity-presentation-column-locale"
                    on_click=toggle_column_locale
                >
                    "Toggle column-level locale"
                </Button>
            </div>
            <EntityTable
                data=data
                columns=columns
                row_key=Rc::new(|row: &EntityTablePresentationRow| row.id.clone())
                dataset_identity=Signal::stored("presentation-fixture".to_owned())
            />
        </section>
    }
}

/// Focused browser fixture for the framework-owned page-size select identity
/// (ldui-kl55). Table A and table B both omit `page_size_control_id` — the
/// bug was that this left the rows-per-page `<select>` with no `id`/`name` at
/// all, and Office satellites mounting several `EntityTable`s on one Setup
/// page had no way to tell the controls apart. Table C supplies an explicit
/// override, which must be honored verbatim rather than replaced by a
/// generated default.
#[component]
pub fn EntityTablePageSizeIdentityFixture() -> impl IntoView {
    let data_a = RwSignal::new_local(rows("office-mx"));
    let data_b = RwSignal::new_local(rows("office-in"));
    let data_c = RwSignal::new_local(rows("office-mx"));

    view! {
        <section
            id="entity-table-page-size-identity-fixture"
            class="mx-auto max-w-3xl space-y-6 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Page-size select identity"</h1>
            <div data-testid="page-size-identity-table-a">
                <EntityTable
                    data=data_a
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-a"
                />
            </div>
            <div data-testid="page-size-identity-table-b">
                <EntityTable
                    data=data_b
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-b"
                />
            </div>
            <div data-testid="page-size-identity-table-c">
                <EntityTable
                    data=data_c
                    columns=columns()
                    row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                    dataset_identity="page-size-identity-c"
                    page_size_control_id="page-size-identity-explicit-override"
                />
            </div>
        </section>
    }
}

/// Focused browser fixture for `EntityTable` controlled single-row selection
/// (ldui-sh3), mirroring `ServerTableSelection`'s cursor-table fixture. The
/// detail panel is a master-detail readout driven purely by the caller's
/// accepted `selected_key` -- proving the same signal that paints
/// `aria-selected`/styling also identifies "which row" for a consumer.
/// `entity-selection-remove-selected` exercises the fail-safe when the
/// accepted key stops matching any row in `data`: no crash, and no row
/// renders selected until the caller supplies a key that matches again.
/// The empty/restore controls exercise the scroll-region keyboard stop when
/// an interactive table temporarily has no displayed rows (`ldui-qsia`).
#[component]
pub fn EntityTableSelectionFixture() -> impl IntoView {
    let data = RwSignal::new_local(rows("office-mx"));
    let selected_key = RwSignal::new(Option::<String>::None);
    let accept_proposals = RwSignal::new(true);
    let proposal_count = RwSignal::new(0_u32);
    let last_proposal = RwSignal::new(Option::<String>::None);
    let activation_count = RwSignal::new(0_u32);

    let remove_selected = move |_: web_sys::MouseEvent| {
        let Some(key) = selected_key.get_untracked() else {
            return;
        };
        data.update(|rows| {
            let mut replacement = rows.as_ref().clone();
            replacement.retain(|row| row.id != key);
            *rows = Rc::new(replacement);
        });
    };
    let action_clicks = RwSignal::new(0_u32);
    let mut selection_columns = columns();
    // A row-action control must neither select nor activate the row it sits
    // in -- `EntityColumn::action` marks the cell `data-entity-action`, which
    // the row's click/keydown handlers stop-propagate past before either
    // selection or activation ever sees the event.
    selection_columns.push(
        EntityColumn::action("view", "Action", |_row: &FixtureRow| String::new()).render_with(
            move |row: &FixtureRow| {
                let id = row.id.clone();
                view! {
                    <EntityRowAction action_id="view">
                        <Button
                            attr:data-testid="entity-selection-row-action"
                            attr:data-entity-row-action-id=id
                            on:click=move |_| action_clicks.update(|count| *count += 1)
                        >
                            "View"
                        </Button>
                    </EntityRowAction>
                }
                .into_any()
            },
        ),
    );

    view! {
        <section
            id="entity-table-selection-fixture"
            class="mx-auto max-w-3xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Entity table selection"</h1>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Selected: "
                    <code data-testid="entity-selection-selected-key">
                        {move || selected_key.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                </span>
                <span>
                    "Proposals: "
                    <code data-testid="entity-selection-proposals">
                        {move || proposal_count.get().to_string()}
                    </code>
                </span>
                <span>
                    "Last proposal: "
                    <code data-testid="entity-selection-last-proposal">
                        {move || last_proposal.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                </span>
                <span>
                    "Activations: "
                    <code data-testid="entity-selection-activations">
                        {move || activation_count.get().to_string()}
                    </code>
                </span>
                <span>
                    "Action clicks: "
                    <code data-testid="entity-selection-action-clicks">
                        {move || action_clicks.get().to_string()}
                    </code>
                </span>
            </div>
            <div
                class="rounded border border-base-300 p-3 text-sm"
                data-testid="entity-selection-detail"
            >
                {move || {
                    let key = selected_key.get();
                    let detail = key.as_ref().and_then(|key| {
                        data.with(|rows| rows.iter().find(|row| &row.id == key).cloned())
                    });
                    match detail {
                        Some(row) => format!("{} — {}", row.client, row.status),
                        None => "(no row selected)".to_owned(),
                    }
                }}
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    on:click=move |_| accept_proposals.update(|accept| *accept = !*accept)
                    attr:data-testid="entity-selection-accept"
                >
                    {move || if accept_proposals.get() {
                        "Reject selection proposals"
                    } else {
                        "Accept selection proposals"
                    }}
                </Button>
                <Button
                    on:click=move |_| selected_key.set(None)
                    attr:data-testid="entity-selection-clear"
                >
                    "Clear selection"
                </Button>
                <Button
                    on:click=remove_selected
                    attr:data-testid="entity-selection-remove-selected"
                >
                    "Remove selected row"
                </Button>
                <Button
                    on:click=move |_| {
                        selected_key.set(None);
                        data.set(Rc::new(Vec::new()));
                    }
                    attr:data-testid="entity-selection-empty"
                >
                    "Empty table"
                </Button>
                <Button
                    on:click=move |_| data.set(rows("office-mx"))
                    attr:data-testid="entity-selection-restore"
                >
                    "Restore rows"
                </Button>
            </div>
            <EntityTable
                data=data
                columns=selection_columns
                row_key=Rc::new(|row: &FixtureRow| row.id.clone())
                dataset_identity="entity-selection-fixture"
                on_row_activate=Callback::new(move |_key: String| {
                    activation_count.update(|count| *count += 1);
                })
                selection=EntityTableSelection::controlled(
                    selected_key.into(),
                    Callback::new(move |proposed: Option<String>| {
                        proposal_count.update(|count| *count += 1);
                        last_proposal.set(proposed.clone());
                        if accept_proposals.get_untracked() {
                            selected_key.set(proposed);
                        }
                    }),
                )
                attr:id="entity-selection-table"
            />
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EmphasisRow {
    id: String,
    client: String,
    amount: i64,
    status: String,
}

fn emphasis_rows() -> Rc<Vec<EmphasisRow>> {
    Rc::new(vec![
        EmphasisRow {
            id: "emphasis-1".to_owned(),
            client: "Ledger row 1".to_owned(),
            amount: 120,
            status: "Ready".to_owned(),
        },
        EmphasisRow {
            id: "emphasis-2".to_owned(),
            client: "Ledger row 2".to_owned(),
            amount: 340,
            status: "Archived".to_owned(),
        },
        EmphasisRow {
            id: "emphasis-3".to_owned(),
            client: "Ledger row 3".to_owned(),
            amount: 60,
            status: "Overdue".to_owned(),
        },
        EmphasisRow {
            id: "emphasis-4".to_owned(),
            client: "Ledger row 4".to_owned(),
            amount: 480,
            status: "Ready".to_owned(),
        },
        // The largest amount, so an ascending sort by amount puts this row
        // last and a descending sort puts it first -- a real change of
        // rendered position for the browser fixture to prove classification
        // survives.
        EmphasisRow {
            id: "emphasis-total".to_owned(),
            client: "Total".to_owned(),
            amount: 1000,
            status: "Total".to_owned(),
        },
    ])
}

fn emphasis_columns() -> Vec<EntityColumn<EmphasisRow>> {
    vec![
        EntityColumn::text("client", "Client", |row: &EmphasisRow| row.client.clone())
            .required()
            .with_min_width(200),
        EntityColumn::text("status", "Status", |row: &EmphasisRow| row.status.clone())
            .with_min_width(120),
        EntityColumn::new("amount", "Amount", |row: &EmphasisRow| {
            row.amount.to_string()
        })
        .sortable_by_key(|row: &EmphasisRow| row.amount)
        .numeric()
        .with_width(140),
    ]
}

/// Focused browser fixture for `EntityTable` typed row emphasis (ldui-mqb):
/// one row of each `EntityRowEmphasis` variant, plus a totals row classified
/// `Summary` whose classification must survive a sort that moves it (the
/// `amount` column is sortable) and must read identically in the compact
/// single-cell presentation. `selection` is also wired so the fixture can
/// prove emphasis composes with, rather than fights, the selected-row
/// background, and `zebra=true` is on so the fixture can prove the same
/// against `table-zebra`'s own alternating-row striping.
#[component]
pub fn EntityTableEmphasisFixture() -> impl IntoView {
    let data = RwSignal::new_local(emphasis_rows());
    let selected_key = RwSignal::new(Option::<String>::None);

    view! {
        <section
            id="entity-table-emphasis-fixture"
            class="mx-auto max-w-3xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Entity table row emphasis"</h1>
            <p class="text-sm text-base-content/70">
                "One row of each EntityRowEmphasis variant classified purely from row content, plus a totals row that stays classified Summary after sorting moves it. Zebra striping is on, proving emphasis composes with it."
            </p>
            <EntityTable
                data=data
                columns=emphasis_columns()
                row_key=Rc::new(|row: &EmphasisRow| row.id.clone())
                dataset_identity="entity-table-emphasis-fixture"
                row_emphasis=Rc::new(|row: &EmphasisRow| match row.status.as_str() {
                    "Total" => EntityRowEmphasis::Summary,
                    "Archived" => EntityRowEmphasis::Muted,
                    "Overdue" => EntityRowEmphasis::Attention,
                    _ => EntityRowEmphasis::Standard,
                })
                selection=EntityTableSelection::controlled(
                    selected_key.into(),
                    Callback::new(move |proposed: Option<String>| selected_key.set(proposed)),
                )
                // Zebra composition (ldui-mqb, task review fix): every
                // emphasis variant is text/border only, never
                // background-color, so it must never fight `table-zebra`'s
                // own alternating-row background CSS.
                zebra=true
                attr:id="entity-emphasis-table"
            />
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BulkRow {
    id: String,
    client: String,
    status: String,
}

fn bulk_rows(dataset: &str) -> Rc<Vec<BulkRow>> {
    Rc::new(
        (1..=30)
            .map(|index| BulkRow {
                id: format!("{dataset}-{index:02}"),
                client: format!("{dataset} conversation {index:02}"),
                status: if index % 3 == 0 {
                    "Assigned"
                } else {
                    "Unassigned"
                }
                .to_owned(),
            })
            .collect(),
    )
}

fn bulk_columns() -> Vec<EntityColumn<BulkRow>> {
    vec![
        EntityColumn::text("client", "Conversation", |row: &BulkRow| row.client.clone())
            .required()
            .with_min_width(240),
        EntityColumn::text("status", "Status", |row: &BulkRow| row.status.clone())
            .with_min_width(140),
    ]
}

/// Controlled checkbox multi-selection over a client snapshot (`ldui-nz6d`).
///
/// Thirty rows against a 25-row default page, so page 2 is a genuine second
/// page: selecting page 1, paging forward and selecting page 2 proves both
/// that the header governs only what is displayed and that off-page keys
/// survive untouched.
#[component]
pub fn EntityTableMultiSelectionFixture() -> impl IntoView {
    let dataset = RwSignal::new("office-mx".to_owned());
    let data = RwSignal::new_local(bulk_rows("office-mx"));
    let accepted = RwSignal::new(BTreeSet::<String>::new());
    let accept_proposals = RwSignal::new(true);
    let proposal_count = RwSignal::new(0_u32);
    let last_cause = RwSignal::new("(none)".to_owned());
    let last_scope = RwSignal::new("(none)".to_owned());
    let unassigned_only = RwSignal::new(false);

    Effect::new(move |_| {
        let identity = dataset.get();
        let only_unassigned = unassigned_only.get();
        let mut rows = bulk_rows(&identity).as_ref().clone();
        if only_unassigned {
            rows.retain(|row| row.status == "Unassigned");
        }
        data.set(Rc::new(rows));
    });

    let remove_selected = move |_: web_sys::MouseEvent| {
        let selected = accepted.get_untracked();
        data.update(|rows| {
            let mut replacement = rows.as_ref().clone();
            replacement.retain(|row| !selected.contains(&row.id));
            *rows = Rc::new(replacement);
        });
    };

    view! {
        <section
            id="entity-table-multi-selection-fixture"
            class="mx-auto max-w-4xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Entity table bulk selection"</h1>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Selected: "
                    <code data-testid="entity-multi-selected-count">
                        {move || accepted.with(BTreeSet::len).to_string()}
                    </code>
                </span>
                <span>
                    "Keys: "
                    <code data-testid="entity-multi-selected-keys">
                        {move || {
                            let keys = accepted
                                .with(|keys| keys.iter().cloned().collect::<Vec<_>>())
                                .join(",");
                            if keys.is_empty() { "(none)".to_owned() } else { keys }
                        }}
                    </code>
                </span>
                <span>
                    "Proposals: "
                    <code data-testid="entity-multi-proposals">
                        {move || proposal_count.get().to_string()}
                    </code>
                </span>
                <span>
                    "Last cause: "
                    <code data-testid="entity-multi-last-cause">{move || last_cause.get()}</code>
                </span>
                <span>
                    "Last scope: "
                    <code data-testid="entity-multi-last-scope">{move || last_scope.get()}</code>
                </span>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    on:click=move |_| accept_proposals.update(|accept| *accept = !*accept)
                    attr:data-testid="entity-multi-accept"
                >
                    {move || if accept_proposals.get() {
                        "Reject selection proposals"
                    } else {
                        "Accept selection proposals"
                    }}
                </Button>
                <Button
                    on:click=move |_| unassigned_only.update(|only| *only = !*only)
                    attr:data-testid="entity-multi-filter"
                >
                    {move || if unassigned_only.get() {
                        "Show every conversation"
                    } else {
                        "Show unassigned only"
                    }}
                </Button>
                <Button
                    on:click=move |_| dataset.update(|identity| {
                        *identity = if identity == "office-mx" { "office-in" } else { "office-mx" }
                            .to_owned();
                    })
                    attr:data-testid="entity-multi-replace-dataset"
                >
                    "Replace dataset"
                </Button>
                <Button on:click=remove_selected attr:data-testid="entity-multi-remove-selected">
                    "Remove selected rows"
                </Button>
                <Button
                    on:click=move |_| accepted.set(BTreeSet::new())
                    attr:data-testid="entity-multi-clear"
                >
                    "Clear selection"
                </Button>
            </div>
            <EntityTable
                data=data
                columns=bulk_columns()
                row_key=Rc::new(|row: &BulkRow| row.id.clone())
                dataset_identity=Signal::derive(move || dataset.get())
                multi_selection=EntityTableMultiSelection::controlled(
                    accepted.into(),
                    Callback::new(move |proposal: EntityTableSelectionProposal| {
                        proposal_count.update(|count| *count += 1);
                        last_scope.set(proposal.scope.clone());
                        last_cause.set(match &proposal.cause {
                            EntityTableSelectionCause::Row { key, selected } => {
                                format!("row:{key}:{selected}")
                            }
                            EntityTableSelectionCause::DisplayedPage { selected, keys } => {
                                format!("page:{}:{selected}", keys.len())
                            }
                        });
                        if accept_proposals.get_untracked() {
                            accepted.set(proposal.keys);
                        }
                    }),
                )
                attr:id="entity-multi-selection-table"
            />
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CoordinatorActivityRow {
    id: String,
    coordinator_id: String,
    kind: String,
    measure: String,
    value: String,
    /// Machine arrival day, `""` for a record that has none. The fixture keeps
    /// the model value and the rendered cell deliberately separate so the date
    /// filter can be proved to compare the former (`ldui-lx5t`).
    arrived: String,
}

fn coordinator_activity_rows() -> Rc<Vec<CoordinatorActivityRow>> {
    let coordinators = [("co-1", 1_u32), ("co-2", 2), ("co-3", 3)];
    let kinds = ["Task", "Goal", "Actual"];
    Rc::new(
        coordinators
            .into_iter()
            .flat_map(|(coordinator_id, seed)| {
                kinds.into_iter().enumerate().flat_map(move |(slot, kind)| {
                    (1..=2).map(move |index| CoordinatorActivityRow {
                        id: format!("{coordinator_id}-{kind}-{index}"),
                        coordinator_id: coordinator_id.to_owned(),
                        kind: kind.to_owned(),
                        measure: format!("Intake week {index}"),
                        value: (seed * 10 + slot as u32 * 3 + index).to_string(),
                        // One kind is deliberately undated: a bounded date
                        // filter must exclude it, and an unbounded one must
                        // not.
                        arrived: if kind == "Actual" {
                            String::new()
                        } else {
                            format!("2026-08-{:02}", seed * 3 + slot as u32 + index)
                        },
                    })
                })
            })
            .collect(),
    )
}

fn coordinator_activity_columns() -> Vec<EntityColumn<CoordinatorActivityRow>> {
    vec![
        // The coordinator is DELIBERATELY not a column. Repeating it in 459
        // rows is the defect ldui-iyfa removes; the heading carries it once
        // and the display projection carries it into every export.
        EntityColumn::text("kind", "Activity", |row: &CoordinatorActivityRow| {
            row.kind.clone()
        })
        .required()
        .with_min_width(160),
        EntityColumn::text("measure", "Measure", |row: &CoordinatorActivityRow| {
            row.measure.clone()
        })
        .with_min_width(220),
        EntityColumn::new("value", "Value", |row: &CoordinatorActivityRow| {
            row.value.clone()
        })
        .sortable_by_key(|row: &CoordinatorActivityRow| {
            row.value.parse::<u32>().unwrap_or_default()
        })
        .numeric()
        .with_width(120),
        EntityColumn::text("arrived", "Arrived", |row: &CoordinatorActivityRow| {
            if row.arrived.is_empty() {
                "--".to_owned()
            } else {
                row.arrived.clone()
            }
        })
        .with_min_width(150),
    ]
}

/// Controlled accessible row groups over a client snapshot (`ldui-iyfa`).
///
/// Three coordinator groups, each holding repeated Task / Goal / Actual rows,
/// under ONE global column header and one controlled filter row -- the shape
/// Office Coordinator Activity needs and could not previously express without
/// repeating the coordinator name in every row or forking one table per
/// coordinator.
///
/// The fixture wires the whole contract so a browser lane can prove it:
/// controlled collapse, a child-row filter that empties a group, controlled
/// multi-selection (headings must never join the displayed-page population),
/// and a display-projection readout showing that the exported rows still carry
/// the group column and the stable group key.
#[component]
pub fn EntityTableGroupingFixture() -> impl IntoView {
    let source = coordinator_activity_rows();
    let kind_filter = RwSignal::new(String::new());
    let arrived_filter = RwSignal::new(String::new());
    let date_proposal = RwSignal::new("(none)".to_owned());
    let collapsed = RwSignal::new(BTreeSet::<String>::new());
    let collapse_proposals = RwSignal::new(0_u32);
    let accepted = RwSignal::new(BTreeSet::<String>::new());
    let label_suffix = RwSignal::new(false);
    let exported_group_cells = RwSignal::new(String::new());
    let exported_group_keys = RwSignal::new(String::new());

    let filtered = {
        let source = Rc::clone(&source);
        Signal::derive_local(move || {
            let kind = kind_filter.get();
            // The date surface is ANDed with the column filter, and it
            // compares the row's own machine value -- never the "--" the
            // undated rows RENDER.
            let cutoff = EntityDateFilter::parse_on_or_before(&arrived_filter.get());
            if kind.is_empty() && !cutoff.constrains() {
                return Rc::clone(&source);
            }
            Rc::new(
                source
                    .iter()
                    .filter(|row| kind.is_empty() || row.kind == kind)
                    .filter(|row| cutoff.matches(EntityDate::parse(&row.arrived).ok()))
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
    };

    // Labels are display copy and change independently of the keys, which is
    // what the fixture's "Relabel groups" control proves: nothing repartitions,
    // nothing reorders, and no collapse flag moves.
    let groups = Signal::derive_local(move || {
        let suffix = if label_suffix.get() { " (Field)" } else { "" };
        vec![
            EntityRowGroup::new("co-1", format!("Ana Ruiz{suffix}")).with_meta("Weekly cadence"),
            EntityRowGroup::new("co-2", format!("Beto Cruz{suffix}")),
            EntityRowGroup::new("co-3", format!("Cami Lopez{suffix}")),
        ]
    });

    let filters = vec![
        EntityColumnFilter::select(
            "kind",
            "entity-grouping-kind-filter",
            Signal::stored("Activity".to_owned()),
            Signal::derive(move || kind_filter.get()),
            Signal::stored("All activities".to_owned()),
            Signal::stored(vec![
                EntityColumnFilterOption::new("Task", "Task"),
                EntityColumnFilterOption::new("Goal", "Goal"),
                EntityColumnFilterOption::new("Actual", "Actual"),
            ]),
            Callback::new(move |value: String| kind_filter.set(value)),
        ),
        EntityColumnFilter::date(
            "arrived",
            "entity-grouping-arrived-filter",
            Signal::stored("Arrived on or before".to_owned()),
            Signal::derive(move || arrived_filter.get()),
            Signal::stored("Enter an arrival date as YYYY-MM-DD".to_owned()),
            Callback::new(move |proposal: EntityDateFilterProposal| {
                date_proposal.set(format!(
                    "{}|{:?}|{}|{}",
                    proposal.raw, proposal.cause, proposal.column_id, proposal.control_id
                ));
                arrived_filter.set(proposal.raw);
            }),
        ),
    ];

    view! {
        <section
            id="entity-table-grouping-fixture"
            class="mx-auto max-w-4xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Coordinator activity groups"</h1>
            <p class="ld-text-body text-base-content/75">
                "Three coordinator groups over one global column header and one filter row. The coordinator name is never a data cell."
            </p>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Collapsed: "
                    <code data-testid="entity-grouping-collapsed">
                        {move || {
                            let keys = collapsed
                                .with(|keys| keys.iter().cloned().collect::<Vec<_>>())
                                .join(",");
                            if keys.is_empty() { "(none)".to_owned() } else { keys }
                        }}
                    </code>
                </span>
                <span>
                    "Collapse proposals: "
                    <code data-testid="entity-grouping-collapse-proposals">
                        {move || collapse_proposals.get().to_string()}
                    </code>
                </span>
                <span>
                    "Selected: "
                    <code data-testid="entity-grouping-selected-count">
                        {move || accepted.with(BTreeSet::len).to_string()}
                    </code>
                </span>
                <span>
                    "Arrived cutoff: "
                    <code data-testid="entity-grouping-arrived-value">
                        {move || {
                            let raw = arrived_filter.get();
                            if raw.is_empty() { "(none)".to_owned() } else { raw }
                        }}
                    </code>
                </span>
                <span>
                    "Date proposal: "
                    <code data-testid="entity-grouping-date-proposal">
                        {move || date_proposal.get()}
                    </code>
                </span>
                <span>
                    "Exported group column: "
                    <code data-testid="entity-grouping-export-cells">
                        {move || exported_group_cells.get()}
                    </code>
                </span>
                <span>
                    "Exported group keys: "
                    <code data-testid="entity-grouping-export-keys">
                        {move || exported_group_keys.get()}
                    </code>
                </span>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    on:click=move |_| label_suffix.update(|suffix| *suffix = !*suffix)
                    attr:data-testid="entity-grouping-relabel"
                >
                    "Relabel groups"
                </Button>
                <Button
                    on:click=move |_| collapsed.set(BTreeSet::new())
                    attr:data-testid="entity-grouping-expand-all"
                >
                    "Expand all groups"
                </Button>
                // A native date picker cannot produce an unreadable value, but
                // a restored URL query or saved view can. This button stands in
                // for that restore so the error state is reachable.
                <Button
                    on:click=move |_| arrived_filter.set("2026-02-30".to_owned())
                    attr:data-testid="entity-grouping-restore-unreadable-date"
                >
                    "Restore an unreadable saved cutoff"
                </Button>
            </div>
            <EntityTable
                data=filtered
                columns=coordinator_activity_columns()
                row_key=Rc::new(|row: &CoordinatorActivityRow| row.id.clone())
                dataset_identity="entity-table-grouping-fixture"
                page_reset_key=Signal::derive(move || {
                    format!("{}|{}", kind_filter.get(), arrived_filter.get())
                })
                column_filters=filters
                row_grouping=EntityRowGrouping::controlled(
                    Rc::new(|row: &CoordinatorActivityRow| row.coordinator_id.clone()),
                    groups,
                )
                    .collapsible(
                        Signal::from(collapsed),
                        Callback::new(move |proposal: EntityGroupCollapseProposal| {
                            collapse_proposals.update(|count| *count += 1);
                            collapsed.set(proposal.keys);
                        }),
                    )
                multi_selection=EntityTableMultiSelection::controlled(
                    accepted.into(),
                    Callback::new(move |proposal: EntityTableSelectionProposal| {
                        accepted.set(proposal.keys);
                    }),
                )
                on_display_projection=Callback::new(move |projection: EntityTableDisplayProjection| {
                    let rows = projection.rows(EntityTableProjectionScope::AllFiltered);
                    exported_group_cells.set(
                        rows.iter()
                            .filter_map(|row| row.cells.first().cloned())
                            .collect::<Vec<_>>()
                            .join("|"),
                    );
                    exported_group_keys.set(
                        rows.iter()
                            .filter_map(|row| row.group_key.clone())
                            .collect::<Vec<_>>()
                            .join("|"),
                    );
                })
                attr:id="entity-grouping-table"
            />
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GroupPagingRow {
    id: String,
    office: String,
    measure: String,
    status: String,
    value: u32,
}

/// Three 17-row offices -- the exact Office Coordinator Activity shape -- plus
/// one 30-row office that cannot fit a 25-row page by any packing.
fn group_paging_rows() -> Rc<Vec<GroupPagingRow>> {
    let offices = [
        ("charlotte", "Charlotte", 17_u32),
        ("durham", "Durham", 17),
        ("raleigh", "Raleigh", 17),
        ("statewide", "Statewide", 30),
    ];
    Rc::new(
        offices
            .into_iter()
            .flat_map(|(key, label, count)| {
                (1..=count).map(move |index| GroupPagingRow {
                    id: format!("{key}-{index:02}"),
                    office: key.to_owned(),
                    measure: format!("{label} intake {index:02}"),
                    status: if index % 2 == 0 { "Open" } else { "Closed" }.to_owned(),
                    value: index,
                })
            })
            .collect(),
    )
}

fn group_paging_columns() -> Vec<EntityColumn<GroupPagingRow>> {
    vec![
        EntityColumn::text("measure", "Measure", |row: &GroupPagingRow| {
            row.measure.clone()
        })
        .required()
        .with_min_width(240),
        EntityColumn::text("status", "Status", |row: &GroupPagingRow| {
            row.status.clone()
        })
        .with_min_width(140),
        EntityColumn::new("value", "Value", |row: &GroupPagingRow| {
            row.value.to_string()
        })
        .sortable_by_key(|row: &GroupPagingRow| row.value)
        .numeric()
        .with_width(120),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NeighborRow {
    id: String,
    label: String,
}

/// Group-aware pagination, empty-state semantics, localized empty ranges, and
/// control identity (`ldui-5in5`, `ldui-g4nw`, `ldui-r50n`, `ldui-izkq`).
///
/// At a 25-row page the three 17-row offices each own a page: filling the
/// remainder of page 1 with eight Durham rows is exactly the defect. Statewide
/// holds 30 rows and therefore CANNOT be kept whole, so it degrades to the
/// previous fill-first behavior under the existing continuation heading -- both
/// branches of the rule are reachable on one page.
///
/// The same fixture carries the other three beads because they need the same
/// shape: a status filter that can select nothing (filtered-empty) beside a
/// control that empties the provider (provider-empty), and a second mounted
/// table proving two `EntityTable`s never mint the same control id.
#[component]
pub fn EntityTableGroupPagingFixture() -> impl IntoView {
    let source = RwSignal::new_local(group_paging_rows());
    let status_filter = RwSignal::new(String::new());
    let empty_range_spanish = RwSignal::new(false);
    let accepted = RwSignal::new(BTreeSet::<String>::new());
    let neighbor_accepted = RwSignal::new(BTreeSet::<String>::new());

    let filtered = Signal::derive_local(move || {
        let status = status_filter.get();
        let rows = source.get();
        if status.is_empty() {
            return rows;
        }
        Rc::new(
            rows.iter()
                .filter(|row| row.status == status)
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    let groups = Signal::derive_local(|| {
        vec![
            EntityRowGroup::new("charlotte", "Charlotte"),
            EntityRowGroup::new("durham", "Durham"),
            EntityRowGroup::new("raleigh", "Raleigh"),
            EntityRowGroup::new("statewide", "Statewide"),
        ]
    });

    // Only `no_rows` is overridden, exactly as a consumer that predates
    // `no_matching_rows` would have it: the provider-empty sentence stays this
    // one, and the filtered-empty case inherits the framework default instead
    // of asserting the provider is empty when it is not.
    let texts = Signal::stored(EntityTableTexts {
        no_rows: "No activity is present in this snapshot.".to_owned(),
        ..EntityTableTexts::default()
    });
    let empty_row_range = Signal::derive(move || {
        if empty_range_spanish.get() {
            "Filas de actividad: 0 de {total}".to_owned()
        } else {
            "Activity rows 0 of {total}".to_owned()
        }
    });

    let filters = vec![EntityColumnFilter::select(
        "status",
        "entity-group-paging-status-filter",
        Signal::stored("Status".to_owned()),
        Signal::derive(move || status_filter.get()),
        Signal::stored("All statuses".to_owned()),
        Signal::stored(vec![
            EntityColumnFilterOption::new("Open", "Open"),
            EntityColumnFilterOption::new("Closed", "Closed"),
            // Matches nothing, so the projection empties while the provider
            // stays full -- the only way to reach the filtered-empty copy.
            EntityColumnFilterOption::new("Void", "Void"),
        ]),
        Callback::new(move |value: String| status_filter.set(value)),
    )];

    let neighbors = Signal::stored_local(Rc::new(
        (1..=3)
            .map(|index| NeighborRow {
                id: format!("neighbor-{index}"),
                label: format!("Neighboring table row {index}"),
            })
            .collect::<Vec<_>>(),
    ));

    view! {
        <section
            id="entity-table-group-paging-fixture"
            class="mx-auto max-w-4xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"Group-aware pagination"</h1>
            <p class="ld-text-body text-base-content/75">
                "Three seventeen-row offices and one thirty-row office. A group that fits a page is never split to fill the previous page's remainder; one that cannot fit keeps its continuation heading."
            </p>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Source rows: "
                    <code data-testid="entity-group-paging-source-count">
                        {move || source.with(|rows| rows.len()).to_string()}
                    </code>
                </span>
                <span>
                    "Selected: "
                    <code data-testid="entity-group-paging-selected-count">
                        {move || accepted.with(BTreeSet::len).to_string()}
                    </code>
                </span>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    on:click=move |_| source.set(Rc::new(Vec::new()))
                    attr:data-testid="entity-group-paging-drain-provider"
                >
                    "Drain the provider"
                </Button>
                <Button
                    on:click=move |_| source.set(group_paging_rows())
                    attr:data-testid="entity-group-paging-restore-provider"
                >
                    "Restore the provider"
                </Button>
                <Button
                    on:click=move |_| empty_range_spanish.set(true)
                    attr:data-testid="entity-group-paging-empty-range-spanish"
                >
                    "Use Spanish empty range"
                </Button>
            </div>
            <EntityTable
                data=filtered
                source_data=source.into()
                columns=group_paging_columns()
                row_key=Rc::new(|row: &GroupPagingRow| row.id.clone())
                dataset_identity="entity-table-group-paging-fixture"
                page_reset_key=Signal::derive(move || status_filter.get())
                viewport_fit=EntityTableViewportFit::max_height("22rem").with_min_rows(3)
                column_filters=filters
                texts=texts
                empty_row_range=empty_row_range
                control_id="group-paging-table"
                row_grouping=EntityRowGrouping::controlled(
                    Rc::new(|row: &GroupPagingRow| row.office.clone()),
                    groups,
                )
                multi_selection=EntityTableMultiSelection::controlled(
                    accepted.into(),
                    Callback::new(move |proposal: EntityTableSelectionProposal| {
                        accepted.set(proposal.keys);
                    }),
                )
                attr:id="entity-group-paging-table"
            />
            // A second mounted table with NO `control_id`: its minted prefix
            // must not collide with the one above, which is the "multiple
            // tables on one page" half of ldui-izkq.
            <EntityTable
                data=neighbors
                columns=vec![
                    EntityColumn::text("label", "Neighbor", |row: &NeighborRow| row.label.clone())
                        .required()
                        .with_min_width(240),
                ]
                row_key=Rc::new(|row: &NeighborRow| row.id.clone())
                dataset_identity="entity-table-group-paging-neighbor"
                multi_selection=EntityTableMultiSelection::controlled(
                    neighbor_accepted.into(),
                    Callback::new(move |proposal: EntityTableSelectionProposal| {
                        neighbor_accepted.set(proposal.keys);
                    }),
                )
                attr:id="entity-group-paging-neighbor-table"
            />
        </section>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NoteRow {
    id: String,
    title: String,
    status: String,
}

fn note_rows() -> Rc<Vec<NoteRow>> {
    Rc::new(
        (1..=5)
            .map(|index| NoteRow {
                id: format!("ON-100{index}"),
                title: format!("Office note {index}"),
                status: if index == 2 { "Archived" } else { "Active" }.to_owned(),
            })
            .collect(),
    )
}

/// Typed focus requests for mutations the table never sees (`ldui-o0iw`).
///
/// The Delete button lives in the editor panel BESIDE the table, so it is
/// destroyed along with the row it deletes and focus falls to `<body>`. The
/// page supplies the stable successor as a typed request; the table resolves it
/// against the rows it is painting and reports what it actually did.
#[component]
pub fn EntityTableExternalFocusFixture() -> impl IntoView {
    let source = RwSignal::new_local(note_rows());
    let selected = RwSignal::new(Some("ON-1003".to_owned()));
    let status_filter = RwSignal::new(String::new());
    let generation = RwSignal::new(1_u32);
    let request_id = RwSignal::new(0_u64);
    let focus_request = RwSignal::new(Option::<EntityFocusRequest>::None);
    let resolution = RwSignal::new("(none)".to_owned());

    let focus_scope = Signal::derive(move || format!("gen-{}", generation.get()));
    let filtered = Signal::derive_local(move || {
        let status = status_filter.get();
        let rows = source.get();
        if status.is_empty() {
            return rows;
        }
        Rc::new(
            rows.iter()
                .filter(|row| row.status == status)
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    let issue = move |row_key: String, action: Option<&'static str>, scope: String| {
        request_id.update(|id| *id += 1);
        let id = request_id.get_untracked();
        focus_request.set(Some(match action {
            Some(action_id) => EntityFocusRequest::row_action(id, scope, row_key, action_id),
            None => EntityFocusRequest::row(id, scope, row_key),
        }));
    };

    // The editor's own Delete: the central data is replaced and the page
    // supplies the stable successor. Issued ONLY because the mutation was
    // accepted -- a declined one issues nothing and leaves editor focus alone.
    let delete_selected = move |action: Option<&'static str>| {
        let Some(key) = selected.get_untracked() else {
            return;
        };
        let rows = source.get_untracked();
        let Some(position) = rows.iter().position(|row| row.id == key) else {
            return;
        };
        let mut replacement = rows.as_ref().clone();
        replacement.remove(position);
        let successor = replacement
            .get(position)
            .or_else(|| replacement.get(position.saturating_sub(1)))
            .map(|row| row.id.clone());
        source.set(Rc::new(replacement));
        selected.set(successor.clone());
        if let Some(successor) = successor {
            issue(successor, action, focus_scope.get_untracked());
        }
    };

    let columns = vec![
        EntityColumn::text("title", "Note", |row: &NoteRow| row.title.clone())
            .required()
            .with_min_width(240),
        EntityColumn::text("status", "Status", |row: &NoteRow| row.status.clone())
            .with_min_width(140),
        EntityColumn::action("open", "Action", |_row: &NoteRow| String::new()).render_with(
            move |row: &NoteRow| {
                let key = row.id.clone();
                view! {
                    <EntityRowAction action_id="open">
                        <Button
                            attr:data-testid="entity-external-focus-open"
                            attr:data-entity-row-action-id=key
                        >
                            "Open"
                        </Button>
                    </EntityRowAction>
                }
                .into_any()
            },
        ),
    ];

    let filters = vec![EntityColumnFilter::select(
        "status",
        "entity-external-focus-status-filter",
        Signal::stored("Status".to_owned()),
        Signal::derive(move || status_filter.get()),
        Signal::stored("All statuses".to_owned()),
        Signal::stored(vec![
            EntityColumnFilterOption::new("Active", "Active"),
            EntityColumnFilterOption::new("Archived", "Archived"),
        ]),
        Callback::new(move |value: String| status_filter.set(value)),
    )];

    view! {
        <section
            id="entity-table-external-focus-fixture"
            class="mx-auto max-w-4xl space-y-3 bg-base-100 p-4"
        >
            <h1 class="ld-text-display font-semibold">"External editor focus requests"</h1>
            <div class="flex flex-wrap items-center gap-3 text-sm">
                <span>
                    "Selected: "
                    <code data-testid="entity-external-focus-selected">
                        {move || selected.get().unwrap_or_else(|| "(none)".to_owned())}
                    </code>
                </span>
                <span>
                    "Scope: "
                    <code data-testid="entity-external-focus-scope">{move || focus_scope.get()}</code>
                </span>
                <span>
                    "Resolution: "
                    <code data-testid="entity-external-focus-resolution">
                        {move || resolution.get()}
                    </code>
                </span>
            </div>
            <div
                class="flex flex-wrap gap-2 rounded border border-base-300 p-3"
                data-testid="entity-external-focus-editor"
            >
                <Button
                    on:click=move |_| delete_selected(None)
                    attr:data-testid="entity-external-focus-delete"
                >
                    "Delete selected note"
                </Button>
                <Button
                    on:click=move |_| delete_selected(Some("open"))
                    attr:data-testid="entity-external-focus-delete-to-action"
                >
                    "Delete and focus the successor's action"
                </Button>
                <Button
                    on:click=move |_| issue(
                        "ON-1002".to_owned(),
                        None,
                        focus_scope.get_untracked(),
                    )
                    attr:data-testid="entity-external-focus-request-hidden"
                >
                    "Request a row that may be filtered away"
                </Button>
                <Button
                    on:click=move |_| issue(
                        "ON-1001".to_owned(),
                        None,
                        "gen-stale".to_owned(),
                    )
                    attr:data-testid="entity-external-focus-request-stale"
                >
                    "Request with a stale scope"
                </Button>
                <Button
                    on:click=move |_| generation.update(|value| *value += 1)
                    attr:data-testid="entity-external-focus-bump-scope"
                >
                    "Bump the access scope"
                </Button>
            </div>
            <EntityTable
                data=filtered
                source_data=source.into()
                columns=columns
                row_key=Rc::new(|row: &NoteRow| row.id.clone())
                dataset_identity="entity-table-external-focus-fixture"
                page_reset_key=Signal::derive(move || status_filter.get())
                column_filters=filters
                focus_scope=focus_scope
                focus_request=focus_request
                on_focus_request_resolved=Callback::new(
                    move |resolved: EntityFocusRequestResolution| {
                        resolution.set(resolved.to_string());
                    },
                )
                selection=EntityTableSelection::controlled(
                    selected.into(),
                    Callback::new(move |proposed: Option<String>| selected.set(proposed)),
                )
                attr:id="entity-external-focus-table"
            />
        </section>
    }
}
