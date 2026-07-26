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

use crate::net::{GameProfile, java::JavaClient};

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

pub async fn velocity_login(client: &JavaClient) {
    // TODO: Validate the packet transaction id from the plugin response with this
    let velocity_message_id: i32 = rand::rng().random();

    let mut buf = BytesMut::new();
    buf.put_u8(MAX_SUPPORTED_FORWARDING_VERSION);
    client
        .enqueue_packet(&CLoginPluginRequest::new(
            velocity_message_id.into(),
            PLAYER_INFO_CHANNEL,
            &buf,
        ))
        .await;
}

#[must_use]
pub fn check_integrity(data: (&[u8], &[u8]), secret: &str) -> bool {
    let (signature, data_without_signature) = data;
    // Our fault, we can panic/expect?
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
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

        if !check_integrity((signature, data_without_signature), config.secret()) {
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
    use super::{VelocityError, receive_velocity_plugin_response};
    use hmac::{Hmac, KeyInit, Mac};
    use pumpkin_config::networking::proxy::VelocityConfig;
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::java::server::login::SLoginPluginResponse;
    use pumpkin_protocol::ser::NetworkWriteExt;
    use sha2::Sha256;
    use std::io::Write;
    use tempfile::NamedTempFile;

    type HmacSha256 = Hmac<Sha256>;

    const SECRET: &str = "s3cret-from-disk";
    const PLAYER_NAME: &str = "Notch";
    const ADDRESS: &str = "127.0.0.1";
    const PORT: u16 = 25565;

    /// Builds the forwarding payload Velocity sends after a player info request,
    /// signed with `signing_secret`.
    ///
    /// The layout matches what `receive_velocity_plugin_response` expects: a
    /// 32 byte HMAC-SHA256 signature followed by the forwarding version, the
    /// player's address, and their game profile.
    fn forwarding_response(signing_secret: &str, id: uuid::Uuid) -> SLoginPluginResponse {
        let mut payload = Vec::new();
        payload
            .write_var_int(&VarInt::from(1))
            .expect("write forwarding version");
        payload.write_string(ADDRESS).expect("write address");
        payload.write_uuid(&id).expect("write profile uuid");
        payload
            .write_string(PLAYER_NAME)
            .expect("write profile name");
        payload
            .write_var_int(&VarInt::from(1))
            .expect("write property count");
        payload
            .write_string("textures")
            .expect("write property name");
        payload.write_string("value").expect("write property value");
        payload
            .write_option(&None::<String>, |writer, value: &String| {
                writer.write_string(value)
            })
            .expect("write property signature");

        let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(&payload);
        let signature = mac.finalize().into_bytes();

        let mut data = Vec::with_capacity(signature.len() + payload.len());
        data.extend_from_slice(&signature);
        data.extend_from_slice(&payload);

        SLoginPluginResponse {
            message_id: VarInt::from(0),
            data: Some(data.into_boxed_slice()),
        }
    }

    /// A config whose secret lives in a file, referenced as `@<path>`.
    ///
    /// The `NamedTempFile` is returned so the caller keeps it alive; dropping it
    /// deletes the file the config points at.
    fn config_with_secret_file() -> (NamedTempFile, VelocityConfig) {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(format!("{SECRET}\n").as_bytes())
            .expect("failed to write secret file");

        let config = VelocityConfig::new(true, format!("@{}", file.path().display()));

        (file, config)
    }

    /// The real point of the `@file` feature: a payload signed with the secret as
    /// stored *in the file* has to pass integrity checking, proving the bytes that
    /// reach the HMAC are the file's contents and not the literal `@path`.
    #[test]
    fn payload_signed_with_the_file_secret_is_accepted() {
        let (_file, config) = config_with_secret_file();
        let id = uuid::Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

        let (profile, address) =
            receive_velocity_plugin_response(PORT, &config, forwarding_response(SECRET, id))
                .expect("a payload signed with the file's secret should be accepted");

        assert_eq!(profile.name, PLAYER_NAME);
        assert_eq!(profile.id, id);
        assert_eq!(address.ip().to_string(), ADDRESS);
        assert_eq!(address.port(), PORT);
    }

    /// Guards against the inverse mistake: if the literal `@path` were used as the
    /// key, or resolution silently produced the wrong value, this would still pass.
    #[test]
    fn payload_signed_with_a_different_secret_is_rejected() {
        let (_file, config) = config_with_secret_file();
        let id = uuid::Uuid::from_u128(1);

        let result = receive_velocity_plugin_response(
            PORT,
            &config,
            forwarding_response("not-the-real-secret", id),
        );

        assert!(matches!(result, Err(VelocityError::FailedVerifyIntegrity)));
    }

    /// The unresolved `@path` string must never be what signs the payload.
    #[test]
    fn payload_signed_with_the_raw_reference_is_rejected() {
        let (_file, config) = config_with_secret_file();
        let id = uuid::Uuid::from_u128(2);
        let raw_reference = config.secret.clone();

        let result = receive_velocity_plugin_response(
            PORT,
            &config,
            forwarding_response(&raw_reference, id),
        );

        assert!(matches!(result, Err(VelocityError::FailedVerifyIntegrity)));
    }
}
