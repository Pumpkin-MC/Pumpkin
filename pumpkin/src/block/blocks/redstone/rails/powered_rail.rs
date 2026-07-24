use pumpkin_data::BlockStateId;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::entity::EntityBase;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

use super::super::block_receives_redstone_power;
use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

// Vanilla 26.2 PoweredRailBlock (CFR):
// updateState only recomputes *this* rail:
//   shouldPower = hasNeighborSignal(pos)
//              || findPoweredRailSignal(forward, 0)
//              || findPoweredRailSignal(backward, 0)
// isSameRailWithPower continues only through already-POWERED rails (depth < 8)
// and returns true if a rail in the chain hasNeighborSignal.
// Neighbor rails re-evaluate via block updates — do NOT BFS the whole track here.

#[pumpkin_block("minecraft:powered_rail")]
pub struct PoweredRailBlock;

impl BlockBehaviour for PoweredRailBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut rail_props = RailProperties::default(args.block);
            let player_facing = args.player.get_entity().get_horizontal_facing();

            rail_props.set_waterlogged(args.replacing.water_source());
            rail_props.set_straight_shape(
                compute_placed_rail_shape(args.world, args.position, player_facing).await,
            );

            rail_props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            update_flanking_rails_shape(args.world, args.block, args.state_id, args.position).await;
            // Vanilla BaseRailBlock.onPlace → updateState(this only).
            self.update_powered_state(args.world, args.block, args.position)
                .await;
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !rail_placement_is_valid(args.world, args.block, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
                return;
            }

            // Vanilla BaseRailBlock.neighborChanged → PoweredRailBlock.updateState(self only).
            // Cascade happens through NOTIFY when this rail's POWERED flips.
            self.update_powered_state(args.world, args.block, args.position)
                .await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla rail removal: notify neighbors; they recompute via neighborChanged.
            let rail_props = RailProperties::new(args.old_state_id, args.block);
            if rail_props.shape().is_ascending() {
                args.world
                    .update_neighbor(&args.position.up(), args.block)
                    .await;
            }
            args.world.update_neighbor(args.position, args.block).await;
            args.world
                .update_neighbor(&args.position.down(), args.block)
                .await;
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position)
    }
}

impl PoweredRailBlock {
    async fn is_powered_by_other_rails(
        &self,
        world: &World,
        pos: &BlockPos,
        state: &RailProperties,
        direction: bool,
        distance: u8,
    ) -> bool {
        if distance >= 8 {
            return false;
        }

        let mut x = pos.0.x;
        let mut y = pos.0.y;
        let mut z = pos.0.z;
        let mut check_down = true;
        let mut next_shape = state.shape();

        match next_shape {
            pumpkin_data::block_properties::RailShape::NorthSouth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                }
            }
            pumpkin_data::block_properties::RailShape::EastWest => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                }
            }
            pumpkin_data::block_properties::RailShape::AscendingEast => {
                if direction {
                    x -= 1;
                } else {
                    x += 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingWest => {
                if direction {
                    x -= 1;
                    y += 1;
                    check_down = false;
                } else {
                    x += 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::EastWest;
            }
            pumpkin_data::block_properties::RailShape::AscendingNorth => {
                if direction {
                    z += 1;
                } else {
                    z -= 1;
                    y += 1;
                    check_down = false;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            pumpkin_data::block_properties::RailShape::AscendingSouth => {
                if direction {
                    z += 1;
                    y += 1;
                    check_down = false;
                } else {
                    z -= 1;
                }
                next_shape = pumpkin_data::block_properties::RailShape::NorthSouth;
            }
            _ => return false,
        }

        let next_pos = BlockPos::new(x, y, z);

        if self
            .is_powered_by_other_rails_at(world, &next_pos, direction, distance, next_shape)
            .await
        {
            return true;
        }

        if check_down {
            let down_pos = BlockPos::new(x, y - 1, z);
            if self
                .is_powered_by_other_rails_at(world, &down_pos, direction, distance, next_shape)
                .await
            {
                return true;
            }
        }

        false
    }

    async fn is_powered_by_other_rails_at(
        &self,
        world: &World,
        pos: &BlockPos,
        direction: bool,
        distance: u8,
        expected_shape: pumpkin_data::block_properties::RailShape,
    ) -> bool {
        let block = world.get_block(pos);
        if *block != Block::POWERED_RAIL {
            return false;
        }

        let state_id = world.get_block_state_id(pos);
        let rail_props = RailProperties::new(state_id, block);
        let rail_shape = rail_props.shape();

        match expected_shape {
            pumpkin_data::block_properties::RailShape::EastWest => {
                if matches!(
                    rail_shape,
                    pumpkin_data::block_properties::RailShape::NorthSouth
                        | pumpkin_data::block_properties::RailShape::AscendingNorth
                        | pumpkin_data::block_properties::RailShape::AscendingSouth
                ) {
                    return false;
                }
            }
            pumpkin_data::block_properties::RailShape::NorthSouth => {
                if matches!(
                    rail_shape,
                    pumpkin_data::block_properties::RailShape::EastWest
                        | pumpkin_data::block_properties::RailShape::AscendingEast
                        | pumpkin_data::block_properties::RailShape::AscendingWest
                ) {
                    return false;
                }
            }
            _ => {}
        }

        if !rail_props.is_powered() {
            return false;
        }

        if block_receives_redstone_power(world, pos).await {
            return true;
        }

        Box::pin(self.is_powered_by_other_rails(world, pos, &rail_props, direction, distance + 1))
            .await
    }

    /// Vanilla `PoweredRailBlock.updateState` — only this rail.
    async fn update_powered_state(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        self.update_powered_state_internal(world, block, pos).await;
    }

    async fn update_powered_state_internal(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
    ) {
        let state_id = world.get_block_state_id(pos);
        let mut rail_props = RailProperties::new(state_id, block);
        let current_powered = rail_props.is_powered();

        // Vanilla: hasNeighborSignal(pos) || findPoweredRailSignal(true) || findPoweredRailSignal(false)
        let direct_power = block_receives_redstone_power(world, pos).await;
        let rail_power = self
            .is_powered_by_other_rails(world, pos, &rail_props, true, 0)
            .await
            || self
                .is_powered_by_other_rails(world, pos, &rail_props, false, 0)
                .await;

        let should_be_powered = direct_power || rail_power;

        if current_powered != should_be_powered {
            rail_props.set_powered(should_be_powered);
            // NOTIFY_ALL → neighbors re-enter on_neighbor_update (vanilla cascade).
            world
                .set_block_state(pos, rail_props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;

            world.update_neighbor(&pos.down(), block).await;

            if rail_props.shape().is_ascending() {
                world.update_neighbor(&pos.up(), block).await;
            }
        }
    }

}
