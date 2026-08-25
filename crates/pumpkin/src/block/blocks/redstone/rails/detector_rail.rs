use std::sync::Arc;

use pumpkin_data::block_properties::{HorizontalFacing, RailShape};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::EmitsRedstonePowerArgs;
use crate::block::GetComparatorOutputArgs;
use crate::block::GetRedstonePowerArgs;
use crate::block::OnEntityCollisionArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::entity::EntityBase;
use crate::entity::vehicle::minecart::{MinecartEntity, is_minecart};
use crate::world::World;

use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

// Vanilla `DetectorRailBlock.getSearchBB`: inset 0.2 X/Z.
const DETECTOR_RAIL_DETECTION_BOX: BoundingBox =
    BoundingBox::new_array([0.2, 0.0, 0.2], [0.8, 0.8, 0.8]);

fn find_minecart_at(world: &World, pos: &BlockPos) -> Option<Arc<dyn EntityBase>> {
    let aabb = DETECTOR_RAIL_DETECTION_BOX.at_pos(*pos);
    world
        .get_entities_at_box(&aabb)
        .into_iter()
        .find(|entity| is_minecart(entity.get_entity().entity_type))
}

/// Vanilla `DetectorRailBlock.updatePowerToConnected`: block update on each cell this
/// shape connects to (slope: one up and one over, outside the six-neighbour sweep).
async fn update_power_to_connected(world: &Arc<World>, block: &Block, pos: &BlockPos) {
    let props = RailProperties::new(world.get_block_state_id(pos), block);
    let ascending_towards = match props.shape() {
        RailShape::AscendingEast => Some(HorizontalFacing::East),
        RailShape::AscendingWest => Some(HorizontalFacing::West),
        RailShape::AscendingNorth => Some(HorizontalFacing::North),
        RailShape::AscendingSouth => Some(HorizontalFacing::South),
        _ => None,
    };

    for direction in props.directions() {
        let mut connection = pos.offset(direction.to_offset());
        if ascending_towards == Some(direction) {
            connection = connection.up();
        }
        // Vanilla: shape only, no rail check (`RailState` skips `removeSoftConnections`).
        let connection_block = world.get_block(&connection);
        world.update_neighbor(&connection, connection_block).await;
    }
}

/// Vanilla `DetectorRailBlock.checkPressed`: `POWERED` from a minecart in the detection box.
async fn check_pressed(world: &Arc<World>, block: &Block, pos: &BlockPos) {
    if !rail_placement_is_valid(world, block, pos).await {
        return;
    }

    let state_id = world.get_block_state_id(pos);
    let mut rail_props = RailProperties::new(state_id, block);
    let was_powered = rail_props.is_powered();
    let is_powered = find_minecart_at(world, pos).is_some();

    if is_powered != was_powered {
        rail_props.set_powered(is_powered);
        world
            .set_block_state(pos, rail_props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        update_power_to_connected(world, block, pos).await;
        // Vanilla `updateNeighborsAt` on the rail and the block below (the support carries
        // the signal to a piston beside or beneath it).
        world.update_neighbors(pos, None).await;
        world.update_neighbors(&pos.down(), None).await;
    }

    if is_powered {
        world.schedule_block_tick(block, *pos, 20, TickPriority::Normal);

        // Vanilla pokes the comparator every 20 ticks (`AbstractMinecartContainer.setChanged`
        // is a no-op). Skip when cargo did not change; a `POWERED` flip already swept neighbors.
        let cargo_changed = find_minecart_at(world, pos).is_some_and(|entity| {
            entity
                .cast_any()
                .downcast_ref::<MinecartEntity>()
                .is_some_and(MinecartEntity::take_container_dirty)
        });
        if cargo_changed {
            world.update_neighbour_for_output_signal(pos, block).await;
        }
    }
}

#[pumpkin_block("minecraft:detector_rail")]
pub struct DetectorRailBlock;

impl BlockBehaviour for DetectorRailBlock {
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
            check_pressed(args.world, args.block, args.position).await;
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !rail_placement_is_valid(args.world, args.block, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position)
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !args.moved && RailProperties::new(args.old_state_id, args.block).is_powered() {
                args.world.update_neighbors(args.position, None).await;
                args.world
                    .update_neighbors(&args.position.down(), None)
                    .await;
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !is_minecart(args.entity.get_entity().entity_type) {
                return;
            }
            let state_id = args.world.get_block_state_id(args.position);
            if !RailProperties::new(state_id, args.block).is_powered() {
                check_pressed(args.world, args.block, args.position).await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla `tick`: re-check only while powered (cart left). Arrival is `on_entity_collision`.
            let state_id = args.world.get_block_state_id(args.position);
            if RailProperties::new(state_id, args.block).is_powered() {
                check_pressed(args.world, args.block, args.position).await;
            }
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if RailProperties::new(args.state.id, args.block).is_powered() {
                15
            } else {
                0
            }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.direction == BlockDirection::Up
                && RailProperties::new(args.state.id, args.block).is_powered()
            {
                15
            } else {
                0
            }
        })
    }

    // Vanilla `DetectorRailBlock.getAnalogOutputSignal`: 0 unless a container minecart
    // (chest/hopper) is on the rail. A plain minecart powers redstone, not the comparator.
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            if !RailProperties::new(args.state.id, args.block).is_powered() {
                return Some(0);
            }

            let Some(entity) = find_minecart_at(args.world, args.position) else {
                return Some(0);
            };
            let Some(minecart) = entity.cast_any().downcast_ref::<MinecartEntity>() else {
                return Some(0);
            };

            Some(minecart.container_comparator_output().await.unwrap_or(0))
        })
    }
}
