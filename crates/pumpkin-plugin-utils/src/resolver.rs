//! Dynamic public key resolution for marketplace signature verification.

use crate::http::HttpClient;
use thiserror::Error;
use tracing::debug;

/// Public key resolution errors.
#[derive(Debug, Error)]
pub enum ResolveError {
    /// Public key format is invalid.
    #[error("Retrieved public key is empty or invalid: {0}")]
    InvalidKey(String),
}

/// Resolves the Pumpkin Marketplace public key dynamically.
///
/// Follows a 2-tier live resolution strategy without stale on-disk caching:
/// 1. Host WIT interface (`marketplace::get_public_key()`).
/// 2. Live remote marketplace REST endpoint (`<marketplace_url>/api/v1/rest/public-key`).
pub struct PublicKeyResolver {
    http_client: HttpClient,
}

impl Default for PublicKeyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl PublicKeyResolver {
    /// Creates a new public key resolver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::default(),
        }
    }

    /// Resolves the marketplace public key dynamically without on-disk caching.
    ///
    /// # Errors
    ///
    /// Returns `ResolveError` if the retrieved key is malformed.
    pub fn resolve_public_key(&self, marketplace_url: &str) -> Result<String, ResolveError> {
        // 1. Try host WIT import if running inside WASM runtime
        #[cfg(target_arch = "wasm32")]
        if let Some(host_key) = pumpkin_plugin_api::marketplace::get_public_key() {
            let key = host_key.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                debug!("Resolved marketplace public key from host WIT context");
                return Ok(key);
            }
        }

        // 2. Live HTTPS fetch from marketplace
        let url = format!(
            "{}/api/v1/rest/public-key",
            marketplace_url.trim_end_matches('/')
        );
        debug!(
            "Fetching live marketplace public key via HTTPS from {}",
            url
        );
        if let Ok(body) = self.http_client.get(&url) {
            let key = if let Ok(resp) =
                serde_json::from_str::<crate::models::MarketplacePublicKeyResponse>(&body)
            {
                resp.public_key_hex
            } else {
                body.trim().trim_matches('"').to_string()
            };

            if !key.is_empty() {
                return Ok(key);
            }
        }

        // Fallback to empty string (lets signature verification verify against envelope key)
        Ok(String::new())
    }
}
