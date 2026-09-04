use std::sync::{Arc, atomic::Ordering};

use crate::block::entities::comparator::ComparatorBlockEntity;
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, HorizontalFacingExt,
    block_properties::{ComparatorLikeProperties, HorizontalFacing, ModeComparator},
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetComparatorOutputArgs, GetRedstonePowerArgs, GetStateForNeighborUpdateArgs,
        NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
        PlacedArgs, PlayerPlacedArgs, registry::BlockActionResult,
    },
    entity::decoration::item_frame::ItemFrameEntity,
    world::World,
};

use super::abstract_redstone_gate::{self, RedstoneGateBlock, RedstoneGateBlockProperties};

#[pumpkin_block("minecraft:comparator")]
pub struct ComparatorBlock;

impl BlockBehaviour for ComparatorBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        RedstoneGateBlock::on_place(self, args.player, args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if !args.player.may_build() {
            return BlockActionResult::Pass;
        }

        let state = args.world.get_block_state(args.position);
        let mut props = ComparatorLikeProperties::from_state_id(state.id);

        props.mode = match props.mode {
            ModeComparator::Compare => ModeComparator::Subtract,
            ModeComparator::Subtract => ModeComparator::Compare,
        };

        let pitch = match props.mode {
            ModeComparator::Subtract => 0.55,
            ModeComparator::Compare => 0.5,
        };

        args.world.play_sound_raw_expect(
            args.player,
            Sound::BlockComparatorClick as u16,
            SoundCategory::Blocks,
            &args.position.to_centered_f64(),
            0.3,
            pitch,
        );

        let state_id = props.to_state_id(args.block);
        args.world
            .set_block_state(args.position, state_id, BlockFlags::NOTIFY_LISTENERS);

        if args.world.get_block(args.position) == args.block {
            self.update(
                args.world,
                *args.position,
                BlockState::from_id(state_id),
                args.block,
            );
        }

        BlockActionResult::Success
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.position)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let comparator = ComparatorBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(comparator));

        RedstoneGateBlock::update_target(
            self,
            args.world,
            *args.position,
            args.state_id,
            args.block,
        );
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        RedstoneGateBlock::player_placed(self, args);
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.remove_block_entity(args.position);
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
        args.state_id
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_weak_redstone_power(self, args)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        RedstoneGateBlock::get_strong_redstone_power(self, args)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        RedstoneGateBlock::on_neighbor_update(self, args);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(args.position);
        self.update(args.world, *args.position, state, block);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        RedstoneGateBlock::on_state_replaced(self, args);
    }
}

impl RedstoneGateBlockProperties for ComparatorLikeProperties {
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

impl RedstoneGateBlock<ComparatorLikeProperties> for ComparatorBlock {
    fn get_output_level(&self, world: &World, pos: BlockPos) -> u8 {
        if let Some(blockentity) = world.get_block_entity(&pos)
            && let Some(comparator) = blockentity.as_any().downcast_ref::<ComparatorBlockEntity>()
        {
            return comparator.output_signal.load(Ordering::Relaxed);
        }
        0
    }

    fn update_powered(&self, world: &World, pos: BlockPos, state: &BlockState, block: &Block) {
        if world.is_block_tick_scheduled(&pos, block) {
            return;
        }
        let i = self.calculate_output_signal(world, pos, state, block);
        let j = RedstoneGateBlock::get_output_level(self, world, pos);
        let props = ComparatorLikeProperties::from_state_id(state.id);

        if i != j || props.powered != self.has_power(world, pos, state, block) {
            let priority =
                if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block) {
                    TickPriority::High
                } else {
                    TickPriority::Normal
                };

            world.schedule_block_tick(
                block,
                pos,
                RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                priority,
            );
        }
    }

    fn has_power(&self, world: &World, pos: BlockPos, state: &BlockState, block: &Block) -> bool {
        let i = self.get_power(world, pos, state, block);
        if i == 0 {
            return false;
        }
        let j =
            RedstoneGateBlock::get_max_input_level_sides(self, world, pos, state.id, block, false);

        if i > j {
            true
        } else {
            let props = ComparatorLikeProperties::from_state_id(state.id);
            i == j && props.mode == ModeComparator::Compare
        }
    }

    fn get_power(&self, world: &World, pos: BlockPos, state: &BlockState, block: &Block) -> u8 {
        let mut result_signal = abstract_redstone_gate::get_power::<ComparatorLikeProperties>(
            world, pos, state.id, block,
        );

        let props = ComparatorLikeProperties::from_state_id(state.id);
        let facing = props.facing;
        let mut target_pos = pos.offset(facing.to_offset());
        let (target_block, target_state) = world.get_block_and_state(&target_pos);

        if target_state.has_analog_output_signal() {
            result_signal = world
                .block_registry
                .get_pumpkin_block(target_block.id)
                .and_then(|pumpkin_block| {
                    pumpkin_block.get_comparator_output(GetComparatorOutputArgs {
                        world,
                        block: target_block,
                        state: target_state,
                        position: &target_pos,
                    })
                })
                .unwrap_or(0);
        } else if result_signal < 15 && target_state.is_solid_block() {
            target_pos = target_pos.offset(facing.to_offset());
            let (target_block, target_state) = world.get_block_and_state(&target_pos);

            let item_frame_signal =
                Self::get_attached_itemframe_level(world, facing, target_pos).map(i32::from);

            let block_signal = target_state.has_analog_output_signal().then(|| {
                let signal = world
                    .block_registry
                    .get_pumpkin_block(target_block.id)
                    .and_then(|pumpkin_block| {
                        pumpkin_block.get_comparator_output(GetComparatorOutputArgs {
                            world,
                            block: target_block,
                            state: target_state,
                            position: &target_pos,
                        })
                    })
                    .unwrap_or(0);
                i32::from(signal)
            });

            let item_frame_or_block_signal = match (item_frame_signal, block_signal) {
                (Some(frame), Some(block)) => Some(frame.max(block)),
                (Some(frame), None) => Some(frame),
                (None, Some(block)) => Some(block),
                (None, None) => None,
            };

            if let Some(signal) = item_frame_or_block_signal {
                result_signal = signal as u8;
            }
        }

        result_signal
    }

    fn get_update_delay_internal(&self, _state_id: BlockStateId, _block: &Block) -> u8 {
        2 // Vanilla Delay
    }
}

impl ComparatorBlock {
    fn calculate_output_signal(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        let power = self.get_power(world, pos, state, block);
        if power == 0 {
            return 0;
        }

        let sub_power = self.get_max_input_level_sides(world, pos, state.id, block, false);

        if sub_power > power {
            return 0;
        }

        let props = ComparatorLikeProperties::from_state_id(state.id);
        if props.mode == ModeComparator::Subtract {
            power - sub_power
        } else {
            power
        }
    }

    fn get_attached_itemframe_level(
        world: &World,
        facing: HorizontalFacing,
        pos: BlockPos,
    ) -> Option<u8> {
        let direction = facing.to_block_direction();
        let mut level = None;
        for entity in world.get_entities_at_box(&BoundingBox::from_block(&pos)) {
            let Some(itemframe) = entity.cast_any().downcast_ref::<ItemFrameEntity>() else {
                continue;
            };
            if itemframe.get_facing() != direction {
                continue;
            }
            if level.is_some() {
                // Vanilla only reads a frame when exactly one hangs on this block.
                return None;
            }
            level = Some(itemframe.get_analog_output());
        }
        level
    }

    fn update(&self, world: &Arc<World>, pos: BlockPos, state: &BlockState, block: &Block) {
        let future_level = i32::from(self.calculate_output_signal(world, pos, state, block));
        let mut now_level = 0;

        if let Some(blockentity) = world.get_block_entity(&pos)
            && let Some(comparator) = blockentity.as_any().downcast_ref::<ComparatorBlockEntity>()
        {
            now_level = i32::from(comparator.output_signal.load(Ordering::Relaxed));
            comparator
                .output_signal
                .store(future_level as u8, Ordering::Relaxed);
        }

        let mut props = ComparatorLikeProperties::from_state_id(state.id);
        if now_level != future_level || props.mode == ModeComparator::Compare {
            let future_power = self.has_power(world, pos, state, block);
            let now_power = props.powered;

            if now_power && !future_power {
                props.powered = false;
                world.set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);
            } else if !now_power && future_power {
                props.powered = true;
                world.set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);
            }

            RedstoneGateBlock::update_target(self, world, pos, props.to_state_id(block), block);
        }
    }
}
