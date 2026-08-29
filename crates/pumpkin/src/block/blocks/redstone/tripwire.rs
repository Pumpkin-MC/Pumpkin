use std::sync::Arc;

use pumpkin_data::item::Item;
use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{BlockProperties, HorizontalFacing},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs, PlacedArgs,
    },
    world::World,
};

use super::tripwire_hook::TripwireHookBlock;

type TripwireProperties = pumpkin_data::block_properties::TripwireLikeProperties;
type TripwireHookProperties = pumpkin_data::block_properties::TripwireHookLikeProperties;

#[pumpkin_block("minecraft:tripwire")]
pub struct TripwireBlock;

impl BlockBehaviour for TripwireBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let mut props = TripwireProperties::from_state_id(args.state.id, args.block);
        if props.powered {
            return;
        }
        props.powered = true;

        let state_id = props.to_state_id(args.block);
        args.world
            .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);

        Self::update(args.world, args.position, state_id);

        args.world
            .schedule_block_tick(args.block, *args.position, 10, TickPriority::Normal);
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let [connect_north, connect_east, connect_south, connect_west] = [
            BlockDirection::North,
            BlockDirection::East,
            BlockDirection::South,
            BlockDirection::West,
        ]
        .map(|dir| {
            let current_pos = args.position.offset(dir.to_offset());
            let state_id = args.world.get_block_state_id(&current_pos);
            Self::should_connect_to(state_id, dir)
        });

        let mut props = TripwireProperties::from_state_id(args.block.default_state.id, args.block);

        props.north = connect_north;
        props.south = connect_south;
        props.west = connect_west;
        props.east = connect_east;

        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if Block::from_state_id(args.old_state_id) == Block::from_state_id(args.state_id) {
            return;
        }

        Self::update(args.world, args.position, args.state_id);
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let has_shears = args.player.inventory().held_item().get_item() == &Item::SHEARS;
        if has_shears {
            let mut props = TripwireProperties::from_state_id(args.state.id, args.block);
            props.disarmed = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::empty(),
            );
            // TODO world.emitGameEvent(player, GameEvent.SHEAR, pos);
            // TODO: Deduct 1 durability from held shears (skip in Creative mode).
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.direction
            .to_horizontal_facing()
            .map_or(args.state_id, |facing| {
                let mut props = TripwireProperties::from_state_id(args.state_id, args.block);
                *match facing {
                    HorizontalFacing::North => &mut props.north,
                    HorizontalFacing::South => &mut props.south,
                    HorizontalFacing::West => &mut props.west,
                    HorizontalFacing::East => &mut props.east,
                } = Self::should_connect_to(args.neighbor_state_id, args.direction);
                props.to_state_id(args.block)
            })
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);

        let mut props = TripwireProperties::from_state_id(state_id, args.block);
        if !props.powered {
            return;
        }

        let aabb = BoundingBox::from_block(args.position);
        // TODO entity.canAvoidTraps()
        if args.world.get_entities_at_box(&aabb).is_empty()
            && args.world.get_players_at_box(&aabb).is_empty()
        {
            props.powered = false;
            let state_id = props.to_state_id(args.block);
            args.world
                .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);
            Self::update(args.world, args.position, state_id);
        } else {
            args.world
                .schedule_block_tick(args.block, *args.position, 10, TickPriority::Normal);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved || Block::from_state_id(args.old_state_id) == args.block {
            return;
        }
        let state_id = args.world.get_block_state_id(args.position);
        Self::update(args.world, args.position, state_id);
    }
}

impl TripwireBlock {
    fn update(world: &Arc<World>, pos: &BlockPos, state_id: BlockStateId) {
        for dir in [BlockDirection::South, BlockDirection::West] {
            for i in 1..42 {
                let current_pos = pos.offset_dir(dir.to_offset(), i);
                let (current_block, current_state) = world.get_block_and_state_id(&current_pos);
                if current_block == &Block::TRIPWIRE_HOOK {
                    let current_props =
                        TripwireHookProperties::from_state_id(current_state, &Block::TRIPWIRE_HOOK);
                    if dir
                        .opposite()
                        .to_horizontal_facing()
                        .is_some_and(|f| current_props.facing == f)
                    {
                        TripwireHookBlock::update(
                            world,
                            current_pos,
                            current_state,
                            false,
                            true,
                            i,
                            Some(state_id),
                        );
                    }
                    break;
                }
                if current_block != &Block::TRIPWIRE {
                    break;
                }
            }
        }
    }

    #[must_use]
    pub fn should_connect_to(state_id: BlockStateId, facing: BlockDirection) -> bool {
        let block = Block::from_state_id(state_id);
        if block == &Block::TRIPWIRE_HOOK {
            let props = TripwireHookProperties::from_state_id(state_id, block);
            Some(props.facing) == facing.opposite().to_horizontal_facing()
        } else {
            block == &Block::TRIPWIRE
        }
    }
}
