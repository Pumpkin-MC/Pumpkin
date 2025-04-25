use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

#[pumpkin_block("minecraft:powered_rail")]
pub struct PoweredRailBlock;

#[async_trait]
impl PumpkinBlock for PoweredRailBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player: &Player,
        _other: bool,
    ) -> BlockStateId {
        let mut rail_props = RailProperties::default(block);
        let player_facing = player.living_entity.entity.get_horizontal_facing();

        rail_props
            .set_straight_shape(compute_placed_rail_shape(world, block_pos, player_facing).await);

        rail_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        update_flanking_rails_shape(world, block, state_id, block_pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        if !rail_placement_is_valid(world, block, block_pos).await {
            world
                .break_block(block_pos, None, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }

    async fn can_place_at(&self, world: &World, pos: &BlockPos) -> bool {
        can_place_rail_at(world, pos).await
    }
}
