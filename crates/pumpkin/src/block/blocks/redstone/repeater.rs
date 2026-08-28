use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, HorizontalFacingExt,
    block_properties::{BlockProperties, HorizontalFacing},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
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
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let state_id = RedstoneGateBlock::on_place(self, args.player, args.block);

        let mut props = RepeaterProperties::from_state_id(state_id, args.block);
        props.locked = self.is_locked(args.world, *args.position, state_id, args.block);

        props.to_state_id(args.block)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        RedstoneGateBlock::on_neighbor_update(self, args);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(args.position);
        if self.is_locked(args.world, *args.position, state.id, block) {
            return;
        }
        let mut props = RepeaterProperties::from_state_id(state.id, block);

        let now_powered = props.powered;
        let should_be_powered = self.has_power(args.world, *args.position, state, block);

        if now_powered && !should_be_powered {
            props.powered = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.position,
                props.to_state_id(block),
                block,
            );
        } else if !now_powered {
            props.powered = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            if !should_be_powered {
                args.world.schedule_block_tick(
                    block,
                    *args.position,
                    RedstoneGateBlock::get_update_delay_internal(
                        self,
                        props.to_state_id(block),
                        block,
                    ),
                    TickPriority::VeryHigh,
                );
            }
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.position,
                props.to_state_id(block),
                block,
            );
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);
        let props = RepeaterProperties::from_state_id(state.id, args.block);
        Self::on_use(props, args.world, *args.position, args.block);

        BlockActionResult::SuccessServer
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_weak_redstone_power(self, args)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_strong_redstone_power(self, args)
    }

    fn emits_redstone_power(&self, args: EmitsRedstonePowerArgs<'_>) -> bool {
        let repeater_props = RepeaterProperties::from_state_id(args.state.id, args.block);
        repeater_props.facing.to_block_direction() == args.direction
            || repeater_props.facing.to_block_direction() == args.direction.opposite()
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.position)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        RedstoneGateBlock::update_target(
            self,
            args.world,
            *args.position,
            args.state_id,
            args.block,
        );
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down
            && !RedstoneGateBlock::can_place_above(
                self,
                args.world,
                *args.neighbor_position,
                BlockState::from_id(args.neighbor_state_id),
            )
        {
            return Block::AIR.default_state.id;
        }
        let mut props = RepeaterProperties::from_state_id(args.state_id, args.block);
        if args.direction.to_axis() != props.facing.to_block_direction().to_axis() {
            props.locked = self.is_locked(args.world, *args.position, args.state_id, args.block);
            return props.to_state_id(args.block);
        }
        args.state_id
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        RedstoneGateBlock::player_placed(self, args);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        RedstoneGateBlock::on_state_replaced(self, args);
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
    fn get_output_level(&self, _world: &World, _pos: BlockPos) -> u8 {
        15
    }

    fn update_powered(&self, world: &World, pos: BlockPos, state: &BlockState, block: &Block) {
        if self.is_locked(world, pos, state.id, block) {
            return;
        }
        let props = RepeaterProperties::from_state_id(state.id, block);
        let powered = props.powered;

        // Note: The signature for has_power must be called without self, as it's a trait method.
        let has_power = RedstoneGateBlock::has_power(self, world, pos, state, block);

        if powered != has_power && !world.is_block_tick_scheduled(&pos, block) {
            let priority =
                if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block) {
                    TickPriority::ExtremelyHigh
                } else if powered {
                    TickPriority::VeryHigh
                } else {
                    TickPriority::High
                };

            world.schedule_block_tick(
                block,
                pos,
                RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                priority,
            );
        }
    }

    fn get_update_delay_internal(&self, state_id: BlockStateId, block: &Block) -> u8 {
        let props = RepeaterProperties::from_state_id(state_id, block);
        props.delay * 2
    }
}

impl RepeaterBlock {
    fn on_use(props: RepeaterProperties, world: &Arc<World>, block_pos: BlockPos, block: &Block) {
        let mut props = props;
        props.delay = if props.delay == 4 { 1 } else { props.delay + 1 };
        let state = props.to_state_id(block);
        world.set_block_state(&block_pos, state, BlockFlags::empty());
    }

    fn is_locked(
        &self,
        world: &World,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) -> bool {
        Self::get_max_input_level_sides(self, world, pos, state_id, block, true) > 0
    }
}
