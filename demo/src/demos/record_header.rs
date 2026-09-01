//! Showcase for the `RecordHeader` pattern (ldui-9d0q).
//!
//! Covers the two originating consumer shapes -- Office Account and No-Hire
//! Detail -- with and without avatar, links, secondary badges, and 2-4 glyph
//! actions, plus every quick-action state, every presentation state, and the
//! full `PageHeader` / `RecordHeader` / controlled `TabSet` composition.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{
    PageHeader, PageHeaderDivider, RecordActionFeedback, RecordAvatar, RecordBadge, RecordHeader,
    RecordHeaderState, RecordMetaItem, RecordQuickAction, RecordStatus, RecordStatusTone,
};

/// Account-style metadata: owner, a linked matter, and a linked external
/// portal record.
fn account_metadata() -> Vec<RecordMetaItem> {
    vec![
        RecordMetaItem::new("owner", "Owner", "Maria Gonzalez").icon("user"),
        RecordMetaItem::new("matter", "Matter", "MAT-1023")
            .link("#matter-1023")
            .icon("file-text"),
        RecordMetaItem::new("portal", "Portal", "ACC-2201")
            .link("#external-portal")
            .external()
            .icon("external-link"),
        RecordMetaItem::new("opened", "Opened", "12 Aug 2026").icon("calendar"),
    ]
}

/// No-Hire-style metadata: a candidate reference, the decision date, and the
/// reviewing office. Deliberately link-free -- the shape a consumer reaches
/// for when nothing in the row is navigable.
fn no_hire_metadata() -> Vec<RecordMetaItem> {
    vec![
        RecordMetaItem::new("ref", "Reference", "NH-4417"),
        RecordMetaItem::new("decided", "Decided", "03 Jul 2026"),
        RecordMetaItem::new("office", "Office", "Chicago"),
    ]
}

#[component]
pub fn RecordHeaderDemo() -> impl IntoView {
    let (last_action, set_last_action) = signal(String::new());
    let on_action = Callback::new(move |id: String| set_last_action.set(id));

    let (selected_tab, set_selected_tab) = signal("overview".to_string());
    let select_tab = Callback::new(move |key: String| set_selected_tab.set(key));

    let (state, set_state) = signal(RecordHeaderState::Ready);

    view! {
        <ContentLayout
            title="Record Header"
            description="Opinionated record identity row: avatar, title, compact metadata, primary status, secondary badges, and glyph quick actions in one responsive line."
        >
            <Section title="Account style -- avatar, links, badges, four glyph actions">
                <div class="w-full" data-testid="record-header-account">
                    <RecordHeader
                        id="record-header-account-title"
                        title="Northwind Logistics"
                        avatar=Some(RecordAvatar::new("Northwind Logistics").initials("NW"))
                        metadata=account_metadata()
                        status=Some(
                            RecordStatus::new("Active")
                                .tone(RecordStatusTone::Success)
                                .detail("Renewed 12 Aug 2026"),
                        )
                        badges=vec![
                            RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info),
                            RecordBadge::new("contract", "Contract"),
                        ]
                        actions=vec![
                            RecordQuickAction::new("call", "phone", "Call account"),
                            RecordQuickAction::new("email", "mail", "Email account"),
                            RecordQuickAction::new("portal", "external-link", "Open in portal")
                                .link("#external-portal")
                                .external(),
                            RecordQuickAction::new("archive", "trash", "Archive account"),
                        ]
                        on_action=on_action
                    />
                </div>
            </Section>

            <Section title="No-Hire style -- no avatar, no links, no badges, two glyph actions">
                <div class="w-full" data-testid="record-header-no-hire">
                    <RecordHeader
                        id="record-header-no-hire-title"
                        title="Jordan Alvarez"
                        metadata=no_hire_metadata()
                        status=Some(
                            RecordStatus::new("Do not hire")
                                .tone(RecordStatusTone::Error)
                                .detail("Decision is final until 03 Jul 2028"),
                        )
                        actions=vec![
                            RecordQuickAction::new("notes", "file-text", "Open decision notes"),
                            RecordQuickAction::new("appeal", "reply", "Record an appeal"),
                        ]
                        on_action=on_action
                    />
                </div>
            </Section>

            <Section title="Neutral status, no glyph -- the label alone carries the meaning">
                <div class="w-full" data-testid="record-header-neutral">
                    <RecordHeader
                        id="record-header-neutral-title"
                        title="Ashford Holdings"
                        metadata=vec![RecordMetaItem::new("stage", "Stage", "Intake")]
                        status=Some(RecordStatus::new("Draft"))
                        badges=vec![
                            RecordBadge::new("watch", "Watchlist").tone(RecordStatusTone::Warning),
                        ]
                        actions=vec![RecordQuickAction::new("edit", "pencil", "Edit record")]
                        on_action=on_action
                    />
                </div>
            </Section>

            <Section title="Long identity text truncates -- the status/actions edge never moves">
                <div class="w-full" data-testid="record-header-long">
                    <RecordHeader
                        id="record-header-long-title"
                        title="Consolidated Transcontinental Freight and Warehousing Cooperative of the Upper Midwest"
                        avatar=Some(RecordAvatar::new("Consolidated Transcontinental Freight"))
                        metadata=vec![
                            RecordMetaItem::new(
                                    "matter",
                                    "Matter",
                                    "MAT-1023-CONSOLIDATED-TRANSCONTINENTAL-FREIGHT-AND-WAREHOUSING",
                                )
                                .link("#matter-1023"),
                            RecordMetaItem::new("owner", "Owner", "Maria Gonzalez"),
                        ]
                        status=Some(
                            RecordStatus::new("Under review").tone(RecordStatusTone::Warning),
                        )
                        badges=vec![RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info)]
                        actions=vec![
                            RecordQuickAction::new("call", "phone", "Call account"),
                            RecordQuickAction::new("email", "mail", "Email account"),
                            RecordQuickAction::new("archive", "trash", "Archive account"),
                        ]
                        on_action=on_action
                    />
                </div>
            </Section>

            <Section title="Action states -- disabled with reason, pending, and keyed feedback">
                <div class="w-full" data-testid="record-header-action-states">
                    <RecordHeader
                        id="record-header-action-states-title"
                        title="Ridgeline Manufacturing"
                        avatar=Some(RecordAvatar::new("Ridgeline Manufacturing"))
                        metadata=vec![
                            RecordMetaItem::new("owner", "Owner", "Priya Patel"),
                            RecordMetaItem::new("matter", "Matter", "MAT-2210").link("#matter-2210"),
                        ]
                        status=Some(RecordStatus::new("Active").tone(RecordStatusTone::Success))
                        actions=vec![
                            RecordQuickAction::new("call", "phone", "Call account")
                                .feedback(
                                    RecordActionFeedback::new("Call connected at 09:14")
                                        .tone(RecordStatusTone::Success),
                                ),
                            RecordQuickAction::new("email", "mail", "Email account").pending(),
                            RecordQuickAction::new("archive", "trash", "Archive account")
                                .disabled("Locked while a compliance review is open"),
                            RecordQuickAction::new("delete", "close", "Delete account")
                                .disabled("Only a firm administrator can delete an account")
                                .feedback(
                                    RecordActionFeedback::new(
                                            "Deletion was requested and refused on 21 Aug",
                                        )
                                        .tone(RecordStatusTone::Warning),
                                ),
                        ]
                        on_action=on_action
                    />
                </div>
            </Section>

            <Section title="Presentation states -- loading, retained, and unavailable">
                <div class="w-full flex flex-col gap-4" data-testid="record-header-states">
                    <div class="flex flex-wrap items-center gap-2">
                        <Button
                            size=ButtonSize::Sm
                            attr:data-testid="record-header-state-ready"
                            on:click=move |_| set_state.set(RecordHeaderState::Ready)
                        >
                            "Ready"
                        </Button>
                        <Button
                            size=ButtonSize::Sm
                            attr:data-testid="record-header-state-loading"
                            on:click=move |_| set_state.set(RecordHeaderState::Loading)
                        >
                            "Loading"
                        </Button>
                        <Button
                            size=ButtonSize::Sm
                            attr:data-testid="record-header-state-retained"
                            on:click=move |_| set_state.set(RecordHeaderState::Retained)
                        >
                            "Retained"
                        </Button>
                        <Button
                            size=ButtonSize::Sm
                            attr:data-testid="record-header-state-unavailable"
                            on:click=move |_| set_state.set(RecordHeaderState::Unavailable)
                        >
                            "Unavailable"
                        </Button>
                    </div>
                    <RecordHeader
                        id="record-header-states-title"
                        title="Northwind Logistics"
                        avatar=Some(RecordAvatar::new("Northwind Logistics").initials("NW"))
                        metadata=account_metadata()
                        status=Some(RecordStatus::new("Active").tone(RecordStatusTone::Success))
                        badges=vec![RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info)]
                        actions=vec![
                            RecordQuickAction::new("call", "phone", "Call account"),
                            RecordQuickAction::new("email", "mail", "Email account"),
                        ]
                        on_action=on_action
                        state=state
                    />
                </div>
            </Section>

            <Section title="Full composition -- PageHeader above, controlled TabSet below">
                <div class="w-full flex flex-col gap-4" data-testid="record-header-composition">
                    <PageHeader
                        title="Accounts"
                        subtitle="Every account this office is responsible for."
                        divider=PageHeaderDivider::Hidden
                        back=Box::new(|| {
                            view! {
                                <LinkButton href="#accounts" style=ButtonStyle::Ghost size=ButtonSize::Sm>
                                    "Back to accounts"
                                </LinkButton>
                            }
                                .into_any()
                        })
                    />
                    <RecordHeader
                        id="record-header-composition-title"
                        title="Northwind Logistics"
                        avatar=Some(RecordAvatar::new("Northwind Logistics").initials("NW"))
                        metadata=account_metadata()
                        status=Some(RecordStatus::new("Active").tone(RecordStatusTone::Success))
                        badges=vec![RecordBadge::new("vip", "VIP").tone(RecordStatusTone::Info)]
                        actions=vec![
                            RecordQuickAction::new("call", "phone", "Call account"),
                            RecordQuickAction::new("email", "mail", "Email account"),
                            RecordQuickAction::new("archive", "trash", "Archive account")
                                .disabled("Locked while a compliance review is open"),
                        ]
                        on_action=on_action
                    />
                    <TabSet
                        id="record-header-composition-tabs"
                        label="Account sections"
                        selected_key=selected_tab
                        on_select=select_tab
                    >
                        <Tabs variant=TabVariant::Lift>
                            <Tab tab_key="overview">"Overview"</Tab>
                            <Tab tab_key="matters">"Matters"</Tab>
                            <Tab tab_key="activity">"Activity"</Tab>
                        </Tabs>
                        <TabPanel tab_key="overview" class="bg-base-200 p-4 rounded-box">
                            <p>"Account overview lives here. The header above owns identity only."</p>
                        </TabPanel>
                        <TabPanel tab_key="matters" class="bg-base-200 p-4 rounded-box">
                            <p>"Matters for this account."</p>
                        </TabPanel>
                        <TabPanel tab_key="activity" class="bg-base-200 p-4 rounded-box">
                            <p>"Recent activity for this account."</p>
                        </TabPanel>
                    </TabSet>
                </div>
            </Section>

            <Section title="Last activated action id (consumer-owned)">
                <p data-testid="record-header-last-action" class="ld-text-body text-base-content/75">
                    {move || {
                        let id = last_action.get();
                        if id.is_empty() { "(none yet)".to_string() } else { id }
                    }}
                </p>
            </Section>
        </ContentLayout>
    }
}
