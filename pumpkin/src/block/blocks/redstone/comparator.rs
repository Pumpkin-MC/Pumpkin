use std::sync::{Arc, atomic::Ordering};

use crate::block::entities::comparator::ComparatorBlockEntity;
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::{
        BlockProperties, ComparatorLikeProperties, HorizontalFacing, ModeComparator,
    },
    entity::EntityType,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetComparatorOutputArgs, GetRedstonePowerArgs, GetStateForNeighborUpdateArgs,
        NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
        PlacedArgs, PlayerPlacedArgs, registry::BlockActionResult,
    },
    world::World,
};

use super::abstract_redstone_gate::{self, RedstoneGateBlock, RedstoneGateBlockProperties};

#[pumpkin_block("minecraft:comparator")]
pub struct ComparatorBlock;

impl BlockBehaviour for ComparatorBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { RedstoneGateBlock::on_place(self, args.player, args.block).await })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let props = ComparatorLikeProperties::from_state_id(state.id, args.block);
            self.on_use(props, args.world, *args.position, args.block)
                .await;

            BlockActionResult::Success
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.position)
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let comparator = ComparatorBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(comparator));

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

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::player_placed(self, args).await;
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.remove_block_entity(args.position);
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
            {
                return Block::AIR.default_state.id;
            }
            args.state_id
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

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_neighbor_update(self, args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            self.update(args.world, *args.position, state, args.block)
                .await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_state_replaced(self, args).await;
        })
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
    fn get_output_level<'a>(&'a self, world: &'a World, pos: BlockPos) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if let Some(blockentity) = world.get_block_entity(&pos)
                && let Some(comparator) =
                    blockentity.as_any().downcast_ref::<ComparatorBlockEntity>()
            {
                return comparator.output_signal.load(Ordering::Relaxed);
            }
            0
        })
    }

    fn update_powered<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if world.is_block_tick_scheduled(&pos, block) {
                return;
            }
            let i = self.calculate_output_signal(world, pos, state, block).await;
            let j = RedstoneGateBlock::get_output_level(self, world, pos).await;
            let props = ComparatorLikeProperties::from_state_id(state.id, block);

            if i != j
                || props.powered
                    != RedstoneGateBlock::has_power(self, world, pos, state, block).await
            {
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
        })
    }

    fn has_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let i = RedstoneGateBlock::get_power(self, world, pos, state, block).await;
            if i == 0 {
                return false;
            }
            let j = RedstoneGateBlock::get_max_input_level_sides(
                self, world, pos, state.id, block, false,
            )
            .await;

            if i > j {
                true
            } else {
                let props = ComparatorLikeProperties::from_state_id(state.id, block);
                i == j && props.mode == ModeComparator::Compare
            }
        })
    }

    fn get_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let redstone_level = abstract_redstone_gate::get_power::<ComparatorLikeProperties>(
                world, pos, state.id, block,
            )
            .await;

            let props = ComparatorLikeProperties::from_state_id(state.id, block);
            let facing = props.facing;
            let source_pos = pos.offset(facing.to_offset());
            let (source_block, source_state) = world.get_block_and_state(&source_pos);

            if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(source_block.id)
                && let Some(level) = pumpkin_block
                    .get_comparator_output(GetComparatorOutputArgs {
                        world,
                        block: source_block,
                        state: source_state,
                        position: &source_pos,
                    })
                    .await
            {
                return level;
            }

            if redstone_level < 15 && source_state.is_solid_block() {
                let deeper_source_pos = source_pos.offset(facing.to_offset());
                let (deeper_block, deeper_state) = world.get_block_and_state(&deeper_source_pos);

                let itemframe_level =
                    Self::get_attached_itemframe_level(world, facing, deeper_source_pos);

                // This is the correct way to handle the async call within the Option
                let block_level = if let Some(pumpkin_block) =
                    world.block_registry.get_pumpkin_block(deeper_block.id)
                {
                    pumpkin_block
                        .get_comparator_output(GetComparatorOutputArgs {
                            world,
                            block: deeper_block,
                            state: deeper_state,
                            position: &deeper_source_pos,
                        })
                        .await
                } else {
                    None
                };

                if let Some(level) = itemframe_level.max(block_level) {
                    return level;
                }
            }
            redstone_level
        })
    }

    fn get_update_delay_internal(&self, _state_id: BlockStateId, _block: &Block) -> u8 {
        2 // Vanilla Delay
    }
}

impl ComparatorBlock {
    async fn on_use(
        &self,
        mut props: ComparatorLikeProperties,
        world: &Arc<World>,
        block_pos: BlockPos,
        block: &Block,
    ) {
        // Vanilla Parity TODO:
        // playSound(player, pos, SoundEvents.COMPARATOR_CLICK, SoundSource.BLOCKS, 0.3F, pitch);
        // Pitch is 0.55F if SUBTRACT, 0.5F if COMPARE.

        props.mode = match props.mode {
            ModeComparator::Compare => ModeComparator::Subtract,
            ModeComparator::Subtract => ModeComparator::Compare,
        };

        let state_id = props.to_state_id(block);
        world
            .set_block_state(&block_pos, state_id, BlockFlags::empty())
            .await;

        self.update(world, block_pos, BlockState::from_id(state_id), block)
            .await;
    }

    async fn calculate_output_signal(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        let power = self.get_power(world, pos, state, block).await;
        if power == 0 {
            return 0;
        }

        let sub_power = self
            .get_max_input_level_sides(world, pos, state.id, block, false)
            .await;

        if sub_power > power {
            return 0;
        }

        let props = ComparatorLikeProperties::from_state_id(state.id, block);
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
        let mut itemframes = world
            .get_entities_at_box(&BoundingBox::from_block(&pos))
            .into_iter()
            .filter(|entity| {
                entity.get_entity().entity_type == &EntityType::ITEM_FRAME
                    && entity.get_entity().get_horizontal_facing() == facing
            });

        if itemframes.next().is_some() && itemframes.next().is_none() {
            // Vanilla Parity TODO:
            // Return itemframe.getAnalogOutput() which calculates based on item rotation
            return Some(1);
        }
        None
    }

    async fn update(&self, world: &Arc<World>, pos: BlockPos, state: &BlockState, block: &Block) {
        let future_level = i32::from(self.calculate_output_signal(world, pos, state, block).await);
        let mut now_level = 0;

        if let Some(blockentity) = world.get_block_entity(&pos)
            && let Some(comparator) = blockentity.as_any().downcast_ref::<ComparatorBlockEntity>()
        {
            now_level = i32::from(comparator.output_signal.load(Ordering::Relaxed));
            comparator
                .output_signal
                .store(future_level as u8, Ordering::Relaxed);
        }

        let mut props = ComparatorLikeProperties::from_state_id(state.id, block);
        if now_level != future_level || props.mode == ModeComparator::Compare {
            let future_power = self.has_power(world, pos, state, block).await;
            let now_power = props.powered;

            if now_power && !future_power {
                props.powered = false;
                world
                    .set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                    .await;
            } else if !now_power && future_power {
                props.powered = true;
                world
                    .set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                    .await;
            }

            RedstoneGateBlock::update_target(self, world, pos, props.to_state_id(block), block)
                .await;
        }
    }
}
