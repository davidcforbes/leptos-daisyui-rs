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
//! The demo loads pele from the pinned jsDelivr URL, so this lane needs
//! network access to `cdn.jsdelivr.net` (or a demo configured with
//! `pele::set_pele_asset_base`). A load failure shows up as every figure in
//! the `error` state with a "could not load the Mermaid renderer" message.
//!
//! Drives the general demo app (`html_target: None`). Kept in its own
//! file/xtask step (`cargo xtask test-mermaid-pele`) rather than folded into
//! `reactivity_smoke.rs`, whose check count is pinned.
mod common;

use common::{assert_no_browser_errors, begin_browser_error_capture, harness_at};
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
                };
            };
            const md = document.querySelector('[data-testid="mermaid-markdown"] .mermaid');
            const out = {};
            for (const id of ['mermaid-flowchart', 'mermaid-sequence', 'mermaid-pie', 'mermaid-venn', 'mermaid-error']) {
                out[id] = figure(id);
            }
            out.markdown = md ? {
                svg: !!md.querySelector('svg.pele'),
                pending: !!md.querySelector('[data-pele-state="pending"]'),
                error: md.querySelector('.mermaid-error') ? md.querySelector('.mermaid-error').textContent : null,
            } : null;
            out.peleGlobal = typeof window.pele;
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
            "pele never settled within 30 s (is cdn.jsdelivr.net reachable?): {snap}"
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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-mermaid-pele)"]
async fn mermaid_renders_with_pele_and_follows_the_daisyui_theme() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    let snap = settled_snapshot(&h).await;
    assert_eq!(snap["peleGlobal"], json!("object"), "pele global: {snap}");

    for id in DIAGRAMS {
        assert_eq!(snap[id]["state"], json!("rendered"), "{id}: {snap}");
        assert_eq!(snap[id]["svg"], json!(true), "{id} draws svg.pele: {snap}");
        assert_eq!(snap[id]["error"], Value::Null, "{id}: {snap}");
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
        !message.contains("could not load"),
        "the error must come from pele's parser, not the loader: {snap}"
    );

    // MarkdownView's fenced block is drawn by the same engine.
    assert_eq!(snap["markdown"]["svg"], json!(true), "{snap}");
    assert_eq!(snap["markdown"]["error"], Value::Null, "{snap}");

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

    assert_no_browser_errors(&h, "mermaid pele rendering").await;
}
