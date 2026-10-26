//! Markdown rendering components, built on the canonical `editmark-core`
//! pipeline (`build_layout` → `render_html`), with Mermaid drawn by pele in
//! the browser (see [`crate::pele`]).
//!
//! Moved here from editmark-leptos so any Leptos/daisyUI app can render
//! markdown with the same view-only renderer, syntax highlighter, link
//! sanitizer, math (native MathML via the Rust math-core crate, see
//! [`math`]; behind the `math` cargo feature since ldui-dtm3, with
//! [`currency`] keeping prices out of math in every build), and DaisyUI
//! `data-theme` → palette bridge.
//! editmark-leptos now consumes these from here and re-exports them for its
//! existing consumers (e.g. llm-wiki), so its public surface is unchanged.
//!
//! Styling is scoped to the `.lds-root` class via the stylesheet injected by
//! [`theme::use_theme`]; the `.lds-*` namespace is already unique (daisyUI uses
//! Tailwind utility classes), so there's no collision with the rest of the
//! component library.

pub mod asset_upload;
pub mod currency;
#[allow(missing_docs)]
pub mod editor;
#[allow(missing_docs)]
pub mod editor_api_leptos;
pub mod file_io;
#[allow(missing_docs)]
pub mod graphic_editor;
#[allow(missing_docs)]
pub mod ime_state;
#[allow(missing_docs)]
pub mod paste_normalizer;
#[allow(missing_docs)]
pub mod table_ui;
// These migrated editor-support components carry many self-descriptive state
// and `#[component]` prop fields; doc-enforcement is relaxed for them rather
// than annotating every macro-generated prop. The other modules below keep the
// crate-wide `#![warn(missing_docs)]` policy.
#[allow(missing_docs)]
pub mod find;
#[allow(missing_docs)]
pub mod find_overlay;
#[allow(missing_docs)]
pub mod help_overlay;
pub mod highlight;
#[allow(missing_docs)]
pub mod image_dialog;
pub mod image_parse;
pub mod inline;
#[cfg(feature = "math")]
pub mod math;
/// The `math` feature is off (the default since ldui-dtm3: math-core 0.9.0
/// needs Rust 1.96 and 4iiz-Office builds on 1.95), so a `.math-inline` or
/// `.math-display` host keeps its `$…$` source as visible text and nothing
/// converts it. [`currency`] still decides which `$…$` pairs are math at all.
#[cfg(not(feature = "math"))]
pub mod math {
    use web_sys::HtmlElement;

    /// No-op without the `math` feature: every formula stays as source text.
    pub fn render_math(_host: &HtmlElement) {}

    /// No-op without the `math` feature: every formula stays as source text.
    pub fn render_math_display_only(_host: &HtmlElement) {}
}
pub mod outline;
pub mod sanitize;
pub mod stats;
pub mod theme;
pub mod view;

pub use asset_upload::{AssetUploadRequest, AssetUploader, UploadFuture};
pub use editor::{Length, MarkdownEditor, Mode, PreviewMode, ToolbarPreset};
pub use editor_api_leptos::SignalEditorApi;
pub use graphic_editor::MarkdownGraphicEditor;
pub use highlight::highlight_to_html;
pub use image_parse::{ImageForm, ImageRef};
pub use inline::MarkdownInline;
pub use math::render_math;
pub use outline::{DocOutline, OutlineEntry};
pub use sanitize::is_safe_href;
pub use stats::{DocStatsBar, DocStatsFields};
pub use theme::{
    ThemeContext, ThemeMode, color_to_css, daisyui_theme_to_scheme, palette_style, use_theme,
};
pub use view::MarkdownView;
