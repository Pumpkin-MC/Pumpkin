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
use tracing::debug;

use pumpkin_util::translation::{localized_log, localized_log_format};

use crate::net::{GameProfile, java::JavaClient};

type HmacSha256 = Hmac<Sha256>;

const MAX_SUPPORTED_FORWARDING_VERSION: u8 = 4;
const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

#[derive(Debug)]
pub enum VelocityError {
    NoData,
    FailedVerifyIntegrity,
    FailedReadForwardVersion,
    UnsupportedForwardVersion(u8, u8),
    FailedReadAddress,
    FailedParseAddress,
    FailedReadProfileName,
    FailedReadProfileUUID,
    FailedReadProfileProperties,
}

impl std::fmt::Display for VelocityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoData => write!(f, "{}", localized_log("proxy.velocity.no_data")),
            Self::FailedVerifyIntegrity => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_verify_integrity")
            ),
            Self::FailedReadForwardVersion => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_read_forward_version")
            ),
            Self::UnsupportedForwardVersion(version, max) => write!(
                f,
                "{}",
                localized_log_format(
                    "proxy.velocity.unsupported_forward_version",
                    &[version.to_string(), max.to_string()],
                )
            ),
            Self::FailedReadAddress => {
                write!(f, "{}", localized_log("proxy.velocity.failed_read_address"))
            }
            Self::FailedParseAddress => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_parse_address")
            ),
            Self::FailedReadProfileName => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_read_profile_name")
            ),
            Self::FailedReadProfileUUID => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_read_profile_uuid")
            ),
            Self::FailedReadProfileProperties => write!(
                f,
                "{}",
                localized_log("proxy.velocity.failed_read_profile_properties")
            ),
        }
    }
}

impl std::error::Error for VelocityError {}

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
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .unwrap_or_else(|_| panic!("{}", localized_log("debug.expect.hmac_key_any_size")));
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
    debug!("{}", localized_log("server.log.velocity_response_received"));
    if let Some(data) = response.data {
        let (signature, mut data_without_signature) = data.split_at(32);

        if !check_integrity((signature, data_without_signature), &config.secret) {
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
