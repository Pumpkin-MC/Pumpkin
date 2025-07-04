use std::sync::Arc;

use pumpkin_config::{BASIC_CONFIG, networking::compression::CompressionInfo};
use pumpkin_protocol::{
    bedrock::{
        RakReliability,
        client::{
            network_settings::CNetworkSettings,
            play_status::{CPlayStatus, PlayStatus},
            resource_pack_stack::CResourcePackStackPacket,
            resource_packs_info::CResourcePacksInfo,
        },
        server::{login::SLogin, request_network_settings::SRequestNetworkSettings},
    },
    codec::var_uint::VarUInt,
};

use crate::{
    net::{ClientPlatform, GameProfile, bedrock::BedrockClientPlatform},
    server::Server,
};

impl BedrockClientPlatform {
    pub async fn handle_request_network_settings(&self, packet: SRequestNetworkSettings) {
        dbg!("requested network settings");
        self.protocol_version.store(
            packet.protocol_version,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.send_game_packet(
            &CNetworkSettings::new(0, 0, false, 0, 0.0),
            RakReliability::Unreliable,
        )
        .await;
        self.set_compression(CompressionInfo::default()).await;
    }
    pub async fn handle_login(self: &Arc<Self>, _packet: SLogin, server: &Server) {
        dbg!("received login");
        // TODO: Enable encryption
        // bedrock
        //     .send_game_packet(
        //         self,
        //         &CHandshake::new(packet.connection_request),
        //         RakReliability::Unreliable,
        //     )
        //     .await;
        // TODO: Batch these
        self.send_game_packet(
            &CPlayStatus::new(PlayStatus::LoginSuccess),
            RakReliability::Unreliable,
        )
        .await;
        self.send_game_packet(
            &CResourcePacksInfo::new(
                false,
                false,
                false,
                false,
                uuid::Uuid::new_v4(),
                "1.21.93".to_string(),
            ),
            RakReliability::Unreliable,
        )
        .await;
        self.send_game_packet(
            &CResourcePackStackPacket::new(
                false,
                VarUInt(0),
                VarUInt(0),
                "1.21.92".to_string(),
                0,
                false,
                false,
            ),
            RakReliability::Unreliable,
        )
        .await;

        // TODO
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "Todo Name".to_string(),
            properties: Vec::new(),
            profile_actions: None,
        };

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
    }
}
