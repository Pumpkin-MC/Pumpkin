use std::sync::Arc;

use pumpkin_data::block_properties::{
    BambooLeaves, BambooLikeProperties, BlockProperties, EnumVariants, Integer0To1,
};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Block::MINECRAFT_BAMBOO_PLANTABLE_ON;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::{self};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::Rng;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs, blocks::plant::PlantBlockBase};
use crate::block::{
    GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, RandomTickArgs,
    UseWithItemArgs,
};
use crate::world::World;

#[pumpkin_block("minecraft:bamboo")]
pub struct BambooBlock;

impl BlockBehaviour for BambooBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let (block_below, state_id_below) = args
                .world
                .get_block_and_state_id(&args.position.down())
                .await;

            if block_below.has_tag(&MINECRAFT_BAMBOO_PLANTABLE_ON) {
                let mut props = BambooLikeProperties::from_state_id(
                    Block::BAMBOO.default_state.id,
                    &Block::BAMBOO,
                );
                if block_below == &Block::BAMBOO_SAPLING {
                    return Block::BAMBOO.default_state.id;
                } else if block_below == &Block::BAMBOO {
                    let props_below =
                        BambooLikeProperties::from_state_id(state_id_below, block_below);
                    if props_below.age.to_index() > 0 {
                        props.age = Integer0To1::L1;
                    }
                } else {
                    let (block_above, state_id_above) =
                        args.world.get_block_and_state_id(&args.position.up()).await;
                    if block_above == &Block::BAMBOO {
                        let props_above =
                            BambooLikeProperties::from_state_id(state_id_above, block_above);
                        props.age = props_above.age;
                    } else {
                        return Block::BAMBOO_SAPLING.default_state.id;
                    }
                }
                return props.to_state_id(&Block::BAMBOO);
            }
            Block::AIR.default_state.id
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let lock = args.item_stack.lock().await;
            if lock.get_item() == &Item::BONE_MEAL {
                bone_meal(Arc::clone(args.world), args.position).await;
                return BlockActionResult::Success;
            }
            BlockActionResult::Pass
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !<Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), args.position)
                .await
            {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            } else if args.world.get_block(&args.position.down()).await == &Block::BAMBOO_SAPLING {
                args.world
                    .set_block_state(
                        &args.position.down(),
                        Block::BAMBOO.default_state.id,
                        BlockFlags::empty(),
                    )
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            let neighbor_block = args.world.get_block(args.neighbor_position).await;
            if args.direction == BlockDirection::Up && neighbor_block == &Block::BAMBOO {
                let neighbor_props =
                    BambooLikeProperties::from_state_id(args.neighbor_state_id, neighbor_block);
                let mut props = BambooLikeProperties::from_state_id(args.state_id, args.block);
                if neighbor_props.age.to_index() > props.age.to_index() {
                    props.age = match props.age {
                        Integer0To1::L0 => Integer0To1::L1,
                        Integer0To1::L1 => Integer0To1::L0,
                    };
                    return props.to_state_id(args.block);
                }
            }
            args.state_id
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if rand::rng().random_range(0..=3) == 0 {
                update_leaves_and_grow(args.world.clone(), args.position).await;
            }
        })
    }
}

async fn update_leaves_and_grow(world: Arc<World>, position: &BlockPos) {
    let above_pos = position.up();
    let below_pos = position.down();
    let two_below_pos = position.down_height(2);

    let (block, state_id) = world.get_block_and_state_id(position).await;
    let state_above = world.get_block_state(&above_pos).await;

    if !state_above.is_air() {
        return;
    }

    let mut props = BambooLikeProperties::from_state_id(state_id, block);
    if props.stage != Integer0To1::L0 {
        return;
    }

    let bamboo_count = count_bamboo_below(world.clone(), position).await;
    if bamboo_count >= 16 {
        return;
    }
    let (block_below, state_id_below) = world.get_block_and_state_id(&below_pos).await;
    let (block_two_below, state_id_two_below) = world.get_block_and_state_id(&two_below_pos).await;

    let mut props_below = BambooLikeProperties::from_state_id(state_id_below, block_below);

    if bamboo_count >= 1 {
        let below_is_bamboo = block_below == &Block::BAMBOO;
        let below_has_leaves = props_below.leaves != BambooLeaves::None;

        props.leaves = if !below_is_bamboo || !below_has_leaves {
            BambooLeaves::Small
        } else {
            BambooLeaves::Large
        };

        if props.leaves == BambooLeaves::Large && block_two_below == &Block::BAMBOO {
            props_below.leaves = BambooLeaves::Small;

            let mut props_two_below =
                BambooLikeProperties::from_state_id(state_id_two_below, block_two_below);
            props_two_below.leaves = BambooLeaves::None;

            world
                .set_block_state(
                    &below_pos,
                    props_below.to_state_id(block_below),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            world
                .set_block_state(
                    &two_below_pos,
                    props_two_below.to_state_id(block_two_below),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    props.age = if props.age != Integer0To1::L1 && block_two_below == &Block::BAMBOO {
        Integer0To1::L0
    } else {
        Integer0To1::L1
    };

    props.stage =
        if (bamboo_count < 11 || rand::rng().random::<f32>() >= 0.25) && bamboo_count != 15 {
            Integer0To1::L0
        } else {
            Integer0To1::L1
        };

    world
        .set_block_state(&above_pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
        .await;
}

async fn count_bamboo_below(world: Arc<World>, pos: &BlockPos) -> usize {
    let mut bamboo_count = 0;
    let mut found_bamboo_below = true;
    let mut current_position = pos.down();
    while found_bamboo_below && bamboo_count < 16 {
        found_bamboo_below = false;
        if world.get_block(&current_position).await == &Block::BAMBOO {
            current_position = current_position.down();
            bamboo_count += 1;
            found_bamboo_below = true;
        }
    }
    bamboo_count
}

async fn count_bamboo_above(world: Arc<World>, pos: &BlockPos) -> usize {
    let mut bamboo_count = 0;
    let mut found_bamboo_below = true;
    let mut current_position = pos.up();
    while found_bamboo_below && bamboo_count < 16 {
        found_bamboo_below = false;
        if world.get_block(&current_position).await == &Block::BAMBOO {
            current_position = current_position.up();
            bamboo_count += 1;
            found_bamboo_below = true;
        }
    }
    bamboo_count
}

async fn bone_meal(world: Arc<World>, position: &BlockPos) {
    let mut bamboo_above = count_bamboo_above(Arc::clone(&world), position).await;
    let bamboo_below = count_bamboo_below(Arc::clone(&world), position).await;
    let mut new_height = bamboo_above + bamboo_below + 1;
    let l = rand::rng().random_range(0..=2) + 1; // what is this?
    for _ in 0..l {
        let next_pos = position.up_height(bamboo_above as i32);
        let next_state = world.get_block_state(&next_pos).await;
        if !next_state.is_air() || new_height >= 16 {
            return;
        }
        let next_props = BambooLikeProperties::from_state_id(next_state.id, &Block::BAMBOO);

        if next_props.stage == Integer0To1::L1 {
            return;
        }
        update_leaves_and_grow(Arc::clone(&world), position).await;
        new_height += 1;
        bamboo_above += 1;
    }
}

impl PlantBlockBase for BambooBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.has_tag(&tag::Block::MINECRAFT_BAMBOO_PLANTABLE_ON)
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }
}
