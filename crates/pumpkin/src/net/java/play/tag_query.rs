#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_nbt::{Nbt, compound::NbtCompound};
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

        let mut compound = NbtCompound::new();
        if let Some(block_entity) = player.world().get_block_entity(&packet.location) {
            block_entity.write_nbt(&mut compound);
        }

        let nbt_bytes = Nbt::new(String::new(), compound).write_unnamed();
        self.send_packet(&CTagQueryResponse::new(packet.transaction_id, &nbt_bytes))
            .await;
    }

    pub async fn handle_entity_tag_query(&self, player: &Player, packet: SEntityTagQuery) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        let mut compound = NbtCompound::new();
        if let Some(entity) = player.world().get_entity_by_id(packet.entity_id.0) {
            entity.write_nbt(&mut compound);
        }

        let nbt_bytes = Nbt::new(String::new(), compound).write_unnamed();
        self.send_packet(&CTagQueryResponse::new(packet.transaction_id, &nbt_bytes))
            .await;
    }
}
