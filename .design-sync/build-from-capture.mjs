// Turns the showcase capture (`cargo xtask capture-design` ->
// target/design-capture/) into the synthetic package the design-sync
// converter builds from, and copies the screenshots into the built bundle.
//
// This repo's components are Rust/Leptos, not React, so the Claude Design
// upload is a tokens-only design system: the REAL compiled stylesheet, plus
// reference pages holding each showcase example's REAL rendered markup and
// screenshot. Nothing here reimplements a component.
//
//   node .design-sync/build-from-capture.mjs pkg      # before package-build
//   node .design-sync/build-from-capture.mjs images   # after package-build
import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync, copyFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';

const ROOT = process.cwd();
const CAP = join(ROOT, 'target', 'design-capture');
const PKG = join(ROOT, '.design-sync', '.cache', 'pkg');
const OUT = join(ROOT, 'ds-bundle');
const MAX_HTML = 12000; // chars of pretty-printed markup per example
const VOID = new Set(['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'source', 'track', 'wbr']);

const manifest = JSON.parse(readFileSync(join(CAP, 'manifest.json'), 'utf8'));
const cfg = JSON.parse(readFileSync(join(ROOT, '.design-sync', 'config.json'), 'utf8'));
const groupDir = (g) => g.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
const pages = manifest.pages.filter((p) => p.page);

function write(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text);
}

// Capture scaffolding is not component output: drop it before publishing.
function clean(html) {
  return html
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/\s(data-cap|data-demo-section|data-demo-section-title)="[^"]*"/g, '');
}

// One tag per line, indented by depth; text stays on its tag's line.
function pretty(html) {
  const tokens = html.match(/<[^>]+>|[^<]+/g) ?? [];
  let depth = 0;
  const lines = [];
  for (const t of tokens) {
    if (t.startsWith('</')) {
      depth = Math.max(0, depth - 1);
      lines.push('  '.repeat(depth) + t);
    } else if (t.startsWith('<')) {
      lines.push('  '.repeat(depth) + t);
      const name = (t.match(/^<([a-zA-Z0-9-]+)/) ?? [])[1]?.toLowerCase();
      if (name && !VOID.has(name) && !t.endsWith('/>')) depth += 1;
    } else if (t.trim()) {
      lines.push('  '.repeat(depth) + t.trim());
    }
  }
  return lines.join('\n');
}

function modulePath(slug) {
  const m = slug.replace(/-/g, '_');
  return existsSync(join(ROOT, 'src', 'components', m)) ? `leptos_daisyui_rs::components::${m}` : null;
}

function buildPkg() {
  rmSync(PKG, { recursive: true, force: true });
  const version = (readFileSync(join(ROOT, 'Cargo.toml'), 'utf8').match(/^version\s*=\s*"([^"]+)"/m) ?? [])[1] ?? '0.0.0';
  write(join(PKG, 'package.json'), JSON.stringify({ name: cfg.pkg, version, main: 'index.js', private: true }, null, 2));
  write(join(PKG, 'index.js'), '// Tokens-only: the components are Rust/Leptos; see guidelines/.\nexport {};\n');

  // The real stylesheet: every sheet the showcase loads, then the runtime
  // preamble <style> blocks it injects (tokens, animations), minus the
  // capture-only freeze sheet.
  const styleDir = join(CAP, 'styles');
  const parts = [];
  for (const f of readdirSync(styleDir).sort()) {
    const text = readFileSync(join(styleDir, f), 'utf8');
    // PixelProof's animation-freeze sheet is capture scaffolding, not the look.
    const inline = f.match(/^inline-(\d+)\.css$/);
    if (inline && manifest.inlineStyles?.[Number(inline[1])]?.['data-pixelproof'] === 'freeze') continue;
    if (!text.trim()) continue;
    parts.push(`/* ---- ${f} (captured from the showcase) ---- */\n${text}`);
  }
  write(join(PKG, 'styles.css'), parts.join('\n\n') + '\n');

  const byGroup = new Map();
  for (const p of pages) {
    const g = groupDir(p.group);
    if (!byGroup.has(p.group)) byGroup.set(p.group, []);
    byGroup.get(p.group).push(p);
    const mod = modulePath(p.slug);
    const lines = [
      `# ${p.name}`,
      '',
      `Showcase page \`${p.href}\` (group: ${p.group}).` + (mod ? ` Leptos module: \`${mod}\`.` : ''),
      '',
      p.page.desc ? `${p.page.desc}\n` : '',
      'Each example below is the markup the real Leptos component rendered in the showcase (light theme),',
      'captured from the live DOM, with its screenshot. Reproduce a design with the same elements and',
      'classes; engineers build it with the Leptos component named above.',
      '',
    ];
    for (const [i, s] of p.page.sections.entries()) {
      const title = s.title || (p.page.sections.length === 1 ? 'Page' : `Example ${i + 1}`);
      lines.push(`## ${title}`, '');
      if (s.png) lines.push(`![${title}](img/${p.slug}-${i}.png)${s.clipped ? ' (screenshot clipped)' : ''}`, '');
      let html = pretty(clean(readFileSync(join(CAP, s.html), 'utf8')));
      if (html.length > MAX_HTML) {
        html = html.slice(0, MAX_HTML).replace(/\n[^\n]*$/, '') + '\n<!-- ...truncated; see the screenshot for the rest -->';
      }
      lines.push('```html', html, '```', '');
    }
    write(join(PKG, 'guides', g, `${p.slug}.md`), lines.join('\n'));
  }

  const index = ['# Component reference', '', 'Real rendered markup and screenshots for every showcase page, by group.', ''];
  for (const [group, list] of byGroup) {
    index.push(`## ${group}`, '');
    for (const p of list) index.push(`- [${p.name}](${groupDir(group)}/${p.slug}.md)`);
    index.push('');
  }
  write(join(PKG, 'guides', 'index.md'), index.join('\n'));
  const failed = manifest.pages.filter((p) => !p.page);
  console.log(`pkg: ${pages.length} pages, ${failed.length} failed captures${failed.length ? ': ' + failed.map((p) => p.slug).join(', ') : ''}`);
}

function copyImages() {
  let n = 0;
  for (const p of pages) {
    for (const [i, s] of p.page.sections.entries()) {
      if (!s.png) continue;
      const dest = join(OUT, 'guidelines', 'guides', groupDir(p.group), 'img', `${p.slug}-${i}.png`);
      mkdirSync(dirname(dest), { recursive: true });
      copyFileSync(join(CAP, s.png), dest);
      n += 1;
    }
  }
  console.log(`images: ${n} screenshots copied into ds-bundle/guidelines/`);
}

const mode = process.argv[2];
if (mode === 'pkg') buildPkg();
else if (mode === 'images') copyImages();
else {
  console.error('usage: node .design-sync/build-from-capture.mjs <pkg|images>');
  process.exit(2);
}
