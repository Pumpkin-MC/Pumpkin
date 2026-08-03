use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnScheduledTickArgs;
use crate::block::RandomTickArgs;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::FarmlandLikeProperties;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type FarmlandProperties = FarmlandLikeProperties;

#[pumpkin_block("minecraft:farmland")]
pub struct FarmlandBlock;

impl BlockBehaviour for FarmlandBlock {
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            push_up_entities(args.world, args.position);
            args.world
                .set_block_state(
                    args.position,
                    Block::DIRT.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                return Block::DIRT.default_state.id;
            }
            args.block.default_state.id
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Up && !can_place_at(args.world, args.position) {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            args.state_id
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if is_water_nearby(args.world, args.position)
                || args.world.is_raining_at(&args.position.up()).await
            {
                let mut props = FarmlandProperties::default(args.block);
                props.moisture = 7;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            } else {
                let state_id = args.world.get_block_state_id(args.position);
                let mut props = FarmlandProperties::from_state_id(state_id, args.block);
                if props.moisture == 0 {
                    if !args
                        .world
                        .get_block(&args.position.up())
                        .has_tag(&tag::Block::MINECRAFT_MAINTAINS_FARMLAND)
                    {
                        push_up_entities(args.world, args.position);
                        args.world
                            .set_block_state(
                                args.position,
                                Block::DIRT.default_state.id,
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                    }
                } else {
                    props.moisture -= 1;
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                }
            }
        })
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(&block_pos.up());
    !state.is_solid() || block.has_tag(&tag::Block::C_FENCE_GATES)
}

/// Simplified port of vanilla's `Block.pushEntitiesUp`: teleports any entity
/// overlapping the 1x1x1 column at `block_pos` up by one block, rather than
/// vanilla's exact old-shape/new-shape collision diff.
fn push_up_entities(world: &Arc<World>, block_pos: &BlockPos) {
    let min = Vector3::new(
        f64::from(block_pos.0.x),
        f64::from(block_pos.0.y),
        f64::from(block_pos.0.z),
    );
    let max = Vector3::new(min.x + 1.0, min.y + 1.0, min.z + 1.0);
    let aabb = BoundingBox::new(min, max);
    for entity in world.get_entities_at_box(&aabb) {
        let entity = entity.get_entity();
        let pos = entity.pos.load();
        entity.pos.store(Vector3::new(pos.x, pos.y + 1.0, pos.z));
    }
}

/// Mirrors vanilla `FarmBlock#isNearWater`, which tests the *fluid* state of every
/// position in the box so that waterlogged blocks count as a water source too.
fn is_water_nearby(world: &Arc<World>, block_pos: &BlockPos) -> bool {
    for dx in -4..=4 {
        for dy in 0..=1 {
            for dz in -4..=4 {
                let check_pos = block_pos.offset(Vector3 {
                    x: dx,
                    y: dy,
                    z: dz,
                });
                if world
                    .get_fluid(&check_pos)
                    .has_tag(&tag::Fluid::MINECRAFT_WATER)
                {
                    return true;
                }
            }
        }
    }
    false
}
