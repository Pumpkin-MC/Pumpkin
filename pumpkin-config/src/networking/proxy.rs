use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::secret::resolve_file_reference;

/// Configuration for proxy support.
///
/// Allows integration with proxy servers like Velocity and `BungeeCord`.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ProxyConfig {
    /// Whether proxy support is enabled.
    pub enabled: bool,
    /// Configuration for Velocity proxy integration.
    pub velocity: VelocityConfig,
    /// Configuration for `BungeeCord` proxy integration.
    pub bungeecord: BungeeCordConfig,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    pub enabled: bool,
}

/// Configuration for Velocity proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VelocityConfig {
    /// Whether Velocity support is enabled.
    pub enabled: bool,
    /// Shared secret for authenticating connections from the Velocity proxy.
    ///
    /// A value beginning with `@` is read from the file at that path rather than
    /// being used literally, e.g. `secret = "@forwarding.secret"`, which keeps the
    /// secret out of `pumpkin.toml`. Double the `@` to begin a literal value with
    /// one. Read it through [`Self::secret`] rather than using this field
    /// directly.
    pub secret: String,
    /// Cache of [`Self::secret`] with any `@file` reference resolved.
    ///
    /// Deliberately skipped during (de)serialization. `secret` has to round-trip
    /// to disk as the original `@file` reference, because
    /// `LoadConfiguration::load` writes the deserialized config back out whenever
    /// it fills in missing defaults. Storing the resolved value in `secret` would
    /// make that write-back copy the real secret into `pumpkin.toml`, defeating
    /// the point of keeping it in a separate file.
    #[serde(skip)]
    resolved_secret: OnceLock<String>,
}

impl VelocityConfig {
    /// Creates a Velocity configuration.
    ///
    /// `secret` may be a literal secret or a `@file` reference; it is resolved
    /// lazily by [`Self::secret`]. This exists because the resolved-secret cache
    /// is private, which would otherwise make the struct impossible to build
    /// outside this crate.
    #[must_use]
    pub const fn new(enabled: bool, secret: String) -> Self {
        Self {
            enabled,
            secret,
            resolved_secret: OnceLock::new(),
        }
    }

    /// The shared secret to authenticate the proxy with.
    ///
    /// Resolves a `@file` reference on first use and caches the result, so the
    /// file is read once rather than on every player connection.
    #[must_use]
    pub fn secret(&self) -> &str {
        self.resolved_secret
            .get_or_init(|| resolve_file_reference(&self.secret, "The Velocity forwarding secret"))
            .as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::VelocityConfig;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn literal_secret_is_used_as_is() {
        let config = VelocityConfig {
            enabled: true,
            secret: "hunter2".to_owned(),
            ..Default::default()
        };

        assert_eq!(config.secret(), "hunter2");
    }

    #[test]
    fn referenced_secret_is_read_from_the_file() {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(b"s3cret-from-disk\n")
            .expect("failed to write temp file");

        let config = VelocityConfig {
            enabled: true,
            secret: format!("@{}", file.path().display()),
            ..Default::default()
        };

        assert_eq!(config.secret(), "s3cret-from-disk");
    }

    /// The config is written back to disk whenever `LoadConfiguration::load`
    /// fills in missing defaults. If the resolved secret ever reached the
    /// serialized form, that write-back would leak it into `pumpkin.toml`.
    #[test]
    fn resolved_secret_is_never_serialized() {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(b"s3cret-from-disk\n")
            .expect("failed to write temp file");

        let reference = format!("@{}", file.path().display());
        let config = VelocityConfig {
            enabled: true,
            secret: reference.clone(),
            ..Default::default()
        };

        // Resolve first: the cache is only populated after the secret is used.
        assert_eq!(config.secret(), "s3cret-from-disk");

        let serialized = toml::to_string(&config).expect("failed to serialize config");

        assert!(
            !serialized.contains("s3cret-from-disk"),
            "the resolved secret leaked into the serialized config:\n{serialized}"
        );
        assert!(
            serialized.contains(reference.as_str()),
            "the `@file` reference was not preserved in the serialized config:\n{serialized}"
        );
    }
}
