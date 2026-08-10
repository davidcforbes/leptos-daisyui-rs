//! Whole-page audit: the engine's generic style/layout sweep composed with
//! the daisyUI component-drift sweep (`crate::drift`), merged into one
//! [`AuditReport`]. Also the two demo-test-support helpers (`click`/`oracle`)
//! consumer suites drive real CDP input and read internal state through,
//! ported here (Result-ified) from `tests/common/mod.rs` so both this crate
//! and the consuming test binaries share one implementation.

use crate::drift::DriftReport;
use pixelproof_style_audit::{AuditReport, StyleProfile, SweepOptions, family};
use pixelproof_web::Harness;

/// Run the full ldui audit on the currently-loaded page: the engine's
/// generated style/layout sweep plus the daisyUI component-drift sweep,
/// merged into one report.
pub async fn audit_page(
    h: &Harness,
    profile: &StyleProfile,
    opts: &SweepOptions,
) -> Result<AuditReport, String> {
    let mut report = pixelproof_style_audit::web::run_sweep(h, profile, opts)
        .await
        .map_err(|e| e.to_string())?;
    let drift = crate::drift::run_drift(h, &opts.mount_selector).await?;
    merge_drift(&mut report, drift, &opts.mount_selector)?;
    Ok(report)
}

/// Fold a [`DriftReport`] into the engine's report: violations into the
/// `component-drift` family, truncation OR-ed into the report's own flag.
///
/// Split out from [`audit_page`] so the merge is unit-testable without a
/// browser — the two ways it can silently lie (a dropped truncation flag, a
/// drift sweep that matched nothing) are exactly what has no browser-free
/// coverage otherwise.
///
/// A drift sweep that scanned nothing while the engine's own sweep scanned
/// elements is an error, not a clean page: both sweeps resolve the same
/// `mount_selector`, so a disagreement means `component-drift` reported zero
/// because it never looked, and `check_ceilings` would read that as a pass.
fn merge_drift(
    report: &mut AuditReport,
    drift: DriftReport,
    mount_selector: &str,
) -> Result<(), String> {
    if drift.scanned == 0 && report.scanned > 0 {
        return Err(format!(
            "component-drift sweep scanned 0 elements under `{mount_selector}` while the engine \
             sweep scanned {} — the two sweeps disagree about the mount subtree, so the drift \
             rules reported nothing rather than nothing being wrong",
            report.scanned
        ));
    }
    // Counts are a floor once either sweep hits its cap, so the merged report
    // must say so — a family saturated at the cap otherwise sits under any
    // ceiling above it forever, passing every future regression.
    report.truncated |= drift.truncated;
    for v in drift.violations {
        report.push(family::COMPONENT_DRIFT, v);
    }
    Ok(())
}

/// Click `selector` with a real CDP mouse event and wait `settle_ms`.
pub async fn click(h: &Harness, selector: &str, settle_ms: u64) -> Result<(), String> {
    h.page()
        .find_element(selector)
        .await
        .map_err(|e| format!("find {selector}: {e}"))?
        .click()
        .await
        .map_err(|e| format!("click {selector}: {e}"))?;
    tokio::time::sleep(std::time::Duration::from_millis(settle_ms)).await;
    Ok(())
}

/// Pull the `window.__APP_DEBUG__.state()` snapshot (`None` if the bridge is
/// absent — e.g. the page was loaded without the freeze/oracle query switch).
pub async fn oracle(h: &Harness) -> Result<Option<serde_json::Value>, String> {
    h.app_debug_state().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drift(json: &str) -> DriftReport {
        serde_json::from_str(json).expect("drift report fixture")
    }

    fn swept(scanned: usize) -> AuditReport {
        AuditReport {
            scanned,
            ..Default::default()
        }
    }

    #[test]
    fn merge_appends_drift_violations_to_the_component_drift_family() {
        let mut report = swept(120);
        merge_drift(
            &mut report,
            drift(
                r#"{"violations":[{"selector":"button#save","value":1.0,"detail":"button-without-btn: raw button lacks .btn"}],"scanned":120,"truncated":false}"#,
            ),
            "main",
        )
        .expect("merge");
        assert_eq!(report.count(family::COMPONENT_DRIFT), 1);
        assert!(!report.truncated);
    }

    #[test]
    fn a_truncated_drift_sweep_truncates_the_merged_report() {
        let mut report = swept(120);
        merge_drift(
            &mut report,
            drift(r#"{"violations":[],"scanned":9000,"truncated":true}"#),
            "main",
        )
        .expect("merge");
        assert!(
            report.truncated,
            "drift truncation must reach the merged report — otherwise a family \
             saturated at the cap passes every ceiling above it forever"
        );
    }

    #[test]
    fn an_already_truncated_report_stays_truncated() {
        let mut report = AuditReport {
            scanned: 120,
            truncated: true,
            ..Default::default()
        };
        merge_drift(
            &mut report,
            drift(r#"{"violations":[],"scanned":120,"truncated":false}"#),
            "main",
        )
        .expect("merge");
        assert!(report.truncated);
    }

    #[test]
    fn a_drift_sweep_that_matched_nothing_is_an_error() {
        let mut report = swept(120);
        let err = merge_drift(
            &mut report,
            drift(r#"{"violations":[],"scanned":0,"truncated":false}"#),
            "main",
        )
        .expect_err("a drift sweep that scanned nothing must not read as a clean page");
        assert!(err.contains("main"), "the message names the mount selector");
        assert!(err.contains("120"), "the message names the engine's count");
    }

    /// Both sweeps scanning nothing is the engine's own `sanity()` failure,
    /// not a disagreement — the surface never rendered. Leave that report to
    /// `sanity`/`verify` rather than reporting a mismatch that does not exist.
    #[test]
    fn both_sweeps_scanning_nothing_is_left_to_sanity() {
        let mut report = swept(0);
        merge_drift(
            &mut report,
            drift(r#"{"violations":[],"scanned":0,"truncated":false}"#),
            "main",
        )
        .expect("no mismatch when neither sweep saw anything");
        assert!(report.sanity().is_err());
    }
}
