//! Tauri commands for secure API key management via OS keychain.

use crate::keyring;

/// Store an API key in the system keyring.
#[tauri::command]
#[specta::specta]
pub fn set_api_key(provider_id: String, api_key: String) -> Result<(), String> {
    keyring::set_api_key(&provider_id, &api_key)
}

/// Get an API key from the system keyring.
/// Returns None if the key doesn't exist.
#[tauri::command]
#[specta::specta]
pub fn get_api_key(provider_id: String) -> Result<Option<String>, String> {
    keyring::get_api_key(&provider_id)
}

/// Delete an API key from the system keyring.
#[tauri::command]
#[specta::specta]
pub fn delete_api_key(provider_id: String) -> Result<(), String> {
    keyring::delete_api_key(&provider_id)
}

/// Check if an API key exists in the system keyring.
#[tauri::command]
#[specta::specta]
pub fn has_api_key(provider_id: String) -> bool {
    keyring::has_api_key(&provider_id)
}

/// Get a masked version of the API key for display (e.g., "sk-••••••••xxxx").
/// Returns None if no key exists.
#[tauri::command]
#[specta::specta]
pub fn get_masked_api_key(provider_id: String) -> Result<Option<String>, String> {
    match keyring::get_api_key(&provider_id)? {
        Some(key) => {
            if key.len() <= 8 {
                Ok(Some("••••••••".to_string()))
            } else {
                let last_four = &key[key.len() - 4..];
                Ok(Some(format!("••••••••{}", last_four)))
            }
        }
        None => Ok(None),
    }
}
