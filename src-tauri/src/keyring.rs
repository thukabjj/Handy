//! OS Keychain integration for secure API key storage.
//!
//! Uses the system's native credential storage:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (via D-Bus)

use keyring::Entry;
use log::{debug, error, warn};

const SERVICE_NAME: &str = "com.pais.handy";

/// Set an API key in the system keyring.
pub fn set_api_key(provider_id: &str, key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(key)
        .map_err(|e| format!("Failed to store API key: {}", e))?;

    debug!("Stored API key for provider: {}", provider_id);
    Ok(())
}

/// Get an API key from the system keyring.
pub fn get_api_key(provider_id: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(key) => {
            debug!("Retrieved API key for provider: {}", provider_id);
            Ok(Some(key))
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No API key found for provider: {}", provider_id);
            Ok(None)
        }
        Err(e) => {
            error!("Failed to get API key for {}: {}", provider_id, e);
            Err(format!("Failed to retrieve API key: {}", e))
        }
    }
}

/// Delete an API key from the system keyring.
pub fn delete_api_key(provider_id: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => {
            debug!("Deleted API key for provider: {}", provider_id);
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No API key to delete for provider: {}", provider_id);
            Ok(())
        }
        Err(e) => {
            warn!("Failed to delete API key for {}: {}", provider_id, e);
            Err(format!("Failed to delete API key: {}", e))
        }
    }
}

/// Check if an API key exists in the system keyring.
pub fn has_api_key(provider_id: &str) -> bool {
    get_api_key(provider_id)
        .map(|opt| opt.is_some())
        .unwrap_or(false)
}

/// Migrate API keys from plaintext settings to keyring.
/// Returns the provider IDs that were successfully migrated.
#[allow(dead_code)]
pub fn migrate_api_keys(keys: &std::collections::HashMap<String, String>) -> Vec<String> {
    let mut migrated = Vec::new();

    for (provider_id, key) in keys {
        if key.is_empty() {
            continue;
        }

        // Check if already in keyring
        if has_api_key(provider_id) {
            debug!(
                "API key for {} already in keyring, skipping migration",
                provider_id
            );
            continue;
        }

        match set_api_key(provider_id, key) {
            Ok(()) => {
                migrated.push(provider_id.clone());
                debug!("Migrated API key for {} to keyring", provider_id);
            }
            Err(e) => {
                error!("Failed to migrate API key for {}: {}", provider_id, e);
            }
        }
    }

    migrated
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a working keyring on the system.
    // They may fail in CI environments without proper setup.

    #[test]
    #[ignore = "Requires system keyring"]
    fn test_set_and_get_api_key() {
        let test_provider = "test_provider_handy";
        let test_key = "test_api_key_12345";

        // Set
        set_api_key(test_provider, test_key).expect("Failed to set key");

        // Get
        let retrieved = get_api_key(test_provider)
            .expect("Failed to get key")
            .expect("Key not found");
        assert_eq!(retrieved, test_key);

        // Delete
        delete_api_key(test_provider).expect("Failed to delete key");

        // Verify deleted
        let after_delete = get_api_key(test_provider).expect("Failed to check key");
        assert!(after_delete.is_none());
    }
}
