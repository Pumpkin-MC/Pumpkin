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
    client: reqwest::blocking::Client,
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
        let builder = reqwest::blocking::Client::builder().user_agent(user_agent);
        #[cfg(target_os = "android")]
        let builder = {
            let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS
                .iter()
                .filter_map(|c| reqwest::Certificate::from_der(c.as_ref()).ok());
            builder.tls_certs_only(certs)
        };
        let client = builder.build().unwrap_or_default();
        Self { client }
    }

    /// Performs an HTTP GET request and returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or returns a non-2xx status code.
    pub fn get(&self, url: &str) -> Result<String, HttpError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .map_err(|e| HttpError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status < 200 || status >= 300 {
            let body = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::BadStatus(status, body));
        }

        response
            .text()
            .map_err(|e| HttpError::BodyRead(e.to_string()))
    }

    /// Performs an HTTP POST request with a JSON payload and returns the response body as a string.
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or returns a non-2xx status code.
    pub fn post_json(&self, url: &str, json_payload: &str) -> Result<String, HttpError> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json_payload.to_string())
            .send()
            .map_err(|e| HttpError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status < 200 || status >= 300 {
            let body = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::BadStatus(status, body));
        }

        response
            .text()
            .map_err(|e| HttpError::BodyRead(e.to_string()))
    }
}
