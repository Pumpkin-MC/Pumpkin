use crate::net::authentication::MOJANG_BEDROCK_PUBLIC_KEY_BASE64;
use crate::{
    net::{ClientPlatform, DisconnectReason, GameProfile, bedrock::BedrockClient},
    server::Server,
};
use pumpkin_config::networking::compression::CompressionInfo;
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

#[derive(Deserialize, Debug)]
struct CertificateChainPayload {
    chain: Vec<String>,
}

impl BedrockClient {
    pub async fn handle_request_network_settings(&self, _packet: SRequestNetworkSettings) {
        self.send_game_packet(&CNetworkSettings::new(0, 0, false, 0, 0.0))
            .await;
        self.set_compression(CompressionInfo::default()).await;
    }

    pub async fn handle_login(self: &Arc<Self>, packet: SLogin, server: &Server) -> Option<()> {
        match self.try_handle_login(packet, server).await {
            Ok(()) => Some(()),
            Err(error) => {
                log::warn!("Bedrock login failed: {error}");
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
        let outer_payload: FullLoginPayload = serde_json::from_slice(&packet.jwt)?;
        let inner_payload: CertificateChainPayload =
            serde_json::from_str(&outer_payload.certificate)?;

        let chain_vec: Vec<&str> = inner_payload.chain.iter().map(String::as_str).collect();
        let player_data = verify_chain(&chain_vec, MOJANG_BEDROCK_PUBLIC_KEY_BASE64)?;

        let profile = GameProfile {
            id: Uuid::parse_str(&player_data.uuid).map_err(|_| LoginError::InvalidUuid)?,
            name: player_data.display_name,
            properties: Vec::new(),
            profile_actions: None,
        };

        //let raw_token = unsafe { String::from_utf8_unchecked(packet.raw_token) };
        //let raw_token: Vec<&str> = raw_token.split('.').collect();
        // We dont care about the validation, we just want to get the data
        //let _raw_token = unsafe {
        //    String::from_utf8_unchecked(general_purpose::URL_SAFE_NO_PAD.decode(raw_token[1]).unwrap())
        //};

        let mut frame_set = FrameSet::default();

        self.write_game_packet_to_set(&CPlayStatus::LoginSuccess, &mut frame_set)
            .await;
        self.write_game_packet_to_set(
            &CResourcePacksInfo::new(
                false,
                false,
                false,
                false,
                uuid::Uuid::default(),
                String::new(),
                Vec::new(),
            ),
            &mut frame_set,
        )
        .await;
        self.write_game_packet_to_set(
            &CResourcePackStackPacket::new(
                false,
                VarUInt(0),
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
}
