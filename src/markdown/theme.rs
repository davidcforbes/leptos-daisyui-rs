//! DaisyUI `data-theme` → editmark `ColorScheme` bridge.
//!
//! Reads `<html data-theme="…">`, maps it to one of editmark's built-in
//! schemes, picks the appropriate dark/light palette, and writes the
//! palette as CSS custom properties scoped to the editmark-leptos root
//! element.  A MutationObserver re-applies the variables whenever the
//! consumer switches themes.
//!
//! The contract:
//! - DaisyUI's `data-theme` is the source of truth for "what theme is active".
//! - editmark-leptos picks the editmark scheme that best matches that
//!   theme's visual identity.
//! - Unknown DaisyUI themes fall back to editmark's Default scheme.
//! - When `data-theme` is absent, `prefers-color-scheme` chooses dark vs light.

use editmark_core::{Color, ColorScheme, Palette, all_builtin_schemes};
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Mode of the picked palette — controls which side of [`ColorScheme`] we use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// Use the scheme's dark palette.
    Dark,
    /// Use the scheme's light palette.
    Light,
}

/// The active editmark theme, in a form a consumer can read reactively.
///
/// Components grab one of these from context (or create one if absent) and
/// use it both to drive their own internal styling decisions and to expose
/// a reactive theme signal to the host application.
#[derive(Clone)]
pub struct ThemeContext {
    /// The currently-resolved editmark scheme (whose palette we render with).
    pub scheme: RwSignal<ColorScheme>,
    /// Dark or light side of the scheme.
    pub mode: RwSignal<ThemeMode>,
}

impl ThemeContext {
    /// Pick the [`Palette`] from the current scheme + mode.
    pub fn palette(&self) -> Palette {
        let scheme = self.scheme.get();
        match self.mode.get() {
            ThemeMode::Dark => scheme.dark,
            ThemeMode::Light => scheme.light,
        }
    }
}

/// Map a DaisyUI theme name to one of editmark's built-in schemes.
///
/// Returns `(scheme_name, mode)`. Unknown themes fall back to
/// `("Default", Light)` — the caller can further override `mode` from
/// `prefers-color-scheme`.
pub fn daisyui_theme_to_scheme(daisy: &str) -> (&'static str, ThemeMode) {
    let normalized = daisy.trim().to_ascii_lowercase();
    match normalized.as_str() {
        // Direct name matches first — these are the lowest-surprise routes.
        "dracula" => ("Dracula", ThemeMode::Dark),
        "nord" => ("Nord", ThemeMode::Dark),
        "winter" => ("Nord", ThemeMode::Light),
        "forest" => ("Gruvbox", ThemeMode::Dark),
        "coffee" => ("Gruvbox", ThemeMode::Dark),
        "autumn" => ("Gruvbox", ThemeMode::Light),
        "synthwave" => ("Tokyo Night", ThemeMode::Dark),
        "night" => ("Tokyo Night", ThemeMode::Dark),
        "cyberpunk" => ("Monokai Pro", ThemeMode::Dark),
        "halloween" => ("Monokai Pro", ThemeMode::Dark),
        "business" => ("One Dark", ThemeMode::Dark),
        "black" => ("One Dark", ThemeMode::Dark),
        "luxury" => ("One Dark", ThemeMode::Dark),
        "valentine" => ("Solarized", ThemeMode::Light),
        "garden" => ("Solarized", ThemeMode::Light),
        "lemonade" => ("Solarized", ThemeMode::Light),
        "aqua" => ("Solarized", ThemeMode::Light),
        // Built-in DaisyUI light/dark land on editmark Default.
        "light" | "cupcake" | "bumblebee" | "emerald" | "corporate" | "retro" | "fantasy"
        | "wireframe" | "lofi" | "pastel" | "cmyk" | "acid" => ("Default", ThemeMode::Light),
        "dark" | "" => ("Default", ThemeMode::Dark),
        // Unknown — Default, mode decided by caller.
        _ => ("Default", ThemeMode::Dark),
    }
}

thread_local! {
    /// The document-lifetime [`ThemeContext`] singleton.
    ///
    /// `use_theme()` may be called from many different, frequently-rebuilt
    /// component subtrees (e.g. every `MessageBubble`/`MarkdownView` in
    /// `AiChat`'s poll loop creates its own). `use_context` alone is not a
    /// safe way to share the context across those calls: context lookups
    /// are scoped to the *reactive owner tree*, and a subtree that doesn't
    /// happen to inherit the context from a still-alive ancestor would fall
    /// through and mint a brand-new `ThemeContext` — including brand-new
    /// signals registered under *that* transient owner. Those signals get
    /// disposed the moment their owning component is torn down, yet the
    /// global `MutationObserver` installed by [`install_theme_observer`]
    /// (deliberately leaked for the document's lifetime) keeps calling
    /// `.set()` on whichever signals it captured first — a disposed-signal
    /// access, which panics and kills the whole reactive runtime.
    ///
    /// Caching the resolved `ThemeContext` here, keyed to nothing but the
    /// thread (i.e. the single-threaded WASM document), makes every caller
    /// — regardless of where in the component tree it lives or how often
    /// its own owner gets rebuilt — observe the exact same signals.
    static THEME_CTX: RefCell<Option<ThemeContext>> = const { RefCell::new(None) };

    /// The root [`Owner`] the singleton's signals are registered under.
    ///
    /// Held here so it is *never dropped*: dropping an `Owner` cleans up
    /// (and thus disposes) every arena node registered while it was the
    /// current owner, which is exactly what would tear the signals above
    /// down.
    static THEME_OWNER: RefCell<Option<Owner>> = const { RefCell::new(None) };
}

/// Build (or reuse) a reactive-graph [`Owner`] that outlives every
/// component: one with **no parent**.
///
/// `Owner::new()` parents itself to whatever `Owner` is "current" on this
/// thread at the moment it's called — normally the calling component's own
/// owner. If `use_theme()` created its signals directly under that owner
/// (or under a plain `Owner::new()` child of it), the signals would be torn
/// down the moment that *calling* component's owner is cleaned up (e.g. a
/// rebuilt `MessageBubble`), because owner cleanup cascades to every
/// descendant owner.
///
/// To avoid inheriting a doomed parent, we briefly clear reactive_graph's
/// notion of the "current owner" (`Owner::unset` only clears that
/// thread-local pointer — it does not run cleanup or otherwise touch the
/// owner), construct our root while nothing is current (so it is
/// genuinely parentless), then restore the caller's owner so we don't
/// disturb its reactive context.
///
/// (`reactive_graph`'s hydration-gated `Owner::new_root` would do this more
/// directly, but this crate only enables the plain `csr` feature, so that
/// constructor isn't available here.)
fn document_root_owner() -> Owner {
    let previous = Owner::current();
    if let Some(previous) = previous.clone() {
        previous.unset();
    }
    let owner = Owner::new();
    if let Some(previous) = previous {
        previous.set();
    }
    owner
}

/// Get-or-init the [`ThemeContext`] singleton, running `init` (under a
/// freshly detached document-root [`Owner`]) only on the very first call.
///
/// Split out from [`use_theme`] so the memoization behavior is unit
/// testable without touching the DOM: `init` is where all
/// `web_sys`/`wasm_bindgen` calls live.
fn get_or_init_theme_ctx(init: impl FnOnce(Owner) -> ThemeContext) -> ThemeContext {
    if let Some(ctx) = THEME_CTX.with(|cell| cell.borrow().clone()) {
        return ctx;
    }

    let owner = document_root_owner();
    let ctx = init(owner.clone());

    // Keep the owner alive forever: dropping it would dispose `ctx`'s
    // signals right back out from under this singleton.
    THEME_OWNER.with(|cell| *cell.borrow_mut() = Some(owner));
    THEME_CTX.with(|cell| *cell.borrow_mut() = Some(ctx.clone()));
    ctx
}

/// Install the global theme observer + base stylesheet if it hasn't been
/// installed already, then return a [`ThemeContext`] reflecting current state.
///
/// Idempotent — calling this many times is safe; the stylesheet, observer,
/// and underlying signals install exactly once per document, no matter how
/// many different component subtrees call this function or how often those
/// subtrees themselves get torn down and rebuilt.
pub fn use_theme() -> ThemeContext {
    let ctx = get_or_init_theme_ctx(|owner| {
        owner.with(|| {
            let (initial_scheme, initial_mode) = resolve_theme_from_dom();
            let scheme = RwSignal::new(initial_scheme);
            let mode = RwSignal::new(initial_mode);

            install_global_styles_once();
            install_theme_observer(scheme, mode);

            ThemeContext { scheme, mode }
        })
    });

    // Also provide via Leptos context for any subtree that looks it up
    // directly with `use_context::<ThemeContext>()`. The thread-local
    // singleton above remains the actual source of truth — this is purely
    // an ergonomic mirror and is never itself read back by `use_theme()`.
    provide_context(ctx.clone());

    ctx
}

/// Read `<html data-theme>` (and, as fallback, `prefers-color-scheme`) and
/// resolve them to a (scheme, mode) pair.
fn resolve_theme_from_dom() -> (ColorScheme, ThemeMode) {
    let document_theme = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|el| el.get_attribute("data-theme"));

    let (name, mut mode) = match document_theme.as_deref() {
        Some(s) => daisyui_theme_to_scheme(s),
        None => ("Default", ThemeMode::Dark),
    };

    // If the DaisyUI theme didn't unambiguously specify dark/light, fall
    // back to the user's OS preference.
    if document_theme.is_none() && prefers_light() {
        mode = ThemeMode::Light;
    }

    let scheme = all_builtin_schemes()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| {
            all_builtin_schemes()
                .into_iter()
                .next()
                .expect("editmark-core ships at least one built-in scheme")
        });
    (scheme, mode)
}

fn prefers_light() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    win.match_media("(prefers-color-scheme: light)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Install a global `<style>` block defining the editmark-leptos base
/// stylesheet.  Called once per document.
fn install_global_styles_once() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    const STYLE_ID: &str = "editmark-leptos-styles";
    if doc.get_element_by_id(STYLE_ID).is_some() {
        return;
    }
    let Ok(style) = doc.create_element("style") else {
        return;
    };
    style.set_id(STYLE_ID);
    style.set_text_content(Some(BASE_STYLES));
    if let Some(head) = doc.head() {
        let _ = head.append_child(&style);
    }
}

/// MutationObserver on `<html>` for the `data-theme` attribute.
fn install_theme_observer(scheme: RwSignal<ColorScheme>, mode: RwSignal<ThemeMode>) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(html) = doc.document_element() else {
        return;
    };
    let cb = Closure::<dyn FnMut(js_sys::Array, web_sys::MutationObserver)>::new(
        move |_records: js_sys::Array, _observer: web_sys::MutationObserver| {
            let (new_scheme, new_mode) = resolve_theme_from_dom();
            scheme.set(new_scheme);
            mode.set(new_mode);
        },
    );
    let Ok(observer) = web_sys::MutationObserver::new(cb.as_ref().unchecked_ref()) else {
        cb.forget();
        return;
    };
    let init = web_sys::MutationObserverInit::new();
    init.set_attributes(true);
    let filter = js_sys::Array::new();
    filter.push(&"data-theme".into());
    init.set_attribute_filter(&filter);
    let _ = observer.observe_with_options(&html, &init);

    // Leak: the observer + closure live for the document's lifetime, which
    // is what we want here.  The host application will tear down the
    // document before this matters.
    cb.forget();
    std::mem::forget(observer);
}

/// Serialize an editmark [`Color`] as a CSS rgb() value.
pub fn color_to_css(c: Color) -> String {
    format!("rgb({} {} {})", c.r, c.g, c.b)
}

/// Build the inline `style="--lds-…: …; …"` declaration for a palette.
///
/// We write CSS custom properties so children can use `var(--lds-text)` etc.
/// without us having to thread the palette through every component.
pub fn palette_style(palette: &Palette) -> String {
    let mut s = String::with_capacity(512);
    let pairs = [
        ("--lds-bg", palette.background),
        ("--lds-text", palette.text),
        ("--lds-heading", palette.heading),
        ("--lds-bold", palette.bold),
        ("--lds-italic", palette.italic),
        ("--lds-code-inline-text", palette.code_inline_text),
        ("--lds-code-inline-bg", palette.code_inline_bg),
        ("--lds-code-block-text", palette.code_block_text),
        ("--lds-code-block-bg", palette.code_block_bg),
        ("--lds-link", palette.link),
        ("--lds-blockquote-bar", palette.blockquote_bar),
        ("--lds-blockquote-text", palette.blockquote_text),
    ];
    for (name, color) in pairs {
        s.push_str(name);
        s.push_str(": ");
        s.push_str(&color_to_css(color));
        s.push_str("; ");
    }
    s
}

/// Base stylesheet — uses CSS custom properties that [`palette_style`] sets.
///
/// Scoped to `.lds-root` so the host application's other styles are unaffected.
/// All visual properties read CSS variables, so a theme switch is a one-shot
/// style attribute rewrite — no rerender required.
const BASE_STYLES: &str = r#"
.lds-root {
    color: var(--lds-text, currentColor);
    background: var(--lds-bg, transparent);
    line-height: 1.55;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif;
    word-wrap: break-word;
}
.lds-root.lds-inline {
    display: inline;
    line-height: inherit;
    background: transparent;
}
.lds-root.lds-inline > p:only-child { display: inline; margin: 0; }
.lds-root h1, .lds-root h2, .lds-root h3,
.lds-root h4, .lds-root h5, .lds-root h6 {
    color: var(--lds-heading, currentColor);
    margin: 1.2em 0 0.6em;
    line-height: 1.25;
    font-weight: 600;
}
.lds-root h1 { font-size: 1.75em; border-bottom: 1px solid rgba(127,127,127,0.2); padding-bottom: 0.2em; }
.lds-root h2 { font-size: 1.4em; border-bottom: 1px solid rgba(127,127,127,0.15); padding-bottom: 0.15em; }
.lds-root h3 { font-size: 1.2em; }
.lds-root p { margin: 0.6em 0; }
.lds-root strong { color: var(--lds-bold, inherit); font-weight: 600; }
.lds-root em { color: var(--lds-italic, inherit); }
.lds-root a { color: var(--lds-link, inherit); text-decoration: underline; text-underline-offset: 2px; }
.lds-root code {
    color: var(--lds-code-inline-text, inherit);
    background: var(--lds-code-inline-bg, rgba(127,127,127,0.12));
    padding: 0.1em 0.35em;
    border-radius: 4px;
    font-family: "JetBrains Mono", "Cascadia Code", Consolas, ui-monospace, monospace;
    font-size: 0.95em;
}
.lds-root pre {
    color: var(--lds-code-block-text, inherit);
    background: var(--lds-code-block-bg, rgba(127,127,127,0.08));
    padding: 0.85em 1em;
    border-radius: 6px;
    overflow: auto;
    margin: 0.8em 0;
    font-size: 0.92em;
    line-height: 1.5;
}
.lds-root pre code {
    background: transparent;
    padding: 0;
    color: inherit;
    font-size: inherit;
}
.lds-root blockquote {
    border-left: 3px solid var(--lds-blockquote-bar, rgba(127,127,127,0.4));
    color: var(--lds-blockquote-text, inherit);
    margin: 0.8em 0;
    padding: 0.2em 0.9em;
    background: rgba(127,127,127,0.04);
    border-radius: 0 4px 4px 0;
}
.lds-root ul, .lds-root ol { padding-left: 1.5em; margin: 0.6em 0; }
.lds-root ul { list-style: disc outside; }
.lds-root ol { list-style: decimal outside; }
.lds-root ul ul, .lds-root ol ul { list-style: circle outside; }
.lds-root ul ul ul, .lds-root ol ol ul { list-style: square outside; }
.lds-root li { margin: 0.2em 0; }
.lds-root li::marker { color: var(--lds-link, currentColor); }
.lds-root table {
    border-collapse: collapse;
    margin: 0.8em 0;
    display: block;
    overflow: auto;
}
.lds-root th, .lds-root td {
    border: 1px solid rgba(127,127,127,0.25);
    padding: 0.4em 0.7em;
}
.lds-root th { background: rgba(127,127,127,0.08); font-weight: 600; }
.lds-root img { max-width: 100%; height: auto; border-radius: 4px; }
.lds-root hr {
    border: 0;
    border-top: 1px solid rgba(127,127,127,0.25);
    margin: 1.2em 0;
}
.lds-root .mermaid { display: flex; justify-content: center; margin: 0.8em 0; }
.lds-root .mermaid svg { max-width: 100%; height: auto; }
/* editmark-mermaid hard-codes #333333 strokes that vanish in dark
   themes.  Override edges + arrowhead strokes/fills only — leave node
   fills and node text alone (they're light by default and the dark
   text-on-light-node combination stays readable in any palette). */
.lds-root .mermaid svg .edge,
.lds-root .mermaid svg .edge-thickness-normal,
.lds-root .mermaid svg .edge-thickness-thick,
.lds-root .mermaid svg .flowchart-link {
    stroke: #c0c8d4 !important;
}
.lds-root .mermaid svg .arrowheadPath,
.lds-root .mermaid svg marker path,
.lds-root .mermaid svg defs marker * {
    stroke: #c0c8d4 !important;
    fill: #c0c8d4 !important;
}
.lds-root .markdown-alert {
    border-left-width: 4px;
    border-left-style: solid;
    padding: 0.6em 1em;
    margin: 0.8em 0;
    background: rgba(127,127,127,0.05);
    border-radius: 0 4px 4px 0;
}
.lds-root .markdown-alert-note { border-left-color: #58a6ff; }
.lds-root .markdown-alert-tip { border-left-color: #3fb950; }
.lds-root .markdown-alert-warning { border-left-color: #d29922; }
.lds-root .markdown-alert-important { border-left-color: #a371f7; }
.lds-root .markdown-alert-caution { border-left-color: #f85149; }
.lds-root .markdown-alert-title { font-weight: 600; margin: 0 0 0.4em; }

/* Editor */
.lds-editor { display: flex; flex-direction: column; gap: 0.5em; }
.lds-editor-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.25em; }
.lds-lint-status {
    margin-left: auto;
    padding: 0 0.5em;
    font-size: 0.85em;
    opacity: 0.75;
    transition: opacity 0.3s ease;
}
.lds-lint-status:empty { opacity: 0; }
.lds-editor-body { display: flex; gap: 0.5em; min-height: 0; }
.lds-editor-body.lds-split > * { flex: 1 1 0; min-width: 0; }
.lds-editor-body.lds-stacked { flex-direction: column; }
.lds-editor-textarea {
    width: 100%;
    min-height: 6em;
    padding: 0.65em 0.85em;
    border: 1px solid rgba(127,127,127,0.25);
    border-radius: 6px;
    font-family: "JetBrains Mono", "Cascadia Code", Consolas, ui-monospace, monospace;
    font-size: 0.95em;
    line-height: 1.5;
    resize: vertical;
    color: inherit;
    background: rgba(127,127,127,0.04);
    box-sizing: border-box;
}
.lds-editor-textarea:focus { outline: 2px solid var(--lds-link, rgba(0,120,215,0.6)); outline-offset: 1px; }
.lds-editor-textarea.lds-dropping {
    outline: 2px dashed var(--lds-link, rgba(0,120,215,0.8));
    outline-offset: 2px;
    background: rgba(0,120,215,0.06);
}
.lds-error-toast {
    position: absolute;
    bottom: 0.8em;
    right: 0.8em;
    padding: 0.5em 0.9em;
    background: #f85149;
    color: white;
    border-radius: 4px;
    font-size: 0.9em;
    box-shadow: 0 2px 12px rgba(0,0,0,0.25);
    z-index: 10;
    max-width: 80%;
    word-wrap: break-word;
}
.lds-editor-preview-pane {
    padding: 0.65em 0.85em;
    border: 1px solid rgba(127,127,127,0.25);
    border-radius: 6px;
    overflow: auto;
}

/* Help overlay */
.lds-help-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
}
.lds-help-dialog {
    background: var(--lds-bg, #1f1f1f);
    color: var(--lds-text, #e6e6e6);
    border: 1px solid rgba(127,127,127,0.3);
    border-radius: 8px;
    padding: 1.2em 1.4em;
    width: min(560px, 92vw);
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 8px 30px rgba(0,0,0,0.4);
}
.lds-help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.8em;
}
.lds-help-title { font-size: 1.1em; font-weight: 600; }
.lds-help-section { margin-bottom: 1em; }
.lds-help-section-title {
    font-size: 0.85em;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    opacity: 0.7;
    margin-bottom: 0.4em;
}
.lds-help-table { width: 100%; border-collapse: collapse; }
.lds-help-table td { padding: 0.25em 0.4em; vertical-align: top; }
.lds-help-keys { width: 38%; white-space: nowrap; }
.lds-help-keys kbd {
    background: rgba(127,127,127,0.15);
    border: 1px solid rgba(127,127,127,0.3);
    border-radius: 3px;
    padding: 0.1em 0.45em;
    font-size: 0.88em;
    font-family: "JetBrains Mono", ui-monospace, monospace;
}
.lds-help-desc { opacity: 0.85; }

/* Outline / TOC */
.lds-outline {
    font-size: 0.9em;
    padding: 0.6em 0.8em;
}
.lds-outline-title {
    font-weight: 600;
    margin-bottom: 0.4em;
    opacity: 0.85;
    font-size: 0.95em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
}
.lds-outline-list { list-style: none; padding: 0; margin: 0; }
.lds-outline-list li { padding: 0.1em 0; }
.lds-outline-list a {
    color: inherit;
    text-decoration: none;
    display: block;
    padding: 0.15em 0.4em;
    border-radius: 3px;
}
.lds-outline-list a:hover {
    background: rgba(127,127,127,0.1);
    color: var(--lds-link, inherit);
}
.lds-outline-h1 { font-weight: 600; }
.lds-outline-h2 a { padding-left: 1.2em; }
.lds-outline-h3 a { padding-left: 2.4em; opacity: 0.85; }
.lds-outline-h4 a { padding-left: 3.6em; opacity: 0.75; }
.lds-outline-h5 a { padding-left: 4.8em; opacity: 0.7; }
.lds-outline-h6 a { padding-left: 6em; opacity: 0.65; }

/* Doc stats bar */
.lds-doc-stats {
    font-size: 0.85em;
    opacity: 0.7;
    padding: 0.35em 0.6em;
    color: inherit;
}

/* Find/Replace overlay */
.lds-find-overlay {
    display: flex;
    flex-direction: column;
    gap: 0.3em;
    padding: 0.5em 0.7em;
    background: rgba(127,127,127,0.07);
    border: 1px solid rgba(127,127,127,0.2);
    border-radius: 6px;
    margin-bottom: 0.5em;
}
.lds-find-row { display: flex; align-items: center; gap: 0.3em; }
.lds-find-input {
    flex: 1 1 0;
    min-width: 0;
    padding: 0.35em 0.55em;
    border: 1px solid rgba(127,127,127,0.3);
    border-radius: 4px;
    background: rgba(127,127,127,0.05);
    color: inherit;
    font-family: inherit;
    font-size: 0.95em;
}
.lds-find-input:focus {
    outline: 2px solid var(--lds-link, rgba(0,120,215,0.6));
    outline-offset: 1px;
}
.lds-find-count {
    font-size: 0.85em;
    opacity: 0.7;
    white-space: nowrap;
    padding: 0 0.4em;
}
.lds-find-toggle-on {
    background: var(--lds-link, rgba(0,120,215,0.6)) !important;
    color: white !important;
}

/* Image dialog */
.lds-image-dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
}
.lds-image-dialog {
    background: var(--lds-bg, #1f1f1f);
    color: var(--lds-text, #e6e6e6);
    border: 1px solid rgba(127,127,127,0.3);
    border-radius: 8px;
    padding: 1.2em;
    min-width: 380px;
    max-width: 90vw;
    box-shadow: 0 8px 30px rgba(0,0,0,0.4);
    display: flex;
    flex-direction: column;
    gap: 0.8em;
}
.lds-image-dialog-title { font-size: 1.1em; font-weight: 600; }
.lds-image-dialog-field { display: flex; flex-direction: column; gap: 0.25em; }
.lds-image-dialog-field > span { font-size: 0.85em; opacity: 0.75; }
.lds-image-dialog-field input[type="text"],
.lds-image-dialog-field input[type="file"] {
    padding: 0.45em 0.6em;
    border: 1px solid rgba(127,127,127,0.3);
    border-radius: 4px;
    background: rgba(127,127,127,0.05);
    color: inherit;
    font-family: inherit;
    font-size: 0.95em;
}
.lds-image-dialog-row { display: flex; gap: 0.6em; }
.lds-image-dialog-row .lds-half { flex: 1 1 0; }
.lds-image-dialog-status {
    min-height: 1.5em;
    font-size: 0.9em;
}
.lds-image-dialog-spinner { opacity: 0.7; }
.lds-image-dialog-error { color: #f85149; }
.lds-image-dialog-ok { color: #3fb950; word-break: break-all; }
.lds-image-dialog-actions { display: flex; justify-content: flex-end; gap: 0.4em; }

/* Syntax highlighting */
.lds-syn-kw { color: #c678dd; }
.lds-syn-str { color: #98c379; }
.lds-syn-num { color: #d19a66; }
.lds-syn-com { color: #7f848e; font-style: italic; }
.lds-syn-fn  { color: #61afef; }
.lds-syn-ty  { color: #e5c07b; }
.lds-syn-op  { color: #56b6c2; }
.lds-syn-pp  { color: #c678dd; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: these tests exercise only the pure reactive-ownership plumbing.
    // Anything that touches `web_sys`/`wasm_bindgen` (e.g.
    // `resolve_theme_from_dom`, `install_global_styles_once`,
    // `install_theme_observer`) panics on a native (non-wasm32) test target
    // — `web_sys::window()` calls `cannot access imported statics on
    // non-wasm targets` — so `use_theme()` itself cannot be unit tested
    // here; that's exactly why the DOM-touching bits are isolated inside
    // the `init` closure passed to `get_or_init_theme_ctx`.

    #[test]
    fn document_root_owner_has_no_parent_even_under_a_caller_owner() {
        let caller = Owner::new();
        caller.with(|| {
            let root = document_root_owner();
            assert!(
                root.parent().is_none(),
                "the document-lifetime owner must never inherit the calling \
                 component's owner as a parent, or it would be disposed \
                 along with that component"
            );
        });
    }

    #[test]
    fn document_root_owner_restores_the_caller_as_current() {
        let caller = Owner::new();
        caller.with(|| {
            let _root = document_root_owner();
            assert_eq!(
                Owner::current(),
                Some(caller.clone()),
                "detaching to build the root owner must not leak: the \
                 caller's owner has to still be current afterward"
            );
        });
    }

    #[test]
    fn use_theme_singleton_shares_signals_across_calls() {
        let ctx1 = get_or_init_theme_ctx(|owner| {
            owner.with(|| ThemeContext {
                scheme: RwSignal::new(
                    all_builtin_schemes()
                        .into_iter()
                        .next()
                        .expect("editmark-core ships at least one built-in scheme"),
                ),
                mode: RwSignal::new(ThemeMode::Dark),
            })
        });

        let ctx2 = get_or_init_theme_ctx(|_owner| {
            panic!("init must not run twice — the singleton should short-circuit");
        });

        // If these are really the same underlying signal, a write through
        // one handle must be visible through the other.
        ctx1.mode.set(ThemeMode::Light);
        assert_eq!(
            ctx2.mode.get(),
            ThemeMode::Light,
            "get_or_init_theme_ctx must return the same signals on every \
             call, not mint a fresh ThemeContext per caller"
        );
    }
}
