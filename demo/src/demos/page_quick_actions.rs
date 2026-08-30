use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{
    PageHeader, PageHeaderDivider, PageQuickActionContent, PageQuickActionLabelVisibility,
    PageQuickActions,
};

/// Seven cross-surface quick actions -- the exact count the originating
/// consumer report overflowed with raw text-only submit buttons.
#[component]
fn SevenActions(
    #[prop(optional, into)] label_visibility: PageQuickActionLabelVisibility,
) -> impl IntoView {
    view! {
        <PageQuickActions label="Case actions">
            <Button style=ButtonStyle::Outline size=ButtonSize::Sm color=ButtonColor::Primary>
                <PageQuickActionContent icon="plus" label="New matter" label_visibility=label_visibility />
            </Button>
            <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="search" label="Search" label_visibility=label_visibility />
            </Button>
            <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="filter" label="Filter" label_visibility=label_visibility />
            </Button>
            <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="refresh" label="Refresh" label_visibility=label_visibility />
            </Button>
            <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="upload" label="Export" label_visibility=label_visibility />
            </Button>
            <LinkButton href="#reports" style=ButtonStyle::Outline size=ButtonSize::Sm>
                <PageQuickActionContent icon="file-text" label="Reports" label_visibility=label_visibility />
            </LinkButton>
            <form action="/office/launch" method="post" target="_blank">
                <input type="hidden" name="doc_id" value="42" />
                <Button
                    button_type=ButtonType::Submit
                    style=ButtonStyle::Outline
                    size=ButtonSize::Sm
                    color=ButtonColor::Secondary
                >
                    <PageQuickActionContent
                        icon="external-link"
                        label="Open in Office"
                        label_visibility=label_visibility
                    />
                </Button>
            </form>
        </PageQuickActions>
    }
}

#[component]
pub fn PageQuickActionsDemo() -> impl IntoView {
    let (french, set_french) = signal(false);
    let localized_title = Signal::derive(move || {
        if french.get() {
            "Federation des dossiers actifs a travers tous les cabinets partenaires".to_string()
        } else {
            "Active matter federation across every partner office".to_string()
        }
    });
    let localized_subtitle = Signal::derive(move || {
        if french.get() {
            "Comprend les dossiers transferes, les dossiers en attente d'approbation et les dossiers archives en lecture seule.".to_string()
        } else {
            "Includes transferred matters, matters pending approval, and read-only archived matters.".to_string()
        }
    });

    view! {
        <ContentLayout
            title="Page Quick Actions"
            description="An opinionated, wrapping icon-action row for PageHeader's actions slot, plus a typed divider option for base-page compositions."
        >
            <Section title="Base page, no back button, seven actions (wide)">
                <div class="w-full" data-testid="page-quick-actions-base">
                    <PageHeader
                        title="Active matters"
                        subtitle="Everything currently open across the firm."
                        actions=Box::new(|| view! { <SevenActions /> }.into_any())
                    />
                </div>
            </Section>

            <Section title="Divider explicitly hidden (open base-page composition)">
                <div class="w-full" data-testid="page-quick-actions-no-divider">
                    <PageHeader
                        title="Coordinator workbench"
                        subtitle="Composed directly against an AppShell content area -- no header rule needed."
                        divider=PageHeaderDivider::Hidden
                        actions=Box::new(|| view! { <SevenActions /> }.into_any())
                    />
                </div>
            </Section>

            <Section title="Long localized title/subtitle beside seven actions">
                <div class="w-full flex flex-col gap-2" data-testid="page-quick-actions-localized">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="page-quick-actions-localized-toggle"
                        on:click=move |_| set_french.update(|v| *v = !*v)
                    >
                        "Toggle language"
                    </Button>
                    <PageHeader
                        title=localized_title
                        subtitle=localized_subtitle
                        actions=Box::new(|| view! { <SevenActions /> }.into_any())
                    />
                </div>
            </Section>

            <Section title="Compact overflow -- constrained width forces wrapping, never horizontal page overflow">
                <div
                    class="w-64 max-w-full border border-dashed border-base-300 p-2"
                    data-testid="page-quick-actions-compact"
                >
                    <PageHeader
                        title="Active matters"
                        subtitle="Everything currently open across the firm."
                        actions=Box::new(|| view! { <SevenActions /> }.into_any())
                    />
                </div>
            </Section>

            <Section title="Icon-only collapse below `sm` -- tooltip and accessible label preserved">
                <div class="w-64 max-w-full border border-dashed border-base-300 p-2" data-testid="page-quick-actions-collapse">
                    <PageHeader
                        title="Active matters"
                        actions=Box::new(|| {
                            view! {
                                <PageQuickActions label="Case actions">
                                    <Tooltip tip="New matter">
                                        <Button style=ButtonStyle::Outline size=ButtonSize::Sm color=ButtonColor::Primary>
                                            <PageQuickActionContent
                                                icon="plus"
                                                label="New matter"
                                                label_visibility=PageQuickActionLabelVisibility::CollapseBelowSm
                                            />
                                        </Button>
                                    </Tooltip>
                                    <Tooltip tip="Search">
                                        <Button style=ButtonStyle::Outline size=ButtonSize::Sm>
                                            <PageQuickActionContent
                                                icon="search"
                                                label="Search"
                                                label_visibility=PageQuickActionLabelVisibility::CollapseBelowSm
                                            />
                                        </Button>
                                    </Tooltip>
                                </PageQuickActions>
                            }
                                .into_any()
                        })
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
