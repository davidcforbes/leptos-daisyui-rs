//! Deterministic consumer regressions, served only by the page-scoped test host.

use leptos::prelude::*;
use leptos_daisyui_rs::components::{BadgeColor, Button, Select, Tooltip};
use leptos_daisyui_rs::patterns::{
    DatasetOption, DatasetSelector, KpiItem, KpiStrip, SelectableSummaryGroup,
    SelectableSummaryItem,
};

fn options() -> Vec<DatasetOption> {
    vec![
        DatasetOption::new("real", "Real office"),
        DatasetOption::new("virtual", "Virtual office"),
        DatasetOption::new("other", "Other office"),
    ]
}

#[component]
pub fn OfficeSelectFixture() -> impl IntoView {
    let mounted = RwSignal::new(false);
    let revision = RwSignal::new("empty".to_owned());
    let delayed = RwSignal::new(Vec::<DatasetOption>::new());
    let datasets = RwSignal::new(options());
    let selected = RwSignal::new("virtual".to_owned());
    let changes = RwSignal::new(0_u32);
    let chip = RwSignal::new(Some("Behind".to_owned()));
    let wrap = RwSignal::new(false);
    let summary_selection = RwSignal::new(None::<String>);
    let tooltip_clicks = RwSignal::new(0_u32);
    let summary_label = "Assignments requiring additional review by the regional office coordinator before the next scheduled client meeting";
    let kpis = Signal::derive(move || {
        let item = KpiItem::new("weekly", "Weekly hires", "3");
        vec![match chip.get() {
            Some(label) => item.with_status_chip(label, BadgeColor::Warning),
            None => item,
        }]
    });
    let accept = Callback::new(move |value: String| {
        changes.update(|count| *count += 1);
        selected.set(value);
    });

    view! {
        <section class="mx-auto flex max-w-3xl flex-col gap-4" id="office-select-fixture">
            <h1 class="ld-text-title">"Office Select regressions"</h1>
            <div class="flex flex-wrap gap-2">
                <Button attr:data-testid="select-mount" on_click=Callback::new(move |_| mounted.set(true))>"Mount"</Button>
                <Button attr:data-testid="select-revision" on_click=Callback::new(move |_| revision.set("loaded".to_owned()))>"Publish revision"</Button>
                <Button attr:data-testid="select-options" on_click=Callback::new(move |_| delayed.set(options()))>"Mount options"</Button>
                <Button attr:data-testid="select-unmount" on_click=Callback::new(move |_| mounted.set(false))>"Unmount"</Button>
                <Button attr:data-testid="dataset-refresh" on_click=Callback::new(move |_| datasets.set(options()))>"Refresh identical datasets"</Button>
                <Button attr:data-testid="dataset-relabel" on_click=Callback::new(move |_| datasets.update(|items| {
                    items[1].label = "Oficina virtual actualizada".to_owned();
                    items[1].disabled = true;
                }))>"Relabel and disable Virtual"</Button>
                <Button attr:data-testid="dataset-reorder" on_click=Callback::new(move |_| datasets.update(|items| items.rotate_left(1)))>"Reorder datasets"</Button>
            </div>
            <Show when=move || mounted.get()>
                <Select id="delayed-office-select" label="Delayed office" value=selected
                    options_revision=revision on_change=accept>
                    <optgroup label="Offices">
                        {move || delayed.get().into_iter().map(|option| view! {
                            <option value=option.value>{option.label}</option>
                        }).collect_view()}
                    </optgroup>
                </Select>
            </Show>
            <DatasetSelector control_id="stable-office-select" label="Dataset"
                selected=selected options=datasets on_change=accept />
            <p>"Accepted: " <output data-testid="select-accepted">{move || selected.get()}</output></p>
            <p>"Change callbacks: " <output data-testid="select-changes">{move || changes.get()}</output></p>
            <div class="flex flex-wrap gap-2">
                <Button attr:data-testid="chip-update" on_click=Callback::new(move |_| chip.set(Some("Needs review".to_owned())))>"Update chip"</Button>
                <Button attr:data-testid="chip-remove" on_click=Callback::new(move |_| chip.set(None))>"Remove chip"</Button>
                <Button attr:data-testid="summary-wrap" on_click=Callback::new(move |_| wrap.update(|value| *value = !*value))>"Toggle label wrapping"</Button>
            </div>
            <KpiStrip items=kpis />
            <Tooltip tip="hint" attr:data-tooltip-probe="landed"
                class:office-tooltip-spread=true style:--office-tooltip-spread="1"
                on:click=move |_| tooltip_clicks.update(|count| *count += 1)>
                <Button attr:data-testid="tooltip-spread-trigger">"Tooltip spread trigger"</Button>
            </Tooltip>
            <output data-testid="tooltip-spread-clicks">{move || tooltip_clicks.get()}</output>
            <div class="w-80 max-w-full" data-testid="wrapped-summary">
                <SelectableSummaryGroup label="Assignment review" items=Signal::stored(vec![SelectableSummaryItem::new("review", summary_label, 4)])
                    selected=summary_selection wrap_labels=wrap on_select=Callback::new(move |id| summary_selection.set(Some(id))) />
            </div>
        </section>
    }
}
