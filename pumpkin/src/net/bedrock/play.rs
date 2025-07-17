use std::{num::NonZero, sync::Arc};

use pumpkin_protocol::bedrock::{
    client::chunk_radius_update::CChunkRadiusUpdate,
    server::{player_auth_input::SPlayerAuthInput, request_chunk_radius::SRequestChunkRadius},
};

use crate::{entity::player::Player, net::bedrock::BedrockClient};

impl BedrockClient {
    pub async fn handle_request_chunk_radius(
        &self,
        player: &Arc<Player>,
        packet: SRequestChunkRadius,
    ) {
        println!("requestet view_distance: {}", packet.chunk_radius.0);
        player.config.write().await.view_distance =
            NonZero::new(packet.chunk_radius.0 as u8).unwrap();
        self.send_game_packet(&CChunkRadiusUpdate {
            chunk_radius: packet.chunk_radius,
        })
        .await;
    }

    pub async fn player_pos_update(&self, _player: &Arc<Player>, _packet: SPlayerAuthInput) {
        //println!("{:?}", packet)
        //self.send_game_packet(&CMovePlayer {
        //     player_runtime_id: VarULong(player.entity_id() as u64),
        //    position: packet.position + Vector3::new(10.0, 0.0, 0.0),
        //    pitch: packet.pitch,
        //    yaw: packet.yaw,
        //    y_head_rotation: packet.head_rotation,
        //    position_mode: 1,
        //    on_ground: false,
        //    riding_runtime_id: VarULong(0),
        //    tick: packet.client_tick,
        //})
        //.await;
    }
}
