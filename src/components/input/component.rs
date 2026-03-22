use super::style::{InputColor, InputSize, InputStyle};
use crate::merge_classes;
use leptos::{html::Input as HtmlInput, prelude::*};

/// # Input Component
///
/// A reactive Leptos wrapper for daisyUI's input component that provides styled
/// text input fields with customizable size, color, and style.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("input input-neutral input-primary input-secondary input-accent input-info input-success input-warning input-error input-ghost input-xs input-sm input-md input-lg input-xl");
/// ```
///
/// ## Node References
/// - `node_ref` - References the input element ([HTMLInputElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement))
#[component]
pub fn Input(
    /// Input style variant
    #[prop(optional, into)]
    style: Signal<InputStyle>,

    /// Input color variant
    #[prop(optional, into)]
    color: Signal<InputColor>,

    /// Input size variant
    #[prop(optional, into)]
    size: Signal<InputSize>,

    /// Whether the input is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Bound value for the input
    #[prop(optional, into)]
    value: Signal<String>,

    /// Placeholder text
    #[prop(optional, into)]
    placeholder: Signal<String>,

    /// Callback fired on input change with the new value
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,

    /// Callback fired on keydown events
    #[prop(optional, into)]
    on_keydown: Option<Callback<web_sys::KeyboardEvent>>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the input element
    #[prop(optional)]
    node_ref: NodeRef<HtmlInput>,
) -> impl IntoView {
    view! {
        <input
            disabled=disabled
            prop:value=move || value.get()
            placeholder=move || placeholder.get()
            on:input=move |e| {
                if let Some(cb) = on_input {
                    cb.run(event_target_value(&e));
                }
            }
            on:keydown=move |e| {
                if let Some(cb) = on_keydown {
                    cb.run(e);
                }
            }
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "input",
                    style.get().as_str(),
                    color.get().as_str(),
                    size.get().as_str(),
                    class
                )
            }
        />
    }
}
