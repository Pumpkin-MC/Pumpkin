use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockStateId, block_properties::RailShape, entity::EntityType,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetComparatorOutputArgs, GetRedstonePowerArgs, OnEntityCollisionArgs, OnNeighborUpdateArgs,
        OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs, PlacedArgs,
    },
    entity::EntityBase,
    world::World,
};

use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

#[pumpkin_block("minecraft:detector_rail")]
pub struct DetectorRailBlock;

// Vanilla DetectorRailBlock.getSearchBB: a centered 0.6 x 0.8 x 0.6 volume.
const DETECTOR_RAIL_SEARCH_BOX: BoundingBox =
    BoundingBox::new_array([0.2, 0.0, 0.2], [0.8, 0.8, 0.8]);

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
            self.check_pressed(args.world, args.block, args.position)
                .await;
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = RailProperties::new(args.state.id, args.block);
            if !props.is_powered() && Self::is_minecart(args.entity) {
                self.check_pressed(args.world, args.block, args.position)
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            if RailProperties::new(state.id, args.block).is_powered() {
                self.check_pressed(args.world, args.block, args.position)
                    .await;
            }
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

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.moved {
                return;
            }

            // Vanilla BaseRailBlock.affectNeighborsAfterRemoval.
            let old_props = RailProperties::new(args.old_state_id, args.block);
            if old_props.shape().is_ascending() {
                args.world.update_neighbors(&args.position.up(), None).await;
            }
            args.world.update_neighbors(args.position, None).await;
            args.world
                .update_neighbors(&args.position.down(), None)
                .await;
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

    fn get_comparator_output<'a>(
        &'a self,
        _args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        // A detector rail is always an analog source. Pumpkin does not yet model
        // command-block success counts or container minecart inventories, so use
        // vanilla's zero fallback instead of leaking its 15-strength rail signal.
        Box::pin(async move { Some(0) })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position)
    }
}

impl DetectorRailBlock {
    async fn check_pressed(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        if !rail_placement_is_valid(world, block, pos).await {
            return;
        }

        let state = world.get_block_state(pos);
        let mut props = RailProperties::new(state.id, block);
        let was_pressed = props.is_powered();
        let is_pressed = world
            .get_entities_at_box(&DETECTOR_RAIL_SEARCH_BOX.at_pos(*pos))
            .iter()
            .any(|entity| Self::is_minecart(entity.as_ref()));

        if was_pressed != is_pressed {
            props.set_powered(is_pressed);
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;

            self.update_power_to_connected(world, pos, props.shape())
                .await;
            world.update_neighbors(pos, None).await;
            world.update_neighbors(&pos.down(), None).await;
        }

        if is_pressed {
            world.schedule_block_tick(block, *pos, 20, TickPriority::Normal);
        }
    }

    fn is_minecart(entity: &dyn EntityBase) -> bool {
        let entity_type = entity.get_entity().entity_type;
        entity_type == &EntityType::MINECART
            || entity_type == &EntityType::CHEST_MINECART
            || entity_type == &EntityType::COMMAND_BLOCK_MINECART
            || entity_type == &EntityType::FURNACE_MINECART
            || entity_type == &EntityType::HOPPER_MINECART
            || entity_type == &EntityType::SPAWNER_MINECART
            || entity_type == &EntityType::TNT_MINECART
    }

    /// Vanilla `DetectorRailBlock.updatePowerToConnected` uses the rail state's
    /// explicit endpoints, including the diagonal endpoint of an ascending rail.
    async fn update_power_to_connected(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        shape: RailShape,
    ) {
        for connection in Self::connection_positions(pos, shape) {
            let connection_block = world.get_block(&connection);
            world.update_neighbor(&connection, connection_block).await;
        }
    }

    fn connection_positions(pos: &BlockPos, shape: RailShape) -> [BlockPos; 2] {
        let north = pos.offset(BlockDirection::North.to_offset());
        let south = pos.offset(BlockDirection::South.to_offset());
        let west = pos.offset(BlockDirection::West.to_offset());
        let east = pos.offset(BlockDirection::East.to_offset());

        match shape {
            RailShape::NorthSouth => [north, south],
            RailShape::EastWest => [west, east],
            RailShape::AscendingEast => [west, east.up()],
            RailShape::AscendingWest => [west.up(), east],
            RailShape::AscendingNorth => [north.up(), south],
            RailShape::AscendingSouth => [north, south.up()],
            RailShape::SouthEast => [east, south],
            RailShape::SouthWest => [west, south],
            RailShape::NorthWest => [west, north],
            RailShape::NorthEast => [east, north],
        }
    }
}
