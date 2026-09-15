//! The ticket detail drawer: header, meta, description, attachments,
//! comments and (support only) the triage selects.

use super::model::*;
use super::state::{RoleCapabilities, relative_age};
use super::texts::HelpdeskTexts;
use crate::components::{
    Badge, BadgeColor, Button, ButtonColor, ButtonSize, ButtonStyle, Drawer, DrawerContent,
    DrawerOverlay, DrawerPlacement, DrawerSide, DrawerToggle, Select, SelectOption, Textarea,
};
use crate::patterns::{
    RecordBadge, RecordHeader, RecordMetaItem, RecordQuickAction, RecordStatus, RecordStatusTone,
};
use leptos::ev;
use leptos::prelude::*;

/// A triage action requested from the drawer. The composite owns the
/// backend call and the optimistic-then-authoritative update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriageAction {
    /// Move the ticket to a new status.
    Transition {
        /// The requested status id.
        to_status_id: String,
    },
    /// Set or clear the assignee.
    Assign {
        /// The requested assignee, or `None` to unassign.
        assignee_id: Option<String>,
    },
    /// Set the priority.
    SetPriority {
        /// The requested priority id.
        priority_id: String,
    },
    /// Add a comment.
    Comment {
        /// The comment body.
        body: String,
    },
}

/// The status badge's tone, by category.
pub(crate) fn status_tone(c: StatusCategory) -> RecordStatusTone {
    match c {
        StatusCategory::New => RecordStatusTone::Info,
        StatusCategory::InProgress => RecordStatusTone::Warning,
        StatusCategory::Done => RecordStatusTone::Success,
    }
}

/// The priority badge's color.
pub(crate) fn priority_color(p: TicketPriority) -> BadgeColor {
    match p {
        TicketPriority::Highest | TicketPriority::High => BadgeColor::Error,
        TicketPriority::Medium => BadgeColor::Warning,
        TicketPriority::Low | TicketPriority::Lowest => BadgeColor::Neutral,
        TicketPriority::Unset => BadgeColor::Default,
    }
}

/// The header's meta rows: requester, created, updated.
pub(crate) fn meta_items(
    t: &HelpdeskTicket,
    texts: &HelpdeskTexts,
    now_ms: i64,
) -> Vec<RecordMetaItem> {
    vec![
        RecordMetaItem::new(
            "requester",
            texts.requester.clone(),
            t.requester
                .as_ref()
                .map(|p| p.display_name.clone())
                .unwrap_or_else(|| texts.unassigned.clone()),
        ),
        RecordMetaItem::new(
            "created",
            texts.created.clone(),
            relative_age(now_ms, t.created_at_ms),
        ),
        RecordMetaItem::new(
            "updated",
            texts.updated.clone(),
            relative_age(now_ms, t.updated_at_ms),
        ),
    ]
}

/// The ticket detail drawer.
#[component]
pub fn TicketDetailDrawer(
    /// Whether the drawer is open.
    #[prop(into)]
    open: Signal<bool>,
    /// The loaded detail, or `None` while it is loading.
    #[prop(into)]
    detail: Signal<Option<TicketDetail>>,
    /// The host's current metadata.
    #[prop(into)]
    meta: Signal<Option<HelpdeskMeta>>,
    /// The active role's capabilities.
    #[prop(into)]
    capabilities: Signal<RoleCapabilities>,
    /// All rendered copy.
    #[prop(optional, into, default = Signal::stored(HelpdeskTexts::default()))]
    texts: Signal<HelpdeskTexts>,
    /// The current time, for age formatting.
    #[prop(into)]
    now_ms: Signal<i64>,
    /// Feedback for the last triage action on this ticket: `(is_error, message)`.
    #[prop(into)]
    feedback: Signal<Option<(bool, String)>>,
    /// Called when the drawer should close.
    on_close: Callback<()>,
    /// Called with a requested triage action.
    on_action: Callback<TriageAction>,
) -> impl IntoView {
    let comment_draft = RwSignal::new(String::new());
    let ticket = Signal::derive(move || detail.get().map(|d| d.ticket));
    let triage = Signal::derive(move || capabilities.get().triage);

    let header = move || {
        let t = ticket.get()?;
        let tx = texts.get();
        let mut actions = Vec::new();
        if let Some(url) = &t.url {
            actions.push(
                RecordQuickAction::new("open-jira", "external-link", tx.open_in_jira.clone())
                    .link(url.clone())
                    .external(),
            );
        }
        Some(view! {
            <RecordHeader
                title=t.summary.clone()
                metadata=meta_items(&t, &tx, now_ms.get())
                status=Some(RecordStatus::new(t.status.name.clone()).tone(status_tone(t.status.category)))
                badges=vec![
                    // `.tone(Info)` on both, never the default/`Neutral` tone: `Badge`'s
                    // `Soft` style paired with `Neutral` measures 1.22:1 in the active
                    // theme (axe `color-contrast`, needs 4.5:1) -- near-invisible, and
                    // pre-existing in `RecordHeader::render_badge`, not new here. Info
                    // is the contained fix; see the composite's design doc for the note.
                    RecordBadge::new("key", t.key.0.clone()).tone(RecordStatusTone::Info),
                    RecordBadge::new("kind", tx.kind_name(&t.kind)).tone(RecordStatusTone::Info),
                ]
                actions=actions
                attr:data-helpdesk-drawer-header=""
            />
        })
    };

    let submit_comment = move || {
        let body = comment_draft.get_untracked();
        if body.trim().is_empty() {
            return;
        }
        on_action.run(TriageAction::Comment { body });
        comment_draft.set(String::new());
    };

    view! {
        <Drawer
            placement=DrawerPlacement::End
            open=open
            attr:data-helpdesk-drawer=""
            attr:data-helpdesk-drawer-open=move || open.get().to_string()
        >
            // Pure CSS plumbing (daisyUI's checkbox-driven drawer visibility
            // mechanism) -- zero-size, never meant to be perceived directly.
            // Without this it is an unlabelled, focusable checkbox (axe
            // `label`, critical); the real controls are the row activation
            // and the Close button.
            <DrawerToggle
                id="helpdesk-drawer-toggle"
                checked=open
                attr:aria-hidden="true"
                attr:tabindex="-1"
            />
            <DrawerContent>""</DrawerContent>
            <DrawerSide class="z-40">
                <DrawerOverlay on:click=move |_| on_close.run(()) />
                <aside
                    class="flex h-full w-full max-w-xl flex-col gap-4 overflow-y-auto bg-base-100 p-6"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || texts.get().drawer_label
                    on:keydown=move |ev: ev::KeyboardEvent| {
                        if ev.key() == "Escape" {
                            on_close.run(());
                        }
                    }
                >
                    <div class="flex justify-end">
                        <Button
                            size=ButtonSize::Sm
                            style=ButtonStyle::Ghost
                            on_click=Callback::new(move |_| on_close.run(()))
                            attr:data-helpdesk-drawer-close=""
                        >
                            {move || texts.get().close}
                        </Button>
                    </div>
                    {header}
                    <Show when=move || triage.get()>
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3" data-helpdesk-triage="">
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().status}</span>
                                <Select
                                    value=Signal::derive(move || {
                                        ticket.get().map(|t| t.status.id).unwrap_or_default()
                                    })
                                    on_change=Callback::new(move |id: String| {
                                        on_action.run(TriageAction::Transition { to_status_id: id })
                                    })
                                    attr:data-helpdesk-transition=""
                                >
                                    <For each=move || meta.get().map(|m| m.statuses).unwrap_or_default() key=|s| s.id.clone() let:s>
                                        <SelectOption attr:value=s.id.clone()>{s.name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().assignee}</span>
                                <Select
                                    value=Signal::derive(move || {
                                        ticket.get().and_then(|t| t.assignee.map(|p| p.id)).unwrap_or_default()
                                    })
                                    on_change=Callback::new(move |id: String| {
                                        on_action.run(TriageAction::Assign { assignee_id: (!id.is_empty()).then_some(id) })
                                    })
                                    attr:data-helpdesk-assign=""
                                >
                                    <SelectOption attr:value="">{move || texts.get().unassigned}</SelectOption>
                                    <For each=move || meta.get().map(|m| m.assignable).unwrap_or_default() key=|p| p.id.clone() let:p>
                                        <SelectOption attr:value=p.id.clone()>{p.display_name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                            <label class="flex flex-col gap-2">
                                <span class="text-sm font-medium">{move || texts.get().priority}</span>
                                <Select
                                    value=Signal::derive(move || {
                                        let current = ticket.get().map(|t| t.priority);
                                        meta.get()
                                            .and_then(|m| m.priorities.into_iter().find(|p| Some(p.priority) == current))
                                            .map(|p| p.id)
                                            .unwrap_or_default()
                                    })
                                    on_change=Callback::new(move |id: String| {
                                        on_action.run(TriageAction::SetPriority { priority_id: id })
                                    })
                                    attr:data-helpdesk-priority=""
                                >
                                    <For each=move || meta.get().map(|m| m.priorities).unwrap_or_default() key=|p| p.id.clone() let:p>
                                        <SelectOption attr:value=p.id.clone()>{p.name.clone()}</SelectOption>
                                    </For>
                                </Select>
                            </label>
                        </div>
                    </Show>
                    <Show when=move || !triage.get()>
                        <dl class="grid grid-cols-2 gap-2 text-sm" data-helpdesk-readonly-meta="">
                            <dt class="text-base-content/75">{move || texts.get().priority}</dt>
                            <dd>
                                <Badge color=priority_color(ticket.get().map(|t| t.priority).unwrap_or(TicketPriority::Unset))>
                                    {move || texts.get().priority_name(ticket.get().map(|t| t.priority).unwrap_or(TicketPriority::Unset))}
                                </Badge>
                            </dd>
                            <dt class="text-base-content/75">{move || texts.get().assignee}</dt>
                            <dd>
                                {move || {
                                    ticket.get().and_then(|t| t.assignee.map(|p| p.display_name)).unwrap_or_else(|| texts.get().unassigned)
                                }}
                            </dd>
                        </dl>
                    </Show>
                    <p
                        class="text-sm"
                        role="status"
                        aria-live="polite"
                        class:text-error=move || feedback.get().is_some_and(|f| f.0)
                        class:text-success=move || feedback.get().is_some_and(|f| !f.0)
                        data-helpdesk-action-feedback=""
                        data-helpdesk-action-feedback-state=move || {
                            feedback.get().map(|f| if f.0 { "error" } else { "success" }).unwrap_or("idle")
                        }
                    >
                        {move || feedback.get().map(|f| f.1).unwrap_or_default()}
                    </p>
                    <section class="flex flex-col gap-2">
                        <h3 class="ld-text-subtitle">{move || texts.get().description}</h3>
                        <p class="whitespace-pre-wrap text-sm" data-helpdesk-description-text="">
                            {move || detail.get().map(|d| d.description).unwrap_or_default()}
                        </p>
                    </section>
                    <Show when=move || detail.get().is_some_and(|d| !d.attachments.is_empty())>
                        <section class="flex flex-col gap-2">
                            <h3 class="ld-text-subtitle">{move || texts.get().attachments}</h3>
                            <ul class="flex flex-wrap gap-2">
                                <For each=move || detail.get().map(|d| d.attachments).unwrap_or_default() key=|a| a.id.clone() let:a>
                                    <li data-helpdesk-attachment="">
                                        <a href=a.url.clone() target="_blank" rel="noopener" class="link text-sm">
                                            <img
                                                src=a.thumbnail_url.clone().unwrap_or_else(|| a.url.clone())
                                                alt=a.filename.clone()
                                                class="h-16 w-16 rounded object-cover"
                                            />
                                            <span class="block max-w-16 truncate">{a.filename.clone()}</span>
                                        </a>
                                    </li>
                                </For>
                            </ul>
                        </section>
                    </Show>
                    <section class="flex flex-col gap-2">
                        <h3 class="ld-text-subtitle">{move || texts.get().comments}</h3>
                        <ol class="flex flex-col gap-2" data-helpdesk-comments="">
                            <For each=move || detail.get().map(|d| d.comments).unwrap_or_default() key=|c| c.id.clone() let:c>
                                <li class="rounded-box bg-base-200 p-3 text-sm" data-helpdesk-comment="">
                                    <div class="mb-1 flex gap-2 text-base-content/75">
                                        <span class="font-medium">{c.author.display_name.clone()}</span>
                                        <span>{move || relative_age(now_ms.get(), c.created_at_ms)}</span>
                                    </div>
                                    <p class="whitespace-pre-wrap">{c.body.clone()}</p>
                                </li>
                            </For>
                        </ol>
                        <Show when=move || detail.get().is_some_and(|d| d.comments.is_empty())>
                            <p class="text-sm text-base-content/75">{move || texts.get().no_comments}</p>
                        </Show>
                        <Show when=move || capabilities.get().comment>
                            <div class="flex flex-col gap-2">
                                <Textarea
                                    value=comment_draft
                                    rows=Some(3)
                                    placeholder=Signal::derive(move || texts.get().comment_placeholder)
                                    on_input=Callback::new(move |v: String| comment_draft.set(v))
                                    attr:data-helpdesk-comment-input=""
                                />
                                <div class="flex justify-end">
                                    <Button
                                        color=ButtonColor::Primary
                                        size=ButtonSize::Sm
                                        disabled=Signal::derive(move || comment_draft.get().trim().is_empty())
                                        on_click=Callback::new(move |_| submit_comment())
                                        attr:data-helpdesk-comment-submit=""
                                    >
                                        {move || texts.get().add_comment}
                                    </Button>
                                </div>
                            </div>
                        </Show>
                    </section>
                </aside>
            </DrawerSide>
        </Drawer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_status_tone_follows_category() {
        assert_eq!(status_tone(StatusCategory::New), RecordStatusTone::Info);
        assert_eq!(
            status_tone(StatusCategory::InProgress),
            RecordStatusTone::Warning
        );
        assert_eq!(status_tone(StatusCategory::Done), RecordStatusTone::Success);
    }

    #[test]
    fn meta_items_list_requester_created_updated() {
        let t = HelpdeskTexts::default();
        let ticket = HelpdeskTicket {
            key: TicketKey("OF-1".into()),
            summary: "s".into(),
            kind: TicketKind::Bug,
            status: TicketStatus {
                id: "1".into(),
                name: "Backlog".into(),
                category: StatusCategory::New,
            },
            priority: TicketPriority::High,
            assignee: None,
            requester: Some(Person::new("w", "Chris Forbes")),
            created_at_ms: 0,
            updated_at_ms: 60_000,
            comment_count: 0,
            attachment_count: 0,
            url: None,
        };
        let items = meta_items(&ticket, &t, 120_000);
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, ["requester", "created", "updated"]);
        assert_eq!(items[0].value, "Chris Forbes");
        assert_eq!(items[1].value, "2m");
        assert_eq!(items[2].value, "1m");
    }
}
