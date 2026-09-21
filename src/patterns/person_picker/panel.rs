use super::listbox::PersonListbox;
use super::model::{PickerPerson, nameable_selection, visible_people};
use super::texts::PersonPickerTexts;
use crate::components::{FilterSidebar, SidebarSide};
use leptos::{ev, prelude::*};

/// The opinionated roster panel. Fixed 360px, text wraps, the card is the
/// selection control. The page owns `selected` and `collapsed`; search text
/// is internal view state and never clears the selection.
#[component]
pub fn PersonPicker(
    /// Everyone on the roster.
    #[prop(into)]
    roster: Signal<Vec<PickerPerson>>,
    /// The page-owned selection.
    #[prop(into)]
    selected: Signal<Option<String>>,
    /// Proposes `Some(id)` to select or `None` to clear.
    on_select: Callback<Option<String>>,
    /// Page-owned so it can be remembered.
    #[prop(into)]
    collapsed: Signal<bool>,
    /// Asks the page to flip `collapsed`.
    on_toggle_collapsed: Callback<()>,
    /// Panel heading.
    #[prop(into)]
    title: Signal<String>,
    /// Copy. A `Signal`, so a language arriving after mount propagates.
    #[prop(into, default = Signal::stored(PersonPickerTexts::default()))]
    texts: Signal<PersonPickerTexts>,
    /// Page-specific content under the roster.
    #[prop(optional)]
    footer: Option<Children>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let visible = Memo::new(move |_| {
        let query = search.get();
        roster.with(|roster| visible_people(roster, &query))
    });
    // Memos on emptiness so the branch below re-renders only when it flips,
    // not on every keystroke (which would remount the listbox).
    let roster_empty = Memo::new(move |_| roster.with(Vec::is_empty));
    let visible_empty = Memo::new(move |_| visible.with(Vec::is_empty));
    let named = Memo::new(move |_| {
        let selected = selected.get();
        roster.with(|roster| nameable_selection(roster, selected.as_deref()).cloned())
    });

    let on_search = move |event: ev::Event| search.set(event_target_value(&event));
    // Counts only a NAMEABLE pick (Office op-4c18j).
    let active_count = Signal::derive(move || usize::from(named.get().is_some()));
    let toggle_label = Signal::derive(move || {
        texts.with(|t| {
            if collapsed.get() {
                t.show.clone()
            } else {
                t.hide.clone()
            }
        })
    });
    let roster_label = Signal::derive(move || texts.with(|t| t.roster_label.clone()));

    let search_clear = move || {
        (!search.with(String::is_empty)).then(|| {
            view! {
                <button
                    type="button"
                    class="btn btn-ghost btn-xs btn-circle absolute right-1 top-1/2 -translate-y-1/2"
                    aria-label=move || texts.with(|t| t.clear_search.clone())
                    data-person-search-clear="true"
                    on:click=move |_| search.set(String::new())
                >
                    "×"
                </button>
            }
        })
    };

    let summary = move || {
        named.get().map(|person| {
            view! {
                <div
                    class="flex min-w-0 items-center gap-2 rounded-box bg-base-200 px-3 py-2 text-sm"
                    data-person-selected-summary=person.id.clone()
                >
                    <span class="min-w-0 flex-1 whitespace-normal [overflow-wrap:anywhere]">
                        {move || texts.with(|t| t.selected_prefix.clone())}
                        " "
                        <strong>{person.name.clone()}</strong>
                    </span>
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs btn-circle shrink-0"
                        aria-label=move || texts.with(|t| t.clear_selection.clone())
                        data-person-selection-clear="true"
                        on:click=move |_| on_select.run(None)
                    >
                        "×"
                    </button>
                </div>
            }
        })
    };

    let roster_body = move || {
        if roster_empty.get() {
            view! {
                <p class="text-sm text-base-content/75" data-person-roster-empty="true">
                    {move || texts.with(|t| t.empty_roster.clone())}
                </p>
            }
            .into_any()
        } else if visible_empty.get() {
            view! {
                <p class="text-sm text-base-content/75" data-person-no-matches="true">
                    {move || texts.with(|t| t.no_matches.clone())}
                </p>
            }
            .into_any()
        } else {
            view! {
                <PersonListbox
                    people=visible
                    selected=selected
                    on_select=on_select
                    label=roster_label
                    texts=texts
                />
            }
            .into_any()
        }
    };

    view! {
        <FilterSidebar
            collapsed=collapsed
            on_toggle=on_toggle_collapsed
            active_count=active_count
            title=title
            toggle_label=toggle_label
            side=SidebarSide::Right
            expanded_width="w-[360px] min-w-[360px] max-w-[360px]"
            attr:data-person-picker="true"
        >
            <div class="grid min-w-0 gap-3 p-3">
                <div class="relative min-w-0">
                    <input
                        type="search"
                        // The native cancel button would be a second x.
                        class="input input-sm w-full pr-8 [&::-webkit-search-cancel-button]:hidden"
                        prop:value=move || search.get()
                        placeholder=move || texts.with(|t| t.search_placeholder.clone())
                        aria-label=move || texts.with(|t| t.search_label.clone())
                        data-person-search="true"
                        on:input=on_search
                    />
                    {search_clear}
                </div>
                {summary}
                // overflow-y-auto ALONE forces overflow-x to auto as well --
                // the cause of the horizontal scroll this replaces.
                <div
                    class="max-h-[70dvh] min-w-0 overflow-y-auto overflow-x-hidden"
                    data-person-roster="true"
                >
                    {roster_body}
                </div>
                {footer.map(|footer| {
                    view! { <div class="min-w-0" data-person-picker-footer="true">{footer()}</div> }
                })}
            </div>
        </FilterSidebar>
    }
}
