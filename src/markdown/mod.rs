//! Markdown rendering components, built on the canonical `editmark-core`
//! pipeline (`build_layout` → `render_html`) plus `editmark-mermaid`.
//!
//! Moved here from editmark-leptos so any Leptos/daisyUI app can render
//! markdown with the same view-only renderer, syntax highlighter, link
//! sanitizer, KaTeX math hook, and DaisyUI `data-theme` → palette bridge.
//! editmark-leptos now consumes these from here and re-exports them for its
//! existing consumers (e.g. llm-wiki), so its public surface is unchanged.
//!
//! Styling is scoped to the `.lds-root` class via the stylesheet injected by
//! [`theme::use_theme`]; the `.lds-*` namespace is already unique (daisyUI uses
//! Tailwind utility classes), so there's no collision with the rest of the
//! component library.

pub mod asset_upload;
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
pub mod math;
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
    color_to_css, daisyui_theme_to_scheme, palette_style, use_theme, ThemeContext, ThemeMode,
};
pub use view::MarkdownView;
