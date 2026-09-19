//! Framework-owned "saved filters" bar for the opinionated `EntityTable`
//! filter row: a **Save Filter** button, its name dialog, and one badge per
//! saved set.
//!
//! The bar renders *inside the table toolbar*, left-justified, when
//! `EntityTable::saved_filters` is `Some`. Everything it does is a proposal
//! against [`super::saved_filters::EntitySavedFilters`] -- the consumer owns
//! the list and the live values and decides; this module owns only the
//! dialog's open state and its draft name, which are internal presentation
//! state, never the saved data.

use leptos::prelude::*;

use super::saved_filters::{EntitySavedFilters, saved_filter_name_is_valid};
use crate::components::badge::Badge;
use crate::components::button::{Button, ButtonType};
use crate::components::modal::{Modal, ModalBox};

/// Renders the saved-filters island for `EntityTable`'s toolbar. Not a
/// public component: `EntityTable` mounts it, so the badge row stays
/// colocated with the filter row it drives.
pub(crate) fn saved_filters_bar(model: EntitySavedFilters, bar_id: String) -> AnyView {
    let dialog_open = RwSignal::new(false);
    let draft_name = RwSignal::new(String::new());

    let texts = model.texts;
    let filters = model.filters;
    let current_values = model.current_values;
    let on_save = model.on_save;
    let on_delete = model.on_delete;
    let on_apply = model.on_apply;

    let open_dialog = Callback::new(move |()| {
        draft_name.set(String::new());
        dialog_open.set(true);
    });

    let confirm_save = Callback::new(move |()| {
        let name = draft_name.get_untracked();
        let name = name.trim().to_owned();
        if !saved_filter_name_is_valid(&name) {
            return;
        }
        on_save.run((name, current_values.get_untracked()));
        dialog_open.set(false);
    });

    let title_id = format!("{bar_id}-title");
    let name_input_id = format!("{bar_id}-name");

    view! {
        <div
            class="flex shrink-0 flex-wrap items-center justify-start gap-2 mr-auto"
            data-entity-saved-filters-bar="true"
        >
            <Button
                class="btn-sm btn-outline"
                attr:data-entity-saved-filters-open="true"
                attr:aria-label=move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.save_button.clone())
                on_click=Callback::new(move |_| open_dialog.run(()))
            >
                {move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.save_button.clone())}
            </Button>

            {move || {
                let saved = filters.get();
                if saved.is_empty() {
                    return view! {
                        <span
                            class="text-sm text-base-content/60"
                            data-entity-saved-filters-empty="true"
                        >
                            {move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.empty.clone())}
                        </span>
                    }
                    .into_any();
                }
                let current = current_values.get();
                let active = EntitySavedFilters::active_name(&saved, &current);
                saved
                    .into_iter()
                    .map(|saved_filter| {
                        let name = saved_filter.name.clone();
                        let apply_label = texts
                            .with(|t: &super::saved_filters::EntitySavedFilterTexts| t.apply_named(&name));
                        let remove_label = texts
                            .with(|t: &super::saved_filters::EntitySavedFilterTexts| t.remove_named(&name));
                        let is_active = active.as_deref() == Some(name.as_str());
                        let saved_for_apply = saved_filter.clone();
                        let name_for_apply = name.clone();
                        let name_for_badge = name.clone();
                        let name_for_remove = name.clone();
                        view! {
                            <Badge
                                class="gap-1"
                                attr:data-entity-saved-filter=name_for_badge
                                attr:data-entity-saved-filter-active=is_active.then_some("true")
                            >
                                <button
                                    type="button"
                                    class="cursor-pointer"
                                    aria-label=apply_label
                                    data-entity-saved-filter-apply=name_for_apply.clone()
                                    on:click=move |_| on_apply.run(saved_for_apply.clone())
                                >
                                    {name.clone()}
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs btn-circle -mr-1"
                                    aria-label=remove_label
                                    data-entity-saved-filter-remove=name_for_remove.clone()
                                    on:click=move |event: web_sys::MouseEvent| {
                                        event.stop_propagation();
                                        on_delete.run(name_for_remove.clone());
                                    }
                                >
                                    "×"
                                </button>
                            </Badge>
                        }
                        .into_any()
                    })
                    .collect_view()
                    .into_any()
            }}

            <Modal
                open=Signal::derive(move || dialog_open.get())
                backdrop=true
                labelled_by=title_id.clone()
            >
                <ModalBox class="max-w-md">
                    <h3 id=title_id.clone() class="text-lg font-bold" data-entity-saved-filters-title="true">
                        {move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.dialog_title.clone())}
                    </h3>
                    <form
                        class="mt-4 flex flex-col gap-4"
                        data-entity-saved-filters-form="true"
                        on:submit=move |event: web_sys::SubmitEvent| {
                            event.prevent_default();
                            confirm_save.run(());
                        }
                    >
                        <label
                            class="flex flex-col gap-1 text-sm font-medium"
                            for=name_input_id.clone()
                        >
                            <span>{move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.name_label.clone())}</span>
                            <input
                                id=name_input_id.clone()
                                class="input input-bordered w-full"
                                type="text"
                                placeholder=move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.name_placeholder.clone())
                                prop:value=move || draft_name.get()
                                on:input=move |event: web_sys::Event| {
                                    use wasm_bindgen::JsCast;
                                    if let Some(target) = event
                                        .target()
                                        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
                                    {
                                        draft_name.set(target.value());
                                    }
                                }
                                data-entity-saved-filters-name="true"
                            />
                        </label>
                        <div class="modal-action">
                            <Button
                                class="btn-ghost"
                                button_type=ButtonType::Button
                                attr:data-entity-saved-filters-cancel="true"
                                on_click=Callback::new(move |_| dialog_open.set(false))
                            >
                                {move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.cancel.clone())}
                            </Button>
                            <Button
                                class="btn-primary"
                                button_type=ButtonType::Submit
                                disabled=Signal::derive(move || !saved_filter_name_is_valid(&draft_name.get()))
                                attr:data-entity-saved-filters-save="true"
                            >
                                {move || texts.with(|t: &super::saved_filters::EntitySavedFilterTexts| t.save.clone())}
                            </Button>
                        </div>
                    </form>
                </ModalBox>
            </Modal>
        </div>
    }
    .into_any()
}
