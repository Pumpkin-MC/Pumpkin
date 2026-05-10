use crate::{
    net::{ClientPlatform, DisconnectReason, GameProfile, PlayerConfig, bedrock::BedrockClient},
    server::Server,
};
use pumpkin_protocol::bedrock::{
    client::{
        network_settings::CNetworkSettings, play_status::CPlayStatus,
        resource_pack_stack::CResourcePackStackPacket, resource_packs_info::CResourcePacksInfo,
        start_game::Experiments,
    },
    frame_set::FrameSet,
    server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
};
use pumpkin_protocol::bedrock::{
    client::{resource_pack_stack::ResourcePackStackEntry, resource_packs_info::ResourcePackEntry},
    server::{login::ClientData, resource_pack_response::SResourcePackResponse},
};
use pumpkin_util::jwt::AuthError;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use serde::{Deserialize, de::Error};
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
    pub async fn handle_request_network_settings(
        &self,
        packet: SRequestNetworkSettings,
        server: &Server,
    ) {
        if packet.protocol_version < CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedClient).await;
            return;
        } else if packet.protocol_version > CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedServer).await;
            return;
        }
        let compression = server
            .advanced_config
            .networking
            .bedrock_compression
            .info
            .clone();
        self.send_game_packet(&CNetworkSettings::new(
            compression.threshold as u16,
            0,
            false,
            0,
            0.0,
        ))
        .await;
        self.set_compression(compression).await;
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

        let raw_token_str = std::str::from_utf8(&packet.raw_token).map_err(|_| {
            LoginError::InvalidTokenFormat(serde_json::Error::custom(
                "raw_token is not valid UTF-8",
            ))
        })?; // You'll need to add a string conversion error to LoginError, or handle it cleanly.

        let mut parts = raw_token_str.split('.');
        let _header = parts.next().ok_or(AuthError::InvalidTokenFormat)?;
        let payload_b64 = parts.next().ok_or(AuthError::InvalidTokenFormat)?;

        let payload_bytes = pumpkin_util::jwt::decode_b64_url_nopad(payload_b64)
            .map_err(|_| LoginError::DecodeExtraError)?;
        let client_data: ClientData = serde_json::from_slice(&payload_bytes)?;

        let real_name = player_data.display_name;
        // IMPORTANT: Bedrock allows spaces in names. While we could support this, it would significantly complicate parsing player arguments in commands, so we don't
        let under_score_name = real_name.replace(' ', "_");

        let profile = GameProfile {
            id: Uuid::parse_str(&player_data.uuid).map_err(|_| LoginError::InvalidUuid)?,
            name: under_score_name,
            properties: Vec::new(),
            profile_actions: None,
        };

        let mut frame_set = FrameSet::default();

        self.write_game_packet_to_set(&CPlayStatus::LoginSuccess, &mut frame_set)
            .await;
        let br_config = &server.advanced_config.resource_pack.bedrock;

        let mut entries = Vec::new();
        if br_config.enabled {
            for pack in &br_config.packs {
                entries.push(ResourcePackEntry {
                    uuid: pack.uuid.to_string(),
                    version: pack.version.clone(),
                    size: pack.size,
                    download_url: pack.download_url.clone(),
                    content_key: pack.content_key.clone(),
                    sub_pack_name: pack.sub_pack_name.clone(),
                    content_id: pack.content_id.clone(),
                    has_scripts: pack.has_scripts,
                    addon_pack: pack.addon_pack,
                    rtx_enabled: pack.rtx_enabled,
                });
            }
        }

        let packs_info = CResourcePacksInfo {
            resource_pack_required: br_config.force,
            has_addon_packs: false,
            has_scripts: false,
            is_vibrant_visuals_force_disabled: false,
            world_template_id: uuid::Uuid::nil(),
            world_template_version: String::new(),
            resource_packs: entries,
        };
        self.write_game_packet_to_set(&packs_info, &mut frame_set)
            .await;

        self.send_frame_set(frame_set, 0x84).await;

        if let Some((player, _world)) = server
            .add_player(ClientPlatform::Bedrock(self.clone()), profile, None)
            .await
        {
            // TODO: kinda sad we don't use more of client_data, we should store it somewhere, at least for plugin devs
            let new_config = PlayerConfig {
                locale: client_data.language_code,
                ..Default::default()
            };

            player.config.store(std::sync::Arc::new(new_config));

            // player spawn happens after resource packs are resolved
            *self.player.lock().await = Some(player);
        }

        Ok(())
    }

    pub async fn handle_resource_pack_response(
        &self,
        packet: SResourcePackResponse,
        server: &Server,
    ) {
        // TODO: warn & ignore if the player is already spawned in

        match packet.response {
            SResourcePackResponse::STATUS_REFUSED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_REFUSED");
                self.kick(
                    DisconnectReason::ResourcePackProblem,
                    "You must accept resource packs to join this server.".into(),
                )
                .await;
            }
            SResourcePackResponse::STATUS_SEND_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_SEND_PACKS");
                // TODO: send packs
            }
            SResourcePackResponse::STATUS_HAVE_ALL_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_HAVE_ALL_PACKS");
                let mut frame_set = FrameSet::default();

                let br_config = &server.advanced_config.resource_pack.bedrock;

                // Convert your config packs into protocol stack entries
                let resource_packs = if br_config.enabled {
                    br_config
                        .packs
                        .iter()
                        .map(|pack| ResourcePackStackEntry {
                            uuid: pack.uuid.to_string(),
                            version: pack.version.clone(),
                            sub_pack_name: String::new(),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                self.write_game_packet_to_set(
                    &CResourcePackStackPacket::new(
                        br_config.force,
                        resource_packs,
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
            SResourcePackResponse::STATUS_COMPLETED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_COMPLETED");
                if let Some(player) = &*self.player.lock().await {
                    player
                        .world()
                        .spawn_bedrock_player(&server.basic_config, player.clone(), server)
                        .await;
                } else {
                    tracing::error!(
                        "Got SResourcePackResponse::STATUS_COMPLETED before authentication was completed."
                    );
                    self.kick(DisconnectReason::Disconnected, String::new())
                        .await;
                }
            }
            _ => {
                tracing::error!("Bedrock: SResourcePackResponse bad response type");
                self.kick(DisconnectReason::Disconnected, String::new())
                    .await;
            }
        }
    }
}
