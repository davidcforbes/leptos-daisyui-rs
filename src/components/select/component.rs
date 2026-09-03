use super::style::{SelectColor, SelectSize, SelectStyle};
use crate::merge_classes;
use leptos::{
    html::{Option_, Select as HtmlSelect},
    prelude::*,
};

/// # Select Component
///
/// A reactive Leptos wrapper for daisyUI's select component that provides dropdown
/// selection controls with multiple styling and size options.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("select select-ghost select-primary select-secondary select-accent select-info select-success select-warning select-error select-xs select-sm select-md select-lg select-xl");
/// ```
///
/// ## Node References
/// - `node_ref` - References the top select element ([HTMLSelectElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLSelectElement))
#[component]
pub fn Select(
    /// Style variant of the select
    #[prop(optional, into)]
    style: Signal<SelectStyle>,

    /// Color scheme of the select
    #[prop(optional, into)]
    color: Signal<SelectColor>,

    /// Size of the select
    #[prop(optional, into)]
    size: Signal<SelectSize>,

    /// Whether the select is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// The `name` attribute used when submitting an enclosing form (and the
    /// form-field a11y identifier). Omitted when `None`.
    #[prop(optional, into)]
    name: Signal<Option<String>>,

    /// Stable DOM identity for standalone controls. When omitted, a wrapping
    /// [`Field`](crate::components::Field) supplies its associated input ID.
    #[prop(optional, into)]
    id: MaybeProp<String>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Accessible name for the select (rendered as `aria-label`). The
    /// first-class labeling path is wrapping in a labeled field (e.g.
    /// `FilterField`, which renders a real `<label>`); for a select OUTSIDE
    /// one, pass this — axe's `select-name` rule (serious) fails a select
    /// with neither. Seven such call sites shipped unlabeled in office-perf
    /// before its PixelProof suite caught them, because the only path was
    /// the `attr:aria-label` escape hatch nobody reached for.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Node reference to the select element
    #[prop(optional)]
    node_ref: NodeRef<HtmlSelect>,

    /// The CURRENT selection, driven as the DOM `value` PROPERTY.
    ///
    /// ★ WHY THIS EXISTS, because the obvious alternative is a trap that has
    /// already cost a production bug. Callers were marking the chosen option with
    /// `attr:selected`, which sets the HTML `selected` ATTRIBUTE — and that is only
    /// the PARSE-TIME DEFAULT. Once the element exists, changing that attribute does
    /// not move the browser's current selection, and if the option list is rebuilt
    /// the browser resets to index 0. The result is a select DISPLAYING the first
    /// option while the application state says something else: office-perf showed
    /// one office's figures under an "All Offices" label that way (op-p44s).
    ///
    /// Pass this whenever the selection is driven by state the user did not just
    /// click — restored from storage, changed by another control, or shown beside a
    /// reactively-rebuilt option list. A select the user is the only author of does
    /// not need it.
    ///
    /// Applied through an effect against `node_ref` rather than a `prop:value` in
    /// the markup, so that omitting it changes NOTHING for existing callers — a
    /// `prop:value` defaulting to `""` would force every current select to empty.
    #[prop(optional, into)]
    value: Option<Signal<String>>,

    /// Optional callback invoked with the newly selected value.
    #[prop(optional)]
    on_change: Option<Callback<String>>,

    /// Opaque revision of the option set (`ldui-uxdw`).
    ///
    /// Changing it re-asserts `value` against the DOM. Supply it whenever the
    /// options can arrive or change **after** first render — otherwise the
    /// browser's own choice (index 0) wins and the bound value is silently
    /// lost. `children` is a `Children` closure, not a signal, so the
    /// component cannot detect this by itself; the caller has to say.
    ///
    /// Any value that changes with the option set works: a length, a hash, a
    /// join of the option keys.
    #[prop(optional, into)]
    options_revision: Option<Signal<String>>,

    /// Child elements (typically SelectOption components)
    children: Children,
) -> impl IntoView {
    // Re-assert the DOM property whenever the bound signal changes. Runs after
    // render, so it wins over whatever the browser chose when the options were
    // (re)created.
    //
    // `ldui-uxdw`: that promise used to be false for ASYNC options, and the
    // comment above was the tell. The effect tracked only `value`, so:
    //   1. first render, options empty -> set_value finds no such <option>,
    //      the browser keeps index 0
    //   2. options arrive, children re-render, the browser selects index 0
    //   3. the effect does NOT re-run, because `value` never changed
    // The selection was then silently wrong -- a real office name in the
    // caption, the alphabetically-first one in the control.
    //
    // Tracking `options_revision` closes it: the caller names when the option
    // set changed, and this effect re-asserts after the browser has rebuilt
    // the list.
    if let Some(v) = value {
        Effect::new(move |_| {
            // Read FIRST and unconditionally, so it is a tracked dependency
            // even on the render where the value happens to be unchanged.
            // A `let _ =` that some future cleanup "simplifies" away silently
            // restores the bug, which is why it is spelled out here.
            if let Some(revision) = options_revision {
                let _options_changed = revision.get();
            }
            let want = v.get();
            if let Some(el) = node_ref.get() {
                el.set_value(&want);
            }
        });
    }
    // Wrapped in a `Field`? Pick up its association contract (id for the
    // visible label's `for`, help/error line ids) — same wiring as `Input`.
    let field = use_context::<crate::components::field::FieldContext>();
    let field_id = field.as_ref().map(|f| f.input_id.clone());
    let field_for_desc = field.clone();
    let described_by = move || {
        field_for_desc
            .as_ref()
            .and_then(|f| f.described_by.get().or_else(|| f.error_id.get()))
    };
    let field_for_err = field.clone();
    let error_message = move || field_for_err.as_ref().and_then(|f| f.error_id.get());
    let field_for_invalid = field.clone();
    let aria_invalid = move || {
        field_for_invalid
            .as_ref()
            .and_then(|f| f.error_id.get().map(|_| "true"))
    };
    view! {
        <select
            aria-label=move || label.get()
            id=move || id.get().or_else(|| field_id.clone())
            aria-describedby=described_by
            aria-errormessage=error_message
            aria-invalid=aria_invalid
            node_ref=node_ref
            class=move || {
                merge_classes!(
                    "select ld-focus-ring",
                style.get().as_str(),
                color.get().as_str(),
                size.get().as_str(),
                class
                )
            }
            name=move || name.get()
            disabled=disabled
            on:change=move |event| {
                if let Some(callback) = on_change {
                    callback.run(event_target_value(&event));
                }
            }
        >
            {children()}
        </select>
    }
}

/// Option element for Select component.
///
/// ## Node References
/// - `node_ref` - References the top option element ([HTMLOptionElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLOptionElement))
#[component]
pub fn SelectOption(
    /// Whether the option is disabled
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference to the option element
    #[prop(optional)]
    node_ref: NodeRef<Option_>,

    /// Content of the option
    children: Children,
) -> impl IntoView {
    view! {
        <option node_ref=node_ref class=class disabled=disabled>
            {children()}
        </option>
    }
}
