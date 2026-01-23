use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{BlockProperties, EnumVariants, HorizontalFacing, Integer1To4},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use pumpkin_world::{BlockStateId, tick::TickPriority};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
        GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
        OnScheduledTickArgs, OnStateReplacedArgs, PlacedArgs, PlayerPlacedArgs,
        registry::BlockActionResult,
    },
    world::World,
};

use super::abstract_redstone_gate::{RedstoneGateBlock, RedstoneGateBlockProperties};

type RepeaterProperties = pumpkin_data::block_properties::RepeaterLikeProperties;

#[pumpkin_block("minecraft:repeater")]
pub struct RepeaterBlock;

impl BlockBehaviour for RepeaterBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let state_id = RedstoneGateBlock::on_place(self, args.player, args.block).await;

            let mut props = RepeaterProperties::from_state_id(state_id, args.block);
            props.locked = self
                .is_locked(args.world, *args.position, state_id, args.block)
                .await;

            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_neighbor_update(self, args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            if self
                .is_locked(args.world, *args.position, state.id, args.block)
                .await
            {
                return;
            }
            let mut props = RepeaterProperties::from_state_id(state.id, args.block);

            let now_powered = props.powered;
            let should_be_powered = self
                .has_power(args.world, *args.position, state, args.block)
                .await;

            if now_powered && !should_be_powered {
                props.powered = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
                RedstoneGateBlock::update_target(
                    self,
                    args.world,
                    *args.position,
                    props.to_state_id(args.block),
                    args.block,
                )
                .await;
            } else if !now_powered {
                props.powered = true;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
                if !should_be_powered {
                    args.world
                        .schedule_block_tick(
                            args.block,
                            *args.position,
                            RedstoneGateBlock::get_update_delay_internal(
                                self,
                                props.to_state_id(args.block),
                                args.block,
                            ),
                            TickPriority::VeryHigh,
                        )
                        .await;
                }
                RedstoneGateBlock::update_target(
                    self,
                    args.world,
                    *args.position,
                    props.to_state_id(args.block),
                    args.block,
                )
                .await;
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let props = RepeaterProperties::from_state_id(state.id, args.block);
            self.on_use(props, args.world, *args.position, args.block)
                .await;

            BlockActionResult::Success
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { RedstoneGateBlock::get_weak_redstone_power(self, args).await })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { RedstoneGateBlock::get_strong_redstone_power(self, args).await })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let repeater_props = RepeaterProperties::from_state_id(args.state.id, args.block);
            repeater_props.facing.to_block_direction() == args.direction
                || repeater_props.facing.to_block_direction() == args.direction.opposite()
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.position).await
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.position,
                args.state_id,
                args.block,
            )
            .await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down
                && !RedstoneGateBlock::can_place_above(
                    self,
                    args.world,
                    *args.neighbor_position,
                    BlockState::from_id(args.neighbor_state_id),
                )
                .await
            {
                return Block::AIR.default_state.id;
            }
            let mut props = RepeaterProperties::from_state_id(args.state_id, args.block);
            if args.direction.to_axis() != props.facing.to_block_direction().to_axis() {
                props.locked = self
                    .is_locked(args.world, *args.position, args.state_id, args.block)
                    .await;
                return props.to_state_id(args.block);
            }
            args.state_id
        })
    }

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::player_placed(self, args).await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_state_replaced(self, args).await;
        })
    }
}

impl RedstoneGateBlockProperties for RepeaterProperties {
    fn is_powered(&self) -> bool {
        self.powered
    }

    fn get_facing(&self) -> HorizontalFacing {
        self.facing
    }

    fn set_facing(&mut self, facing: HorizontalFacing) {
        self.facing = facing;
    }
}

impl RedstoneGateBlock<RepeaterProperties> for RepeaterBlock {
    fn get_output_level<'a>(&'a self, _world: &'a World, _pos: BlockPos) -> BlockFuture<'a, u8> {
        Box::pin(async { 15 })
    }

    fn update_powered<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Note: is_locked is assumed to remain an async fn or return a future
            if self.is_locked(world, pos, state.id, block).await {
                return;
            }
            let props = RepeaterProperties::from_state_id(state.id, block);
            let powered = props.powered;

            // 💡 FIX 3: Trait method calls now return futures and must be awaited.
            // Note: The signature for has_power must be called without self, as it's a trait method.
            let has_power = RedstoneGateBlock::has_power(self, world, pos, state, block).await;

            if powered != has_power && !world.is_block_tick_scheduled(&pos, block).await {
                // 💡 FIX 4: is_target_not_aligned returns a Future and must be awaited.
                let priority =
                    if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block)
                        .await
                    {
                        TickPriority::ExtremelyHigh
                    } else if powered {
                        TickPriority::VeryHigh
                    } else {
                        TickPriority::High
                    };

                world
                    .schedule_block_tick(
                        block,
                        pos,
                        // 💡 FIX 5: get_update_delay_internal is not async and is called normally.
                        RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                        priority,
                    )
                    .await;
            }
        })
    }

    fn get_update_delay_internal(&self, state_id: BlockStateId, block: &Block) -> u8 {
        let props = RepeaterProperties::from_state_id(state_id, block);
        (props.delay.to_index() as u8 + 1) * 2
    }
}

impl RepeaterBlock {
    async fn on_use(
        &self,
        props: RepeaterProperties,
        world: &Arc<World>,
        block_pos: BlockPos,
        block: &Block,
    ) {
        let mut props = props;
        props.delay = match props.delay {
            Integer1To4::L1 => Integer1To4::L2,
            Integer1To4::L2 => Integer1To4::L3,
            Integer1To4::L3 => Integer1To4::L4,
            Integer1To4::L4 => Integer1To4::L1,
        };
        let state = props.to_state_id(block);
        world
            .set_block_state(&block_pos, state, BlockFlags::empty())
            .await;
    }

    async fn is_locked(
        &self,
        world: &World,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) -> bool {
        Self::get_max_input_level_sides(self, world, pos, state_id, block, true).await > 0
    }
}
