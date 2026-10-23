import fs from 'node:fs';
import path from 'node:path';
const dir = process.argv[2];
function mk(n) {
  const mod = new WebAssembly.Module(fs.readFileSync(path.join(dir, `sz_${n}.wasm`)));
  return (src) => {
    const inst = new WebAssembly.Instance(mod, {});
    const { memory, alloc, conv, out_ptr } = inst.exports;
    const b = new TextEncoder().encode(src);
    const p = alloc(b.length);
    new Uint8Array(memory.buffer, p, b.length).set(b);
    try {
      const len = conv(p, b.length, 1);
      return { ok: true, out: new TextDecoder().decode(new Uint8Array(memory.buffer, out_ptr(), len)) };
    } catch (e) { return { ok: false, out: e.message }; }
  };
}
const gens = {
  braces: (d) => '{'.repeat(d) + 'x' + '}'.repeat(d),
  frac: (d) => String.raw`\frac{`.repeat(d) + 'x' + '}{y}'.repeat(d),
  sup: (d) => 'x^{'.repeat(d) + 'y' + '}'.repeat(d),
};
// 1) trap thresholds by binary search
for (const n of ['mathcore', 'l2m', 'katexrs', 'pulldown']) {
  const f = mk(n);
  for (const [g, gen] of Object.entries(gens)) {
    let lo = 1, hi = 20000;
    if (f(gen(hi)).ok) { console.log(`THRESH ${n} ${g} >=${hi} (no trap)`); continue; }
    while (hi - lo > 1) { const m = (lo + hi) >> 1; if (f(gen(m)).ok) lo = m; else hi = m; }
    console.log(`THRESH ${n} ${g} max_ok_depth=${lo} trap_at=${hi}`);
  }
}
// 2) pulldown timing scaling
{
  const f = mk('pulldown');
  for (const d of [500, 1000, 2000, 4000]) { const t = performance.now(); f(String.raw`\sqrt{`.repeat(d) + 'x' + '}'.repeat(d)); console.log(`PDTIME sqrt ${d} ${(performance.now() - t).toFixed(0)}ms`); }
}
// 3) vocabulary of tags/attrs on a broad corpus
const corpus = [
  String.raw`x = \frac{-b \pm \sqrt[3]{b^2-4ac}}{2a}`, String.raw`\begin{aligned} a &= b \\ c &= d \end{aligned}`,
  String.raw`\begin{align} a &= b \\ c &= d \end{align}`, String.raw`\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}`,
  String.raw`\begin{cases} 0 & x<0 \\ 1 \end{cases}`, String.raw`\left[\begin{array}{c|c} a & b \\ \hline c & d \end{array}\right]`,
  String.raw`\hat{x}\vec{v}\bar{y}\widetilde{z}\overline{AB}\underline{q}`, String.raw`\operatorname{argmax}_x \mathbb{R}\mathcal{L}\mathfrak{g}\mathbf{v}`,
  String.raw`x \text{ if } y`, String.raw`\color{red} x \textcolor{blue}{y} \colorbox{yellow}{z}`, String.raw`\sum_{i=1}^n \int_0^\infty \lim_{n\to\infty} \prod`,
  String.raw`\binom{n}{k}\overset{!}{=}\underbrace{a+b}_{c}\overbrace{d}^{e}`, String.raw`\boxed{x}\cancel{y}\sout{z}`, String.raw`\hspace{1cm} \quad \, \! \phantom{x}`,
  String.raw`\tag{1} x`, String.raw`\xrightarrow{abc} \stackrel{a}{b} \mathrm{d}x`, String.raw`a<b \& c > d`, String.raw`{}_a^b X_c^d \prescript{a}{b}{X}`,
  String.raw`\displaystyle \frac12 \tfrac{1}{2} \dfrac{1}{2}`, String.raw`\big( \Big[ \bigg\{ \Bigg\langle`,
];
for (const n of ['mathcore', 'pulldown', 'tex2math', 'l2m', 'katexrs', 'ratex']) {
  const f = mk(n); const tags = new Set(), attrs = new Set(); let raw_lt = 0;
  for (const c of corpus) {
    const r = f(c); if (!r.ok || r.out.startsWith('ERR ')) continue;
    for (const m of r.out.matchAll(/<([a-zA-Z][\w:-]*)([^>]*)>/g)) { tags.add(m[1]); for (const a of m[2].matchAll(/([\w:-]+)\s*=\s*"/g)) attrs.add(a[1]); }
    if (/<mo><<\/mo>|<mo>&<\/mo>/.test(r.out)) raw_lt++;
  }
  console.log(`VOCAB ${n} tags=${[...tags].sort().join(',')}\n      attrs=${[...attrs].sort().join(',')} unescaped_lt_amp=${raw_lt}`);
}
// 4) escaping probe
for (const n of ['mathcore', 'pulldown', 'tex2math', 'l2m']) { const r = mk(n)(String.raw`a<b \& c`); console.log(`ESC ${n} ${r.out.slice(0, 200)}`); }
