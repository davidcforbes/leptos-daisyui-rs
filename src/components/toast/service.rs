//! Imperative toast notification service.
//!
//! [`provide_toast`] installs a [`ToastService`] into Leptos context; from
//! anywhere deeper in the tree, [`use_toast`] retrieves it to fire
//! notifications programmatically:
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::components::*;
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     provide_toast();
//!
//!     view! {
//!         <ToastHost />
//!         <button on:click=move |_| { use_toast().push("Saved!", ToastVariant::Success); }>
//!             "Save"
//!         </button>
//!     }
//! }
//! ```
//!
//! [`ToastHost`](super::host::ToastHost) renders the live queue as daisyUI
//! `Alert`s inside the existing [`Toast`](super::component::Toast)
//! positioning container, auto-dismissing each entry after its
//! `duration_ms` (`0` = sticky).

use leptos::prelude::*;

use crate::components::alert::AlertColor;

/// Default auto-dismiss duration (ms) used by [`ToastService::push`].
pub const DEFAULT_TOAST_DURATION_MS: u32 = 4000;

/// Semantic variant of a toast entry. Maps 1:1 to an [`AlertColor`] when
/// [`ToastHost`](super::host::ToastHost) renders the entry as an `Alert`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ToastVariant {
    /// No semantic color (plain alert).
    #[default]
    Default,

    /// Informational message.
    Info,

    /// Positive/success feedback.
    Success,

    /// Cautionary message.
    Warning,

    /// Critical/error message.
    Error,
}

impl ToastVariant {
    /// Maps this variant to the daisyUI Alert color that renders it.
    pub fn as_alert_color(&self) -> AlertColor {
        match self {
            ToastVariant::Default => AlertColor::Default,
            ToastVariant::Info => AlertColor::Info,
            ToastVariant::Success => AlertColor::Success,
            ToastVariant::Warning => AlertColor::Warning,
            ToastVariant::Error => AlertColor::Error,
        }
    }
}

/// One queued toast notification.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastEntry {
    /// Monotonic id — used as the `<For>` key and the `dismiss` handle.
    pub id: u64,
    /// Message body rendered inside the Alert.
    pub message: String,
    /// Semantic variant, mapped to an `AlertColor` by the host.
    pub variant: ToastVariant,
    /// Auto-dismiss delay in ms. `0` means sticky (no auto-dismiss).
    pub duration_ms: u32,
}

/// Reactive handle returned by [`provide_toast`] / [`use_toast`]. Cheap to
/// copy; backed by an `RwSignal<Vec<ToastEntry>>` so
/// [`ToastHost`](super::host::ToastHost) re-renders whenever the queue
/// changes.
#[derive(Clone, Copy)]
pub struct ToastService {
    entries: RwSignal<Vec<ToastEntry>>,
    next_id: RwSignal<u64>,
}

impl ToastService {
    /// Creates a fresh service with an empty queue.
    pub fn new() -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(0),
        }
    }

    /// Reactive read of the live queue — used by
    /// [`ToastHost`](super::host::ToastHost).
    pub fn entries(&self) -> RwSignal<Vec<ToastEntry>> {
        self.entries
    }

    /// Pushes a toast using the default duration
    /// ([`DEFAULT_TOAST_DURATION_MS`]). Returns the new entry's id.
    pub fn push(&self, message: impl Into<String>, variant: ToastVariant) -> u64 {
        self.push_with_duration(message, variant, DEFAULT_TOAST_DURATION_MS)
    }

    /// Pushes a toast with an explicit duration in ms. `0` = sticky (no
    /// auto-dismiss — the caller/user must dismiss it). Returns the new
    /// entry's id.
    pub fn push_with_duration(
        &self,
        message: impl Into<String>,
        variant: ToastVariant,
        duration_ms: u32,
    ) -> u64 {
        let id = self.next_id.get_untracked();
        self.next_id.set(id.wrapping_add(1));
        self.entries.update(|v| {
            push_entry(
                v,
                ToastEntry {
                    id,
                    message: message.into(),
                    variant,
                    duration_ms,
                },
            )
        });
        id
    }

    /// Removes a toast by id. No-op if it's already gone.
    pub fn dismiss(&self, id: u64) {
        self.entries.update(|v| dismiss_entry(v, id));
    }

    /// Removes all queued toasts.
    pub fn clear(&self) {
        self.entries.update(|v| v.clear());
    }

    /// Number of active toasts.
    pub fn count(&self) -> usize {
        self.entries.get_untracked().len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.get_untracked().is_empty()
    }
}

impl Default for ToastService {
    fn default() -> Self {
        Self::new()
    }
}

fn push_entry(v: &mut Vec<ToastEntry>, entry: ToastEntry) {
    v.push(entry);
}

fn dismiss_entry(v: &mut Vec<ToastEntry>, id: u64) {
    v.retain(|e| e.id != id);
}

/// Installs a fresh [`ToastService`] into Leptos context. Call once near the
/// app root (typically right before rendering
/// [`ToastHost`](super::host::ToastHost)); descendants call [`use_toast`] to
/// fire notifications.
pub fn provide_toast() -> ToastService {
    let service = ToastService::new();
    provide_context(service);
    service
}

/// Retrieves the [`ToastService`] installed by [`provide_toast`].
///
/// # Panics
/// Panics if no `ToastService` is in context — call `provide_toast()`
/// (e.g. in your app root) first.
pub fn use_toast() -> ToastService {
    use_context::<ToastService>()
        .expect("ToastService not found. Call provide_toast() before use_toast().")
}

/// Attempts to retrieve the [`ToastService`] without panicking.
pub fn try_use_toast() -> Option<ToastService> {
    use_context::<ToastService>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u64, msg: &str, variant: ToastVariant, duration_ms: u32) -> ToastEntry {
        ToastEntry {
            id,
            message: msg.to_string(),
            variant,
            duration_ms,
        }
    }

    // -- pure Vec<ToastEntry> helpers (no timers, no signals) --

    #[test]
    fn push_entry_appends_in_order() {
        let mut v = Vec::new();
        push_entry(&mut v, entry(0, "a", ToastVariant::Info, 1000));
        push_entry(&mut v, entry(1, "b", ToastVariant::Error, 2000));
        assert_eq!(
            v,
            vec![
                entry(0, "a", ToastVariant::Info, 1000),
                entry(1, "b", ToastVariant::Error, 2000),
            ]
        );
    }

    #[test]
    fn dismiss_entry_removes_by_id_only() {
        let mut v = vec![
            entry(0, "a", ToastVariant::Info, 1000),
            entry(1, "b", ToastVariant::Info, 1000),
            entry(2, "c", ToastVariant::Info, 1000),
        ];
        dismiss_entry(&mut v, 1);
        assert_eq!(
            v,
            vec![
                entry(0, "a", ToastVariant::Info, 1000),
                entry(2, "c", ToastVariant::Info, 1000),
            ]
        );
    }

    #[test]
    fn dismiss_entry_unknown_id_is_noop() {
        let mut v = vec![entry(0, "a", ToastVariant::Info, 1000)];
        dismiss_entry(&mut v, 999);
        assert_eq!(v.len(), 1);
    }

    // -- ToastVariant --

    #[test]
    fn toast_variant_default_is_default() {
        assert_eq!(ToastVariant::default(), ToastVariant::Default);
    }

    #[test]
    fn toast_variant_maps_to_alert_color() {
        assert_eq!(ToastVariant::Default.as_alert_color(), AlertColor::Default);
        assert_eq!(ToastVariant::Info.as_alert_color(), AlertColor::Info);
        assert_eq!(ToastVariant::Success.as_alert_color(), AlertColor::Success);
        assert_eq!(ToastVariant::Warning.as_alert_color(), AlertColor::Warning);
        assert_eq!(ToastVariant::Error.as_alert_color(), AlertColor::Error);
    }

    // -- ToastService (RwSignal-backed queue) --

    #[test]
    fn new_service_is_empty() {
        let service = ToastService::new();
        assert!(service.is_empty());
        assert_eq!(service.count(), 0);
    }

    #[test]
    fn push_uses_default_duration() {
        let service = ToastService::new();
        service.push("Hello", ToastVariant::Info);
        let entries = service.entries().get_untracked();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].duration_ms, DEFAULT_TOAST_DURATION_MS);
        assert_eq!(entries[0].message, "Hello");
        assert_eq!(entries[0].variant, ToastVariant::Info);
    }

    #[test]
    fn push_with_duration_honors_explicit_duration() {
        let service = ToastService::new();
        service.push_with_duration("Sticky", ToastVariant::Warning, 0);
        let entries = service.entries().get_untracked();
        assert_eq!(entries[0].duration_ms, 0);
    }

    #[test]
    fn push_returns_new_entry_id() {
        let service = ToastService::new();
        let id = service.push("a", ToastVariant::Info);
        assert_eq!(id, 0);
    }

    #[test]
    fn ids_are_monotonic_across_pushes() {
        let service = ToastService::new();
        let id1 = service.push("a", ToastVariant::Info);
        let id2 = service.push("b", ToastVariant::Info);
        let id3 = service.push("c", ToastVariant::Info);
        assert_eq!([id1, id2, id3], [0, 1, 2]);
    }

    #[test]
    fn ids_keep_advancing_after_a_dismiss() {
        let service = ToastService::new();
        let id1 = service.push("a", ToastVariant::Info);
        service.dismiss(id1);
        let id2 = service.push("b", ToastVariant::Info);
        assert_eq!(id2, 1, "ids must stay monotonic, not reuse dismissed ids");
    }

    #[test]
    fn dismiss_removes_only_the_matching_toast() {
        let service = ToastService::new();
        let id1 = service.push("a", ToastVariant::Info);
        let id2 = service.push("b", ToastVariant::Info);
        service.dismiss(id1);
        assert_eq!(service.count(), 1);
        let entries = service.entries().get_untracked();
        assert_eq!(entries[0].id, id2);
    }

    #[test]
    fn dismiss_unknown_id_is_noop() {
        let service = ToastService::new();
        service.push("a", ToastVariant::Info);
        service.dismiss(999);
        assert_eq!(service.count(), 1);
    }

    #[test]
    fn clear_empties_the_queue() {
        let service = ToastService::new();
        service.push("a", ToastVariant::Info);
        service.push("b", ToastVariant::Error);
        service.clear();
        assert!(service.is_empty());
        assert_eq!(service.count(), 0);
    }

    #[test]
    fn clear_then_push_continues_monotonic_ids() {
        let service = ToastService::new();
        service.push("a", ToastVariant::Info);
        service.push("b", ToastVariant::Info);
        service.clear();
        let id = service.push("c", ToastVariant::Info);
        assert_eq!(id, 2, "clearing entries must not reset the id counter");
    }

    // `provide_context`/`use_context` need a reactive owner (unlike a bare
    // `RwSignal`), so these two run inside one — see
    // `src/markdown/editor_api_leptos.rs` tests for the same pattern.
    fn with_runtime<F: FnOnce()>(f: F) {
        let owner = leptos::reactive::owner::Owner::new();
        owner.with(f);
    }

    #[test]
    fn provide_and_use_toast_round_trip() {
        with_runtime(|| {
            provide_toast();
            let service = use_toast();
            service.push("via context", ToastVariant::Success);
            assert_eq!(use_toast().count(), 1);
        });
    }

    #[test]
    fn try_use_toast_without_provide_is_none() {
        with_runtime(|| {
            assert!(try_use_toast().is_none());
        });
    }
}
