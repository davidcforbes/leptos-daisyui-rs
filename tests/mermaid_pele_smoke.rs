//! Real-browser proof that Mermaid renders through pele (ldui-qe72).
//!
//! On the `/components/mermaid_display` showcase route this checks that:
//! - `MermaidDisplay` draws `svg.pele` for a flowchart, a sequence diagram,
//!   a pie chart and a Venn diagram (a type the retired native Rust
//!   renderer could not draw), each figure reporting
//!   `data-mermaid-state="rendered"`;
//! - broken source reports `data-mermaid-state="error"` with pele's own
//!   message in the `mermaid-error` box and no SVG;
//! - a fenced `mermaid` block inside `MarkdownView` is drawn by pele too;
//! - switching `data-theme` recolours a drawn diagram WITHOUT re-rendering
//!   it: the same SVG node stays mounted while the `--pele-*` colours it
//!   resolves change (the daisyUI mapping in `styles/tokens.css`).
//!
//! And for the worker design (ldui-o55o):
//! - pele never runs on the page's thread (`window.pele` is undefined) and
//!   nothing is fetched from a CDN: the demo serves `assets/pele/` at
//!   `/pele/`, same-origin;
//! - the sanitiser removes nothing from pele's own output
//!   (`data-pele-stripped="0"` on every drawn figure);
//! - the pele review's 663-byte hostile Gantt chart (pele-qd0.4) times out
//!   in the worker (`data-pele-error="timeout"`) while the page's event loop
//!   keeps turning, and the diagram queued after it draws in a fresh worker.
//!
//! No network beyond the demo server is needed. A worker that cannot start
//! shows up as every figure in the `error` state with
//! `data-pele-error="load"`.
//!
//! Drives the general demo app (`html_target: None`). Kept in its own
//! file/xtask step (`cargo xtask test-mermaid-pele`) rather than folded into
//! `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const PAGE: &str = "/components/mermaid_display";
const DIAGRAMS: [&str; 4] = [
    "mermaid-flowchart",
    "mermaid-sequence",
    "mermaid-pie",
    "mermaid-venn",
];

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate mermaid fixture")
        .into_value()
        .expect("mermaid expression returns JSON")
}

/// Every fixture's render state, SVG presence and error text.
async fn snapshot(h: &pixelproof_web::Harness) -> Value {
    eval_json(
        h,
        r#"(() => {
            const figure = (id) => {
                const f = document.querySelector(`[data-testid="${id}"]`);
                if (!f) return null;
                const err = f.querySelector('.mermaid-error');
                return {
                    state: f.getAttribute('data-mermaid-state'),
                    svg: !!f.querySelector('svg.pele'),
                    error: err ? err.textContent : null,
                    stripped: f.getAttribute('data-pele-stripped'),
                };
            };
            const md = document.querySelector('[data-testid="mermaid-markdown"] .mermaid');
            const out = {};
            for (const id of ['mermaid-flowchart', 'mermaid-sequence', 'mermaid-pie', 'mermaid-venn', 'mermaid-error']) {
                out[id] = figure(id);
            }
            out.markdown = md ? {
                svg: !!md.querySelector('svg.pele'),
                stripped: md.getAttribute('data-pele-stripped'),
                pending: !!md.querySelector('[data-pele-state="pending"]'),
                error: md.querySelector('.mermaid-error') ? md.querySelector('.mermaid-error').textContent : null,
            } : null;
            out.peleGlobal = typeof window.pele;
            out.cdn = performance.getEntriesByType('resource')
                .map(e => e.name)
                .filter(n => /jsdelivr|unpkg|cdnjs/i.test(n));
            return out;
        })()"#,
    )
    .await
}

/// Poll until no figure is still pending (pele loads once, asynchronously).
async fn settled_snapshot(h: &pixelproof_web::Harness) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let snap = snapshot(h).await;
        let figures_pending = DIAGRAMS
            .iter()
            .chain(std::iter::once(&"mermaid-error"))
            .any(|id| snap[*id]["state"] == json!("pending") || snap[*id].is_null());
        let markdown_pending =
            snap["markdown"].is_null() || snap["markdown"]["pending"] == json!(true);
        if !figures_pending && !markdown_pending {
            return snap;
        }
        assert!(
            Instant::now() < deadline,
            "pele never settled within 30 s (is /pele/ldui-pele-worker.js served?): {snap}"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Set `data-theme` on `<html>` and read the flowchart SVG's resolved
/// colours, plus whether it is still the node marked before the switch.
async fn colours_under(h: &pixelproof_web::Harness, theme: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(async () => {{
                document.documentElement.setAttribute('data-theme', '{theme}');
                await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                const svg = document.querySelector('[data-testid="mermaid-flowchart"] svg.pele');
                const cs = svg ? getComputedStyle(svg) : null;
                return {{
                    sameNode: !!svg && svg === window.__lduiPeleNode,
                    bg: cs ? cs.getPropertyValue('--pele-bg').trim() : null,
                    fg: cs ? cs.getPropertyValue('--pele-fg').trim() : null,
                    surface: cs ? cs.getPropertyValue('--pele-surface').trim() : null,
                }};
            }})()"#
        ),
    )
    .await
}

/// Click the hostile fixture, sample the page's event loop while the
/// worker grinds, then wait for the timeout and the recovery render.
async fn hostile_diagram_times_out_without_freezing_the_page(h: &pixelproof_web::Harness) {
    // Before the click: `click` itself waits the harness settle (~500 ms).
    let started = Instant::now();
    click(h, "[data-testid=\"mermaid-hostile-start\"]").await;
    // The worker is busy on the hostile chart for this whole window (the
    // settle plus 1.5 s ends before the 2 s limit plus the click's own
    // latency): the timeout asserted below only fires after the full limit.
    let gaps = eval_json(
        h,
        r#"(async () => {
            const gaps = [];
            let last = performance.now();
            const until = last + 1500;
            while (performance.now() < until) {
                await new Promise((r) => setTimeout(r, 0));
                const now = performance.now();
                gaps.push(now - last);
                last = now;
            }
            return { max: Math.max(...gaps), samples: gaps.length };
        })()"#,
    )
    .await;
    assert!(
        gaps["max"].as_f64().is_some_and(|max| max < 250.0),
        "the page's event loop stalled while pele drew the hostile chart: {gaps}"
    );

    let deadline = Instant::now() + Duration::from_secs(20);
    let outcome = loop {
        let outcome = eval_json(
            h,
            r#"(() => {
                const fig = (id) => document.querySelector('[data-testid="' + id + '"]');
                const hostile = fig('mermaid-hostile');
                const after = fig('mermaid-after-hostile');
                const err = hostile.querySelector('.mermaid-error');
                return {
                    hostile: hostile.getAttribute('data-mermaid-state'),
                    code: err ? err.getAttribute('data-pele-error') : null,
                    message: err ? err.textContent : null,
                    after: after.getAttribute('data-mermaid-state'),
                    afterSvg: !!after.querySelector('svg.pele'),
                };
            })()"#,
        )
        .await;
        if outcome["after"] != json!("pending") && outcome["hostile"] != json!("pending") {
            break outcome;
        }
        assert!(
            Instant::now() < deadline,
            "the hostile render never settled: {outcome}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert_eq!(outcome["hostile"], json!("error"), "{outcome}");
    assert_eq!(outcome["code"], json!("timeout"), "{outcome}");
    assert!(
        outcome["message"]
            .as_str()
            .is_some_and(|m| m.contains("took longer than")),
        "{outcome}"
    );
    assert!(
        started.elapsed() >= Duration::from_secs(2),
        "the time limit is 2 s, so the timeout cannot be earlier"
    );
    assert_eq!(
        outcome["after"],
        json!("rendered"),
        "the next diagram must draw in a fresh worker: {outcome}"
    );
    assert_eq!(outcome["afterSvg"], json!(true), "{outcome}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-mermaid-pele)"]
async fn mermaid_renders_with_pele_and_follows_the_daisyui_theme() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let snap = settled_snapshot(&h).await;
    assert_eq!(
        snap["peleGlobal"],
        json!("undefined"),
        "pele must run in its worker, never on the page's thread: {snap}"
    );
    assert_eq!(
        snap["cdn"],
        json!([]),
        "nothing may load from a CDN: {snap}"
    );

    for id in DIAGRAMS {
        assert_eq!(snap[id]["state"], json!("rendered"), "{id}: {snap}");
        assert_eq!(snap[id]["svg"], json!(true), "{id} draws svg.pele: {snap}");
        assert_eq!(snap[id]["error"], Value::Null, "{id}: {snap}");
        assert_eq!(
            snap[id]["stripped"],
            json!("0"),
            "{id}: the sanitiser must not remove anything pele draws: {snap}"
        );
    }

    // Broken source: pele's own message, escaped as text, and no diagram.
    let broken = &snap["mermaid-error"];
    assert_eq!(broken["state"], json!("error"), "{snap}");
    assert_eq!(broken["svg"], json!(false), "{snap}");
    let message = broken["error"].as_str().unwrap_or_default();
    assert!(
        message.starts_with("Diagram error: ") && message.len() > "Diagram error: ".len(),
        "the error box must carry pele's message: {snap}"
    );
    assert!(
        !message.contains("could not start"),
        "the error must come from pele's parser, not the worker start: {snap}"
    );

    // MarkdownView's fenced block is drawn by the same engine.
    assert_eq!(snap["markdown"]["svg"], json!(true), "{snap}");
    assert_eq!(snap["markdown"]["error"], Value::Null, "{snap}");
    assert_eq!(snap["markdown"]["stripped"], json!("0"), "{snap}");

    // Theme switch recolours without a re-render.
    let original: Value = eval_json(
        &h,
        r#"(() => {
            window.__lduiPeleNode = document.querySelector('[data-testid="mermaid-flowchart"] svg.pele');
            return document.documentElement.getAttribute('data-theme');
        })()"#,
    )
    .await;
    let light = colours_under(&h, "light").await;
    let dark = colours_under(&h, "dark").await;
    for (name, colours) in [("light", &light), ("dark", &dark)] {
        assert_eq!(
            colours["sameNode"],
            json!(true),
            "{name}: the diagram must not be re-rendered by a theme switch: {colours}"
        );
        for var in ["bg", "fg", "surface"] {
            assert!(
                colours[var].as_str().is_some_and(|v| !v.is_empty()),
                "{name}: --pele-{var} must resolve from the daisyUI theme: {colours}"
            );
        }
    }
    for var in ["bg", "fg", "surface"] {
        assert_ne!(
            light[var], dark[var],
            "--pele-{var} must change with data-theme (light {light}, dark {dark})"
        );
    }
    let restore = match original.as_str() {
        Some(theme) => {
            format!("document.documentElement.setAttribute('data-theme', '{theme}'), true")
        }
        None => "document.documentElement.removeAttribute('data-theme'), true".to_string(),
    };
    eval_json(&h, &restore).await;

    hostile_diagram_times_out_without_freezing_the_page(&h).await;

    assert_no_browser_errors(&h, "mermaid pele rendering").await;
}
