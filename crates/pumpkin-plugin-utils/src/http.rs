//! HTTP client helpers for online license checks and marketplace queries.

use thiserror::Error;

/// HTTP request errors.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Network or connection error.
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    /// Response status code was not successful (2xx).
    #[error("HTTP response returned error status {0}: {1}")]
    BadStatus(u16, String),
    /// Error reading response body.
    #[error("Failed to read HTTP response body: {0}")]
    BodyRead(String),
}

/// Helper client for querying Pumpkin marketplace REST APIs.
pub struct HttpClient {
    user_agent: String,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new("Pumpkin-Plugin-Utils/0.1.0")
    }
}

impl HttpClient {
    /// Creates a new HTTP client with the specified User-Agent header.
    #[must_use]
    pub fn new(user_agent: &str) -> Self {
        Self {
            user_agent: user_agent.to_string(),
        }
    }

    /// Performs an HTTP GET request and returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or returns a non-2xx status code.
    pub fn get(&self, url: &str) -> Result<String, HttpError> {
        let mut response = ureq::get(url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .call()
            .map_err(|e| HttpError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status < 200 || status >= 300 {
            let body = response
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::BadStatus(status, body));
        }

        response
            .body_mut()
            .read_to_string()
            .map_err(|e| HttpError::BodyRead(e.to_string()))
    }

    /// Performs an HTTP POST request with a JSON payload and returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or returns a non-2xx status code.
    pub fn post_json(&self, url: &str, json_payload: &str) -> Result<String, HttpError> {
        let mut response = ureq::post(url)
            .header("User-Agent", &self.user_agent)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send(json_payload)
            .map_err(|e| HttpError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status < 200 || status >= 300 {
            let body = response
                .body_mut()
                .read_to_string()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::BadStatus(status, body));
        }

        response
            .body_mut()
            .read_to_string()
            .map_err(|e| HttpError::BodyRead(e.to_string()))
    }
}
