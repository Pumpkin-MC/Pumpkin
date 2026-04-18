use crate::net::authentication::MOJANG_BEDROCK_PUBLIC_KEY_BASE64;
use crate::{
    net::{ClientPlatform, DisconnectReason, GameProfile, bedrock::BedrockClient},
    server::Server,
};
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::bedrock::server::resource_pack_response::SResourcePackResponse;
use pumpkin_protocol::{
    bedrock::{
        client::{
            network_settings::CNetworkSettings, play_status::CPlayStatus,
            resource_pack_stack::CResourcePackStackPacket, resource_packs_info::CResourcePacksInfo,
            start_game::Experiments,
        },
        frame_set::FrameSet,
        server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
    },
    codec::var_uint::VarUInt,
};
use pumpkin_util::jwt::{AuthError, verify_chain};
use pumpkin_world::CURRENT_BEDROCK_MC_VERSION;
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Login packet data is not a valid JSON array of tokens")]
    InvalidTokenFormat(#[from] serde_json::Error),
    #[error("JWT chain validation failed: {0}")]
    ChainValidationFailed(#[from] AuthError),
    #[error("The validated username is invalid")]
    InvalidUsername,
    #[error("Could not parse UUID from validated token")]
    InvalidUuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FullLoginPayload {
    certificate: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthPayload {
    /// Present in legacy certificate-chain auth (protocol versions before 1.26.10).
    #[serde(default)]
    certificate: Option<String>,
    /// Present in 1.26.10+ OIDC token auth.
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CertificateChainPayload {
    chain: Vec<String>,
}

/// Verifies OIDC tokens for Bedrock 1.26.10+ clients.
async fn verify_oidc_token_path(
    server: &Server,
    token: &str,
) -> Result<pumpkin_util::jwt::PlayerClaims, LoginError> {
    let (issuer, jwks) = server
        .bedrock_oidc_keys
        .get()
        .ok_or(LoginError::ChainValidationFailed(
            AuthError::PublicKeyBuild("OIDC keys not initialized".into()),
        ))?;

    pumpkin_util::jwt::verify_oidc_token(token, issuer, jwks)
        .map_err(|e| LoginError::ChainValidationFailed(e))
}

/// Verifies certificate chains for legacy Bedrock clients (pre-1.26.10).
fn verify_certificate_chain_path(certificate: &str) -> Result<pumpkin_util::jwt::PlayerClaims, LoginError> {
    let inner_payload: CertificateChainPayload = serde_json::from_str(certificate)?;
    let chain_vec: Vec<&str> = inner_payload.chain.iter().map(String::as_str).collect();
    verify_chain(&chain_vec, MOJANG_BEDROCK_PUBLIC_KEY_BASE64)
        .map_err(|e| LoginError::ChainValidationFailed(e))
}

/// Routes authentication to the appropriate verification path based on available credentials.
async fn extract_player_data_from_auth(
    auth: &AuthPayload,
    server: &Server,
) -> Result<pumpkin_util::jwt::PlayerClaims, LoginError> {
    if let Some(token) = auth.token.as_ref().filter(|t| !t.is_empty()) {
        verify_oidc_token_path(server, token).await
    } else if let Some(certificate) = auth.certificate.as_ref().filter(|c| !c.is_empty()) {
        verify_certificate_chain_path(certificate)
    } else {
        Err(LoginError::ChainValidationFailed(
            AuthError::InvalidTokenFormat,
        ))
    }
}

impl BedrockClient {
    pub async fn handle_request_network_settings(&self, packet: SRequestNetworkSettings) {
        self.protocol_version
            .store(packet.protocol_version as u32, std::sync::atomic::Ordering::Relaxed);
        self.send_game_packet(&CNetworkSettings::new(0, 0, false, 0, 0.0))
            .await;
        self.set_compression(CompressionInfo::default()).await;
    }

    pub async fn handle_login(self: &Arc<Self>, packet: SLogin, server: &Server) -> Option<()> {
        match self.try_handle_login(packet, server).await {
            Ok(()) => Some(()),
            Err(error) => {
                warn!("Bedrock login failed: {error}");
                let message = match error {
                    LoginError::InvalidUsername => "Your username is invalid.".to_string(),
                    _ => "Failed to log in. The data sent by your client was invalid.".to_string(),
                };
                self.kick(DisconnectReason::LoginPacketNoRequest, message)
                    .await;
                None
            }
        }
    }

    pub async fn try_handle_login(
        self: &Arc<Self>,
        packet: SLogin,
        server: &Server,
    ) -> Result<(), LoginError> {
        let protocol_version = self.protocol_version.load(std::sync::atomic::Ordering::Relaxed);
        let player_data = if protocol_version == pumpkin_util::BedrockVersion::V1_26_0.protocol_version() {
            let outer_payload: FullLoginPayload = serde_json::from_slice(&packet.jwt)?;
            verify_certificate_chain_path(&outer_payload.certificate)?
        } else {
            let auth_payload: AuthPayload = serde_json::from_slice(&packet.jwt)?;
            extract_player_data_from_auth(&auth_payload, server).await?
        };

        let profile = GameProfile {
            id: Uuid::parse_str(&player_data.uuid).map_err(|_| LoginError::InvalidUuid)?,
            name: player_data.display_name,
            properties: Vec::new(),
            profile_actions: None,
        };

        let mut frame_set = FrameSet::default();

        self.write_game_packet_to_set(&CPlayStatus::LoginSuccess, &mut frame_set)
            .await;
        self.write_game_packet_to_set(
            &CResourcePacksInfo::new(false, false, false, false, Uuid::default(), String::new()),
            &mut frame_set,
        )
        .await;

        self.send_frame_set(frame_set, 0x84).await;

        if let Some((player, world)) = server
            .add_player(ClientPlatform::Bedrock(self.clone()), profile, None)
            .await
        {
            world
                .spawn_bedrock_player(&server.basic_config, player.clone(), server, self.bedrock_version())
                .await;
            *self.player.lock().await = Some(player);
        }

        Ok(())
    }

    pub async fn handle_resource_pack_response(&self, packet: SResourcePackResponse) {
        // TODO: Add all
        if packet.response == SResourcePackResponse::STATUS_HAVE_ALL_PACKS {
            debug!("Bedrock: STATUS_HAVE_ALL_PACKS");
            let mut frame_set = FrameSet::default();

            self.write_game_packet_to_set(
                &CResourcePackStackPacket::new(
                    false,
                    VarUInt(0),
                    CURRENT_BEDROCK_MC_VERSION.to_string(),
                    Experiments {
                        names_size: 0,
                        experiments_ever_toggled: false,
                    },
                    false,
                ),
                &mut frame_set,
            )
            .await;
            self.send_frame_set(frame_set, 0x84).await;
        }
    }
}
