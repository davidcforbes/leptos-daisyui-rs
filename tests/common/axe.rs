//! The axe-core gate every browser suite shares (ldui-3kix).
//!
//! axe-core is the accessibility oracle, with one blind spot this build
//! creates. daisyUI 5's `.btn` declares
//! `background-image: none, var(--btn-noise)`, so a plain button's computed
//! `background-image` is the two-layer `none, none`. axe-core takes any value
//! other than `none` for an image, and files the contrast of every daisyUI
//! button's text as INCOMPLETE (`messageKey: "bgImage"`), never as a
//! violation. `AxeReport::assert_no_blocking` reads violations only, so a
//! button's text could fall to 1.79:1 with every axe check green: MonthRoster's
//! week buttons did (ldui-on0i, measured 2026-10-12).
//!
//! [`axe_findings`] runs axe and then re-measures exactly those nodes: axe
//! still picks the node and judges its visibility, and the background is taken
//! from `document.elementsFromPoint` (as axe takes it), composited down to the
//! first opaque fill. A stack with a REAL image or gradient stays unjudged,
//! because axe is right that it cannot be measured from colours.
//!
//! One limit remains and is axe's own: it does not contrast-check text
//! outside the visible part of the page's scroll container (the demo scrolls
//! inside `main`), and reports such nodes nowhere. Scroll the subject into
//! view before calling the gate.

use pixelproof_web::Harness;
use serde_json::{Value, json};

/// Runs axe on `scope` (or the document) and returns
/// `{ violations, unjudged }`: `violations` are axe's serious and critical
/// findings (every finding when `ruleIds` narrows the run to named rules),
/// and `unjudged` lists the nodes whose contrast axe could not judge because
/// of daisyUI's `none, none` image and which measure below AA.
const AXE_GATE_JS: &str = r#"async (scopeSelector, ruleIds) => {
    const scope = scopeSelector ? document.querySelector(scopeSelector) : document;
    if (!scope) return { error: 'no element matches ' + scopeSelector };
    const report = await axe.run(scope, {
        runOnly: ruleIds
            ? { type: 'rule', values: ruleIds }
            : { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] },
        resultTypes: ['violations', 'incomplete'],
    });
    const violations = report.violations
        .filter((v) => ruleIds || v.impact === 'serious' || v.impact === 'critical')
        .map((v) => ({
            id: v.id,
            impact: v.impact,
            help: v.help,
            nodes: v.nodes.map((n) => ({ target: n.target, html: n.html.slice(0, 200), summary: n.failureSummary })),
        }));
    const ctx = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
    const rgba = (css) => {
        ctx.clearRect(0, 0, 1, 1);
        ctx.fillStyle = 'rgba(0, 0, 0, 0)';
        ctx.fillStyle = css;
        ctx.fillRect(0, 0, 1, 1);
        const d = ctx.getImageData(0, 0, 1, 1).data;
        return [d[0], d[1], d[2], d[3] / 255];
    };
    const over = (top, under) => [0, 1, 2].map((i) => top[i] * top[3] + under[i] * (1 - top[3])).concat([1]);
    const lum = (c) => c.slice(0, 3).map((v) => {
        v /= 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    }).reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
    const ratio = (a, b) => {
        const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
        return (x + 0.05) / (y + 0.05);
    };
    const unjudged = [];
    for (const rule of report.incomplete) {
        if (rule.id !== 'color-contrast') continue;
        for (const node of rule.nodes) {
            const check = [...node.any, ...node.all, ...node.none]
                .find((c) => c.data && c.data.messageKey === 'bgImage');
            if (!check || node.target.length !== 1) continue;
            const el = document.querySelector(node.target[0]);
            if (!el) continue;
            const box = el.getBoundingClientRect();
            const points = document.elementsFromPoint(box.left + box.width / 2, box.top + box.height / 2);
            let stack = points.slice(Math.max(points.indexOf(el), 0));
            if (!stack.includes(el)) {
                stack = [];
                for (let n = el; n; n = n.parentElement) stack.push(n);
            }
            let image = false;
            const layers = [];
            for (const n of stack) {
                const s = getComputedStyle(n);
                if (s.backgroundImage.split(',').some((layer) => layer.trim() !== 'none')) {
                    image = true;
                    break;
                }
                const c = rgba(s.backgroundColor);
                if (c[3] > 0) layers.push(c);
                if (c[3] >= 1) break;
            }
            if (image) continue;
            const fill = layers.reverse().reduce((under, top) => over(top, under), [255, 255, 255, 1]);
            const value = ratio(over(rgba(getComputedStyle(el).color), fill), fill);
            const need = parseFloat(check.data.expectedContrastRatio) || 4.5;
            if (value < need) {
                unjudged.push({
                    target: node.target,
                    html: node.html.slice(0, 200),
                    ratio: Math.round(value * 100) / 100,
                    need,
                });
            }
        }
    }
    return { violations, unjudged };
}"#;

/// Inject the vendored axe-core once per page.
pub async fn inject_axe(h: &Harness) {
    let present = h
        .page()
        .evaluate("typeof window.axe === 'object' && typeof window.axe.run === 'function'")
        .await
        .ok()
        .and_then(|v| v.into_value::<bool>().ok())
        .unwrap_or(false);
    if present {
        return;
    }
    let path = super::repo_root().join("tests/vendor/axe-core/axe.min.js");
    let script = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read vendored axe-core {}: {e}", path.display()));
    h.page()
        .evaluate(script.as_str())
        .await
        .unwrap_or_else(|e| panic!("inject axe-core: {e}"));
}

/// axe's serious and critical findings on `scope` (a CSS selector, or the
/// whole document), plus the button text it could not judge that measures
/// below AA. See the module docs.
pub async fn axe_findings(h: &Harness, scope: Option<&str>) -> Value {
    run_gate(h, scope, None).await
}

async fn run_gate(h: &Harness, scope: Option<&str>, rules: Option<&[&str]>) -> Value {
    inject_axe(h).await;
    let scope = scope.map_or_else(|| json!(null), |s| json!(s));
    let rules = rules.map_or_else(|| json!(null), |r| json!(r));
    h.page()
        .evaluate(format!("({AXE_GATE_JS})({scope}, {rules})"))
        .await
        .unwrap_or_else(|e| panic!("run axe-core: {e}"))
        .into_value()
        .unwrap_or(Value::Null)
}

/// Fails `context` on any violation of the named axe `rules` on `scope`,
/// whatever its tag, e.g. WCAG 2.2's `target-size`, which the default
/// wcag2a/2aa/21aa run does not include.
pub async fn assert_axe_rules_clean(
    h: &Harness,
    scope: Option<&str>,
    rules: &[&str],
    context: &str,
) {
    let findings = axe_rule_findings(h, scope, rules).await;
    assert!(
        findings["error"].is_null() && findings["violations"].is_array(),
        "{context}: axe did not run: {findings}"
    );
    assert_eq!(
        findings["violations"],
        json!([]),
        "{context}: axe {rules:?} findings: {:#}",
        findings["violations"]
    );
}

/// The named axe `rules` on `scope`: `{ violations, unjudged }`, every
/// violation of those rules whatever its impact.
pub async fn axe_rule_findings(h: &Harness, scope: Option<&str>, rules: &[&str]) -> Value {
    run_gate(h, scope, Some(rules)).await
}

/// Every contrast problem on `scope`: axe's `color-contrast` violations
/// followed by the unjudged button text below AA, as one array (empty when
/// clean), for suites that compare the result across states.
pub async fn contrast_problems(h: &Harness, scope: Option<&str>) -> Value {
    let findings = run_gate(h, scope, Some(&["color-contrast"])).await;
    assert!(
        findings["error"].is_null() && findings["violations"].is_array(),
        "axe did not run: {findings}"
    );
    let mut problems = findings["violations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    problems.extend(findings["unjudged"].as_array().cloned().unwrap_or_default());
    Value::Array(problems)
}

/// Fails `context` on any contrast problem on `scope` (see
/// [`contrast_problems`]).
pub async fn assert_contrast_clean(h: &Harness, scope: Option<&str>, context: &str) {
    let problems = contrast_problems(h, scope).await;
    assert_eq!(
        problems,
        json!([]),
        "{context}: text below AA (axe color-contrast, plus the daisyUI button          text axe filed as unjudged, ldui-3kix): {problems:#}"
    );
}

/// Fails `context` on any serious or critical axe finding, and on any button
/// text below AA that axe filed as unjudged (ldui-3kix).
pub async fn assert_axe_clean(h: &Harness, scope: Option<&str>, context: &str) {
    let findings = axe_findings(h, scope).await;
    assert!(
        findings["error"].is_null() && findings["violations"].is_array(),
        "{context}: axe did not run: {findings}"
    );
    assert_eq!(
        findings["violations"],
        json!([]),
        "{context}: blocking axe findings: {:#}",
        findings["violations"]
    );
    assert_eq!(
        findings["unjudged"],
        json!([]),
        "{context}: text below AA that axe filed as unjudged (a daisyUI button's \
         `none, none` background image, ldui-3kix): {:#}",
        findings["unjudged"]
    );
}
