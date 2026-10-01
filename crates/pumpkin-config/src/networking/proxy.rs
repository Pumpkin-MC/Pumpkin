use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{fs, io, path::PathBuf, sync::OnceLock};

const MAX_SECRET_FILE_BYTES: usize = 1024 * 1024;

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
    /// Configuration for Vine modern proxy integration with Ed25519 authentication.
    pub vine: VineConfig,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    pub enabled: bool,
    /// Shared secret for authenticating connections from the `BungeeCord`
    /// proxy, as provided by the `BungeeGuard` plugin. When set, the forwarded
    /// profile properties must contain a `bungeeguard-token` property holding
    /// this secret, otherwise the connection is rejected. This also blocks
    /// players connecting directly instead of through the proxy.
    pub secret: String,
}

/// Configuration for Velocity proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VelocityConfig {
    /// Whether Velocity support is enabled.
    pub enabled: bool,
    /// Shared secret for authenticating connections from the Velocity proxy.
    pub secret: String,
    /// File containing the forwarding secret, relative to the server's working directory.
    /// Mutually exclusive with a nonempty inline `secret`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_file: Option<PathBuf>,
    #[serde(skip)]
    resolved_secret: OnceLock<Result<String, io::Error>>,
}

impl VelocityConfig {
    /// Returns the forwarding secret, reading and caching a configured file on first use.
    /// Files are limited to 1 MiB and trimmed; inline secrets are preserved verbatim.
    /// Restart the server to reload a changed file.
    ///
    /// # Errors
    /// Returns an error for conflicting settings, unreadable files or empty file contents.
    pub fn forwarding_secret(&self) -> Result<&str, &io::Error> {
        let Some(path) = &self.secret_file else {
            return Ok(&self.secret);
        };
        self.resolved_secret
            .get_or_init(|| {
                if !self.secret.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "set either velocity.secret or velocity.secret_file, not both",
                    ));
                }
                let mut contents = String::new();
                fs::File::open(path)
                    .and_then(|file| {
                        file.take((MAX_SECRET_FILE_BYTES + 1) as u64)
                            .read_to_string(&mut contents)
                    })
                    .map_err(|error| {
                        io::Error::new(
                            error.kind(),
                            format!(
                                "could not read Velocity secret file {}: {error}",
                                path.display()
                            ),
                        )
                    })?;
                if contents.len() > MAX_SECRET_FILE_BYTES {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Velocity secret file exceeds 1 MiB",
                    ));
                }
                let secret = contents.trim();
                if secret.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Velocity secret file must not be empty",
                    ));
                }
                Ok(secret.to_owned())
            })
            .as_deref()
    }
}

/// Configuration for Vine proxy integration with Ed25519 authentication and replay protection.
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct VineConfig {
    /// Whether Vine support is enabled.
    pub enabled: bool,
    /// Ed25519 public key (64 hex characters) of the Vine proxy.
    /// Backend only needs this public key to verify forwarded player identities.
    pub public_key: String,
    /// Optional shared secret string. If provided and `public_key` is empty,
    /// the public key is automatically derived from this secret.
    pub secret: String,
}

#[cfg(test)]
mod tests {
    use super::{MAX_SECRET_FILE_BYTES, VelocityConfig};
    use std::{fs, io};

    #[test]
    fn inline_secrets_remain_literal() {
        for secret in ["", "inline-key", "@forwarding.secret", "@@key", " key \n"] {
            let config = VelocityConfig {
                secret: secret.to_owned(),
                ..Default::default()
            };
            assert_eq!(config.forwarding_secret().unwrap(), secret);
        }
    }

    #[test]
    fn file_secret_is_trimmed_cached_and_not_serialized() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("forwarding.secret");
        fs::write(&path, "  fixture-forwarding-key\r\n").unwrap();
        let config = VelocityConfig {
            secret_file: Some(path.clone()),
            ..Default::default()
        };
        assert_eq!(
            config.forwarding_secret().unwrap(),
            "fixture-forwarding-key"
        );
        fs::write(&path, "rotated-key").unwrap();
        assert_eq!(
            config.forwarding_secret().unwrap(),
            "fixture-forwarding-key"
        );
        let serialized = toml::to_string(&config).unwrap();
        assert!(!serialized.contains("fixture-forwarding-key"));
        let reloaded: VelocityConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(reloaded.forwarding_secret().unwrap(), "rotated-key");
    }

    #[test]
    fn invalid_files_and_conflicting_settings_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("forwarding.secret");
        let config = || VelocityConfig {
            secret_file: Some(path.clone()),
            ..Default::default()
        };
        assert_eq!(
            config().forwarding_secret().unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        for contents in [b"".as_slice(), b" \r\n", b"\xff"] {
            fs::write(&path, contents).unwrap();
            assert_eq!(
                config().forwarding_secret().unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
        fs::write(&path, vec![b'x'; MAX_SECRET_FILE_BYTES + 1]).unwrap();
        assert_eq!(
            config().forwarding_secret().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        fs::write(&path, "file-key").unwrap();
        let config = VelocityConfig {
            secret: "inline-key".to_owned(),
            ..config()
        };
        let error = config.forwarding_secret().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!error.to_string().contains("inline-key"));
        assert!(!error.to_string().contains("file-key"));
    }
}
