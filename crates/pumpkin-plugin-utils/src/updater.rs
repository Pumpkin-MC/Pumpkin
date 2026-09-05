//! Non-blocking update checks against the marketplace `/api/v1/rest/check-update` endpoint.

use crate::{
    http::{HttpClient, HttpError},
    models::CheckUpdateResponse,
};
use thiserror::Error;
use tracing::debug;

/// Update checking errors.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Plugin has not been initialized.
    #[error(
        "Plugin-utils has not been initialized (call pumpkin_plugin_utils::init(context) first)"
    )]
    NotInitialized,
    /// HTTP error when querying update endpoint.
    #[error("Failed to query update API: {0}")]
    Http(#[from] HttpError),
    /// JSON parsing error from response.
    #[error("Failed to parse update response JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Checks for plugin updates against the Pumpkin Marketplace API.
pub struct UpdateChecker {
    http_client: HttpClient,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateChecker {
    /// Creates a new `UpdateChecker`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::default(),
        }
    }

    /// Checks if a newer version exists on the marketplace:
    /// `GET /api/v1/rest/check-update?plugin_name={name}&current_version={version}`
    ///
    /// # Errors
    ///
    /// Returns `UpdateError` if the network request or JSON parsing fails.
    pub fn check_for_updates(
        &self,
        plugin_name: &str,
        current_version_str: &str,
        marketplace_url: &str,
    ) -> Result<CheckUpdateResponse, UpdateError> {
        let url = format!(
            "{}/api/v1/rest/check-update?plugin_name={}&current_version={}",
            marketplace_url.trim_end_matches('/'),
            urlencoding(plugin_name),
            urlencoding(current_version_str),
        );
        debug!("Checking for updates at {url}");

        let response_str = self.http_client.get(&url)?;
        let update_response: CheckUpdateResponse = serde_json::from_str(&response_str)?;

        Ok(update_response)
    }
}

/// Minimal URL encoding helper for query parameters.
fn urlencoding(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
