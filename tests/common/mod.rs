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

/// Viewport-suffix a state name at an explicit width, for captures taken at
/// a non-default viewport (e.g. the tablet-width chart baseline).
pub fn state_at(name: &str, width: u32) -> String {
    format!("{name}.w{width}")
}

/// Viewport-suffix a state name: `"default"` -> `"default.w1280"`, so the
/// harness writes/reads `tests/visual/baselines/<page>/default.w1280.png`.
pub fn state(name: &str) -> String {
    state_at(name, VIEWPORT.width)
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
    assert_stylesheet_is_current(&h, path).await;
    h.set_viewport(VIEWPORT).await.expect("set viewport");
    tokio::time::sleep(std::time::Duration::from_millis(settle_ms)).await;
    h
}

/// Prepare a component-region screenshot target inside a nested page scroller,
/// then prove it is fully contained before asking PixelProof to clip it.
///
/// PixelProof reads viewport-relative bounding rectangles, while Chrome
/// consumes the screenshot clip in page coordinates. Scrolling between those
/// operations displaces the rendered component by scrollY and clips its
/// bottom. This helper may scroll a nested content container, but restores the
/// browser window to the page origin before measuring. A partial/off-screen
/// target then fails instead of producing a shifted or blank baseline.
pub fn region_fits_viewport(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    viewport: ViewportSize,
) -> bool {
    x.is_finite()
        && y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width > 0.0
        && height > 0.0
        && x >= 0.0
        && y >= 0.0
        && x + width <= f64::from(viewport.width)
        && y + height <= f64::from(viewport.height)
}

pub async fn prepare_region_capture(h: &Harness, selector: &str, viewport: ViewportSize) {
    let selector_json = serde_json::to_string(selector).expect("serialize region selector");
    let script = format!(
        r#"(() => {{
          const element = document.querySelector({selector_json});
          if (!element) return false;
          element.scrollIntoView({{ block: 'center', inline: 'nearest' }});
          window.scrollTo(0, 0);
          return true;
        }})()"#
    );
    let found: bool = h
        .page()
        .evaluate(script)
        .await
        .expect("prepare visual region")
        .into_value()
        .expect("region preparation result is boolean");
    assert!(found, "visual region '{selector}' does not exist");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let window_scroll: [f64; 2] = h
        .page()
        .evaluate("(() => [window.scrollX, window.scrollY])()")
        .await
        .expect("read window scroll position")
        .into_value()
        .expect("window scroll position is numeric");
    assert!(
        window_scroll[0].abs() <= 0.5 && window_scroll[1].abs() <= 0.5,
        "visual region '{selector}' left the browser window scrolled at ({}, {})",
        window_scroll[0],
        window_scroll[1],
    );

    let bounds = h
        .element_box(selector)
        .await
        .unwrap_or_else(|e| panic!("read visual region '{selector}' bounds: {e}"));
    assert!(
        region_fits_viewport(bounds.x, bounds.y, bounds.width, bounds.height, viewport,),
        "visual region '{selector}' does not fit inside the {}x{} viewport after nested scrolling: \
         x={}, y={}, width={}, height={}",
        viewport.width,
        viewport.height,
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
    );
}

// ---------------------------------------------------------------------------
// Stylesheet freshness guard (ldui-hun)
// ---------------------------------------------------------------------------

/// Where `demo/build-css.mjs` records the marker it stamped into the
/// stylesheet it just built. Contents: `ok <token>` or `fail <token>`.
fn css_stamp_file() -> PathBuf {
    repo_root().join("demo/.ldui-css-stamp")
}

/// Collect every `#ldui-css-stamp-*` selector present in the stylesheets the
/// page actually loaded, straight off the CSSOM.
const COLLECT_STAMPS_JS: &str = r#"(() => {
  const found = [];
  for (const sheet of Array.from(document.styleSheets)) {
    let rules;
    try { rules = sheet.cssRules; } catch (e) { continue; }
    for (const rule of Array.from(rules || [])) {
      const sel = rule.selectorText;
      if (typeof sel === 'string' && sel.startsWith('#ldui-css-stamp-')) {
        found.push(sel.slice(1));
      }
    }
  }
  return found.join(',');
})()"#;

/// Refuse to audit a stylesheet we cannot prove is the one just built.
///
/// **The defect this exists for (ldui-hun):** trunk swallows its `pre_build`
/// hook's failure, so a `CssSyntaxError` in `demo/input.css` leaves it serving
/// the *previous* `output.css` — and both audit suites passed green against
/// that stale stylesheet. Every ceiling and every ratchet in the visual-quality
/// system was measuring whatever CSS last succeeded rather than the CSS just
/// written.
///
/// Checking tailwind's exit code alone is not enough, because `dist/` also goes
/// stale for reasons tailwind never sees — a concurrent session's `trunk build`
/// rewriting `demo/dist` underneath a running suite did exactly that, and
/// surfaced as nine layout failures on pages it never touched. So the check is
/// the general one: `demo/build-css.mjs` stamps a **run-unique** marker into
/// `output.css` on every successful build, and this asserts the marker in the
/// CSS the browser loaded is the current one.
///
/// Called from [`harness_at`], the single funnel every browser suite goes
/// through (`style_audit_smoke`, `layout_audit_smoke`, `visual_smoke`,
/// `reactivity_smoke`), and called per navigation rather than once per process
/// so a rebuild *during* a suite is caught too.
///
/// The marker is an unmatchable id rule (`#ldui-css-stamp-<token>`), so it
/// cannot perturb a computed style — nothing in the demo carries that id.
pub async fn assert_stylesheet_is_current(h: &Harness, path: &str) {
    let stamp_path = css_stamp_file();
    let recorded = std::fs::read_to_string(&stamp_path).unwrap_or_else(|e| {
        panic!(
            "STYLESHEET BUILD GUARD (ldui-hun): {} is missing ({e}).\n\
             The demo stylesheet has not been built by `demo/build-css.mjs`, so there is \
             nothing proving the CSS being audited is current.\n\
             Run `node build-css.mjs` in demo/ (or let `cargo xtask test-style` / \
             `test-layout` start the server, which does it for you).",
            stamp_path.display()
        )
    });
    let recorded = recorded.trim();
    let (status, token) = recorded.split_once(' ').unwrap_or(("fail", recorded));

    assert!(
        status == "ok",
        "STYLESHEET BUILD FAILED (ldui-hun): {} records `{recorded}`.\n\
         Tailwind did not produce a stylesheet on the last build, so `demo/output.css` \
         is STALE and trunk is serving the CSS that last succeeded — not the CSS you \
         just wrote. Auditing it would be meaningless.\n\
         Fix `demo/input.css` and re-run; `node build-css.mjs` in demo/ reproduces the \
         failure on its own.",
        stamp_path.display()
    );

    let found: String = h
        .page()
        .evaluate(COLLECT_STAMPS_JS)
        .await
        .expect("evaluate stylesheet stamp collector")
        .into_value()
        .expect("stamp collector returns a string");
    let expected = format!("ldui-css-stamp-{token}");

    assert!(
        found.split(',').any(|s| s == expected),
        "STALE STYLESHEET (ldui-hun): the CSS loaded by {path} is not the CSS that was \
         last built.\n  expected marker: {expected}   (from {})\n  markers found in the \
         served stylesheets: {}\n\
         `demo/output.css` or `demo/dist` is stale — commonly a failed tailwind build \
         (trunk swallows its pre_build hook's exit code and keeps serving the previous \
         stylesheet) or a concurrent `trunk build` rewriting `demo/dist` underneath this \
         run. Every ceiling and ratchet below would have been measured against the wrong \
         CSS, so the suite stops here rather than reporting a green it cannot justify.",
        stamp_path.display(),
        if found.is_empty() {
            "(none)".to_string()
        } else {
            found.clone()
        },
    );
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

/// Shift-click `selector` with real CDP mouse events and wait the settle delay.
pub async fn shift_click(h: &Harness, selector: &str) {
    let b = h
        .element_box(selector)
        .await
        .unwrap_or_else(|e| panic!("element_box {selector}: {e}"));
    let x = b.x + b.width / 2.0;
    let y = b.y + b.height / 2.0;
    let moved = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(x)
        .y(y)
        .modifiers(8)
        .build()
        .expect("shift mouse move params");
    dispatch_mouse(h, moved, "Shift+MouseMoved").await;
    let pressed = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MousePressed)
        .x(x)
        .y(y)
        .button(MouseButton::Left)
        .click_count(1)
        .modifiers(8)
        .build()
        .expect("shift mouse press params");
    dispatch_mouse(h, pressed, "Shift+MousePressed").await;
    let released = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseReleased)
        .x(x)
        .y(y)
        .button(MouseButton::Left)
        .click_count(1)
        .modifiers(8)
        .build()
        .expect("shift mouse release params");
    dispatch_mouse(h, released, "Shift+MouseReleased").await;
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Pull the `window.__APP_DEBUG__.state()` snapshot, panicking if the bridge
/// is absent (it must exist on any page loaded with `?pp-freeze=1`).
pub async fn oracle(h: &Harness) -> serde_json::Value {
    ldui_audit::oracle(h)
        .await
        .expect("app_debug_state call failed")
        .expect("window.__APP_DEBUG__ missing — was the page loaded with ?pp-freeze=1?")
}

// ── Line-chart interaction seams (ldui-9tr.5) ────────────────────────────────
//
// Real CDP input at fractional positions inside an element's box, plus a
// buffered browser-error observer (D1: no browser panic or hidden error).

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams, DispatchMouseEventType,
    MouseButton,
};

async fn dispatch_key(h: &Harness, params: DispatchKeyEventParams, what: &str) {
    h.page()
        .execute(params)
        .await
        .unwrap_or_else(|e| panic!("dispatch {what}: {e}"));
}

/// Press Shift+Enter with real CDP key events and wait the settle delay.
pub async fn shift_enter(h: &Harness) {
    let shift_down = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::RawKeyDown)
        .key("Shift")
        .code("ShiftLeft")
        .windows_virtual_key_code(16)
        .native_virtual_key_code(16)
        .modifiers(8)
        .build()
        .expect("Shift key-down params");
    dispatch_key(h, shift_down, "Shift key-down").await;

    let enter_down = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::RawKeyDown)
        .key("Enter")
        .code("Enter")
        .windows_virtual_key_code(13)
        .native_virtual_key_code(13)
        .modifiers(8)
        .build()
        .expect("Shift+Enter key-down params");
    dispatch_key(h, enter_down, "Shift+Enter key-down").await;

    let enter_up = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyUp)
        .key("Enter")
        .code("Enter")
        .windows_virtual_key_code(13)
        .native_virtual_key_code(13)
        .modifiers(8)
        .build()
        .expect("Shift+Enter key-up params");
    dispatch_key(h, enter_up, "Shift+Enter key-up").await;

    let shift_up = DispatchKeyEventParams::builder()
        .r#type(DispatchKeyEventType::KeyUp)
        .key("Shift")
        .code("ShiftLeft")
        .windows_virtual_key_code(16)
        .native_virtual_key_code(16)
        .build()
        .expect("Shift key-up params");
    dispatch_key(h, shift_up, "Shift key-up").await;

    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

async fn fraction_point(
    h: &Harness,
    selector: &str,
    x_fraction: f64,
    y_fraction: f64,
) -> (f64, f64) {
    let b = h
        .element_box(selector)
        .await
        .unwrap_or_else(|e| panic!("element_box {selector}: {e}"));
    (b.x + b.width * x_fraction, b.y + b.height * y_fraction)
}

async fn dispatch_mouse(h: &Harness, params: DispatchMouseEventParams, what: &str) {
    h.page()
        .execute(params)
        .await
        .unwrap_or_else(|e| panic!("dispatch {what}: {e}"));
}

/// Make this page's `(hover)`/`(pointer)` media features report a real
/// desktop mouse, undoing a `chromiumoxide` 0.7.0 bug (ldui-jdzr).
///
/// Root cause, traced with a raw-CDP probe outside chromiumoxide entirely
/// (a bare `chrome.exe --headless=new` reports `hover: hover` correctly
/// from first paint): every page `pixelproof-web::Harness` creates goes
/// through chromiumoxide's `Browser::new_page`, whose
/// `EmulationManager::init_commands` (`chromiumoxide-0.7.0/src/handler/
/// emulation.rs`) unconditionally sends
/// `Emulation.setTouchEmulationEnabled(true)` -- hardcoded `true`, ignoring
/// the configured `Viewport::has_touch` (which defaults to `false` and is
/// never overridden here). That's a live touch-emulation override on every
/// harness page, independent of `Harness::set_viewport`'s later
/// `Emulation.setDeviceMetricsOverride{mobile: false}` (which doesn't touch
/// it), and Chromium reports `hover: none` / `pointer: coarse` while touch
/// emulation is enabled -- Tailwind wraps every `hover:` utility in
/// `@media (hover: hover)` specifically to avoid sticky-hover on touch
/// devices, so that flag silently drops every `hover:`-utility style in a
/// harness-driven test even though the element is genuinely `:hover`ed
/// (`Element.matches(':hover')` is `true`; a real CDP `MouseMoved` dispatch
/// always lands the element in `:hover`, media features notwithstanding).
/// `Emulation.setEmulatedMedia`'s `features` list does not cover `hover`/
/// `pointer` in this Chrome build (verified: setting them there has no
/// effect) -- the fix has to undo the actual cause, touch emulation, not
/// paper over its symptom. Call once per page, after `harness_at`, before
/// asserting on any `hover:`-gated computed style.
pub async fn force_desktop_hover_media(h: &Harness) {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    h.page()
        .execute(SetTouchEmulationEnabledParams::new(false))
        .await
        .unwrap_or_else(|e| panic!("Emulation.setTouchEmulationEnabled(false): {e}"));
}

/// Move the real CDP pointer to `(x_fraction, y_fraction)` of `selector`'s
/// bounding box (0.0 = left/top edge, 1.0 = right/bottom), then settle.
pub async fn move_pointer_to_svg_fraction(
    h: &Harness,
    selector: &str,
    x_fraction: f64,
    y_fraction: f64,
) {
    let (x, y) = fraction_point(h, selector, x_fraction, y_fraction).await;
    let moved = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseMoved)
        .x(x)
        .y(y)
        .build()
        .expect("mouse move params");
    dispatch_mouse(h, moved, "MouseMoved").await;
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Click at `(x_fraction, y_fraction)` of `selector`'s bounding box with real
/// CDP `MousePressed`/`MouseReleased` events (a move first, so hover state
/// matches what a person's click produces), then settle.
pub async fn click_svg_fraction(h: &Harness, selector: &str, x_fraction: f64, y_fraction: f64) {
    move_pointer_to_svg_fraction(h, selector, x_fraction, y_fraction).await;
    let (x, y) = fraction_point(h, selector, x_fraction, y_fraction).await;
    let pressed = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MousePressed)
        .x(x)
        .y(y)
        .button(MouseButton::Left)
        .click_count(1)
        .build()
        .expect("mouse press params");
    dispatch_mouse(h, pressed, "MousePressed").await;
    let released = DispatchMouseEventParams::builder()
        .r#type(DispatchMouseEventType::MouseReleased)
        .x(x)
        .y(y)
        .button(MouseButton::Left)
        .click_count(1)
        .build()
        .expect("mouse release params");
    dispatch_mouse(h, released, "MouseReleased").await;
    tokio::time::sleep(std::time::Duration::from_millis(h.config().settle_ms)).await;
}

/// Install (or reset) one buffered `console.error` / `window.error` /
/// `unhandledrejection` observer. Call once before a journey; read the
/// buffer back with [`assert_no_browser_errors`] after it.
pub async fn begin_browser_error_capture(h: &Harness) {
    let installed: bool = h
        .page()
        .evaluate(
            r#"(() => {
                if (window.__lduiErrs) { window.__lduiErrs.length = 0; return true; }
                window.__lduiErrs = [];
                const push = (kind, message) => {
                    window.__lduiErrs.push(kind + ': ' + String(message));
                };
                const original = console.error.bind(console);
                console.error = (...args) => {
                    push('console.error', args.map(String).join(' '));
                    original(...args);
                };
                window.addEventListener('error', (e) => push('window.error', e.message));
                window.addEventListener('unhandledrejection', (e) => push('unhandledrejection', e.reason));
                return true;
            })()"#,
        )
        .await
        .expect("install browser error capture")
        .into_value()
        .expect("error capture installer returns a bool");
    assert!(installed, "browser error capture failed to install");
}

/// Panic (naming `context`) if the error buffer holds anything.
pub async fn assert_no_browser_errors(h: &Harness, context: &str) {
    let raw: String = h
        .page()
        .evaluate("JSON.stringify(window.__lduiErrs || [])")
        .await
        .expect("read browser error buffer")
        .into_value()
        .expect("error buffer serializes");
    let errors: Vec<String> = serde_json::from_str(&raw).expect("error buffer is a string array");
    assert!(
        errors.is_empty(),
        "browser errors after {context}:\n  {}",
        errors.join("\n  ")
    );
}
