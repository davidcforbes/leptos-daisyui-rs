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
//! - a sankey chart keeps its gradients (`linearGradient`, `fill="url(#..)"`)
//!   through the sanitiser;
//! - the pele review's 663-byte hostile Gantt chart (pele-qd0.4), which kept
//!   1.8.0 busy for 7+ s, is refused at once by the vendored fixed build
//!   (`data-pele-error="limit"`);
//! - a diagram with a 1 ms `render_timeout_ms` times out in the worker
//!   (`data-pele-error="timeout"`), and the diagram queued after it draws in
//!   a fresh worker.
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
const DIAGRAMS: [&str; 6] = [
    "mermaid-flowchart",
    "mermaid-sequence",
    "mermaid-pie",
    "mermaid-venn",
    "mermaid-sankey",
    "mermaid-nested",
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
                    gradients: f.querySelectorAll('linearGradient').length,
                    gradientFills: [...f.querySelectorAll('[fill^="url(#"]')].length,
                    colorMix: [...f.querySelectorAll('[style*="color-mix("], [fill*="color-mix("], [stroke*="color-mix("]')].length,
                };
            };
            const md = document.querySelector('[data-testid="mermaid-markdown"] .mermaid');
            const out = {};
            for (const id of ['mermaid-flowchart', 'mermaid-sequence', 'mermaid-pie', 'mermaid-venn', 'mermaid-sankey', 'mermaid-nested', 'mermaid-error']) {
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

/// Click the hostile fixture and wait for its three figures to settle:
/// the review's Gantt chart is refused fast by the fixed build, the 1 ms
/// chart times out, and the chart queued after them draws in a fresh worker.
async fn hostile_and_time_limited_diagrams(h: &pixelproof_web::Harness) {
    let started = Instant::now();
    click(h, "[data-testid=\"mermaid-hostile-start\"]").await;
    let deadline = Instant::now() + Duration::from_secs(20);
    let outcome = loop {
        let outcome = eval_json(
            h,
            r#"(() => {
                const fig = (id) => document.querySelector('[data-testid="' + id + '"]');
                const read = (id) => {
                    const f = fig(id);
                    const err = f.querySelector('.mermaid-error');
                    return {
                        state: f.getAttribute('data-mermaid-state'),
                        code: err ? err.getAttribute('data-pele-error') : null,
                        message: err ? err.textContent : null,
                        svg: !!f.querySelector('svg.pele'),
                    };
                };
                return {
                    hostile: read('mermaid-hostile'),
                    limited: read('mermaid-time-limited'),
                    after: read('mermaid-after-hostile'),
                };
            })()"#,
        )
        .await;
        let settled = ["hostile", "limited", "after"]
            .iter()
            .all(|k| !matches!(outcome[*k]["state"].as_str(), Some("pending" | "empty")));
        if settled {
            break outcome;
        }
        assert!(
            Instant::now() < deadline,
            "the hostile fixtures never settled: {outcome}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    // pele 1.8.0 spent 7+ s here; the fixed build refuses it with a limit
    // error (and the worker's 2 s limit would report timeout instead).
    assert_eq!(outcome["hostile"]["state"], json!("error"), "{outcome}");
    assert_eq!(
        outcome["hostile"]["code"],
        json!("limit"),
        "the vendored fixed build must refuse the review's Gantt chart: {outcome}"
    );
    assert_eq!(outcome["limited"]["state"], json!("error"), "{outcome}");
    assert_eq!(outcome["limited"]["code"], json!("timeout"), "{outcome}");
    assert!(
        outcome["limited"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("took longer than 1 ms")),
        "{outcome}"
    );
    assert_eq!(
        outcome["after"]["state"],
        json!("rendered"),
        "the next diagram must draw in a fresh worker: {outcome}"
    );
    assert_eq!(outcome["after"]["svg"], json!(true), "{outcome}");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "all three should settle well inside the 2 s limit each"
    );
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
    assert!(
        snap["mermaid-sankey"]["gradients"].as_u64().unwrap_or(0) > 0
            && snap["mermaid-sankey"]["gradientFills"]
                .as_u64()
                .unwrap_or(0)
                > 0,
        "the sankey's gradients must survive the sanitiser: {snap}"
    );
    assert!(
        snap["mermaid-nested"]["colorMix"].as_u64().unwrap_or(0) > 0,
        "the struck cluster titles' color-mix() halo must survive the sanitiser: {snap}"
    );

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

    hostile_and_time_limited_diagrams(&h).await;

    assert_no_browser_errors(&h, "mermaid pele rendering").await;
}
