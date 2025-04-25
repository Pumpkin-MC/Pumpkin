use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::RailShape;
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

use super::RailElevation;
use super::{Rail, RailProperties, compute_neighbor_rail_new_shape};

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

        rail_props.set_shape(match player.living_entity.entity.get_horizontal_facing() {
            HorizontalFacing::North => RailShape::NorthSouth,
            HorizontalFacing::South => RailShape::NorthSouth,
            HorizontalFacing::West => RailShape::EastWest,
            HorizontalFacing::East => RailShape::EastWest,
        });

        for direction in rail_props.directions() {
            if let Some(neighbor_rail) = Rail::find_if_unlocked(world, block_pos, direction).await {
                if neighbor_rail.elevation == RailElevation::Up {
                    rail_props.set_shape(match direction {
                        BlockDirection::South => RailShape::AscendingSouth,
                        BlockDirection::North => RailShape::AscendingNorth,
                        BlockDirection::East => RailShape::AscendingEast,
                        BlockDirection::West => RailShape::AscendingWest,
                        _ => unreachable!("Bruh"),
                    });
                }

                return rail_props.to_state_id(block);
            }
        }

        for direction in rail_props
            .directions()
            .iter()
            .map(|d| d.rotate_clockwise())
            .rev()
        {
            if let Some(neighbor_rail) = Rail::find_if_unlocked(world, block_pos, direction).await {
                if neighbor_rail.elevation == RailElevation::Up {
                    rail_props.set_shape(match direction {
                        BlockDirection::South => RailShape::AscendingSouth,
                        BlockDirection::North => RailShape::AscendingNorth,
                        BlockDirection::East => RailShape::AscendingEast,
                        BlockDirection::West => RailShape::AscendingWest,
                        _ => unreachable!("Bruh"),
                    });
                } else {
                    rail_props.set_shape(match direction {
                        BlockDirection::North => RailShape::NorthSouth,
                        BlockDirection::South => RailShape::NorthSouth,
                        BlockDirection::West => RailShape::EastWest,
                        BlockDirection::East => RailShape::EastWest,
                        _ => unreachable!("Bruh"),
                    });
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
