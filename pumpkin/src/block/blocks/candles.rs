use crate::block::{
    BlockFuture, GetStateForNeighborUpdateArgs, OnExplosionHitArgs, OnScheduledTickArgs,
};
use crate::entity::player::Player;
use crate::world::World;
use crate::{
    block::{
        BlockIsReplacing,
        registry::BlockActionResult,
        {
            BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, NormalUseArgs, OnPlaceArgs,
            UseWithItemArgs,
        },
    },
    entity::EntityBase,
};
use pumpkin_data::item::Item;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{BlockProperties, CandleLikeProperties, EnumVariants, Integer1To4},
    entity::EntityPose,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::{BlockStateId, world::BlockFlags};
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:candles")]
pub struct CandleBlock;

/// A trait for a block that can be extinguished.
pub trait ExtinguishableBlock {
    type Properties: BlockProperties + Send;

    #[must_use]
    fn lit(props: &Self::Properties) -> bool;

    fn set_lit(props: &mut Self::Properties, to: bool);

    #[must_use]
    fn extinguish(
        _player: Option<&Player>,
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state: &BlockState,
    ) -> impl Future<Output = ()> + Send {
        async {
            let mut props = Self::Properties::from_state_id(state.id, block);
            Self::set_lit(&mut props, false);

            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
            world
                .play_block_sound(Sound::BlockCandleExtinguish, SoundCategory::Blocks, *pos)
                .await;
            world
                .spawn_particle(
                    pos.to_centered_f64(),
                    Vector3::new(0.0, 0.5, 0.0),
                    0.0,
                    1,
                    Particle::Smoke,
                )
                .await;
        }
    }

    #[must_use]
    fn extinguish_on_explosion_hit(
        args: &OnExplosionHitArgs<'_>,
    ) -> impl Future<Output = ()> + Send {
        async {
            if args.explosion.triggers_blocks() {
                let props = Self::Properties::from_state_id(args.state.id, args.block);
                if Self::lit(&props) {
                    Self::extinguish(None, args.world, args.position, args.block, args.state).await;
                }
            }
        }
    }
}

impl BlockBehaviour for CandleBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.player.get_entity().pose.load() != EntityPose::Crouching
                && let BlockIsReplacing::Itself(state_id) = args.replacing
            {
                let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);
                if properties.candles.to_index() < 3 {
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
                }
                return properties.to_state_id(args.block);
            }

            let mut properties = CandleLikeProperties::default(args.block);
            properties.waterlogged = args.replacing.water_source();
            properties.to_state_id(args.block)
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut properties = CandleLikeProperties::from_state_id(state.id, args.block);

            let item_lock = args.item_stack.lock().await;
            let item = item_lock.item;
            drop(item_lock);
            match item.id {
                id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id)
                    && item.id == args.block.id =>
                {
                    if properties.candles.to_index() < 3 {
                        properties.candles =
                            Integer1To4::from_index(properties.candles.to_index() + 1);
                    }

                    args.world
                        .set_block_state(
                            args.position,
                            properties.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::Consume
                }
                _ => {
                    if properties.lit {
                        properties.lit = false;
                    } else {
                        return BlockActionResult::Pass;
                    }

                    args.world
                        .set_block_state(
                            args.position,
                            properties.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::Consume
                }
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);

            if properties.lit {
                properties.lit = false;
            }

            args.world
                .set_block_state(
                    args.position,
                    properties.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            BlockActionResult::Consume
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.block_accessor, args.position).await })
    }

    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let b = args.world.get_block(args.position).await;
            args.player.get_entity().pose.load() != EntityPose::Crouching
                && CandleLikeProperties::from_state_id(args.state_id, args.block).candles
                    != Integer1To4::L4
                && args.block.id == b.id // only the same color can update
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position).await {
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
            if !can_place_at(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }

    fn on_explosion_hit<'a>(
        &'a self,
        args: OnExplosionHitArgs<'a>,
    ) -> BlockFuture<'a, Option<Vec<ItemStack>>> {
        Box::pin(async move {
            Self::extinguish_on_explosion_hit(&args).await;

            self.on_explosion_hit_base(args).await
        })
    }
}

impl ExtinguishableBlock for CandleBlock {
    type Properties = CandleLikeProperties;

    fn lit(props: &Self::Properties) -> bool {
        props.lit
    }

    fn set_lit(props: &mut Self::Properties, to: bool) {
        props.lit = to;
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let (support_block, state) = block_accessor.get_block_and_state(&position.down()).await;
    !support_block.is_waterlogged(state.id) && state.is_center_solid(BlockDirection::Up)
}
