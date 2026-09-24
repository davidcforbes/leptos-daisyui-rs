//! Showcase capture for the Claude Design sync (`cargo xtask capture-design`).
//!
//! Not a test of anything: it walks every route in the showcase sidebar and
//! records what the real Leptos components rendered, so the design-system
//! upload is built from real output rather than from a reimplementation.
//! For each page it writes the title, the description, and one entry per
//! `Section` example (`[data-demo-section]`): the example's rendered
//! `outerHTML` plus an element screenshot. A page with no Section is
//! captured whole. The page's stylesheets and runtime `<style>` blocks (the
//! token and animation preambles) are written once.
//!
//! Output: `target/design-capture/` (`manifest.json`, `html/`, `png/`,
//! `styles/`). A page that fails is recorded in the manifest with its error
//! and the walk carries on; the lane fails only if no page was captured.

mod common;

use common::{harness_at, wait_for_selector};
use pixelproof_web::{Harness, ViewportSize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 900;
/// Tallest viewport used to fit one example; taller examples are clipped.
const MAX_HEIGHT: u32 = 3200;

/// Run a script in the local headless page (CDP `Runtime.evaluate`).
async fn run_js(h: &Harness, expr: &str) -> Result<Value, String> {
    h.page()
        .evaluate(expr)
        .await
        .map_err(|e| format!("evaluate: {e}"))?
        .into_value()
        .map_err(|e| format!("decode: {e}"))
}

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/design-capture")
}

fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create capture dir");
    }
    std::fs::write(path, bytes).expect("write capture file");
}

/// The sidebar's categories and their component links, read from the DOM.
const MENU_JS: &str = r#"(() => {
    const out = [];
    for (const t of document.querySelectorAll('.menu-title')) {
        const li = t.closest('li');
        if (!li) continue;
        const items = [...li.querySelectorAll('a[href^="/components/"]')].map(a => ({
            name: a.textContent.trim(),
            href: a.getAttribute('href'),
        }));
        if (items.length) out.push({ title: t.textContent.trim(), items });
    }
    return out;
})()"#;

/// Every stylesheet the page loads, and every runtime `<style>` block.
const STYLES_JS: &str = r#"(async () => {
    const links = [];
    for (const l of document.querySelectorAll('link[rel="stylesheet"]')) {
        const res = await fetch(l.href);
        links.push({ href: l.getAttribute('href'), text: await res.text() });
    }
    const styles = [...document.querySelectorAll('style')].map(s => ({
        attrs: Object.fromEntries([...s.attributes].map(a => [a.name, a.value])),
        text: s.textContent,
    }));
    return { links, styles, theme: document.documentElement.dataset.theme || '' };
})()"#;

/// The page's examples. Each gets a `data-cap` index so it can be
/// screenshotted by selector; a page with no Section is captured whole.
const PAGE_JS: &str = r#"(() => {
    const main = document.querySelector('main');
    if (!main) return { error: 'no <main>' };
    const h1 = main.querySelector('h1') || main.querySelector('h2');
    const next = h1 ? h1.nextElementSibling : null;
    const desc = next && next.tagName === 'P' ? next.textContent.trim() : '';
    const sections = [...main.querySelectorAll('[data-demo-section]')].map((el, i) => {
        el.setAttribute('data-cap', String(i));
        const r = el.getBoundingClientRect();
        return { title: el.dataset.demoSection, html: el.outerHTML, w: r.width, h: r.height };
    });
    let page = null;
    if (!sections.length) {
        const root = main.firstElementChild || main;
        root.setAttribute('data-cap', 'page');
        const r = root.getBoundingClientRect();
        page = { html: root.outerHTML, w: r.width, h: r.height };
    }
    return { title: h1 ? h1.textContent.trim() : '', desc, sections, page };
})()"#;

async fn shoot(h: &Harness, cap: &str, height: f64) -> Result<Vec<u8>, String> {
    let tall = height > f64::from(HEIGHT) - 80.0;
    if tall {
        let vh = ((height + 160.0) as u32).min(MAX_HEIGHT);
        h.set_viewport(ViewportSize::new(WIDTH, vh))
            .await
            .map_err(|e| format!("viewport: {e}"))?;
    }
    let sel = format!("[data-cap=\"{cap}\"]");
    run_js(
        h,
        &format!(
            "(() => {{ const e = document.querySelector('{sel}'); \
             if (e) e.scrollIntoView({{ block: 'start' }}); window.scrollTo(0, 0); return !!e; }})()"
        ),
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let png = h
        .screenshot_element(&sel)
        .await
        .map_err(|e| format!("screenshot: {e}"));
    if tall {
        let _ = h.set_viewport(ViewportSize::new(WIDTH, HEIGHT)).await;
    }
    png
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "capture tool, not a test: run via cargo xtask capture-design"]
async fn capture_showcase_for_design_sync() {
    let out = out_dir();
    let _ = std::fs::remove_dir_all(&out);
    let h = harness_at("/components/button").await;
    h.set_viewport(ViewportSize::new(WIDTH, HEIGHT))
        .await
        .expect("viewport");
    wait_for_selector(&h, ".menu-title").await;

    let menu = run_js(&h, MENU_JS).await.expect("read sidebar menu");
    let styles = run_js(&h, STYLES_JS).await.expect("read stylesheets");
    for (i, link) in styles["links"].as_array().into_iter().flatten().enumerate() {
        let href = link["href"].as_str().unwrap_or("");
        let name = href.rsplit('/').next().unwrap_or("sheet.css");
        write(
            &out.join(format!("styles/link-{i}-{name}")),
            link["text"].as_str().unwrap_or("").as_bytes(),
        );
    }
    for (i, style) in styles["styles"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        write(
            &out.join(format!("styles/inline-{i}.css")),
            style["text"].as_str().unwrap_or("").as_bytes(),
        );
    }

    let mut pages = Vec::new();
    let mut captured = 0usize;
    for category in menu.as_array().into_iter().flatten() {
        let group = category["title"].as_str().unwrap_or("Other").to_owned();
        for item in category["items"].as_array().into_iter().flatten() {
            let href = item["href"].as_str().unwrap_or("").to_owned();
            let name = item["name"].as_str().unwrap_or("").to_owned();
            let slug = href.trim_start_matches("/components/").to_owned();
            let mut record = json!({ "group": group, "name": name, "href": href, "slug": slug });
            let result: Result<Value, String> = async {
                h.navigate(&format!("{href}?pp-freeze=1"))
                    .await
                    .map_err(|e| format!("navigate: {e}"))?;
                // Some pages build raw markup with no <h1>: wait for any content.
                ldui_audit::web::wait_for_selector(&h, "main > *", 30_000)
                    .await
                    .map_err(|e| format!("wait: {e}"))?;
                tokio::time::sleep(Duration::from_millis(600)).await;
                let page = run_js(&h, PAGE_JS).await?;
                if let Some(e) = page["error"].as_str() {
                    return Err(e.to_owned());
                }
                let mut sections = Vec::new();
                let list = page["sections"].as_array().cloned().unwrap_or_default();
                let whole = page["page"].clone();
                let entries: Vec<(String, Value)> = if list.is_empty() {
                    vec![("page".to_owned(), whole)]
                } else {
                    list.into_iter()
                        .enumerate()
                        .map(|(i, s)| (i.to_string(), s))
                        .collect()
                };
                for (cap, s) in entries {
                    let html_rel = format!("html/{slug}/{cap}.html");
                    write(
                        &out.join(&html_rel),
                        s["html"].as_str().unwrap_or("").as_bytes(),
                    );
                    let (w, ht) = (
                        s["w"].as_f64().unwrap_or(0.0),
                        s["h"].as_f64().unwrap_or(0.0),
                    );
                    let mut entry = json!({
                        "title": s["title"].as_str().unwrap_or(""),
                        "html": html_rel, "w": w, "h": ht,
                    });
                    if w >= 1.0 && ht >= 1.0 {
                        match shoot(&h, &cap, ht).await {
                            Ok(png) => {
                                let png_rel = format!("png/{slug}/{cap}.png");
                                write(&out.join(&png_rel), &png);
                                entry["png"] = json!(png_rel);
                                entry["clipped"] = json!(ht + 160.0 > f64::from(MAX_HEIGHT));
                            }
                            Err(e) => entry["png_error"] = json!(e),
                        }
                    }
                    sections.push(entry);
                }
                Ok(json!({
                    "title": page["title"], "desc": page["desc"], "sections": sections,
                }))
            }
            .await;
            match result {
                Ok(v) => {
                    record["page"] = v;
                    captured += 1;
                }
                Err(e) => record["error"] = json!(e),
            }
            eprintln!(
                "captured {slug}: {}",
                record
                    .get("error")
                    .map_or("ok".to_owned(), |e| e.to_string())
            );
            pages.push(record);
        }
    }

    let manifest = json!({
        "theme": styles["theme"],
        "viewport": [WIDTH, HEIGHT],
        "stylesheets": styles["links"].as_array().map(|l| l.iter().map(|x| x["href"].clone()).collect::<Vec<_>>()),
        "inlineStyles": styles["styles"].as_array().map(|s| s.iter().map(|x| x["attrs"].clone()).collect::<Vec<_>>()),
        "categories": menu,
        "pages": pages,
    });
    write(
        &out.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)
            .expect("serialize manifest")
            .as_bytes(),
    );
    eprintln!("design capture: {captured} pages -> {}", out.display());
    assert!(captured > 0, "no showcase page was captured");
}
