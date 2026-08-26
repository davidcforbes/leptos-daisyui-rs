//! Versioned preference serialization and browser storage access.

use super::model::normalize_preferences;
use super::types::{EntityColumn, EntityTablePreferences};

const STORAGE_PREFIX: &str = "ldui-entity-table:";

/// Serializes preferences to deterministic JSON suitable for `localStorage`.
pub fn encode_preferences(
    preferences: &EntityTablePreferences,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(preferences)
}

/// Parses, version-checks, and normalizes stored preferences without panicking.
pub fn decode_preferences<T>(
    payload: &str,
    schema_version: u16,
    columns: &[EntityColumn<T>],
) -> EntityTablePreferences {
    let mut preferences = serde_json::from_str(payload)
        .unwrap_or_else(|_| EntityTablePreferences::new(schema_version));
    normalize_preferences(&mut preferences, schema_version, columns);
    preferences
}

pub(crate) fn load_preferences<T>(
    storage_key: Option<&str>,
    schema_version: u16,
    columns: &[EntityColumn<T>],
) -> EntityTablePreferences {
    let Some(storage_key) = storage_key else {
        return EntityTablePreferences::new(schema_version);
    };
    let Some(storage) = browser_storage() else {
        return EntityTablePreferences::new(schema_version);
    };
    let key = format!("{STORAGE_PREFIX}{storage_key}");
    storage
        .get_item(&key)
        .ok()
        .flatten()
        .map(|payload| decode_preferences(&payload, schema_version, columns))
        .unwrap_or_else(|| EntityTablePreferences::new(schema_version))
}

pub(crate) fn save_preferences(storage_key: Option<&str>, preferences: &EntityTablePreferences) {
    let (Some(storage_key), Some(storage)) = (storage_key, browser_storage()) else {
        return;
    };
    let Ok(payload) = encode_preferences(preferences) else {
        return;
    };
    let key = format!("{STORAGE_PREFIX}{storage_key}");
    let _ = storage.set_item(&key, &payload);
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_storage() -> Option<web_sys::Storage> {
    None
}
