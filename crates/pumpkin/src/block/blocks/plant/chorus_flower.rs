use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::ChorusFlowerLikeProperties,
    tag::{self, Taggable},
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;

use super::chorus_plant;
use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
        RandomTickArgs,
    },
    world::World,
};

pub const DEAD_AGE: u8 = 5;

const HORIZONTAL_DIRECTIONS: [BlockDirection; 4] = [
    BlockDirection::North,
    BlockDirection::South,
    BlockDirection::West,
    BlockDirection::East,
];

#[pumpkin_block("minecraft:chorus_flower")]
pub struct ChorusFlowerBlock;

impl BlockBehaviour for ChorusFlowerBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_survive(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction != BlockDirection::Up && !can_survive(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_survive(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let above = args.position.up();
        let max_y = args.world.dimension.min_y + args.world.dimension.height - 1;
        if args.world.get_block(&above).default_state.is_air() && above.0.y <= max_y {
            let state_id = args.world.get_block_state_id(args.position);
            let state_props = ChorusFlowerLikeProperties::from_state_id(state_id);
            let current_age = state_props.age;
            if current_age < DEAD_AGE {
                let mut grow_upwards = false;
                let mut pillar_on_support_block = false;
                let below_pos = args.position.down();
                let (below_block, _) = args.world.get_block_and_state(&below_pos);

                if below_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_FLOWER) {
                    grow_upwards = true;
                } else if below_block == &Block::CHORUS_PLANT {
                    let mut height = 1;
                    for _ in 0..4 {
                        let test_pos = args.position.offset(Vector3::new(0, -(height + 1), 0));
                        let (test_block, _) = args.world.get_block_and_state(&test_pos);
                        if test_block != &Block::CHORUS_PLANT {
                            if test_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_FLOWER) {
                                pillar_on_support_block = true;
                            }
                            break;
                        }
                        height += 1;
                    }

                    let max_chance = if pillar_on_support_block { 5 } else { 4 };
                    if height < 2 || height <= rand::rng().random_range(0..max_chance) {
                        grow_upwards = true;
                    }
                } else if below_block.default_state.is_air() {
                    grow_upwards = true;
                }

                let above_2 = args.position.offset(Vector3::new(0, 2, 0));
                if grow_upwards
                    && all_neighbors_empty(args.world.as_ref(), &above, None)
                    && args.world.get_block(&above_2).default_state.is_air()
                {
                    let plant_state_id = chorus_plant::get_state_with_connections(
                        args.world.as_ref(),
                        &Block::CHORUS_PLANT,
                        args.position,
                    );
                    args.world.set_block_state(
                        args.position,
                        plant_state_id,
                        BlockFlags::NOTIFY_ALL,
                    );
                    place_grown_flower(args.world, &above, current_age);
                } else if current_age < 4 {
                    let mut num_branch_attempts = rand::rng().random_range(0..4);
                    if pillar_on_support_block {
                        num_branch_attempts += 1;
                    }

                    let mut created_branch = false;

                    for _ in 0..num_branch_attempts {
                        let direction = HORIZONTAL_DIRECTIONS[rand::rng().random_range(0..4)];
                        let target = args.position.offset(direction.to_offset());
                        let target_below = target.down();

                        if args.world.get_block(&target).default_state.is_air()
                            && args.world.get_block(&target_below).default_state.is_air()
                            && all_neighbors_empty(
                                args.world.as_ref(),
                                &target,
                                Some(direction.opposite()),
                            )
                        {
                            place_grown_flower(args.world, &target, current_age + 1);
                            created_branch = true;
                        }
                    }

                    if created_branch {
                        let plant_state_id = chorus_plant::get_state_with_connections(
                            args.world.as_ref(),
                            &Block::CHORUS_PLANT,
                            args.position,
                        );
                        args.world.set_block_state(
                            args.position,
                            plant_state_id,
                            BlockFlags::NOTIFY_ALL,
                        );
                    } else {
                        place_dead_flower(args.world, args.position);
                    }
                } else {
                    place_dead_flower(args.world, args.position);
                }
            }
        }
    }
}

pub fn place_grown_flower(world: &Arc<World>, pos: &BlockPos, age: u8) {
    let mut props = ChorusFlowerLikeProperties::default(&Block::CHORUS_FLOWER);
    props.age = age;
    world.set_block_state(
        pos,
        props.to_state_id(&Block::CHORUS_FLOWER),
        BlockFlags::NOTIFY_ALL,
    );
    world.sync_world_event(WorldEvent::SoundChorusGrow, *pos, 0);
}

pub fn place_dead_flower(world: &Arc<World>, pos: &BlockPos) {
    let mut props = ChorusFlowerLikeProperties::default(&Block::CHORUS_FLOWER);
    props.age = DEAD_AGE;
    world.set_block_state(
        pos,
        props.to_state_id(&Block::CHORUS_FLOWER),
        BlockFlags::NOTIFY_ALL,
    );
    world.sync_world_event(WorldEvent::SoundChorusDeath, *pos, 0);
}

#[must_use]
pub fn all_neighbors_empty(
    world: &dyn BlockAccessor,
    pos: &BlockPos,
    ignore: Option<BlockDirection>,
) -> bool {
    for direction in HORIZONTAL_DIRECTIONS {
        if Some(direction) != ignore {
            let neighbor_pos = pos.offset(direction.to_offset());
            if !world.get_block(&neighbor_pos).default_state.is_air() {
                return false;
            }
        }
    }
    true
}

#[must_use]
pub fn can_survive(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let (block_below, _) = block_accessor.get_block_and_state(&pos.down());

    if block_below == &Block::CHORUS_PLANT
        || block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_FLOWER)
    {
        return true;
    }

    if !block_below.default_state.is_air() {
        return false;
    }

    // Below is air: the flower is the tip of a horizontal branch.
    // Exactly one horizontal neighbor must be a chorus plant stem, and other horizontal neighbors must be air.
    let mut plant_count = 0u32;
    for dir in HORIZONTAL_DIRECTIONS {
        let neighbor = block_accessor.get_block(&pos.offset(dir.to_offset()));
        if neighbor == &Block::CHORUS_PLANT {
            plant_count += 1;
            if plant_count > 1 {
                return false;
            }
        } else if !neighbor.default_state.is_air() {
            return false;
        }
    }

    plant_count == 1
}
