//! Programmatic label/description association for [`Field`](super::Field).
//!
//! `Field` renders a visible `<label>` and help/warning/success/error lines,
//! but they used to be bare siblings of the wrapped control: no `for`/`id`,
//! no `aria-describedby`, no `aria-errormessage` — a screen reader could not
//! connect any of that text to the input (office-perf op-99t7/op-cy77.2).
//!
//! `Field` now mints stable element ids and provides a [`FieldContext`];
//! this crate's `Input`, `Select` and `Textarea` pick it up automatically
//! (no call-site changes), so wrapping one of them in a `Field` yields a
//! fully associated control: `label[for]` → `input[id]`, the rendered line
//! referenced via `aria-describedby` (or `aria-errormessage` +
//! `aria-invalid` when it's the error). Raw children can read the context
//! themselves via `use_context::<FieldContext>()`.

use super::style::FieldState;
use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIELD_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique id base for one `Field` instance (`ld-field-0`, …).
/// Monotonic counter, not randomness — stable within a page's lifetime,
/// which is all `for`/`aria-describedby` need.
pub(super) fn next_field_id() -> String {
    format!("ld-field-{}", NEXT_FIELD_ID.fetch_add(1, Ordering::Relaxed))
}

/// Which message line a `Field` is currently showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldLineKind {
    /// The error line ([`FieldState::Error`] with an error message).
    Error,
    /// The success line ([`FieldState::Success`] with a success message).
    Success,
    /// The warning line ([`FieldState::Warning`]; shows the help text).
    Warning,
    /// The plain help line (default state with help text).
    Help,
}

/// The single message line `Field` renders for the given state and texts,
/// if any. One pure function shared by the render path and the
/// [`FieldContext`] signal derivation, so what assistive technology is told
/// can never diverge from what is drawn.
pub fn field_line(
    state: FieldState,
    error: Option<String>,
    success: Option<String>,
    help: Option<String>,
) -> Option<(FieldLineKind, String)> {
    match state {
        FieldState::Error => error.map(|m| (FieldLineKind::Error, m)),
        FieldState::Success => success.map(|m| (FieldLineKind::Success, m)),
        FieldState::Warning => Some((FieldLineKind::Warning, help.unwrap_or_default())),
        FieldState::Default => help.map(|m| (FieldLineKind::Help, m)),
    }
}

/// What a [`Field`](super::Field) provides to the form control it wraps.
///
/// Consumed automatically by this crate's `Input`, `Select` and `Textarea`;
/// a raw child element can apply it by hand:
///
/// ```rust,ignore
/// let field = use_context::<FieldContext>();
/// view! {
///     <input
///         id=field.as_ref().map(|f| f.input_id.clone())
///         aria-describedby=move || field.as_ref().and_then(|f| f.described_by.get())
///     />
/// }
/// ```
#[derive(Clone)]
pub struct FieldContext {
    /// The id the wrapped control should carry; the `Field`'s visible label
    /// points at it via `for`.
    pub input_id: String,
    /// Id of the currently rendered help/success/warning line, for
    /// `aria-describedby`. `None` while no line is shown or while the line
    /// is the error (which travels via `error_id` instead).
    pub described_by: Signal<Option<String>>,
    /// Id of the error line while the field is in [`FieldState::Error`] with
    /// a message — drives `aria-errormessage` and `aria-invalid="true"`.
    /// (Consumers also mirror it into `aria-describedby`: support for
    /// `aria-errormessage` is still uneven across screen readers.)
    pub error_id: Signal<Option<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_state_with_message_is_the_error_line() {
        assert_eq!(
            field_line(
                FieldState::Error,
                Some("Required".into()),
                Some("ok".into()),
                Some("help".into()),
            ),
            Some((FieldLineKind::Error, "Required".to_string()))
        );
    }

    #[test]
    fn error_state_without_message_shows_nothing() {
        assert_eq!(
            field_line(FieldState::Error, None, None, Some("help".into())),
            None
        );
    }

    #[test]
    fn success_state_with_message_is_the_success_line() {
        assert_eq!(
            field_line(FieldState::Success, None, Some("Saved".into()), None),
            Some((FieldLineKind::Success, "Saved".to_string()))
        );
    }

    #[test]
    fn warning_state_shows_the_help_text() {
        assert_eq!(
            field_line(FieldState::Warning, None, None, Some("Careful".into())),
            Some((FieldLineKind::Warning, "Careful".to_string()))
        );
    }

    #[test]
    fn default_state_shows_help_when_present() {
        assert_eq!(
            field_line(FieldState::Default, None, None, Some("Hint".into())),
            Some((FieldLineKind::Help, "Hint".to_string()))
        );
        assert_eq!(field_line(FieldState::Default, None, None, None), None);
    }

    #[test]
    fn field_ids_are_unique() {
        let a = next_field_id();
        let b = next_field_id();
        assert_ne!(a, b);
        assert!(a.starts_with("ld-field-"));
    }
}
