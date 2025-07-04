use pumpkin_config::networking::compression::CompressionInfo;
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

use crate::net::{Client, bedrock::BedrockClientPlatform};

impl Client {
    pub async fn handle_request_network_settings(
        &self,
        bedrock: &BedrockClientPlatform,
        packet: SRequestNetworkSettings,
    ) {
        dbg!("requested network settings");
        self.protocol_version.store(
            packet.protocol_version,
            std::sync::atomic::Ordering::Relaxed,
        );
        bedrock
            .send_game_packet(
                self,
                &CNetworkSettings::new(0, 0, false, 0, 0.0),
                RakReliability::Unreliable,
            )
            .await;
        bedrock.set_compression(CompressionInfo::default()).await;
    }
    pub async fn handle_login(&self, bedrock: &BedrockClientPlatform, _packet: SLogin) {
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
        bedrock
            .send_game_packet(
                self,
                &CPlayStatus::new(PlayStatus::LoginSuccess),
                RakReliability::Unreliable,
            )
            .await;
        bedrock
            .send_game_packet(
                self,
                &CResourcePacksInfo::new(false, false, false),
                RakReliability::Unreliable,
            )
            .await;
        bedrock
            .send_game_packet(
                self,
                &CResourcePackStackPacket::new(false, VarUInt(0)),
                RakReliability::Unreliable,
            )
            .await;
    }
}
