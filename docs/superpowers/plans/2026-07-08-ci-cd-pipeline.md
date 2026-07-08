# Local-only CI/CD Pipeline (xtask + cargo-make) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give leptos-daisyui-rs a local-only, two-layer CI/CD gate — a zero-dependency `xtask` Rust binary holding the check logic, orchestrated by thin cargo-make tasks — per the approved design in `doc/ci-cd.md`.

**Architecture:** Convert the single crate into a 3-member Cargo workspace (`.`, `xtask`, `demo`). All check *logic* lives in `xtask/src/main.rs` (advisory-first runner + pure, unit-tested SemVer bump). `Makefile.toml` tasks and `launcher.ps1` become thin pass-throughs to `cargo xtask <sub>`. An opt-in advisory `.githooks/pre-push` runs the gate.

**Tech Stack:** Rust (edition 2024, cargo 1.95, std-only xtask), cargo-make, trunk (demo wasm build), Git Bash hooks, PowerShell launcher. Windows-native (`C:\dev\leptos-daisyui-rs`).

## Global Constraints

- The `xtask` crate has **zero external dependencies** — std only. (Reference doc: "zero-dependency Rust binary".)
- `verify` is **advisory-first**: run *every* step even after a failure; print a PASS/FAIL summary; process exit code = number of failed steps (0 = all green).
- Workspace members are exactly `[".", "xtask", "demo"]`, `resolver = "3"` (preserves the edition-2024 feature resolution; a workspace otherwise regresses toward resolver 1). The six sibling path-deps stay dependencies, never members.
- `build` step is **library only** (`-p leptos-daisyui-rs`); the CSR demo is *checked* (`cargo check -p leptos-daisyui-showcase`), not natively built. The real wasm build is `trunk`, only in `verify-full`.
- `bump` rewrites **only** the `[package] version` of the `leptos-daisyui-rs` library; never a dependency version, never the demo or xtask version. Level is a human argument, never inferred.
- Commit messages end with the trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- Do not push. Do not run `bd` commands (the controller manages beads).
- The demo crate's package name is `leptos-daisyui-showcase`; the lib is `leptos-daisyui-rs`.

## File Structure

| File | Responsibility | Action |
|---|---|---|
| `Cargo.toml` (root) | add `[workspace]` block (members/resolver) | Modify |
| `.cargo/config.toml` | the `xtask = "run --package xtask --"` alias | Create |
| `xtask/Cargo.toml` | xtask crate manifest (zero deps, publish=false) | Create |
| `xtask/src/main.rs` | CLI dispatch, advisory runner, step definitions, pure `bump_version`/`set_package_version` + their `#[cfg(test)]` tests | Create |
| `Makefile.toml` | rewire `verify`/`verify-full`/`fmt-check`/`clippy`/`build`/`check-demo`/`test`/`bump-*` to `cargo xtask` pass-throughs | Modify |
| `launcher.ps1` | point the `check`/`fix`/`verify` menu tasks at the new flow | Modify |
| `.githooks/pre-push` | advisory `cargo xtask verify` hook | Create |
| `doc/ci-cd.md` | correct the testing-policy section to reference the real PixelProof `test-visual` suite | Modify |

**Not in scope (already correct):** `.git/hooks/pre-commit` already guards `bd sync` with `if bd sync --help >/dev/null 2>&1` — the bd-v1 bug is already patched here; leave it.

---

### Task 1: Convert to a workspace and verify the gate commands empirically

This task de-risks everything: it proves which exact cargo invocations work once `demo` is a workspace member (feature unification can change native builds), so later tasks encode verified commands rather than guesses.

**Files:**
- Modify: `Cargo.toml` (root) — add `[workspace]` after the `[package]`/`publish` lines, before `[dependencies]`.

**Interfaces:**
- Produces: a working 3-→(here 2-)member workspace and a written record (in the commit body / this task's notes) of the exact verified commands for `fmt-check`, `clippy`, `build`, `check-demo`, `test`, and `trunk build`. Task 3 consumes those exact command strings.

- [ ] **Step 1: Add the workspace block to `Cargo.toml`**

Insert immediately after the `publish = false` line (before `[dependencies]`). Note: `xtask` is intentionally **omitted** here — it doesn't exist yet; it's added in Task 2. Cargo requires every listed member to exist.

```toml
[workspace]
members = [".", "demo"]
resolver = "3"
```

- [ ] **Step 2: Confirm the workspace resolves**

Run: `cargo metadata --no-deps --format-version 1 > /dev/null`
Expected: exits 0, no error. If cargo complains the demo package "was not specified in members/exclude", the block above already lists it — re-check the path. If it errors on `resolver = "3"`, the toolchain is <1.84; fall back to `resolver = "2"` and note it.

- [ ] **Step 3: Verify each intended gate command works and record the winner**

Run each; record PASS/FAIL and the exact command that passed. These become the xtask steps in Task 3.

```bash
cargo fmt --all -- --check          # fmt-check
cargo clippy --workspace --all-targets -- -D warnings   # clippy
cargo build -p leptos-daisyui-rs    # build (lib only)
cargo check -p leptos-daisyui-showcase                  # check-demo
cargo test -p leptos-daisyui-rs --lib                   # test (the ~1766 lib tests)
```

Expected: all PASS (the lib + demo already compiled this way pre-workspace). **If `cargo clippy --workspace` fails** because the demo's `leptos csr` feature unifies onto the lib for a native build, fall back to running clippy per-crate and record THAT as the clippy command:

```bash
cargo clippy -p leptos-daisyui-rs --all-targets -- -D warnings
cargo clippy -p leptos-daisyui-showcase --all-targets -- -D warnings
```

- [ ] **Step 4: Verify the demo still builds for real via trunk**

Run: `cd demo && trunk build --release && cd ..`
Expected: builds to `demo/dist` with no error (the pre_build tailwind hook runs). If trunk isn't installed, note "trunk build unverified — install trunk" and continue; `verify-full` will surface it later. This confirms workspace membership didn't break the wasm build.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "$(printf 'build: convert to a 3-crate workspace for the xtask gate\n\nAdd [workspace] members=[\".\", \"demo\"] resolver=\"3\" (xtask added in the\nnext task). Verified fmt --all / clippy --workspace / build -p lib /\ncheck -p demo / test -p lib --lib and trunk build all still pass.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 2: Scaffold the xtask crate, the alias, and wire it into the workspace

**Files:**
- Create: `xtask/Cargo.toml`
- Create: `xtask/src/main.rs` (minimal dispatch stub)
- Create: `.cargo/config.toml`
- Modify: `Cargo.toml` (root) — add `"xtask"` to `members`.

**Interfaces:**
- Produces: `cargo xtask <sub>` resolves and runs; `main()` returns `std::process::ExitCode`; an `enum`-free string-dispatch on the first CLI arg. Task 3 replaces the stub bodies.

- [ ] **Step 1: Create `xtask/Cargo.toml`**

```toml
[package]
name = "xtask"
version = "0.0.0"
edition = "2024"
publish = false

# Zero external dependencies — std only (design constraint).
[dependencies]
```

- [ ] **Step 2: Create the minimal `xtask/src/main.rs`**

```rust
//! leptos-daisyui-rs local CI/CD logic. See `doc/ci-cd.md`.
//! Run via the `cargo xtask <sub>` alias (`.cargo/config.toml`).

use std::process::ExitCode;

fn main() -> ExitCode {
    let sub = std::env::args().nth(1).unwrap_or_default();
    match sub.as_str() {
        "verify" | "verify-full" | "fmt-check" | "clippy" | "build" | "check-demo"
        | "test" | "bump" => {
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
```

- [ ] **Step 3: Create `.cargo/config.toml` with the alias**

```toml
[alias]
xtask = "run --package xtask --"
```

- [ ] **Step 4: Add `xtask` to the workspace members**

Edit root `Cargo.toml` `[workspace]` so members is:

```toml
members = [".", "xtask", "demo"]
```

- [ ] **Step 5: Verify the alias runs**

Run: `cargo xtask verify`
Expected: prints `xtask: 'verify' not implemented yet` and exits 1.

Run: `cargo xtask bogus`
Expected: prints the usage line and exits 2.

- [ ] **Step 6: Commit**

```bash
git add xtask/Cargo.toml xtask/src/main.rs .cargo/config.toml Cargo.toml Cargo.lock
git commit -m "$(printf 'build(xtask): scaffold the xtask crate + cargo xtask alias\n\nZero-dep std-only binary, added as the third workspace member. Stub\ndispatch; step logic lands next.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 3: The advisory-first runner and the five gate steps

**Files:**
- Modify: `xtask/src/main.rs`

**Interfaces:**
- Consumes: the verified command strings from Task 1.
- Produces:
  - `struct Step { name: &'static str, program: &'static str, args: Vec<String>, cwd: Option<&'static str> }`
  - `fn gate_steps() -> Vec<Step>` — the five steps in order.
  - `fn run_step(step: &Step) -> bool` — runs it, streams output, returns success.
  - `fn summarize(results: &[(&str, bool)]) -> (String, u8)` — **pure**; returns the summary text and the exit code (count of failures, saturating at 255). Task 5's tests and Task 4 reuse it.
  - `fn verify(extra: &[Step]) -> ExitCode` — runs `gate_steps()` + `extra`, prints the summary, returns its code.

- [ ] **Step 1: Write the failing test for the pure `summarize`**

Add to a `#[cfg(test)] mod tests` at the bottom of `xtask/src/main.rs`:

```rust
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
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p xtask summarize`
Expected: FAIL — `summarize` not found.

- [ ] **Step 3: Implement `summarize` (and the Step types) minimally**

Add above `main`:

```rust
use std::process::Command;

struct Step {
    name: &'static str,
    program: &'static str,
    args: Vec<String>,
    cwd: Option<&'static str>,
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// Pure: render the PASS/FAIL summary and compute the exit code
/// (= number of failed steps, saturating at 255).
fn summarize(results: &[(&str, bool)]) -> (String, u8) {
    let mut out = String::from("\n===== xtask verify summary =====\n");
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
```

- [ ] **Step 4: Run the tests and watch them pass**

Run: `cargo test -p xtask summarize`
Expected: PASS (2 tests).

- [ ] **Step 5: Implement `run_step`, `gate_steps`, `verify`, and wire dispatch**

Use the command strings verified in Task 1. (If Task 1 recorded per-crate clippy as the winner, encode two clippy steps instead of the one `--workspace` step shown.)

```rust
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

fn gate_steps() -> Vec<Step> {
    vec![
        Step { name: "fmt-check", program: "cargo",
            args: args(&["fmt", "--all", "--", "--check"]), cwd: None },
        Step { name: "clippy", program: "cargo",
            args: args(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]), cwd: None },
        Step { name: "build", program: "cargo",
            args: args(&["build", "-p", "leptos-daisyui-rs"]), cwd: None },
        Step { name: "check-demo", program: "cargo",
            args: args(&["check", "-p", "leptos-daisyui-showcase"]), cwd: None },
        Step { name: "test-lib", program: "cargo",
            args: args(&["test", "-p", "leptos-daisyui-rs", "--lib"]), cwd: None },
        Step { name: "test-xtask", program: "cargo",
            args: args(&["test", "-p", "xtask"]), cwd: None },
    ]
}

fn verify(extra: Vec<Step>) -> ExitCode {
    let mut steps = gate_steps();
    steps.extend(extra);
    // Advisory: run EVERY step, even after a failure.
    let results: Vec<(&str, bool)> = steps.iter().map(|s| (s.name, run_step(s))).collect();
    let (summary, code) = summarize(&results);
    println!("{summary}");
    ExitCode::from(code)
}

/// Run one named single step (for `cargo xtask fmt-check` etc.).
fn run_named(name: &str) -> ExitCode {
    match gate_steps().into_iter().find(|s| s.name == name || (name == "test" && s.name == "test-lib")) {
        Some(step) => ExitCode::from(if run_step(&step) { 0 } else { 1 }),
        None => { eprintln!("xtask: no step named {name:?}"); ExitCode::from(2) }
    }
}
```

Replace the `match sub.as_str()` arm bodies in `main`:

```rust
    match sub.as_str() {
        "verify" => verify(Vec::new()),
        "fmt-check" | "clippy" | "build" | "check-demo" => run_named(&sub),
        "test" => {
            // both the lib suite and the xtask's own tests
            let lib = run_step(&Step { name: "test-lib", program: "cargo",
                args: args(&["test", "-p", "leptos-daisyui-rs", "--lib"]), cwd: None });
            let xt = run_step(&Step { name: "test-xtask", program: "cargo",
                args: args(&["test", "-p", "xtask"]), cwd: None });
            ExitCode::from(if lib && xt { 0 } else { 1 })
        }
        "verify-full" | "bump" => { eprintln!("xtask: '{sub}' not implemented yet"); ExitCode::from(1) }
        other => { /* keep the existing usage/unknown arm */
            eprintln!("xtask: unknown subcommand {other:?}");
            eprintln!("usage: cargo xtask <verify|verify-full|fmt-check|clippy|build|check-demo|test|bump>");
            ExitCode::from(2)
        }
    }
```

- [ ] **Step 6: Run the real gate end-to-end**

Run: `cargo xtask verify`
Expected: runs all six steps, prints the summary, exits 0 (all green — the repo is green as of this plan). Confirm the summary lists each step and `6/6 passed`.

- [ ] **Step 7: Commit**

```bash
git add xtask/src/main.rs
git commit -m "$(printf 'feat(xtask): advisory-first verify gate + individual steps\n\nRuns fmt-check/clippy/build/check-demo/test-lib/test-xtask, always runs\nevery step, prints a PASS/FAIL summary, exits with the failure count.\nPure summarize() is unit-tested.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 4: `verify-full` — add the real trunk wasm build

**Files:**
- Modify: `xtask/src/main.rs`

**Interfaces:**
- Consumes: `verify()`, `Step`, `args()` from Task 3.
- Produces: a `verify-full` subcommand = the gate steps + a `trunk build --release` step run in `demo/`.

- [ ] **Step 1: Add the trunk step and wire the subcommand**

Replace the `"verify-full"` arm:

```rust
        "verify-full" => verify(vec![Step {
            name: "trunk-build",
            program: "trunk",
            args: args(&["build", "--release"]),
            cwd: Some("demo"),
        }]),
```

- [ ] **Step 2: Run it**

Run: `cargo xtask verify-full`
Expected: the six gate steps PASS, then `trunk-build` runs the wasm+tailwind build; summary shows `7/7 passed` and exit 0. If `trunk` is not installed it shows `FAIL trunk-build` ("failed to launch trunk") and exit 1 — that is correct advisory behavior; note the tooling requirement.

- [ ] **Step 3: Commit**

```bash
git add xtask/src/main.rs
git commit -m "$(printf 'feat(xtask): verify-full adds the trunk wasm build\n\nverify-full = verify + trunk build --release in demo/. Kept separate so\nthe default gate stays fast and zero-tooling.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 5: `bump` — pure SemVer + section-aware Cargo.toml rewrite

**Files:**
- Modify: `xtask/src/main.rs`

**Interfaces:**
- Produces:
  - `fn bump_version(current: &str, level: &str) -> Result<String, String>` — pure SemVer arithmetic; `level` ∈ `{"patch","minor","major"}`.
  - `fn set_package_version(cargo_toml: &str, new_version: &str) -> Result<String, String>` — pure; rewrites the `version = "..."` line **inside the first `[package]` table only**, leaving dependency versions untouched.
  - a `bump <patch|minor|major> [--dry-run]` subcommand operating on the root `Cargo.toml`.

- [ ] **Step 1: Write the failing tests (pure logic)**

Add to `mod tests`:

```rust
    #[test]
    fn bump_version_arithmetic() {
        assert_eq!(bump_version("0.0.4", "patch").unwrap(), "0.0.5");
        assert_eq!(bump_version("0.0.4", "minor").unwrap(), "0.1.0");
        assert_eq!(bump_version("1.2.3", "major").unwrap(), "2.0.0");
        assert!(bump_version("0.0.4", "sideways").is_err());
        assert!(bump_version("not.a.version", "patch").is_err());
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
```

- [ ] **Step 2: Run and watch fail**

Run: `cargo test -p xtask bump_version set_package_version`
Expected: FAIL — functions not found.

- [ ] **Step 3: Implement the pure functions**

```rust
fn bump_version(current: &str, level: &str) -> Result<String, String> {
    let mut parts = current.split('.');
    let mut nums = [0u64; 3];
    for slot in nums.iter_mut() {
        let p = parts.next().ok_or_else(|| format!("not MAJOR.MINOR.PATCH: {current:?}"))?;
        *slot = p.parse().map_err(|_| format!("non-numeric version segment: {p:?}"))?;
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

/// Rewrite the `version = "..."` line inside the first `[package]` table only.
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
```

- [ ] **Step 4: Run and watch pass**

Run: `cargo test -p xtask`
Expected: PASS — all summarize/bump/set tests green.

- [ ] **Step 5: Wire the `bump` subcommand (impure I/O)**

Replace the `"bump"` arm:

```rust
        "bump" => {
            let level = std::env::args().nth(2).unwrap_or_default();
            let dry = std::env::args().any(|a| a == "--dry-run");
            let path = "Cargo.toml";
            let src = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => { eprintln!("xtask bump: read {path}: {e}"); return ExitCode::from(1); }
            };
            let current = src.lines()
                .skip_while(|l| l.trim_start() != "[package]")
                .find_map(|l| l.trim().strip_prefix("version").and_then(|r| r.split('"').nth(1)))
                .unwrap_or("");
            let next = match bump_version(current, &level) {
                Ok(v) => v,
                Err(e) => { eprintln!("xtask bump: {e}"); return ExitCode::from(1); }
            };
            match set_package_version(&src, &next) {
                Ok(new_src) if dry => {
                    println!("xtask bump: {current} -> {next} (dry run, Cargo.toml unchanged)");
                    let _ = new_src;
                    ExitCode::from(0)
                }
                Ok(new_src) => match std::fs::write(path, new_src) {
                    Ok(()) => {
                        println!("xtask bump: {current} -> {next}");
                        println!("next: review, `cargo build` to refresh Cargo.lock, commit, tag v{next}");
                        ExitCode::from(0)
                    }
                    Err(e) => { eprintln!("xtask bump: write {path}: {e}"); ExitCode::from(1) }
                },
                Err(e) => { eprintln!("xtask bump: {e}"); ExitCode::from(1) }
            }
        }
```

- [ ] **Step 6: Smoke-test the dry run (must not mutate)**

Run: `cargo xtask bump patch --dry-run`
Expected: prints `0.0.4 -> 0.0.5 (dry run, Cargo.toml unchanged)`, exit 0.
Run: `git diff --quiet Cargo.toml && echo CLEAN`
Expected: prints `CLEAN` (dry run touched nothing).

- [ ] **Step 7: Commit**

```bash
git add xtask/src/main.rs
git commit -m "$(printf 'feat(xtask): bump patch|minor|major (pure, unit-tested)\n\nbump_version + set_package_version are pure and TDD-tested; set only\ntouches the [package] version, never dep versions. --dry-run previews.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 6: Rewire Makefile.toml and launcher.ps1 to the xtask gate

**Files:**
- Modify: `Makefile.toml`
- Modify: `launcher.ps1`

**Interfaces:**
- Consumes: the `cargo xtask` subcommands.
- Produces: `cargo make verify | verify-full | bump-patch | bump-minor | bump-major` as thin pass-throughs; the launcher's verify/fix/check entries route through them.

- [ ] **Step 1: Add pass-through tasks to `Makefile.toml`**

Append (do not delete the existing tasks — they stay usable):

```toml
# ============================================================================
# CI/CD gate (delegates to the xtask — see doc/ci-cd.md)
# ============================================================================

[tasks.verify]
description = "Local CI gate (advisory): fmt/clippy/build/check-demo/test"
command = "cargo"
args = ["xtask", "verify"]

[tasks.verify-full]
description = "verify + the real trunk wasm build"
command = "cargo"
args = ["xtask", "verify-full"]

[tasks.bump-patch]
description = "Bump the library patch version"
command = "cargo"
args = ["xtask", "bump", "patch"]

[tasks.bump-minor]
description = "Bump the library minor version"
command = "cargo"
args = ["xtask", "bump", "minor"]

[tasks.bump-major]
description = "Bump the library major version"
command = "cargo"
args = ["xtask", "bump", "major"]
```

Then repoint the default task from `verify-all` to the new gate:

```toml
[tasks.default]
description = "Default task: the local CI gate"
alias = "verify"
```

- [ ] **Step 2: Verify cargo-make delegates correctly**

Run: `cargo make verify`
Expected: identical output to `cargo xtask verify` (all steps + summary), exit 0.

- [ ] **Step 3: Point the launcher's verify/check/fix entries at the gate**

In `launcher.ps1`, find the task table entries that run `cargo make verify-all` / `cargo make ci` / `check` (grep `verify-all`, `-Task check`, `-Task verify` in the file) and change the *command they invoke* to `cargo make verify` (leave `fix` → `cargo make fix-all`, which still exists). Do not restructure the menu; only swap the command strings. Show the exact before/after lines in the commit.

- [ ] **Step 4: Verify the launcher path**

Run: `pwsh -NoProfile -File launcher.ps1 -Task verify`
Expected: runs the gate (same summary), exits 0. (If the launcher lacks a `verify` task name, use whichever task name it exposes for verification; confirm it now calls `cargo make verify`.)

- [ ] **Step 5: Commit**

```bash
git add Makefile.toml launcher.ps1
git commit -m "$(printf 'build: route cargo-make + launcher through the xtask gate\n\nThin verify/verify-full/bump-* pass-throughs; default task is now the\nlocal CI gate. Orchestration holds no logic.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 7: Opt-in advisory pre-push hook

**Files:**
- Create: `.githooks/pre-push`

**Interfaces:**
- Produces: an advisory hook that runs `cargo xtask verify` and never blocks.

- [ ] **Step 1: Create `.githooks/pre-push`** (LF line endings; Git Bash runs it)

```sh
#!/bin/sh
# Advisory pre-push: run the local CI gate, report, but NEVER block the push.
# Enable per clone with:  git config core.hooksPath .githooks
# See doc/ci-cd.md.

if command -v cargo >/dev/null 2>&1; then
    echo "[pre-push] cargo xtask verify (advisory) ..."
    if cargo xtask verify; then
        echo "[pre-push] gate green."
    else
        echo "[pre-push] gate reported failures above — pushing anyway (advisory)." >&2
    fi
else
    echo "[pre-push] cargo not found; skipping gate." >&2
fi
exit 0
```

- [ ] **Step 2: Make it executable and enable it locally**

```bash
chmod +x .githooks/pre-push
git config core.hooksPath .githooks
```

- [ ] **Step 3: Verify it runs advisory (never blocks)**

Run: `.githooks/pre-push </dev/null`
Expected: runs the gate, prints `[pre-push] gate green.` (or the advisory failure note), exits 0 regardless.

- [ ] **Step 4: Commit**

```bash
git add .githooks/pre-push
git commit -m "$(printf 'build: add opt-in advisory pre-push hook\n\nRuns cargo xtask verify and reports, never blocks. Enable with\ngit config core.hooksPath .githooks.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

### Task 8: Correct the testing-policy section in `doc/ci-cd.md`

The committed doc says "no committed browser/screenshot suite today" — but the PixelProof `test-visual` suite exists (`scripts/test-visual.ps1`, `tests/visual/baselines`, `#[ignore]d`, `cargo make test-visual`). Correct it so the doc matches reality.

**Files:**
- Modify: `doc/ci-cd.md` — the "Testing policy — screenshot vs. no-screenshot" section.

- [ ] **Step 1: Replace the second bullet of that section**

Replace the "Screenshot / live-browser → manual" bullet with:

```markdown
- **Screenshot / live-browser → manual.** The committed PixelProof suite
  (`tests/visual/**`, baselines under `tests/visual/baselines`) is `#[ignore]`d
  and run on demand via `cargo make test-visual` (`scripts/test-visual.ps1`:
  idempotent `npm install`, `trunk serve` on :3010, run the ignored tests, tear
  down; refresh baselines with `VISUAL_TEST_MODE=capture`). SSIM/baseline
  comparisons are DPI/monitor-specific, so they stay out of `verify`. The
  reactivity/DOM-oracle subset (no screenshot) is a candidate to un-ignore and
  auto-gate in a future pass, mirroring Rust-DeskApp.
```

- [ ] **Step 2: Verify the whole gate is green after all changes**

Run: `cargo xtask verify`
Expected: `6/6 passed`, exit 0.

- [ ] **Step 3: Commit**

```bash
git add doc/ci-cd.md
git commit -m "$(printf 'docs(ci-cd): reference the real PixelProof test-visual suite\n\nCorrect the testing-policy section: the screenshot/baseline suite exists\n(tests/visual, cargo make test-visual) and stays manual; the gate runs\ncargo test --lib only.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Notes for the executor

- **If Task 1 finds `cargo clippy --workspace` fails on native csr unification**, encode the two per-crate clippy steps in `gate_steps()` (Task 3) instead of the single `--workspace` one, and update the `clippy` step name handling in `run_named`.
- **`trunk` may not be installed** on every machine; `verify` never invokes it (only `verify-full` does), so the default gate stays green regardless.
- The bd `pre-commit` hook is already correct here — do not touch it.
- Keep every xtask edit std-only; if you reach for a crate, stop — the design forbids it.
