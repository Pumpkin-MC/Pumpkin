use std::{num::NonZero, sync::Arc};

use pumpkin_protocol::bedrock::{
    client::chunk_radius_update::CChunkRadiusUpdate,
    server::request_chunk_radius::SRequestChunkRadius,
};

use crate::{entity::player::Player, net::bedrock::BedrockClientPlatform};

impl BedrockClientPlatform {
    pub async fn handle_request_chunk_radius(&self, player: &Arc<Player>, packet: SRequestChunkRadius) {
        dbg!(&packet);
        player.config.write().await.view_distance = NonZero::new(packet.chunk_radius.0 as u8).unwrap();
        self.send_game_packet(&CChunkRadiusUpdate {
            chunk_radius: packet.chunk_radius,
        })
        .await;
    }
}
