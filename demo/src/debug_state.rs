//! Generic per-page state map for the `window.__APP_DEBUG__` oracle
//! (ldui-49w.3).
//!
//! `debug.rs` handles *registered signal getters* (computed on demand,
//! app-root-permanent — `route`, `theme`). This module is the complementary
//! *write* side for demo pages: a component with some ad-hoc bit of state a
//! test might want to assert on (a modal's open/closed flag, a table's
//! current sort column) calls [`set`] imperatively wherever it already
//! mutates that state, no signal wiring required.
//!
//! `debug.rs` registers a single `"state"` getter (see `register_signal`
//! calls in `main.rs`) that reads [`get_all`], so everything written here
//! surfaces under the `state.state` key of `window.__APP_DEBUG__.state()` /
//! `.dump()`.
//!
//! [`set`] is a no-op unless [`crate::test_mode::is_test_mode`] is true, so
//! demo components can call it unconditionally without worrying about
//! polluting production behavior or paying for JSON serialization when the
//! harness isn't attached.

use std::cell::RefCell;
use std::collections::BTreeMap;

use serde::Serialize;

thread_local! {
    static STATE: RefCell<BTreeMap<String, serde_json::Value>> = const { RefCell::new(BTreeMap::new()) };
}

/// Record `key -> value` in the generic per-page debug-state map. No-op if
/// test mode is not active (see module docs) or if `value` fails to
/// serialize.
///
/// Not yet called anywhere in this batch (ldui-49w.2/.3 only lay the seam) —
/// intentionally unused-but-public: the *next* batch's demo pages/components
/// call this from their own state changes (modal open/close, table sort,
/// etc.). See the contract notes in the report for the exact call shape.
#[allow(dead_code)]
pub fn set(key: &str, value: impl Serialize) {
    if !crate::test_mode::is_test_mode() {
        return;
    }
    let Ok(json) = serde_json::to_value(value) else {
        return;
    };
    STATE.with(|s| {
        s.borrow_mut().insert(key.to_string(), json);
    });
}

/// Remove `key` from the map (e.g. from `on_cleanup` when a component
/// unmounts and its state no longer applies).
#[allow(dead_code)]
pub fn remove(key: &str) {
    STATE.with(|s| {
        s.borrow_mut().remove(key);
    });
}

/// Snapshot the whole map as a JSON object. Used by the `"state"` signal
/// registered with `debug::register_signal` in `main.rs`.
pub fn get_all() -> serde_json::Value {
    STATE.with(|s| {
        let map: serde_json::Map<String, serde_json::Value> = s
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::Value::Object(map)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_is_noop_outside_test_mode() {
        // No `?pp-freeze=1` in this native test process's (nonexistent)
        // window, so `is_test_mode()` is false and `set` must not store
        // anything. Assert indirectly via `get_all` staying empty for a
        // key unique to this test.
        set("debug_state_tests.noop_marker", 123);
        let all = get_all();
        assert!(all.get("debug_state_tests.noop_marker").is_none());
    }

    #[test]
    fn get_all_returns_a_json_object() {
        assert!(get_all().is_object());
    }
}
