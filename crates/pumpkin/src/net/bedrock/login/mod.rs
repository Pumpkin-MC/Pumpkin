use crate::{
    net::{
        DisconnectReason, GameProfile, PacketHandlerResult, PlayerConfig, bedrock::BedrockClient,
    },
    server::Server,
};
use arc_swap::ArcSwap;
use pumpkin_protocol::bedrock::{
    client::{
        network_settings::CNetworkSettings, play_status::CPlayStatus,
        resource_pack_stack::CResourcePackStackPacket, resource_packs_info::CResourcePacksInfo,
        start_game::Experiments,
    },
    server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
};
use pumpkin_protocol::bedrock::{
    client::{resource_pack_stack::PackInstanceId, resource_packs_info::PackInfoData},
    server::{login::ClientData, resource_pack_client_response::SResourcePackClientResponse},
};
use pumpkin_util::jwt::AuthError;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use serde::{Deserialize, de::Error};
use serde_repr::Deserialize_repr;
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Login packet data is not valid JSON")]
    InvalidTokenFormat(#[from] serde_json::Error),
    #[error("JWT chain validation failed: {0}")]
    ChainValidationFailed(#[from] AuthError),
    #[error("The validated username is invalid")]
    InvalidUsername,
    #[error("Could not parse UUID from validated token")]
    InvalidUuid,
    #[error("Cannot accept self-signed token. Authentication is enforced by server config.")]
    SelfSignedNotAllowed,
    #[error("Got a guest/splitscreen login request. Currently unimplemented.")]
    GuestUnimplemented,
    #[error("Failed to decode extra using decode_b64_url_nopad.")]
    DecodeExtraError,
}

#[derive(Deserialize_repr)]
#[repr(u8)]
enum AuthenticationType {
    Full,
    Guest,
    SelfSigned,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthPayload {
    authentication_type: AuthenticationType,
    token: String,
}

/// Verifies OIDC tokens for Bedrock 1.26.10+ clients on Rayon thread pool.
async fn verify_oidc_token_path(
    server: &Server,
    token: &str,
    self_signed: bool,
) -> Result<pumpkin_util::jwt::PlayerClaims, LoginError> {
    let token = token.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    if self_signed {
        rayon::spawn(move || {
            let res = pumpkin_util::jwt::verify_oidc_token_self_signed(&token)
                .map_err(LoginError::ChainValidationFailed);
            let _ = tx.send(res);
        });
    } else {
        let (issuer, jwks) =
            server
                .bedrock_oidc_keys
                .get()
                .cloned()
                .ok_or(LoginError::ChainValidationFailed(
                    AuthError::PublicKeyBuild("OIDC keys not initialized".into()),
                ))?;

        rayon::spawn(move || {
            let res = pumpkin_util::jwt::verify_oidc_token(&token, &issuer, &jwks)
                .map_err(LoginError::ChainValidationFailed);
            let _ = tx.send(res);
        });
    }

    rx.await.map_err(|_| {
        LoginError::ChainValidationFailed(AuthError::PublicKeyBuild("Task cancelled".into()))
    })?
}

#[allow(clippy::module_inception)]
pub mod login;
pub mod request_network_settings;
pub mod resource_pack_response;
