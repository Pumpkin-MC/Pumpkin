//! HTTP client utilities.

/// Creates a `reqwest::ClientBuilder` configured with appropriate root certificates.
///
/// On Android, where the default `rustls-platform-verifier` assumes an Android app runtime (JVM/JNI)
/// and panics when running as a standalone binary (e.g. in Termux), this configures the builder
/// with Mozilla root certificates from `webpki-root-certs`.
pub fn client_builder() -> reqwest::ClientBuilder {
    let builder = reqwest::Client::builder();
    #[cfg(target_os = "android")]
    let builder = {
        let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .filter_map(|c| reqwest::Certificate::from_der(c.as_ref()).ok());
        builder.tls_certs_only(certs)
    };
    builder
}

/// Creates a default `reqwest::Client`.
#[must_use]
pub fn client() -> reqwest::Client {
    client_builder().build().unwrap_or_default()
}
