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

const violations = [];
const push = (selector, detail) => violations.push({ selector, value: 1.0, detail });

const EXEMPT_CLOSEST = '.menu, .tabs, .dropdown, .modal-backdrop, [data-ld-audit-exempt]';
const NON_FIELD_INPUT_TYPES = ['checkbox', 'radio', 'range', 'hidden'];

const mount = document.querySelector(OPTS.mount_selector);
const all = mount ? [mount, ...mount.querySelectorAll('*')] : [];
const els = all.filter(visible);

for (const el of els) {
  const tag = el.tagName;

  // 1. button-without-btn: a <button> whose classList lacks .btn, unless it
  // sits inside one of the exempt overlay/nav constructs.
  if (tag === 'BUTTON' && !el.classList.contains('btn')) {
    if (!el.closest(EXEMPT_CLOSEST)) {
      push(path(el), 'button-without-btn: raw <button> lacks .btn');
    }
  }

  // 2. table-without-table-class: a <table> whose classList lacks .table.
  if (tag === 'TABLE' && !el.classList.contains('table')) {
    push(path(el), 'table-without-table-class: raw <table> lacks .table');
  }

  // 3. badge-lookalike: a small, rounded, filled, text-bearing <span>/<div>
  // that never opted into .badge (or .btn, which shares the same shape).
  if (tag === 'SPAN' || tag === 'DIV') {
    const s = cs(el);
    const btlr = parseFloat(s.borderTopLeftRadius) || 0;
    const r = el.getBoundingClientRect();
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

return JSON.stringify({ violations, scanned: els.length });
