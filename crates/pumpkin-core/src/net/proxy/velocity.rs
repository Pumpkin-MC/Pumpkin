use arc_swap::ArcSwap;
use bytes::{BufMut, BytesMut};
use hmac::{Hmac, KeyInit, Mac};
use pumpkin_config::networking::proxy::VelocityConfig;
use pumpkin_protocol::{
    Property, java::client::login::CLoginPluginRequest, java::server::login::SLoginPluginResponse,
    ser::NetworkReadExt,
};
use rand::RngExt;
use sha2::Sha256;
use std::sync::Arc;
/// Proxy implementation for Velocity <https://papermc.io/software/velocity> by `PaperMC`
/// Sadly, `PaperMC` does not care about 3rd parties providing support for Velocity. There is no documentation.
/// I had to understand the code logic by looking at `PaperMC`'s Velocity implementation: <https://github.com/PaperMC/Paper/blob/0cf731589a3b6923542cdfc36dbcee9c47c51076/paper-server/src/main/java/com/destroystokyo/paper/proxy/VelocityProxy.java>
use std::{
    io::Read,
    net::{IpAddr, SocketAddr},
};
use thiserror::Error;
use tracing::debug;

use crate::net::{GameProfile, java::pending::PendingConnection};

type HmacSha256 = Hmac<Sha256>;

const MAX_SUPPORTED_FORWARDING_VERSION: u8 = 4;
const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

#[derive(Error, Debug)]
pub enum VelocityError {
    #[error("No response data received")]
    NoData,
    #[error("Unable to verify player details")]
    FailedVerifyIntegrity,
    #[error("Failed to read forward version")]
    FailedReadForwardVersion,
    #[error("Unsupported forwarding version {0}. Maximum supported version is {1}")]
    UnsupportedForwardVersion(u8, u8),
    #[error("Failed to read address")]
    FailedReadAddress,
    #[error("Failed to parse address")]
    FailedParseAddress,
    #[error("Failed to read game profile name")]
    FailedReadProfileName,
    #[error("Failed to read game profile UUID")]
    FailedReadProfileUUID,
    #[error("Failed to read game profile properties")]
    FailedReadProfileProperties,
}

pub async fn velocity_login(connection: &mut PendingConnection) {
    // TODO: Validate the packet transaction id from the plugin response with this
    let velocity_message_id: i32 = rand::rng().random();

    let mut buf = BytesMut::new();
    buf.put_u8(MAX_SUPPORTED_FORWARDING_VERSION);
    connection
        .send_packet_now(&CLoginPluginRequest::new(
            velocity_message_id.into(),
            PLAYER_INFO_CHANNEL,
            &buf,
        ))
        .await;
}

#[must_use]
pub fn check_integrity(data: (&[u8], &[u8]), secret: &str) -> bool {
    if secret.is_empty() {
        return false;
    }
    let (signature, data_without_signature) = data;
    // Our fault, we can panic/expect?
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(data_without_signature);
    mac.verify_slice(signature).is_ok()
}

fn read_game_profile(read: impl Read) -> Result<GameProfile, VelocityError> {
    let mut read = read;
    let id = read
        .get_uuid()
        .map_err(|_| VelocityError::FailedReadProfileUUID)?;

    let name = read
        .get_str()
        .map_err(|_| VelocityError::FailedReadProfileName)?;

    let properties = read
        .get_list(|data| {
            let name = data.get_str()?;
            let value = data.get_str()?;
            let signature = data.get_option(NetworkReadExt::get_str)?;

            Ok(Property {
                name,
                value,
                signature,
            })
        })
        .map_err(|_| VelocityError::FailedReadProfileProperties)?;

    Ok(GameProfile {
        id,
        name: name.into_string(),
        properties: ArcSwap::new(Arc::from(properties)),
        profile_actions: None,
    })
}

/// Verifies the forwarding signature before decoding the player's profile and address.
///
/// # Errors
/// Returns an error for missing data, an invalid secret or signature, or malformed player data.
pub fn receive_velocity_plugin_response(
    port: u16,
    config: &VelocityConfig,
    response: SLoginPluginResponse,
) -> Result<(GameProfile, SocketAddr), VelocityError> {
    debug!("Received velocity response");
    if let Some(data) = response.data {
        if data.len() < 32 {
            return Err(VelocityError::FailedVerifyIntegrity);
        }
        let (signature, mut data_without_signature) = data.split_at(32);

        let secret = config
            .forwarding_secret()
            .map_err(|_| VelocityError::FailedVerifyIntegrity)?;
        if !check_integrity((signature, data_without_signature), secret) {
            return Err(VelocityError::FailedVerifyIntegrity);
        }

        // Check velocity version
        let version = data_without_signature
            .get_var_int()
            .map_err(|_| VelocityError::FailedReadForwardVersion)?;

        let version = version.0 as u8;
        if version > MAX_SUPPORTED_FORWARDING_VERSION {
            return Err(VelocityError::UnsupportedForwardVersion(
                version,
                MAX_SUPPORTED_FORWARDING_VERSION,
            ));
        }
        let addr = data_without_signature
            .get_str()
            .map_err(|_| VelocityError::FailedReadAddress)?;

        let socket_addr: SocketAddr = SocketAddr::new(
            addr.parse::<IpAddr>()
                .map_err(|_| VelocityError::FailedParseAddress)?,
            port,
        );

        let profile = read_game_profile(&mut data_without_signature)?;
        return Ok((profile, socket_addr));
    }
    Err(VelocityError::NoData)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signed_response(secret: &str) -> SLoginPluginResponse {
        // Forwarding v1: loopback address, UUID, player name, empty property list.
        let mut payload = b"\x01\x09127.0.0.1".to_vec();
        payload.extend_from_slice(&[7; 16]);
        payload.extend_from_slice(b"\x07Fixture\x00");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(&payload);
        let mut data = mac.finalize().into_bytes().to_vec();
        data.extend_from_slice(&payload);
        SLoginPluginResponse {
            message_id: 0.into(),
            data: Some(data.into_boxed_slice()),
        }
    }

    #[test]
    fn forwarding_uses_file_contents_not_the_path_or_an_empty_inline_key() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("forwarding.secret");
        std::fs::write(&path, " \tfixture-hmac-\r\nkey \t\r\n").unwrap();
        let config: VelocityConfig =
            serde_json::from_value(serde_json::json!({ "secret_file": path })).unwrap();
        let (profile, address) = receive_velocity_plugin_response(
            25565,
            &config,
            signed_response(" \tfixture-hmac-key \t"),
        )
        .unwrap();
        assert_eq!(profile.name, "Fixture");
        assert_eq!(address, "127.0.0.1:25565".parse::<SocketAddr>().unwrap());
        for wrong_key in ["fixture-hmac-key", "wrong-key", path.to_str().unwrap(), ""] {
            assert!(matches!(
                receive_velocity_plugin_response(25565, &config, signed_response(wrong_key)),
                Err(VelocityError::FailedVerifyIntegrity)
            ));
        }
    }
}
