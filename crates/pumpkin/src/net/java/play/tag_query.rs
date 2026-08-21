#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::{
    client::play::CTagQueryResponse,
    server::play::{SBlockEntityTagQuery, SEntityTagQuery},
};

impl JavaClient {
    pub async fn handle_block_entity_tag_query(
        &self,
        player: &Player,
        packet: SBlockEntityTagQuery,
    ) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        // Empty NBT compound (TAG_End = 0)
        let empty_nbt = [0u8];
        self.send_packet(&CTagQueryResponse::new(packet.transaction_id, &empty_nbt))
            .await;
    }

    pub async fn handle_entity_tag_query(&self, player: &Player, packet: SEntityTagQuery) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        // Empty NBT compound (TAG_End = 0)
        let empty_nbt = [0u8];
        self.send_packet(&CTagQueryResponse::new(packet.transaction_id, &empty_nbt))
            .await;
    }
}
