//! Showcase for the `SelectableSummaryGroup` / `SelectableSummaryCard`
//! pattern (ldui-l5cw).
//!
//! Covers the originating consumer shape -- a fourteen-card diagnostic
//! check selector including measured zeroes and unmeasured checks -- plus
//! every status, a disabled card, controlled selection, a deliberately
//! narrow column (to show the container-query grid, not viewport
//! breakpoints), a group named by a visible heading instead of an
//! `aria-label`, and reactive localized copy.
//!
//! The check names here are the DEMO's, not the pattern's: the pattern
//! embeds no domain vocabulary.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::{Button, ButtonSize};
use leptos_daisyui_rs::patterns::{
    SelectableSummaryGroup, SelectableSummaryItem, SelectableSummaryStatus, SelectableSummaryTexts,
};

/// Fourteen compact diagnostic checks: measured counts across every status,
/// two measured ZEROES (a real, clean nothing), and two unmeasured checks
/// (no measurement at all) so the two are visibly and audibly different.
fn diagnostic_checks() -> Vec<SelectableSummaryItem> {
    vec![
        SelectableSummaryItem::new("duplicate-records", "Duplicate records", 12)
            .status(SelectableSummaryStatus::Warning)
            .description("Same identifier on more than one row"),
        SelectableSummaryItem::new("missing-email", "Missing email", 48)
            .status(SelectableSummaryStatus::Error)
            .description("Contactable party with no address"),
        SelectableSummaryItem::new("orphaned-rows", "Orphaned rows", 0)
            .status(SelectableSummaryStatus::Clean)
            .description("No parent record"),
        SelectableSummaryItem::new("stale-status", "Stale status", 7)
            .status(SelectableSummaryStatus::Warning),
        SelectableSummaryItem::new("invalid-phone", "Invalid phone", 3)
            .status(SelectableSummaryStatus::Warning),
        SelectableSummaryItem::new("future-dates", "Future dates", 0)
            .status(SelectableSummaryStatus::Clean),
        SelectableSummaryItem::new("unassigned-owner", "Unassigned owner", 21)
            .status(SelectableSummaryStatus::Error),
        SelectableSummaryItem::new("currency-mismatch", "Currency mismatch", 2)
            .status(SelectableSummaryStatus::Warning),
        SelectableSummaryItem::new("truncated-notes", "Truncated notes", 12_483)
            .count_text("12 483")
            .status(SelectableSummaryStatus::Warning)
            .description("Locale-grouped presentational count"),
        SelectableSummaryItem::new("timezone-drift", "Timezone drift", 5),
        SelectableSummaryItem::new("closed-with-tasks", "Closed with tasks", 9)
            .status(SelectableSummaryStatus::Warning),
        SelectableSummaryItem::unmeasured("feed-freshness", "Feed freshness")
            .description("Upstream feed did not report"),
        SelectableSummaryItem::unmeasured("archive-integrity", "Archive integrity"),
        SelectableSummaryItem::new("retired-codes", "Retired codes", 4)
            .status(SelectableSummaryStatus::Clean)
            .disabled(true)
            .description("Read-only in this environment"),
    ]
}

/// One card per status, plus a disabled card -- the state matrix a visual
/// audit needs on one row.
fn status_matrix() -> Vec<SelectableSummaryItem> {
    vec![
        SelectableSummaryItem::new("neutral", "Neutral", 5).description("No emphasis"),
        SelectableSummaryItem::new("clean", "Clean", 0)
            .status(SelectableSummaryStatus::Clean)
            .description("A measured zero"),
        SelectableSummaryItem::new("warning", "Warning", 12)
            .status(SelectableSummaryStatus::Warning),
        SelectableSummaryItem::new("error", "Error", 48).status(SelectableSummaryStatus::Error),
        SelectableSummaryItem::unmeasured("unavailable", "Unavailable")
            .description("No measurement at all"),
        SelectableSummaryItem::new("disabled", "Disabled", 3).disabled(true),
    ]
}

#[component]
pub fn SelectableSummaryDemo() -> impl IntoView {
    // Controlled selection: the page owns accepted truth, the group only
    // proposes.
    let (selected, set_selected) = signal(Some("duplicate-records".to_string()));
    let on_select = Callback::new(move |id: String| set_selected.set(Some(id)));

    let (matrix_selected, set_matrix_selected) = signal(None::<String>);
    let on_matrix_select = Callback::new(move |id: String| set_matrix_selected.set(Some(id)));

    let (narrow_selected, set_narrow_selected) = signal(Some("orphaned-rows".to_string()));
    let on_narrow_select = Callback::new(move |id: String| set_narrow_selected.set(Some(id)));

    let (labelled_selected, set_labelled_selected) = signal(None::<String>);
    let on_labelled_select = Callback::new(move |id: String| set_labelled_selected.set(Some(id)));

    let (french, set_french) = signal(false);
    let (localized_selected, set_localized_selected) = signal(Some("doublons".to_string()));
    let on_localized_select = Callback::new(move |id: String| set_localized_selected.set(Some(id)));

    let localized_items = Signal::derive(move || {
        if french.get() {
            vec![
                SelectableSummaryItem::new("doublons", "Doublons", 12)
                    .status(SelectableSummaryStatus::Warning),
                SelectableSummaryItem::new("orphelins", "Lignes orphelines", 0)
                    .status(SelectableSummaryStatus::Clean),
                SelectableSummaryItem::unmeasured("fraicheur", "Fraicheur du flux"),
            ]
        } else {
            vec![
                SelectableSummaryItem::new("doublons", "Duplicate records", 12)
                    .status(SelectableSummaryStatus::Warning),
                SelectableSummaryItem::new("orphelins", "Orphaned rows", 0)
                    .status(SelectableSummaryStatus::Clean),
                SelectableSummaryItem::unmeasured("fraicheur", "Feed freshness"),
            ]
        }
    });

    let localized_texts = Signal::derive(move || {
        if french.get() {
            SelectableSummaryTexts {
                unavailable: "Non mesure".to_string(),
                clean: "propre".to_string(),
                warning: "a verifier".to_string(),
                error: "en echec".to_string(),
            }
        } else {
            SelectableSummaryTexts::default()
        }
    });

    let localized_label = Signal::derive(move || {
        if french.get() {
            "Controles de qualite".to_string()
        } else {
            "Data quality checks".to_string()
        }
    });

    let selected_readout = move || {
        selected
            .get()
            .unwrap_or_else(|| "(nothing selected)".to_string())
    };
    let matrix_readout = move || {
        matrix_selected
            .get()
            .unwrap_or_else(|| "(nothing selected)".to_string())
    };

    view! {
        <ContentLayout
            title="Selectable Summary"
            description="Opinionated single-selection group of compact count cards: one radiogroup, one tab stop, arrow-key navigation, status as shape and colour, and an unmeasured check that is never rendered as zero."
        >
            <Section title="Fourteen diagnostic checks -- controlled single selection">
                <div class="w-full flex flex-col gap-4" data-testid="selectable-summary-checks">
                    <SelectableSummaryGroup
                        label="Data quality checks"
                        items=Signal::derive(diagnostic_checks)
                        selected=selected
                        on_select=on_select
                    />
                    <p class="ld-text-small text-base-content/75" data-testid="selectable-summary-readout">
                        "Selected: " {selected_readout}
                    </p>
                    <p class="ld-text-small text-base-content/75">
                        "Tab reaches the group once, then Arrow keys move and select, Home/End jump to the ends, and Space or Enter selects the focused card. The disabled card is skipped."
                    </p>
                </div>
            </Section>

            <Section title="Every status, plus disabled -- nothing selected at first">
                <div class="w-full flex flex-col gap-4" data-testid="selectable-summary-matrix">
                    <SelectableSummaryGroup
                        label="Status matrix"
                        items=Signal::derive(status_matrix)
                        selected=matrix_selected
                        on_select=on_matrix_select
                    />
                    <p class="ld-text-small text-base-content/75" data-testid="selectable-summary-matrix-readout">
                        "Selected: " {matrix_readout}
                    </p>
                    <p class="ld-text-small text-base-content/75">
                        "Clean shows a measured 0. Unavailable shows the localized placeholder and announces it instead of a number -- the two are never the same reading."
                    </p>
                </div>
            </Section>

            <Section title="Narrow column -- the grid follows the GROUP's width, not the window's">
                <div class="max-w-md" data-testid="selectable-summary-narrow">
                    <SelectableSummaryGroup
                        label="Data quality checks, narrow column"
                        items=Signal::derive(diagnostic_checks)
                        selected=narrow_selected
                        on_select=on_narrow_select
                    />
                </div>
            </Section>

            <Section title="Named by a visible heading instead of an aria-label">
                <div class="w-full flex flex-col gap-3" data-testid="selectable-summary-labelled">
                    <h3 id="selectable-summary-labelled-heading" class="ld-text-subtitle font-semibold">
                        "Import diagnostics"
                    </h3>
                    <SelectableSummaryGroup
                        label="Import diagnostics"
                        labelled_by="selectable-summary-labelled-heading"
                        items=Signal::derive(status_matrix)
                        selected=labelled_selected
                        on_select=on_labelled_select
                    />
                </div>
            </Section>

            <Section title="Reactive localized copy">
                <div class="w-full flex flex-col gap-4" data-testid="selectable-summary-localized">
                    <Button
                        size=ButtonSize::Sm
                        attr:data-testid="selectable-summary-locale-toggle"
                        on:click=move |_| set_french.update(|french| *french = !*french)
                    >
                        {move || if french.get() { "Switch to English" } else { "Passer au francais" }}
                    </Button>
                    <SelectableSummaryGroup
                        label=localized_label
                        items=localized_items
                        selected=localized_selected
                        texts=localized_texts
                        on_select=on_localized_select
                    />
                </div>
            </Section>
        </ContentLayout>
    }
}
