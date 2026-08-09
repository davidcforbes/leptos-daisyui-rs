//! daisyUI component-drift heuristics: a second, small in-page sweep separate
//! from the engine's generic style sweep. The engine (`pixelproof-style-audit`)
//! knows no framework, only values; these four rules are daisyUI knowledge —
//! "a raw `<button>` without `.btn`", "a raw `<table>` without `.table`" —
//! so they live here rather than in the engine.
//!
//! Follows the engine's `sweep_js` pattern (see
//! `pixelproof_style_audit::sweep_js` / `sweep.rs`): a self-contained JS IIFE
//! with the mount selector embedded as a JSON constant, walking the visible
//! elements under the mount subtree and returning a JSON-stringified
//! `{ violations, scanned, truncated }` object — including the engine's hard
//! per-family cap, so a pathological page cannot build a multi-megabyte JSON
//! string and hang the CDP round-trip. [`crate::page::audit_page`] merges this
//! report's violations into the engine's
//! [`pixelproof_style_audit::AuditReport`].

use pixelproof_web::Harness;
use serde::Serialize;

/// Raw result of the drift sweep on one page.
#[derive(Debug, serde::Deserialize)]
pub struct DriftReport {
    pub violations: Vec<pixelproof_style_audit::Violation>,
    /// Elements considered — 0 means the mount selector matched nothing.
    pub scanned: usize,
    /// Whether the sweep's per-family cap was hit, so `violations` is a floor
    /// rather than a total. Mirrors `AuditReport::truncated`; `serde(default)`
    /// so an older sweep body still deserializes.
    #[serde(default)]
    pub truncated: bool,
}

/// Surface-shape options for the drift sweep — currently just where to look.
/// Kept as its own JSON-embedded struct (rather than a bare string) to mirror
/// the engine's `OPTS` pattern and leave room for future exemption knobs.
#[derive(Debug, Clone, Serialize)]
struct DriftOpts {
    mount_selector: String,
}

/// Generate the drift sweep IIFE for one mount selector (embedded as a JSON
/// constant, mirroring the engine's `sweep_js` pattern).
pub fn drift_js(mount_selector: &str) -> String {
    let opts = DriftOpts {
        mount_selector: mount_selector.to_string(),
    };
    let opts_json = serde_json::to_string(&opts).expect("drift options serialize");
    format!("(() => {{\nconst OPTS = {opts_json};\n{DRIFT_BODY}\n}})()")
}

/// The static drift-sweep body, authored in `drift.js` for readability (a
/// plain `.js` file, not a Rust string literal) and pulled in at compile
/// time. Reads only `OPTS`. Its braces are safe because it enters the
/// `format!` wrapper as an argument, not as format text.
const DRIFT_BODY: &str = include_str!("drift.js");

/// Evaluate the drift sweep on the currently-loaded page.
///
/// Mirrors the engine's `web::run_sweep` mechanics: evaluate the generated
/// JS, expect a JSON string back (CDP value marshalling flattens nested
/// arrays inconsistently across driver versions — a string round-trips
/// identically everywhere), `serde_json::from_str`, map errors to a
/// descriptive `String` (this layer does not own the engine's error enum).
pub async fn run_drift(h: &Harness, mount_selector: &str) -> Result<DriftReport, String> {
    let js = drift_js(mount_selector);
    let raw: String = h
        .page()
        .evaluate(js)
        .await
        .map_err(|e| format!("drift sweep did not evaluate: {e}"))?
        .into_value()
        .map_err(|e| format!("drift sweep did not evaluate to a string: {e}"))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("drift sweep returned unparseable JSON ({error}): {raw}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_js_embeds_the_mount_selector_as_json() {
        let js = drift_js("main");
        assert!(js.contains("\"main\""));
        // The four rule ids must appear in the generated source, so a rule
        // cannot be silently dropped.
        for rule in [
            "button-without-btn",
            "table-without-table-class",
            "badge-lookalike",
            "input-outside-field",
        ] {
            assert!(js.contains(rule), "rule {rule} missing from generated JS");
        }
    }

    #[test]
    fn drift_report_deserializes_the_sweep_shape() {
        let r: DriftReport = serde_json::from_str(
            r#"{"violations":[{"selector":"button#save","value":1.0,"detail":"button-without-btn: raw button lacks .btn"}],"scanned":42,"truncated":false}"#,
        )
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.scanned, 42);
        assert!(!r.truncated);
    }

    #[test]
    fn drift_report_carries_truncation() {
        let r: DriftReport =
            serde_json::from_str(r#"{"violations":[],"scanned":9000,"truncated":true}"#).unwrap();
        assert!(
            r.truncated,
            "a capped sweep must report that its counts are a floor"
        );
        // And an older sweep body that predates the flag still parses.
        let r: DriftReport = serde_json::from_str(r#"{"violations":[],"scanned":9000}"#).unwrap();
        assert!(!r.truncated);
    }

    /// The cap is the whole point of mirroring the engine here: without it a
    /// form-heavy consumer screen builds a multi-megabyte JSON string and the
    /// CDP round-trip appears to hang.
    #[test]
    fn generated_drift_sweep_caps_its_violation_list() {
        let js = drift_js("main");
        assert!(
            js.contains("MAX_PER_CATEGORY = 200"),
            "drift sweep must carry the engine's per-family cap"
        );
        assert!(
            js.contains("truncated = true"),
            "hitting the cap must latch the truncation flag"
        );
        assert!(
            js.contains("scanned: els.length, truncated"),
            "the sweep must return the truncation flag, not just record it"
        );
    }

    /// A percentage radius resolves against the box, exactly as the engine's
    /// shape rule does — `parseFloat("50%")` would read as 50px and mis-flag
    /// a small round chip as a badge lookalike.
    #[test]
    fn badge_lookalike_resolves_percentage_radius_against_the_box() {
        let js = drift_js("main");
        assert!(
            js.contains("rawRadius.endsWith('%')"),
            "percentage radius must be detected"
        );
        assert!(
            js.contains("(parseFloat(rawRadius) / 100) * Math.min(r.width, r.height)"),
            "percentage radius must resolve against the shorter box side"
        );
    }

    #[test]
    fn generated_drift_sweep_is_a_single_invoked_iife() {
        let js = drift_js("main");
        let js = js.trim();
        assert!(js.starts_with("(() => {"), "must be an IIFE");
        assert!(js.ends_with("})()"), "must be invoked");
    }

    #[test]
    fn custom_mount_selector_is_respected() {
        let js = drift_js("#app");
        assert!(js.contains(r##""mount_selector":"#app""##));
    }
}
