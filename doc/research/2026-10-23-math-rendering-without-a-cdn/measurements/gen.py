import os
root = os.path.dirname(os.path.abspath(__file__))

ws = """[workspace]
resolver = "2"
members = ["base", "mathcore", "pulldown", "l2m", "katexrs", "tex2math", "ratex"]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
"""
open(os.path.join(root, "Cargo.toml"), "w").write(ws)

prelude = r'''
use std::sync::Mutex;
static OUT: Mutex<String> = Mutex::new(String::new());
#[unsafe(no_mangle)]
pub extern "C" fn alloc(n: usize) -> *mut u8 { let mut v = Vec::<u8>::with_capacity(n); let p = v.as_mut_ptr(); std::mem::forget(v); p }
fn input<'a>(p: *const u8, n: usize) -> &'a str { unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(p, n)) } }
fn store(s: String) -> usize { let l = s.len(); *OUT.lock().unwrap() = s; l }
#[unsafe(no_mangle)]
pub extern "C" fn out_ptr() -> *const u8 { OUT.lock().unwrap().as_ptr() }
'''

crates = {
 "base": ("", r'''
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let out = if d == 1 { format!("<math display=\"block\"><mtext>{}</mtext></math>", s) } else { format!("<math><mtext>{}</mtext></math>", s) };
    store(out)
}
'''),
 "mathcore": ('math-core = { version = "=0.9.0", default-features = false }', r'''
use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let c = LatexToMathML::new(MathCoreConfig::default(), vec![]).unwrap();
    let disp = if d == 1 { MathDisplay::Block } else { MathDisplay::Inline };
    match c.convert_with_local_state(s, disp) {
        Ok(r) => store(r.mathml),
        Err(e) => store(format!("ERR {}", e.error_message())),
    }
}
'''),
 "pulldown": ('pulldown-latex = "=0.8.0"', r'''
use pulldown_latex::{mathml::push_mathml, Parser, RenderConfig, Storage};
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let storage = Storage::new();
    // collect errors first
    let errs: Vec<String> = Parser::new(s, &storage).filter_map(|e| e.err()).map(|e| e.to_string()).collect();
    if !errs.is_empty() { return store(format!("ERR {}", errs.join("; "))); }
    let storage2 = Storage::new();
    let parser = Parser::new(s, &storage2);
    let mut out = String::new();
    let mut cfg = RenderConfig::default();
    if d == 1 { cfg.display_mode = pulldown_latex::config::DisplayMode::Block; }
    push_mathml(&mut out, parser, cfg).unwrap();
    store(out)
}
'''),
 "l2m": ('latex2mathml = "=0.2.3"', r'''
use latex2mathml::{latex_to_mathml, DisplayStyle};
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let st = if d == 1 { DisplayStyle::Block } else { DisplayStyle::Inline };
    match latex_to_mathml(s, st) { Ok(m) => store(m), Err(e) => store(format!("ERR {}", e)) }
}
'''),
 "katexrs": ('katex-rs = { version = "=0.3.0", default-features = false }', r'''
use katex::{render_to_string, KatexContext, Settings};
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let ctx = KatexContext::default();
    let settings = Settings::builder().display_mode(d == 1).build();
    match render_to_string(&ctx, s, &settings) { Ok(m) => store(m), Err(e) => store(format!("ERR {}", e)) }
}
'''),
 "tex2math": ('tex2math = { version = "=2.4.1", default-features = false }', r'''
use tex2math::{convert, ConvertOptions};
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let o = if d == 1 { ConvertOptions::display() } else { ConvertOptions::default() };
    match convert(s, &o) { Ok(m) => store(m), Err(e) => store(format!("ERR {}", e)) }
}
'''),
 "ratex": ('ratex-parser = "=0.1.15"\nratex-layout = "=0.1.15"\nratex-svg = { version = "=0.1.15", default-features = false }', r'''
#[unsafe(no_mangle)]
pub extern "C" fn conv(p: *const u8, n: usize, d: u32) -> usize {
    let s = input(p, n);
    let nodes = match ratex_parser::parse(s) { Ok(v) => v, Err(e) => return store(format!("ERR {}", e)) };
    let mut opts = ratex_layout::LayoutOptions::default();
    let _ = d;
    let b = ratex_layout::layout(&nodes, &opts);
    let dl = ratex_layout::to_display_list(&b);
    let _ = &mut opts;
    store(ratex_svg::render_to_svg(&dl, &ratex_svg::SvgOptions::default()))
}
'''),
}

for name, (dep, body) in crates.items():
    toml = f"""[package]
name = "sz-{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
{dep}
"""
    open(os.path.join(root, name, "Cargo.toml"), "w").write(toml)
    open(os.path.join(root, name, "src", "lib.rs"), "w").write(prelude + body)
print("written")
