use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::components::*;
use leptos_daisyui_rs::patterns::{SearchPickerDialog, SearchPickerStatus};
use std::time::Duration;

/// Typed activation payload the display row (a person's name) does not
/// fully determine -- mirrors the `KeyedResultList` showcase's duplicate-
/// title fixture (`case-a`/`case-b` both render "Alex Morgan").
#[derive(Clone, Debug, PartialEq)]
struct PickedCase {
    case_number: &'static str,
}

fn case_dataset() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("case-a", "Alex Morgan", "A-100"),
        ("case-b", "Alex Morgan", "B-200"),
        ("case-c", "Priya Natarajan", "C-300"),
        ("case-d", "Jordan Blake", "D-400"),
    ]
}

fn filter_cases(query: &str) -> Vec<ResultListItem<PickedCase>> {
    let needle = query.trim().to_lowercase();
    case_dataset()
        .into_iter()
        .filter(|(_, title, _)| needle.is_empty() || title.to_lowercase().contains(&needle))
        .map(|(key, title, case_number)| {
            ResultListItem::new(key, ResultRow::new(title), PickedCase { case_number })
        })
        .collect()
}

/// One independent `SearchPickerDialog` instance: its own open/query/status/
/// items/activation state, plus fixture controls to deterministically drive
/// every acceptance scenario (loading, error, empty, retained error, and a
/// slow-then-fast query race proving the stale response never lands).
#[component]
fn SearchPickerDialogFixture(instance: &'static str) -> impl IntoView {
    let open = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let status = RwSignal::new(SearchPickerStatus::Idle);
    let items = RwSignal::new(Vec::<ResultListItem<PickedCase>>::new());
    let activated = RwSignal::new(String::from("None"));
    let request_seq = RwSignal::new(0u64);
    let pending_handle = RwSignal::new(None::<TimeoutHandle>);

    let cancel_pending = move || {
        if let Some(handle) = pending_handle.get_untracked() {
            handle.clear();
        }
        pending_handle.set(None);
    };

    // Simulates an async fetch: bumps a monotonic request id, schedules a
    // delayed response, and discards it on arrival if a newer request has
    // since started -- the caller-owned half of the "stale responses can
    // never activate an old payload" contract (KeyedResultList supplies the
    // other half: even a landed response is only ever resolved by the
    // *current* key against the *current* items).
    let run_search = move |text: String, delay_ms: u64| {
        cancel_pending();
        request_seq.update(|seq| *seq += 1);
        let my_request = request_seq.get_untracked();
        let was_refresh = !items.get_untracked().is_empty();
        status.set(if was_refresh {
            SearchPickerStatus::Refreshing
        } else {
            SearchPickerStatus::Loading
        });
        let handle = set_timeout_with_handle(
            move || {
                if request_seq.get_untracked() != my_request {
                    return;
                }
                items.set(filter_cases(&text));
                status.set(SearchPickerStatus::Ready);
            },
            Duration::from_millis(delay_ms),
        );
        if let Ok(handle) = handle {
            pending_handle.set(Some(handle));
        }
    };

    let on_query_change = Callback::new(move |text: String| {
        query.set(text.clone());
        if text.trim().is_empty() {
            cancel_pending();
            status.set(SearchPickerStatus::Idle);
            items.set(Vec::new());
            return;
        }
        run_search(text, 120);
    });

    let on_select = Callback::new(move |item: ResultListItem<PickedCase>| {
        activated.set(format!("{} ({})", item.key, item.payload.case_number));
        open.set(false);
    });

    let on_retry = Callback::new(move |_| {
        run_search(query.get_untracked(), 120);
    });

    let force_error = move || {
        cancel_pending();
        status.set(if items.get_untracked().is_empty() {
            SearchPickerStatus::Error
        } else {
            SearchPickerStatus::RefreshError
        });
    };

    // Deterministic proof of the race: starts a slow search for "Alex"
    // (500ms) then, before it resolves, a fast search for "Priya" (30ms).
    // If the slow response were not discarded, `items` would briefly or
    // permanently show the Alex Morgan rows after the Priya search settles.
    let run_stale_response_race = move |_| {
        query.set("Alex".to_string());
        run_search("Alex".to_string(), 500);
        query.set("Priya".to_string());
        run_search("Priya".to_string(), 30);
    };

    view! {
        <div class="flex flex-col gap-3" data-testid=format!("{instance}-fixture")>
            <div class="alert alert-info" data-testid=format!("{instance}-status")>
                <span>
                    "Status: " <strong>{move || format!("{:?}", status.get())}</strong>
                    " | Activated: " <strong>{move || activated.get()}</strong>
                </span>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    color=ButtonColor::Primary
                    attr:data-testid=format!("{instance}-trigger")
                    on:click=move |_| open.set(true)
                >
                    "Open " {instance}
                </Button>
                <Button
                    style=ButtonStyle::Outline
                    attr:data-testid=format!("{instance}-force-error")
                    on:click=move |_| force_error()
                >
                    "Simulate error"
                </Button>
                <Button
                    style=ButtonStyle::Outline
                    attr:data-testid=format!("{instance}-race")
                    on:click=run_stale_response_race
                >
                    "Slow-then-fast query race"
                </Button>
            </div>

            <SearchPickerDialog
                open=open
                title=format!("Find a case ({instance})")
                query=query
                status=status
                items=items
                on_query_change=on_query_change
                on_select=on_select
                on_close=Callback::new(move |_| open.set(false))
                on_retry=on_retry
            />
        </div>
    }
}

#[component]
pub fn SearchPickerDialogDemo() -> impl IntoView {
    view! {
        <ContentLayout
            title="Search Picker Dialog"
            description="Composable typed search-picker dialog: a labelled search box plus a keyboard-navigable, typed result list inside a focus-trapped modal. Built from Modal, Field/Input, KeyedResultList, and PageStatePanel."
        >
            <Section title="Two independent instances">
                <p class="text-sm opacity-70 mb-2">
                    "Each dialog owns its own query/status/items/activation state. "
                    <strong>"Case-a"</strong> " and " <strong>"case-b"</strong>
                    " share the display name " <em>"Alex Morgan"</em>
                    " -- activation always returns the exact case behind the row you picked."
                </p>
                <p class="text-sm opacity-70 mb-4">
                    "Opening focuses the search field. "
                    <kbd class="kbd kbd-sm">"↑"</kbd> " " <kbd class="kbd kbd-sm">"↓"</kbd> " "
                    <kbd class="kbd kbd-sm">"Home"</kbd> " " <kbd class="kbd kbd-sm">"End"</kbd>
                    " and " <kbd class="kbd kbd-sm">"Enter"</kbd>
                    " work from the search field without tabbing to the list. "
                    <kbd class="kbd kbd-sm">"Escape"</kbd> " or Cancel closes and returns focus."
                </p>
                <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                    <SearchPickerDialogFixture instance="dialog-a" />
                    <SearchPickerDialogFixture instance="dialog-b" />
                </div>
            </Section>

            <Section title="Features">
                <ul class="list-disc list-inside space-y-1 text-base-content/70">
                    <li>"Controlled query, status, and typed keyed items -- the caller owns fetching"</li>
                    <li>"Opening focuses the search field; Escape/Cancel closes and restores focus"</li>
                    <li>"Arrow/Home/End/Enter operate the result list from the search field"</li>
                    <li>"Loading/error/empty/retained-error presentation via PageStatePanel"</li>
                    <li>"Activation always resolves the current payload for the current key"</li>
                    <li>"A superseded async response can never overwrite a newer one (see the race button)"</li>
                    <li>"Fully reactive localized copy via SearchPickerDialogTexts/PageStatePanelTexts"</li>
                </ul>
            </Section>
        </ContentLayout>
    }
}
