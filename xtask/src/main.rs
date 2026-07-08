//! leptos-daisyui-rs local CI/CD logic. See `doc/ci-cd.md`.
//! Run via the `cargo xtask <sub>` alias (`.cargo/config.toml`).

use std::process::ExitCode;

fn main() -> ExitCode {
    let sub = std::env::args().nth(1).unwrap_or_default();
    match sub.as_str() {
        "verify" | "verify-full" | "fmt-check" | "clippy" | "build" | "check-demo" | "test"
        | "bump" => {
            eprintln!("xtask: '{sub}' not implemented yet");
            ExitCode::from(1)
        }
        other => {
            eprintln!("xtask: unknown subcommand {other:?}");
            eprintln!(
                "usage: cargo xtask <verify|verify-full|fmt-check|clippy|build|check-demo|test|bump>"
            );
            ExitCode::from(2)
        }
    }
}
