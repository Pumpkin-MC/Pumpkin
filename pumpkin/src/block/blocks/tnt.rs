use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::entity::tnt::TNTEntity;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use rand::Rng;

#[pumpkin_block("minecraft:tnt")]
pub struct TNTBlock;

const DEFAULT_FUSE: u32 = 80;
const DEFAULT_POWER: f32 = 4.0;

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
        let entity = server.add_entity(location.to_f64(), EntityType::TNT, &world);
        let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, DEFAULT_FUSE));
        world.spawn_entity(tnt.clone()).await;
        tnt.send_meta_packet().await;
        BlockActionResult::Consume
    }
    async fn explode(
        &self,
        _block: &Block,
        world: &Arc<World>,
        location: BlockPos,
        server: &Server,
    ) {
        let entity = server.add_entity(location.to_f64(), EntityType::TNT, world);
        let fuse = rand::thread_rng().gen_range(0..DEFAULT_FUSE / 4) + DEFAULT_FUSE / 8;
        let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, fuse));
        world.spawn_entity(tnt.clone()).await;
        tnt.send_meta_packet().await;
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        false
    }
}
