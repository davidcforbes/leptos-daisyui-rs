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
    /// Spawn the demo dev server on a free port, run a browser-driven test
    /// binary against it, then tear the server down. See [`run_browser_suite`].
    BrowserSuite(&'static str),
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
        run: Run::BrowserSuite("reactivity_smoke"),
    }
}

/// The layout-audit step (ldui-dg2): overlap / grid / internal-vs-external
/// assertions swept over the rendered DOM. Same tooling requirements as the
/// reactivity suite, so it lives alongside it in `verify-full` rather than in
/// the fast `verify` gate.
fn layout_step() -> Step {
    Step {
        name: "test-layout",
        run: Run::BrowserSuite("layout_audit_smoke"),
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
        Run::BrowserSuite(test) => run_browser_suite(test),
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
fn run_browser_suite(test: &str) -> bool {
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
            test,
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
    use ui_tokens::{radius, spacing, stroke, typography as ty};

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
            Some(current) if current == want => {
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

    if have.as_deref() == Some(want.as_str()) {
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
            // NOTE: layout_step() is deliberately NOT in verify-full yet.
            // The sweep logic is verified (see doc/plans/2026-07-26-spacing-
            // audit.md), but the Rust->CDP call that ships it into the page
            // does not yet return — wiring it here would hang the gate.
            // Run it explicitly with `cargo xtask test-layout`. Tracked by
            // ldui-mai.5.
            steps.push(cmd(
                "trunk-build",
                "trunk",
                &["build", "--release"],
                Some("demo"),
            ));
            run_steps(&steps)
        }
        "test-reactivity" => run_steps(&[reactivity_step()]),
        "test-layout" => run_steps(&[layout_step()]),
        "gen-tokens" => {
            let check = std::env::args().any(|a| a == "--check");
            gen_tokens(check)
        }
        "bump" => {
            let level = std::env::args().nth(2).unwrap_or_default();
            let dry = std::env::args().any(|a| a == "--dry-run");
            bump(&level, dry)
        }
        other => {
            eprintln!("xtask: unknown subcommand {other:?}");
            eprintln!(
                "usage: cargo xtask <verify|verify-full|fmt-check|clippy|build|check-demo|test|test-reactivity|test-layout|gen-tokens|bump>"
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
        assert_eq!(names, vec!["test-lib", "test-xtask", "test-daisyui5"]);
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
        assert!(matches!(s.run, Run::BrowserSuite("reactivity_smoke")));
    }

    #[test]
    fn layout_step_is_in_process() {
        let s = layout_step();
        assert_eq!(s.name, "test-layout");
        assert!(matches!(s.run, Run::BrowserSuite("layout_audit_smoke")));
    }

    #[test]
    fn browser_suites_are_not_in_the_fast_gate() {
        // Both need npm/trunk/Chrome and a wasm build; `verify` is
        // deliberately fast and zero-tooling.
        let names: Vec<&str> = gate_steps().iter().map(|s| s.name).collect();
        assert!(!names.contains(&"test-reactivity"));
        assert!(!names.contains(&"test-layout"));
    }

    #[test]
    fn test_subcommand_does_not_pick_up_the_browser_suites() {
        // `steps_for("test")` filters on the `test` prefix, which
        // `test-reactivity` and `test-layout` also match by name — but they
        // are not gate steps, so the filter must not surface them.
        let names: Vec<&str> = steps_for("test").iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["test-lib", "test-xtask", "test-daisyui5"]);
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
                css.contains(&format!("  --text-{key}: {size}\n").trim_end())
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
    fn generated_css_is_a_single_theme_block() {
        let css = tokens_css();
        assert_eq!(css.matches("@theme {").count(), 1);
        assert!(css.trim_end().ends_with('}'));
    }
}
