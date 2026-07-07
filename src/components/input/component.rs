use super::filter::InputFilter;
use super::style::{InputColor, InputSize, InputStyle, InputType};
use crate::merge_classes;
use leptos::{html::Input as HtmlInput, prelude::*};
use web_sys::wasm_bindgen::JsCast;

/// Converts an optional numeric HTML attribute value (used for `maxlength`)
/// into the string Leptos expects, omitting the attribute entirely when
/// `None` so the browser falls back to its own default. Mirrors
/// `Textarea::optional_numeric_attr`.
pub(super) fn optional_numeric_attr(value: Option<u32>) -> Option<String> {
    value.map(|v| v.to_string())
}

/// # Input Component
///
/// A reactive Leptos wrapper for daisyUI's input component that provides styled
/// text input fields with customizable size, color, and style. Also supports a
/// type-safe `input_type` (password/email/number/tel/search/url), an opt-in
/// password reveal toggle, character-class input filtering, and a leading icon
/// slot rendered via daisyUI 5's `<label class="input">` wrapper markup.
///
/// ```rust,ignore
/// let (value, set_value) = signal(String::new());
///
/// view! {
///     // Plain text input (unchanged bare-<input> rendering).
///     <Input value=value on_input=move |v| set_value.set(v) />
///
///     // Password field with a show/hide reveal toggle.
///     <Input input_type=InputType::Password revealable=true value=value />
///
///     // Numeric-only filtering.
///     <Input filter=InputFilter::Numeric value=value />
///
///     // Search field with a leading icon (daisyUI `label.input` wrapper).
///     <Input
///         input_type=InputType::Search
///         value=value
///         leading_icon=Box::new(|| view! { <span>"search"</span> }.into_any())
///     />
/// }
/// ```
///
/// ### Add to `input.css`
/// ```css
/// @source inline("input input-neutral input-primary input-secondary input-accent input-info input-success input-warning input-error input-ghost input-xs input-sm input-md input-lg input-xl");
/// @source inline("btn btn-ghost btn-xs btn-circle");
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

    /// HTML `type` attribute (Text, Password, Email, Number, Tel, Search, Url)
    #[prop(optional, into)]
    input_type: Signal<InputType>,

    /// Opt-in password reveal toggle. When `true` and `input_type` is
    /// `Password`, a trailing eye button is rendered that flips the field
    /// between masked and plaintext. Structural (not reactive): decided once
    /// when the component is created, like `leading_icon`.
    #[prop(optional, default = false)]
    revealable: bool,

    /// Character-class filter applied to typed input, stripping disallowed
    /// characters (see [`InputFilter`])
    #[prop(optional, into)]
    filter: Signal<InputFilter>,

    /// Whether the input is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Whether the input is read-only
    #[prop(optional, into)]
    readonly: Signal<bool>,

    /// Whether the input is a required form field
    #[prop(optional, into)]
    required: Signal<bool>,

    /// Maximum number of characters allowed (HTML `maxlength` attribute). Omitted when `None`.
    #[prop(optional, into)]
    maxlength: Signal<Option<u32>>,

    /// The `name` attribute used when submitting an enclosing form
    #[prop(optional, into)]
    name: Signal<Option<String>>,

    /// Bound value for the input
    #[prop(optional, into)]
    value: Signal<String>,

    /// Placeholder text
    #[prop(optional, into)]
    placeholder: Signal<String>,

    /// Callback fired on input change with the new (post-filter) value
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,

    /// Callback fired on keydown events
    #[prop(optional, into)]
    on_keydown: Option<Callback<web_sys::KeyboardEvent>>,

    /// Optional leading icon/content rendered inside the daisyUI
    /// `label.input` wrapper, ahead of the `<input>` element. Presence of
    /// this slot (like `revealable`) decides -- once, at creation -- whether
    /// the component renders the wrapped `label.input` markup or the plain
    /// bare `<input>` (the default, backward-compatible path).
    #[prop(optional)]
    leading_icon: Option<Children>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the input element
    #[prop(optional)]
    node_ref: NodeRef<HtmlInput>,
) -> impl IntoView {
    // Internal reveal state for the password show/hide toggle. Harmless to
    // create even when `revealable` is false.
    let reveal = RwSignal::new(false);

    // The type actually applied to the DOM element: flips Password -> Text
    // while revealed, otherwise mirrors `input_type`.
    let effective_type = Signal::derive(move || {
        if revealable && reveal.get() && input_type.get() == InputType::Password {
            InputType::Text.as_str()
        } else {
            input_type.get().as_str()
        }
    });

    // Structural decision, made once: use the daisyUI `label.input` wrapper
    // markup only when a leading icon or the password-reveal toggle was
    // requested. Otherwise render the plain bare `<input>` unchanged, so the
    // existing Input API/markup stays backward compatible by default.
    let use_wrapper = leading_icon.is_some() || revealable;

    if use_wrapper {
        view! {
            <label
                class=move || {
                    merge_classes!(
                        "input",
                        style.get().as_str(),
                        color.get().as_str(),
                        size.get().as_str(),
                        class
                    )
                }
            >
                {leading_icon.map(|icon| icon())}
                <input
                    type=move || effective_type.get()
                    disabled=disabled
                    readonly=readonly
                    required=required
                    maxlength=move || optional_numeric_attr(maxlength.get())
                    name=move || name.get()
                    prop:value=move || value.get()
                    placeholder=move || placeholder.get()
                    on:input=move |e| {
                        let raw = event_target_value(&e);
                        let filtered = filter.get().apply(&raw);
                        // The browser already applied the raw keystroke; write the
                        // filtered value back so the field never shows a disallowed
                        // character, even momentarily. Simplest-possible caret
                        // handling: the browser places the caret at the end.
                        if filtered != raw
                            && let Some(el) = e
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        {
                            el.set_value(&filtered);
                        }
                        if let Some(cb) = on_input {
                            cb.run(filtered);
                        }
                    }
                    on:keydown=move |e| {
                        if let Some(cb) = on_keydown {
                            cb.run(e);
                        }
                    }
                    node_ref=node_ref
                    class="grow ld-focus-ring"
                />
                {move || {
                    if revealable && input_type.get() == InputType::Password {
                        view! {
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs btn-circle"
                                aria-label=move || {
                                    if reveal.get() { "Hide password" } else { "Show password" }
                                }
                                on:click=move |_| reveal.update(|r| *r = !*r)
                            >
                                {move || {
                                    if reveal.get() {
                                        view! {
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                class="h-4 w-4"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                stroke="currentColor"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21"
                                                />
                                            </svg>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                class="h-4 w-4"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                stroke="currentColor"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                                                />
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                                />
                                            </svg>
                                        }
                                            .into_any()
                                    }
                                }}

                            </button>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}

            </label>
        }
            .into_any()
    } else {
        view! {
            <input
                type=move || effective_type.get()
                disabled=disabled
                readonly=readonly
                required=required
                maxlength=move || optional_numeric_attr(maxlength.get())
                name=move || name.get()
                prop:value=move || value.get()
                placeholder=move || placeholder.get()
                on:input=move |e| {
                    let raw = event_target_value(&e);
                    let filtered = filter.get().apply(&raw);
                    if filtered != raw
                        && let Some(el) = e
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                    {
                        el.set_value(&filtered);
                    }
                    if let Some(cb) = on_input {
                        cb.run(filtered);
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
                        "input ld-focus-ring",
                        style.get().as_str(),
                        color.get().as_str(),
                        size.get().as_str(),
                        class
                    )
                }
            />
        }
        .into_any()
    }
}
