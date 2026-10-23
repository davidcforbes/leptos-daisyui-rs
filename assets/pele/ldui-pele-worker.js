/*
 * leptos-daisyui-rs pele worker (ldui-o55o).
 *
 * Runs pele.render off the page's UI thread. pele draws synchronously, and
 * a hostile diagram can keep it busy for many seconds, so the page posts one
 * render at a time here and terminates this worker when it overruns its
 * time limit (src/pele.rs, PELE_RENDER_TIMEOUT_MS).
 *
 * Serve this file next to pele.global.js from the page's own origin: under a
 * CSP with script-src 'self' a same-origin worker file is allowed and a blob:
 * worker is not. importScripts resolves pele.global.js relative to this file.
 *
 * Message in:  { id, source, options }
 * Message out: { id, ok: true, svg, width, height }
 *          or  { id, ok: false, error: { message, code, line, column, snippet } }
 */
'use strict';

importScripts('pele.global.js');

// pele can throw values that are not PeleErrors (a plain TypeError from
// object-valued YAML), and a hostile value's toString can itself throw, so
// every read is guarded.
function text(read, fallback) {
  try {
    var value = read();
    return typeof value === 'string' ? value : fallback;
  } catch (_) {
    return fallback;
  }
}

function count(read) {
  try {
    var value = Number(read());
    return isFinite(value) && value >= 0 ? Math.floor(value) : 0;
  } catch (_) {
    return 0;
  }
}

self.onmessage = function (event) {
  var job = event.data || {};
  try {
    var out = self.pele.render(String(job.source), job.options || {});
    self.postMessage({
      id: job.id,
      ok: true,
      svg: String(out.svg),
      width: Number(out.width) || 0,
      height: Number(out.height) || 0
    });
  } catch (thrown) {
    var message = text(function () { return thrown.message; }, '');
    if (!message) {
      message = text(function () { return String(thrown); }, 'unknown error');
    }
    self.postMessage({
      id: job.id,
      ok: false,
      error: {
        message: message,
        code: text(function () { return thrown.code; }, 'render') || 'render',
        line: count(function () { return thrown.line; }),
        column: count(function () { return thrown.column; }),
        snippet: text(function () { return thrown.snippet; }, '')
      }
    });
  }
};
