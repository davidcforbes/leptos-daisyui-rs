//! Focused browser proof for EntityTable's page-size select identity
//! (ldui-kl55): the framework-owned rows-per-page `<select>` had no `id`/
//! `name` at all when the caller omitted `page_size_control_id`, and Office
//! satellites mounting several `EntityTable`s on one Setup page had no way
//! to tell the controls apart. `demo/src/demos/snapshot_table_page.rs`'s
//! `EntityTablePageSizeIdentityFixture` mounts two tables without an
//! override plus one with an explicit override.

mod common;

use common::{
    assert_no_browser_errors, begin_browser_error_capture, harness_at, wait_for_selector,
};
use serde_json::{Value, json};

async fn eval_json(harness: &pixelproof_web::Harness, expression: &str) -> Value {
    harness
        .page()
        .evaluate(expression)
        .await
        .unwrap_or_else(|error| panic!("evaluate `{expression}`: {error}"))
        .into_value()
        .unwrap_or_else(|error| panic!("JSON value for `{expression}`: {error}"))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-client-snapshot)"]
async fn page_size_select_gets_unique_identity_without_an_override_and_honors_one() {
    let harness = harness_at("/components/entity-table-page-size-identity").await;
    wait_for_selector(
        &harness,
        "#entity-table-page-size-identity-fixture [data-entity-table-grid] tbody tr",
    )
    .await;
    begin_browser_error_capture(&harness).await;

    let identities = eval_json(
        &harness,
        r#"(() => {
            const describe = (testid) => {
                const root = document.querySelector(`[data-testid="${testid}"]`);
                const select = root.querySelector('label select');
                const label = root.querySelector('label');
                return {
                    id: select.id,
                    name: select.name,
                    labelWrapsSelect: label != null && label.contains(select),
                    associatedLabelCount: select.labels ? select.labels.length : 0,
                };
            };
            return {
                a: describe('page-size-identity-table-a'),
                b: describe('page-size-identity-table-b'),
                c: describe('page-size-identity-table-c'),
            };
        })()"#,
    )
    .await;

    let a = &identities["a"];
    let b = &identities["b"];
    let c = &identities["c"];

    // Acceptance: without `page_size_control_id`, the select renders a
    // non-empty id AND name.
    assert!(
        a["id"].as_str().is_some_and(|id| !id.is_empty()),
        "table A's page-size select must render a non-empty id: {identities}"
    );
    assert!(
        a["name"].as_str().is_some_and(|name| !name.is_empty()),
        "table A's page-size select must render a non-empty name: {identities}"
    );
    assert_eq!(
        a["id"], a["name"],
        "the generated default must drive both id and name: {identities}"
    );

    // Acceptance: two or more tables mounted together receive unique values.
    assert_ne!(
        a["id"], b["id"],
        "two EntityTables without an override must not share a page-size select id: {identities}"
    );
    assert_ne!(
        a["name"], b["name"],
        "two EntityTables without an override must not share a page-size select name: {identities}"
    );

    // Acceptance: caller-supplied identity remains stable and honored.
    assert_eq!(
        c["id"],
        json!("page-size-identity-explicit-override"),
        "a caller-supplied page_size_control_id must be honored verbatim: {identities}"
    );
    assert_eq!(
        c["name"],
        json!("page-size-identity-explicit-override"),
        "a caller-supplied page_size_control_id must also drive name: {identities}"
    );

    // Acceptance: labels remain correctly associated (the select is nested
    // inside the visible `<label>`, so the implicit association holds
    // regardless of which id path — generated or override — is in play).
    for (name, table) in [("a", a), ("b", b), ("c", c)] {
        assert_eq!(
            table["labelWrapsSelect"],
            json!(true),
            "table {name}'s page-size select must remain inside its visible label: {identities}"
        );
        assert!(
            table["associatedLabelCount"]
                .as_u64()
                .is_some_and(|count| count >= 1),
            "table {name}'s page-size select must have at least one associated label: {identities}"
        );
    }

    let axe = pixelproof_web::a11y::Axe::from_path("tests/vendor/axe-core/axe.min.js")
        .expect("load vendored axe-core");
    let report = axe.run(harness.page()).await.expect("run axe-core");
    report
        .assert_no_blocking("entity-table-page-size-identity")
        .unwrap_or_else(|error| {
            panic!(
                "{error}; {}\nviolations: {:#?}",
                report.summary(),
                report.violations
            )
        });

    assert_no_browser_errors(&harness, "EntityTable page-size select identity").await;
}
