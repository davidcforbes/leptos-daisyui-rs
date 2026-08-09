//! Determinism seam for visual-regression testing (ldui-49w.2).
//!
//! The desktop side (`pixelproof_desktop::Harness`) has native `/api/freeze` +
//! `/api/time` endpoints to make animated UI deterministic for SSIM baselines.
//! There is no such native surface here, so the web analogue is a URL query
//! param checked once at app mount: **`?pp-freeze=1`**.
//!
//! A query param was chosen over a compile-time cargo feature (the pattern
//! `wasm-csr-introspection.md` and `llm-wiki` use for the debug bridge)
//! because the PixelProof web harness drives an ordinary `trunk build` /
//! `trunk serve` of this demo app — there is no special "test" binary for a
//! feature flag to gate. A query param lets one normal build serve both
//! production and test-mode requests; the harness just navigates to a URL
//! with `?pp-freeze=1` appended.
//!
//! When active, [`install_style_kill_switch`] injects a global stylesheet
//! that disables animations/transitions/caret-blink/smooth-scroll, so
//! toasts, drawers, loading spinners, and focus carets render as a single
//! deterministic frame instead of flapping mid-transition.

/// The query param that activates test/freeze mode.
///
/// `#[allow(dead_code)]`: only reachable from [`is_test_mode`]'s `wasm32`
/// branch and from the unit tests below; a native, non-test clippy/build
/// pass (host target, no `--tests`) sees neither caller and flags it dead.
#[allow(dead_code)]
pub const QUERY_PARAM: &str = "pp-freeze";

/// Pure parse: does `search` (a `?a=b&c=d`-style query string, with or
/// without the leading `?`) request test mode?
///
/// Recognizes `pp-freeze=1`, `pp-freeze=true`, and a bare `pp-freeze` (no
/// value) as truthy. A missing param, `pp-freeze=0`, or `pp-freeze=false` is
/// not. This is the native-testable half of the seam; reading the real
/// `window.location.search` is DOM-bound (see [`is_test_mode`]).
///
/// Same `#[allow(dead_code)]` rationale as [`QUERY_PARAM`] above.
#[allow(dead_code)]
pub fn is_enabled(search: &str) -> bool {
    let search = search.trim_start_matches('?');
    search.split('&').any(|pair| {
        let mut it = pair.splitn(2, '=');
        let key = it.next().unwrap_or("");
        if key != QUERY_PARAM {
            return false;
        }
        match it.next() {
            None => true,
            Some(v) => matches!(v, "1" | "true"),
        }
    })
}

/// Is test mode active for the current page? Reads `window.location.search`.
/// DOM-bound; not unit-testable — see [`is_enabled`] for the pure logic this
/// wraps.
///
/// Off `wasm32` (e.g. `cargo test` on the host target — web-sys's imported
/// statics panic if touched outside a wasm32 binary) this always returns
/// `false`, which is also the semantically correct answer: there is no
/// browser `window` in a native test process.
#[cfg(target_arch = "wasm32")]
pub fn is_test_mode() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| is_enabled(&s))
        .unwrap_or(false)
}

/// See the `wasm32` version above; this is the native-test-safe fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn is_test_mode() -> bool {
    false
}

/// CSS injected as a `<style>` element when test mode is active. Kills
/// animations, transitions, the blinking text caret, and smooth scrolling so
/// every screenshot is a deterministic single frame.
pub const FREEZE_CSS: &str = "\
*, *::before, *::after {\n\
  animation: none !important;\n\
  animation-delay: 0s !important;\n\
  animation-duration: 0s !important;\n\
  transition: none !important;\n\
  transition-duration: 0s !important;\n\
  caret-color: transparent !important;\n\
}\n\
html, body {\n\
  scroll-behavior: auto !important;\n\
}\n";

/// Inject [`FREEZE_CSS`] as a `<style data-pixelproof=\"freeze\">` element in
/// `<head>`. No-op if there is no `window`/`document`/`head` to attach to.
///
/// DOM-bound by nature (creates and appends a real element) — not
/// unit-testable outside a browser/wasm-bindgen-test environment. Call once,
/// at app startup, only when [`is_test_mode`] is true.
pub fn install_style_kill_switch() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(head) = document.head() else {
        return;
    };
    let Ok(style) = document.create_element("style") else {
        return;
    };
    style.set_text_content(Some(FREEZE_CSS));
    let _ = style.set_attribute("data-pixelproof", "freeze");
    let _ = head.append_child(&style);
}

/// One-call setup: returns whether test mode is active, installing the
/// freeze stylesheet when it is. Apps still install their own debug bridge
/// (app state is app-specific) inside the returned-true branch.
pub fn install_test_mode() -> bool {
    let active = is_test_mode();
    if active {
        install_style_kill_switch();
    }
    active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_param_is_disabled() {
        assert!(!is_enabled(""));
        assert!(!is_enabled("?foo=1"));
    }

    #[test]
    fn explicit_one_enables() {
        assert!(is_enabled("?pp-freeze=1"));
        assert!(is_enabled("pp-freeze=1")); // no leading '?'
    }

    #[test]
    fn explicit_true_enables() {
        assert!(is_enabled("?pp-freeze=true"));
    }

    #[test]
    fn bare_param_enables() {
        assert!(is_enabled("?pp-freeze"));
    }

    #[test]
    fn zero_or_false_disables() {
        assert!(!is_enabled("?pp-freeze=0"));
        assert!(!is_enabled("?pp-freeze=false"));
    }

    #[test]
    fn combined_with_other_params() {
        assert!(is_enabled("?theme=dark&pp-freeze=1&x=2"));
        assert!(is_enabled("?pp-freeze=1&theme=dark"));
        assert!(!is_enabled("?theme=dark&x=2"));
    }

    #[test]
    fn similar_but_wrong_key_disabled() {
        assert!(!is_enabled("?pp-freezex=1"));
        assert!(!is_enabled("?xpp-freeze=1"));
    }
}
