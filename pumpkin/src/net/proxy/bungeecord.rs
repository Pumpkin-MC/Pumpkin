use arc_swap::ArcSwap;
use pumpkin_config::networking::proxy::BungeeCordConfig;
use std::sync::Arc;
use std::{net::IpAddr, net::SocketAddr};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::net::{GameProfile, offline_uuid};

#[derive(Error, Debug)]
pub enum BungeeCordError {
    #[error("Failed to parse address")]
    FailedParseAddress,
    #[error("Failed to parse UUID")]
    FailedParseUUID,
    #[error("Failed to parse properties")]
    FailedParseProperties,
    #[error("Failed to make offline UUID")]
    FailedMakeOfflineUUID,
    #[error("BungeeGuard authentication failed: invalid or missing token")]
    BungeeGuardFailedAuth,
}

/// Attempts to login a player via `BungeeCord`.
///
/// This function should be called when receiving the `SLoginStart` packet.
/// It utilizes the `server_address` received in the `SHandShake` packet,
/// which may contain optional data about the client:
///
/// 1. IP address (if `ip_forward` is enabled on the `BungeeCord` server)
/// 2. UUID (if `ip_forward` is enabled on the `BungeeCord` server)
/// 3. Game profile properties (if `ip_forward` and `online_mode` are enabled on the `BungeeCord` server)
///
/// If any of the optional data is missing, the function will attempt to
/// determine the player's information locally.
///
/// When `config.secret` is set, the handshake must include a matching
/// `BungeeGuard` token. When `config.secret` is empty, no `BungeeGuard`
/// token should be present — connections with an unexpected token
/// are rejected to prevent proxy/server misconfiguration.
pub async fn bungeecord_login(
    config: &BungeeCordConfig,
    client_address: &Mutex<SocketAddr>,
    server_address: &str,
    name: String,
) -> Result<(IpAddr, GameProfile), BungeeCordError> {
    let mut parts = server_address.split('\0');

    // Skip the first part (the actual server address/host)
    let _host = parts.next();

    let ip = match parts.next() {
        Some(ip_str) if !ip_str.is_empty() => ip_str
            .parse()
            .map_err(|_| BungeeCordError::FailedParseAddress)?,
        _ => client_address.lock().await.ip(),
    };

    let id = match parts.next() {
        Some(uuid_str) if !uuid_str.is_empty() => uuid_str
            .parse()
            .map_err(|_| BungeeCordError::FailedParseUUID)?,
        _ => offline_uuid(&name).map_err(|_| BungeeCordError::FailedMakeOfflineUUID)?,
    };

    let properties = match parts.next() {
        Some(json_str) if !json_str.is_empty() => {
            serde_json::from_str(json_str).map_err(|_| BungeeCordError::FailedParseProperties)?
        }
        _ => Vec::new(),
    };

    // BungeeGuard: verify the authentication token
    // - When a secret is configured, the token must be present and match.
    // - When no secret is configured, no token should be present
    //   (prevents misconfiguration where the proxy uses BungeeGuard
    //   but the server does not).
    let token = parts.next();
    match (config.secret.is_empty(), token) {
        (false, Some(t)) if t == config.secret => {}
        (true, None) => {}
        _ => return Err(BungeeCordError::BungeeGuardFailedAuth),
    }

    Ok((
        ip,
        GameProfile {
            id,
            name,
            properties: ArcSwap::new(Arc::new(properties)),
            profile_actions: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn make_config(secret: &str) -> BungeeCordConfig {
        BungeeCordConfig {
            enabled: true,
            secret: secret.to_string(),
        }
    }

    fn make_address(ip: &str, port: u16) -> Mutex<SocketAddr> {
        Mutex::new(SocketAddr::new(ip.parse().unwrap(), port))
    }

    /// Handshake with full BungeeCord data + matching BungeeGuard token.
    #[tokio::test]
    async fn secret_set_token_matches() {
        let config = make_config("mysecret");
        let addr = make_address("127.0.0.1", 25565);
        let handshake = "localhost\0127.0.0.2\000000000-0000-0000-0000-000000000001\0\0mysecret";

        let result = bungeecord_login(&config, &addr, handshake, "test".into()).await;
        assert!(result.is_ok());
        let (ip, profile) = result.unwrap();
        assert_eq!(ip, "127.0.0.2".parse::<IpAddr>().unwrap());
        assert_eq!(profile.name, "test");
    }

    /// Handshake with BungeeCord data but no BungeeGuard token — should fail.
    #[tokio::test]
    async fn secret_set_no_token() {
        let config = make_config("mysecret");
        let addr = make_address("127.0.0.1", 25565);
        let handshake = "localhost\0127.0.0.2\000000000-0000-0000-0000-000000000001\0";

        let result = bungeecord_login(&config, &addr, handshake, "test".into()).await;
        assert!(matches!(result, Err(BungeeCordError::BungeeGuardFailedAuth)));
    }

    /// Handshake with wrong BungeeGuard token — should fail.
    #[tokio::test]
    async fn secret_set_wrong_token() {
        let config = make_config("mysecret");
        let addr = make_address("127.0.0.1", 25565);
        let handshake = "localhost\0127.0.0.2\000000000-0000-0000-0000-000000000001\0\0wrong";

        let result = bungeecord_login(&config, &addr, handshake, "test".into()).await;
        assert!(matches!(result, Err(BungeeCordError::BungeeGuardFailedAuth)));
    }

    /// Normal BungeeCord handshake without BungeeGuard — backward compatible.
    #[tokio::test]
    async fn secret_empty_no_token() {
        let config = make_config("");
        let addr = make_address("127.0.0.1", 25565);
        let handshake = "localhost\0127.0.0.2\000000000-0000-0000-0000-000000000001\0";

        let result = bungeecord_login(&config, &addr, handshake, "test".into()).await;
        assert!(result.is_ok());
    }

    /// BungeeGuard token present but server not configured — misconfiguration.
    #[tokio::test]
    async fn secret_empty_has_token() {
        let config = make_config("");
        let addr = make_address("127.0.0.1", 25565);
        let handshake = "localhost\0127.0.0.2\000000000-0000-0000-0000-000000000001\0\0sometoken";

        let result = bungeecord_login(&config, &addr, handshake, "test".into()).await;
        assert!(matches!(result, Err(BungeeCordError::BungeeGuardFailedAuth)));
    }
}
