//! Reusable multi-value badge / tag input.
//!
//! Used by the EUC-AI Registry form for fields like "Project EUC Technologies",
//! "Project Data Required", IAM role lists etc. Starts with an empty Vec, lets
//! the user type a value and press Enter to add a badge, and exposes an `x`
//! button on each badge for removal.
//!
//! UT-15 / UT-39 close-out.

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

use crate::components::{Badge, BadgeColor, BadgeSize, Input, InputSize};

/// Multi-value tag input bound to a `RwSignal<Vec<String>>`.
///
/// * `tags`       — the signal storing the current list of tag values.
/// * `placeholder`— text shown in the input box.
/// * `color`      — badge color (defaults to Primary).
#[component]
pub fn TagInput(
    /// Signal holding the current list of tag values.
    tags: RwSignal<Vec<String>>,
    /// Placeholder text for the inline input.
    #[prop(into, default = "Add and press Enter...".to_string())]
    placeholder: String,
    /// Optional badge color (defaults to Primary).
    #[prop(default = BadgeColor::Primary)]
    color: BadgeColor,
) -> impl IntoView {
    let draft = RwSignal::new(String::new());
    let stored_color = StoredValue::new(color);

    let push = move || {
        let value = draft.get_untracked().trim().to_string();
        if value.is_empty() {
            return;
        }
        tags.update(|list| {
            if !list.iter().any(|t| t == &value) {
                list.push(value);
            }
        });
        draft.set(String::new());
    };

    view! {
        <div class="flex flex-wrap items-center gap-2">
            {move || {
                tags.get()
                    .into_iter()
                    .enumerate()
                    .map(|(idx, tag)| {
                        let label = tag.clone();
                        view! {
                            <Badge size=BadgeSize::Sm color=stored_color.get_value() class="gap-1">
                                <span>{label}</span>
                                <button
                                    type="button"
                                    class="text-xs opacity-70 hover:opacity-100"
                                    on:click=move |_| {
                                        tags.update(|list| {
                                            if idx < list.len() {
                                                list.remove(idx);
                                            }
                                        });
                                    }
                                >
                                    "x"
                                </button>
                            </Badge>
                        }
                    })
                    .collect_view()
            }}
            <Input
                size=InputSize::Sm
                class="input-bordered flex-1 min-w-[8rem]"
                attr:r#type="text"
                attr:placeholder=placeholder
                prop:value=move || draft.get()
                on:input=move |ev| draft.set(event_target_value(&ev))
                on:keydown=move |ev: KeyboardEvent| {
                    if ev.key() == "Enter" {
                        ev.prevent_default();
                        push();
                    }
                }
            />
        </div>
    }
}
