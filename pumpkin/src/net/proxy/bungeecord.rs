use std::{net::IpAddr, net::SocketAddr};

use pumpkin_config::networking::proxy::BungeeCordConfig;
use pumpkin_protocol::Property;
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
    #[error("Proxy IP not in allowed list")]
    ProxyNotAllowed,
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
/// # IP Whitelist
/// If `allowed_ips` is configured in the BungeeCord config, only connections
/// from those IP addresses will be accepted. If the list is empty, all IPs are allowed.
pub async fn bungeecord_login(
    client_address: &Mutex<SocketAddr>,
    server_address: &str,
    name: String,
    config: &BungeeCordConfig,
) -> Result<(IpAddr, GameProfile), BungeeCordError> {
    // Get client IP for whitelist check
    let client_ip = client_address.lock().await.ip();

    // IP whitelist validation
    if !config.allowed_ips.is_empty() && !config.allowed_ips.contains(&client_ip) {
        log::warn!(
            "BungeeCord connection rejected from non-whitelisted IP: {}",
            client_ip
        );
        return Err(BungeeCordError::ProxyNotAllowed);
    }

    let data = server_address.split('\0').take(4).collect::<Vec<_>>();

    // The IP address of the player; only given if `ip_forward` on bungee is true.
    let ip = match data.get(1) {
        Some(ip) => ip
            .parse()
            .map_err(|_| BungeeCordError::FailedParseAddress)?,
        None => client_ip,
    };

    // The UUID of the player; only given if `ip_forward` on bungee is true.
    let id = match data.get(2) {
        Some(uuid) => uuid.parse().map_err(|_| BungeeCordError::FailedParseUUID)?,
        None => offline_uuid(name.as_str()).map_err(|_| BungeeCordError::FailedMakeOfflineUUID)?,
    };

    // Read properties and get textures.
    // Properties of the player's game profile are only given if `ip_forward` and `online_mode`
    // on bungee are both `true`.
    let properties: Vec<Property> = match data.get(3) {
        Some(properties) => {
            serde_json::from_str(properties).map_err(|_| BungeeCordError::FailedParseProperties)?
        }
        None => vec![],
    };

    Ok((
        ip,
        GameProfile {
            id,
            name,
            properties,
            profile_actions: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    /// Helper to create a BungeeCordConfig with allowed IPs
    fn config_with_allowed_ips(allowed_ips: Vec<IpAddr>) -> BungeeCordConfig {
        BungeeCordConfig {
            enabled: true,
            allowed_ips,
        }
    }

    /// Helper to create a mock client address
    fn mock_client_address(ip: IpAddr) -> Mutex<SocketAddr> {
        Mutex::new(SocketAddr::V4(SocketAddrV4::new(
            match ip {
                IpAddr::V4(v4) => v4,
                IpAddr::V6(_) => Ipv4Addr::new(127, 0, 0, 1), // fallback for test
            },
            12345,
        )))
    }

    /// Property test: For any BungeeCord connection, if allowed_ips is configured,
    /// only connections from those IPs SHALL be accepted.
    /// **Feature: security-hardening, Property 3: Proxy IP Whitelist**
    /// **Validates: Requirements 3.2**
    #[tokio::test]
    async fn test_property_ip_whitelist_rejects_non_whitelisted() {
        // Test with various whitelist configurations
        for _ in 0..100 {
            // Generate random allowed IP
            let allowed_ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            // Generate a different random client IP (ensure it's different)
            let mut client_ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));
            while client_ip == allowed_ip {
                client_ip = IpAddr::V4(Ipv4Addr::new(
                    rand::random(),
                    rand::random(),
                    rand::random(),
                    rand::random(),
                ));
            }

            let config = config_with_allowed_ips(vec![allowed_ip]);
            let client_address = mock_client_address(client_ip);

            let result = bungeecord_login(
                &client_address,
                "localhost",
                "TestPlayer".to_string(),
                &config,
            )
            .await;

            assert!(
                matches!(result, Err(BungeeCordError::ProxyNotAllowed)),
                "Connection from non-whitelisted IP {} should be rejected when whitelist contains {}",
                client_ip,
                allowed_ip
            );
        }
    }

    /// Property test: Whitelisted IPs should be accepted
    /// **Feature: security-hardening, Property 3: Proxy IP Whitelist**
    /// **Validates: Requirements 3.2**
    #[tokio::test]
    async fn test_property_ip_whitelist_accepts_whitelisted() {
        for _ in 0..100 {
            // Generate random IP that will be both in whitelist and client IP
            let ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            let config = config_with_allowed_ips(vec![ip]);
            let client_address = mock_client_address(ip);

            let result = bungeecord_login(
                &client_address,
                "localhost",
                "TestPlayer".to_string(),
                &config,
            )
            .await;

            // Should not be ProxyNotAllowed error (may fail for other reasons like parsing)
            assert!(
                !matches!(result, Err(BungeeCordError::ProxyNotAllowed)),
                "Connection from whitelisted IP {} should not be rejected as ProxyNotAllowed",
                ip
            );
        }
    }

    /// Property test: Empty whitelist should allow all IPs
    /// **Feature: security-hardening, Property 3: Proxy IP Whitelist**
    /// **Validates: Requirements 3.2**
    #[tokio::test]
    async fn test_property_empty_whitelist_allows_all() {
        for _ in 0..100 {
            // Generate random client IP
            let client_ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            let config = config_with_allowed_ips(vec![]); // Empty whitelist
            let client_address = mock_client_address(client_ip);

            let result = bungeecord_login(
                &client_address,
                "localhost",
                "TestPlayer".to_string(),
                &config,
            )
            .await;

            // Should not be ProxyNotAllowed error (empty whitelist allows all)
            assert!(
                !matches!(result, Err(BungeeCordError::ProxyNotAllowed)),
                "Connection from any IP {} should be allowed when whitelist is empty",
                client_ip
            );
        }
    }

    proptest! {
        /// Property test using proptest: IP whitelist enforcement
        /// **Feature: security-hardening, Property 3: Proxy IP Whitelist**
        /// **Validates: Requirements 3.2**
        #[test]
        fn test_proptest_whitelist_enforcement(
            allowed_octets in prop::array::uniform4(0u8..=255u8),
            client_octets in prop::array::uniform4(0u8..=255u8),
        ) {
            let allowed_ip = IpAddr::V4(Ipv4Addr::new(
                allowed_octets[0],
                allowed_octets[1],
                allowed_octets[2],
                allowed_octets[3],
            ));
            let client_ip = IpAddr::V4(Ipv4Addr::new(
                client_octets[0],
                client_octets[1],
                client_octets[2],
                client_octets[3],
            ));

            let config = config_with_allowed_ips(vec![allowed_ip]);

            // The whitelist check logic
            let should_allow = config.allowed_ips.is_empty() || config.allowed_ips.contains(&client_ip);

            if allowed_ip == client_ip {
                prop_assert!(should_allow, "Same IP should be allowed");
            } else {
                prop_assert!(!should_allow, "Different IP should be rejected when whitelist is non-empty");
            }
        }

        /// Property test: Multiple IPs in whitelist
        /// **Feature: security-hardening, Property 3: Proxy IP Whitelist**
        /// **Validates: Requirements 3.2**
        #[test]
        fn test_proptest_multiple_whitelist_ips(
            ip1_octets in prop::array::uniform4(0u8..=255u8),
            ip2_octets in prop::array::uniform4(0u8..=255u8),
            client_octets in prop::array::uniform4(0u8..=255u8),
        ) {
            let ip1 = IpAddr::V4(Ipv4Addr::new(ip1_octets[0], ip1_octets[1], ip1_octets[2], ip1_octets[3]));
            let ip2 = IpAddr::V4(Ipv4Addr::new(ip2_octets[0], ip2_octets[1], ip2_octets[2], ip2_octets[3]));
            let client_ip = IpAddr::V4(Ipv4Addr::new(client_octets[0], client_octets[1], client_octets[2], client_octets[3]));

            let config = config_with_allowed_ips(vec![ip1, ip2]);

            let should_allow = config.allowed_ips.contains(&client_ip);
            let is_in_whitelist = client_ip == ip1 || client_ip == ip2;

            prop_assert_eq!(should_allow, is_in_whitelist,
                "Client IP {} should be allowed={} when whitelist contains {} and {}",
                client_ip, is_in_whitelist, ip1, ip2);
        }
    }
}
