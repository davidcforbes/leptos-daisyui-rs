use super::context::{FieldContext, FieldLineKind, field_line, next_field_id};
use super::style::FieldState;
use crate::merge_classes;
use leptos::{html::Div, prelude::*};

/// # Field Component
///
/// A form field wrapper that provides labels, validation states, error messages,
/// and help text for form inputs. Wraps any form control with consistent styling
/// and validation feedback.
///
/// ## Programmatic association
///
/// The visible label and the message lines are wired to the wrapped control,
/// not just drawn near it: `Field` mints an id, points its `<label for>` at
/// it, and provides a [`FieldContext`] that this crate's `Input`, `Select`
/// and `Textarea` consume automatically — the control gets the `id`, the
/// rendered help/success/warning line is referenced via `aria-describedby`,
/// and the error line via `aria-errormessage` + `aria-invalid="true"` (also
/// mirrored into `aria-describedby`, since screen-reader support for
/// `aria-errormessage` is uneven). A raw child element can read the context
/// with `use_context::<FieldContext>()` and apply the same attributes.
///
/// ### Add to `input.css`
/// ```css
/// @source inline("flex flex-col gap-2 label text-sm text-xs text-error text-success text-warning");
/// ```
///
/// ## Node References
/// - `node_ref` - References the container div element ([HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement))
#[component]
pub fn Field(
    /// Label text displayed above the field
    #[prop(optional, into)]
    label: Signal<Option<String>>,

    /// Help text displayed below the field
    #[prop(optional, into)]
    help_text: Signal<Option<String>>,

    /// Error message displayed when state is Error
    #[prop(optional, into)]
    error: Signal<Option<String>>,

    /// Success message displayed when state is Success
    #[prop(optional, into)]
    success: Signal<Option<String>>,

    /// Validation state (Default, Error, Success, Warning)
    #[prop(optional, into)]
    state: Signal<FieldState>,

    /// Whether the field is required
    #[prop(optional, into)]
    required: Signal<bool>,

    /// Additional CSS classes for the container
    #[prop(optional, into)]
    class: &'static str,

    /// Additional CSS classes for the label
    #[prop(optional, into)]
    label_class: &'static str,

    /// Node reference for the container div
    #[prop(optional)]
    node_ref: NodeRef<Div>,

    /// The form control (input, select, textarea, etc.) to wrap
    children: Children,
) -> impl IntoView {
    // Stable ids for this Field instance: the control's id (label `for`
    // target) and the message line's id (`aria-describedby` /
    // `aria-errormessage` target).
    let input_id = next_field_id();
    let line_id = format!("{input_id}-line");

    // The single message line currently shown, derived through the same pure
    // function the render below uses — the context can't tell assistive
    // technology about a line that isn't drawn.
    let current_line = Signal::derive(move || {
        field_line(state.get(), error.get(), success.get(), help_text.get())
    });

    // Provide the association contract BEFORE `children()` runs, so the
    // wrapped Input/Select/Textarea sees it when it is created.
    {
        let line_id = line_id.clone();
        let described_line_id = line_id.clone();
        provide_context(FieldContext {
            input_id: input_id.clone(),
            described_by: Signal::derive(move || {
                current_line
                    .get()
                    .filter(|(kind, _)| *kind != FieldLineKind::Error)
                    .map(|_| described_line_id.clone())
            }),
            error_id: Signal::derive(move || {
                current_line
                    .get()
                    .filter(|(kind, _)| *kind == FieldLineKind::Error)
                    .map(|_| line_id.clone())
            }),
        });
    }

    let label_for = input_id.clone();
    view! {
        <div
            node_ref=node_ref
            class=move || merge_classes!("flex flex-col gap-2", class)
        >
            {move || {
                let label_for = label_for.clone();
                label
                    .get()
                    .map(|label_text| {
                        view! {
                            <label
                                r#for=label_for
                                class=move || merge_classes!("label", label_class)
                            >
                                <span class="text-sm">
                                    {label_text}
                                    {move || {
                                        if required.get() {
                                            view! { <span class="text-error ml-1">"*"</span> }.into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}

                                </span>
                            </label>
                        }
                            .into_any()
                    })
            }}

            {children()}

            {move || {
                current_line
                    .get()
                    .map(|(kind, msg)| {
                        let text_class = match kind {
                            FieldLineKind::Error => "text-xs text-error",
                            FieldLineKind::Success => "text-xs text-success",
                            FieldLineKind::Warning => "text-xs text-warning",
                            FieldLineKind::Help => "text-xs",
                        };
                        // A <div>, not the former <label>: this text labels
                        // nothing (a label without a control is itself an
                        // a11y smell) — it *describes*, via the id the
                        // context hands to the wrapped control.
                        view! {
                            <div class="label" id=line_id.clone()>
                                <span class=text_class>{msg}</span>
                            </div>
                        }
                    })
            }}

        </div>
    }
}
