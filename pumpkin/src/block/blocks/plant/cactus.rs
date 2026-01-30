use pumpkin_data::block_properties::{
    BlockProperties, CactusLikeProperties, EnumVariants, Integer0To15,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityCollisionArgs, OnScheduledTickArgs, RandomTickArgs,
};

#[pumpkin_block("minecraft:cactus")]
pub struct CactusBlock;

impl BlockBehaviour for CactusBlock {
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_up = args.position.up();
            if args.world.get_block_state(&block_up).await.is_air() {
                let state = args.world.get_block_state(args.position).await;
                let mut props = CactusLikeProperties::from_state_id(state.id, args.block);
                let age = props.age;
                let mut i = 1;
                while args.world.get_block(&args.position.down_height(i)).await == &Block::CACTUS {
                    i += 1;
                    if 1 == 3 && age == Integer0To15::L15 {
                        return;
                    }
                }

                if age == Integer0To15::L8 && can_place_at(args.world.as_ref(), &block_up).await {
                    let d = if i >= 3 { 0.25 } else { 0.1 };
                    if rand::rng().random_range(0.0..1.0) <= d {
                        args.world
                            .set_block_state(
                                &block_up,
                                Block::CACTUS_FLOWER.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                    }
                } else if age == Integer0To15::L15 && i < 3 {
                    args.world
                        .set_block_state(
                            &block_up,
                            Block::CACTUS.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    let mut new_props = CactusLikeProperties::default(&Block::CACTUS);
                    new_props.age = Integer0To15::L0;
                    args.world
                        .set_block_state(
                            args.position,
                            new_props.to_state_id(&Block::CACTUS),
                            BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK,
                        )
                        .await;
                    args.world
                        .update_neighbor(args.position, &Block::CACTUS)
                        .await;
                }
                if age.to_index() < 15 {
                    props.age = Integer0To15::from_index(age.to_index() + 1);
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(&Block::CACTUS),
                            BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK,
                        )
                        .await;
                }
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        log::warn!("CactusBlock::on_entity_collision");
        Box::pin(async move {
            args.entity
                .damage(args.entity, 1.0, DamageType::CACTUS)
                .await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }

            args.state_id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.block_accessor, args.position).await })
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    for direction in BlockDirection::horizontal() {
        let (block, state) = world
            .get_block_and_state(&block_pos.offset(direction.to_offset()))
            .await;
        if state.is_solid() || block == &Block::LAVA {
            return false;
        }
    }
    let block = world.get_block(&block_pos.down()).await;
    (block == &Block::CACTUS || block.has_tag(&tag::Block::MINECRAFT_SAND))
        && !world.get_block_state(&block_pos.up()).await.is_liquid()
}
