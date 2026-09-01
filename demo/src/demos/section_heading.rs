use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{HeadingLevel, SectionHeading, SectionHeadingStatusPlacement};

#[component]
pub fn SectionHeadingDemo() -> impl IntoView {
    let (french, set_french) = signal(false);

    let localized_eyebrow = Signal::derive(move || {
        if french.get() {
            "APERCU DU DOSSIER".to_string()
        } else {
            "CASE OVERVIEW".to_string()
        }
    });
    let localized_title = Signal::derive(move || {
        if french.get() {
            "Resume du dossier".to_string()
        } else {
            "Case summary".to_string()
        }
    });
    let localized_description = Signal::derive(move || {
        if french.get() {
            "Les dernieres mises a jour et le statut de facturation apparaissent ci-dessous."
                .to_string()
        } else {
            "The latest updates and billing status appear below.".to_string()
        }
    });

    view! {
        <ContentLayout
            title="Section Heading"
            description="A small opinionated eyebrow/title/description composition for content beneath a PageHeader, with optional status and action slots."
        >
            <Section title="Plain">
                <div class="w-full max-w-2xl" data-testid="section-heading-plain">
                    <SectionHeading
                        id="section-heading-plain-heading"
                        eyebrow="STAFF"
                        title="Team roster"
                        description="Everyone currently assigned to this office."
                    />
                </div>
            </Section>

            <Section title="With status">
                <div class="w-full max-w-2xl" data-testid="section-heading-status">
                    <SectionHeading
                        id="section-heading-status-heading"
                        eyebrow="INTEGRATIONS"
                        title="Sync status"
                        level=HeadingLevel::H3
                        status=Box::new(|| {
                            view! {
                                <Badge color=BadgeColor::Success attr:data-testid="section-heading-status-badge">
                                    "Up to date"
                                </Badge>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="With actions">
                <div class="w-full max-w-2xl" data-testid="section-heading-action">
                    <SectionHeading
                        id="section-heading-action-heading"
                        eyebrow="STAFF"
                        title="Team roster"
                        description="Everyone currently assigned to this office."
                        actions=Box::new(|| {
                            view! {
                                <div class="flex gap-2">
                                    <Button
                                        size=ButtonSize::Sm
                                        attr:data-testid="section-heading-action-invite"
                                    >
                                        "Invite"
                                    </Button>
                                    <Button
                                        size=ButtonSize::Sm
                                        color=ButtonColor::Primary
                                        attr:data-testid="section-heading-action-add"
                                    >
                                        "Add member"
                                    </Button>
                                </div>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="Long copy, status and actions together">
                <div class="w-full max-w-2xl" data-testid="section-heading-long-copy">
                    <SectionHeading
                        id="section-heading-long-copy-heading"
                        eyebrow="BILLING"
                        title="Outstanding invoices across every active matter this quarter"
                        description="This figure includes invoices issued in the last ninety days across every practice group, net of write-offs, pending trust transfers, and disputed line items still under review by the billing coordinator."
                        status=Box::new(|| {
                            view! {
                                <Badge color=BadgeColor::Warning>"3 overdue"</Badge>
                            }
                            .into_any()
                        })
                        actions=Box::new(|| {
                            view! {
                                <Button size=ButtonSize::Sm>"Export"</Button>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="Trailing status">
                <div class="w-full max-w-2xl" data-testid="section-heading-trailing-status">
                    <SectionHeading
                        id="section-heading-trailing-status-heading"
                        eyebrow="COMMITMENTS"
                        title="Commitments"
                        status_placement=SectionHeadingStatusPlacement::Trailing
                        status=Box::new(|| {
                            view! {
                                <span
                                    class="ld-text-caption text-base-content/75"
                                    attr:data-testid="section-heading-trailing-status-text"
                                >
                                    "Provisional -- pending measure review"
                                </span>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="Trailing status with actions">
                <div class="w-full max-w-2xl" data-testid="section-heading-trailing-status-action">
                    <SectionHeading
                        id="section-heading-trailing-status-action-heading"
                        eyebrow="RESULTS"
                        title="Results produced"
                        status_placement=SectionHeadingStatusPlacement::Trailing
                        status=Box::new(|| {
                            view! {
                                <span
                                    class="ld-text-caption text-base-content/75"
                                    attr:data-testid="section-heading-trailing-status-action-text"
                                >
                                    "Provisional -- pending measure review"
                                </span>
                            }
                            .into_any()
                        })
                        actions=Box::new(|| {
                            view! {
                                <Button
                                    size=ButtonSize::Sm
                                    attr:data-testid="section-heading-trailing-status-action-export"
                                >
                                    "Export"
                                </Button>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="Trailing status, long title">
                <div class="w-full max-w-2xl" data-testid="section-heading-trailing-long-title">
                    <SectionHeading
                        id="section-heading-trailing-long-title-heading"
                        title="Outstanding invoices across every active matter this quarter"
                        status_placement=SectionHeadingStatusPlacement::Trailing
                        status=Box::new(|| {
                            view! {
                                <span
                                    class="ld-text-caption text-base-content/75"
                                    attr:data-testid="section-heading-trailing-long-title-text"
                                >
                                    "Provisional -- pending measure review"
                                </span>
                            }
                            .into_any()
                        })
                    />
                </div>
            </Section>

            <Section title="Localized (reactive)">
                <div class="w-full max-w-2xl flex flex-col gap-2" data-testid="section-heading-localized">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="section-heading-localized-toggle"
                        on:click=move |_| set_french.update(|v| *v = !*v)
                    >
                        "Toggle language"
                    </Button>
                    <SectionHeading
                        id="section-heading-localized-heading"
                        eyebrow=localized_eyebrow
                        title=localized_title
                        description=localized_description
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
