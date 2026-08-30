//! leptos-daisyui-rs local CI/CD logic. See `doc/ci-cd.md`.
//! Run via the `cargo xtask <sub>` alias (`.cargo/config.toml`).
//!
//! The gate is advisory-first: every step runs even after one fails, a
//! PASS/FAIL summary is printed, and the process exit code is the number of
//! failed steps (0 = all green).

mod pattern_checks;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use pattern_checks::{PatternCheck, PatternLane};

/// What a gate step actually does.
enum Run {
    /// Shell out to a subprocess.
    Cmd {
        program: &'static str,
        args: Vec<String>,
        cwd: Option<&'static str>,
    },
    /// Spawn the demo dev server on a free port, run a browser-driven test
    /// binary against it, then tear the server down. See [`run_browser_suite`].
    BrowserSuite {
        test: &'static str,
        html_target: Option<&'static str>,
    },
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
        // Cheapest step, and the one whose failure invalidates the visual
        // baselines: if `styles/tokens.css` no longer matches the tokens,
        // the two faces have silently forked.
        cmd(
            "tokens-fresh",
            "cargo",
            &["run", "-q", "-p", "xtask", "--", "gen-tokens", "--check"],
            None,
        ),
        // Cheap, and it catches the one failure mode this repo's own gate is
        // structurally blind to: a `ui_tokens` item that exists only on an
        // unmerged sibling branch (ldui-ae5). Everything below compiles
        // happily against a checked-out branch and tells you nothing.
        cmd(
            "sibling-tokens",
            "cargo",
            &["run", "-q", "-p", "xtask", "--", "check-sibling-tokens"],
            None,
        ),
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
                "-p",
                "ldui-audit",
                "--",
                "--check",
            ],
            None,
        ),
        // `--features test-mode`: `src/test_mode.rs` is behind that feature,
        // so a default-feature clippy never lints it and a default-feature
        // `cargo test` never runs its unit tests — the module the browser
        // suites depend on was invisible to the gate.
        cmd(
            "clippy-lib",
            "cargo",
            &[
                "clippy",
                "-p",
                "leptos-daisyui-rs",
                "--all-targets",
                "--features",
                "test-mode",
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
            "clippy-audit",
            "cargo",
            &[
                "clippy",
                "-p",
                "ldui-audit",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            None,
        ),
        // The gate must lint the crate that IS the gate. Omitting this let a
        // `needless_borrows_for_generic_args` sit in `xtask` from 2026-07-26
        // until 2026-08-10 while `verify` reported a clean 13/13 (ldui-mpm).
        // `test-xtask` was already running its tests, so only the lint was
        // missing. Per-crate, never `--workspace`: that fails on leptos `csr`
        // feature unification.
        cmd(
            "clippy-xtask",
            "cargo",
            &[
                "clippy",
                "-p",
                "xtask",
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
        // `--features test-mode`, same reason as `clippy-lib` above: without
        // it the 7 `test_mode` unit tests silently do not run.
        cmd(
            "test-lib",
            "cargo",
            &[
                "test",
                "-p",
                "leptos-daisyui-rs",
                "--lib",
                "--features",
                "test-mode",
            ],
            None,
        ),
        cmd("test-xtask", "cargo", &["test", "-p", "xtask"], None),
        cmd(
            "test-audit",
            "cargo",
            &["test", "-p", "ldui-audit", "--lib"],
            None,
        ),
        // Guards against the daisyUI 4 form classes coming back. They are
        // no-ops in daisyUI 5 and were silently inert in 206 places
        // (ldui-mai.3) — a pure source scan, so it needs no browser.
        cmd(
            "test-daisyui5",
            "cargo",
            &[
                "test",
                "-p",
                "leptos-daisyui-rs",
                "--test",
                "no_dead_daisyui4_classes",
            ],
            None,
        ),
        // Guards every SVG colour in `src/` against riding on a
        // `fill=`/`stroke=` presentation attribute, where `var()` substitution
        // is unspecified (ldui-1g5, widened past `src/charts` in ldui-xxc after
        // a dead daisyUI-4 token shipped in the Gantt timeline while this step
        // read green). Also a pure source scan, so also browser-free — and it
        // has to be its own step: `test-lib` runs the library's unit tests
        // only, so an integration test not named here never runs in the gate.
        cmd(
            "test-svg-paint",
            "cargo",
            &[
                "test",
                "-p",
                "leptos-daisyui-rs",
                "--test",
                "svg_paint_routing",
            ],
            None,
        ),
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
        run: Run::BrowserSuite {
            test: "reactivity_smoke",
            html_target: None,
        },
    }
}

/// Focused browser proof for the opinionated client-snapshot list pattern.
fn client_snapshot_step() -> Step {
    Step {
        name: "test-client-snapshot",
        run: Run::BrowserSuite {
            test: "entity_table_smoke",
            html_target: Some("client-snapshot-test-host.html"),
        },
    }
}

/// Focused browser proof for the typed SnapshotTablePage composition root.
fn snapshot_table_step() -> Step {
    Step {
        name: "test-snapshot-table",
        run: Run::BrowserSuite {
            test: "snapshot_table_smoke",
            html_target: Some("client-snapshot-test-host.html"),
        },
    }
}

fn pattern_steps(pattern: &str, lane: PatternLane) -> Result<Vec<Step>, String> {
    pattern_checks::checks_for(pattern, lane)?
        .iter()
        .map(|check| match check {
            PatternCheck::Cargo { name, args } => Ok(cmd(name, "cargo", args, None)),
            PatternCheck::Browser {
                name,
                test,
                html_target,
            } => Ok(Step {
                name,
                run: Run::BrowserSuite {
                    test,
                    html_target: Some(html_target),
                },
            }),
        })
        .collect()
}

/// The layout-audit step (ldui-dg2): overlap / grid / internal-vs-external
/// assertions swept over the rendered DOM. Same tooling requirements as the
/// reactivity suite, so it lives alongside it in `verify-full` rather than in
/// the fast `verify` gate.
fn layout_step() -> Step {
    Step {
        name: "test-layout",
        run: Run::BrowserSuite {
            test: "layout_audit_smoke",
            html_target: None,
        },
    }
}

/// The style-audit step (Task 7 of the visual-quality-checks plan): the
/// engine's typography/shape/depth sweep plus daisyUI component-drift,
/// ratcheted per page. Same tooling requirements as the other browser
/// suites, so it lives alongside them in `verify-full` rather than in the
/// fast `verify` gate.
fn style_step() -> Step {
    Step {
        name: "test-style",
        run: Run::BrowserSuite {
            test: "style_audit_smoke",
            html_target: None,
        },
    }
}

/// Focused browser proof for `KeyedResultList` (ldui-r1z Task 3): stable-key
/// selection/activation across duplicate labels and asynchronous result
/// replacement. Lives on the general demo app (`html_target: None`, like
/// [`reactivity_step`]/[`layout_step`]/[`style_step`]) rather than a
/// dedicated test-host page, and stays in its own file/step rather than
/// growing `reactivity_smoke.rs` because that suite's check count is pinned.
fn result_list_step() -> Step {
    Step {
        name: "test-keyed-result-list",
        run: Run::BrowserSuite {
            test: "keyed_result_list_smoke",
            html_target: None,
        },
    }
}

/// The full release gate. The catalog browser suites are deliberately
/// consecutive: [`run_steps`] reuses one verified release server for adjacent
/// suites targeting the same HTML entry point.
fn full_steps() -> Vec<Step> {
    let mut steps = gate_steps();
    steps.push(client_snapshot_step());
    steps.push(snapshot_table_step());
    steps.push(reactivity_step());
    steps.push(layout_step());
    steps.push(style_step());
    steps.push(result_list_step());
    steps
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
        Run::BrowserSuite { test, html_target } => run_browser_suite(test, *html_target),
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

/// Build the demo's stylesheet **before** trunk gets a chance to, so a broken
/// `demo/input.css` fails the gate step in seconds with a message naming the
/// stylesheet build.
///
/// Trunk runs the same script from its own `pre_build` hook, but it
/// **swallows the hook's non-zero exit** and carries on serving the previous
/// `output.css` (ldui-hun). Running it here is the fast, loud half of the
/// guard; the durable half is the run-unique marker `build-css.mjs` stamps
/// into the stylesheet, which the browser harness verifies against the CSS the
/// page actually loaded. See `demo/build-css.mjs`.
fn build_demo_stylesheet() -> Result<(), String> {
    eprintln!("xtask: building demo stylesheet (node build-css.mjs)");
    let ok = Command::new("node")
        .arg("build-css.mjs")
        .current_dir("demo")
        .status()
        .map_err(|e| format!("failed to launch node for the stylesheet build: {e}"))?
        .success();
    if ok {
        Ok(())
    } else {
        Err(
            "the demo STYLESHEET BUILD failed (node demo/build-css.mjs) — \
             demo/output.css is stale, so the browser suites would be auditing \
             the last stylesheet that built rather than the current one. Fix \
             demo/input.css."
                .into(),
        )
    }
}

const CLIENT_SNAPSHOT_FINGERPRINT_FILE: &str =
    "target/pattern-checks/client-snapshot-list/browser.fingerprint";
const CLIENT_SNAPSHOT_SOURCE_INPUTS: &[&str] = &[
    "src",
    "demo/src/demos/client_snapshot_list.rs",
    "demo/src/demos/snapshot_table_page.rs",
    "demo/src/client_snapshot_test_host.rs",
    "demo/client-snapshot-test-host.html",
    "demo/Cargo.toml",
    "demo/input.css",
    "demo/custom-components.css",
    "Cargo.toml",
];

struct BrowserBuildFingerprint {
    path: PathBuf,
    manifest: String,
    matched: bool,
}

impl BrowserBuildFingerprint {
    fn store(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| format!("fingerprint path has no parent: {}", self.path.display()))?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
        std::fs::write(&self.path, &self.manifest)
            .map_err(|error| format!("could not write {}: {error}", self.path.display()))
    }
}

fn browser_build_fingerprint(
    html_target: Option<&str>,
) -> Result<Option<BrowserBuildFingerprint>, String> {
    let Some(html_target) = html_target else {
        return Ok(None);
    };
    if html_target != "client-snapshot-test-host.html" {
        return Err(format!(
            "no checked browser-build fingerprint manifest for {html_target:?}"
        ));
    }

    let source_hash = hash_paths(CLIENT_SNAPSHOT_SOURCE_INPUTS)?;
    let cargo_lock_hash = hash_file(Path::new("Cargo.lock"))?;
    let generated_token_hash = hash_file(Path::new("styles/tokens.css"))?;
    let rust_version = tool_version("rustc")?;
    let trunk_version = tool_version("trunk")?;
    let manifest = pattern_checks::BrowserFingerprintParts {
        command_version: pattern_checks::BROWSER_FINGERPRINT_COMMAND_VERSION,
        source_hash: &source_hash,
        cargo_lock_hash: &cargo_lock_hash,
        generated_token_hash: &generated_token_hash,
        rust_version: &rust_version,
        trunk_version: &trunk_version,
    }
    .manifest();
    let path = PathBuf::from(CLIENT_SNAPSHOT_FINGERPRINT_FILE);
    let matched = std::fs::read_to_string(&path).is_ok_and(|stored| stored == manifest);
    Ok(Some(BrowserBuildFingerprint {
        path,
        manifest,
        matched,
    }))
}

fn hash_paths(paths: &[&str]) -> Result<String, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_input_files(Path::new(path), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut material = Vec::new();
    for path in files {
        material.extend_from_slice(path.to_string_lossy().replace('\\', "/").as_bytes());
        material.push(0);
        material.extend_from_slice(
            &std::fs::read(&path)
                .map_err(|error| format!("could not read {}: {error}", path.display()))?,
        );
        material.push(0xff);
    }
    Ok(pattern_checks::hash_bytes(&material))
}

fn collect_input_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        files.push(path.to_owned());
        return Ok(());
    }
    if !path.is_dir() {
        return Err(format!(
            "browser fingerprint input is missing: {}",
            path.display()
        ));
    }
    let entries = std::fs::read_dir(path)
        .map_err(|error| format!("could not enumerate {}: {error}", path.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("could not enumerate {}: {error}", path.display()))?;
        let child = entry.path();
        if child.is_dir() {
            collect_input_files(&child, files)?;
        } else if child.is_file() {
            files.push(child);
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, String> {
    std::fs::read(path)
        .map(|bytes| pattern_checks::hash_bytes(&bytes))
        .map_err(|error| format!("could not read {}: {error}", path.display()))
}

fn tool_version(program: &str) -> Result<String, String> {
    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|error| format!("could not launch `{program} --version`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`{program} --version` failed with {}",
            output.status
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Ask the OS for an unused loopback port, then release it. Each `xtask`
/// invocation therefore gets its own port instead of contending on a shared
/// 3010 (the shared-port flake documented in Rust-DeskApp's `doc/ci-cd.md`).
///
/// This is a bind-then-close race in principle; in practice the window is
/// microseconds and the server binds immediately after.
fn free_port() -> Result<u16, String> {
    for _ in 0..32 {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("could not reserve a free port: {e}"))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("could not read the reserved port: {e}"))?
            .port();
        if browser_allows_port(port) {
            return Ok(port);
        }
    }
    Err("the OS repeatedly selected ports Chromium refuses to navigate to".into())
}

/// Mirrors Chromium's generic restricted-port list for HTTP navigation.
///
/// An ephemeral listener can still receive one of these values (Windows chose
/// X11 port 6000 in CI), which otherwise surfaces later as `ERR_UNSAFE_PORT`.
/// Source: Chromium `net/base/port_util.cc`.
fn browser_allows_port(port: u16) -> bool {
    !matches!(
        port,
        0 | 1
            | 7
            | 9
            | 11
            | 13
            | 15
            | 17
            | 19
            | 20
            | 21
            | 22
            | 23
            | 25
            | 37
            | 42
            | 43
            | 53
            | 69
            | 77
            | 79
            | 87
            | 95
            | 101
            | 102
            | 103
            | 104
            | 109
            | 110
            | 111
            | 113
            | 115
            | 117
            | 119
            | 123
            | 135
            | 137
            | 139
            | 143
            | 161
            | 179
            | 389
            | 427
            | 465
            | 512
            | 513
            | 514
            | 515
            | 526
            | 530
            | 531
            | 532
            | 540
            | 548
            | 554
            | 556
            | 563
            | 587
            | 601
            | 636
            | 989
            | 990
            | 993
            | 995
            | 1719
            | 1720
            | 1723
            | 2049
            | 3659
            | 4045
            | 5060
            | 5061
            | 6000
            | 6566
            | 6665
            | 6666
            | 6667
            | 6668
            | 6669
            | 6697
            | 10080
    )
}

/// Stable Trunk arguments for a headless browser-suite server.
fn trunk_serve_args(port: u16, html_target: Option<&str>) -> Vec<String> {
    let mut args: Vec<String> = [
        "serve".to_owned(),
        "--address".to_owned(),
        "127.0.0.1".to_owned(),
        "--port".to_owned(),
        port.to_string(),
        "--no-autoreload=true".to_owned(),
        "--open=false".to_owned(),
        "--color".to_owned(),
        "never".to_owned(),
    ]
    .into();
    if html_target.is_none() {
        args.push("--release=true".to_owned());
    }
    if let Some(target) = html_target {
        args.push(target.to_owned());
    }
    args
}

/// Fetch an asset from Trunk over a raw socket. Std-only because `xtask`
/// deliberately has zero dependencies (see `doc/ci-cd.md`).
fn http_get(port: u16, path: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|error| format!("server is not accepting connections: {error}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("could not request {path}: {error}"))?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| format!("could not read {path}: {error}"))?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| format!("{path} returned an invalid HTTP response"))?;
    let headers = String::from_utf8_lossy(&response[..header_end]);
    let status = headers.lines().next().unwrap_or_default();
    if !status.starts_with("HTTP/1.1 200") && !status.starts_with("HTTP/1.0 200") {
        return Err(format!("{path} returned {status}"));
    }
    Ok(String::from_utf8_lossy(&response[header_end + 4..]).into_owned())
}

fn output_stylesheet_path(html: &str) -> Option<&str> {
    const PREFIX: &str = "href=\"/output-";
    let start = html.find(PREFIX)? + "href=\"".len();
    let remainder = &html[start..];
    let end = remainder.find('"')?;
    Some(&remainder[..end])
}

fn browser_assets_match(html: &str, css: &str, stamp: &str, html_target: Option<&str>) -> bool {
    let expected_app = if html_target.is_some() {
        "client-snapshot-test-host-"
    } else {
        "leptos-daisyui-showcase-"
    };
    if !html.contains(expected_app) {
        return false;
    }

    let mut stamp_parts = stamp.split_whitespace();
    let Some("ok") = stamp_parts.next() else {
        return false;
    };
    let Some(token) = stamp_parts.next() else {
        return false;
    };
    stamp_parts.next().is_none() && css.contains(&format!("#ldui-css-stamp-{token}"))
}

/// A listening Trunk port is not enough: during a cold build it can serve the
/// previous contents of `dist/` before the new asset pipeline completes. Wait
/// for both the requested app binary and this build's run-unique CSS stamp.
fn browser_server_ready(port: u16, html_target: Option<&str>) -> Result<bool, String> {
    let html = match http_get(port, "/") {
        Ok(html) => html,
        Err(_) => return Ok(false),
    };
    let Some(stylesheet_path) = output_stylesheet_path(&html) else {
        return Ok(false);
    };
    let css = match http_get(port, stylesheet_path) {
        Ok(css) => css,
        Err(_) => return Ok(false),
    };
    let stamp = std::fs::read_to_string("demo/.ldui-css-stamp")
        .map_err(|error| format!("could not read demo/.ldui-css-stamp: {error}"))?;
    if stamp.starts_with("fail ") {
        return Err("Trunk's stylesheet hook failed; demo/output.css was not regenerated".into());
    }
    Ok(browser_assets_match(&html, &css, &stamp, html_target))
}

/// A `trunk serve` child, killed (with its whole process tree) on drop.
struct DemoServer {
    child: Child,
    port: u16,
}

impl DemoServer {
    /// Idempotent `npm install` (Trunk's tailwind pre-build hook needs
    /// `demo/node_modules`, and a fresh worktree has none), then the demo
    /// stylesheet build (see [`build_demo_stylesheet`]), then `trunk serve` on
    /// a free port, then poll until the requested app and current stylesheet
    /// are both present in the served distribution.
    fn start(html_target: Option<&str>) -> Result<Self, String> {
        let build_fingerprint = browser_build_fingerprint(html_target)?;
        if let Some(fingerprint) = &build_fingerprint {
            if fingerprint.matched {
                eprintln!("xtask: matching page-build fingerprint; reusing Cargo/Trunk artifacts");
            } else {
                eprintln!(
                    "xtask: page-build fingerprint changed; Cargo/Trunk will rebuild affected inputs"
                );
            }
        }
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

        build_demo_stylesheet()?;

        let port = free_port()?;
        let target_label = html_target.unwrap_or("index.html");
        eprintln!("xtask: starting `trunk serve {target_label}` in demo/ on port {port}");
        let child = Command::new("trunk")
            // Trunk 0.21 treats the conventional `NO_COLOR=1` value as a
            // Boolean CLI value and rejects it. Isolate the child from that
            // ambient setting and express the intent through Trunk's stable
            // color enum instead.
            .env_remove("NO_COLOR")
            .args(trunk_serve_args(port, html_target))
            .current_dir("demo")
            // Trunk reports asset-pipeline failures on stdout and keeps its
            // watcher alive. Preserve that output so an ambiguous/missing
            // Wasm target or asset failure is visible immediately in CI.
            .stdout(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("failed to launch trunk: {e}"))?;

        let mut server = DemoServer { child, port };

        // A first full-catalog release build can take several minutes. Do not
        // accept a mere HTTP 200: Trunk may temporarily serve the previous
        // `dist/` while its new asset pipeline is still finishing.
        let deadline = Instant::now() + Duration::from_secs(900);
        loop {
            match browser_server_ready(port, html_target) {
                Ok(true) => break,
                Ok(false) => {}
                Err(error) => return Err(error),
            }
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
        eprintln!("xtask: demo server assets are current on port {port}");
        if let Some(fingerprint) = &build_fingerprint {
            fingerprint.store()?;
        }
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
/// catalog Wasm; parallel instances starve each other past the mount-wait
/// budget.
///
fn run_browser_suite(test: &str, html_target: Option<&str>) -> bool {
    let server = match DemoServer::start(html_target) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("xtask: {error}");
            return false;
        }
    };
    run_browser_test(test, &server.base_url())
    // `server` drops here -> trunk process tree killed.
}

fn run_browser_test(test: &str, base: &str) -> bool {
    Command::new("cargo")
        .args([
            "test",
            "-p",
            "leptos-daisyui-rs",
            "--test",
            test,
            "--",
            "--ignored",
            "--test-threads=1",
        ])
        .env("VISUAL_TEST_BASE_URL", base)
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|e| {
            eprintln!("xtask: failed to launch cargo test: {e}");
            false
        })
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

// ---------------------------------------------------------------------------
// gen-tokens — emit the Tailwind theme from `ui-tokens` (ldui-1mx)
// ---------------------------------------------------------------------------

/// Where the generated Tailwind theme lands. `demo/input.css` imports it, and
/// so should any consuming app's own `input.css`.
const TOKENS_CSS_PATH: &str = "styles/tokens.css";

/// The CSS root font size the DIP -> rem conversion assumes, in px. This is
/// the browser default and the value Tailwind's own scale is built around.
const ROOT_FONT_PX: f32 = 16.0;

/// Format a DIP as a CSS px length, without a trailing `.0`.
///
/// Used only where a dimension must NOT scale with the user's font size —
/// border widths. A 1px hairline that grows to 1.5px when someone bumps their
/// browser font size is a rendering bug, not an accessibility win.
fn px(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{}px", v as i64)
    } else {
        format!("{v}px")
    }
}

/// Format a DIP as rem, relative to [`ROOT_FONT_PX`].
///
/// Spacing, type sizes and radii are emitted in rem so they keep scaling with
/// the user's browser font-size preference. The token crate stores DIPs
/// because Direct2D has no notion of rem; converting here is the whole reason
/// the two faces can share one scale without the web side breaking
/// WCAG 1.4.4 (Resize Text).
///
/// Every value on the canonical scale divides 16 exactly, so this is lossless
/// in practice (4 -> 0.25rem, 96 -> 6rem, 11 -> 0.6875rem).
fn rem(v: f32) -> String {
    let r = v / ROOT_FONT_PX;
    if r.fract() == 0.0 {
        format!("{}rem", r as i64)
    } else {
        format!("{r}rem")
    }
}

/// Format a line height as the unitless ratio `line / size`.
///
/// Unitless is deliberate and matches what Tailwind ships: a ratio inherits
/// correctly into nested elements, where an absolute length would pin
/// descendants to the ancestor's leading. Emitted as `calc(20 / 14)` rather
/// than a rounded decimal so the arithmetic stays exact and the source
/// numbers stay legible.
fn line_ratio(line: f32, size: f32) -> String {
    if (line - size).abs() < f32::EPSILON {
        return "1".to_string();
    }
    format!("calc({} / {})", trim(line), trim(size))
}

/// Render an `f32` without a trailing `.0`.
fn trim(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// Build the Tailwind v4 `@theme` block from the shared token crate.
///
/// Pure — takes no input and touches no files, so the drift check and the
/// writer compare the same bytes.
///
/// Two things are happening here, and only the second is cosmetic:
///
/// 1. `--spacing` is pinned to the token sub-unit, so Tailwind's *numeric*
///    scale (`p-4`, `gap-2`, …) is derived from `ui-tokens` rather than
///    merely agreeing with it by coincidence. Every existing `p-4` in the
///    codebase keeps its current computed value; what changes is where that
///    value comes from.
/// 2. Named aliases (`p-m`, `gap-l`, `text-body`) are added for call sites
///    that want to say which *step* they mean instead of which multiple.
///
/// Units are not incidental. Spacing and type are emitted in **rem** and line
/// heights as **unitless ratios**, exactly as Tailwind ships them, so the
/// output is byte-for-byte equivalent in behaviour to the defaults it
/// replaces. Emitting the tokens' raw DIPs as `px` here would have pinned
/// every font size and gap against the user's browser font-size preference —
/// a WCAG 1.4.4 regression traded for nothing.
fn tokens_css() -> String {
    use ui_tokens::{color, radius, spacing, stroke, typography as ty};

    let mut css = String::with_capacity(2048);
    css.push_str(concat!(
        "/* GENERATED by `cargo xtask gen-tokens` — do not edit by hand.\n",
        " *\n",
        " * Source of truth: the `ui-tokens` crate, shared with the Direct2D\n",
        " * desktop face in Rust-DeskApp. Edit the tokens there and re-run the\n",
        " * generator; `cargo xtask verify` fails if this file drifts.\n",
        " *\n",
        " * The token crate stores DIPs (1 DIP = 1 CSS px at a 16px root).\n",
        " * Spacing, type and radii are converted to rem here so they keep\n",
        " * scaling with the user's font-size preference; only border widths\n",
        " * stay in px, because a hairline must not grow with the type.\n",
        " */\n\n",
        "@theme {\n",
    ));

    css.push_str("  /* Base unit. Tailwind derives its whole numeric spacing\n");
    css.push_str("     scale from this, so `p-4` is 4 x 0.25rem = 1rem *because\n");
    css.push_str("     of* the token, not by coincidence. */\n");
    css.push_str(&format!("  --spacing: {};\n\n", rem(spacing::SPACE_XXS)));

    css.push_str("  /* Named steps are deliberately NOT emitted into the\n");
    css.push_str("     --spacing-* namespace. Tailwind resolves width and\n");
    css.push_str("     max-width utilities against --spacing-* before\n");
    css.push_str("     --container-*, so a `--spacing-xs` key silently\n");
    css.push_str("     redefines `max-w-xs` from 20rem to 0.5rem. The numeric\n");
    css.push_str("     scale above is already token-derived and is what every\n");
    css.push_str("     call site uses; semantic aliases would buy nothing and\n");
    css.push_str("     cost a namespace collision. See ldui-1mx. */\n");

    css.push_str("\n  /* Strokes. Borders and dividers are NOT spacing — keeping\n");
    css.push_str("     them in their own family is what stops a 1px hairline\n");
    css.push_str("     being reported as an off-grid gap. */\n");
    for (name, dips) in [
        ("hairline", stroke::HAIRLINE),
        ("thin", stroke::THIN),
        ("accent", stroke::ACCENT),
        ("emphasis", stroke::EMPHASIS),
    ] {
        css.push_str(&format!("  --border-width-{}: {};\n", name, px(dips)));
    }

    css.push_str("\n  /* Opinionated table hierarchy, from ui_tokens::color::table —\n");
    css.push_str("     the dedicated semantic module shared with the Direct2D desktop\n");
    css.push_str("     face, rather than the generic status-color aliases it derives\n");
    css.push_str("     from. This decouples the table role from the generic STATUS_BLUE,\n");
    css.push_str("     TEXT_PRIMARY and CONTROL_BORDER aliases drifting for unrelated\n");
    css.push_str("     reasons. */\n");
    for (name, value) in [
        ("table-header", color::table::HEADER),
        ("table-header-content", color::table::HEADER_CONTENT),
        ("table-filter", color::table::FILTER),
        ("table-filter-content", color::table::FILTER_CONTENT),
        ("table-grid", color::table::GRID),
    ] {
        css.push_str(&format!(
            "  --color-{name}: {};\n",
            color::to_css_hex(value)
        ));
    }

    css.push_str("\n  /* Corner radii. */\n");
    for (name, dips) in [
        ("control", radius::CONTROL),
        ("card", radius::CARD),
        ("badge", radius::BADGE),
    ] {
        css.push_str(&format!("  --radius-{}: {};\n", name, rem(dips)));
    }

    css.push_str("\n  /* Type ramp. Every step pins its line height from the\n");
    css.push_str("     shared LINE_* ramp, so N stacked lines occupy exactly\n");
    css.push_str("     N x line-height instead of N x a font metric. */\n");
    for (name, size, line) in [
        ("display", ty::SIZE_DISPLAY, ty::LINE_DISPLAY),
        ("title", ty::SIZE_TITLE, ty::LINE_TITLE),
        ("subtitle", ty::SIZE_SUBTITLE, ty::LINE_SUBTITLE),
        ("body", ty::SIZE_BODY, ty::LINE_BODY),
        ("caption", ty::SIZE_CAPTION, ty::LINE_CAPTION),
        ("small", ty::SIZE_SMALL, ty::LINE_SMALL),
    ] {
        css.push_str(&format!("  --text-{}: {};\n", name, rem(size)));
        css.push_str(&format!(
            "  --text-{}--line-height: {};\n",
            name,
            line_ratio(line, size)
        ));
    }

    css.push_str("\n  /* Tailwind's own size keys, re-pointed at the same ramp.\n");
    css.push_str("     These resolve to exactly what Tailwind ships — that is the\n");
    css.push_str("     point. The existing `text-xs`/`text-sm` call sites do not\n");
    css.push_str("     move a pixel; they stop being a coincidence. */\n");
    for (key, size, line) in [
        ("xs", ty::SIZE_CAPTION, ty::LINE_CAPTION),
        ("sm", ty::SIZE_BODY, ty::LINE_BODY),
        ("base", ty::SIZE_SUBTITLE, ty::LINE_SUBTITLE),
        ("xl", ty::SIZE_TITLE, ty::LINE_TITLE),
    ] {
        css.push_str(&format!("  --text-{}: {};\n", key, rem(size)));
        css.push_str(&format!(
            "  --text-{}--line-height: {};\n",
            key,
            line_ratio(line, size)
        ));
    }

    css.push_str("}\n");
    css
}

/// Whether two files differ only in line endings.
///
/// The generator writes LF, but this repo is checked out with
/// `core.autocrlf=true`, so git materialises the committed file as CRLF. A
/// raw byte comparison therefore reports the file STALE on every fresh
/// checkout on Windows — which is exactly what happened when the branch was
/// merged: `--check` failed while `git diff` was empty and regenerating
/// produced no change. Compare content, not line terminators.
fn same_ignoring_line_endings(a: &str, b: &str) -> bool {
    a.lines().eq(b.lines())
}

/// `cargo xtask gen-tokens [--check]` — write (or verify) the generated
/// Tailwind theme.
///
/// `--check` is the gate step: it never writes, and fails if the committed
/// file does not match what the tokens currently produce.
fn gen_tokens(check: bool) -> ExitCode {
    let want = tokens_css();
    let have = std::fs::read_to_string(TOKENS_CSS_PATH).ok();

    if check {
        return match have.as_deref() {
            Some(current) if same_ignoring_line_endings(current, &want) => {
                println!("xtask gen-tokens: {TOKENS_CSS_PATH} is up to date");
                ExitCode::from(0)
            }
            Some(_) => {
                eprintln!(
                    "xtask gen-tokens: {TOKENS_CSS_PATH} is STALE — the tokens changed.\n\
                     run `cargo xtask gen-tokens` and commit the result."
                );
                ExitCode::from(1)
            }
            None => {
                eprintln!(
                    "xtask gen-tokens: {TOKENS_CSS_PATH} is missing — run `cargo xtask gen-tokens`"
                );
                ExitCode::from(1)
            }
        };
    }

    if have
        .as_deref()
        .is_some_and(|current| same_ignoring_line_endings(current, &want))
    {
        println!("xtask gen-tokens: {TOKENS_CSS_PATH} already up to date");
        return ExitCode::from(0);
    }
    if let Some(parent) = std::path::Path::new(TOKENS_CSS_PATH).parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!("xtask gen-tokens: create {}: {e}", parent.display());
        return ExitCode::from(1);
    }
    match std::fs::write(TOKENS_CSS_PATH, &want) {
        Ok(()) => {
            println!("xtask gen-tokens: wrote {TOKENS_CSS_PATH}");
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("xtask gen-tokens: write {TOKENS_CSS_PATH}: {e}");
            ExitCode::from(1)
        }
    }
}

// ---------------------------------------------------------------------------
// Sibling-token guard (ldui-ae5)
// ---------------------------------------------------------------------------
//
// `src/tokens/preamble.rs` imports `ui_tokens` items across a *path*
// dependency. Cargo resolves that path to whatever the sibling repo currently
// has checked out, so an item that exists only on an unmerged branch still
// builds here — and this repo's gate stays green while `main` is, in fact,
// unbuildable for anyone whose sibling sits on its default branch. That is
// exactly what happened on 2026-07-29: SPACE_HUGE, SPACE_XXXL, LINE_DISPLAY
// and the whole stroke module were branch-only, and the break surfaced in a
// downstream consumer where it read as the consumer's own fault.
//
// The guard resolves the sibling's DEFAULT branch and asserts every referenced
// item exists there, deliberately ignoring the working tree.

/// The repo providing the `ui-tokens` path dependency, relative to this
/// repo's root. (`cargo xtask` must be run from the root — see `doc/ci-cd.md`.)
const SIBLING_REPO: &str = "../Rust-DeskApp";

/// The `ui-tokens` source directory, relative to [`SIBLING_REPO`].
const UI_TOKENS_SRC: &str = "crates/ui-tokens/src";

/// The file whose `ui_tokens` references this guard checks.
const PREAMBLE_PATH: &str = "src/tokens/preamble.rs";

/// A `ui_tokens` item the preamble depends on — a module (`stroke`), or an
/// item within one (`spacing::SPACE_HUGE`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TokenRef {
    module: String,
    symbol: Option<String>,
}

impl std::fmt::Display for TokenRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.symbol {
            Some(s) => write!(f, "ui_tokens::{}::{}", self.module, s),
            None => write!(f, "ui_tokens::{}", self.module),
        }
    }
}

/// Drop `//` comments so prose naming a token — the doc comment mentioning the
/// `LINE_*` ramp, for instance — is never mistaken for a real reference.
///
/// A `//` inside a string literal would truncate that line early. Nothing in
/// the preamble does that, and the cost of a false *negative* here is only a
/// missed check, never a spurious failure.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse the `use ui_tokens::…;` statements into item references plus the
/// local module aliases they establish (`typography as ty` yields `ty`).
fn parse_use_statements(src: &str) -> (Vec<TokenRef>, BTreeMap<String, String>) {
    const NEEDLE: &str = "use ui_tokens::";
    let mut refs = Vec::new();
    let mut aliases = BTreeMap::new();
    let mut rest = src;

    while let Some(i) = rest.find(NEEDLE) {
        let after = &rest[i + NEEDLE.len()..];
        let Some(end) = after.find(';') else { break };
        let stmt = &after[..end];
        rest = &after[end..];

        if let Some((module, list)) = stmt.split_once("::{") {
            // A braced list, possibly spanning several lines.
            let module = module.trim().to_string();
            for name in list.trim_end().trim_end_matches('}').split(',') {
                let name = name.trim();
                if !name.is_empty() {
                    refs.push(TokenRef {
                        module: module.clone(),
                        symbol: Some(name.to_string()),
                    });
                }
            }
        } else if let Some((module, symbol)) = stmt.split_once("::") {
            refs.push(TokenRef {
                module: module.trim().to_string(),
                symbol: Some(symbol.trim().to_string()),
            });
        } else if let Some((module, alias)) = stmt.split_once(" as ") {
            let module = module.trim().to_string();
            aliases.insert(alias.trim().to_string(), module.clone());
            refs.push(TokenRef {
                module,
                symbol: None,
            });
        } else {
            // A bare module import: its own name is the alias.
            let module = stmt.trim().to_string();
            aliases.insert(module.clone(), module.clone());
            refs.push(TokenRef {
                module,
                symbol: None,
            });
        }
    }
    (refs, aliases)
}

/// Find `alias::SYMBOL` uses in the body, resolving each alias to its real
/// module.
///
/// This is what catches `ty::LINE_DISPLAY`: reached through an alias, so it
/// never appears in a `use` line and a use-statement scan alone would miss it
/// — which is half of the original breakage.
fn parse_qualified_refs(src: &str, aliases: &BTreeMap<String, String>) -> Vec<TokenRef> {
    let chars: Vec<char> = src.chars().collect();
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == ':' && chars.get(i + 1) == Some(&':') {
            let mut start = i;
            while start > 0 && ident(chars[start - 1]) {
                start -= 1;
            }
            let mut end = i + 2;
            while end < chars.len() && ident(chars[end]) {
                end += 1;
            }
            let alias: String = chars[start..i].iter().collect();
            let symbol: String = chars[i + 2..end].iter().collect();
            if !symbol.is_empty()
                && let Some(module) = aliases.get(&alias)
            {
                out.push(TokenRef {
                    module: module.clone(),
                    symbol: Some(symbol),
                });
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// Every `ui_tokens` item the preamble source depends on.
fn token_refs(preamble_src: &str) -> BTreeSet<TokenRef> {
    let src = strip_line_comments(preamble_src);
    let (mut refs, aliases) = parse_use_statements(&src);
    refs.extend(parse_qualified_refs(&src, &aliases));
    refs.into_iter().collect()
}

/// Module names declared by `ui-tokens`' `lib.rs`.
fn modules_declared(lib_src: &str) -> BTreeSet<String> {
    lib_src
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("pub mod ")
                .and_then(|r| r.split(';').next())
                .map(|s| s.trim().to_string())
        })
        .collect()
}

/// Public item names a `ui-tokens` module file defines.
fn defined_items(module_src: &str) -> BTreeSet<String> {
    const KINDS: [&str; 6] = ["const ", "struct ", "enum ", "fn ", "type ", "static "];
    module_src
        .lines()
        .filter_map(|l| {
            let rest = l.trim().strip_prefix("pub ")?;
            let rest = KINDS.iter().find_map(|k| rest.strip_prefix(k))?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

/// The references a tree does not satisfy, given each declared module's items.
/// A module absent from `defined` is itself missing, so every reference into
/// it fails.
fn missing_refs(
    refs: &BTreeSet<TokenRef>,
    defined: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<TokenRef> {
    refs.iter()
        .filter(|r| match defined.get(&r.module) {
            None => true,
            Some(items) => r.symbol.as_ref().is_some_and(|s| !items.contains(s)),
        })
        .cloned()
        .collect()
}

/// Run a git command in `repo`, returning trimmed stdout on success.
fn git(repo: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Which other refs in the sibling repo *do* define an item — the "found on
/// this branch instead" half of the failure message, and usually the whole
/// answer (it names the branch that needs merging).
fn refs_defining(repo: &str, item: &TokenRef) -> Vec<String> {
    let Some(list) = git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes",
        ],
    ) else {
        return Vec::new();
    };
    let path = format!("{UI_TOKENS_SRC}/{}.rs", item.module);
    list.lines()
        .filter(|r| !r.is_empty() && !r.ends_with("/HEAD"))
        .filter(|r| match &item.symbol {
            // A module reference: does the module file exist on that ref?
            None => git(repo, &["cat-file", "-e", &format!("{r}:{path}")]).is_some(),
            Some(sym) => git(repo, &["show", &format!("{r}:{path}")])
                .is_some_and(|src| defined_items(&src).contains(sym)),
        })
        .map(|r| r.to_string())
        .collect()
}

/// What the guard concluded.
enum Guard {
    /// All references resolve on the default branch; carries how many.
    Ok(usize),
    /// The sibling is not available to check against — not a failure.
    Skipped(String),
    Missing {
        branch: String,
        missing: Vec<TokenRef>,
    },
}

/// Check the preamble's `ui_tokens` references against the sibling's default
/// branch. Pure decision-making lives in the helpers above; this is the I/O.
fn check_sibling_tokens_inner() -> Guard {
    if !std::path::Path::new(SIBLING_REPO).is_dir() {
        return Guard::Skipped(format!("{SIBLING_REPO} is not present"));
    }
    // `origin/HEAD` is the sibling's default branch. It is a local alias and
    // can be absent on a shallow or hand-made clone, in which case there is
    // nothing authoritative to check against.
    let Some(branch) = git(SIBLING_REPO, &["rev-parse", "--abbrev-ref", "origin/HEAD"]) else {
        return Guard::Skipped(format!(
            "{SIBLING_REPO} has no origin/HEAD (run: git remote set-head origin -a)"
        ));
    };
    let preamble = match std::fs::read_to_string(PREAMBLE_PATH) {
        Ok(s) => s,
        Err(e) => return Guard::Skipped(format!("cannot read {PREAMBLE_PATH}: {e}")),
    };
    let Some(lib) = git(
        SIBLING_REPO,
        &["show", &format!("{branch}:{UI_TOKENS_SRC}/lib.rs")],
    ) else {
        return Guard::Skipped(format!("cannot read ui-tokens lib.rs at {branch}"));
    };

    let mut defined: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for m in modules_declared(&lib) {
        let path = format!("{branch}:{UI_TOKENS_SRC}/{m}.rs");
        if let Some(src) = git(SIBLING_REPO, &["show", &path]) {
            defined.insert(m, defined_items(&src));
        }
    }

    let refs = token_refs(&preamble);
    let missing = missing_refs(&refs, &defined);
    if missing.is_empty() {
        Guard::Ok(refs.len())
    } else {
        Guard::Missing { branch, missing }
    }
}

fn check_sibling_tokens() -> ExitCode {
    match check_sibling_tokens_inner() {
        Guard::Skipped(why) => {
            println!("xtask sibling-tokens: skipped — {why}");
            ExitCode::from(0)
        }
        Guard::Ok(n) => {
            println!(
                "xtask sibling-tokens: {n} ui_tokens references all resolve on the sibling's default branch"
            );
            ExitCode::from(0)
        }
        Guard::Missing { branch, missing } => {
            eprintln!(
                "xtask sibling-tokens: {PREAMBLE_PATH} references {} ui_tokens item(s) that do NOT exist on {SIBLING_REPO}'s default branch ({branch}):",
                missing.len()
            );
            for item in &missing {
                let found = refs_defining(SIBLING_REPO, item);
                if found.is_empty() {
                    eprintln!("  {item} — found on no branch");
                } else {
                    eprintln!("  {item} — found instead on: {}", found.join(", "));
                }
            }
            eprintln!(
                "\nThis builds locally because the path dependency resolves to whatever the\n\
                 sibling has checked out, but it is unbuildable for anyone on the default\n\
                 branch. Land the upstream change before committing the reference."
            );
            ExitCode::from(1)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BrowserTarget(Option<&'static str>);

#[cfg(test)]
fn browser_server_segment_count(steps: &[Step]) -> usize {
    let mut active = None;
    let mut starts = 0;
    for step in steps {
        match &step.run {
            Run::BrowserSuite { html_target, .. } => {
                let requested = BrowserTarget(*html_target);
                if active != Some(requested) {
                    starts += 1;
                    active = Some(requested);
                }
            }
            Run::Cmd { .. } => active = None,
        }
    }
    starts
}

#[cfg(test)]
fn html_selects_exactly_one_wasm_binary(html: &str, expected_binary: &str) -> bool {
    let mut remaining = html;
    let mut rust_entries = Vec::new();
    while let Some(start) = remaining.find("<link") {
        remaining = &remaining[start..];
        let Some(end) = remaining.find('>') else {
            return false;
        };
        let tag = &remaining[..=end];
        if tag.contains("data-trunk") && tag.contains("rel=\"rust\"") {
            rust_entries.push(tag);
        }
        remaining = &remaining[end + 1..];
    }

    rust_entries.len() == 1 && rust_entries[0].contains(&format!("data-bin=\"{expected_binary}\""))
}

/// Run a set of steps advisory-first: every step runs, then print the summary
/// and return the failure count as the exit code. Consecutive browser suites
/// with the same HTML target share one current Trunk server/build.
fn run_steps(steps: &[Step]) -> ExitCode {
    if steps.is_empty() {
        eprintln!("xtask: no steps to run");
        return ExitCode::from(2);
    }
    let mut active_target = None;
    let mut server: Option<DemoServer> = None;
    let mut results = Vec::with_capacity(steps.len());

    for step in steps {
        let passed = match &step.run {
            Run::Cmd { .. } => {
                server.take();
                active_target = None;
                run_step(step)
            }
            Run::BrowserSuite { test, html_target } => {
                eprintln!("\n----- {} -----", step.name);
                let requested = BrowserTarget(*html_target);
                if active_target != Some(requested) {
                    server.take();
                    active_target = Some(requested);
                    server = match DemoServer::start(*html_target) {
                        Ok(started) => Some(started),
                        Err(error) => {
                            eprintln!("xtask: {error}");
                            None
                        }
                    };
                } else if server.is_some() {
                    eprintln!(
                        "xtask: reusing current {} server",
                        html_target.unwrap_or("index.html")
                    );
                }

                match &server {
                    Some(current) => run_browser_test(test, &current.base_url()),
                    None => {
                        eprintln!(
                            "xtask: {} server is unavailable; build was not retried",
                            html_target.unwrap_or("index.html")
                        );
                        false
                    }
                }
            }
        };
        results.push((step.name, passed));
    }

    let (summary, code) = summarize(&results);
    println!("{summary}");
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let sub = std::env::args().nth(1).unwrap_or_default();
    match sub.as_str() {
        "verify" => run_steps(&gate_steps()),
        "fmt-check" | "clippy" | "build" | "check-demo" | "test" => run_steps(&steps_for(&sub)),
        "verify-full" => run_steps(&full_steps()),
        "test-reactivity" => run_steps(&[reactivity_step()]),
        "test-client-snapshot" => run_steps(&[client_snapshot_step(), snapshot_table_step()]),
        "verify-pattern" => {
            let pattern = std::env::args().nth(2).unwrap_or_default();
            let lane_flag = std::env::args().nth(3).unwrap_or_default();
            if std::env::args().nth(4).is_some() {
                eprintln!("xtask: verify-pattern accepts exactly one pattern and one lane");
                return ExitCode::from(2);
            }
            let lane = match PatternLane::parse_flag(&lane_flag) {
                Ok(lane) => lane,
                Err(error) => {
                    eprintln!("xtask: {error}; expected --inner or --browser");
                    return ExitCode::from(2);
                }
            };
            match pattern_steps(&pattern, lane) {
                Ok(steps) => run_steps(&steps),
                Err(error) => {
                    eprintln!("xtask: {error}");
                    ExitCode::from(2)
                }
            }
        }
        "test-layout" => run_steps(&[layout_step()]),
        "test-style" => run_steps(&[style_step()]),
        "test-keyed-result-list" => run_steps(&[result_list_step()]),
        "gen-tokens" => {
            let check = std::env::args().any(|a| a == "--check");
            gen_tokens(check)
        }
        "check-sibling-tokens" => check_sibling_tokens(),
        "bump" => {
            let level = std::env::args().nth(2).unwrap_or_default();
            let dry = std::env::args().any(|a| a == "--dry-run");
            bump(&level, dry)
        }
        other => {
            eprintln!("xtask: unknown subcommand {other:?}");
            eprintln!(
                "usage: cargo xtask <verify|verify-full|verify-pattern <name> <--inner|--browser>|fmt-check|clippy|build|check-demo|test|test-client-snapshot|test-reactivity|test-layout|test-style|test-keyed-result-list|gen-tokens|check-sibling-tokens|bump>"
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
    fn clippy_subcommand_runs_every_crate_step() {
        let names: Vec<&str> = steps_for("clippy").iter().map(|s| s.name).collect();
        assert_eq!(
            names,
            vec!["clippy-lib", "clippy-demo", "clippy-audit", "clippy-xtask"],
            "every workspace crate must be lint-gated, including xtask itself"
        );
    }

    #[test]
    fn test_subcommand_runs_lib_and_xtask() {
        let names: Vec<&str> = steps_for("test").iter().map(|s| s.name).collect();
        assert_eq!(
            names,
            vec![
                "test-lib",
                "test-xtask",
                "test-audit",
                "test-daisyui5",
                "test-svg-paint"
            ]
        );
    }

    /// Argument-level guard: `src/test_mode.rs` is behind the `test-mode`
    /// feature, so the two library steps must opt into it. Without the flag
    /// the module's unit tests do not run and clippy never lints it, while
    /// the gate still reports green.
    #[test]
    fn library_steps_enable_the_test_mode_feature() {
        for name in ["clippy-lib", "test-lib"] {
            let steps = gate_steps();
            let step = steps
                .iter()
                .find(|s| s.name == name)
                .unwrap_or_else(|| panic!("{name} is not a gate step"));
            let Run::Cmd { args, .. } = &step.run else {
                panic!("{name} must be a subprocess step");
            };
            let i = args
                .iter()
                .position(|a| a == "--features")
                .unwrap_or_else(|| panic!("{name} does not pass --features: {args:?}"));
            assert_eq!(
                args.get(i + 1).map(String::as_str),
                Some("test-mode"),
                "{name} must enable the test-mode feature: {args:?}"
            );
        }
    }

    // ---- sibling-token guard (ldui-ae5) --------------------------------

    /// The shape of the real preamble's imports, trimmed to the interesting
    /// cases: a braced multi-line list, a bare module, an aliased module, and
    /// a doc comment that names a token in prose.
    const PREAMBLE_SAMPLE: &str = r#"
use leptos::prelude::*;
use ui_tokens::elevation::{LEVEL_2, Shadow};
use ui_tokens::radius;
use ui_tokens::spacing::{
    SPACE_HUGE, SPACE_XS,
};
use ui_tokens::stroke;
use ui_tokens::typography as ty;

/// The line height comes from the `LINE_*` ramp, never a font metric.
pub const TYPE_STEPS: [(&str, f32); 1] = [("display", ty::LINE_DISPLAY)];
pub const STROKE_STEPS: [(&str, f32); 1] = [("accent", stroke::ACCENT)];
pub fn r() -> f32 { radius::CARD }
"#;

    fn tref(module: &str, symbol: Option<&str>) -> TokenRef {
        TokenRef {
            module: module.to_string(),
            symbol: symbol.map(|s| s.to_string()),
        }
    }

    #[test]
    fn braced_and_bare_and_aliased_imports_all_parse() {
        let refs = token_refs(PREAMBLE_SAMPLE);
        assert!(refs.contains(&tref("elevation", Some("LEVEL_2"))));
        assert!(refs.contains(&tref("elevation", Some("Shadow"))));
        assert!(refs.contains(&tref("spacing", Some("SPACE_HUGE"))));
        assert!(refs.contains(&tref("spacing", Some("SPACE_XS"))));
        assert!(refs.contains(&tref("stroke", None)));
        assert!(refs.contains(&tref("radius", None)));
        assert!(refs.contains(&tref("typography", None)));
    }

    /// The half a use-statement scan misses: reached only through an alias.
    #[test]
    fn alias_qualified_references_resolve_to_the_real_module() {
        let refs = token_refs(PREAMBLE_SAMPLE);
        assert!(refs.contains(&tref("typography", Some("LINE_DISPLAY"))));
        assert!(refs.contains(&tref("stroke", Some("ACCENT"))));
        assert!(refs.contains(&tref("radius", Some("CARD"))));
        // `ty` is an alias, never a module in its own right.
        assert!(!refs.iter().any(|r| r.module == "ty"));
    }

    /// Prose naming a token must not become a reference — otherwise the guard
    /// fails on documentation.
    #[test]
    fn tokens_named_in_comments_are_not_references() {
        let refs = token_refs("/// See `ty::LINE_NOPE` for detail.\nuse ui_tokens::radius;\n");
        assert!(
            !refs
                .iter()
                .any(|r| r.symbol.as_deref() == Some("LINE_NOPE"))
        );
    }

    /// Unrelated paths must not be mistaken for token references.
    #[test]
    fn non_ui_tokens_paths_are_ignored() {
        let refs = token_refs("use ui_tokens::radius;\nfn f() { crate::tokens::helper(); }\n");
        assert!(!refs.iter().any(|r| r.module == "crate"));
        assert_eq!(refs, BTreeSet::from([tref("radius", None)]));
    }

    #[test]
    fn declarations_are_read_from_lib_and_module_sources() {
        let mods = modules_declared("pub mod spacing;\npub mod stroke;\nmod private;\n");
        assert_eq!(
            mods,
            BTreeSet::from(["spacing".to_string(), "stroke".to_string()])
        );

        let items = defined_items(
            "pub const SCALE: [f32; 9] = [\npub struct Shadow {\npub enum Easing {\nconst HIDDEN: f32 = 1.0;\n",
        );
        assert!(items.contains("SCALE"));
        assert!(items.contains("Shadow"));
        assert!(items.contains("Easing"));
        assert!(
            !items.contains("HIDDEN"),
            "private items are not importable"
        );
    }

    #[test]
    fn a_fully_satisfied_tree_reports_nothing_missing() {
        let refs = BTreeSet::from([
            tref("spacing", Some("SPACE_XS")),
            tref("stroke", None),
            tref("typography", Some("LINE_DISPLAY")),
        ]);
        let defined = BTreeMap::from([
            (
                "spacing".to_string(),
                BTreeSet::from(["SPACE_XS".to_string()]),
            ),
            ("stroke".to_string(), BTreeSet::new()),
            (
                "typography".to_string(),
                BTreeSet::from(["LINE_DISPLAY".to_string()]),
            ),
        ]);
        assert!(missing_refs(&refs, &defined).is_empty());
    }

    /// The 2026-07-29 breakage, reproduced: the four items that existed only
    /// on the unmerged branch must all be reported against a default-branch
    /// tree that predates them.
    #[test]
    fn the_original_branch_only_items_are_all_caught() {
        let refs = BTreeSet::from([
            tref("spacing", Some("SPACE_HUGE")),
            tref("spacing", Some("SPACE_XXXL")),
            tref("spacing", Some("SPACE_XS")),
            tref("stroke", None),
            tref("typography", Some("LINE_DISPLAY")),
        ]);
        // ui-tokens as master had it: no stroke module, short spacing scale,
        // no line ramp.
        let defined = BTreeMap::from([
            (
                "spacing".to_string(),
                BTreeSet::from(["SPACE_XS".to_string()]),
            ),
            (
                "typography".to_string(),
                BTreeSet::from(["SIZE_BODY".to_string()]),
            ),
        ]);

        let missing = missing_refs(&refs, &defined);
        assert_eq!(missing.len(), 4, "got {missing:?}");
        assert!(missing.contains(&tref("spacing", Some("SPACE_HUGE"))));
        assert!(missing.contains(&tref("spacing", Some("SPACE_XXXL"))));
        assert!(missing.contains(&tref("typography", Some("LINE_DISPLAY"))));
        assert!(
            missing.contains(&tref("stroke", None)),
            "an absent module is itself missing"
        );
        // And the message names the item.
        assert_eq!(
            tref("spacing", Some("SPACE_HUGE")).to_string(),
            "ui_tokens::spacing::SPACE_HUGE"
        );
    }

    /// The guard is a gate step, and — unlike the browser suites — a cheap
    /// enough one to sit in the fast `verify`.
    #[test]
    fn sibling_token_guard_is_a_default_gate_step() {
        assert!(gate_steps().iter().any(|s| s.name == "sibling-tokens"));
    }

    /// The SVG paint guard is a source scan, so it belongs in the fast gate
    /// next to the daisyUI-4 class scan — and it must be its OWN step, because
    /// `test-lib` never runs an integration test (ldui-1g5).
    #[test]
    fn svg_paint_guard_is_a_default_gate_step() {
        assert!(gate_steps().iter().any(|s| s.name == "test-svg-paint"));
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
        assert_eq!(
            full_steps()
                .iter()
                .filter(|step| step.name == "test-reactivity")
                .count(),
            1,
            "the selectable lane belongs exactly once in the requested full gate"
        );
    }

    #[test]
    fn reactivity_step_is_in_process() {
        let s = reactivity_step();
        assert_eq!(s.name, "test-reactivity");
        assert!(matches!(
            s.run,
            Run::BrowserSuite {
                test: "reactivity_smoke",
                html_target: None
            }
        ));
    }

    #[test]
    fn reactivity_lane_has_exactly_34_browser_checks() {
        let source = include_str!("../../tests/reactivity_smoke.rs");
        assert_eq!(
            source.matches("#[tokio::test").count(),
            34,
            "the explicitly requested reactivity lane must keep its 34 checks"
        );
    }

    #[test]
    fn client_snapshot_step_is_targeted_and_in_process() {
        let step = client_snapshot_step();
        assert_eq!(step.name, "test-client-snapshot");
        assert!(matches!(
            step.run,
            Run::BrowserSuite {
                test: "entity_table_smoke",
                html_target: Some("client-snapshot-test-host.html")
            }
        ));
        assert!(
            !gate_steps()
                .iter()
                .any(|step| step.name == "test-client-snapshot")
        );
    }

    #[test]
    fn trunk_serve_args_pin_color_without_the_legacy_no_color_flag() {
        let args = trunk_serve_args(4321, None);
        assert!(args.windows(2).any(|pair| pair == ["--color", "never"]));
        assert!(!args.iter().any(|arg| arg.starts_with("--no-color")));
    }

    #[test]
    fn snapshot_table_step_reuses_the_page_scoped_host() {
        let step = snapshot_table_step();
        assert_eq!(step.name, "test-snapshot-table");
        assert!(matches!(
            step.run,
            Run::BrowserSuite {
                test: "snapshot_table_smoke",
                html_target: Some("client-snapshot-test-host.html")
            }
        ));
        assert!(
            full_steps()
                .iter()
                .any(|step| step.name == "test-snapshot-table")
        );
    }

    #[test]
    fn result_list_step_is_in_process_and_full_only() {
        let step = result_list_step();
        assert_eq!(step.name, "test-keyed-result-list");
        assert!(matches!(
            step.run,
            Run::BrowserSuite {
                test: "keyed_result_list_smoke",
                html_target: None
            }
        ));
        assert!(
            !gate_steps()
                .iter()
                .any(|s| s.name == "test-keyed-result-list")
        );
        assert!(
            full_steps()
                .iter()
                .any(|s| s.name == "test-keyed-result-list")
        );
    }

    #[test]
    fn powershell_visual_runner_sanitizes_no_color_for_trunk() {
        let source = include_str!("../../scripts/test-visual.ps1");
        assert!(
            source.contains("Remove-Item Env:NO_COLOR"),
            "Trunk 0.21 rejects the conventional NO_COLOR=1 value"
        );
        assert!(
            source.contains("$env:NO_COLOR = $savedNoColor"),
            "the wrapper must restore its process environment after launch"
        );
        assert!(
            source.contains("node build-css.mjs"),
            "the wrapper needs a fresh pre-start stamp that cannot match stale dist assets"
        );
        assert!(
            source.contains("Test-StylesheetCurrent"),
            "HTTP readiness alone can expose Trunk's previous distribution"
        );
        assert!(
            source.contains("--release=true"),
            "the visual wrapper must use the Windows-proven release WASM path"
        );
    }

    #[test]
    fn client_snapshot_trunk_args_select_the_page_scoped_host() {
        let args = trunk_serve_args(4321, Some("client-snapshot-test-host.html"));
        assert_eq!(
            args.last().map(String::as_str),
            Some("client-snapshot-test-host.html")
        );
    }

    #[test]
    fn browser_gates_do_not_reuse_unverified_external_servers() {
        let source = include_str!("main.rs");
        let run_steps = source
            .split_once("fn run_steps(steps: &[Step]) -> ExitCode")
            .expect("run_steps function")
            .1
            .split_once("\nfn main()")
            .expect("main follows run_steps")
            .0;
        assert!(
            !run_steps.contains(concat!("VISUAL_TEST_BASE", "_URL")),
            "verification gates must always launch and validate their own target-specific server"
        );
    }

    #[test]
    fn every_demo_html_entry_selects_exactly_one_wasm_binary() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask has a repository parent");
        let catalog = std::fs::read_to_string(repo.join("demo/index.html")).expect("catalog HTML");
        assert!(html_selects_exactly_one_wasm_binary(
            &catalog,
            "leptos-daisyui-showcase"
        ));

        let page_host = std::fs::read_to_string(repo.join("demo/client-snapshot-test-host.html"))
            .expect("page-host HTML");
        assert!(html_selects_exactly_one_wasm_binary(
            &page_host,
            "client-snapshot-test-host"
        ));

        let ambiguous = r#"
            <link data-trunk rel="rust" data-bin="client-snapshot-test-host" />
            <link data-trunk rel="rust" data-bin="leptos-daisyui-showcase" />
        "#;
        assert!(!html_selects_exactly_one_wasm_binary(
            ambiguous,
            "client-snapshot-test-host"
        ));
    }

    #[test]
    fn catalog_browser_suites_use_the_reliable_release_bundle() {
        let catalog = trunk_serve_args(4321, None);
        assert!(catalog.iter().any(|arg| arg == "--release=true"));

        let page_scoped = trunk_serve_args(4321, Some("client-snapshot-test-host.html"));
        assert!(!page_scoped.iter().any(|arg| arg.starts_with("--release")));
    }

    #[test]
    fn browser_readiness_rejects_a_stale_stylesheet() {
        let html = r#"<script src="/leptos-daisyui-showcase-current.js"></script>
            <link rel="stylesheet" href="/output-current.css">"#;
        let stale_css = "#ldui-css-stamp-previous { --ldui-css-stamp: 1 }";

        assert!(!browser_assets_match(html, stale_css, "ok current", None));
    }

    #[test]
    fn browser_readiness_rejects_the_previous_html_target() {
        let html = r#"<script src="/client-snapshot-test-host-current.js"></script>
            <link rel="stylesheet" href="/output-current.css">"#;
        let css = "#ldui-css-stamp-current { --ldui-css-stamp: 1 }";

        assert!(!browser_assets_match(html, css, "ok current", None));
        assert!(browser_assets_match(
            html,
            css,
            "ok current",
            Some("client-snapshot-test-host.html")
        ));
    }

    #[test]
    fn browser_readiness_requires_a_successful_current_stamp() {
        let html = r#"<script src="/leptos-daisyui-showcase-current.js"></script>
            <link rel="stylesheet" href="/output-current.css">"#;
        let css = "#ldui-css-stamp-current { --ldui-css-stamp: 1 }";

        assert!(browser_assets_match(html, css, "ok current\n", None));
        assert!(!browser_assets_match(html, css, "fail current\n", None));
        assert!(!browser_assets_match(html, css, "ok different\n", None));
    }

    #[test]
    fn browser_readiness_extracts_the_hashed_stylesheet_path() {
        let html = r#"<link rel="stylesheet" href="/output-a1b2c3.css" integrity="x">"#;
        assert_eq!(output_stylesheet_path(html), Some("/output-a1b2c3.css"));
    }

    #[test]
    fn layout_step_is_in_process() {
        let s = layout_step();
        assert_eq!(s.name, "test-layout");
        assert!(matches!(
            s.run,
            Run::BrowserSuite {
                test: "layout_audit_smoke",
                html_target: None
            }
        ));
    }

    #[test]
    fn style_step_is_in_process() {
        let s = style_step();
        assert_eq!(s.name, "test-style");
        assert!(matches!(
            s.run,
            Run::BrowserSuite {
                test: "style_audit_smoke",
                html_target: None
            }
        ));
    }

    #[test]
    fn browser_suites_are_not_in_the_fast_gate() {
        // All three need npm/trunk/Chrome and a wasm build; `verify` is
        // deliberately fast and zero-tooling.
        let names: Vec<&str> = gate_steps().iter().map(|s| s.name).collect();
        assert!(!names.contains(&"test-reactivity"));
        assert!(!names.contains(&"test-layout"));
        assert!(!names.contains(&"test-style"));
    }

    #[test]
    fn verify_full_browser_steps_need_only_two_server_builds() {
        let steps = full_steps();
        assert_eq!(browser_server_segment_count(&steps), 2);
    }

    #[test]
    fn verify_full_catalog_server_is_the_release_build() {
        let steps = full_steps();
        assert!(!steps.iter().any(|step| step.name == "trunk-build"));
        assert!(steps.iter().any(|step| {
            matches!(
                step.run,
                Run::BrowserSuite {
                    html_target: None,
                    ..
                }
            )
        }));
    }

    #[test]
    fn test_subcommand_does_not_pick_up_the_browser_suites() {
        // `steps_for("test")` filters on the `test` prefix, which
        // `test-reactivity`, `test-layout` and `test-style` also match by
        // name — but they are not gate steps, so the filter must not surface
        // them.
        let names: Vec<&str> = steps_for("test").iter().map(|s| s.name).collect();
        assert_eq!(
            names,
            vec![
                "test-lib",
                "test-xtask",
                "test-audit",
                "test-daisyui5",
                "test-svg-paint"
            ]
        );
    }

    /// A port the OS hands out must actually be bindable.
    #[test]
    fn free_port_is_bindable() {
        let p = free_port().expect("free port");
        assert!(p > 0);
        assert!(browser_allows_port(p));
        std::net::TcpListener::bind(("127.0.0.1", p)).expect("port should be free");
    }

    #[test]
    fn chromium_restricted_ports_are_not_browser_safe() {
        for port in [0, 21, 554, 6000, 6566, 6667, 10080] {
            assert!(!browser_allows_port(port), "port {port}");
        }
        for port in [3010, 4321, 49152, u16::MAX] {
            assert!(browser_allows_port(port), "port {port}");
        }
    }

    /// Nothing is listening on a just-released port, so the probe says "not up".
    #[test]
    fn http_get_fails_when_nothing_listens() {
        let p = free_port().expect("free port");
        assert!(http_get(p, "/").is_err());
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

#[cfg(test)]
mod gen_tokens_tests {
    use super::*;

    #[test]
    fn rem_converts_dips_against_a_16px_root() {
        assert_eq!(rem(4.0), "0.25rem");
        assert_eq!(rem(16.0), "1rem");
        assert_eq!(rem(96.0), "6rem");
        assert_eq!(rem(11.0), "0.6875rem");
    }

    #[test]
    fn px_drops_the_trailing_zero() {
        assert_eq!(px(1.0), "1px");
        assert_eq!(px(3.0), "3px");
    }

    #[test]
    fn line_ratio_is_exact_and_unitless() {
        // Unitless so it inherits as a ratio; exact so it matches what
        // Tailwind ships to the last decimal.
        assert_eq!(line_ratio(20.0, 14.0), "calc(20 / 14)");
        assert_eq!(line_ratio(16.0, 12.0), "calc(16 / 12)");
        // Equal size and leading collapses to a plain 1 rather than
        // `calc(16 / 16)`.
        assert_eq!(line_ratio(16.0, 16.0), "1");
    }

    #[test]
    fn base_spacing_unit_derives_from_the_token() {
        let css = tokens_css();
        let want = format!("  --spacing: {};\n", rem(ui_tokens::spacing::SPACE_XXS));
        assert!(
            css.contains(&want),
            "missing token-derived --spacing:\n{css}"
        );
    }

    #[test]
    fn opinionated_table_colors_pin_the_shared_semantic_palette() {
        let css = tokens_css();
        for declaration in [
            "  --color-table-header: #004578;",
            "  --color-table-header-content: #ffffff;",
            "  --color-table-filter: #e5f1fb;",
            "  --color-table-filter-content: #1a1a1a;",
            "  --color-table-grid: #e0e0e0;",
        ] {
            assert!(
                css.to_ascii_lowercase().contains(declaration),
                "missing semantic table declaration {declaration}:\n{css}"
            );
        }
    }

    #[test]
    fn opinionated_table_colors_are_sourced_from_ui_tokens_color_table() {
        // ldui-gp34: the five --color-table-* values must be *derived* from
        // ui_tokens::color::table, not hand-copied literals that happen to
        // agree today. Computing the expected declarations from the live
        // constants means a sibling change to the `table` module is caught
        // here (and by `tokens-fresh`) instead of silently drifting.
        use ui_tokens::color::{table, to_css_hex};

        let css = tokens_css();
        for (name, value) in [
            ("header", table::HEADER),
            ("header-content", table::HEADER_CONTENT),
            ("filter", table::FILTER),
            ("filter-content", table::FILTER_CONTENT),
            ("grid", table::GRID),
        ] {
            let want = format!("  --color-table-{name}: {};\n", to_css_hex(value));
            assert!(
                css.contains(&want),
                "expected {want:?} derived from ui_tokens::color::table::{} to appear in generated css:\n{css}",
                name.to_ascii_uppercase().replace('-', "_")
            );
        }
    }

    #[test]
    fn generated_css_never_leaks_comment_text_outside_a_comment() {
        // Regression guard (ldui-gp34 fix-forward): a stray `*/` INSIDE a
        // comment body (e.g. an alias name written as `STATUS_BLUE_*/`)
        // closes that comment early. Everything after it, up to the next
        // `/*`, then reads as bare prose inside the `@theme` block —
        // Tailwind v4 rejects that with "`@theme` blocks must only contain
        // custom properties or `@keyframes`", failing every browser lane
        // that builds the demo stylesheet.
        //
        // Walk the generated css the way a CSS tokenizer would, splitting it
        // into comment / non-comment segments on `/*` and `*/`, and assert
        // every non-comment segment is only ever blank lines, the `@theme`
        // block braces, or a `--custom-property: value;` declaration.
        let css = tokens_css();

        fn assert_segment_is_only_declarations_or_structure(segment: &str, css: &str) {
            for line in segment.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed == "@theme {" || trimmed == "}" {
                    continue;
                }
                assert!(
                    trimmed.starts_with("--") && trimmed.ends_with(';'),
                    "non-comment content in generated css is not a custom-property \
                     declaration, which Tailwind's @theme grammar rejects \
                     (a comment likely closed early via a stray `*/`): \
                     {trimmed:?}\nfull css:\n{css}"
                );
            }
        }

        let mut rest = css.as_str();
        let mut in_comment = false;
        loop {
            let marker = if in_comment { "*/" } else { "/*" };
            match rest.find(marker) {
                Some(i) => {
                    if !in_comment {
                        assert_segment_is_only_declarations_or_structure(&rest[..i], &css);
                    }
                    rest = &rest[i + marker.len()..];
                    in_comment = !in_comment;
                }
                None => {
                    assert!(
                        !in_comment,
                        "generated css ends with an unterminated /* comment:\n{css}"
                    );
                    assert_segment_is_only_declarations_or_structure(rest, &css);
                    break;
                }
            }
        }
    }

    #[test]
    fn no_named_spacing_keys_are_emitted() {
        // Regression guard. Tailwind resolves `max-w-*` against --spacing-*
        // before --container-*, so emitting `--spacing-xs` silently redefines
        // `max-w-xs` from 20rem to 0.5rem — a 40x shrink that compiles
        // cleanly and is invisible until someone looks at the page.
        // Scan declarations only — the generated file *talks about*
        // `--spacing-xs` in the comment explaining why it is absent.
        let css = tokens_css();
        let offenders: Vec<&str> = css
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("--spacing-") && l.contains(':'))
            .collect();
        assert!(
            offenders.is_empty(),
            "named spacing keys collide with Tailwind's container scale: {offenders:?}"
        );
    }

    #[test]
    fn every_type_ramp_step_emits_size_and_line_height() {
        let css = tokens_css();
        for name in ["display", "title", "subtitle", "body", "caption", "small"] {
            assert!(
                css.contains(&format!("  --text-{name}: ")),
                "missing size for {name}"
            );
            assert!(
                css.contains(&format!("  --text-{name}--line-height: ")),
                "missing line height for {name}"
            );
        }
    }

    #[test]
    fn tailwind_size_keys_resolve_to_their_shipped_values() {
        // The re-pointed keys must be behaviour-preserving. These are the
        // values Tailwind v4 ships; if a token moves, this test is where the
        // visual change surfaces instead of in the browser.
        let css = tokens_css();
        for (key, size, ratio) in [
            ("xs", "0.75rem", "calc(16 / 12)"),
            ("sm", "0.875rem", "calc(20 / 14)"),
            ("base", "1rem", "calc(24 / 16)"),
            ("xl", "1.25rem", "calc(28 / 20)"),
        ] {
            assert!(
                css.contains(format!("  --text-{key}: {size}\n").trim_end())
                    || css.contains(&format!("  --text-{key}: {size};")),
                "text-{key} is not {size}"
            );
            assert!(
                css.contains(&format!("  --text-{key}--line-height: {ratio};")),
                "text-{key} line height is not {ratio}"
            );
        }
    }

    #[test]
    fn stroke_widths_stay_in_px() {
        // A hairline that scales with the user's font size is a bug, not an
        // accessibility feature.
        let css = tokens_css();
        assert!(css.contains("  --border-width-hairline: 1px;"), "{css}");
        assert!(css.contains("  --border-width-thin: 2px;"), "{css}");
    }

    #[test]
    fn drift_check_ignores_line_endings() {
        // Regression: the generator writes LF, git checks the file out as
        // CRLF under core.autocrlf=true, and a raw byte comparison then
        // reported STALE on every fresh checkout while `git diff` was empty.
        // Caught only by running the gate on a freshly merged tree.
        let lf = "a\nb\nc\n";
        let crlf = "a\r\nb\r\nc\r\n";
        assert!(same_ignoring_line_endings(lf, crlf));
        assert!(same_ignoring_line_endings(lf, lf));
        // ...but a real content change is still a difference.
        assert!(!same_ignoring_line_endings(lf, "a\r\nb\r\nd\r\n"));
        assert!(!same_ignoring_line_endings(lf, "a\nb\n"));
    }

    #[test]
    fn drift_check_accepts_the_real_generated_css_as_crlf() {
        // The same property against the actual payload, not a toy string.
        let want = tokens_css();
        let crlf = want.replace('\n', "\r\n");
        assert!(same_ignoring_line_endings(&crlf, &want));
    }

    #[test]
    fn generated_css_is_a_single_theme_block() {
        let css = tokens_css();
        assert_eq!(css.matches("@theme {").count(), 1);
        assert!(css.trim_end().ends_with('}'));
    }
}
