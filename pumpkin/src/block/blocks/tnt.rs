use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::entity::tnt::TNTEntity;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::SoundCategory;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use rand::Rng;

use super::redstone::block_receives_redstone_power;

#[pumpkin_block("minecraft:tnt")]
pub struct TNTBlock;

impl TNTBlock {
    pub async fn prime(world: &Arc<World>, location: &BlockPos) {
        let entity = world.create_entity(location.to_f64(), EntityType::TNT);
        let pos = entity.pos.load();
        let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, DEFAULT_FUSE));
        world.spawn_entity(tnt).await;
        world
            .play_sound(
                pumpkin_data::sound::Sound::EntityTntPrimed,
                SoundCategory::Blocks,
                &pos,
            )
            .await;
        world
            .set_block_state(location, 0, BlockFlags::NOTIFY_ALL)
            .await;
    }
}

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
        _server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        if *item != Item::FLINT_AND_STEEL || *item == Item::FIRE_CHARGE {
            return BlockActionResult::Continue;
        }
        let world = player.world().await;
        Self::prime(&world, &location).await;

        BlockActionResult::Consume
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if block_receives_redstone_power(world, pos).await {
            Self::prime(world, pos).await;
        }
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        _block: &Block,
        pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        if block_receives_redstone_power(world, pos).await {
            Self::prime(world, pos).await;
        }
    }

    async fn explode(&self, _block: &Block, world: &Arc<World>, location: BlockPos) {
        let entity = world.create_entity(location.to_f64(), EntityType::TNT);
        let angle = rand::random::<f64>() * std::f64::consts::TAU;
        entity
            .set_velocity(Vector3::new(-angle.sin() * 0.02, 0.2, -angle.cos() * 0.02))
            .await;
        let fuse = rand::thread_rng().gen_range(0..DEFAULT_FUSE / 4) + DEFAULT_FUSE / 8;
        let tnt = Arc::new(TNTEntity::new(entity, DEFAULT_POWER, fuse));
        world.spawn_entity(tnt).await;
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        false
    }
}
