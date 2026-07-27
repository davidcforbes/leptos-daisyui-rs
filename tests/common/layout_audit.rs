//! Layout audit (ldui-dg2) — the article's manual "10-minute spacing audit"
//! turned into a permanent assertion, web half.
//!
//! The desktop half (`d2d_ui::layout_audit`, `GET /api/overlaps`) walks a
//! semantic tree; this walks the real DOM through `getBoundingClientRect`.
//! The assertion logic is deliberately the same shape on both faces.
//!
//! **One important difference, in the web's favour.** The desktop suite
//! carries a loud caveat: a control's semantic rect is a *declaration*, not a
//! *measurement*, so a clean sweep there is a ratchet against new declared-box
//! collisions rather than proof the pixels are fine. `getBoundingClientRect`
//! is an actual post-layout measurement, so that caveat does not transfer —
//! an overlap reported here is an overlap that really renders.
//!
//! Three checks, per `ldui-dg2`:
//!
//! 1. **Overlap** — no two visible in-flow siblings may intersect. The
//!    regression test for the whole class of bug where a component grows and
//!    its neighbour does not move.
//! 2. **Grid** — every gap between vertically adjacent siblings must be on
//!    the canonical scale.
//! 3. **Internal <= external** — a container's padding must not exceed the
//!    gap to its siblings, or the containers visually merge (Gestalt
//!    proximity; this is what `ldui-6qb` then fixes).

#![allow(dead_code)]

use serde::Deserialize;

/// The canonical spacing scale, mirrored from `ui_tokens::spacing::SCALE`.
/// A unit test below pins the two together so this cannot drift.
pub const CANONICAL: [f64; 9] = [4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0];

/// Sub-pixel tolerance. Browsers lay out in fractional pixels (a flex item can
/// land on 15.99px), and a fractional-DPI viewport shifts everything slightly.
/// Anything within half a pixel of a scale step counts as on the step.
pub const EPSILON: f64 = 0.5;

/// A single reported violation.
#[derive(Debug, Clone, Deserialize)]
pub struct Violation {
    /// CSS-ish path to the offending element, for a human to go look at.
    pub selector: String,
    /// The measured value in px (a gap, an inset, or an overlap area).
    pub value: f64,
    /// Free-text detail — the sibling involved, the padding compared, etc.
    pub detail: String,
}

/// Everything the in-page sweep found on one page.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditReport {
    pub overlaps: Vec<Violation>,
    pub grid: Vec<Violation>,
    pub internal: Vec<Violation>,
    /// How many elements the sweep actually considered — a sanity check that
    /// the page really rendered rather than the selector matching nothing.
    pub scanned: usize,
    /// Whether a cap was hit, so the counts are a floor rather than a total.
    /// Reported, never silent: a truncated sweep that looked complete would
    /// read as "we checked everything" when it did not.
    #[serde(default)]
    pub truncated: bool,
}

impl AuditReport {
    pub fn total(&self) -> usize {
        self.overlaps.len() + self.grid.len() + self.internal.len()
    }

    /// Render the report for a failing assertion message.
    pub fn describe(&self, page: &str) -> String {
        let mut s = format!(
            "layout audit for {page}: {} scanned, {} overlap, {} off-grid, {} internal>external\n",
            self.scanned,
            self.overlaps.len(),
            self.grid.len(),
            self.internal.len()
        );
        for (label, list) in [
            ("OVERLAP", &self.overlaps),
            ("OFF-GRID", &self.grid),
            ("INTERNAL>EXTERNAL", &self.internal),
        ] {
            for v in list.iter().take(20) {
                s.push_str(&format!(
                    "  {label}  {:>8.2}px  {}  {}\n",
                    v.value, v.selector, v.detail
                ));
            }
            if list.len() > 20 {
                s.push_str(&format!("  {label}  ... and {} more\n", list.len() - 20));
            }
        }
        s
    }
}

/// Whether `value` sits on the canonical scale, within [`EPSILON`].
///
/// Zero is always acceptable: flush-adjacent elements are a deliberate layout,
/// not a missing gap.
pub fn on_scale(value: f64) -> bool {
    if value.abs() < EPSILON {
        return true;
    }
    CANONICAL.iter().any(|s| (s - value).abs() < EPSILON)
}

/// The in-page sweep, injected verbatim by the harness.
///
/// Kept as one self-contained expression so it can be handed straight to
/// `page.evaluate` with no build step.
///
/// Exemptions, mirroring the desktop contract plus two the DOM forces:
/// - invisible or zero-area elements
/// - out-of-flow elements (`position: absolute|fixed|sticky`) — an overlay is
///   *supposed* to cover its siblings
/// - anything inside an open `<dialog>`, or a daisyUI dropdown/modal
/// - inline boxes, whose rects legitimately overlap line boxes
pub const SWEEP_JS: &str = r#"
(() => {
  const CANON = [4, 8, 12, 16, 24, 32, 48, 64, 96];
  // ui_tokens::typography::LINE_RAMP.
  const LINE_RAMP = [36, 28, 24, 20, 16, 16];
  const EPS = 0.5;
  const onScale = v => Math.abs(v) < EPS || CANON.some(s => Math.abs(s - v) < EPS);
  // Whether a top-to-top pitch is a step on the line-height ramp. Two 14px
  // text rows set at a 20px (LINE_BODY) pitch leave a 6px gap: 6 is off the
  // spacing scale, but nobody chose it — it falls out of correct, grid-aligned
  // typography. Flagging it would train people to ignore the checker.
  // (Finding handed over from the desktop half, which hit this first.)
  const onLineRamp = pitch => LINE_RAMP.some(l => Math.abs(l - pitch) < EPS);

  const path = el => {
    const bits = [];
    for (let n = el; n && n.nodeType === 1 && bits.length < 4; n = n.parentElement) {
      let b = n.tagName.toLowerCase();
      if (n.id) { bits.unshift(b + '#' + n.id); break; }
      const cls = (n.getAttribute('class') || '').trim().split(/\s+/).filter(Boolean).slice(0, 2);
      if (cls.length) b += '.' + cls.join('.');
      bits.unshift(b);
    }
    return bits.join(' > ');
  };

  const styleOf = new Map();
  const cs = el => {
    let s = styleOf.get(el);
    if (!s) { s = getComputedStyle(el); styleOf.set(el, s); }
    return s;
  };

  const visible = el => {
    const s = cs(el);
    if (s.display === 'none' || s.visibility === 'hidden') return false;
    if (parseFloat(s.opacity) === 0) return false;
    const r = el.getBoundingClientRect();
    return r.width > 0.5 && r.height > 0.5;
  };

  // Out-of-flow elements are exempt: overlays are meant to cover things.
  const inFlow = el => {
    const p = cs(el).position;
    return p === 'static' || p === 'relative';
  };

  // Inline boxes overlap line boxes legitimately.
  const blockish = el => {
    const d = cs(el).display;
    return d !== 'inline' && d !== 'contents';
  };

  const exemptSubtree = el =>
    el.closest('dialog[open], .modal-open, .dropdown-open, [role="dialog"], [aria-modal="true"]') !== null;

  // daisyUI `join` collapses the shared borders of adjacent items by pulling
  // them together, so joined buttons overlap by exactly the border width.
  // That is the component working, not a collision — measured on
  // /components/data-table it accounted for every single reported overlap.
  const joined = el => el.parentElement && el.parentElement.classList.contains('join');

  // Hard caps. Without them a page with a pathological number of hits builds
  // a multi-megabyte JSON string and the CDP round-trip appears to hang —
  // which is exactly what happened before these were added. Truncation is
  // reported rather than silent: `truncated` tells the caller the numbers are
  // a floor, not a total.
  const MAX_PER_CATEGORY = 200;
  const MAX_SIBLINGS = 400;
  const out = { overlaps: [], grid: [], internal: [], scanned: 0, truncated: false };
  const push = (list, v) => {
    if (list.length < MAX_PER_CATEGORY) list.push(v);
    else out.truncated = true;
  };

  const parents = new Set();
  for (const el of document.querySelectorAll('main, main *')) {
    if (el.parentElement) parents.add(el.parentElement);
  }

  for (const parent of parents) {
    if (exemptSubtree(parent)) continue;
    const kids = Array.from(parent.children).filter(
      k => visible(k) && inFlow(k) && blockish(k) && !exemptSubtree(k)
    );
    if (!kids.length) continue;
    out.scanned += kids.length;
    // O(k^2) pairwise below; a parent with thousands of children (a long
    // virtualised list) would dominate the whole sweep for no extra signal.
    if (kids.length > MAX_SIBLINGS) { out.truncated = true; continue; }

    const rects = kids.map(k => k.getBoundingClientRect());

    // --- 1. overlap: no two visible in-flow siblings may intersect --------
    for (let i = 0; i < kids.length; i++) {
      for (let j = i + 1; j < kids.length; j++) {
        const a = rects[i], b = rects[j];
        const ox = Math.min(a.right, b.right) - Math.max(a.left, b.left);
        const oy = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
        if (ox > EPS && oy > EPS) {
          // Full containment is a nesting artefact, not a collision.
          const contains = (p, q) =>
            p.left <= q.left + EPS && p.right >= q.right - EPS &&
            p.top <= q.top + EPS && p.bottom >= q.bottom - EPS;
          if (contains(a, b) || contains(b, a)) continue;
          if (joined(kids[i]) && joined(kids[j])) continue;
          push(out.overlaps, {
            selector: path(kids[i]),
            value: ox * oy,
            detail: 'intersects ' + path(kids[j]) + ' by ' +
                    ox.toFixed(1) + 'x' + oy.toFixed(1) + 'px'
          });
        }
      }
    }

    // Vertical stacks only: siblings laid out one above the next. A row of
    // flex items has no vertical gap to speak of and its horizontal gap is
    // the flex `gap`, already token-driven.
    const stacked = kids
      .map((k, i) => ({ k, r: rects[i] }))
      .sort((p, q) => p.r.top - q.r.top);

    // --- 2. grid: vertical gaps land on the canonical scale ---------------
    for (let i = 1; i < stacked.length; i++) {
      const prev = stacked[i - 1].r, cur = stacked[i].r;
      // Only compare things actually stacked, not side-by-side.
      const horizontallyApart = cur.left >= prev.right - EPS || prev.left >= cur.right - EPS;
      if (horizontallyApart) continue;
      const gap = cur.top - prev.bottom;
      if (gap < -EPS) continue;              // handled by the overlap check
      if (!onScale(gap)) {
        push(out.grid, {
          selector: path(stacked[i].k),
          value: gap,
          detail: 'gap below ' + path(stacked[i - 1].k)
        });
      }
    }

    // --- 3a. a child's padding vs the gap between children ----------------
    // The direction the desktop audit's Kanban finding took: a card with 12px
    // of its own padding sitting in a list with an 8px gap groups more
    // strongly with the container edge than with the list it belongs to.
    // Only meaningful when the children are visually distinct surfaces —
    // a transparent wrapper has no edge to merge with anything.
    if (stacked.length > 1) {
      let minGap = Infinity;
      for (let i = 1; i < stacked.length; i++) {
        const g = stacked[i].r.top - stacked[i - 1].r.bottom;
        if (g >= -EPS && g < minGap) minGap = g;
      }
      if (minGap > EPS && minGap !== Infinity) {
        for (const { k } of stacked) {
          const ks = cs(k);
          const surface =
            ks.backgroundColor !== 'rgba(0, 0, 0, 0)' ||
            parseFloat(ks.borderTopWidth) > 0 ||
            ks.boxShadow !== 'none';
          if (!surface) continue;
          const kpad = Math.max(
            parseFloat(ks.paddingTop) || 0,
            parseFloat(ks.paddingBottom) || 0
          );
          if (kpad > minGap + EPS) {
            push(out.internal, {
              selector: path(k),
              value: kpad,
              detail: 'own padding ' + kpad.toFixed(1) +
                      'px > ' + minGap.toFixed(1) + 'px gap between siblings'
            });
          }
        }
      }
    }

    // --- 3b. a container's padding vs the gap to ITS siblings -------------
    // A container whose own padding exceeds the gap to its neighbours
    // visually merges with them (Gestalt proximity).
    const ps = cs(parent);
    const padTop = parseFloat(ps.paddingTop) || 0;
    const padBottom = parseFloat(ps.paddingBottom) || 0;
    const pad = Math.max(padTop, padBottom);
    if (pad > 0 && parent.parentElement) {
      const sibs = Array.from(parent.parentElement.children)
        .filter(s => s !== parent && visible(s) && inFlow(s));
      const pr = parent.getBoundingClientRect();
      let nearest = Infinity;
      for (const s of sibs) {
        const sr = s.getBoundingClientRect();
        const g = sr.top >= pr.bottom ? sr.top - pr.bottom
                : pr.top >= sr.bottom ? pr.top - sr.bottom
                : Infinity;
        if (g < nearest) nearest = g;
      }
      // A nearest gap of exactly 0 means the siblings are deliberately
      // flush — stacked sections of one surface (card-body above
      // card-actions, a kanban column's list above its footer). They are not
      // competing visual groups, so proximity has nothing to disambiguate and
      // the rule does not apply.
      if (nearest > EPS && nearest !== Infinity && pad > nearest + EPS) {
        push(out.internal, {
          selector: path(parent),
          value: pad,
          detail: 'padding ' + pad.toFixed(1) + 'px > gap to nearest sibling ' +
                  nearest.toFixed(1) + 'px'
        });
      }
    }
  }
  return JSON.stringify(out);
})()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ramp_mirror_matches_ui_tokens() {
        // The sweep carries LINE_RAMP as a JS literal; if the shared ramp
        // moves, the typography exemption silently stops matching.
        let shared: Vec<String> = ui_tokens::typography::LINE_RAMP
            .iter()
            .map(|v| format!("{}", *v as i64))
            .collect();
        let js = format!("const LINE_RAMP = [{}];", shared.join(", "));
        assert!(
            SWEEP_JS.contains(&js),
            "sweep LINE_RAMP drifted; expected: {js}"
        );
    }

    #[test]
    fn canonical_scale_matches_ui_tokens() {
        // If the shared scale moves, this file must move with it.
        let shared: Vec<f64> = ui_tokens::spacing::SCALE
            .iter()
            .map(|&v| v as f64)
            .collect();
        assert_eq!(CANONICAL.to_vec(), shared);
    }

    #[test]
    fn zero_gap_is_on_scale() {
        // Flush-adjacent elements are a layout choice, not a missing gap.
        assert!(on_scale(0.0));
        assert!(on_scale(0.2));
    }

    #[test]
    fn scale_steps_are_on_scale() {
        for step in CANONICAL {
            assert!(on_scale(step), "{step} should be on the scale");
        }
    }

    #[test]
    fn subpixel_drift_still_counts_as_on_scale() {
        // Browsers lay out in fractional pixels; 15.7px is a 16px gap.
        assert!(on_scale(15.7));
        assert!(on_scale(16.3));
    }

    #[test]
    fn off_scale_values_are_rejected() {
        for v in [6.0, 14.0, 20.0, 40.0, 17.0] {
            assert!(!on_scale(v), "{v} should be off the scale");
        }
    }

    #[test]
    fn sweep_js_is_a_single_expression() {
        // It is handed straight to `page.evaluate`; a stray statement would
        // make it evaluate to undefined and the deserialize would fail with
        // a far less obvious message.
        let js = SWEEP_JS.trim();
        assert!(js.starts_with("(() => {"), "sweep must be an IIFE");
        assert!(js.ends_with("})()"), "sweep must be invoked");
    }

    #[test]
    fn report_describes_every_category() {
        let r = AuditReport {
            overlaps: vec![Violation {
                selector: "div.card".into(),
                value: 42.0,
                detail: "intersects div.card".into(),
            }],
            grid: vec![Violation {
                selector: "p".into(),
                value: 6.0,
                detail: "gap below h2".into(),
            }],
            internal: vec![],
            scanned: 12,
            truncated: false,
        };
        let s = r.describe("/components/card");
        assert!(s.contains("OVERLAP"));
        assert!(s.contains("OFF-GRID"));
        assert!(s.contains("12 scanned"));
        assert_eq!(r.total(), 2);
    }
}
