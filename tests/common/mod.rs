//! Shared plumbing for the PixelProof smoke suites (ldui-49w.1).
//!
//! The suites drive the **demo app** (`demo/`, served by `trunk serve` on
//! port 3010) through headless Chrome via `pixelproof-web`. Every URL gets
//! `?pp-freeze=1` appended, which (a) kills CSS animations/transitions/caret
//! blink/smooth scroll before first paint and (b) installs the
//! `window.__APP_DEBUG__` state oracle — see `demo/src/test_mode.rs` and
//! `demo/src/debug.rs` (ldui-49w.2/.3).
//!
//! Baseline convention (ldui-49w.4): committed PNGs live at
//! `tests/visual/baselines/<page>/<state>.w<width>.png`, viewport-suffixed
//! because captures only match baselines taken at the same viewport. The
//! single smoke viewport is 1280x800 ([`VIEWPORT`]).

// Each integration-test binary compiles this module independently, so any
// helper unused by one binary would warn there.
#![allow(dead_code)]

pub mod layout_audit;

use pixelproof_web::{Harness, HarnessConfig, ViewportSize};
use std::path::PathBuf;

/// The single smoke viewport: 1280x800, pixelproof's "smallest supported
/// desktop" preset. Baseline filenames carry a `.w1280` suffix so a future
/// second viewport can coexist in the same tree.
pub const VIEWPORT: ViewportSize = ViewportSize::SMALL;

/// Default base URL of the demo dev server (`trunk serve` in `demo/`,
/// port from `demo/Trunk.toml`). Override with `VISUAL_TEST_BASE_URL`.
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:3010";

/// Viewport-suffix a state name: `"default"` -> `"default.w1280"`, so the
/// harness writes/reads `tests/visual/baselines/<page>/default.w1280.png`.
pub fn state(name: &str) -> String {
    format!("{name}.w{}", VIEWPORT.width)
}

/// Repo root (this crate's manifest dir).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Harness config: committed baselines under `tests/visual/baselines`,
/// ephemeral renders/diffs under `.review/` (gitignored), 500 ms settle for
/// Leptos CSR hydration (per PixelProof's wasm-csr-introspection guidance).
pub fn config() -> HarnessConfig {
    let root = repo_root();
    let mut cfg = HarnessConfig::default()
        .with_baseline_root(root.join("tests/visual/baselines"))
        .with_settle_ms(500);
    cfg.render_root = root.join(".review/visual-renders");
    cfg.diff_root = root.join(".review/visual-diffs");
    // Respect an explicit VISUAL_TEST_BASE_URL (already read by default());
    // otherwise point at the demo app's port instead of pixelproof's 9090.
    if std::env::var("VISUAL_TEST_BASE_URL").is_err() {
        cfg = cfg.with_base_url(DEFAULT_BASE_URL);
    }
    // Profile isolation: without an explicit --user-data-dir, headless Chrome
    // reuses a shared profile, so localStorage (e.g. the persisted
    // `leptos-daisyui-demo-theme`) leaks between tests and runs — the theme
    // test's "dark" bled into later launches. Give every harness a unique
    // throwaway profile dir under the OS temp dir.
    let unique = format!(
        "ldui-pp-profile-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let profile_dir = std::env::temp_dir().join(unique);
    cfg = cfg.with_launch_arg(format!("--user-data-dir={}", profile_dir.display()));
    cfg
}

/// Launch Chrome, set the smoke viewport, navigate to `path` with the
/// `?pp-freeze=1` determinism/oracle switch appended, and wait for the CSR
/// app to actually mount.
///
/// The fixed `settle_ms` alone is NOT enough here: the dev-profile wasm is
/// ~60 MB, so first paint of real content can lag navigation by seconds
/// (the first capture attempt produced six identical blank PNGs). We poll
/// for (a) the freeze style tag — `test_mode::install_style_kill_switch`
/// runs in `main()` before `mount_to_body`, so its presence proves the wasm
/// booted in test mode — and (b) the Layout's `<main>` element, which proves
/// the component tree mounted.
pub async fn harness_at(path: &str) -> Harness {
    let cfg = config();
    let base = cfg.base_url.clone();
    let h = Harness::launch_with_config(cfg).await.unwrap_or_else(|e| {
        panic!("failed to launch headless Chrome: {e}. Is Chrome/Chromium installed?")
    });
    h.set_viewport(VIEWPORT).await.expect("set viewport");
    h.navigate(&format!("{path}?pp-freeze=1"))
        .await
        .unwrap_or_else(|e| {
            panic!(
                "navigation to {path} failed ({e}).\n\
                 Is the demo dev server running at {base}?\n\
                 Start it with `cargo make test-visual` (orchestrated) or\n\
                 `trunk serve` from demo/ (manual)."
            )
        });
    wait_for_selector(&h, r#"style[data-pixelproof="freeze"]"#).await;
    wait_for_selector(&h, "main").await;
    // One settle beat after mount so fonts/layout are final.
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
    h
}

/// Poll (100 ms interval, 60 s budget) until `document.querySelector(sel)`
/// matches. Panics on timeout with the selector in the message. The budget is
/// generous because the dev-profile wasm is ~60 MB and Chrome instances can
/// contend when tests overlap.
pub async fn wait_for_selector(h: &Harness, sel: &str) {
    let expr = format!(
        "document.querySelector({}) !== null",
        serde_json::to_string(sel).unwrap()
    );
    for _ in 0..600 {
        let found: bool = h
            .page()
            .evaluate(expr.as_str())
            .await
            .ok()
            .and_then(|v| v.into_value().ok())
            .unwrap_or(false);
        if found {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("timed out (30s) waiting for selector {sel:?} — did the wasm app mount?");
}

/// Click `selector` with a real CDP mouse event and wait the settle delay.
/// (The harness only exposes click_and_capture; reactivity tests need a
/// capture-free click.)
pub async fn click(h: &Harness, selector: &str) {
    h.page()
        .find_element(selector)
        .await
        .unwrap_or_else(|e| panic!("find {selector}: {e}"))
        .click()
        .await
        .unwrap_or_else(|e| panic!("click {selector}: {e}"));
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Pull the `window.__APP_DEBUG__.state()` snapshot, panicking if the bridge
/// is absent (it must exist on any page loaded with `?pp-freeze=1`).
pub async fn oracle(h: &Harness) -> serde_json::Value {
    h.app_debug_state()
        .await
        .expect("app_debug_state call failed")
        .expect("window.__APP_DEBUG__ missing — was the page loaded with ?pp-freeze=1?")
}

/// Run the [`layout_audit`] sweep against the currently-loaded page and
/// deserialize the report.
///
/// The sweep returns a JSON *string* rather than an object: CDP's value
/// marshalling flattens nested arrays inconsistently across driver versions,
/// and a string round-trips identically everywhere.
pub async fn layout_report(h: &Harness) -> layout_audit::AuditReport {
    let raw: String = h
        .page()
        .evaluate_function(layout_audit::SWEEP_JS)
        .await
        .expect("layout sweep failed to evaluate")
        .into_value()
        .expect("layout sweep did not return a string");
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("layout sweep returned unparseable JSON ({e}): {raw}"))
}
