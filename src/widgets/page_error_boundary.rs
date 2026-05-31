//! Reusable scoped error boundary for page sections (P-5.5 / EUC-lehe).
//!
//! Wrap a region of a page that consumes a `LocalResource<Result<T, ApiError>>`
//! in [`PageErrorBoundary`] so a failed query renders a single localised
//! Alert instead of unmounting the section (or, worse, propagating up to the
//! top-level boundary and reloading the whole shell).
//!
//! Errors propagate to a Leptos `<ErrorBoundary>` only when a child view
//! returns `Result::Err(E)` where `E: std::error::Error + Send + Sync +
//! 'static`. Our `ApiError` already implements those bounds (see
//! the app API), so the typical pattern is:
//!
//! ```ignore
//! <PageErrorBoundary>
//!   {move || resource.get().map(|r| r.map(|data| view! { ... })) }
//! </PageErrorBoundary>
//! ```
//!
//! The fallback uses the daisyUI Alert component (DaisyUI library wrapper) so styling
//! stays consistent with the rest of the app.

use leptos::prelude::*;

use crate::components::{Alert, AlertColor};

/// Scoped error boundary intended for individual page sections.
///
/// Renders the child view normally; if any descendant view returns
/// `Result::Err(_)` the boundary swaps to an `Alert` listing the propagated
/// error messages. Sibling sections continue to render — the rest of the
/// page is unaffected.
#[component]
pub fn PageErrorBoundary(children: Children) -> impl IntoView {
    view! {
        <ErrorBoundary
            fallback=|errors| view! {
                <Alert color=AlertColor::Error class="my-4">
                    <span>
                        "This section failed to load: "
                        {move || errors.with(|errs| {
                            errs.iter()
                                .map(|(_, e)| e.to_string())
                                .collect::<Vec<_>>()
                                .join("; ")
                        })}
                    </span>
                </Alert>
            }
        >
            {children()}
        </ErrorBoundary>
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
//
// Runtime tests for this component require `wasm-bindgen-test` plus a
// browser-driver harness (e.g. `wasm-pack test --headless --chrome`). That
// infrastructure is NOT yet wired up in `frontend/Cargo.toml`'s
// dev-dependencies, so we ship a single compile-only sanity test for now.
// Once the harness lands (tracked as a follow-up), add a test that mounts a
// `PageErrorBoundary` around a child returning `Err(ApiError::…)` and asserts
// the fallback Alert appears in the DOM.

#[cfg(test)]
mod tests {
    /// Trivial sanity test — verifies the module compiles when test-mode is
    /// active. Real rendering tests are deferred until the wasm browser
    /// harness is in place.
    #[test]
    fn module_compiles() {
        // No-op: the fact that this file type-checks under `cargo test` is
        // the assertion. If `PageErrorBoundary` accidentally referenced a
        // non-existent component or trait, this build would fail.
    }
}
