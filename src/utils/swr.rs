//! Stale-while-revalidate keyed resource cache -- extracted from the
//! `WallCache` a host app (`inventory-web`'s Lifecycle screen) hand-rolled
//! because no reusable primitive was exported, and which its drilldown screens
//! were about to re-derive a second and third time (same framework-purity
//! finding as [`crate::utils::debounce`]).
//!
//! ## The problem it solves
//!
//! A `LocalResource` is re-created every time its screen re-mounts, so
//! back-navigating to a route re-fetches from scratch and shows a spinner over
//! data the user was looking at seconds ago. Caching the last value *outside*
//! the route -- keyed by the params it was fetched with -- lets the screen
//! render instantly on return while a background fetch revalidates. On the
//! originating screen this took back-navigation from ~2270ms to ~142ms with no
//! spinner.
//!
//! ## Usage
//!
//! Provide the cache once, above the router, so it survives route changes:
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::utils::provide_swr_cache;
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     provide_swr_cache();
//!     // ... <Router/> ...
//! }
//! ```
//!
//! Then, on a screen:
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_daisyui_rs::utils::{SwrState, use_swr_resource};
//! # #[derive(Clone)] struct Dto { rows: usize }
//! # async fn fetch_json(_url: &str) -> Result<Dto, String> { unimplemented!() }
//!
//! #[component]
//! fn LifecyclePage() -> impl IntoView {
//!     let (period, _set_period) = signal("month".to_string());
//!
//!     let wall = use_swr_resource(
//!         move || format!("/api/lifecycle?period={}", period.get()),
//!         |key: String| async move { fetch_json(&key).await },
//!     );
//!
//!     view! {
//!         {move || match wall.state.get() {
//!             SwrState::Loading => view! { <span class="loading loading-spinner" /> }.into_any(),
//!             SwrState::Error(e) => view! { <div class="alert alert-error">{e}</div> }.into_any(),
//!             SwrState::Ready { data, revalidating } => view! {
//!                 <div class:opacity-70=revalidating>{data.rows}</div>
//!             }.into_any(),
//!         }}
//!     }
//! }
//! ```
//!
//! The fetch layer stays pluggable: this module never performs a request. It
//! takes any async `Fn(String) -> Future<Output = Result<T, E>>`, so
//! `gloo-net`, `reqwest`, or a hand-rolled `fetch` binding all work.

use leptos::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

/// What a screen should render for one [`use_swr_resource`] call.
///
/// The three states are exhaustive and mutually exclusive, so a `match` on this
/// cannot forget the cached-but-stale case the way an ad-hoc
/// `Option<Result<_, _>>` match does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwrState<T, E> {
    /// First load for this key: nothing cached and a fetch is in flight. This
    /// is the only state that should show a spinner.
    Loading,
    /// There is a value to render.
    Ready {
        /// The value -- fresh from the fetch, or the cached value for this
        /// exact key while a fetch is in flight.
        data: T,
        /// `true` when `data` came from the cache and a background fetch is
        /// still running, i.e. it may be stale. Use it for a subtle affordance
        /// (a dimmed panel, a small spinner in the corner) -- never to swap in
        /// a full-page spinner, which would defeat the point.
        revalidating: bool,
    },
    /// The fetch failed and no cached value exists for this key.
    ///
    /// A failed *revalidation* over a cached value also lands here rather than
    /// silently rendering the stale value: an error the user never sees is an
    /// error they cannot act on.
    Error(E),
}

/// App-wide stale-while-revalidate cache, keyed by string.
///
/// Type-erased so one context serves every screen regardless of DTO type.
/// A key stored by one type and read as another simply misses (the downcast
/// fails), which degrades to "no cache" rather than misrendering.
#[derive(Clone, Copy)]
pub struct SwrCache {
    entries: RwSignal<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl Default for SwrCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SwrCache {
    /// An empty cache. Prefer [`provide_swr_cache`], which puts one in context
    /// where it can outlive a route.
    pub fn new() -> Self {
        Self {
            entries: RwSignal::new(HashMap::new()),
        }
    }

    /// The cached value for `key`, if one was stored under this exact key *and*
    /// this type.
    ///
    /// Reads reactively: a component reading this re-runs when the cache is
    /// written.
    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.entries.with(|m| {
            m.get(key)
                .and_then(|v| v.clone().downcast::<T>().ok())
                .map(|v| (*v).clone())
        })
    }

    /// As [`get`](Self::get), without subscribing the caller to cache writes.
    pub fn get_untracked<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.entries.with_untracked(|m| {
            m.get(key)
                .and_then(|v| v.clone().downcast::<T>().ok())
                .map(|v| (*v).clone())
        })
    }

    /// Store `value` under `key`, replacing any previous entry.
    pub fn set<T: Send + Sync + 'static>(&self, key: impl Into<String>, value: T) {
        self.entries.update(|m| {
            m.insert(key.into(), Arc::new(value) as Arc<dyn Any + Send + Sync>);
        });
    }

    /// Forget one key. Use after a mutation that invalidates exactly that
    /// entry, so the next visit refetches instead of flashing a wrong value.
    pub fn invalidate(&self, key: &str) {
        self.entries.update(|m| {
            m.remove(key);
        });
    }

    /// Forget everything -- e.g. on logout, so the next user never sees the
    /// previous one's cached data.
    pub fn clear(&self) {
        self.entries.update(|m| m.clear());
    }

    /// Number of cached entries (across all types).
    pub fn len(&self) -> usize {
        self.entries.with(|m| m.len())
    }

    /// Whether the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.with(|m| m.is_empty())
    }
}

/// Put an [`SwrCache`] in context and return it.
///
/// Call this **above the router**. A cache provided inside a route is destroyed
/// with that route, which is exactly the re-mount this primitive exists to
/// survive.
pub fn provide_swr_cache() -> SwrCache {
    let cache = SwrCache::new();
    provide_context(cache);
    cache
}

/// The [`SwrCache`] from context, or a fresh component-local one if no
/// provider exists.
///
/// The fallback keeps a screen working standalone (tests, a storybook) at the
/// cost of the cross-route caching -- a local cache dies with the component, so
/// every mount misses. If back-navigation is not instant, check that
/// [`provide_swr_cache`] is called above the router.
pub fn use_swr_cache() -> SwrCache {
    use_context::<SwrCache>().unwrap_or_default()
}

/// Handle returned by [`use_swr_resource`].
pub struct SwrResource<T: Send + Sync + 'static, E: Send + Sync + 'static> {
    /// What to render: cached-or-loading view state plus the revalidating flag.
    pub state: Signal<SwrState<T, E>>,
    /// `true` while a background fetch is in flight over a cached value.
    /// Convenience mirror of [`SwrState::Ready::revalidating`] for callers that
    /// want it independently of the match (e.g. to disable a Refresh button).
    pub revalidating: Signal<bool>,
    /// The key the current value was fetched with -- handy for
    /// [`SwrCache::invalidate`].
    pub key: Memo<String>,
    /// Nonce bumped by [`refetch`](Self::refetch).
    reload: RwSignal<u32>,
    cache: SwrCache,
}

// `#[derive(Clone, Copy)]` would demand `T: Clone`/`T: Copy`, but every field
// is already `Copy` regardless of `T` -- the type parameters appear only inside
// signals. Implement both by hand so a handle can be captured by more than one
// closure.
impl<T: Send + Sync + 'static, E: Send + Sync + 'static> Clone for SwrResource<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static, E: Send + Sync + 'static> Copy for SwrResource<T, E> {}

impl<T: Clone + Send + Sync + 'static, E: Send + Sync + 'static> SwrResource<T, E> {
    /// Re-run the fetch for the current key, even if the key has not changed
    /// (e.g. a Refresh button, or after a mutation).
    ///
    /// The cached value keeps rendering while the refetch runs -- the state
    /// goes to `Ready { revalidating: true }`, not back to `Loading`.
    pub fn refetch(&self) {
        self.reload.update(|n| *n = n.wrapping_add(1));
    }

    /// Drop the cached value for the current key *and* refetch, so the next
    /// render shows [`SwrState::Loading`] rather than a value known to be
    /// wrong. Use after a mutation that invalidates this exact entry.
    pub fn invalidate_and_refetch(&self) {
        self.cache.invalidate(&self.key.get_untracked());
        self.refetch();
    }
}

/// Resolve the view state from a fetch result and whatever is cached.
///
/// The whole decision table of the hook, kept pure so it is unit-testable
/// without a DOM, a resource, or an async runtime:
///
/// | fetch | cached | state |
/// |---|---|---|
/// | `Some(Ok(v))` | any | `Ready { v, revalidating: false }` |
/// | `Some(Err(e))` | any | `Error(e)` |
/// | `None` (in flight) | `Some(v)` | `Ready { v, revalidating: true }` |
/// | `None` (in flight) | `None` | `Loading` |
pub fn swr_state_from<T, E>(fetch: Option<Result<T, E>>, cached: Option<T>) -> SwrState<T, E> {
    match fetch {
        Some(Ok(data)) => SwrState::Ready {
            data,
            revalidating: false,
        },
        Some(Err(e)) => SwrState::Error(e),
        None => match cached {
            Some(data) => SwrState::Ready {
                data,
                revalidating: true,
            },
            None => SwrState::Loading,
        },
    }
}

/// Stale-while-revalidate resource: renders the cached value for `key`
/// instantly while a fetch revalidates it in the background.
///
/// `key` derives a cache key from reactive state (typically the request URL and
/// its params). Changing it starts a new fetch and looks up a different cache
/// entry -- so a value cached under the old key never renders for the new one.
/// Make it capture *every* param the fetch depends on, or a screen will show
/// another param's data.
///
/// `fetcher` receives the current key and returns the future. It is called
/// again whenever the key changes or [`SwrResource::refetch`] is invoked.
///
/// Requires an [`SwrCache`] in context (see [`provide_swr_cache`]) for the
/// cross-route caching that is the point of this hook; without one it still
/// works, but every mount misses.
pub fn use_swr_resource<T, E, Fut>(
    key: impl Fn() -> String + Send + Sync + 'static,
    fetcher: impl Fn(String) -> Fut + 'static,
) -> SwrResource<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    let cache = use_swr_cache();
    let reload = RwSignal::new(0u32);
    let key = Memo::new(move |_| key());

    // The resource resolves to `(key_it_was_fetched_with, result)`, never a bare
    // result. A Leptos resource keeps serving its *previous* value while it
    // re-runs, so after the key changes `resource.get()` still returns the old
    // key's data -- and rendering that verbatim showed one key's payload under
    // another key's heading, labelled fresh (observed: switching to an uncached
    // `quarter` rendered `month`'s payload for 1.2s). Tagging the value with its
    // key is what lets the reader below tell "loaded" from "loaded something
    // else".
    let resource = LocalResource::new(move || {
        let k = key.get();
        // Read the nonce so `refetch()` re-runs the fetch even when the key is
        // unchanged (value itself unused).
        let _ = reload.get();
        let fut = fetcher(k.clone());
        async move { (k, fut.await) }
    });

    // Cache every successful fetch under the key it was actually fetched with --
    // taken from the resource's own tag, not from `key` now, which may already
    // have moved on. This is a side effect and so belongs in an `Effect`, not in
    // the `state` signal below: a signal that wrote to the cache would make
    // rendering order observable.
    Effect::new(move |_| {
        if let Some((fetched_key, Ok(data))) = resource.get() {
            cache.set(fetched_key, data);
        }
    });

    let state = Signal::derive(move || {
        let current = key.get();
        // Discard a resolved value belonging to a previous key: for the current
        // key, that fetch has not happened.
        let fetch = resource
            .get()
            .and_then(|(k, result)| (k == current).then_some(result));
        // Only consult the cache while this key's fetch is pending, and only
        // under the current key -- a hit under the previous key is another
        // screen's data as far as this render is concerned.
        let cached = if fetch.is_none() {
            cache.get::<T>(&current)
        } else {
            None
        };
        swr_state_from(fetch, cached)
    });

    let revalidating = Signal::derive(move || {
        matches!(
            state.get(),
            SwrState::Ready {
                revalidating: true,
                ..
            }
        )
    });

    SwrResource {
        state,
        revalidating,
        key,
        reload,
        cache,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type State = SwrState<&'static str, &'static str>;

    #[test]
    fn fresh_ok_is_ready_and_not_revalidating() {
        assert_eq!(
            swr_state_from(Some(Ok("fresh")), None),
            State::Ready {
                data: "fresh",
                revalidating: false
            }
        );
    }

    #[test]
    fn fresh_ok_wins_over_the_cached_value() {
        assert_eq!(
            swr_state_from(Some(Ok("fresh")), Some("stale")),
            State::Ready {
                data: "fresh",
                revalidating: false
            }
        );
    }

    #[test]
    fn pending_with_cache_renders_the_cached_value_as_revalidating() {
        // The whole point: no spinner on return, but the staleness is visible.
        assert_eq!(
            swr_state_from(None, Some("stale")),
            State::Ready {
                data: "stale",
                revalidating: true
            }
        );
    }

    #[test]
    fn pending_without_cache_is_loading() {
        // First load is the only spinner.
        assert_eq!(swr_state_from(None, None), State::Loading);
    }

    #[test]
    fn error_without_cache_is_an_error() {
        assert_eq!(
            swr_state_from(Some(Err("boom")), None),
            State::Error("boom")
        );
    }

    #[test]
    fn error_surfaces_even_when_a_cached_value_exists() {
        // A failed revalidation must not be hidden behind stale data.
        assert_eq!(
            swr_state_from(Some(Err("boom")), Some("stale")),
            State::Error("boom")
        );
    }

    // A resolved value belonging to a *previous* key must reach `swr_state_from`
    // as `None`, not as that key's data -- see the key-tagging in
    // `use_swr_resource`. These pin the two outcomes that depended on it.

    #[test]
    fn a_previous_keys_value_must_not_render_as_this_keys_fresh_data() {
        // What the hook passes once it has discarded the stale-key value and the
        // new key has nothing cached: a spinner, not the old key's payload.
        assert_eq!(swr_state_from(None, None), State::Loading);
    }

    #[test]
    fn a_previous_keys_value_falls_back_to_this_keys_cache_when_it_has_one() {
        assert_eq!(
            swr_state_from(None, Some("this key's cached value")),
            State::Ready {
                data: "this key's cached value",
                revalidating: true
            }
        );
    }

    #[test]
    fn loading_is_the_only_state_without_data_or_error() {
        for (fetch, cached) in [
            (Some(Ok("v")), None),
            (Some(Ok("v")), Some("c")),
            (None, Some("c")),
        ] {
            assert!(
                !matches!(swr_state_from(fetch, cached), State::Loading),
                "anything with a renderable value must not be Loading"
            );
        }
    }
}
