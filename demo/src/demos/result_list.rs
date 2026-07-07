use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

#[component]
pub fn ResultListDemo() -> impl IntoView {
    let (activated, set_activated) = signal(String::from("None"));
    let (highlighted, set_highlighted) = signal(String::from("None"));

    let files = vec![
        ResultRow {
            title: "README.md".into(),
            subtitle: "/".into(),
            snippet: String::new(),
        },
        ResultRow {
            title: "component.rs".into(),
            subtitle: String::new(),
            snippet: "...pub fn ResultList(items: Signal<Vec<ResultRow>>...".into(),
        },
        ResultRow {
            title: "types.rs".into(),
            subtitle: String::new(),
            snippet: "...pub fn move_selection(current: Option<usize>, delta: i32, len: usize)...".into(),
        },
        ResultRow {
            title: "mod.rs".into(),
            subtitle: "/src/components/result_list".into(),
            snippet: String::new(),
        },
        ResultRow {
            title: "conventions.md".into(),
            subtitle: String::new(),
            snippet: "...tests.rs — unit tests (enum as_str mappings, pure logic, builder methods)...".into(),
        },
        ResultRow {
            title: "task-11-brief.md".into(),
            subtitle: "/.superpowers/sdd".into(),
            snippet: String::new(),
        },
    ];

    let empty: Vec<ResultRow> = vec![];

    view! {
        <ContentLayout
            title="Result List"
            description="Flat, ranked, keyboard-navigable search-results picker with variable-height rows"
        >
            <Section title="Basic Usage">
                <p class="text-sm opacity-70 mb-2">
                    "Click a row, or focus the list and use "
                    <kbd class="kbd kbd-sm">"↑"</kbd> " " <kbd class="kbd kbd-sm">"↓"</kbd> " "
                    <kbd class="kbd kbd-sm">"Home"</kbd> " " <kbd class="kbd kbd-sm">"End"</kbd>
                    " and " <kbd class="kbd kbd-sm">"Enter"</kbd> "."
                </p>
                <div class="alert alert-info mb-4">
                    <span>
                        "Highlighted: " <strong>{move || highlighted.get()}</strong>
                        " | Activated: " <strong>{move || activated.get()}</strong>
                    </span>
                </div>
                <div class="max-w-md">
                    <ResultList
                        items=Signal::derive(move || files.clone())
                        on_selection_change=Callback::new(move |idx: Option<usize>| {
                            set_highlighted
                                .set(
                                    idx
                                        .map(|i| format!("row {i}"))
                                        .unwrap_or_else(|| "None".to_string()),
                                );
                        })
                        on_select=Callback::new(move |row: ResultRow| {
                            set_activated.set(row.title);
                        })
                    />
                </div>
            </Section>

            <Section title="Empty State">
                <div class="max-w-md">
                    <ResultList
                        items=Signal::derive(move || empty.clone())
                        empty_message=Signal::derive(|| "No matches for your search.".to_string())
                    />
                </div>
            </Section>

            <Section title="Features">
                <ul class="list-disc list-inside space-y-1 text-base-content/70">
                    <li>"Variable-height rows — the secondary line wraps naturally"</li>
                    <li>"ArrowUp / ArrowDown move selection, clamped at the ends"</li>
                    <li>"Home / End jump to the first / last row"</li>
                    <li>"Enter activates the selected row"</li>
                    <li>"Hover previews a row; click both selects and activates it"</li>
                    <li>"Selected row auto-scrolls into view"</li>
                    <li>"WAI-ARIA listbox pattern: role=listbox/option, aria-selected, aria-activedescendant"</li>
                </ul>
            </Section>
        </ContentLayout>
    }
}
