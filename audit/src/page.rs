//! Whole-page audit: the engine's generic style/layout sweep composed with
//! the daisyUI component-drift sweep (`crate::drift`), merged into one
//! [`AuditReport`]. Also the two demo-test-support helpers (`click`/`oracle`)
//! consumer suites drive real CDP input and read internal state through,
//! ported here (Result-ified) from `tests/common/mod.rs` so both this crate
//! and the consuming test binaries share one implementation.

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
    for v in drift.violations {
        report.push(family::COMPONENT_DRIFT, v);
    }
    Ok(report)
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
