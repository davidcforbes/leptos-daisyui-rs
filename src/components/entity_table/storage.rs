//! Versioned preference serialization and browser storage access.

use super::model::normalize_preferences;
use super::types::{EntityColumn, EntityTablePreferencePersistence, EntityTablePreferences};

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
    let preferences = serde_json::from_str(payload)
        .unwrap_or_else(|_| EntityTablePreferences::new(schema_version));
    normalize_preferences(&preferences, schema_version, columns)
}

pub(crate) fn load_preferences<T>(
    persistence: EntityTablePreferencePersistence,
    schema_version: u16,
    columns: &[EntityColumn<T>],
) -> EntityTablePreferences {
    load_preferences_with(persistence, schema_version, columns, |key| {
        browser_storage()?.get_item(key).ok().flatten()
    })
}

pub(crate) fn load_preferences_with<T>(
    persistence: EntityTablePreferencePersistence,
    schema_version: u16,
    columns: &[EntityColumn<T>],
    read: impl FnOnce(&str) -> Option<String>,
) -> EntityTablePreferences {
    let EntityTablePreferencePersistence::LegacyLocalStorage { storage_key } = persistence else {
        return EntityTablePreferences::new(schema_version);
    };
    let key = format!("{STORAGE_PREFIX}{storage_key}");
    read(&key)
        .map(|payload| decode_preferences(&payload, schema_version, columns))
        .unwrap_or_else(|| EntityTablePreferences::new(schema_version))
}

pub(crate) fn save_preferences(
    persistence: EntityTablePreferencePersistence,
    preferences: &EntityTablePreferences,
) {
    save_preferences_with(persistence, preferences, |key, payload| {
        if let Some(storage) = browser_storage() {
            let _ = storage.set_item(key, payload);
        }
    });
}

pub(crate) fn save_preferences_with(
    persistence: EntityTablePreferencePersistence,
    preferences: &EntityTablePreferences,
    write: impl FnOnce(&str, &str),
) {
    let EntityTablePreferencePersistence::LegacyLocalStorage { storage_key } = persistence else {
        return;
    };
    let Ok(payload) = encode_preferences(preferences) else {
        return;
    };
    let key = format!("{STORAGE_PREFIX}{storage_key}");
    write(&key, &payload);
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_storage() -> Option<web_sys::Storage> {
    None
}
