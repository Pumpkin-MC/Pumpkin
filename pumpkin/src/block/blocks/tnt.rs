use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::entity::tnt::TNTEntity;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;

#[pumpkin_block("minecraft:tnt")]
pub struct TNTBlock;

#[async_trait]
impl PumpkinBlock for TNTBlock {
    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        if *item != Item::FLINT_AND_STEEL || *item == Item::FIRE_CHARGE {
            return BlockActionResult::Continue;
        }
        let world = player.world().await;
        world.break_block(server, &location, None, false).await;
        let entity = server.add_entity(location.0.to_f64(), EntityType::TNT, &world);
        let tnt = Arc::new(TNTEntity::new(entity, 4.0, 80));
        world.spawn_entity(tnt.clone()).await;
        tnt.send_meta_packet().await;
        BlockActionResult::Consume
    }
}
