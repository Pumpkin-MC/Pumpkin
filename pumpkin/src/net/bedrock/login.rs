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
use pumpkin_util::jwt::AuthError;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};
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

/// Verifies OIDC tokens for Bedrock 1.26.10+ clients.
fn verify_oidc_token_path(
    server: &Server,
    token: &str,
    self_signed: bool,
) -> Result<pumpkin_util::jwt::PlayerClaims, LoginError> {
    if self_signed {
        pumpkin_util::jwt::verify_oidc_token_self_signed(token)
            .map_err(LoginError::ChainValidationFailed)
    } else {
        let (issuer, jwks) =
            server
                .bedrock_oidc_keys
                .get()
                .ok_or(LoginError::ChainValidationFailed(
                    AuthError::PublicKeyBuild("OIDC keys not initialized".into()),
                ))?;

        pumpkin_util::jwt::verify_oidc_token(token, issuer, jwks)
            .map_err(LoginError::ChainValidationFailed)
    }
}

impl BedrockClient {
    pub async fn handle_request_network_settings(&self, packet: SRequestNetworkSettings) {
        if packet.protocol_version < CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedClient).await;
            return;
        } else if packet.protocol_version > CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedServer).await;
            return;
        }
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
        let auth_payload: AuthPayload = serde_json::from_slice(&packet.jwt)?;
        let player_data = if server.basic_config.online_mode {
            match auth_payload.authentication_type {
                AuthenticationType::Full => {
                    verify_oidc_token_path(server, &auth_payload.token, false)?
                }
                AuthenticationType::SelfSigned => {
                    if server.advanced_config.networking.authentication.enabled {
                        return Err(LoginError::SelfSignedNotAllowed);
                    }

                    verify_oidc_token_path(server, &auth_payload.token, true)?
                }
                AuthenticationType::Guest => {
                    return Err(LoginError::GuestUnimplemented);
                }
            }
        } else {
            pumpkin_util::jwt::extract_oidc_token_player_claims(&auth_payload.token)?
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
                .spawn_bedrock_player(&server.basic_config, player.clone(), server)
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
