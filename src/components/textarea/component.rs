use super::style::{TextareaColor, TextareaSize};
use crate::merge_classes;
use leptos::{html::Textarea as HtmlTextarea, prelude::*};

/// Converts an optional numeric HTML attribute value (used for `rows` and
/// `maxlength`) into the string Leptos expects, omitting the attribute
/// entirely when `None` so the browser falls back to its own default.
pub(super) fn optional_numeric_attr(value: Option<u32>) -> Option<String> {
    value.map(|v| v.to_string())
}

/// # Textarea Component
///
/// A multi-line text input component for entering longer text content.
/// Supports controlled usage via the `value` prop, matching the `Input`
/// component's binding idiom.
///
/// ```rust,ignore
/// let (text, set_text) = signal(String::new());
///
/// view! {
///     <Textarea
///         value=text
///         placeholder="Type here..."
///         on_input=move |v| set_text.set(v)
///     />
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("textarea textarea-ghost textarea-primary textarea-secondary textarea-accent textarea-info textarea-success textarea-warning textarea-error textarea-xs textarea-sm textarea-md textarea-lg textarea-xl");
/// ```
///
/// ## Node References
/// - `node_ref` - References the textarea element ([HTMLTextareaElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLTextareaElement))
#[component]
pub fn Textarea(
    /// The color variant of the textarea
    #[prop(optional, into)]
    color: Signal<TextareaColor>,

    /// The size variant of the textarea
    #[prop(optional, into)]
    size: Signal<TextareaSize>,

    /// Whether the textarea is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Bound value for the textarea (controlled component)
    #[prop(optional, into)]
    value: Signal<String>,

    /// Placeholder text shown when the textarea is empty
    #[prop(optional, into)]
    placeholder: Signal<String>,

    /// Number of visible text lines (HTML `rows` attribute). Omitted when `None`.
    #[prop(optional, into)]
    rows: Signal<Option<u32>>,

    /// Maximum number of characters allowed (HTML `maxlength` attribute). Omitted when `None`.
    #[prop(optional, into)]
    maxlength: Signal<Option<u32>>,

    /// Whether the textarea is read-only
    #[prop(optional, into)]
    readonly: Signal<bool>,

    /// Whether the textarea is a required form field
    #[prop(optional, into)]
    required: Signal<bool>,

    /// The `name` attribute used when submitting an enclosing form
    #[prop(optional, into)]
    name: Signal<Option<String>>,

    /// Callback fired on every input event with the new value
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,

    /// Callback fired on change (commit) events with the new value
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,

    /// Additional CSS classes to apply
    #[prop(optional, into)]
    class: &'static str,

    /// Reference to the underlying HTML textarea element
    #[prop(optional)]
    node_ref: NodeRef<HtmlTextarea>,

    /// Accessible name (rendered as `aria-label`) for a textarea outside a
    /// labeled field — the `select`/`capacity_bar` convention.
    #[prop(optional, into)]
    label: MaybeProp<String>,
) -> impl IntoView {
    view! {
        <textarea
            aria-label=move || label.get()
            disabled=disabled
            readonly=readonly
            required=required
            prop:value=move || value.get()
            placeholder=move || placeholder.get()
            rows=move || optional_numeric_attr(rows.get())
            maxlength=move || optional_numeric_attr(maxlength.get())
            name=move || name.get()
            on:input=move |e| {
                if let Some(cb) = on_input {
                    cb.run(event_target_value(&e));
                }
            }
            on:change=move |e| {
                if let Some(cb) = on_change {
                    cb.run(event_target_value(&e));
                }
            }
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "textarea ld-focus-ring",
                color.get().as_str(),
                size.get().as_str(),
                class
                )
            }
        />
    }
}
