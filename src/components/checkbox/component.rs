use super::state::{
    CheckboxBinding, CheckboxChangeProposal, CheckboxOwnership, next_checkbox_control_id,
    resolve_checkbox_id, resolve_checkbox_name, resolve_checkbox_ownership,
};
use super::style::{CheckboxColor, CheckboxSize};
use crate::merge_classes;
use leptos::{html::Input as HtmlInput, prelude::*};

/// # Checkbox Component
///
/// A daisyUI checkbox that can either stay natively uncontrolled (the default,
/// unchanged) or take part in controlled application state through one atomic
/// change proposal.
///
/// ## Uncontrolled (default)
///
/// With no `binding`, the browser owns the checked state exactly as it always
/// did. `default_checked` seeds the initial value; nothing else is written.
///
/// ```rust,ignore
/// view! {
///     <Checkbox />
///     <Checkbox color=CheckboxColor::Primary default_checked=true />
/// }
/// ```
///
/// ## Controlled
///
/// Supply a [`CheckboxBinding`]: the caller holds accepted truth as a
/// `Signal`, and every click or Space emits exactly one
/// [`CheckboxChangeProposal`]. The rendered state follows *only* the accepted
/// signal, so a caller may delay or decline a proposal without the checkbox
/// drifting away from what was actually accepted. No `node_ref`, no DOM
/// synchronisation, no second source of truth.
///
/// ```rust,ignore
/// let past_due_only = RwSignal::new(false);
///
/// view! {
///     <Checkbox
///         id="past-due-only"
///         label=Signal::derive(move || t("filters.past_due_only"))
///         binding=CheckboxBinding::controlled(
///             past_due_only.into(),
///             Callback::new(move |p: CheckboxChangeProposal| past_due_only.set(p.checked)),
///         )
///     />
/// }
/// ```
///
/// A rejected proposal is a visual no-op: the browser toggles the element
/// natively *before* any handler runs, so the change handler re-asserts the
/// accepted value onto the element before it proposes anything.
///
/// ## Indeterminate
///
/// Declare it on the binding with `CheckboxBinding::with_indeterminate`. There
/// is no `indeterminate` content attribute — it is a DOM property only — and
/// the browser clears the flag while handling a click, so the component writes
/// it as a property on render *and* re-asserts it in the change handler. Mixed
/// state is additionally announced as `aria-checked="mixed"`, and a gesture
/// from mixed proposes `true`.
///
/// ## Identity and labelling
///
/// `id` and `name` follow the same scheme as the table controls (`ldui-j6sh`):
/// a caller-supplied value wins, otherwise the id a surrounding
/// [`Field`](crate::components::Field) minted, otherwise — only when the
/// component needs an id of its own — a process-unique minted one. A supplied
/// `id` also becomes the `name` when no `name` was given; a *minted* id never
/// does, because a mount-order-dependent form key would silently change what
/// the form submits.
///
/// `label` renders visible text beside the box inside a wrapping `<label>`;
/// `aria_label` names a checkbox that has no visible text. Supplying both is
/// refused rather than resolved — different visible and accessible names is a
/// WCAG 2.5.3 failure. Both are reactive, so swapping locales replaces the
/// name in place.
///
/// > **Note:** `label` is *structural* — its presence is read once when the
/// > component is created, like `Input`'s `leading_icon` — and when it is
/// > present the component's root element is the wrapping `<label>`, so spread
/// > attributes land there rather than on the input. Use the typed props in
/// > that configuration.
///
/// ## Refused configurations
///
/// Supplying `binding` together with `default_checked`, or `label` together
/// with `aria_label`, renders a visible `role="alert"` panel and no input at
/// all, following [`ServerDataTable`](crate::components::ServerDataTable)
/// rather than `EntityTable`'s panic: a checkbox is a leaf control that may be
/// rendered hundreds of times in a list, and a panic in a CSR wasm app takes
/// the whole page down with it.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("checkbox checkbox-primary checkbox-secondary checkbox-accent checkbox-neutral checkbox-success checkbox-warning checkbox-info checkbox-error");
/// @source inline("checkbox-xs checkbox-sm checkbox-md checkbox-lg checkbox-xl");
/// @source inline("flex items-center gap-2 cursor-pointer cursor-not-allowed text-base-content/75 text-error");
/// ```
///
/// ## Node References
/// - `node_ref` - References the `<input>` element ([HTMLInputElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement))
#[component]
pub fn Checkbox(
    /// Color variant for the checkbox (reactive)
    #[prop(optional, into)]
    color: Signal<CheckboxColor>,

    /// Size variant for the checkbox (reactive)
    #[prop(optional, into)]
    size: Signal<CheckboxSize>,

    /// Whether the checkbox is disabled. A disabled checkbox emits no
    /// proposals.
    #[prop(optional, into)]
    disabled: Signal<bool>,

    /// Stable DOM `id`. Wins over a surrounding `Field`'s minted id and over
    /// the component's own mint; becomes the `name` when no `name` is given.
    #[prop(optional, into)]
    id: MaybeProp<String>,

    /// Form `name`, passed through verbatim — a form key is the server's
    /// vocabulary, not an HTML id.
    #[prop(optional, into)]
    name: MaybeProp<String>,

    /// Visible label text rendered beside the box. Reactive, so a locale
    /// change replaces it in place. Its presence is structural (read once when
    /// the component is created) and switches the root element to a wrapping
    /// `<label>`.
    #[prop(optional, into)]
    label: MaybeProp<String>,

    /// Accessible name for a checkbox with no visible text. Mutually exclusive
    /// with `label`.
    #[prop(optional, into)]
    aria_label: MaybeProp<String>,

    /// Uncontrolled initial checked state. Mutually exclusive with `binding`.
    #[prop(optional, into)]
    default_checked: MaybeProp<bool>,

    /// Opt-in controlled ownership: the caller's accepted value plus the
    /// callback that receives one proposal per gesture. See
    /// [`CheckboxBinding`].
    #[prop(optional, into)]
    binding: Option<CheckboxBinding>,

    /// Additional CSS classes
    #[prop(optional, into)]
    class: &'static str,

    /// Node reference for the input element
    #[prop(optional)]
    node_ref: NodeRef<HtmlInput>,
) -> impl IntoView {
    let has_label = label.get_untracked().is_some();
    let has_aria_label = aria_label.get_untracked().is_some();
    let has_default_checked = default_checked.get_untracked().is_some();

    let ownership = match resolve_checkbox_ownership(
        binding.is_some(),
        has_default_checked,
        has_label,
        has_aria_label,
    ) {
        Ok(ownership) => ownership,
        Err(message) => {
            // Fail closed and visibly: no input is rendered at all, so no
            // ambiguously-owned value can be read back or submitted.
            return view! {
                <span class=merge_classes!("text-error", class) role="alert" data-checkbox-config-error=message>
                    {message}
                </span>
            }
            .into_any();
        }
    };

    // Wrapped in a `Field`? Pick up its association contract, exactly as
    // `Input`/`Select`/`Textarea` do: the id its visible label points at, plus
    // the ids of the currently rendered help/error lines.
    let field = use_context::<crate::components::field::FieldContext>();
    let field_id = field.as_ref().map(|f| f.input_id.clone());
    let field_for_desc = field.clone();
    let described_by = move || {
        field_for_desc.as_ref().and_then(|f| {
            // aria-errormessage support is uneven across screen readers, so
            // the error line is mirrored into aria-describedby too.
            f.described_by.get().or_else(|| f.error_id.get())
        })
    };
    let field_for_err = field.clone();
    let error_message = move || field_for_err.as_ref().and_then(|f| f.error_id.get());
    let field_for_invalid = field;
    let aria_invalid = move || {
        field_for_invalid
            .as_ref()
            .and_then(|f| f.error_id.get().map(|_| "true"))
    };

    // Minted ONCE per instance: minting inside the reactive closure would hand
    // the control a new id on every re-render.
    let minted_id = next_checkbox_control_id();
    let resolved_id = Signal::derive(move || {
        resolve_checkbox_id(id.get(), field_id.clone(), has_label, &minted_id)
    });
    let resolved_name = Signal::derive(move || resolve_checkbox_name(name.get(), id.get()));

    let input_class = move || {
        merge_classes!(
            "checkbox ld-eased ld-focus-ring",
            color.get().as_str(),
            size.get().as_str(),
            class
        )
    };

    let input_view = match ownership {
        CheckboxOwnership::Controlled => {
            let model = binding.expect("controlled ownership is only reachable with a binding");
            let accepted_state = Signal::derive(move || model.state());
            view! {
                <input
                    type="checkbox"
                    node_ref=node_ref
                    id=move || resolved_id.get()
                    name=move || resolved_name.get()
                    disabled=disabled
                    aria-label=move || aria_label.get()
                    aria-describedby=described_by
                    aria-errormessage=error_message
                    aria-invalid=aria_invalid
                    // Emitted ONLY for mixed. A native checkbox already
                    // computes true/false correctly, and restating it would
                    // give assistive technology a second copy to contradict.
                    aria-checked=move || {
                        let state = accepted_state.get();
                        state.is_indeterminate().then(|| state.aria_checked())
                    }
                    data-checkbox-state=move || accepted_state.get().as_str()
                    class=input_class
                    prop:checked=move || accepted_state.get().is_checked()
                    // `indeterminate` has no HTML attribute at all -- it is a
                    // DOM property only, so it MUST be written as one or the
                    // mixed state is simply never exposed.
                    prop:indeterminate=move || accepted_state.get().is_indeterminate()
                    on:change=move |_| {
                        let accepted = model.state_untracked();
                        // Controlled: re-assert accepted truth on the element
                        // the browser just toggled, BEFORE proposing. A
                        // declined or delayed proposal then leaves no
                        // optimistic divergence to reconcile, and the browser's
                        // click-time clearing of `indeterminate` is undone.
                        if let Some(input) = node_ref.get_untracked() {
                            input.set_checked(accepted.is_checked());
                            input.set_indeterminate(accepted.is_indeterminate());
                        }
                        if disabled.get_untracked() {
                            return;
                        }
                        model.on_change.run(CheckboxChangeProposal::from_state(accepted));
                    }
                />
            }
            .into_any()
        }
        // Deliberately carries neither a change handler nor a checked
        // property: either one would fight a caller's own spread handler or its
        // spread `checked` seed. With none of the new props supplied this
        // renders byte-for-byte what it always did.
        //
        // ── uncontrolled ──
        CheckboxOwnership::Uncontrolled => view! {
            <input
                type="checkbox"
                node_ref=node_ref
                id=move || resolved_id.get()
                name=move || resolved_name.get()
                disabled=disabled
                checked=move || default_checked.get().unwrap_or(false)
                aria-label=move || aria_label.get()
                aria-describedby=described_by
                aria-errormessage=error_message
                aria-invalid=aria_invalid
                class=input_class
            />
        }
        .into_any(),
        // ── end uncontrolled ──
    };

    if !has_label {
        return input_view;
    }

    view! {
        <label
            class=move || {
                if disabled.get() {
                    "flex items-center gap-2 cursor-not-allowed text-base-content/75"
                } else {
                    "flex items-center gap-2 cursor-pointer"
                }
            }
            r#for=move || resolved_id.get()
        >
            {input_view}
            <span data-checkbox-label="true">{move || label.get()}</span>
        </label>
    }
    .into_any()
}
