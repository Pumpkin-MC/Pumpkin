use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::HorizontalFacingExt;
use std::sync::Arc;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

use super::{
    HorizontalFacingRailExt, Rail, RailElevation, RailProperties, compute_neighbor_rail_new_shape,
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

        rail_props.set_shape(
            player
                .living_entity
                .entity
                .get_horizontal_facing()
                .to_rail_shape_flat(),
        );

        for direction in rail_props.directions() {
            if let Some(neighbor_rail) = Rail::find_if_unlocked(world, block_pos, direction).await {
                if neighbor_rail.elevation == RailElevation::Up {
                    rail_props.set_shape(direction.to_rail_shape_ascending_towards());
                }

                return rail_props.to_state_id(block);
            }
        }

        for direction in rail_props.directions().iter().rev() {
            let direction = direction.rotate_clockwise();
            if let Some(neighbor_rail) = Rail::find_if_unlocked(world, block_pos, direction).await {
                if neighbor_rail.elevation == RailElevation::Up {
                    rail_props.set_shape(direction.to_rail_shape_ascending_towards());
                } else {
                    rail_props.set_shape(direction.to_rail_shape_flat());
                }

                return rail_props.to_state_id(block);
            }
        }

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
        for direction in RailProperties::new(state_id, block).directions() {
            let Some(mut neighbor_rail) =
                Rail::find_with_elevation(world, block_pos.offset(direction.to_offset())).await
            else {
                // Skip non-rail blocks
                continue;
            };

            let new_shape =
                compute_neighbor_rail_new_shape(world, &direction, &neighbor_rail).await;

            if new_shape != neighbor_rail.properties.shape() {
                neighbor_rail.properties.set_shape(new_shape);
                world
                    .set_block_state(
                        &neighbor_rail.position,
                        neighbor_rail.properties.to_state_id(&neighbor_rail.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
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
        if !self.can_place_at(world, pos).await {
            world.break_block(pos, None, BlockFlags::NOTIFY_ALL).await;
        }
    }

    async fn can_place_at(&self, world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(&pos.down()).await.unwrap();
        state.is_solid
    }
}
