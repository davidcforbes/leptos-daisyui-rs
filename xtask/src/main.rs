//! leptos-daisyui-rs local CI/CD logic. See `doc/ci-cd.md`.
//! Run via the `cargo xtask <sub>` alias (`.cargo/config.toml`).
//!
//! The gate is advisory-first: every step runs even after one fails, a
//! PASS/FAIL summary is printed, and the process exit code is the number of
//! failed steps (0 = all green).

use std::process::{Command, ExitCode};

/// A single gate step: a subprocess to run, named for the summary.
struct Step {
    name: &'static str,
    program: &'static str,
    args: Vec<String>,
    cwd: Option<&'static str>,
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// The gate steps, in order, with the exact commands verified empirically
/// against this workspace:
/// - `fmt` is **per-package** — `cargo fmt --all` reaches into sibling repos.
/// - `clippy` is **per-crate** — `cargo clippy --workspace` fails on
///   leptos-csr feature unification (the demo's `csr` enables on the lib
///   when they are co-built).
fn gate_steps() -> Vec<Step> {
    vec![
        Step {
            name: "fmt-check",
            program: "cargo",
            args: args(&[
                "fmt",
                "-p",
                "leptos-daisyui-rs",
                "-p",
                "leptos-daisyui-showcase",
                "-p",
                "xtask",
                "--",
                "--check",
            ]),
            cwd: None,
        },
        Step {
            name: "clippy-lib",
            program: "cargo",
            args: args(&[
                "clippy",
                "-p",
                "leptos-daisyui-rs",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]),
            cwd: None,
        },
        Step {
            name: "clippy-demo",
            program: "cargo",
            args: args(&[
                "clippy",
                "-p",
                "leptos-daisyui-showcase",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]),
            cwd: None,
        },
        Step {
            name: "build",
            program: "cargo",
            args: args(&["build", "-p", "leptos-daisyui-rs"]),
            cwd: None,
        },
        Step {
            name: "check-demo",
            program: "cargo",
            args: args(&["check", "-p", "leptos-daisyui-showcase"]),
            cwd: None,
        },
        Step {
            name: "test-lib",
            program: "cargo",
            args: args(&["test", "-p", "leptos-daisyui-rs", "--lib"]),
            cwd: None,
        },
        Step {
            name: "test-xtask",
            program: "cargo",
            args: args(&["test", "-p", "xtask"]),
            cwd: None,
        },
    ]
}

/// The subset of gate steps an individual subcommand runs
/// (`cargo xtask clippy` runs both clippy-lib and clippy-demo, etc.).
fn steps_for(sub: &str) -> Vec<Step> {
    gate_steps()
        .into_iter()
        .filter(|s| match sub {
            "clippy" => s.name.starts_with("clippy"),
            "test" => s.name.starts_with("test"),
            "fmt-check" => s.name == "fmt-check",
            "build" => s.name == "build",
            "check-demo" => s.name == "check-demo",
            _ => false,
        })
        .collect()
}

fn run_step(step: &Step) -> bool {
    eprintln!("\n----- {} -----", step.name);
    let mut cmd = Command::new(step.program);
    cmd.args(&step.args);
    if let Some(dir) = step.cwd {
        cmd.current_dir(dir);
    }
    match cmd.status() {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("xtask: failed to launch {}: {e}", step.program);
            false
        }
    }
}

/// Pure: render the PASS/FAIL summary and compute the exit code
/// (= number of failed steps, saturating at 255).
fn summarize(results: &[(&str, bool)]) -> (String, u8) {
    let mut out = String::from("\n===== xtask summary =====\n");
    let mut failures: u32 = 0;
    for (name, ok) in results {
        out.push_str(if *ok { "  PASS " } else { "  FAIL " });
        out.push_str(name);
        out.push('\n');
        if !*ok {
            failures += 1;
        }
    }
    let passed = results.len() as u32 - failures;
    out.push_str(&format!("{}/{} passed\n", passed, results.len()));
    (out, failures.min(255) as u8)
}

/// Run a set of steps advisory-first: every step runs, then print the summary
/// and return the failure count as the exit code.
fn run_steps(steps: &[Step]) -> ExitCode {
    if steps.is_empty() {
        eprintln!("xtask: no steps to run");
        return ExitCode::from(2);
    }
    let results: Vec<(&str, bool)> = steps.iter().map(|s| (s.name, run_step(s))).collect();
    let (summary, code) = summarize(&results);
    println!("{summary}");
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let sub = std::env::args().nth(1).unwrap_or_default();
    match sub.as_str() {
        "verify" => run_steps(&gate_steps()),
        "fmt-check" | "clippy" | "build" | "check-demo" | "test" => run_steps(&steps_for(&sub)),
        "verify-full" => {
            // verify + the real wasm build (needs npm/trunk/tailwind installed).
            let mut steps = gate_steps();
            steps.push(Step {
                name: "trunk-build",
                program: "trunk",
                args: args(&["build", "--release"]),
                cwd: Some("demo"),
            });
            run_steps(&steps)
        }
        "bump" => {
            eprintln!("xtask: 'bump' not implemented yet");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_all_pass_is_zero() {
        let (text, code) = summarize(&[("fmt-check", true), ("test", true)]);
        assert_eq!(code, 0);
        assert!(text.contains("PASS fmt-check"));
        assert!(text.contains("PASS test"));
        assert!(text.contains("2/2 passed"));
    }

    #[test]
    fn summarize_counts_failures_as_exit_code() {
        let (text, code) = summarize(&[("fmt-check", false), ("clippy", true), ("test", false)]);
        assert_eq!(code, 2);
        assert!(text.contains("FAIL fmt-check"));
        assert!(text.contains("FAIL test"));
        assert!(text.contains("1/3 passed"));
    }

    #[test]
    fn clippy_subcommand_runs_both_crate_steps() {
        let names: Vec<&str> = steps_for("clippy").iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["clippy-lib", "clippy-demo"]);
    }

    #[test]
    fn test_subcommand_runs_lib_and_xtask() {
        let names: Vec<&str> = steps_for("test").iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["test-lib", "test-xtask"]);
    }
}
