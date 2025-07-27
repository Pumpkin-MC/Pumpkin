use std::sync::Arc;

use base64::{Engine, engine::general_purpose};
use pumpkin_config::{BASIC_CONFIG, networking::compression::CompressionInfo};
use pumpkin_protocol::{
    bedrock::{
        client::{
            network_settings::CNetworkSettings, play_status::CPlayStatus,
            resource_pack_stack::CResourcePackStackPacket, resource_packs_info::CResourcePacksInfo,
            start_game::Experiments,
        },
        server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
    },
    codec::var_uint::VarUInt,
};
use serde_json::Value;

use crate::{
    net::{ClientPlatform, DisconnectReason, GameProfile, bedrock::BedrockClient},
    server::{CURRENT_BEDROCK_MC_VERSION, Server},
};

impl BedrockClient {
    pub async fn handle_request_network_settings(&self, _packet: SRequestNetworkSettings) {
        self.send_game_packet(&CNetworkSettings::new(0, 0, false, 0, 0.0))
            .await;
        self.set_compression(CompressionInfo::default()).await;
    }
    pub async fn handle_login(self: &Arc<Self>, packet: SLogin, server: &Server) -> Option<()> {
        // This is a mess to extract the PlayerName
        // TODO Verify Player
        // This also contains the public key for encryption!
        let jwt = unsafe { String::from_utf8_unchecked(packet.jwt) };
        let i = jwt
            .split_once('[')
            .unwrap()
            .1
            .split_once(']')
            .unwrap()
            .0
            .split(',')
            .collect::<Vec<&str>>();

        // TODO Make this right!!!
        if i.len() != 3 {
            self.kick(
                DisconnectReason::LoginPacketNoRequest,
                "Something went wrong try again".to_string(),
            )
            .await;
            return None;
        }

        let i = i[2].split_once('\"').unwrap().1.split_once('\"').unwrap().0;
        let parts: Vec<&str> = i.split('.').collect();

        let payload = unsafe {
            String::from_utf8_unchecked(general_purpose::URL_SAFE_NO_PAD.decode(parts[1]).unwrap())
        };
        let payload: Value = serde_json::from_str(&payload).unwrap();

        // TODO
        let profile = GameProfile {
            id: uuid::Uuid::parse_str(payload["extraData"]["identity"].as_str().unwrap()).unwrap(),
            name: payload["extraData"]["displayName"]
                .as_str()
                .unwrap()
                .to_string(),
            properties: Vec::new(),
            profile_actions: None,
        };

        //let raw_token = unsafe { String::from_utf8_unchecked(packet.raw_token) };
        //let raw_token: Vec<&str> = raw_token.split('.').collect();
        // We dont care about the validation, we just want to get the data
        //let _raw_token = unsafe {
        //    String::from_utf8_unchecked(general_purpose::URL_SAFE_NO_PAD.decode(raw_token[1]).unwrap())
        //};

        // TODO: Batch these
        self.send_game_packet(&CPlayStatus::LoginSuccess).await;
        self.send_game_packet(&CResourcePacksInfo::new(
            false,
            false,
            false,
            false,
            uuid::Uuid::default(),
            String::new(),
        ))
        .await;
        self.send_game_packet(&CResourcePackStackPacket::new(
            false,
            VarUInt(0),
            VarUInt(0),
            CURRENT_BEDROCK_MC_VERSION.to_string(),
            Experiments {
                names_size: 0,
                experiments_ever_toggled: false,
            },
            false,
        ))
        .await;

        if let Some((player, world)) = server
            .add_player(
                ClientPlatform::Bedrock(self.clone()),
                profile,
                None, // TODO
            )
            .await
        {
            world
                .spawn_bedrock_player(&BASIC_CONFIG, player.clone(), server)
                .await;
            *self.player.lock().await = Some(player);
        }

        Some(())
    }
}
