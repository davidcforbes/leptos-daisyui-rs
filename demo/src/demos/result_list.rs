use crate::core::{ContentLayout, Section};
use crate::debug_state;
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;

/// Typed activation payload for the `KeyedResultList` showcase: a case
/// reference the display row (a person's name) does not fully determine.
#[derive(Clone, Debug, PartialEq)]
struct CaseRef {
    case_number: &'static str,
}

/// Baseline keyed fixture. Two rows share the identical display name "Alex
/// Morgan" on purpose — the point of `KeyedResultList` is that activation
/// still returns the right one.
fn keyed_case_fixture() -> Vec<ResultListItem<CaseRef>> {
    vec![
        ResultListItem::new(
            "case-a",
            ResultRow {
                title: "Alex Morgan".into(),
                subtitle: "Intake — 2026-08-12".into(),
                snippet: String::new(),
            },
            CaseRef {
                case_number: "A-100",
            },
        ),
        ResultListItem::new(
            "case-b",
            ResultRow {
                title: "Alex Morgan".into(),
                subtitle: "Intake — 2026-08-19".into(),
                snippet: String::new(),
            },
            CaseRef {
                case_number: "B-200",
            },
        ),
        ResultListItem::new(
            "case-c",
            ResultRow {
                title: "Priya Natarajan".into(),
                subtitle: "Intake — 2026-08-21".into(),
                snippet: String::new(),
            },
            CaseRef {
                case_number: "C-300",
            },
        ),
    ]
}

/// Reverses the fixture order — every key keeps its original identity, only
/// positions change.
fn reorder_keyed_fixture(items: &[ResultListItem<CaseRef>]) -> Vec<ResultListItem<CaseRef>> {
    let mut reordered = items.to_vec();
    reordered.reverse();
    reordered
}

/// Drops the `case-a` row, simulating an asynchronous result set that no
/// longer contains the previously selected/activated identity.
fn remove_case_a(items: &[ResultListItem<CaseRef>]) -> Vec<ResultListItem<CaseRef>> {
    items
        .iter()
        .filter(|item| item.key != "case-a")
        .cloned()
        .collect()
}

/// Inserts a new highest-ranked result ahead of the existing rows, without
/// touching any existing key.
fn insert_new_top_result(items: &[ResultListItem<CaseRef>]) -> Vec<ResultListItem<CaseRef>> {
    let mut next = vec![ResultListItem::new(
        "case-d",
        ResultRow {
            title: "Jordan Blake".into(),
            subtitle: "Intake — 2026-08-27".into(),
            snippet: String::new(),
        },
        CaseRef {
            case_number: "D-400",
        },
    )];
    next.extend(items.iter().cloned());
    next
}

/// Relabels `case-a` (a display-text change) while keeping its key and
/// payload — selection must follow the key, not the old label.
fn relabel_case_a(items: &[ResultListItem<CaseRef>]) -> Vec<ResultListItem<CaseRef>> {
    items
        .iter()
        .cloned()
        .map(|mut item| {
            if item.key == "case-a" {
                item.row.title = "Alexandra Morgan".into();
            }
            item
        })
        .collect()
}

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
            snippet: "...pub fn move_selection(current: Option<usize>, delta: i32, len: usize)..."
                .into(),
        },
        ResultRow {
            title: "mod.rs".into(),
            subtitle: "/src/components/result_list".into(),
            snippet: String::new(),
        },
        ResultRow {
            title: "conventions.md".into(),
            subtitle: String::new(),
            snippet:
                "...tests.rs — unit tests (enum as_str mappings, pure logic, builder methods)..."
                    .into(),
        },
        ResultRow {
            title: "task-11-brief.md".into(),
            subtitle: "/.superpowers/sdd".into(),
            snippet: String::new(),
        },
    ];

    let empty: Vec<ResultRow> = vec![];

    let keyed_items = RwSignal::new(keyed_case_fixture());
    let (keyed_activated, set_keyed_activated) = signal(String::from("None"));
    let (keyed_highlighted, set_keyed_highlighted) = signal(String::from("None"));

    let on_keyed_selection_change = Callback::new(move |key: Option<String>| {
        let label = key.clone().unwrap_or_else(|| "None".to_string());
        set_keyed_highlighted.set(label);
        debug_state::set("keyed_result_list.highlight", key);
    });
    let on_keyed_select = Callback::new(move |item: ResultListItem<CaseRef>| {
        set_keyed_activated.set(format!("{} ({})", item.key, item.payload.case_number));
        debug_state::set(
            "keyed_result_list.activation",
            serde_json::json!({
                "key": item.key,
                "caseNumber": item.payload.case_number,
                "title": item.row.title,
            }),
        );
    });

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

            <Section title="Keyed & Typed Results (KeyedResultList)">
                <p class="text-sm opacity-70 mb-2">
                    "Each result carries a stable "
                    <code class="kbd kbd-sm">"key"</code>
                    " plus a typed payload separate from its display row. "
                    <strong>"Case-a"</strong>
                    " and "
                    <strong>"case-b"</strong>
                    " below intentionally share the display name "
                    <em>"Alex Morgan"</em>
                    " — activation still returns the exact case behind the row you picked, "
                    "and replacing the results below (reorder, remove, insert, relabel) never "
                    "reassigns the highlight or activation to a different identity."
                </p>
                <div class="alert alert-info mb-4" data-testid="keyed-result-list-status">
                    <span>
                        "Highlighted key: " <strong>{move || keyed_highlighted.get()}</strong>
                        " | Activated: " <strong>{move || keyed_activated.get()}</strong>
                    </span>
                </div>
                <div class="max-w-md mb-4" id="keyed-result-list" data-testid="keyed-result-list">
                    <KeyedResultList
                        items=keyed_items
                        on_selection_change=on_keyed_selection_change
                        on_select=on_keyed_select
                    />
                </div>
                <div class="flex flex-wrap gap-2">
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-reorder"
                        on:click=move |_| keyed_items.update(|items| *items = reorder_keyed_fixture(items))
                    >
                        "Reorder results"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-remove"
                        on:click=move |_| keyed_items.update(|items| *items = remove_case_a(items))
                    >
                        "Remove case-a"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-clear"
                        on:click=move |_| keyed_items.set(Vec::new())
                    >
                        "Clear all"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-insert"
                        on:click=move |_| keyed_items.update(|items| *items = insert_new_top_result(items))
                    >
                        "Insert new top result"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-relabel"
                        on:click=move |_| keyed_items.update(|items| *items = relabel_case_a(items))
                    >
                        "Relabel case-a"
                    </button>
                    <button
                        class="btn btn-sm"
                        data-testid="keyed-result-list-restore"
                        on:click=move |_| keyed_items.set(keyed_case_fixture())
                    >
                        "Restore fixture"
                    </button>
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
                    <li>"KeyedResultList: selection and activation are tracked by stable key, never by index or display text"</li>
                </ul>
            </Section>
        </ContentLayout>
    }
}
