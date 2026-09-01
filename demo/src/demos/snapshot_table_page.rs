use leptos::prelude::*;
use leptos_daisyui_rs::components::{
    BadgeColor, Button, EntityBadgePresentation, EntityColumn, EntityColumnChooserTrigger,
    EntityColumnFilter, EntityIconColor, EntityIconPresentation, EntityNullOrder, EntityPageSize,
    EntityRowAction, EntityRowEmphasis, EntityTable, EntityTableDisplayProjection,
    EntityTablePreferenceOwnership, EntityTablePreferencePersistence, EntityTableProjectionScope,
    EntityTableSelection, EntityTableTexts, EntityTableViewportFit,
};
use leptos_daisyui_rs::patterns::{
    ActionFeedbackContent, ActionFeedbackState, PageHeader, PageHeaderNavigationLayout,
    SnapshotData, SnapshotDatasetOption, SnapshotDatasetSelectorConfig, SnapshotDeltaDisposition,
    SnapshotDeltaHandle, SnapshotEntityTableConfig, SnapshotLocalRowProjection,
    SnapshotRequestHandle, SnapshotTablePage, SnapshotTableState, SnapshotTransitionDisposition,
};
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
    // 300px, not the original 160px: at the 1280x800 smoke viewport the
    // rows-per-page row, the sr-only live region, and the pagination footer
    // already consume ~95px of any budget before the scrollable region gets
    // a share, and a real measured row here is ~35px with a ~44px header.
    // 160px left the region only ~64px tall -- less than one row -- so
    // `auto_page_size_for_height` legitimately took its documented
    // below-`min_rows` branch and retained the full configured page size
    // (25) instead of a fitted count. That is the library behaving exactly
    // as designed (see `auto_page_size_for_height`'s doc comment); the fix
    // is a fixture budget that actually clears `min_rows` while still
    // paging before all 8 rows. 300px measures to 4 rows here, comfortably
    // between `min_rows(2)` and 8.
    .with_viewport_fit(EntityTableViewportFit::max_height("300px").with_min_rows(2))
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
            />
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
