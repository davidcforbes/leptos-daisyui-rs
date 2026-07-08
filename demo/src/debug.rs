//! `window.__APP_DEBUG__` state oracle (ldui-49w.3).
//!
//! Exposes a `window.__APP_DEBUG__` object matching the contract
//! `pixelproof-web`'s `debug_bridge` consumer expects (see
//! `PixelProof/docs/wasm-csr-introspection.md` and
//! `PixelProof/crates/pixelproof-web/src/debug_bridge.rs`), so the visual
//! suite can assert on internal reactive state — not just pixels. Adapted
//! from the reference implementation in `llm-wiki`
//! (`crates/wiki-ui/frontend/src/debug.rs`), with one deliberate difference:
//! llm-wiki gates this module behind a compile-time `debug-hooks` cargo
//! feature; here it's gated at the call site by [`crate::test_mode::is_test_mode`]
//! (the same URL-query-param check `test_mode` uses), so one normal
//! `trunk build` serves both production and test-mode requests. See
//! `test_mode.rs` module docs for why a query param beats a feature here.
//!
//! Surface (all synchronous, all JSON-friendly, matching the documented
//! contract exactly):
//!   - `domHtml()`         -> full `<html>` outerHTML (string)
//!   - `state()`           -> registered signal values, JSON object as a string
//!   - `dump()`            -> `{ url, title, state, dom }` combined snapshot (JSON string)
//!   - `styles(sel, prop)` -> computed style value for first match (string | null)
//!
//! `state()` is the one that matters for this app: it always contains
//! `route` (current pathname) and `theme` (active daisyUI base theme),
//! registered once from `AppInner`, plus `state` — the generic per-page map
//! any component can write into via `debug_state::set(key, value)` (see
//! `debug_state.rs`).

use std::cell::RefCell;
use std::collections::BTreeMap;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

thread_local! {
    /// `name -> getter` producing a JSON value for that signal's current
    /// state. WASM is single-threaded, so `thread_local!` + `RefCell` is
    /// sound here.
    static REGISTRY: RefCell<BTreeMap<String, Box<dyn Fn() -> serde_json::Value>>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Register a signal getter under `name`. The getter MUST read with
/// `get_untracked()` / `with_untracked()` (or an equivalent untracked read)
/// so it does not subscribe the (non-reactive) debug caller to the signal.
///
/// Only register signals you own. For App-root-permanent state (route,
/// theme, the generic per-page map) this is fire-and-forget; for
/// per-component state that can unmount, pair with [`unregister_signal`] in
/// `on_cleanup`.
pub fn register_signal<F>(name: &str, getter: F)
where
    F: Fn() -> serde_json::Value + 'static,
{
    REGISTRY.with(|r| {
        r.borrow_mut().insert(name.to_string(), Box::new(getter));
    });
}

/// Remove a previously registered signal. Call from `on_cleanup` for signals
/// owned by components that unmount.
#[allow(dead_code)]
pub fn unregister_signal(name: &str) {
    REGISTRY.with(|r| {
        r.borrow_mut().remove(name);
    });
}

/// Snapshot every registered signal into a JSON object.
fn snapshot_state() -> serde_json::Value {
    REGISTRY.with(|r| {
        let map: serde_json::Map<String, serde_json::Value> = r
            .borrow()
            .iter()
            .map(|(k, get)| (k.clone(), get()))
            .collect();
        serde_json::Value::Object(map)
    })
}

/// Serialize the whole document (`<html>` outerHTML).
fn dump_dom() -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .map(|el| el.outer_html())
        .unwrap_or_default()
}

/// Computed value of CSS `prop` for the first element matching `selector`.
fn computed_style(selector: &str, prop: &str) -> Option<String> {
    let win = web_sys::window()?;
    let doc = win.document()?;
    let el = doc.query_selector(selector).ok().flatten()?;
    let decl = win.get_computed_style(&el).ok().flatten()?;
    decl.get_property_value(prop).ok()
}

fn current_url() -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.url().ok())
        .unwrap_or_default()
}

fn current_title() -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| d.title())
        .unwrap_or_default()
}

/// Pure assembly of the combined `{ url, title, state, dom }` value from
/// already-computed pieces. Split out from [`dump_value`] specifically so
/// the JSON *shape* is unit-testable without ever calling into web-sys
/// (whose imported statics panic if touched outside a wasm32 binary, e.g.
/// under a native `cargo test`) — see the `tests` module below.
fn build_snapshot(
    url: impl Into<String>,
    title: impl Into<String>,
    state: serde_json::Value,
    dom: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "url": url.into(),
        "title": title.into(),
        "state": state,
        "dom": dom.into(),
    })
}

/// The combined `{ url, title, state, dom }` value (not yet stringified).
/// DOM-bound (calls `current_url`/`current_title`/`dump_dom`, all web-sys);
/// not unit-testable — see [`build_snapshot`] for the pure logic this wraps.
fn dump_value() -> serde_json::Value {
    build_snapshot(current_url(), current_title(), snapshot_state(), dump_dom())
}

/// Combined snapshot, serialized to a JSON string (what `dump()` returns to
/// JS). DOM-bound; not unit-testable — see [`build_snapshot`].
fn dump_json() -> String {
    dump_value().to_string()
}

/// Attach a zero-arg JS function (returning a `JsValue`) as `ns[name]`. The
/// closure is leaked deliberately — the bridge lives for the page session.
fn set_fn0<F>(ns: &Object, name: &str, f: F)
where
    F: Fn() -> JsValue + 'static,
{
    let cb = Closure::<dyn Fn() -> JsValue>::new(f);
    let _ = Reflect::set(ns, &JsValue::from_str(name), cb.as_ref());
    cb.forget();
}

/// Build `window.__APP_DEBUG__` and install it. No-op if there is no
/// `window`. Callers must gate this behind [`crate::test_mode::is_test_mode`]
/// — installing unconditionally would put a debug surface on the production
/// window.
pub fn install_debug_bridge() {
    let Some(win) = web_sys::window() else {
        return;
    };
    let ns = Object::new();

    set_fn0(&ns, "domHtml", || JsValue::from_str(&dump_dom()));
    set_fn0(&ns, "state", || {
        JsValue::from_str(&snapshot_state().to_string())
    });
    set_fn0(&ns, "dump", || JsValue::from_str(&dump_json()));

    // styles(selector, prop) -> string | null
    let styles = Closure::<dyn Fn(String, String) -> JsValue>::new(|sel: String, prop: String| {
        match computed_style(&sel, &prop) {
            Some(v) => JsValue::from_str(&v),
            None => JsValue::NULL,
        }
    });
    let _ = Reflect::set(&ns, &JsValue::from_str("styles"), styles.as_ref());
    styles.forget();

    let _ = Reflect::set(&win, &JsValue::from_str("__APP_DEBUG__"), &ns);

    web_sys::console::log_1(&JsValue::from_str(
        "[__APP_DEBUG__] debug bridge installed (dump/state/domHtml/styles)",
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_snapshots_to_empty_object() {
        // Note: REGISTRY is a thread_local shared across tests in this
        // module; this test only asserts the *shape* holds (object, not
        // array/null), not that it's literally empty, since test order/
        // other tests in this file may have registered signals first.
        let v = snapshot_state();
        assert!(v.is_object());
    }

    #[test]
    fn register_signal_appears_in_snapshot() {
        register_signal("test.marker", || serde_json::json!(42));
        let v = snapshot_state();
        assert_eq!(v["test.marker"], serde_json::json!(42));
    }

    #[test]
    fn unregister_signal_removes_it() {
        register_signal("test.transient", || serde_json::json!("x"));
        unregister_signal("test.transient");
        let v = snapshot_state();
        assert!(v.get("test.transient").is_none());
    }

    #[test]
    fn build_snapshot_has_documented_shape() {
        let v = build_snapshot(
            "http://x/",
            "Title",
            serde_json::json!({"a": 1}),
            "<html></html>",
        );
        assert_eq!(v["url"], serde_json::json!("http://x/"));
        assert_eq!(v["title"], serde_json::json!("Title"));
        assert_eq!(v["state"], serde_json::json!({"a": 1}));
        assert_eq!(v["dom"], serde_json::json!("<html></html>"));
    }

    #[test]
    fn build_snapshot_serializes_to_valid_json() {
        let v = build_snapshot("u", "t", serde_json::json!({}), "d");
        let s = v.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("must be valid JSON");
        assert!(parsed.is_object());
        assert!(parsed.get("state").is_some());
    }
}
