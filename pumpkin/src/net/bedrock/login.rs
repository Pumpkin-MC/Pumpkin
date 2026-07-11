use crate::{
    net::{
        DisconnectReason, GameProfile, PacketHandlerResult, PlayerConfig, bedrock::BedrockClient,
    },
    server::Server,
};
use arc_swap::ArcSwap;
use pumpkin_protocol::bedrock::client::handshake::CHandshake;
use pumpkin_protocol::bedrock::{
    client::{
        network_settings::CNetworkSettings, play_status::CPlayStatus,
        resource_pack_stack::CResourcePackStackPacket, resource_packs_info::CResourcePacksInfo,
        start_game::Experiments,
    },
    server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
};
use pumpkin_protocol::bedrock::{
    client::{resource_pack_stack::ResourcePackStackEntry, resource_packs_info::ResourcePackEntry},
    server::{login::ClientData, resource_pack_response::SResourcePackResponse},
};
use pumpkin_util::jwt::AuthError;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use rand::RngExt;
use serde::{Deserialize, de::Error};
use serde_repr::Deserialize_repr;
use sha2::{Digest, Sha256};
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

        self.version.store(BedrockMinecraftVersion::from_protocol(
            packet.protocol_version as u32,
        ));

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

    pub async fn handle_login(
        self: &Arc<Self>,
        packet: SLogin,
        server: &Server,
    ) -> Result<Option<PacketHandlerResult>, LoginError> {
        self.try_handle_login(packet, server).await
    }

    #[expect(clippy::too_many_lines)]
    pub async fn try_handle_login(
        self: &Arc<Self>,
        packet: SLogin,
        server: &Server,
    ) -> Result<Option<PacketHandlerResult>, LoginError> {
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
            properties: ArcSwap::new(Arc::new(Vec::new())),
            profile_actions: None,
        };

        if server.basic_config.bedrock_encryption {
            let client_public_key = pumpkin_util::jwt::extract_cpk_from_token(&auth_payload.token)
                .map_err(LoginError::ChainValidationFailed)?;

            let (server_private_key, salt) = {
                let server_key_arc = server
                    .bedrock_private_key
                    .get_or_init(|| async {
                        let mut rng = rand::rng();
                        loop {
                            let mut private_key_bytes = [0u8; 48];
                            for b in &mut private_key_bytes {
                                *b = rng.random();
                            }
                            if let Ok(key) = pumpkin_util::p384::ecdsa::SigningKey::from_slice(
                                &private_key_bytes,
                            ) {
                                break Arc::new(key);
                            }
                        }
                    })
                    .await
                    .clone();

                let mut salt = [0u8; 16];
                for b in &mut salt {
                    *b = rand::rng().random();
                }

                (server_key_arc, salt)
            };

            let handshake_jwt =
                pumpkin_util::jwt::generate_handshake_jwt(&server_private_key, &salt)
                    .map_err(LoginError::ChainValidationFailed)?;
            let handshake_packet = CHandshake::new(handshake_jwt);
            self.send_game_packet(&handshake_packet).await;

            let shared_secret =
                pumpkin_util::jwt::compute_shared_secret(&server_private_key, &client_public_key);
            let mut hasher = Sha256::new();
            hasher.update(salt);
            hasher.update(shared_secret);
            let key_bytes: [u8; 32] = hasher.finalize().into();

            self.network_reader
                .lock()
                .await
                .set_encryption(&key_bytes)
                .map_err(|_| {
                    LoginError::ChainValidationFailed(AuthError::PublicKeyBuild(
                        "encryption enable error".into(),
                    ))
                })?;
            self.network_writer
                .write()
                .await
                .set_encryption(&key_bytes)
                .map_err(|_| {
                    LoginError::ChainValidationFailed(AuthError::PublicKeyBuild(
                        "encryption enable error".into(),
                    ))
                })?;

            let new_config = PlayerConfig {
                locale: client_data.language_code.clone(),
                ..Default::default()
            };
            self.client_data
                .store(std::sync::Arc::new(Some(std::sync::Arc::new(client_data))));
            *self.pending_profile.lock().await = Some((profile, new_config));
            return Ok(None);
        }

        self.enqueue_packet_internal(&CPlayStatus::LoginSuccess)
            .await;
        let br_config = &server.advanced_config.resource_pack.bedrock;

        let mut entries = Vec::new();
        if br_config.enabled {
            for pack in &br_config.packs {
                entries.push(ResourcePackEntry {
                    uuid: pack.uuid,
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
        self.enqueue_packet_internal(&packs_info).await;

        let new_config = PlayerConfig {
            locale: client_data.language_code.clone(),
            ..Default::default()
        };

        self.client_data
            .store(std::sync::Arc::new(Some(std::sync::Arc::new(client_data))));

        Ok(Some(PacketHandlerResult::ReadyToPlay(profile, new_config)))
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

                self.enqueue_packet_internal(&CResourcePackStackPacket::new(
                    br_config.force,
                    resource_packs,
                    CURRENT_BEDROCK_MC_VERSION.to_string(),
                    Experiments {
                        names_size: 0,
                        experiments_ever_toggled: false,
                    },
                    false,
                ))
                .await;
            }
            SResourcePackResponse::STATUS_COMPLETED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_COMPLETED");
                let player = self.player.lock().await.clone();
                if let Some(player) = player {
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
