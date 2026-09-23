//! Showcase for the canonical row-action presets (ldui-bmqj), the standard
//! Export action (ldui-e6x8), and `Button::disabled_reason` (ldui-p82h).
//!
//! Browser proof: `tests/row_action_presets_smoke.rs`
//! (`cargo xtask test-row-action-presets`). The fixtures are keyed by
//! `data-testid` / `data-row-action` / `data-entity-export`, never by
//! document position.

use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// One row of the six presets for a named person. `complete_reason`, when
/// non-blank, disables the Complete action with that reason.
#[component]
fn PresetRow(
    testid: &'static str,
    name: &'static str,
    #[prop(optional, into)] complete_reason: Signal<String>,
    last: RwSignal<String>,
) -> impl IntoView {
    let fire = move |kind: RowActionKind| {
        Callback::new(move |()| last.set(format!("{}:{name}", kind.as_str())))
    };
    view! {
        <tr data-testid=testid>
            <td>{name}</td>
            <td>
                <div class="flex items-center gap-1">
                    <RowActionButton kind=RowActionKind::Open name=name on_click=fire(RowActionKind::Open) />
                    <RowActionButton kind=RowActionKind::Delete name=name on_click=fire(RowActionKind::Delete) />
                    <RowActionButton kind=RowActionKind::Call name=name on_click=fire(RowActionKind::Call) />
                    <RowActionButton kind=RowActionKind::Text name=name on_click=fire(RowActionKind::Text) />
                    <RowActionButton kind=RowActionKind::Email name=name on_click=fire(RowActionKind::Email) />
                    <RowActionButton
                        kind=RowActionKind::Complete
                        name=name
                        on_click=fire(RowActionKind::Complete)
                        disabled_reason=complete_reason
                    />
                </div>
            </td>
        </tr>
    }
}

#[component]
pub fn RowActionPresetsDemo() -> impl IntoView {
    let last_row_action = RwSignal::new(String::new());
    let exports = RwSignal::new(0usize);
    let on_export = Callback::new(move |()| exports.update(|n| *n += 1));
    let forty_two = Signal::stored(42usize);
    let none = Signal::stored(0usize);

    view! {
        <ContentLayout
            title="Row Action Presets"
            description="Six canonical icon row actions, the standard Export action, and Button's disabled_reason"
        >
            <Section title="The six presets, per row" col=true>
                <p class="text-sm text-base-content/75">
                    "Icon actions only in rows; text buttons live in the quick-action row. "
                    "Each button is named for its row (" <code>"Delete Ana Ruiz"</code> "), carries the same "
                    "text as its tooltip, and is findable by " <code>"data-row-action"</code> ". "
                    "The second row's Complete is disabled with a reason."
                </p>
                <table class="table table-sm" data-testid="row-action-presets-table">
                    <thead>
                        <tr>
                            <th>"Client"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <PresetRow testid="row-ana" name="Ana Ruiz" last=last_row_action />
                        <PresetRow
                            testid="row-luis"
                            name="Luis Ortega"
                            complete_reason="Already complete"
                            last=last_row_action
                        />
                    </tbody>
                </table>
                <p class="text-sm text-base-content/75">
                    "Last row action: "
                    <span class="font-mono" data-testid="row-action-last">{move || last_row_action.get()}</span>
                </p>
            </Section>

            <Section title="The Export action" col=true>
                <p class="text-sm text-base-content/75">
                    "Rendered through a table's " <code>"toolbar_actions"</code> " slot. With rows it is live; "
                    "at zero rows it is disabled with " <code>"No rows to export"</code> " as its reason. "
                    "The framework owns none of the encoding or download policy."
                </p>
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2" data-testid="export-with-rows">
                        <span class="text-sm">"42 rows"</span>
                        <EntityExportAction rows=forty_two on_export=on_export />
                    </div>
                    <div class="flex items-center gap-2" data-testid="export-empty">
                        <span class="text-sm">"0 rows"</span>
                        <EntityExportAction rows=none on_export=on_export />
                    </div>
                    <span class="text-sm text-base-content/75">
                        "Exports fired: "
                        <span class="font-mono" data-testid="export-count">{move || exports.get()}</span>
                    </span>
                </div>
            </Section>

            <Section title="Button::disabled_reason" col=true>
                <p class="text-sm text-base-content/75">
                    "A reasoned button is disabled, described through " <code>"aria-describedby"</code>
                    ", and titled. A button disabled with no reason is stamped "
                    <code>"data-disabled-without-reason"</code> " for the audit's "
                    <code>"disabled-without-reason"</code> " rule -- the second button here is that finding, on purpose."
                </p>
                <div class="flex items-center gap-2">
                    <Button
                        disabled_reason=Signal::stored("Sign-off is pending".to_owned())
                        attr:data-testid="reasoned-button"
                    >
                        "Publish"
                    </Button>
                    <Button disabled=true attr:data-testid="disabled-without-reason-probe">
                        "Publish (no reason)"
                    </Button>
                </div>
            </Section>
        </ContentLayout>
    }
}
