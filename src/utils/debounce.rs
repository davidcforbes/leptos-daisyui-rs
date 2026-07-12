//! Trailing-debounce signal pair -- extracted from the search-input pattern
//! duplicated identically across `components::data_table::component`
//! (`DataTable`) and `components::data_table::server_component`
//! (`ServerDataTable`), and re-derived a third time by a host app
//! (`inventory-web`'s Explorer screen) because no reusable hook was exported
//! (framework-purity audit finding #4, `bd_4iiz-inventory-xsa`).

use leptos::prelude::*;
use std::time::Duration;

/// A `(raw, debounced)` signal pair sharing one pending-timer slot.
///
/// Call [`DebouncedSignal::set`] on every raw update (typically from an
/// `on:input` handler): `raw` reflects it immediately -- bind it to the
/// input element so typing stays instantly responsive -- while `debounced`
/// only catches up `delay_ms` later, unless another `set` call arrives
/// first, which cancels the pending timer and restarts the delay. This is a
/// standard trailing/restart-on-every-update debounce, coalescing a burst of
/// rapid updates (e.g. keystrokes) into one downstream reaction (a
/// filter/search re-fetch).
///
/// `T` must be `Clone` (the pending value is captured by the timer closure)
/// and `Send + Sync + 'static` -- the same bounds Leptos signals themselves
/// require.
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_daisyui_rs::utils::use_debounced_signal;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let search = use_debounced_signal(String::new(), 300);
///     let on_input = move |ev: leptos::ev::Event| {
///         search.set(event_target_value(&ev));
///     };
///
///     // `search.raw` for the controlled input's `value`, `search.debounced`
///     // for the signal a `LocalResource` fetch actually reads.
///     view! {
///         <input prop:value=move || search.raw.get() on:input=on_input />
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DebouncedSignal<T: 'static> {
    /// The immediately-updated value -- bind this to the input element so
    /// typing feels instant.
    pub raw: ReadSignal<T>,
    /// The value `delay_ms` after the last [`DebouncedSignal::set`] (or
    /// [`DebouncedSignal::set_immediate`]) call -- read this from the
    /// fetch/effect that should be coalesced, not `raw`.
    pub debounced: ReadSignal<T>,
    set_raw: WriteSignal<T>,
    set_debounced: WriteSignal<T>,
    handle: RwSignal<Option<TimeoutHandle>>,
    delay_ms: u64,
}

// Manual Clone/Copy (not derived): every field is itself `Copy` regardless
// of `T` (Leptos signals are index handles into the reactive graph, not the
// value itself), so `DebouncedSignal<T>` should be too, the same way
// `ReadSignal<T>`/`WriteSignal<T>` are `Copy` without requiring `T: Copy`. A
// derived `Clone`/`Copy` would incorrectly add a `T: Clone`/`T: Copy` bound.
impl<T> Clone for DebouncedSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DebouncedSignal<T> {}

impl<T: Clone + Send + Sync + 'static> DebouncedSignal<T> {
    /// Update `raw` immediately and schedule `debounced` to catch up after
    /// `delay_ms`, cancelling any timer still pending from a previous `set`
    /// call first. If scheduling the timer fails outright (no `window` --
    /// doesn't happen in a browser tab, but this crate also compiles for
    /// native `cargo test`), fails open: applies the value to `debounced`
    /// immediately rather than silently dropping the update.
    pub fn set(&self, value: T) {
        self.set_raw.set(value.clone());
        self.cancel();
        let set_debounced = self.set_debounced;
        let value_for_timeout = value.clone();
        match set_timeout_with_handle(
            move || {
                set_debounced.set(value_for_timeout);
            },
            Duration::from_millis(self.delay_ms),
        ) {
            Ok(h) => self.handle.set(Some(h)),
            Err(_) => {
                set_debounced.set(value);
                self.handle.set(None);
            }
        }
    }

    /// Apply `value` to both `raw` and `debounced` immediately, cancelling
    /// any pending timer -- for programmatic updates (e.g. syncing from the
    /// URL) that should bypass the debounce delay entirely rather than race
    /// a keystroke's in-flight timer.
    pub fn set_immediate(&self, value: T) {
        self.cancel();
        self.set_raw.set(value.clone());
        self.set_debounced.set(value);
    }

    /// Cancel any pending debounce timer without changing either signal --
    /// call from `on_cleanup` so an unmounted component's timer doesn't fire
    /// into a dropped signal.
    pub fn cancel(&self) {
        if let Some(h) = self.handle.get_untracked() {
            h.clear();
        }
        self.handle.set(None);
    }
}

/// Creates a [`DebouncedSignal`] seeded with `initial`, restarting its
/// `delay_ms`-millisecond trailing debounce on every [`DebouncedSignal::set`]
/// call.
pub fn use_debounced_signal<T>(initial: T, delay_ms: u64) -> DebouncedSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    let (raw, set_raw) = signal(initial.clone());
    let (debounced, set_debounced) = signal(initial);
    let handle = RwSignal::new(None::<TimeoutHandle>);
    DebouncedSignal {
        raw,
        debounced,
        set_raw,
        set_debounced,
        handle,
        delay_ms,
    }
}
