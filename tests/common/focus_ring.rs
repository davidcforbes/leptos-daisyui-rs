//! The focus ring against the fills beside it (ldui-fq9x, ldui-lhcf).
//!
//! A focus indicator needs 3:1 against the colours adjacent to it (WCAG
//! 1.4.11). An outline has two neighbours: the fill AROUND the control (its
//! parent's background, which the outline is drawn over) and the control's
//! own fill inside it. Primary on the navy table header measured about 1.4:1
//! in 4iiz-Office, first on MonthRoster's week button and then on the
//! table column resize handles.

use pixelproof_web::Harness;
use serde_json::{Value, json};

/// Measures the outline of `selector` in its current state:
/// `{ focused, focusVisible, style, width, color, outer, inner }`, where
/// `outer` is the ring's contrast against the parent's fill and `inner`
/// against the element's own fill, each composited down to the first opaque
/// background.
const RING_JS: &str = r#"(selector) => {
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
    const fillFrom = (start) => {
        const layers = [];
        for (let n = start; n; n = n.parentElement) {
            const c = rgba(getComputedStyle(n).backgroundColor);
            if (c[3] > 0) layers.push(c);
            if (c[3] >= 1) break;
        }
        return layers.reverse().reduce((under, top) => over(top, under), [255, 255, 255, 1]);
    };
    const el = document.querySelector(selector);
    if (!el) return { error: 'no element matches ' + selector };
    const s = getComputedStyle(el);
    const ringOn = (fill) => {
        const ring = over(rgba(s.outlineColor), fill);
        const [x, y] = [lum(ring), lum(fill)].sort((p, q) => q - p);
        return Math.round((x + 0.05) / (y + 0.05) * 100) / 100;
    };
    return {
        focused: document.activeElement === el,
        focusVisible: el.matches(':focus-visible'),
        style: s.outlineStyle,
        width: s.outlineWidth,
        color: s.outlineColor,
        outer: ringOn(fillFrom(el.parentElement)),
        inner: ringOn(fillFrom(el)),
    };
}"#;

/// The ring of `selector` against the fills beside it (see the module docs).
pub async fn ring(h: &Harness, selector: &str) -> Value {
    h.page()
        .evaluate(format!("({RING_JS})({})", json!(selector)))
        .await
        .unwrap_or_else(|e| panic!("measure the focus ring of {selector}: {e}"))
        .into_value()
        .unwrap_or(Value::Null)
}
