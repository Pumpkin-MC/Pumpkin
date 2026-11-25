use std::sync::Arc;

use pumpkin_data::block_properties::RailShapeStraight;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::EmitsRedstonePowerArgs;
use crate::block::GetComparatorOutputArgs;
use crate::block::GetRedstonePowerArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnEntityCollisionArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::entity::EntityBase;
use crate::world::World;

use super::RailProperties;
use super::StraightRailShapeExt;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};
use crate::entity::vehicle::minecart::{AbstractMinecart, is_minecart};
use pumpkin_data::block_properties::DetectorRailProperties;

#[pumpkin_block("minecraft:detector_rail")]
pub struct DetectorRailBlock;

/// Vanilla `updatePowerToConnected`: only the rails' two connections hear the change.
fn update_power_to_connected(world: &Arc<World>, pos: &BlockPos, shape: RailShapeStraight) {
    let (c1, c2) = match shape {
        RailShapeStraight::NorthSouth => (pos.north(), pos.south()),
        RailShapeStraight::EastWest => (pos.east(), pos.west()),
        RailShapeStraight::AscendingEast => (pos.west(), pos.east().up()),
        RailShapeStraight::AscendingWest => (pos.east(), pos.west().up()),
        RailShapeStraight::AscendingNorth => (pos.south(), pos.north().up()),
        RailShapeStraight::AscendingSouth => (pos.north(), pos.south().up()),
    };
    world.update_neighbor(&c1, world.get_block(&c1));
    world.update_neighbor(&c2, world.get_block(&c2));
}

impl DetectorRailBlock {
    pub fn check_pressed(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
    ) {
        if !can_place_rail_at(world.as_ref(), pos) {
            return;
        }

        // Search bounding box matching vanilla AABB (0.2 to 0.8 on X and Z, 0.0 to 0.8 on Y)
        let search_box = BoundingBox::new_array([0.2, 0.0, 0.2], [0.8, 0.8, 0.8]).at_pos(*pos);

        let has_minecart = !world
            .get_entities_of_type_at_box(&search_box, is_minecart)
            .is_empty();

        let mut props = DetectorRailProperties::from_state_id(state_id);
        let was_pressed = props.powered;

        if has_minecart && !was_pressed {
            props.powered = true;
            let new_state_id = props.to_state_id(block);
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);
            update_power_to_connected(world, pos, props.shape);
            world.update_neighbors_at(pos, block, None);
            world.update_neighbors_at(&pos.down(), block, None);
        } else if !has_minecart && was_pressed {
            props.powered = false;
            let new_state_id = props.to_state_id(block);
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);
            update_power_to_connected(world, pos, props.shape);
            world.update_neighbors_at(pos, block, None);
            world.update_neighbors_at(&pos.down(), block, None);
        }

        if has_minecart {
            world.schedule_block_tick(block, *pos, 20, TickPriority::Normal);
        }

        world.update_neighbour_for_output_signal(pos, block);
    }
}

impl BlockBehaviour for DetectorRailBlock {
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
        self.check_pressed(args.world, args.position, args.block, args.state_id);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if RailProperties::new(args.state_id, args.block).is_waterlogged() {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                pumpkin_data::fluid::Fluid::WATER.flow.tick_delay,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let props = DetectorRailProperties::from_state_id(args.state.id);
        if !props.powered {
            self.check_pressed(args.world, args.position, args.block, args.state.id);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = DetectorRailProperties::from_state_id(state_id);
        if props.powered {
            self.check_pressed(args.world, args.position, args.block, state_id);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let props = DetectorRailProperties::from_state_id(args.old_state_id);
            if props.shape.as_shape().is_ascending() {
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

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = DetectorRailProperties::from_state_id(args.state.id);
        if props.powered { 15 } else { 0 }
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = DetectorRailProperties::from_state_id(args.state.id);
        if props.powered && args.direction == BlockDirection::Up {
            15
        } else {
            0
        }
    }

    /// Vanilla `getAnalogOutputSignal`: a command block cart's success count first, else the
    /// first live container cart's fill.
    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if !DetectorRailProperties::from_state_id(args.state.id).powered {
            return Some(0);
        }
        let search_box =
            BoundingBox::new_array([0.2, 0.0, 0.2], [0.8, 0.8, 0.8]).at_pos(*args.position);
        let found = args
            .world
            .get_entities_of_type_at_box(&search_box, is_minecart);
        let carts = || {
            found
                .iter()
                .filter_map(|cart| cart.cast_any().downcast_ref::<AbstractMinecart>())
        };
        if let Some(command_block) = carts().find_map(AbstractMinecart::command_block) {
            // Vanilla passes the count on unclamped; saturate instead of wrapping.
            let count = command_block.get_success_count().max(0);
            return Some(u8::try_from(count).unwrap_or(u8::MAX));
        }
        Some(
            carts()
                .filter(|cart| cart.vehicle.entity.is_alive())
                .find_map(AbstractMinecart::container_signal)
                .unwrap_or(0),
        )
    }
}
