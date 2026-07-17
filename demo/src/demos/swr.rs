use crate::core::{ContentLayout, Section};
use leptos::prelude::*;
use leptos_daisyui_rs::utils::{SwrState, use_swr_resource};

/// Stand-in for a real HTTP call: resolves after `SIMULATED_LATENCY_MS` so the
/// Loading -> Ready and the cached-instant paths are both observable by eye.
/// `use_swr_resource` never performs a request itself -- any async
/// `Fn(String) -> Future<Output = Result<T, E>>` works here.
const SIMULATED_LATENCY_MS: i32 = 1200;

/// Bumped on every fetch so the payload visibly differs between a cached render
/// and a revalidated one.
static FETCH_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

async fn fake_fetch(key: String) -> Result<String, String> {
    // A JS promise resolved by `setTimeout` -- the smallest way to await real
    // wall-clock time without pulling in a timer crate.
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                SIMULATED_LATENCY_MS,
            );
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

    let n = FETCH_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    Ok(format!("payload for {key} (fetch #{n})"))
}

/// The screen under test. Mounting/unmounting it via the toggle below stands in
/// for a route re-mount: its `LocalResource` is destroyed and rebuilt, but the
/// `SwrCache` lives above it and survives.
#[component]
fn CachedScreen() -> impl IntoView {
    let (key, set_key) = signal("month".to_string());

    let res = use_swr_resource(move || key.get(), fake_fetch);

    view! {
        <div class="flex flex-col gap-3" data-testid="swr-screen">
            <div class="flex gap-2">
                {["month", "quarter", "year"]
                    .into_iter()
                    .map(|k| {
                        view! {
                            <button
                                class="btn btn-sm"
                                class:btn-primary=move || key.get() == k
                                on:click=move |_| set_key.set(k.to_string())
                            >
                                {k}
                            </button>
                        }
                    })
                    .collect_view()}
                <button class="btn btn-sm btn-outline" on:click=move |_| res.refetch()>
                    "Refetch"
                </button>
            </div>

            <div class="rounded-box border border-base-300 p-4 min-h-24" data-testid="swr-body">
                {move || match res.state.get() {
                    SwrState::Loading => {
                        view! {
                            <span class="loading loading-spinner" data-state="loading"></span>
                        }
                            .into_any()
                    }
                    SwrState::Error(e) => {
                        view! { <div class="alert alert-error" data-state="error">{e}</div> }
                            .into_any()
                    }
                    SwrState::Ready { data, revalidating } => {
                        view! {
                            <div
                                data-state=if revalidating { "revalidating" } else { "fresh" }
                                class:opacity-60=revalidating
                            >
                                <p class="font-mono text-sm">{data}</p>
                                <p class="text-xs opacity-70 mt-1">
                                    {if revalidating {
                                        "cached \u{2014} revalidating in the background\u{2026}"
                                    } else {
                                        "fresh"
                                    }}
                                </p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn SwrDemo() -> impl IntoView {
    let mounted = RwSignal::new(true);

    view! {
        <ContentLayout
            title="Stale-While-Revalidate Cache"
            description="use_swr_resource: render the cached value for a key instantly while a background fetch revalidates it."
        >
            <Section title="Instant re-mount (the point of the primitive)">
                <p class="text-sm opacity-70 mb-4">
                    "A " <code>"LocalResource"</code>
                    " is rebuilt every time its screen re-mounts, so back-navigating re-fetches from "
                    "scratch and shows a spinner over data the user saw seconds ago. "
                    <code>"use_swr_resource"</code>
                    " keeps the last value in an app-wide cache (provided above the router, so it "
                    "outlives a route) keyed by the params it was fetched with."
                </p>
                <p class="text-sm opacity-70 mb-4">
                    "Unmount and re-mount below \u{2014} standing in for leaving and returning to a "
                    "route. The first mount spins for "
                    {SIMULATED_LATENCY_MS.to_string()} "ms; every re-mount renders the cached "
                    "payload immediately (dimmed, marked \"revalidating\") and swaps in the fresh "
                    "one when the refetch lands. Switching keys is a cache miss, so it spins again "
                    "\u{2014} then that key is cached too."
                </p>
                <label class="flex items-center gap-2 cursor-pointer mb-4 w-fit">
                    <input
                        type="checkbox"
                        class="toggle toggle-primary"
                        data-testid="swr-mount-toggle"
                        prop:checked=move || mounted.get()
                        on:change=move |_| mounted.update(|m| *m = !*m)
                    />
                    <span class="text-sm">"Screen mounted"</span>
                </label>

                <Show when=move || mounted.get() fallback=|| view! {
                    <div class="opacity-50 text-sm italic p-4">"Screen unmounted."</div>
                }>
                    <CachedScreen />
                </Show>
            </Section>
        </ContentLayout>
    }
}
