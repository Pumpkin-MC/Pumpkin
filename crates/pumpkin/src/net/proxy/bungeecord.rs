use arc_swap::ArcSwap;
use std::sync::Arc;
use std::{net::IpAddr, net::SocketAddr};
use thiserror::Error;

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
pub fn bungeecord_login(
    client_address: &SocketAddr,
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
        _ => client_address.ip(),
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
}
