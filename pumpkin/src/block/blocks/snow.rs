use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, block_properties::SnowLikeProperties, item::Item, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
    UseWithItemArgs, registry::BlockActionResult,
};

#[pumpkin_block("minecraft:snow")]
pub struct LayeredSnowBlock;

impl BlockBehaviour for LayeredSnowBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                return Block::AIR.default_state.id;
            }
            let mut props = SnowLikeProperties::default(args.block);
            props.layers = 1;
            props.to_state_id(&Block::SNOW)
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item = {
                let lock = args.item_stack.lock().await;
                lock.item
            };

            if item == &Item::SNOW {
                let pos = if args.hit.face.is_horizontal() {
                    &args.position.offset(args.hit.face.to_offset())
                } else {
                    args.position
                };
                if !can_place_at(args.world.as_ref(), pos) {
                    return BlockActionResult::Pass;
                }
                let (block, state_id) = args.world.get_block_and_state_id(pos);

                if block != &Block::SNOW {
                    return BlockActionResult::Pass;
                }

                let mut props = SnowLikeProperties::from_state_id(state_id, &Block::SNOW);
                if props.layers >= 8 {
                    args.world
                        .set_block_state(
                            pos,
                            Block::SNOW_BLOCK.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    return BlockActionResult::Success;
                }
                props.layers += 1;

                let state_id = props.to_state_id(&Block::SNOW);
                args.world
                    .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                return BlockActionResult::Success;
            }
            BlockActionResult::Pass
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position) {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            args.state_id
        })
    }
}

fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let below_pos = position.down();
    let (below_block, state) = block_accessor.get_block_and_state(&below_pos);

    if below_block.has_tag(&tag::Block::MINECRAFT_CANNOT_SUPPORT_SNOW_LAYER) {
        return false;
    }
    if below_block.has_tag(&tag::Block::MINECRAFT_SUPPORT_OVERRIDE_SNOW_LAYER) {
        return true;
    }

    // Block.isFaceFullSquare(collisionShape, Direction.UP): the collision shape must fully cover
    // the top face, e.g. leaves are not "side solid" but do support snow layers.
    state.get_block_collision_shapes().any(|shape| {
        shape.max.y >= 1.0
            && shape.min.x <= 0.0
            && shape.max.x >= 1.0
            && shape.min.z <= 0.0
            && shape.max.z >= 1.0
    }) || (below_block == &Block::SNOW
        && SnowLikeProperties::from_state_id(state.id, below_block).layers == 8)
}
