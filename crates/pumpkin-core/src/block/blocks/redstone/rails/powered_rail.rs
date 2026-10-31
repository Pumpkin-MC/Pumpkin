use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
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

#[pumpkin_block("minecraft:powered_rail")]
pub struct PoweredRailBlock;

impl BlockBehaviour for PoweredRailBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut rail_props = RailProperties::default(args.block);
        let player_facing = args.player.get_entity().get_horizontal_facing();

        rail_props.set_waterlogged(args.replacing.water_source());
        rail_props.set_straight_shape(compute_placed_rail_shape(
            args.world,
            args.position,
            player_facing,
        ));

        rail_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        update_flanking_rails_shape(args.world, args.block, args.state_id, args.position);
        self.update_powered_state(args.world, args.block, args.position);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if RailProperties::new(args.state_id, args.block).is_waterlogged() {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
            return;
        }

        self.update_powered_state(args.world, args.block, args.position);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let rail_props = RailProperties::new(args.old_state_id, args.block);

            if rail_props.shape().is_ascending() {
                args.world
                    .update_neighbors_at(&args.position.up(), args.block, None);
            }

            args.world
                .update_neighbors_at(args.position, args.block, None);
            args.world
                .update_neighbors_at(&args.position.down(), args.block, None);
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position)
    }
}

impl PoweredRailBlock {
    fn is_powered_by_other_rails(
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

        if self.is_powered_at_position(
            world,
            &BlockPos::new(x, y, z),
            direction,
            distance,
            next_shape,
        ) {
            return true;
        }

        check_down
            && self.is_powered_at_position(
                world,
                &BlockPos::new(x, y - 1, z),
                direction,
                distance,
                next_shape,
            )
    }

    fn is_powered_at_position(
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

        if block_receives_redstone_power(world, pos) {
            return true;
        }

        self.is_powered_by_other_rails(world, pos, &rail_props, direction, distance + 1)
    }

    fn update_powered_state(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        let state_id = world.get_block_state_id(pos);
        let mut rail_props = RailProperties::new(state_id, block);
        let current_powered = rail_props.is_powered();

        let should_be_powered = block_receives_redstone_power(world, pos)
            || self.is_powered_by_other_rails(world, pos, &rail_props, true, 0)
            || self.is_powered_by_other_rails(world, pos, &rail_props, false, 0);

        if current_powered != should_be_powered {
            rail_props.set_powered(should_be_powered);
            world.set_block_state(pos, rail_props.to_state_id(block), BlockFlags::NOTIFY_ALL);

            world.update_neighbors_at(&pos.down(), block, None);

            if rail_props.shape().is_ascending() {
                world.update_neighbors_at(&pos.up(), block, None);
            }
        }
    }
}
