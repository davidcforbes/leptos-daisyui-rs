//! Real-browser proof that `MarkdownView` math renders as native MathML
//! through math-core (ldui-r0co).
//!
//! On the `/components/markdown_math` showcase route this checks that:
//! - no KaTeX is involved (`window.katex` is undefined) and nothing loads
//!   from a CDN;
//! - every everyday formula reports `data-math-state="rendered"` and
//!   `data-math-stripped="0"`, holds one MathML-namespace `<math>` with
//!   `<semantics>` and an `application/x-tex` annotation equal to its
//!   source, has no `aria-label`, takes up real space, and display math
//!   carries `display="block"`;
//! - math-core's fixes stylesheet is installed exactly once;
//! - a formula math-core rejects, and a macro definition the guard refuses,
//!   keep their `$…$` source visible with `data-math-error` and a reason in
//!   `title`;
//! - hostile input: nesting at the guard's limit renders, 500
//!   levels is refused by the guard (on wasm it would otherwise overflow
//!   the stack and trap the whole app), a `\href{javascript:…}` is a LaTeX
//!   error, markup in `\text{}` stays text, a `\color` is dropped, the
//!   unbraced `\frac1\frac1…` and nested `\sqrt[` shapes are refused by the
//!   guard, a `\hspace{100em}` is dropped rather than pushing math past its
//!   container, and the page still answers a click afterwards;
//! - a `data-theme` switch recolours math with no re-render;
//! - every section is axe-clean (WCAG 2.2 AA, through `common::axe`) in the
//!   light and dark themes.
//!
//! Drives the general demo app (`html_target: None`) through
//! `cargo xtask test-math`. The lane builds the demo with `--release`, so
//! the "at the limit" formula exercises the guard's release limit (64
//! entries) in real wasm; the debug limit (20) rests on the native
//! measurements in `src/markdown/math/guard.rs` and its 1 MiB-stack test.
mod common;

use common::axe::assert_axe_clean;
use common::{assert_no_browser_errors, begin_browser_error_capture, click, harness_at};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const PAGE: &str = "/components/markdown_math";
const SECTIONS: [&str; 3] = ["math-everyday", "math-errors", "math-hostile"];

async fn eval_json(h: &pixelproof_web::Harness, expr: &str) -> Value {
    h.page()
        .evaluate(expr)
        .await
        .expect("evaluate markdown math fixture")
        .into_value()
        .expect("markdown math expression returns JSON")
}

/// Poll `expr` until it returns `true` (10 s budget).
async fn wait_until(h: &pixelproof_web::Harness, expr: &str, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if eval_json(h, expr).await == json!(true) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Every formula host in a section, described.
async fn formulas(h: &pixelproof_web::Harness, section: &str) -> Value {
    eval_json(
        h,
        &format!(
            r#"(() => {{
                const NS = 'http://www.w3.org/1998/Math/MathML';
                const root = document.querySelector('[data-testid="{section}"]');
                return [...root.querySelectorAll('.math-inline, .math-display')].map((el) => {{
                    const maths = el.getElementsByTagNameNS(NS, 'math');
                    const math = maths[0] || null;
                    const ann = math ? math.querySelector('semantics > annotation') : null;
                    const rect = math ? math.getBoundingClientRect() : null;
                    const body = math ? math.querySelector('semantics > mrow') : null;
                    return {{
                        display: el.classList.contains('math-display'),
                        state: el.getAttribute('data-math-state'),
                        stripped: el.getAttribute('data-math-stripped'),
                        error: el.getAttribute('data-math-error'),
                        title: el.getAttribute('title'),
                        text: el.textContent,
                        maths: maths.length,
                        mathDisplay: math ? math.getAttribute('display') : null,
                        ariaLabel: math ? math.getAttribute('aria-label') : null,
                        semantics: !!(math && math.firstElementChild
                            && math.firstElementChild.localName === 'semantics'
                            && math.firstElementChild.namespaceURI === NS),
                        annotationEncoding: ann ? ann.getAttribute('encoding') : null,
                        annotation: ann ? ann.textContent : null,
                        bodyChildren: body ? body.childElementCount : 0,
                        width: rect ? rect.width : 0,
                        height: rect ? rect.height : 0,
                        foreign: math ? [...math.querySelectorAll('*')].filter((n) =>
                            n.namespaceURI !== NS && !n.closest('mtext')).map((n) => n.localName) : [],
                    }};
                }});
            }})()"#
        ),
    )
    .await
}

async fn everyday(h: &pixelproof_web::Harness) {
    wait_until(
        h,
        r#"(() => {
            const els = document.querySelectorAll('[data-testid="math-everyday"] .math-inline, [data-testid="math-everyday"] .math-display');
            return els.length > 0 && [...els].every((el) => el.hasAttribute('data-math-state'));
        })()"#,
        "everyday formulas to render",
    )
    .await;
    let list = formulas(h, "math-everyday").await;
    let list = list.as_array().expect("formula list");
    assert_eq!(list.len(), 8, "4 inline + 4 display: {list:?}");
    assert_eq!(
        list.iter().filter(|f| f["display"] == json!(true)).count(),
        4,
        "{list:?}"
    );
    for f in list {
        assert_eq!(f["state"], json!("rendered"), "{f}");
        assert_eq!(
            f["stripped"],
            json!("0"),
            "the allow-list removed something: {f}"
        );
        assert_eq!(f["error"], Value::Null, "{f}");
        assert_eq!(f["maths"], json!(1), "exactly one MathML <math>: {f}");
        assert_eq!(f["semantics"], json!(true), "{f}");
        assert_eq!(f["annotationEncoding"], json!("application/x-tex"), "{f}");
        assert!(
            f["annotation"]
                .as_str()
                .is_some_and(|a| !a.is_empty() && !a.contains('$')),
            "the annotation is the TeX source without delimiters: {f}"
        );
        assert_eq!(
            f["ariaLabel"],
            Value::Null,
            "no aria-label over the MathML: {f}"
        );
        assert!(f["bodyChildren"].as_u64().unwrap_or(0) > 0, "{f}");
        assert!(
            f["width"].as_f64().unwrap_or(0.0) > 4.0 && f["height"].as_f64().unwrap_or(0.0) > 4.0,
            "the formula takes up real space: {f}"
        );
        assert_eq!(f["foreign"], json!([]), "only MathML outside mtext: {f}");
        let block = if f["display"] == json!(true) {
            json!("block")
        } else {
            Value::Null
        };
        assert_eq!(f["mathDisplay"], block, "{f}");
    }
    let annotations: Vec<&str> = list
        .iter()
        .filter_map(|f| f["annotation"].as_str())
        .collect();
    assert!(annotations.contains(&"E=mc^2"), "{annotations:?}");

    let page = eval_json(
        h,
        r#"(() => ({
            katex: typeof window.katex,
            fixes: document.querySelectorAll('style#ldui-mathmlfixes').length,
            fixesHasMtd: (document.getElementById('ldui-mathmlfixes')?.textContent || '').includes('mtd'),
            cdn: [...document.querySelectorAll('script[src], link[href]')]
                .map((n) => n.getAttribute('src') || n.getAttribute('href'))
                .filter((u) => /^https?:\/\//.test(u) && !u.startsWith(location.origin)),
            mathFont: getComputedStyle(document.querySelector('[data-testid="math-everyday"] math')).fontFamily,
        }))()"#,
    )
    .await;
    assert_eq!(page["katex"], json!("undefined"), "{page}");
    assert_eq!(
        page["fixes"],
        json!(1),
        "fixes stylesheet installed once: {page}"
    );
    assert_eq!(page["fixesHasMtd"], json!(true), "{page}");
    let cdn = page["cdn"].as_array().cloned().unwrap_or_default();
    assert!(
        !cdn.iter()
            .any(|u| u.as_str().is_some_and(|u| u.contains("katex"))),
        "no KaTeX from a CDN: {page}"
    );
    assert!(
        page["mathFont"]
            .as_str()
            .is_some_and(|f| f.contains("Math")),
        "math uses an OpenType MATH font stack: {page}"
    );
}

async fn errors(h: &pixelproof_web::Harness) {
    let list = formulas(h, "math-errors").await;
    let list = list.as_array().expect("formula list");
    assert_eq!(list.len(), 2, "{list:?}");
    let kinds: Vec<&str> = list.iter().filter_map(|f| f["error"].as_str()).collect();
    assert_eq!(kinds, ["latex", "guard"], "{list:?}");
    for f in list {
        assert_eq!(f["state"], json!("error"), "{f}");
        assert_eq!(f["maths"], json!(0), "{f}");
        let text = f["text"].as_str().unwrap_or_default();
        assert!(
            text.starts_with('$') && text.ends_with('$') && text.len() > 2,
            "the source stays visible: {f}"
        );
        assert!(f["title"].as_str().is_some_and(|t| t.len() > 10), "{f}");
    }
    assert!(
        list[0]["title"]
            .as_str()
            .unwrap_or_default()
            .contains("notacommand"),
        "math-core's own message: {list:?}"
    );
}

async fn hostile(h: &pixelproof_web::Harness) {
    click(h, "[data-testid=\"math-hostile-start\"]").await;
    wait_until(
        h,
        r#"(() => {
            const els = document.querySelectorAll('[data-testid="math-hostile"] .math-inline');
            return els.length === 8 && [...els].every((el) => el.hasAttribute('data-math-state'));
        })()"#,
        "hostile formulas to settle",
    )
    .await;
    let list = formulas(h, "math-hostile").await;
    let list = list.as_array().expect("formula list");
    let (under, deep, link, markup, colour) = (&list[0], &list[1], &list[2], &list[3], &list[4]);
    let (unbraced, optional, space) = (&list[5], &list[6], &list[7]);
    assert_eq!(
        under["state"],
        json!("rendered"),
        "just under the guard renders: {under}"
    );
    assert_eq!(under["stripped"], json!("0"), "{under}");
    assert_eq!(deep["state"], json!("error"), "{deep}");
    assert_eq!(
        deep["error"],
        json!("guard"),
        "500 levels never reach math-core: {deep}"
    );
    assert_eq!(
        link["error"],
        json!("latex"),
        "\\href is not a command: {link}"
    );
    assert_eq!(markup["state"], json!("rendered"), "{markup}");
    assert_eq!(colour["state"], json!("rendered"), "{colour}");
    assert_ne!(
        colour["stripped"],
        json!("0"),
        "the colour is stripped: {colour}"
    );
    // The two shapes that got past the first guard (security review):
    // unbraced arguments and optional-argument brackets nest too.
    for (name, f) in [
        (r"unbraced \frac1 x500", unbraced),
        (r"\sqrt[ x200", optional),
    ] {
        assert_eq!(
            f["error"],
            json!("guard"),
            "{name} never reaches math-core: {f}"
        );
    }
    assert_eq!(space["state"], json!("rendered"), "{space}");
    assert_ne!(
        space["stripped"],
        json!("0"),
        "a 100em space is dropped: {space}"
    );

    let probe = eval_json(
        h,
        r#"(() => {
            const root = document.querySelector('[data-testid="math-hostile"]');
            const els = root.querySelectorAll('.math-inline');
            const markup = els[3];
            const colour = els[4].querySelector('mi');
            return {
                imgs: root.querySelectorAll('img').length,
                // math-core writes text-mode spaces as U+00A0.
                markupText: (markup.querySelector('mtext')?.textContent || '').replace(/\u00a0/g, ' '),
                hrefs: root.querySelectorAll('[href]').length,
                // \color pins the whole formula; MarkdownView's own text
                // colour must win.
                colourOk: getComputedStyle(colour).color
                    === getComputedStyle(els[4].closest('.lds-root')).color,
                spaceWidth: els[7].querySelector('math').getBoundingClientRect().width,
                fontPx: parseFloat(getComputedStyle(els[7]).fontSize),
                rootWidth: root.getBoundingClientRect().width,
                viewport: document.documentElement.clientWidth,
            };
        })()"#,
    )
    .await;
    assert_eq!(
        probe["imgs"],
        json!(0),
        "markup in \\text{{}} must stay text: {probe}"
    );
    assert!(
        probe["markupText"]
            .as_str()
            .is_some_and(|t| t.contains("<img src=x onerror=alert(1)>")),
        "{probe}"
    );
    assert_eq!(probe["hrefs"], json!(0), "{probe}");
    assert_eq!(
        probe["colourOk"],
        json!(true),
        "a pinned colour must not survive: {probe}"
    );
    assert!(
        probe["spaceWidth"].as_f64().unwrap_or(f64::MAX)
            < 25.0 * probe["fontPx"].as_f64().unwrap_or(0.0),
        "a 100em \\hspace must be dropped, not drawn: {probe}"
    );
    assert!(
        probe["rootWidth"].as_f64().unwrap_or(f64::MAX)
            <= probe["viewport"].as_f64().unwrap_or(0.0),
        "a refused formula's long source must wrap, not widen the page: {probe}"
    );

    // The wasm instance is still alive: a click still updates a signal.
    let before = eval_json(
        h,
        "document.querySelector('[data-testid=\"math-ping-count\"]').textContent",
    )
    .await;
    click(h, "[data-testid=\"math-ping\"]").await;
    wait_until(
        h,
        &format!(
            "document.querySelector('[data-testid=\"math-ping-count\"]').textContent !== {before}"
        ),
        "the page to answer a click after hostile input",
    )
    .await;
}

async fn theme_switch(h: &pixelproof_web::Harness) {
    let read = |theme: &'static str| {
        format!(
            r#"(async () => {{
                document.documentElement.setAttribute('data-theme', '{theme}');
                await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                const mi = document.querySelector('[data-testid="math-everyday"] .math-display mi');
                return {{ same: mi === window.__lduiMathNode, color: getComputedStyle(mi).color }};
            }})()"#
        )
    };
    eval_json(
        h,
        r#"(window.__lduiMathNode = document.querySelector('[data-testid="math-everyday"] .math-display mi'), true)"#,
    )
    .await;
    let light = eval_json(h, &read("light")).await;
    let dark = eval_json(h, &read("dark")).await;
    assert_eq!(light["same"], json!(true), "no re-render: {light}");
    assert_eq!(dark["same"], json!(true), "no re-render: {dark}");
    assert_ne!(
        light["color"], dark["color"],
        "math follows the theme: {light} {dark}"
    );
}

async fn axe_in_theme(h: &pixelproof_web::Harness, theme: &str) {
    eval_json(
        h,
        &format!(
            r#"(async () => {{
                document.documentElement.setAttribute('data-theme', '{theme}');
                await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                return true;
            }})()"#
        ),
    )
    .await;
    for section in SECTIONS {
        let scope = format!("[data-testid=\"{section}\"]");
        eval_json(
            h,
            &format!(
                r#"(async () => {{
                    document.querySelector('{scope}').scrollIntoView({{ block: 'start' }});
                    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
                    return true;
                }})()"#,
                scope = scope.replace('\'', "\\'")
            ),
        )
        .await;
        assert_axe_clean(h, Some(&scope), &format!("{section} ({theme} theme)")).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires demo dev server (cargo xtask test-math)"]
async fn markdown_math_renders_native_mathml_safely_and_accessibly() {
    let h = harness_at(PAGE).await;
    begin_browser_error_capture(&h).await;

    everyday(&h).await;
    errors(&h).await;
    hostile(&h).await;

    let original = eval_json(&h, "document.documentElement.getAttribute('data-theme')").await;
    theme_switch(&h).await;
    axe_in_theme(&h, "light").await;
    axe_in_theme(&h, "dark").await;
    let restore = match original.as_str() {
        Some(theme) => {
            format!("document.documentElement.setAttribute('data-theme', '{theme}'), true")
        }
        None => "document.documentElement.removeAttribute('data-theme'), true".to_string(),
    };
    eval_json(&h, &restore).await;

    assert_no_browser_errors(&h, "markdown math rendering").await;
}
