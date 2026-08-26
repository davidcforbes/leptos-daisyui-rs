//! Checked manifests for focused, opinionated-pattern verification.

/// Which half of a pattern gate to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PatternLane {
    Inner,
    Browser,
}

impl PatternLane {
    pub(crate) fn parse_flag(flag: &str) -> Result<Self, String> {
        match flag {
            "--inner" => Ok(Self::Inner),
            "--browser" => Ok(Self::Browser),
            _ => Err(format!("unknown pattern lane {flag:?}")),
        }
    }
}

/// One statically declared command in a focused pattern gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PatternCheck {
    Cargo {
        name: &'static str,
        args: &'static [&'static str],
    },
    Browser {
        name: &'static str,
        test: &'static str,
        html_target: &'static str,
    },
}

pub(crate) const BROWSER_FINGERPRINT_COMMAND_VERSION: &str = "client-snapshot-browser-command-v2";

/// Inputs recorded beside a reusable page-scoped browser build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BrowserFingerprintParts<'a> {
    pub(crate) command_version: &'a str,
    pub(crate) source_hash: &'a str,
    pub(crate) cargo_lock_hash: &'a str,
    pub(crate) generated_token_hash: &'a str,
    pub(crate) rust_version: &'a str,
    pub(crate) trunk_version: &'a str,
}

impl BrowserFingerprintParts<'_> {
    pub(crate) fn manifest(&self) -> String {
        let inputs = format!(
            "command-version={}\nsource-hash={}\ncargo-lock-hash={}\n\
             generated-token-hash={}\nrust-version={}\ntrunk-version={}\n",
            self.command_version,
            self.source_hash,
            self.cargo_lock_hash,
            self.generated_token_hash,
            self.rust_version.trim(),
            self.trunk_version.trim(),
        );
        format!("{inputs}fingerprint={}\n", hash_bytes(inputs.as_bytes()))
    }
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

const CLIENT_SNAPSHOT_INNER: &[PatternCheck] = &[
    PatternCheck::Cargo {
        name: "test-client-snapshot-contracts",
        args: &["test", "-p", "leptos-daisyui-rs", "--lib", "patterns"],
    },
    PatternCheck::Cargo {
        name: "test-client-snapshot-entity-table",
        args: &[
            "test",
            "-p",
            "leptos-daisyui-rs",
            "--lib",
            "components::entity_table",
        ],
    },
];

const CLIENT_SNAPSHOT_BROWSER: &[PatternCheck] = &[PatternCheck::Browser {
    name: "test-client-snapshot",
    test: "entity_table_smoke",
    html_target: "client-snapshot-test-host.html",
}];

pub(crate) fn checks_for(
    pattern: &str,
    lane: PatternLane,
) -> Result<&'static [PatternCheck], String> {
    match (pattern, lane) {
        ("client-snapshot-list", PatternLane::Inner) => Ok(CLIENT_SNAPSHOT_INNER),
        ("client-snapshot-list", PatternLane::Browser) => Ok(CLIENT_SNAPSHOT_BROWSER),
        _ => Err(format!("unknown UI pattern {pattern:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_snapshot_inner_selects_only_contract_model_and_pattern_tests() {
        let checks = checks_for("client-snapshot-list", PatternLane::Inner).expect("known pattern");
        let debug = format!("{checks:?}");
        assert_eq!(checks.len(), 2);
        assert!(debug.contains("patterns"));
        assert!(debug.contains("components::entity_table"));
        assert!(!debug.contains("chart"));
        assert!(!debug.contains("editor"));
    }

    #[test]
    fn client_snapshot_browser_selects_only_page_scoped_journey() {
        assert_eq!(
            checks_for("client-snapshot-list", PatternLane::Browser).expect("known pattern"),
            &[PatternCheck::Browser {
                name: "test-client-snapshot",
                test: "entity_table_smoke",
                html_target: "client-snapshot-test-host.html",
            }]
        );
    }

    #[test]
    fn unknown_pattern_and_lane_fail_closed() {
        assert!(checks_for("unknown", PatternLane::Inner).is_err());
        assert!(PatternLane::parse_flag("--everything").is_err());
    }

    #[test]
    fn browser_fingerprint_names_every_required_invalidation_input() {
        let parts = BrowserFingerprintParts {
            command_version: "v1",
            source_hash: "source-a",
            cargo_lock_hash: "lock-a",
            generated_token_hash: "tokens-a",
            rust_version: "rustc-a",
            trunk_version: "trunk-a",
        };
        let manifest = parts.manifest();
        for expected in [
            "command-version=v1",
            "source-hash=source-a",
            "cargo-lock-hash=lock-a",
            "generated-token-hash=tokens-a",
            "rust-version=rustc-a",
            "trunk-version=trunk-a",
            "fingerprint=",
        ] {
            assert!(
                manifest.contains(expected),
                "missing {expected:?}: {manifest}"
            );
        }
    }

    #[test]
    fn browser_fingerprint_changes_when_a_source_hash_changes() {
        let first = BrowserFingerprintParts {
            command_version: "v1",
            source_hash: "source-a",
            cargo_lock_hash: "lock-a",
            generated_token_hash: "tokens-a",
            rust_version: "rustc-a",
            trunk_version: "trunk-a",
        }
        .manifest();
        let second = BrowserFingerprintParts {
            source_hash: "source-b",
            ..BrowserFingerprintParts {
                command_version: "v1",
                source_hash: "source-a",
                cargo_lock_hash: "lock-a",
                generated_token_hash: "tokens-a",
                rust_version: "rustc-a",
                trunk_version: "trunk-a",
            }
        }
        .manifest();
        assert_ne!(first, second);
    }
}
