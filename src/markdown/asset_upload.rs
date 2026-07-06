//! Asset upload contract — the bridge from the editor to llm-wiki's
//! `POST /v1/assets` endpoint (or any other backend the consumer wires up).
//!
//! Design:
//! - The editor doesn't know how to talk to your backend.  It hands you
//!   bytes, filename, and content-type, then awaits the URL you give back.
//! - You wrap your `async fn upload(...)` in an [`AssetUploader`] and pass
//!   it as the `on_asset_upload` prop on [`crate::markdown::MarkdownEditor`].
//! - On each picker / paste / drop, the editor calls
//!   `uploader(request).await` and inserts `![filename](<returned url>)` at
//!   the cursor.  On error, the editor surfaces the error string in a small
//!   inline notice but does not insert anything.
//!
//! The wrapped function must be `Send + Sync` so it satisfies Leptos's
//! reactive runtime bounds.  In a wasm-only consumer this is trivially
//! satisfied — closures over `String`, `web_sys::*`, and similar are
//! `Send + Sync` by default.  The returned future is *not* required to be
//! `Send`; `wasm-bindgen-futures::spawn_local` runs it on the JS microtask
//! queue without any thread crossing.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// The payload handed to the consumer's upload function.
#[derive(Debug, Clone)]
pub struct AssetUploadRequest {
    /// Raw bytes of the asset.
    pub bytes: Vec<u8>,
    /// Original filename (or a synthesized one for clipboard pastes, e.g.
    /// `"clipboard-image.png"`). Useful as a default `alt` text.
    pub filename: String,
    /// MIME type (e.g. `"image/png"`).  Inferred from the picker/paste/drop
    /// source; not validated against the server's allowlist on this side.
    pub content_type: String,
}

/// A future-returning callback that knows how to upload one asset.
pub type UploadFuture = Pin<Box<dyn Future<Output = Result<String, String>> + 'static>>;

/// Wrapper around the consumer's async upload function.
///
/// Cheaply cloneable (`Arc` inside); `Send + Sync` so it satisfies Leptos's
/// reactive bounds when passed as a component prop.
#[derive(Clone)]
pub struct AssetUploader(Arc<dyn Fn(AssetUploadRequest) -> UploadFuture + Send + Sync + 'static>);

impl AssetUploader {
    /// Build an uploader from an async closure.
    ///
    /// ```ignore
    /// let uploader = AssetUploader::new(|req| async move {
    ///     // Your fetch/XHR/whatever, returning the URL string.
    ///     Ok(format!("/v1/assets/{}", uuid::Uuid::new_v4()))
    /// });
    /// ```
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(AssetUploadRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, String>> + 'static,
    {
        Self(Arc::new(move |req| Box::pin(f(req))))
    }

    /// Invoke the uploader.  Returns the pinned future the consumer's
    /// closure produced.
    pub fn upload(&self, req: AssetUploadRequest) -> UploadFuture {
        (self.0)(req)
    }
}

impl std::fmt::Debug for AssetUploader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AssetUploader(<closure>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uploader_round_trips_request() {
        let uploader = AssetUploader::new(|req: AssetUploadRequest| async move {
            Ok(format!("/v1/assets/{}-{}", req.filename, req.bytes.len()))
        });
        let fut = uploader.upload(AssetUploadRequest {
            bytes: vec![1, 2, 3, 4, 5],
            filename: "hello.png".into(),
            content_type: "image/png".into(),
        });
        // Drop the future without awaiting — this test runs on a native
        // target with no wasm executor; we only care that wrapping a
        // closure into an AssetUploader doesn't drop the inner Fn.
        drop(fut);
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn uploader_is_send_sync() {
        assert_send_sync::<AssetUploader>();
    }
}
