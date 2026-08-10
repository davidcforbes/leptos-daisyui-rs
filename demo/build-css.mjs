// Tailwind build + freshness stamp for the demo's stylesheet (ldui-hun).
//
// WHY THIS EXISTS
// ---------------
// `Trunk.toml`'s `pre_build` hook used to run `npx tailwindcss` directly.
// **Trunk swallows a pre_build hook's non-zero exit**: when a syntax error in
// `input.css` made tailwind exit with `CssSyntaxError`, trunk carried on and
// kept serving the PREVIOUS `output.css`. Both browser audit suites then ran
// against that stale stylesheet and PASSED (`test-style` 7/7, `test-layout`
// 9/9, exit 0) — the entire computed-style audit system reporting green while
// measuring whatever CSS last succeeded rather than the CSS just written.
//
// So the guard cannot be "check tailwind's exit code" — nothing downstream is
// guaranteed to see it. Instead every successful build stamps a **run-unique
// marker** into `output.css`, and the browser harness (`tests/common/mod.rs`)
// refuses to run unless the stylesheet the page actually loaded carries the
// marker of the most recent build. That covers three failure modes, not one:
//
//   1. tailwind failed          -> the stamp file says `fail`, and `output.css`
//                                  still carries the PREVIOUS build's marker.
//   2. `dist/` is stale for any other reason (e.g. a concurrent `trunk build`
//      rewriting `demo/dist` underneath a running suite) -> the served
//      stylesheet's marker is not the current one.
//   3. the served stylesheet is simply not the one just built -> same check.
//
// THE MARKER IS AN UNMATCHABLE RULE, NOT A COMMENT
// ------------------------------------------------
// `#ldui-css-stamp-<token> { --ldui-css-stamp: 1 }`. Two reasons over a CSS
// comment: it survives minification (trunk minifies CSS in release builds,
// which strips comments), and it is readable straight off the CSSOM
// (`document.styleSheets[].cssRules[].selectorText`) with no network fetch —
// comments are not in the CSSOM at all. It is inert by construction: no
// element in the demo has that id, so the rule matches nothing and cannot
// perturb a single computed style. The audit suites' zero-slack ceilings are
// the standing proof of that.
//
// Usage: `node build-css.mjs` from `demo/`. Run by Trunk's pre_build hook and,
// belt-and-braces, by `xtask`'s `DemoServer::start` before it spawns trunk so
// a broken stylesheet fails the gate step immediately instead of after an
// eight-minute wasm build.

import { spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import { appendFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const INPUT = join(here, "input.css");
const OUTPUT = join(here, "output.css");
const STAMP_FILE = join(here, ".ldui-css-stamp");

/** Ident-safe and unique per invocation: `t<ms>x<hex>`. */
function newToken() {
  return `t${Date.now().toString(36)}x${randomUUID().replace(/-/g, "").slice(0, 12)}`;
}

const token = newToken();

// `@tailwindcss/cli` installs `node_modules/.bin/tailwindcss`; on Windows that
// is a `.cmd` shim, which `spawnSync` will only run with `shell: true`.
const isWindows = process.platform === "win32";
const bin = join(here, "node_modules", ".bin", isWindows ? "tailwindcss.cmd" : "tailwindcss");

const result = spawnSync(bin, ["-i", INPUT, "-o", OUTPUT], {
  stdio: "inherit",
  shell: isWindows,
  cwd: here,
});

const failed = result.error != null || result.status !== 0;

if (failed) {
  // Record the failure where the harness will see it EVEN IF our exit code is
  // swallowed. `output.css` is deliberately left alone: rewriting it would
  // destroy the evidence, and its stale marker is itself part of the signal.
  writeFileSync(STAMP_FILE, `fail ${token}\n`, "utf8");
  const why = result.error ? `${result.error.message}` : `exit code ${result.status}`;
  console.error(
    `\nbuild-css: TAILWIND BUILD FAILED (${why}).\n` +
      `build-css: demo/output.css was NOT regenerated and is now STALE.\n` +
      `build-css: trunk will keep serving the previous stylesheet — the browser\n` +
      `build-css: audit suites will refuse to run against it (ldui-hun).\n`,
  );
  process.exit(result.status === 0 || result.status == null ? 1 : result.status);
}

appendFileSync(
  OUTPUT,
  `\n/* freshness stamp — see demo/build-css.mjs (ldui-hun) */\n` +
    `#ldui-css-stamp-${token} { --ldui-css-stamp: 1 }\n`,
  "utf8",
);
writeFileSync(STAMP_FILE, `ok ${token}\n`, "utf8");
console.log(`build-css: stylesheet built and stamped ldui-css-stamp-${token}`);
