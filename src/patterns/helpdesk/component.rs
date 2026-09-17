//! The `Helpdesk` composite: buckets, filters, the ticket table, the detail
//! drawer and the New Request dialog, wired to a host-supplied backend.

use super::backend::HelpdeskBackend;
use super::drawer::{TicketDetailDrawer, TriageAction, priority_color, status_tone};
use super::model::*;
use super::request_dialog::NewRequestDialog;
use super::state::{Bucket, RoleCapabilities, TicketFilter, bucket_counts, relative_age};
use super::texts::HelpdeskTexts;
use crate::components::{
    Badge, BadgeColor, BadgeSize, Button, ButtonColor, ButtonSize, EntityAutoFilterTexts,
    EntityAutoFilters, EntityColumn, EntityColumnChooserTrigger, EntityTable, Icon, IconSize,
    Input, Select, SelectOption, Toast, Toggle,
};
use crate::patterns::{
    FilterBar, FilterResultSummary, LIST_PAGE_BASE_CLASS, PageHeader, PageStatePanel,
    PageStatePanelKind, SelectableSummaryGroup, SelectableSummaryItem,
};
use crate::widgets::{AvatarBadge, AvatarBadgeSize};
use leptos::prelude::*;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

/// Upsert a ticket by key, preserving position for an update and prepending
/// an unknown key (a freshly-created ticket).
pub(crate) fn replace_by_key(v: &mut Vec<HelpdeskTicket>, t: HelpdeskTicket) {
    match v.iter().position(|x| x.key == t.key) {
        Some(i) => v[i] = t,
        None => v.insert(0, t),
    }
}

/// A copy of the list, most-recently-updated first.
pub(crate) fn sorted_desc(v: &[HelpdeskTicket]) -> Vec<HelpdeskTicket> {
    let mut s = v.to_vec();
    s.sort_by_key(|a| std::cmp::Reverse(a.updated_at_ms));
    s
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListPhase {
    Loading,
    Ready,
    Error,
}

fn js_now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// The role-switched Jira-helpdesk composite. See
/// `doc/plans/2026-09-14-helpdesk-composite-design.md`.
#[component]
pub fn Helpdesk(
    /// The transport. `Rc` so the same instance can back both the standalone
    /// page and an F2 drawer.
    backend: Rc<dyn HelpdeskBackend>,
    /// Which mode to render in.
    #[prop(into)]
    role: Signal<HelpdeskRole>,
    /// Where a filed request is coming from.
    #[prop(into)]
    context: Signal<RequestContext>,
    /// The caller's own id, for "mine only" filtering.
    #[prop(optional, into)]
    current_user_id: Signal<Option<String>>,
    /// All rendered copy.
    #[prop(optional, into, default = Signal::stored(HelpdeskTexts::default()))]
    texts: Signal<HelpdeskTexts>,
    /// Set true to open the New Request dialog directly (an F2 launcher).
    #[prop(optional)]
    open_request: Option<RwSignal<bool>>,
    /// Overrides the clock, for deterministic proofs.
    #[prop(optional, into)]
    now_ms: Option<Signal<i64>>,
    /// Extra classes merged onto the root.
    #[prop(optional, into)]
    class: &'static str,
    /// Node reference for the root `<div>`.
    /// Whether the New Request dialog offers image attachments. Defaults to
    /// `true`; a host whose backend takes no images sets `false` (ldui-8tlg).
    #[prop(optional, default = true)]
    attachments: bool,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let backend = StoredValue::new_local(backend);
    let caps = Signal::derive(move || RoleCapabilities::for_role(role.get()));
    let now = now_ms.unwrap_or_else(|| Signal::derive(js_now_ms));
    let dialog_open = open_request.unwrap_or_else(|| RwSignal::new(false));

    let meta: RwSignal<Option<HelpdeskMeta>> = RwSignal::new(None);
    let tickets: RwSignal<Vec<HelpdeskTicket>> = RwSignal::new(Vec::new());
    let phase = RwSignal::new(ListPhase::Loading);
    let list_error = RwSignal::new(Option::<String>::None);
    let filter = RwSignal::new(TicketFilter::default());
    let selected: RwSignal<Option<TicketKey>> = RwSignal::new(None);
    let detail: RwSignal<Option<TicketDetail>> = RwSignal::new(None);
    let feedback: RwSignal<Option<(bool, String)>> = RwSignal::new(None);
    let toast: RwSignal<Option<HelpdeskTicket>> = RwSignal::new(None);
    let generation = RwSignal::new(0u32);

    let load_meta = move || {
        spawn_local(async move {
            let fut = backend.with_value(|b| b.meta());
            if let Ok(m) = fut.await {
                meta.try_set(Some(m));
            }
        });
    };

    let load_list = move || {
        let scope = caps.get_untracked().scope;
        let generation_at_request = generation.get_untracked() + 1;
        generation.set(generation_at_request);
        phase.set(ListPhase::Loading);
        spawn_local(async move {
            let fut = backend.with_value(|b| b.list(scope));
            let r = fut.await;
            if generation.try_get_untracked() != Some(generation_at_request) {
                return;
            }
            match r {
                Ok(v) => {
                    tickets.try_set(v);
                    phase.try_set(ListPhase::Ready);
                }
                Err(e) => {
                    list_error.try_set(Some(e.message));
                    phase.try_set(ListPhase::Error);
                }
            }
        });
    };

    load_meta();
    Effect::new(move |_| {
        let _ = role.get();
        load_list();
    });

    let open_detail = move |key: TicketKey| {
        selected.set(Some(key.clone()));
        feedback.set(None);
        detail.set(None);
        spawn_local(async move {
            let fut = backend.with_value(|b| b.detail(&key));
            if let Ok(d) = fut.await
                && selected.try_get_untracked().flatten().as_ref() == Some(&key)
            {
                detail.try_set(Some(d));
            }
        });
    };
    let close_detail = move || {
        selected.set(None);
        detail.set(None);
    };

    let apply_ticket = move |t: HelpdeskTicket| {
        tickets.update(|v| replace_by_key(v, t.clone()));
        detail.update(|d| {
            if let Some(d) = d.as_mut()
                && d.ticket.key == t.key
            {
                d.ticket = t;
            }
        });
    };

    let on_action = Callback::new(move |a: TriageAction| {
        let Some(key) = selected.get_untracked() else {
            return;
        };
        let before = detail.get_untracked();
        feedback.set(None);
        spawn_local(async move {
            let r: Result<(), HelpdeskError> = match a {
                TriageAction::Transition { to_status_id } => backend
                    .with_value(|b| b.transition(&key, &to_status_id))
                    .await
                    .map(apply_ticket),
                TriageAction::Assign { assignee_id } => backend
                    .with_value(|b| b.assign(&key, assignee_id.as_deref()))
                    .await
                    .map(apply_ticket),
                TriageAction::SetPriority { priority_id } => backend
                    .with_value(|b| b.set_priority(&key, &priority_id))
                    .await
                    .map(apply_ticket),
                TriageAction::Comment { body } => backend
                    .with_value(|b| b.comment(&key, &body))
                    .await
                    .map(|c| {
                        detail.update(|d| {
                            if let Some(d) = d.as_mut() {
                                d.comments.push(c);
                                d.ticket.comment_count += 1;
                            }
                        });
                        if let Some(d) = detail.get_untracked() {
                            apply_ticket(d.ticket);
                        }
                    }),
            };
            match r {
                Ok(()) => {
                    feedback.try_set(Some((false, texts.get_untracked().saved)));
                }
                Err(e) => {
                    detail.try_set(before);
                    feedback.try_set(Some((
                        true,
                        format!("{}: {}", texts.get_untracked().action_failed, e.message),
                    )));
                }
            }
        });
    });

    let on_submit = Callback::new(
        move |(nt, done): (NewTicket, Callback<Result<HelpdeskTicket, HelpdeskError>>)| {
            spawn_local(async move {
                let r = backend.with_value(|b| b.create(nt)).await;
                if let Ok(t) = &r {
                    tickets.update(|v| replace_by_key(v, t.clone()));
                    toast.set(Some(t.clone()));
                }
                done.run(r);
            });
        },
    );

    let counts = Signal::derive(move || bucket_counts(&tickets.get()));
    let bucket_items = Signal::derive(move || {
        let t = texts.get();
        let c = counts.get();
        vec![
            SelectableSummaryItem::new(Bucket::Open.id(), t.bucket_open, c.open),
            SelectableSummaryItem::new(
                Bucket::InProgress.id(),
                t.bucket_in_progress,
                c.in_progress,
            ),
            SelectableSummaryItem::new(Bucket::Done.id(), t.bucket_done, c.done),
        ]
    });
    let visible = Signal::derive(move || {
        let f = filter.get();
        let me = current_user_id.get();
        sorted_desc(&tickets.get())
            .into_iter()
            .filter(|t| f.matches(t, me.as_deref()))
            .collect::<Vec<_>>()
    });
    let table_data = Signal::derive_local(move || Rc::new(visible.get()));
    let result = Signal::derive(move || FilterResultSummary {
        visible: visible.get().len(),
        total: tickets.get().len(),
    });

    // `EntityColumn<HelpdeskTicket>` holds `Rc<dyn Fn>` renderers, so it is
    // neither `Send` nor `Sync`. `<Show>`'s children slot is
    // `Arc<dyn Fn() -> C + Send + Sync>`: the CLOSURE's captured environment
    // must be `Send + Sync`, even though `C` itself need not be. A
    // `StoredValue` handle is `Send + Sync` regardless of what it stores, so
    // capturing the handle (and cloning the columns out of it only when the
    // closure runs) is what lets the table live inside `Show` at all --
    // capturing `columns` directly, as a bare `Vec`, does not compile.
    let columns = StoredValue::new_local({
        let tx = texts;
        vec![
            EntityColumn::<HelpdeskTicket>::new("key", tx.get_untracked().col_key, |t| t.key.0.clone())
                .identifier()
                .filterable()
                .with_width(96)
                .required(),
            EntityColumn::new("summary", tx.get_untracked().col_summary, |t: &HelpdeskTicket| t.summary.clone())
                .render_with(move |t: &HelpdeskTicket| {
                    let k = tx.get_untracked().kind_name(&t.kind);
                    view! {
                        <span class="flex items-center gap-2">
                            // `Sm`, not `Xs`: daisyUI's `badge-xs` is 10px, off the type
                            // ramp (style audit TYPOGRAPHY, one per row).
                            <Badge size=BadgeSize::Sm color=BadgeColor::Neutral>
                                {k}
                            </Badge>
                            <span class="truncate">{t.summary.clone()}</span>
                        </span>
                    }
                    .into_any()
                })
                .ellipsis()
                .filterable()
                .required(),
            EntityColumn::new("priority", tx.get_untracked().col_priority, move |t: &HelpdeskTicket| {
                tx.get_untracked().priority_name(t.priority)
            })
            .render_with(move |t: &HelpdeskTicket| {
                let color = priority_color(t.priority);
                let label = tx.get_untracked().priority_name(t.priority);
                view! {
                    <Badge size=BadgeSize::Sm color=color>
                        {label}
                    </Badge>
                }
                .into_any()
            })
            .sortable_by_key(|t: &HelpdeskTicket| t.priority.rank())
            .filterable_options()
            .with_width(112),
            EntityColumn::new("status", tx.get_untracked().col_status, |t: &HelpdeskTicket| t.status.name.clone())
                .render_with(move |t: &HelpdeskTicket| {
                    let color = match status_tone(t.status.category) {
                        crate::patterns::RecordStatusTone::Success => BadgeColor::Success,
                        crate::patterns::RecordStatusTone::Warning => BadgeColor::Warning,
                        _ => BadgeColor::Info,
                    };
                    let label = t.status.name.clone();
                    view! {
                        <Badge size=BadgeSize::Sm color=color>
                            {label}
                        </Badge>
                    }
                    .into_any()
                })
                .filterable_options()
                .with_width(160),
            EntityColumn::new("assignee", tx.get_untracked().col_assignee, move |t: &HelpdeskTicket| {
                t.assignee
                    .as_ref()
                    .map(|p| p.display_name.clone())
                    .unwrap_or_else(|| tx.get_untracked().unassigned)
            })
            .render_with(move |t: &HelpdeskTicket| match &t.assignee {
                Some(p) => view! {
                    <AvatarBadge
                        initials=p.initials.clone()
                        // Not `name=`: `NAME_PALETTE`'s bg-primary/text-primary-content
                        // pair measures 4.12:1 at this size in the active theme (axe
                        // `color-contrast`, needs 4.5:1) — the palette's own
                        // "contrast-safe by construction" doc comment does not hold
                        // here. A solid neutral pairing is safe; see
                        // entity-table-columns-need-storedvalue-inside-show memory
                        // for the follow-up.
                        bg_class="bg-neutral text-neutral-content".to_owned()
                        size=AvatarBadgeSize::Sm
                    />
                }
                .into_any(),
                None => view! { <span class="text-base-content/75">{tx.get_untracked().unassigned}</span> }.into_any(),
            })
            .filterable_options()
            .with_width(96),
            EntityColumn::new("age", tx.get_untracked().col_age, move |t: &HelpdeskTicket| relative_age(now.get_untracked(), t.created_at_ms))
                .render_with(move |t: &HelpdeskTicket| {
                    let age = relative_age(now.get_untracked(), t.created_at_ms);
                    view! {
                        <time datetime=t.created_at_ms.to_string() class="tabular-nums">
                            {age}
                        </time>
                    }
                    .into_any()
                })
                .sortable_by_key(|t: &HelpdeskTicket| std::cmp::Reverse(t.created_at_ms))
                .filterable()
                .with_width(72),
        ]
    });

    // The framework-built filter row (ldui-ei8v; Office op-ulgfu's
    // "standard record-list" features). Every column but the (absent) action
    // column joins it: categorical columns (priority/status/assignee) get an
    // option list derived from their distinct cell texts, the rest a
    // case-insensitive substring box. It composes ON TOP of the FilterBar
    // above (`visible` -> `table_data` is the auto filters' SOURCE, so an
    // over-narrow column filter still classifies as "no matching rows" via
    // `source_data`).
    let auto_filter_texts = Signal::derive(move || {
        let t = texts.get();
        EntityAutoFilterTexts {
            label: t.filter_label.clone(),
            placeholder: t.filter_placeholder.clone(),
            all: t.filter_all.clone(),
        }
    });
    // The control-id prefix namespaces the generated `<label for>`/`input id`
    // pairs; the test fixture mounts BOTH roles on one document, so the
    // mount-time role keeps the two tables' ids distinct.
    let auto_filter_prefix = format!(
        "helpdesk-{}",
        match role.get_untracked() {
            HelpdeskRole::Requester => "requester",
            HelpdeskRole::Support => "support",
        }
    );
    // Same `StoredValue` rationale as `columns` above: `EntityAutoFilters`
    // carries the per-column value signals' `Rc`-based predicates, so only
    // its Send+Sync handle can cross into `Show`'s children closure.
    let auto_filters = StoredValue::new_local(EntityAutoFilters::new(
        &columns.get_value(),
        table_data,
        auto_filter_prefix,
        auto_filter_texts,
    ));

    let on_bucket = Callback::new(move |id: String| {
        filter.update(|f| {
            f.bucket = if f.bucket == Bucket::from_id(&id) {
                None
            } else {
                Bucket::from_id(&id)
            };
        });
    });
    let selected_bucket = Signal::derive(move || filter.get().bucket.map(|b| b.id().to_owned()));

    view! {
        <div
            class=format!("{LIST_PAGE_BASE_CLASS} {class}")
            data-helpdesk=""
            data-helpdesk-role=move || match role.get() {
                HelpdeskRole::Requester => "requester",
                HelpdeskRole::Support => "support",
            }
            node_ref=node_ref
        >
            <PageHeader
                title=Signal::derive(move || texts.get().title)
                actions=Box::new(move || {
                    view! {
                        <Button size=ButtonSize::Sm on_click=Callback::new(move |_| load_list()) attr:data-helpdesk-refresh="">
                            {move || texts.get().refresh}
                        </Button>
                        <Button
                            color=ButtonColor::Primary
                            size=ButtonSize::Sm
                            on_click=Callback::new(move |_| dialog_open.set(true))
                            attr:data-helpdesk-new-request=""
                        >
                            {move || texts.get().new_request}
                        </Button>
                    }
                        .into_any()
                })
            />
            <SelectableSummaryGroup
                label=Signal::derive(move || texts.get().buckets_label)
                items=bucket_items
                selected=selected_bucket
                on_select=on_bucket
                attr:data-helpdesk-buckets=""
            />
            <FilterBar
                result=result
                on_reset=Callback::new(move |_| filter.set(TicketFilter::default()))
                search=Box::new(move || {
                    // Every filter control sits inside a real `<label>` with
                    // visually hidden text: an `aria-label` alone satisfies axe
                    // but is still an input outside any field (component-drift
                    // `input-outside-field`), and a visible label is only a
                    // placeholder's worth of words here.
                    view! {
                        <label class="flex min-w-0 flex-1">
                            <span class="sr-only">{move || texts.get().search_placeholder}</span>
                            <Input
                                value=Signal::derive(move || filter.get().search)
                                placeholder=Signal::derive(move || texts.get().search_placeholder)
                                on_input=Callback::new(move |v: String| filter.update(|f| f.search = v))
                                attr:data-helpdesk-search=""
                            />
                        </label>
                    }
                        .into_any()
                })
            >
                <label class="flex min-w-0">
                <span class="sr-only">{move || texts.get().filter_priority}</span>
                <Select
                    value=Signal::derive(move || {
                        filter.get().priority.map(|p| format!("{p:?}")).unwrap_or_default()
                    })
                    on_change=Callback::new(move |v: String| {
                        filter.update(|f| {
                            f.priority = [
                                TicketPriority::Highest,
                                TicketPriority::High,
                                TicketPriority::Medium,
                                TicketPriority::Low,
                                TicketPriority::Lowest,
                            ]
                                .into_iter()
                                .find(|p| format!("{p:?}") == v);
                        });
                    })
                    attr:data-helpdesk-filter-priority=""
                >
                    <SelectOption attr:value="">{move || texts.get().any_priority}</SelectOption>
                    {[
                        TicketPriority::Highest,
                        TicketPriority::High,
                        TicketPriority::Medium,
                        TicketPriority::Low,
                        TicketPriority::Lowest,
                    ]
                        .into_iter()
                        .map(|p| {
                            view! { <SelectOption attr:value=format!("{p:?}")>{move || texts.get().priority_name(p)}</SelectOption> }
                        })
                        .collect_view()}
                </Select>
                </label>
                <Show when=move || caps.get().assignee_filter>
                    <label class="flex min-w-0">
                    <span class="sr-only">{move || texts.get().filter_assignee}</span>
                    <Select
                        value=Signal::derive(move || filter.get().assignee_id.unwrap_or_default())
                        on_change=Callback::new(move |v: String| {
                            filter.update(|f| f.assignee_id = (!v.is_empty()).then_some(v));
                        })
                        attr:data-helpdesk-filter-assignee=""
                    >
                        <SelectOption attr:value="">{move || texts.get().any_assignee}</SelectOption>
                        <For each=move || meta.get().map(|m| m.assignable).unwrap_or_default() key=|p| p.id.clone() let:p>
                            <SelectOption attr:value=p.id.clone()>{p.display_name.clone()}</SelectOption>
                        </For>
                    </Select>
                    </label>
                </Show>
                <Show when=move || caps.get().mine_only_toggle>
                    <label class="flex items-center gap-2 text-sm">
                        <Toggle
                            attr:data-helpdesk-mine-only=""
                            prop:checked=move || filter.get().mine_only
                            on:change=move |_| filter.update(|f| f.mine_only = !f.mine_only)
                        />
                        {move || texts.get().mine_only}
                    </label>
                </Show>
            </FilterBar>
            <Show
                when=move || matches!(phase.get(), ListPhase::Ready) && !tickets.get().is_empty()
                fallback=move || {
                    // `EntityColumn<HelpdeskTicket>` holds `Rc<dyn Fn>` renderers, which
                    // is not `Send`; a bare `{move || ...}` reactive child requires its
                    // output to be `Send` regardless of `.into_any()`, so the table can
                    // only ever live in `Show`'s own non-Send-bound children slot (as
                    // `snapshot_table_page.rs` does), never in this fallback closure.
                    if matches!(phase.get(), ListPhase::Loading) && tickets.get().is_empty() {
                        view! {
                            <div data-helpdesk-state="loading">
                                <PageStatePanel kind=PageStatePanelKind::InitialLoading />
                            </div>
                        }
                    } else if matches!(phase.get(), ListPhase::Error) {
                        view! {
                            <div data-helpdesk-state="error">
                                <PageStatePanel
                                    kind=PageStatePanelKind::InitialError
                                    detail=list_error
                                    on_retry=Callback::new(move |_| load_list())
                                />
                            </div>
                        }
                    } else {
                        view! {
                            <div data-helpdesk-state="empty">
                                <PageStatePanel kind=PageStatePanelKind::EmptyDataset />
                            </div>
                        }
                    }
                }
            >
                <div data-helpdesk-state="ready" data-helpdesk-table="">
                    <EntityTable
                        data=auto_filters.get_value().rows()
                        source_data=table_data
                        column_filters=auto_filters.get_value().filters()
                        columns=columns.get_value()
                        row_key=Rc::new(|t: &HelpdeskTicket| t.key.0.clone())
                        dataset_identity=Signal::derive(move || format!("helpdesk-{}", generation.get()))
                        on_row_activate=Callback::new(move |k: String| open_detail(TicketKey(k)))
                        // The gear-glyph chooser trigger: the compact presentation of
                        // the standard record-list table (Office op-ulgfu).
                        column_chooser_trigger=Signal::stored(EntityColumnChooserTrigger::Icon)
                        // The "add a record" affordance of the same standard table.
                        // A Helpdesk ticket is filed through the whole New Request
                        // dialog (kind picker, attachment capture, rate-limit
                        // handling), not an inline draft row, so the `+` lives in the
                        // caller-rendered toolbar slot rather than in
                        // `EntityDraftRow`'s built-in inline `+`.
                        toolbar_actions=Box::new(move || {
                            view! {
                                <Button
                                    color=ButtonColor::Primary
                                    size=ButtonSize::Sm
                                    // `gap-2` (8px) replaces daisyUI's stock
                                    // `.btn` `gap: .375rem` (6px, off the
                                    // canonical spacing scale): this button has
                                    // an icon AND a label, so the gap is live
                                    // spacing, not the dead CSS the toolbar
                                    // chooser pins `gap-0` against.
                                    class="gap-2"
                                    on_click=Callback::new(move |_| dialog_open.set(true))
                                    attr:data-helpdesk-add-row=""
                                >
                                    <Icon name="plus".to_owned() size=IconSize::XSmall />
                                    {move || texts.get().new_request}
                                </Button>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Show>
            <TicketDetailDrawer
                open=Signal::derive(move || selected.get().is_some())
                detail=detail
                meta=meta
                capabilities=caps
                texts=texts
                now_ms=now
                feedback=feedback
                on_close=Callback::new(move |_| close_detail())
                on_action=on_action
            />
            <NewRequestDialog
                open=dialog_open
                meta=meta
                context=context
                texts=texts
                attachments=attachments
                on_submit=on_submit
            />
            <Show when=move || toast.get().is_some()>
                <Toast attr:data-helpdesk-toast="">
                    <div class="alert alert-success">
                        {move || {
                            let t = toast.get();
                            let tx = texts.get();
                            t.map(|t| {
                                let msg = tx.filed.replace("{key}", &t.key.0);
                                match t.url {
                                    Some(u) => {
                                        view! {
                                            <a class="link" href=u target="_blank" rel="noopener">
                                                {msg}
                                            </a>
                                        }
                                            .into_any()
                                    }
                                    None => view! { <span>{msg}</span> }.into_any(),
                                }
                            })
                        }}
                        <Button size=ButtonSize::Xs on_click=Callback::new(move |_| toast.set(None))>
                            "×"
                        </Button>
                    </div>
                </Toast>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(key: &str, updated: i64) -> HelpdeskTicket {
        HelpdeskTicket {
            key: TicketKey(key.into()),
            summary: key.into(),
            kind: TicketKind::Bug,
            status: TicketStatus {
                id: "1".into(),
                name: "B".into(),
                category: StatusCategory::New,
            },
            priority: TicketPriority::Medium,
            assignee: None,
            requester: None,
            created_at_ms: 0,
            updated_at_ms: updated,
            comment_count: 0,
            attachment_count: 0,
            url: None,
        }
    }

    #[test]
    fn replace_by_key_keeps_order_and_upserts() {
        let mut v = vec![t("A", 1), t("B", 2)];
        replace_by_key(&mut v, t("B", 9));
        assert_eq!(v[1].updated_at_ms, 9);
        replace_by_key(&mut v, t("C", 3));
        assert_eq!(v[0].key.0, "C", "unknown keys are prepended");
    }

    #[test]
    fn sorted_view_is_updated_desc() {
        let v = vec![t("A", 1), t("B", 3), t("C", 2)];
        let s = sorted_desc(&v);
        assert_eq!(
            s.iter().map(|t| t.key.0.as_str()).collect::<Vec<_>>(),
            ["B", "C", "A"]
        );
    }
}
