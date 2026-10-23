import fs from 'node:fs';
import path from 'node:path';
const dir = process.argv[2];
const names = ['mathcore', 'pulldown', 'tex2math', 'l2m', 'katexrs', 'ratex'];
const cases = [
  ['frac', String.raw`x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}`],
  ['align', String.raw`\begin{aligned} a &= b + c \\ d &= e \end{aligned}`],
  ['align_env', String.raw`\begin{align} a &= b \\ c &= d \end{align}`],
  ['pmatrix', String.raw`\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}`],
  ['cases', String.raw`f(x)=\begin{cases} 0 & x<0 \\ 1 & x\ge 0 \end{cases}`],
  ['array', String.raw`\left[\begin{array}{c|c} a & b \\ \hline c & d \end{array}\right]`],
  ['accents', String.raw`\hat{x} + \vec{v} + \bar{y} + \tilde{z} + \dot{a} + \overline{AB}`],
  ['operatorname', String.raw`\operatorname{argmax}_x f(x)`],
  ['fonts', String.raw`\mathbb{R} \mathcal{L} \mathfrak{g} \mathscr{F} \mathbf{v}`],
  ['text', String.raw`x \text{ if } y > 0`],
  ['leftright', String.raw`\left( \frac{a}{b} \right\rangle \middle|`],
  ['color', String.raw`\color{red} x + \textcolor{blue}{y}`],
  ['newcommand', String.raw`\newcommand{\R}{\mathbb{R}} x \in \R`],
  ['sum_int', String.raw`\sum_{i=1}^n \int_0^\infty e^{-x^2} dx \lim_{n\to\infty}`],
  ['binom_overset', String.raw`\binom{n}{k} \overset{!}{=} \underbrace{a+b}_{c}`],
  ['boxed_cancel', String.raw`\boxed{x} \cancel{y}`],
  ['ce', String.raw`\ce{H2O}`],
  ['unknown_cmd', String.raw`\foo{x}`],
  ['unbalanced', String.raw`\frac{a`],
  ['bad_env', String.raw`\begin{matrix} a \end{pmatrix}`],
  ['href_js', String.raw`\href{javascript:alert(1)}{click}`],
  ['url_js', String.raw`\url{javascript:alert(1)}`],
  ['htmlId', String.raw`\htmlId{x}{y} \htmlClass{c}{z} \htmlStyle{color:red}{w}`],
  ['raw_html_text', String.raw`\text{<script>alert(1)</script>} <img src=x onerror=alert(1)> \& "q"`],
  ['includegraphics', String.raw`\includegraphics{http://evil/x.png}`],
  ['macro_loop', String.raw`\newcommand{\a}{\a\a} \a`],
  ['macro_bomb', String.raw`\def\a{xxxxxxxxxx}\def\b{\a\a\a\a\a\a\a\a\a\a}\def\c{\b\b\b\b\b\b\b\b\b\b}\def\d{\c\c\c\c\c\c\c\c\c\c}\def\e{\d\d\d\d\d\d\d\d\d\d}\def\f{\e\e\e\e\e\e\e\e\e\e}\f`],
  ['nest_braces_5k', '{'.repeat(5000) + 'x' + '}'.repeat(5000)],
  ['nest_frac_2k', String.raw`\frac{`.repeat(2000) + 'x' + '}{y}'.repeat(2000)],
  ['nest_sup_5k', 'x^{'.repeat(5000) + 'y' + '}'.repeat(5000)],
  ['nest_left_2k', String.raw`\left(`.repeat(2000) + 'x' + String.raw`\right)`.repeat(2000)],
  ['nest_sqrt_10k', String.raw`\sqrt{`.repeat(10000) + 'x' + '}'.repeat(10000)],
];
const only = process.argv[3] ? process.argv[3].split(',') : names;
for (const n of only) {
  const buf = fs.readFileSync(path.join(dir, `sz_${n}.wasm`));
  const mod = new WebAssembly.Module(buf);
  for (const [label, src] of cases) {
    // fresh instance per case so a trap does not poison later cases
    let inst;
    try { inst = new WebAssembly.Instance(mod, {}); } catch (e) { console.log(`${n}\t${label}\tINSTANTIATE_FAIL ${e.message}`); break; }
    const { memory, alloc, conv, out_ptr } = inst.exports;
    const bytes = new TextEncoder().encode(src);
    const p = alloc(bytes.length);
    new Uint8Array(memory.buffer, p, bytes.length).set(bytes);
    const t0 = performance.now();
    try {
      const len = conv(p, bytes.length, 1);
      const op = out_ptr();
      const out = new TextDecoder().decode(new Uint8Array(memory.buffer, op, len));
      const ms = (performance.now() - t0).toFixed(1);
      const flags = [];
      if (/javascript:/i.test(out)) flags.push('JS_URL');
      if (/href=/i.test(out)) flags.push('HREF');
      if (/<script/i.test(out)) flags.push('RAW_SCRIPT');
      if (/<img/i.test(out)) flags.push('RAW_IMG');
      if (/onerror=/i.test(out) && !/&lt;img/.test(out)) flags.push('ONERROR');
      if (/ id="/i.test(out)) flags.push('ID');
      if (/ style="/i.test(out)) flags.push('STYLE');
      if (/merror|math-core-error|katex-error|color:#b22222|mathcolor="#b22222"|mathcolor="red"/i.test(out)) flags.push('ERRMARK');
      const head = out.startsWith('ERR ') ? out.slice(0, 160) : out.replace(/\s+/g, ' ').slice(0, 140);
      console.log(`${n}\t${label}\tOK len=${len} ${ms}ms [${flags.join(',')}]\t${head}`);
    } catch (e) {
      console.log(`${n}\t${label}\tTRAP ${(performance.now() - t0).toFixed(1)}ms ${e.constructor.name}: ${e.message}`);
    }
  }
}
