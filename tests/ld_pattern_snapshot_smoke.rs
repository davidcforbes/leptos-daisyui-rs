//! Real-DOM proof for ldui-wra5 on the client-snapshot test host, where
//! `SnapshotTablePage` lives: its root declares
//! `data-ld-pattern="SnapshotTable"`, and the page's dedicated-row
//! `PageHeader` (the layout the general demo never renders) declares
//! `PageHeader`. The general demo's patterns are `ld_pattern_smoke.rs`;
//! `cargo xtask test-ld-pattern` runs both files.
mod common;
mod ld_pattern;

use ld_pattern::{Host, check_host};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the client-snapshot test host (cargo xtask test-ld-pattern)"]
async fn snapshot_host_pattern_roots_declare_data_ld_pattern() {
    check_host(Host::ClientSnapshot).await;
}
