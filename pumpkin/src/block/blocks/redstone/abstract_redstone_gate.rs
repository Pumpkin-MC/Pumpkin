use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{
        BlockProperties, ComparatorLikeProperties, EnumVariants, HorizontalFacing,
        RedstoneWireLikeProperties, RepeaterLikeProperties,
    },
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::{
        BlockFuture, GetRedstonePowerArgs, OnNeighborUpdateArgs, OnStateReplacedArgs,
        PlayerPlacedArgs,
    },
    entity::player::Player,
    world::World,
};

use super::{get_redstone_power, is_diode};

pub trait RedstoneGateBlockProperties {
    fn is_powered(&self) -> bool;
    fn get_facing(&self) -> HorizontalFacing;
    fn set_facing(&mut self, facing: HorizontalFacing);
}

pub trait RedstoneGateBlock<T: Send + Sync + BlockProperties + RedstoneGateBlockProperties> {
    // 💡 Converted async fn to fn returning BlockFuture
    fn can_place_at<'a>(
        &'a self,
        world: &'a dyn BlockAccessor,
        pos: BlockPos,
    ) -> BlockFuture<'a, bool>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            let under_pos = pos.down();
            let under_state = world.get_block_state(&under_pos).await;
            self.can_place_above(world, under_pos, under_state).await
        })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn can_place_above<'a>(
        &'a self,
        _world: &'a dyn BlockAccessor,
        _pos: BlockPos,
        state: &'a BlockState,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async { state.is_side_solid(BlockDirection::Up) })
    }

    fn get_weak_redstone_power<'a>(&'a self, args: GetRedstonePowerArgs<'a>) -> BlockFuture<'a, u8>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            let props = T::from_state_id(args.state.id, args.block);
            if props.is_powered() && props.get_facing().to_block_direction() == args.direction {
                self.get_output_level(args.world, *args.position).await
            } else {
                0
            }
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8>
    where
        Self: Send + Sync,
    {
        Box::pin(async move { self.get_weak_redstone_power(args).await })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn get_output_level<'a>(&'a self, world: &'a World, pos: BlockPos) -> BlockFuture<'a, u8>;

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            if RedstoneGateBlock::can_place_at(self, args.world.as_ref(), *args.position).await {
                self.update_powered(args.world, *args.position, state, args.block)
                    .await;
                return;
            }
            args.world
                .set_block_state(
                    args.position,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            for dir in BlockDirection::all() {
                args.world
                    .update_neighbor(&args.position.offset(dir.to_offset()), args.source_block)
                    .await;
            }
        })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn update_powered<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, ()>;

    // 💡 Converted async fn to fn returning BlockFuture
    fn has_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, bool>
    where
        Self: Send + Sync,
    {
        Box::pin(async move { self.get_power(world, pos, state, block).await > 0 })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn get_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, u8>
    where
        Self: Send + Sync,
    {
        Box::pin(async move { get_power::<T>(world, pos, state.id, block).await })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn get_max_input_level_sides<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &'a Block,
        only_gate: bool,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let props = T::from_state_id(state_id, block);
            let facing = props.get_facing();

            let power_left =
                get_power_on_side(world, &pos, facing.rotate_clockwise(), only_gate).await;
            let power_right =
                get_power_on_side(world, &pos, facing.rotate_counter_clockwise(), only_gate).await;

            std::cmp::max(power_left, power_right)
        })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn update_target<'a>(
        &'a self,
        world: &'a Arc<World>,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &'a Block,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = T::from_state_id(state_id, block);
            let facing = props.get_facing();
            let front_pos = pos.offset(facing.opposite().to_offset());
            world.update_neighbor(&front_pos, block).await;
            world
                .update_neighbors(&front_pos, Some(facing.to_block_direction()))
                .await;
        })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn on_place<'a>(
        &'a self,
        player: &'a Player,
        block: &'a Block,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async {
            let mut props = T::default(block);
            let dir = player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.set_facing(dir);

            props.to_state_id(block)
        })
    }

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            if RedstoneGateBlock::has_power(
                self,
                args.world,
                *args.position,
                BlockState::from_id(args.state_id),
                args.block,
            )
            .await
            {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()>
    where
        Self: Send + Sync,
    {
        Box::pin(async move {
            if args.moved || Block::from_state_id(args.old_state_id) == args.block {
                return;
            }
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.position,
                args.old_state_id,
                args.block,
            )
            .await;
        })
    }

    // 💡 Converted async fn to fn returning BlockFuture
    fn is_target_not_aligned<'a>(
        &'a self,
        world: &'a dyn BlockAccessor,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let props = T::from_state_id(state.id, block);
            let facing = props.get_facing().opposite();
            let (target_block, target_state) = world
                .get_block_and_state(&pos.offset(facing.to_offset()))
                .await;
            if target_block == &Block::COMPARATOR {
                let props = ComparatorLikeProperties::from_state_id(target_state.id, target_block);
                props.facing != facing
            } else if target_block == &Block::REPEATER {
                let props = RepeaterLikeProperties::from_state_id(target_state.id, target_block);
                props.facing != facing
            } else {
                false
            }
        })
    }

    fn get_update_delay_internal(&self, state_id: BlockStateId, block: &Block) -> u8;
}

pub async fn get_power<T: BlockProperties + RedstoneGateBlockProperties + Send>(
    world: &World,
    pos: BlockPos,
    state_id: BlockStateId,
    block: &Block,
) -> u8 {
    let props = T::from_state_id(state_id, block);
    let facing = props.get_facing();
    let source_pos = pos.offset(facing.to_offset());
    let (source_block, source_state) = world.get_block_and_state(&source_pos).await;
    let source_level = get_redstone_power(
        source_block,
        source_state,
        world,
        &source_pos,
        facing.to_block_direction(),
    )
    .await;
    if source_level >= 15 {
        source_level
    } else {
        source_level.max(if source_block == &Block::REDSTONE_WIRE {
            let props = RedstoneWireLikeProperties::from_state_id(source_state.id, source_block);
            props.power.to_index() as u8
        } else {
            0
        })
    }
}

async fn get_power_on_side(
    world: &World,
    pos: &BlockPos,
    side: HorizontalFacing,
    only_gate: bool,
) -> u8 {
    let side_pos = pos.offset(side.to_block_direction().to_offset());
    let (side_block, side_state) = world.get_block_and_state(&side_pos).await;
    if !only_gate || is_diode(side_block) {
        world
            .block_registry
            .get_weak_redstone_power(
                side_block,
                world,
                &side_pos,
                side_state,
                side.to_block_direction(),
            )
            .await
    } else {
        0
    }
}
