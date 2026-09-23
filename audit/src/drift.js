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

// Hard cap, mirroring the engine's sweep (`pixelproof-style-audit`'s
// `sweep.rs`): without one a pathological page — a form-heavy consumer screen
// with thousands of unlabelled inputs — builds a multi-megabyte JSON string
// and the CDP round-trip appears to hang. All six rules feed the single
// `component-drift` family, so one list shares the engine's per-family cap.
// Truncation is reported, never silent.
const MAX_PER_CATEGORY = 200;
const violations = [];
let truncated = false;
const push = (selector, detail) => {
  if (violations.length < MAX_PER_CATEGORY) violations.push({ selector, value: 1.0, detail });
  else truncated = true;
};

const EXEMPT_CLOSEST = '.menu, .tabs, .dropdown, .modal-backdrop, [data-ld-audit-exempt]';
// Rule 6 only. A control disabled by STATE rather than a withheld action has
// nothing to explain: the pager's current page and its boundary arrows
// ("you are here"), a busy control, and the already-selected option of a
// single-select group. Everything else that is disabled needs a reason.
const DISABLED_STATE_EXEMPT =
  '[data-pagination], [aria-current="page"], .loading, [aria-busy="true"], ' +
  '[aria-checked="true"], [aria-pressed="true"], [aria-selected="true"]';
const NON_FIELD_INPUT_TYPES = ['checkbox', 'radio', 'range', 'hidden'];

const mount = document.querySelector(OPTS.mount_selector);
const all = mount ? [mount, ...mount.querySelectorAll('*')] : [];
const els = all.filter(visible);

for (const el of els) {
  const tag = el.tagName;

  // 1. button-without-btn: a <button> whose classList lacks .btn, unless it
  // sits inside one of the exempt overlay/nav constructs, or declares itself
  // a designed unstyled action via the `data-pressable` marker — emitted by
  // the library's Pressable primitive, so intent lives in the markup rather
  // than in an exemption comment. Genuinely raw, unmarked buttons still flag.
  if (tag === 'BUTTON' && !el.classList.contains('btn')) {
    if (!el.closest(EXEMPT_CLOSEST) && !el.hasAttribute('data-pressable')) {
      push(path(el), 'button-without-btn: raw <button> lacks .btn');
    }
  }

  // 2. table-without-table-class: a <table> whose classList lacks .table.
  //
  // `.sr-only` is exempt. This rule exists to catch a table that LOOKS
  // undesigned, and a screen-reader-only table has no appearance to get
  // wrong -- every chart in this library ships one as its accessible data
  // equivalent, so without the exemption a consumer's audit reports one
  // finding per chart for markup they neither wrote nor can see. That is the
  // same failure ldui-zl58 documented for the nav rail: our own markup
  // drowning a consumer's real signal.
  if (
    tag === 'TABLE' &&
    !el.classList.contains('table') &&
    !el.classList.contains('sr-only')
  ) {
    push(path(el), 'table-without-table-class: raw <table> lacks .table');
  }

  // 3. badge-lookalike: a small, rounded, filled, text-bearing <span>/<div>
  // that never opted into .badge (or .btn, which shares the same shape).
  if (tag === 'SPAN' || tag === 'DIV') {
    const s = cs(el);
    const r = el.getBoundingClientRect();
    // Computed radius is "Npx", "N%", or "X Y" for elliptical. A percentage
    // resolves against the box, exactly as the engine's shape rule does
    // (`sweep.rs`) — `parseFloat("50%")` would otherwise read as 50px and
    // mis-flag a small round chip.
    const rawRadius = (s.borderTopLeftRadius || '').split(' ')[0];
    let btlr;
    if (rawRadius.endsWith('%')) {
      btlr = (parseFloat(rawRadius) / 100) * Math.min(r.width, r.height);
    } else {
      btlr = parseFloat(rawRadius);
    }
    if (isNaN(btlr)) btlr = 0;
    const bg = s.backgroundColor;
    const hasOwnText = Array.from(el.childNodes)
      .some(n => n.nodeType === 3 && n.textContent.trim());
    if (
      btlr >= 8 &&
      r.height > 0 && r.height <= 24 &&
      bg !== 'rgba(0, 0, 0, 0)' &&
      hasOwnText &&
      !el.classList.contains('badge') &&
      !el.classList.contains('btn')
    ) {
      push(path(el), 'badge-lookalike: pill-shaped text chip lacks .badge');
    }
  }

  // 4. input-outside-field: a text-ish <input>/<select>/<textarea> with none
  // of fieldset ancestor, wrapping label ancestor, or a matching label[for]
  // in the document.
  const isFieldCandidate =
    (tag === 'INPUT' && !NON_FIELD_INPUT_TYPES.includes((el.type || 'text').toLowerCase())) ||
    tag === 'SELECT' || tag === 'TEXTAREA';
  if (isFieldCandidate) {
    const hasFieldset = el.closest('fieldset') !== null;
    const hasWrappingLabel = el.closest('label') !== null;
    let hasLabelFor = false;
    if (el.id) {
      hasLabelFor = document.querySelector(
        'label[for="' + el.id.replace(/"/g, '\\"') + '"]'
      ) !== null;
    }
    if (!hasFieldset && !hasWrappingLabel && !hasLabelFor) {
      push(
        path(el),
        'input-outside-field: ' + tag.toLowerCase() +
        ' has no fieldset ancestor, wrapping label, or label[for]'
      );
    }
  }
}

// 5. target-too-small: WCAG 2.2 SC 2.5.8 Target Size (Minimum), level AA.
// axe-core does not run it by default (it sits outside axe's default tags),
// so a consumer's clean axe pass says nothing about it (4iiz-Office op-7ndp7:
// a primary per-row action measured 18px tall on every row).
//
// The SC is size OR spacing, and this implements both halves rather than a
// bare size check, which would flag every correctly spaced compact control:
// a target under 24x24 CSS px passes when a 24px-diameter circle centred on
// its bounding box intersects no other target and no other undersized
// target's circle. The SC's inline exception (a link in a sentence) is out of
// scope because plain links are not collected -- only buttons, `.btn`-styled
// links and ARIA buttons, which is where a dense UI's action targets live.
const TARGET_MIN = 24;
const RADIUS = TARGET_MIN / 2;
const targets = els.filter(el => {
  if (!(el.tagName === 'BUTTON' || el.matches('a.btn[href], [role="button"]'))) return false;
  if (el.disabled || el.closest('[aria-hidden="true"], [inert], [data-ld-audit-exempt]')) return false;
  const r = el.getBoundingClientRect();
  // `.sr-only` and zero-size plumbing are 1px boxes nobody can aim at.
  return r.width > 1 && r.height > 1;
}).map(el => {
  const r = el.getBoundingClientRect();
  return {
    el, r,
    cx: r.left + r.width / 2,
    cy: r.top + r.height / 2,
    small: r.width < TARGET_MIN - 0.5 || r.height < TARGET_MIN - 0.5,
  };
});
// A target nested in another target (an icon <span role="button"> inside a
// <button>) is one hit area, not two neighbours.
const related = (a, b) => a.el.contains(b.el) || b.el.contains(a.el);
const circleHitsRect = (t, r) => {
  const dx = Math.max(r.left - t.cx, 0, t.cx - r.right);
  const dy = Math.max(r.top - t.cy, 0, t.cy - r.bottom);
  return dx * dx + dy * dy < RADIUS * RADIUS;
};
for (const t of targets) {
  if (!t.small) continue;
  const crowded = targets.some(o => {
    if (o === t || related(o, t)) return false;
    if (circleHitsRect(t, o.r)) return true;
    return o.small && Math.hypot(o.cx - t.cx, o.cy - t.cy) < TARGET_MIN;
  });
  if (crowded) {
    push(
      path(t.el),
      'target-too-small: ' + Math.round(t.r.width) + 'x' + Math.round(t.r.height) +
      'px target is under 24x24 and a neighbour sits inside its 24px spacing circle (WCAG 2.5.8)'
    );
  }
}

// 6. disabled-without-reason (ldui-p82h): a disabled control -- a native
// `<button disabled>`, anything carrying daisyUI's `.btn-disabled`, or an
// ARIA button with `aria-disabled="true"` -- must still have an accessible
// name (a disabled control is still announced, and an icon-only one with no
// name is an unlabelled dead end) AND an `aria-describedby` that resolves to
// non-empty text saying WHY it is disabled. The library's `Button` mints
// both from its `disabled_reason` prop, and stamps
// `data-disabled-without-reason="true"` when it was rendered `disabled=true`
// without a reason, so that marker is reported verbatim in the finding
// wherever it appears -- it names the fix (pass `disabled_reason`).
//
// The name is the accname computation's first three steps, not the whole
// algorithm: `aria-label`, then `aria-labelledby`, then the element's own
// text with `aria-hidden` subtrees excluded (the Button's reason hint is one,
// deliberately, so a reason can never masquerade as a name), then `title`.
// A description resolves through `aria-describedby` only; `title` is not
// accepted as a description because it is not announced on every platform.
const ownText = el => {
  let out = '';
  const walk = n => {
    for (const c of n.childNodes) {
      if (c.nodeType === 3) out += c.textContent;
      else if (c.nodeType === 1 && c.getAttribute('aria-hidden') !== 'true') walk(c);
    }
  };
  walk(el);
  return out.replace(/\s+/g, ' ').trim();
};
const idrefsText = (el, attr) => (el.getAttribute(attr) || '')
  .split(/\s+/)
  .filter(Boolean)
  .map(id => { const t = document.getElementById(id); return t ? t.textContent.trim() : ''; })
  .filter(Boolean)
  .join(' ');
for (const el of els) {
  const isDisabled =
    (el.tagName === 'BUTTON' && el.disabled) ||
    el.classList.contains('btn-disabled') ||
    (el.matches('[role="button"]') && el.getAttribute('aria-disabled') === 'true');
  if (!isDisabled) continue;
  if (el.closest(EXEMPT_CLOSEST) || el.closest('[aria-hidden="true"], [inert]')) continue;
  if (el.closest(DISABLED_STATE_EXEMPT)) continue;
  const name =
    (el.getAttribute('aria-label') || '').trim() ||
    idrefsText(el, 'aria-labelledby') ||
    ownText(el) ||
    (el.getAttribute('title') || '').trim();
  const reason = idrefsText(el, 'aria-describedby');
  const marker = el.hasAttribute('data-disabled-without-reason')
    ? ' [data-disabled-without-reason]'
    : '';
  if (!name) {
    push(path(el), 'disabled-without-reason: disabled control has no accessible name' + marker);
  } else if (!reason) {
    push(
      path(el),
      'disabled-without-reason: disabled control has no aria-describedby resolving to non-empty text' + marker
    );
  }
}

return JSON.stringify({ violations, scanned: els.length, truncated });
