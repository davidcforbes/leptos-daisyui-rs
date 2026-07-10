//! leptos-daisyui-rs local CI/CD logic. See `doc/ci-cd.md`.
//! Run via the `cargo xtask <sub>` alias (`.cargo/config.toml`).
//!
//! The gate is advisory-first: every step runs even after one fails, a
//! PASS/FAIL summary is printed, and the process exit code is the number of
//! failed steps (0 = all green).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

/// What a gate step actually does.
enum Run {
    /// Shell out to a subprocess.
    Cmd {
        program: &'static str,
        args: Vec<String>,
        cwd: Option<&'static str>,
    },
    /// Spawn the demo dev server on a free port, run the reactivity/DOM-oracle
    /// suite against it, then tear the server down. See [`run_reactivity_suite`].
    Reactivity,
}

/// A single gate step, named for the summary.
struct Step {
    name: &'static str,
    run: Run,
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// Build a subprocess step.
fn cmd(
    name: &'static str,
    program: &'static str,
    parts: &[&str],
    cwd: Option<&'static str>,
) -> Step {
    Step {
        name,
        run: Run::Cmd {
            program,
            args: args(parts),
            cwd,
        },
    }
}

/// The gate steps, in order, with the exact commands verified empirically
/// against this workspace:
/// - `fmt` is **per-package** — `cargo fmt --all` reaches into sibling repos.
/// - `clippy` is **per-crate** — `cargo clippy --workspace` fails on
///   leptos-csr feature unification (the demo's `csr` enables on the lib
///   when they are co-built).
fn gate_steps() -> Vec<Step> {
    vec![
        cmd(
            "fmt-check",
            "cargo",
            &[
                "fmt",
                "-p",
                "leptos-daisyui-rs",
                "-p",
                "leptos-daisyui-showcase",
                "-p",
                "xtask",
                "--",
                "--check",
            ],
            None,
        ),
        cmd(
            "clippy-lib",
            "cargo",
            &[
                "clippy",
                "-p",
                "leptos-daisyui-rs",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            None,
        ),
        cmd(
            "clippy-demo",
            "cargo",
            &[
                "clippy",
                "-p",
                "leptos-daisyui-showcase",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            None,
        ),
        cmd(
            "build",
            "cargo",
            &["build", "-p", "leptos-daisyui-rs"],
            None,
        ),
        cmd(
            "check-demo",
            "cargo",
            &["check", "-p", "leptos-daisyui-showcase"],
            None,
        ),
        cmd(
            "test-lib",
            "cargo",
            &["test", "-p", "leptos-daisyui-rs", "--lib"],
            None,
        ),
        cmd("test-xtask", "cargo", &["test", "-p", "xtask"], None),
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

/// The reactivity step, appended to `verify-full` (never to `verify`: it needs
/// npm/trunk/Chrome and a wasm build, and `verify` is deliberately fast and
/// zero-tooling — see `doc/ci-cd.md`).
fn reactivity_step() -> Step {
    Step {
        name: "test-reactivity",
        run: Run::Reactivity,
    }
}

fn run_step(step: &Step) -> bool {
    eprintln!("\n----- {} -----", step.name);
    match &step.run {
        Run::Cmd { program, args, cwd } => {
            let mut c = Command::new(program);
            c.args(args);
            if let Some(dir) = cwd {
                c.current_dir(dir);
            }
            match c.status() {
                Ok(s) => s.success(),
                Err(e) => {
                    eprintln!("xtask: failed to launch {program}: {e}");
                    false
                }
            }
        }
        Run::Reactivity => run_reactivity_suite(),
    }
}

// ---------------------------------------------------------------------------
// Demo dev server (for the reactivity/DOM-oracle suite)
// ---------------------------------------------------------------------------

/// On Windows `npm` is a `.cmd` shim, and `Command::new` only auto-appends
/// `.exe` — so it must be named explicitly. (`trunk` is a real `.exe`, installed
/// by cargo, and needs no such treatment.)
fn npm_bin() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

/// Ask the OS for an unused loopback port, then release it. Each `xtask`
/// invocation therefore gets its own port instead of contending on a shared
/// 3010 (the shared-port flake documented in Rust-DeskApp's `doc/ci-cd.md`).
///
/// This is a bind-then-close race in principle; in practice the window is
/// microseconds and the server binds immediately after.
fn free_port() -> Result<u16, String> {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .map_err(|e| format!("could not reserve a free port: {e}"))
}

/// `GET /` over a raw socket, true only on `HTTP/1.1 200`. Std-only because
/// `xtask` deliberately has zero dependencies (see `doc/ci-cd.md`).
///
/// A 200 on `/` means Trunk has written `index.html` into `dist`, i.e. the
/// first wasm build finished — a stricter and more useful signal than "the
/// port is bound", which Trunk does before it starts building.
fn http_ok(port: u16) -> bool {
    let Ok(mut s) = TcpStream::connect(("127.0.0.1", port)) else {
        return false;
    };
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let req = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    if s.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0u8; 64];
    let Ok(n) = s.read(&mut buf) else {
        return false;
    };
    String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/1.1 200")
}

/// A `trunk serve` child, killed (with its whole process tree) on drop.
struct DemoServer {
    child: Child,
    port: u16,
}

impl DemoServer {
    /// Idempotent `npm install` (Trunk's tailwind pre-build hook needs
    /// `demo/node_modules`, and a fresh worktree has none), then `trunk serve`
    /// on a free port, then poll until it answers 200.
    fn start() -> Result<Self, String> {
        if !std::path::Path::new("demo/node_modules").exists() {
            eprintln!("xtask: npm install in demo/ (tailwind pre-build hook)");
            let ok = Command::new(npm_bin())
                .arg("install")
                .current_dir("demo")
                .status()
                .map_err(|e| format!("failed to launch npm: {e}"))?
                .success();
            if !ok {
                return Err("npm install failed".into());
            }
        }

        let port = free_port()?;
        eprintln!("xtask: starting `trunk serve` in demo/ on port {port}");
        let child = Command::new("trunk")
            .args([
                "serve",
                "--address",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--no-autoreload=true",
                "--open=false",
            ])
            .current_dir("demo")
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to launch trunk: {e}"))?;

        let mut server = DemoServer { child, port };

        // The first dev-profile wasm build can take several minutes.
        let deadline = Instant::now() + Duration::from_secs(900);
        while !http_ok(port) {
            match server.child.try_wait() {
                Ok(Some(status)) => return Err(format!("trunk serve exited early: {status}")),
                Ok(None) => {}
                Err(e) => return Err(format!("waiting on trunk: {e}")),
            }
            if Instant::now() > deadline {
                return Err(format!(
                    "demo server did not come up on port {port} in 15 min"
                ));
            }
            std::thread::sleep(Duration::from_secs(3));
        }
        eprintln!("xtask: demo server is up on port {port}");
        Ok(server)
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for DemoServer {
    fn drop(&mut self) {
        eprintln!("xtask: stopping trunk serve (pid {})", self.child.id());
        if cfg!(windows) {
            // trunk spawns cargo/wasm-bindgen children; /T kills the tree.
            let _ = Command::new("taskkill")
                .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        } else {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

/// Run the reactivity/DOM-oracle suite against a dedicated demo server.
///
/// The suite stays `#[ignore]`d (hence `--ignored`) so a bare `cargo test`
/// still passes with no server running; the gate opts in explicitly. Only the
/// *screenshot* suite (`visual_smoke`) is genuinely manual — its SSIM
/// comparisons are DPI/monitor-specific. See `doc/ci-cd.md`.
///
/// `--test-threads=1`: each test drives its own headless Chrome loading the
/// ~60 MB dev wasm; parallel instances starve each other past the mount-wait
/// budget.
///
/// An externally supplied `VISUAL_TEST_BASE_URL` (a server the caller already
/// has running) is honoured, and no server is spawned.
fn run_reactivity_suite() -> bool {
    let reused = std::env::var("VISUAL_TEST_BASE_URL").ok();
    let _server;
    let base = match &reused {
        Some(url) => {
            eprintln!("xtask: reusing demo server at {url} (VISUAL_TEST_BASE_URL)");
            url.clone()
        }
        None => match DemoServer::start() {
            Ok(s) => {
                let url = s.base_url();
                _server = s;
                url
            }
            Err(e) => {
                eprintln!("xtask: {e}");
                return false;
            }
        },
    };

    Command::new("cargo")
        .args([
            "test",
            "-p",
            "leptos-daisyui-rs",
            "--test",
            "reactivity_smoke",
            "--",
            "--ignored",
            "--test-threads=1",
        ])
        .env("VISUAL_TEST_BASE_URL", &base)
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|e| {
            eprintln!("xtask: failed to launch cargo test: {e}");
            false
        })
    // `_server` drops here -> trunk process tree killed.
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

/// Pure: SemVer arithmetic. `level` is one of "patch" | "minor" | "major".
fn bump_version(current: &str, level: &str) -> Result<String, String> {
    let mut parts = current.split('.');
    let mut nums = [0u64; 3];
    for slot in nums.iter_mut() {
        let p = parts
            .next()
            .ok_or_else(|| format!("not MAJOR.MINOR.PATCH: {current:?}"))?;
        *slot = p
            .parse()
            .map_err(|_| format!("non-numeric version segment: {p:?}"))?;
    }
    if parts.next().is_some() {
        return Err(format!("too many version segments: {current:?}"));
    }
    let [maj, min, pat] = nums;
    let bumped = match level {
        "patch" => (maj, min, pat + 1),
        "minor" => (maj, min + 1, 0),
        "major" => (maj + 1, 0, 0),
        other => return Err(format!("unknown level {other:?} (patch|minor|major)")),
    };
    Ok(format!("{}.{}.{}", bumped.0, bumped.1, bumped.2))
}

/// Pure: rewrite the `version = "..."` line inside the first `[package]` table
/// only, leaving dependency versions untouched.
fn set_package_version(cargo_toml: &str, new_version: &str) -> Result<String, String> {
    let mut in_package = false;
    let mut done = false;
    let mut out = String::with_capacity(cargo_toml.len());
    for line in cargo_toml.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_package = trimmed.starts_with("[package]");
        }
        if in_package && !done && trimmed.starts_with("version") && trimmed.contains('=') {
            let indent = &line[..line.len() - trimmed.len()];
            out.push_str(&format!("{indent}version = \"{new_version}\""));
            out.push('\n');
            done = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !done {
        return Err("no `version` under a [package] table".into());
    }
    Ok(out)
}

/// Read the `leptos-daisyui-rs` library version from the root `Cargo.toml`.
fn current_package_version(cargo_toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let t = line.trim_start();
        if t.starts_with('[') {
            in_package = t.starts_with("[package]");
        }
        if in_package && t.starts_with("version") {
            return t.split('"').nth(1).map(|s| s.to_string());
        }
    }
    None
}

/// `cargo xtask bump <patch|minor|major> [--dry-run]` — rewrites only the
/// library's `[package] version` in the root Cargo.toml.
fn bump(level: &str, dry_run: bool) -> ExitCode {
    let path = "Cargo.toml";
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("xtask bump: read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let current = current_package_version(&src).unwrap_or_default();
    let next = match bump_version(&current, level) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("xtask bump: {e}");
            return ExitCode::from(1);
        }
    };
    let new_src = match set_package_version(&src, &next) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("xtask bump: {e}");
            return ExitCode::from(1);
        }
    };
    if dry_run {
        println!("xtask bump: {current} -> {next} (dry run, Cargo.toml unchanged)");
        return ExitCode::from(0);
    }
    match std::fs::write(path, new_src) {
        Ok(()) => {
            println!("xtask bump: {current} -> {next}");
            println!("next: review, `cargo build` to refresh Cargo.lock, commit, tag v{next}");
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("xtask bump: write {path}: {e}");
            ExitCode::from(1)
        }
    }
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
            // verify + the reactivity suite + the real wasm build
            // (needs npm/trunk/tailwind/Chrome installed).
            let mut steps = gate_steps();
            steps.push(reactivity_step());
            steps.push(cmd(
                "trunk-build",
                "trunk",
                &["build", "--release"],
                Some("demo"),
            ));
            run_steps(&steps)
        }
        "test-reactivity" => run_steps(&[reactivity_step()]),
        "bump" => {
            let level = std::env::args().nth(2).unwrap_or_default();
            let dry = std::env::args().any(|a| a == "--dry-run");
            bump(&level, dry)
        }
        other => {
            eprintln!("xtask: unknown subcommand {other:?}");
            eprintln!(
                "usage: cargo xtask <verify|verify-full|fmt-check|clippy|build|check-demo|test|test-reactivity|bump>"
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

    /// The reactivity suite must never leak into the fast, zero-tooling gate.
    #[test]
    fn reactivity_is_not_a_default_gate_step() {
        assert!(!gate_steps().iter().any(|s| s.name == "test-reactivity"));
        assert!(
            gate_steps()
                .iter()
                .all(|s| matches!(s.run, Run::Cmd { .. }))
        );
    }

    #[test]
    fn reactivity_step_is_in_process() {
        let s = reactivity_step();
        assert_eq!(s.name, "test-reactivity");
        assert!(matches!(s.run, Run::Reactivity));
    }

    /// A port the OS hands out must actually be bindable.
    #[test]
    fn free_port_is_bindable() {
        let p = free_port().expect("free port");
        assert!(p > 0);
        std::net::TcpListener::bind(("127.0.0.1", p)).expect("port should be free");
    }

    /// Nothing is listening on a just-released port, so the probe says "not up".
    #[test]
    fn http_ok_is_false_when_nothing_listens() {
        let p = free_port().expect("free port");
        assert!(!http_ok(p));
    }

    #[test]
    fn bump_version_arithmetic() {
        assert_eq!(bump_version("0.0.4", "patch").unwrap(), "0.0.5");
        assert_eq!(bump_version("0.0.4", "minor").unwrap(), "0.1.0");
        assert_eq!(bump_version("1.2.3", "major").unwrap(), "2.0.0");
        assert!(bump_version("0.0.4", "sideways").is_err());
        assert!(bump_version("not.a.version", "patch").is_err());
        assert!(bump_version("1.2.3.4", "patch").is_err());
    }

    #[test]
    fn set_package_version_only_touches_package_table() {
        let input = "\
[package]
name = \"leptos-daisyui-rs\"
version = \"0.0.4\"
publish = false

[dependencies]
some-dep = { version = \"0.0.4\", path = \"../x\" }
";
        let out = set_package_version(input, "0.0.5").unwrap();
        assert!(out.contains("[package]\nname = \"leptos-daisyui-rs\"\nversion = \"0.0.5\""));
        // the dependency's version must be untouched
        assert!(out.contains("some-dep = { version = \"0.0.4\""));
    }

    #[test]
    fn set_package_version_errors_without_package_version() {
        assert!(set_package_version("[workspace]\nmembers = []\n", "1.0.0").is_err());
    }

    #[test]
    fn current_package_version_reads_the_package_table() {
        let input =
            "[package]\nname = \"x\"\nversion = \"1.2.3\"\n\n[dependencies]\nd = \"9.9.9\"\n";
        assert_eq!(current_package_version(input).as_deref(), Some("1.2.3"));
    }
}
