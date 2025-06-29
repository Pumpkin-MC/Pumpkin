use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, CampfireLikeProperties},
    damage::DamageType,
    fluid::Fluid,
};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::{
    block::{
        BlockIsReplacing,
        pumpkin_block::{BlockMetadata, PumpkinBlock},
    },
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
};

pub struct CampfireBlock;

impl BlockMetadata for CampfireBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::CAMPFIRE.name, Block::SOUL_CAMPFIRE.name]
    }
}

#[async_trait]
impl PumpkinBlock for CampfireBlock {
    // TODO: cooking food on campfire (CampfireBlockEntity)
    async fn on_entity_collision(
        &self,
        _world: &Arc<World>,
        entity: &dyn EntityBase,
        _pos: BlockPos,
        block: Block,
        state: BlockState,
        _server: &Server,
    ) {
        if CampfireLikeProperties::from_state_id(state.id, &block).lit
            && entity.get_living_entity().is_some()
        {
            entity.damage(1.0, DamageType::CAMPFIRE).await;
        }
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let is_replacing_water = matches!(replacing, BlockIsReplacing::Water(_));
        let mut props = CampfireLikeProperties::from_state_id(block.default_state.id, block);
        props.waterlogged = is_replacing_water;
        props.signal_fire = is_signal_fire_base_block(&world.get_block(&block_pos.down()).await);
        props.lit = !is_replacing_water;
        props.facing = player.get_entity().get_horizontal_facing();
        props.to_state_id(block)
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        direction: BlockDirection,
        neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let mut props = CampfireLikeProperties::from_state_id(state, block);
        if props.waterlogged {
            props.lit = false;
            world
                .schedule_fluid_tick(block.id, *pos, Fluid::WATER.flow_speed as u16)
                .await;
        }

        if direction == BlockDirection::Down {
            props.signal_fire = is_signal_fire_base_block(&world.get_block(neighbor_pos).await);
        }

        props.to_state_id(block)
    }

    // TODO: onProjectileHit
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}
