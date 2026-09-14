use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::{net::IpAddr, net::SocketAddr};
use thiserror::Error;
use tracing::warn;

use crate::net::{GameProfile, offline_uuid};
use pumpkin_protocol::Property;

/// The property name the `BungeeGuard` plugin uses to forward its shared
/// secret inside the profile properties.
const BUNGEEGUARD_TOKEN_PROPERTY: &str = "bungeeguard-token";

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
    #[error("No BungeeGuard token in forwarded data")]
    MissingToken,
    #[error("Invalid BungeeGuard token")]
    InvalidToken,
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
/// If a `secret` is configured, the properties must contain a property named
/// `bungeeguard-token` holding the secret, as injected by the `BungeeGuard`
/// plugin. The token property is stripped from the profile, and a missing or
/// mismatched token rejects the connection. This also blocks players
/// connecting directly to this server, bypassing the proxy.
///
/// If any of the optional data is missing, the function will attempt to
/// determine the player's information locally.
pub fn bungeecord_login(
    client_address: &SocketAddr,
    server_address: &str,
    name: String,
    secret: &str,
) -> Result<(IpAddr, GameProfile), BungeeCordError> {
    let mut parts = server_address.split('\0');

    // Skip the first part (the actual server address/host)
    let _host = parts.next();

    let ip = match parts.next() {
        Some(ip_str) if !ip_str.is_empty() => ip_str
            .parse()
            .map_err(|_| BungeeCordError::FailedParseAddress)?,
        _ => client_address.ip(),
    };

    let id = match parts.next() {
        Some(uuid_str) if !uuid_str.is_empty() => uuid_str
            .parse()
            .map_err(|_| BungeeCordError::FailedParseUUID)?,
        _ => offline_uuid(&name).map_err(|_| BungeeCordError::FailedMakeOfflineUUID)?,
    };

    let mut properties: Vec<Property> = match parts.next() {
        Some(json_str) if !json_str.is_empty() => {
            serde_json::from_str(json_str).map_err(|_| BungeeCordError::FailedParseProperties)?
        }
        _ => Vec::new(),
    };

    // The `BungeeGuard` plugin injects the shared secret as a property named
    // `bungeeguard-token` inside the forwarded profile properties. When a
    // secret is configured, that property must be present and hold the
    // secret; the property is then stripped so it never reaches the game
    // profile. This also blocks players connecting directly instead of
    // through the proxy.
    if !secret.is_empty() {
        let token_props: Vec<&Property> = properties
            .iter()
            .filter(|property| property.name.as_ref() == BUNGEEGUARD_TOKEN_PROPERTY)
            .collect();

        match token_props.as_slice() {
            [token] if token.value.as_ref() == secret => {
                properties.retain(|property| property.name.as_ref() != BUNGEEGUARD_TOKEN_PROPERTY);
            }
            [] => {
                warn!(
                    "Rejecting login: forwarded data has no `{}` property \
                     ({} parts, property names: {:?})",
                    BUNGEEGUARD_TOKEN_PROPERTY,
                    server_address.split('\0').count(),
                    properties
                        .iter()
                        .map(|p| p.name.as_ref())
                        .collect::<Vec<_>>()
                );
                return Err(BungeeCordError::MissingToken);
            }
            _ => {
                // Log only SHA-256 hashes: one-way, so the secret never leaks,
                // but enough to tell a mismatch from duplicated tokens apart.
                let token_hashes: Vec<String> = token_props
                    .iter()
                    .map(|property| hex::encode(Sha256::digest(property.value.as_bytes())))
                    .collect();
                warn!(
                    "Rejecting login: expected exactly one matching `{}` property, \
                     found {} (token hashes: {token_hashes:?}, configured secret \
                     hash: {})",
                    BUNGEEGUARD_TOKEN_PROPERTY,
                    token_props.len(),
                    hex::encode(Sha256::digest(secret.as_bytes()))
                );
                return Err(BungeeCordError::InvalidToken);
            }
        }
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
    use pumpkin_protocol::ser::NetworkWriteExt;
    use pumpkin_protocol::{
        ServerPacket, codec::var_int::VarInt, java::server::handshake::SHandShake,
    };
    use pumpkin_util::version::JavaMinecraftVersion;

    /// Drives the whole path a proxied login takes: the handshake is encoded as
    /// `BungeeCord` puts it on the wire, decoded by the real packet reader, and
    /// the address it yields is handed to `bungeecord_login`. This is what fails
    /// when the reader's bound on `server_address` is too small to hold the
    /// forwarded profile properties.
    #[tokio::test]
    async fn logs_in_from_a_handshake_decoded_off_the_wire() {
        let textures = "e".repeat(432);
        let signature = "s".repeat(684);
        let address = format!(
            "mc.example.com\0192.0.2.10\0d8f4a1e0-0f1b-4c3a-9f2e-1a2b3c4d5e6f\0\
             [{{\"name\":\"textures\",\"value\":\"{textures}\",\"signature\":\"{signature}\"}}]"
        );

        let mut buf = Vec::new();
        let protocol_version = JavaMinecraftVersion::V_1_21_11.protocol_version();
        buf.write_var_int(&VarInt(protocol_version))
            .expect("write protocol version");
        buf.write_string(&address).expect("write server address");
        buf.write_u16_be(25565).expect("write server port");
        buf.write_var_int(&VarInt(2)).expect("write next state");

        let handshake = SHandShake::read(&mut &buf[..], &JavaMinecraftVersion::V_1_21_11)
            .expect("a handshake sent by BungeeCord should be readable");

        let client_address = SocketAddr::from(([10, 0, 0, 1], 51234));
        let (ip, profile) = bungeecord_login(
            &client_address,
            &handshake.server_address,
            "Steve".to_string(),
            "",
        )
        .expect("the forwarded address should produce a game profile");

        // The forwarded IP and UUID are used, not the proxy's own socket address.
        assert_eq!(ip, IpAddr::from([192, 0, 2, 10]));
        assert_eq!(
            profile.id,
            "d8f4a1e0-0f1b-4c3a-9f2e-1a2b3c4d5e6f"
                .parse::<uuid::Uuid>()
                .expect("valid uuid")
        );

        // The signed skin survives, so the player keeps their appearance.
        let properties = profile.properties.load();
        assert_eq!(properties.len(), 1);
        assert_eq!(&*properties[0].name, "textures");
        assert_eq!(&*properties[0].value, textures.as_str());
        assert_eq!(properties[0].signature.as_deref(), Some(signature.as_str()));
    }

    const SECRET: &str = "bungeeguard-token";
    const FORWARDED_HOST: &str = concat!(
        // Split at the digits so the `\0` is not read as an octal escape.
        "mc.example.com\0",
        "192.0.2.10\0",
        "d8f4a1e0-0f1b-4c3a-9f2e-1a2b3c4d5e6f"
    );

    fn client_address() -> SocketAddr {
        SocketAddr::from(([10, 0, 0, 1], 51234))
    }

    /// The forwarded address with the given profile `properties` as its
    /// fourth part, as `BungeeCord` puts them on the wire.
    fn forwarded_address(properties: &str) -> String {
        format!("{FORWARDED_HOST}\0{properties}")
    }

    /// The `BungeeGuard` token property alone, as the plugin injects it into
    /// the forwarded profile properties.
    fn token_property(token: &str) -> String {
        format!(r#"{{"name":"bungeeguard-token","value":"{token}","signature":""}}"#)
    }

    /// A properties array containing the given property objects.
    fn properties_array(properties: &[&str]) -> String {
        format!("[{}]", properties.join(","))
    }

    #[test]
    fn accepts_matching_bungeeguard_token() {
        let properties = format!(
            r#"[{{"name":"textures","value":"skin","signature":"sig"}},{{"name":"bungeeguard-token","value":"{SECRET}","signature":""}}]"#
        );
        let address = forwarded_address(&properties);

        let (ip, profile) =
            bungeecord_login(&client_address(), &address, "Steve".to_string(), SECRET)
                .expect("a matching token should be accepted");

        assert_eq!(ip, IpAddr::from([192, 0, 2, 10]));

        // The token property is stripped so it never reaches the game profile.
        let properties = profile.properties.load();
        assert_eq!(properties.len(), 1);
        assert_eq!(&*properties[0].name, "textures");
    }

    #[test]
    fn rejects_missing_bungeeguard_token() {
        let address =
            forwarded_address(r#"[{"name":"textures","value":"skin","signature":"sig"}]"#);

        let result = bungeecord_login(&client_address(), &address, "Steve".to_string(), SECRET);

        assert!(matches!(result, Err(BungeeCordError::MissingToken)));
    }

    #[test]
    fn rejects_mismatched_bungeeguard_token() {
        let address = forwarded_address(&properties_array(&[&token_property("wrong-token")]));

        let result = bungeecord_login(&client_address(), &address, "Steve".to_string(), SECRET);

        assert!(matches!(result, Err(BungeeCordError::InvalidToken)));
    }

    #[test]
    fn rejects_multiple_bungeeguard_tokens() {
        let properties = properties_array(&[&token_property(SECRET), &token_property(SECRET)]);
        let address = forwarded_address(&properties);

        let result = bungeecord_login(&client_address(), &address, "Steve".to_string(), SECRET);

        assert!(matches!(result, Err(BungeeCordError::InvalidToken)));
    }

    #[test]
    fn rejects_direct_connection_when_secret_is_configured() {
        let result = bungeecord_login(
            &client_address(),
            "mc.example.com",
            "Steve".to_string(),
            SECRET,
        );

        assert!(matches!(result, Err(BungeeCordError::MissingToken)));
    }

    #[test]
    fn ignores_token_when_no_secret_is_configured() {
        let address = forwarded_address(&properties_array(&[&token_property(SECRET)]));

        let result = bungeecord_login(&client_address(), &address, "Steve".to_string(), "");

        assert!(
            result.is_ok(),
            "an unconfigured secret must not reject logins"
        );
    }
}
