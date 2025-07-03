use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{
        BlockProperties, EnumVariants, HorizontalFacing, Integer1To4, get_state_by_state_id,
    },
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use pumpkin_world::{BlockStateId, chunk::TickPriority};

use crate::{
    block::{
        pumpkin_block::{
            CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
            GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
            OnScheduledTickArgs, OnStateReplacedArgs, PlacedArgs, PlayerPlacedArgs, PumpkinBlock,
            UseWithItemArgs,
        },
        registry::BlockActionResult,
    },
    world::World,
};

use super::abstruct_redstone_gate::{RedstoneGateBlock, RedstoneGateBlockProperties};

type RepeaterProperties = pumpkin_data::block_properties::RepeaterLikeProperties;

#[pumpkin_block("minecraft:repeater")]
pub struct RepeaterBlock;

#[async_trait]
impl PumpkinBlock for RepeaterBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let state_id = RedstoneGateBlock::on_place(self, args.player, args.block).await;

        let mut props = RepeaterProperties::from_state_id(state_id, args.block);
        props.locked = self
            .is_locked(args.world, *args.location, state_id, args.block)
            .await;

        props.to_state_id(args.block)
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        RedstoneGateBlock::on_neighbor_update(self, args).await;
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        if self
            .is_locked(args.world, *args.location, state.id, args.block)
            .await
        {
            return;
        }
        let mut props = RepeaterProperties::from_state_id(state.id, args.block);

        let now_powered = props.powered;
        let should_be_powered = self
            .has_power(args.world, *args.location, state, args.block)
            .await;

        if now_powered && !should_be_powered {
            props.powered = false;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.location,
                props.to_state_id(args.block),
                args.block,
            )
            .await;
        } else if !now_powered {
            props.powered = true;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            if !should_be_powered {
                args.world
                    .schedule_block_tick(
                        args.block,
                        *args.location,
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
                *args.location,
                props.to_state_id(args.block),
                args.block,
            )
            .await;
        }
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let props = RepeaterProperties::from_state_id(state.id, args.block);
        self.on_use(props, args.world, *args.location, args.block)
            .await;
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.location).await;
        let props = RepeaterProperties::from_state_id(state.id, args.block);
        self.on_use(props, args.world, *args.location, args.block)
            .await;
        BlockActionResult::Consume
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_weak_redstone_power(self, args).await
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_strong_redstone_power(self, args).await
    }

    async fn emits_redstone_power(&self, args: EmitsRedstonePowerArgs<'_>) -> bool {
        let repeater_props = RepeaterProperties::from_state_id(args.state.id, args.block);
        repeater_props.facing.to_block_direction() == args.direction
            || repeater_props.facing.to_block_direction() == args.direction.opposite()
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.location).await
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        if let Some(state) = get_state_by_state_id(args.state_id) {
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.location,
                state.id,
                args.block,
            )
            .await;
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down {
            if let Some(neighbor_state) = get_state_by_state_id(args.neighbor_state_id) {
                if !RedstoneGateBlock::can_place_above(
                    self,
                    args.world,
                    *args.neighbor_location,
                    neighbor_state,
                )
                .await
                {
                    return Block::AIR.default_state.id;
                }
            }
        }
        let mut props = RepeaterProperties::from_state_id(args.state_id, args.block);
        if args.direction.to_axis() != props.facing.to_block_direction().to_axis() {
            props.locked = self
                .is_locked(args.world, *args.location, args.state_id, args.block)
                .await;
            return props.to_state_id(args.block);
        }
        args.state_id
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        RedstoneGateBlock::player_placed(self, args).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        RedstoneGateBlock::on_state_replaced(self, args).await;
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

#[async_trait]
impl RedstoneGateBlock<RepeaterProperties> for RepeaterBlock {
    async fn get_output_level(&self, _world: &World, _pos: BlockPos) -> u8 {
        15
    }

    async fn update_powered(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) {
        if self.is_locked(world, pos, state.id, block).await {
            return;
        }
        let props = RepeaterProperties::from_state_id(state.id, block);
        let powered = props.powered;
        let has_power = RedstoneGateBlock::has_power(self, world, pos, state, block).await;
        if powered != has_power && !world.is_block_tick_scheduled(&pos, block).await {
            let priority =
                if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block).await {
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
                    RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                    priority,
                )
                .await;
        }
    }

    fn get_update_delay_internal(&self, state_id: BlockStateId, block: &Block) -> u16 {
        let props = RepeaterProperties::from_state_id(state_id, block);
        (props.delay.to_index() + 1) * 2
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
