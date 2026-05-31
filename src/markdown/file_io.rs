//! Helpers for reading bytes out of `web_sys::File` / clipboard / drop
//! sources, asynchronously.
//!
//! `FileReader::read_as_array_buffer` is event-driven; we wrap it in a
//! `js_sys::Promise` so the editor's `spawn_local` upload path can `.await`
//! it normally.

use js_sys::{ArrayBuffer, Promise, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileReader};

/// Read a [`File`]'s entire contents into a `Vec<u8>`.
pub async fn read_file_bytes(file: &File) -> Result<Vec<u8>, String> {
    let reader = FileReader::new().map_err(|_| "FileReader unavailable".to_string())?;

    let reader_for_promise = reader.clone();
    let promise = Promise::new(&mut |resolve, reject| {
        let reader_load = reader_for_promise.clone();
        let resolve_load = resolve.clone();
        let onload = Closure::<dyn FnMut()>::new(move || {
            let result = reader_load.result().unwrap_or(JsValue::NULL);
            let _ = resolve_load.call1(&JsValue::NULL, &result);
        });
        reader_for_promise.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        let onerror = Closure::<dyn FnMut()>::new(move || {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("FileReader error"));
        });
        reader_for_promise.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
    });

    reader
        .read_as_array_buffer(file)
        .map_err(|_| "read_as_array_buffer failed".to_string())?;

    let result = JsFuture::from(promise)
        .await
        .map_err(|_| "FileReader rejected".to_string())?;
    let array_buffer: ArrayBuffer = result
        .dyn_into()
        .map_err(|_| "FileReader returned non-ArrayBuffer".to_string())?;
    let uint8 = Uint8Array::new(&array_buffer);
    Ok(uint8.to_vec())
}
