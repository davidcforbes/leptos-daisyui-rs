//! Shared plumbing for the PixelProof smoke suites (ldui-49w.1).
//!
//! The suites drive the **demo app** (`demo/`, served by `trunk serve` on
//! port 3010) through headless Chrome via `pixelproof-web`. Every URL gets
//! `?pp-freeze=1` appended, which (a) kills CSS animations/transitions/caret
//! blink/smooth scroll before first paint and (b) installs the
//! `window.__APP_DEBUG__` state oracle — see `src/test_mode.rs` (library,
//! `test-mode` feature) and `demo/src/debug.rs` (ldui-49w.2/.3).
//!
//! Baseline convention (ldui-49w.4): committed PNGs live at
//! `tests/visual/baselines/<page>/<state>.w<width>.png`, viewport-suffixed
//! because captures only match baselines taken at the same viewport. The
//! single smoke viewport is 1280x800 ([`VIEWPORT`]).
//!
//! The browser plumbing itself (readiness polling, the freeze/oracle query
//! suffix, `click`/`oracle`) now lives in `pixelproof-style-audit::web` +
//! `ldui-audit` (Task 7, ldui-audit's `audit/src/page.rs`); this module
//! composes those with the SSIM-specific bits (baseline/render/diff roots)
//! the visual suite still needs.

// Each integration-test binary compiles this module independently, so any
// helper unused by one binary would warn there.
#![allow(dead_code)]

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
    // test's "dark" bled into later launches.
    //
    // This used to hand-roll the directory and pass `--user-data-dir`
    // directly, which isolated the profile but never removed it: ~1.5 GB of
    // `%TEMP%` in a single working session (ldui-r93). The harness now owns
    // the directory, deletes it on `close()`, and reaps any left behind by a
    // killed test process on the next launch.
    cfg.with_isolated_profile()
}

/// Launch Chrome, navigate to `path` with the `?pp-freeze=1`
/// determinism/oracle switch appended, wait for the CSR app to actually
/// mount, then set the smoke viewport.
///
/// Composes `ldui_audit::ldui_web_config` (the freeze/oracle query suffix,
/// the freeze style tag as readiness proof, the demo's base URL) with this
/// module's SSIM roots grafted onto its `.harness`, so `visual_smoke.rs`'s
/// `capture_and_compare` still finds the committed baselines. The engine's
/// `web::harness_at` owns the readiness polling and appends `query_suffix`
/// itself — no local `?pp-freeze=1` append or extra waits needed here.
///
/// The engine harness always launches at `ViewportSize::CANONICAL`
/// (1440x900, hard-coded in `pixelproof_web::Harness::launch_with_config`)
/// and has no hook to set a viewport before its own navigate/wait sequence,
/// so the smoke viewport (1280x800 — what the committed baselines and
/// ceilings assume) is set *after* it returns, mirroring the order
/// `pixelproof_web::responsive::capture_at_viewports` uses for its own
/// per-viewport passes: `set_viewport` (a CDP device-metrics override,
/// which reflows the already-loaded page) then one settle beat so
/// fonts/layout are final at the new size before anything reads the DOM.
pub async fn harness_at(path: &str) -> Harness {
    let mut cfg = ldui_audit::ldui_web_config(DEFAULT_BASE_URL);
    let ssim = config();
    cfg.harness.baseline_root = ssim.baseline_root;
    cfg.harness.render_root = ssim.render_root;
    cfg.harness.diff_root = ssim.diff_root;
    let base = cfg.harness.base_url.clone();
    let settle_ms = cfg.harness.settle_ms;

    let h = ldui_audit::web::harness_at(&cfg, path)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "navigation to {path} failed ({e}).\n\
                 Is the demo dev server running at {base}?\n\
                 Start it with `cargo make test-visual` (orchestrated) or\n\
                 `trunk serve` from demo/ (manual)."
            )
        });
    h.set_viewport(VIEWPORT).await.expect("set viewport");
    tokio::time::sleep(std::time::Duration::from_millis(settle_ms)).await;
    h
}

/// The demo's real computed `<body>` font family (first family in the stack,
/// quotes stripped).
///
/// Both audit suites pin this into their `StyleProfile` rather than naming a
/// family: a silent font fallback (the declared family never loads) is itself
/// a regression the sweep should catch, and a *wrong* declared family is
/// worse than useless — the engine compares every text-bearing element's
/// computed family against it, so a sentinel like `"__any__"` makes every
/// element on a text-heavy page a typography violation, overruns the engine's
/// 200-per-family cap and latches `truncated` on every run forever (see
/// `assert_not_truncated`).
pub async fn body_font_family(h: &Harness) -> String {
    let raw: String = h
        .page()
        .evaluate("getComputedStyle(document.body).fontFamily")
        .await
        .expect("evaluate body font-family")
        .into_value()
        .expect("font-family computed style is a string");
    raw.split(',')
        .next()
        .unwrap_or(&raw)
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

/// Fail when a sweep hit the engine's per-family cap.
///
/// The engine caps each family at 200 violations and sets
/// `AuditReport::truncated`, documenting the counts as "a floor, not a total".
/// A ratcheted ceiling above the cap is then permanently uncheckable: the
/// count saturates at 200, stays `<= ceiling` forever, and every further
/// regression passes while the report reads clean.
///
/// Both audit suites call this — `ldui_audit::verify` only *notes* truncation
/// in its `Ok` path, so surfacing it needs an explicit assertion either way,
/// and the layout suite cannot adopt `verify` at all (it governs only
/// overlap/grid/internal, while `verify`'s ratchet demands a ceiling for
/// every reporting family).
pub fn assert_not_truncated(report: &ldui_audit::AuditReport, surface: &str) {
    assert!(
        !report.truncated,
        "sweep of {surface} hit the engine's per-family cap — counts are a floor, \
         not a total, so any ceiling at or above the cap can never fail again.\n{}",
        report.describe(surface)
    );
}

/// Poll until `document.querySelector(sel)` matches (60 s budget). Panics on
/// timeout with the selector in the message.
pub async fn wait_for_selector(h: &Harness, sel: &str) {
    ldui_audit::web::wait_for_selector(h, sel, 60_000)
        .await
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Click `selector` with a real CDP mouse event and wait the settle delay.
/// (The harness only exposes click_and_capture; reactivity tests need a
/// capture-free click.)
pub async fn click(h: &Harness, selector: &str) {
    ldui_audit::click(h, selector, h.config().settle_ms)
        .await
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Pull the `window.__APP_DEBUG__.state()` snapshot, panicking if the bridge
/// is absent (it must exist on any page loaded with `?pp-freeze=1`).
pub async fn oracle(h: &Harness) -> serde_json::Value {
    ldui_audit::oracle(h)
        .await
        .expect("app_debug_state call failed")
        .expect("window.__APP_DEBUG__ missing — was the page loaded with ?pp-freeze=1?")
}
