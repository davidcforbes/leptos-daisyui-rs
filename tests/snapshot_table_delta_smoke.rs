//! Real-browser proof for generation-bound displayed-snapshot deltas
//! (ldui-vn81 / ldui-cb29). Compile-only pending a gate run on this
//! machine (the demo trunk build is currently broken here); the native
//! reducer coverage in `src/patterns/snapshot_table/tests.rs` is the
//! primary evidence for this feature.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, click, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .expect("evaluate snapshot-table delta fixture")
        .into_value()
        .expect("snapshot-table delta expression returns JSON")
}

async fn delta_snapshot(harness: &pixelproof_web::Harness) -> Value {
    eval_json(
        harness,
        r#"(() => {
            const root = document.getElementById('snapshot-page');
            const table = document.getElementById('snapshot-page-table');
            return {
                generation: root?.dataset.snapshotGeneration,
                phase: root?.dataset.snapshotPhase,
                rows: table?.querySelectorAll('[data-entity-table-grid] tbody tr').length ?? 0,
                disposition: document.querySelector(
                    '[data-testid="delta-last-disposition"]'
                )?.textContent?.trim() ?? null,
            };
        })()"#,
    )
    .await
}

/// Own-claim row removal: a delta atomically replaces rows without bumping
/// the dataset/access generation `EntityTable`'s `focus_scope`/
/// `dataset_identity` are bound to.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn own_claim_removal_delta_replaces_rows_without_bumping_generation() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let initial = delta_snapshot(&harness).await;
    assert_eq!(initial["rows"], json!(3));
    assert_eq!(initial["phase"], json!("Displaying"));

    click(&harness, "[data-testid='delta-own-claim']").await;
    let after = delta_snapshot(&harness).await;
    assert_eq!(after["disposition"], json!("applied"));
    assert_eq!(after["rows"], json!(2));
    assert_eq!(after["phase"], json!("Displaying"));
    assert_eq!(
        after["generation"], initial["generation"],
        "a delta must never bump the dataset/access generation"
    );

    assert_no_browser_errors(&harness, "own-claim delta").await;
}

/// Two independent deltas -- the caller's own claim, then another user's
/// SSE-delivered removal -- both apply in sequence against the freshly
/// updated displayed snapshot.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn sequential_own_claim_and_sse_removal_deltas_both_apply() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    click(&harness, "[data-testid='delta-own-claim']").await;
    let after_first = delta_snapshot(&harness).await;
    assert_eq!(after_first["disposition"], json!("applied"));
    assert_eq!(after_first["rows"], json!(2));

    click(&harness, "[data-testid='delta-sse-removal']").await;
    let after_second = delta_snapshot(&harness).await;
    assert_eq!(after_second["disposition"], json!("applied"));
    assert_eq!(after_second["rows"], json!(1));
    assert_eq!(after_second["generation"], after_first["generation"]);

    assert_no_browser_errors(&harness, "sequential own-claim and SSE-removal deltas").await;
}

/// A stale/duplicate replay of an already-consumed delta handle is rejected
/// and never regresses the rows a later delta already removed.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn duplicate_stale_delta_replay_is_rejected() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    // The first click both mints and applies a handle, and stashes a clone
    // of that now-consumed handle for the replay button.
    click(&harness, "[data-testid='delta-own-claim']").await;
    click(&harness, "[data-testid='delta-sse-removal']").await;
    let before_replay = delta_snapshot(&harness).await;
    assert_eq!(before_replay["rows"], json!(1));

    click(&harness, "[data-testid='delta-replay-stale']").await;
    let after_replay = delta_snapshot(&harness).await;
    assert_eq!(after_replay["disposition"], json!("ignored-stale"));
    assert_eq!(
        after_replay["rows"], before_replay["rows"],
        "a stale replay must not resurrect rows a later delta already removed"
    );
    assert_eq!(after_replay["generation"], before_replay["generation"]);

    assert_no_browser_errors(&harness, "duplicate/stale delta replay").await;
}

/// A delta may be minted and applied while an unrelated office replacement
/// remains in flight; the replacement is left completely intact and still
/// completes normally afterward, superseding the delta's rows.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn delta_during_replacement_leaves_it_intact_and_it_still_completes() {
    let harness = harness_at("/components/snapshot-table-page").await;
    wait_for_selector(
        &harness,
        "#snapshot-page-table [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let initial = delta_snapshot(&harness).await;

    click(&harness, "[data-testid='snapshot-start-replacement']").await;
    let replacing = delta_snapshot(&harness).await;
    assert_eq!(replacing["phase"], json!("Replacing"));
    assert_eq!(replacing["generation"], initial["generation"]);

    click(&harness, "[data-testid='delta-own-claim']").await;
    let mid_replacement = delta_snapshot(&harness).await;
    assert_eq!(mid_replacement["disposition"], json!("applied"));
    assert_eq!(
        mid_replacement["phase"],
        json!("Replacing"),
        "the delta must not disturb the in-flight replacement's phase"
    );
    assert_eq!(mid_replacement["rows"], json!(2));
    assert_eq!(mid_replacement["generation"], initial["generation"]);

    click(&harness, "[data-testid='snapshot-complete-replacement']").await;
    let completed = delta_snapshot(&harness).await;
    assert_eq!(completed["phase"], json!("Displaying"));
    assert_ne!(
        completed["generation"], initial["generation"],
        "completing the still-pending replacement must bump the generation"
    );
    assert_eq!(
        completed["rows"],
        json!(3),
        "the replacement dataset's own rows are shown"
    );

    assert_no_browser_errors(&harness, "delta during an unrelated office replacement").await;
}
