use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{
        BlockProperties, ComparatorLikeProperties, ComparatorMode, HorizontalFacing,
        get_state_by_state_id,
    },
    entity::EntityType,
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{
    BlockStateId,
    block::entities::{BlockEntity, comparator::ComparatorBlockEntity},
    chunk::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::{BlockIsReplacing, pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
    world::World,
};

use super::abstruct_redstone_gate::{self, RedstoneGateBlock, RedstoneGateBlockProperties};

#[pumpkin_block("minecraft:comparator")]
pub struct ComparatorBlock;

#[async_trait]
impl PumpkinBlock for ComparatorBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        RedstoneGateBlock::on_place(self, player, block).await
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        let state = world.get_block_state(&location).await;
        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        self.on_use(props, world, location, block).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let state = world.get_block_state(&location).await;
        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        self.on_use(props, world, location, block).await;
        BlockActionResult::Consume
    }

    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: BlockDirection,
    ) -> bool {
        true
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        RedstoneGateBlock::can_place_at(self, block_accessor, *block_pos).await
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        let comparator = ComparatorBlockEntity::new(*pos);
        world.add_block_entity(Arc::new(comparator)).await;
        if let Some(state) = get_state_by_state_id(state_id) {
            RedstoneGateBlock::update_target(self, world, *pos, state.id, block).await;
        }
    }

    async fn player_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: u16,
        pos: &BlockPos,
        _face: BlockDirection,
        _player: &Player,
    ) {
        RedstoneGateBlock::player_placed(self, world, block, state_id, pos).await;
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Arc<Player>,
        block_pos: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        world.remove_block_entity(&block_pos).await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state: BlockStateId,
        _pos: &BlockPos,
        direction: BlockDirection,
        neighbor_pos: &BlockPos,
        neighbor_state_id: BlockStateId,
    ) -> BlockStateId {
        if direction == BlockDirection::Down {
            if let Some(neighbor_state) = get_state_by_state_id(neighbor_state_id) {
                if !RedstoneGateBlock::can_place_above(self, world, *neighbor_pos, &neighbor_state)
                    .await
                {
                    return Block::AIR.default_state.id;
                }
            }
        }
        state
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        world: &World,
        block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        RedstoneGateBlock::get_weak_redstone_power(self, block, world, block_pos, state, direction)
            .await
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        RedstoneGateBlock::get_strong_redstone_power(
            self, block, world, block_pos, state, direction,
        )
        .await
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        source_block: &Block,
        _notify: bool,
    ) {
        RedstoneGateBlock::on_neighbor_update(self, world, block, pos, source_block).await;
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, pos: &BlockPos) {
        let state = world.get_block_state(pos).await;
        self.update(world, *pos, &state, block).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        location: BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        RedstoneGateBlock::on_state_replaced(self, world, block, location, old_state_id, moved)
            .await;
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

#[async_trait]
impl RedstoneGateBlock<ComparatorLikeProperties> for ComparatorBlock {
    async fn get_output_level(&self, world: &World, pos: BlockPos) -> u8 {
        if let Some((nbt, raw_blockentity)) = world.get_block_entity(&pos).await {
            if raw_blockentity.resource_location() == ComparatorBlockEntity::ID {
                let comparator = ComparatorBlockEntity::from_nbt(&nbt, pos);
                return comparator.output_signal as u8;
            }
        }
        0
    }

    async fn update_powered(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) {
        if world.is_block_tick_scheduled(&pos, block).await {
            return;
        }
        let i = self.calculate_output_signal(world, pos, state, block).await;
        let j = RedstoneGateBlock::get_output_level(self, world, pos).await;

        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        if i != j || props.powered != self.has_power(world, pos, state, block).await {
            world
                .schedule_block_tick(
                    block,
                    pos,
                    RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                    if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block)
                        .await
                    {
                        TickPriority::High
                    } else {
                        TickPriority::Normal
                    },
                )
                .await;
        }
    }

    async fn has_power(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> bool {
        let i = self.get_power(world, pos, state, block).await;
        if i == 0 {
            return false;
        }
        let j = self
            .get_max_input_level_sides(world, pos, state.id, block, false)
            .await;
        if i > j {
            true
        } else {
            let props = ComparatorLikeProperties::from_state_id(state.id, block);
            i == j && props.mode == ComparatorMode::Compare
        }
    }

    async fn get_power(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        let redstone_level = abstruct_redstone_gate::get_power::<ComparatorLikeProperties>(
            world, pos, state.id, block,
        )
        .await;

        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        let facing = props.facing;
        let source_pos = pos.offset(facing.to_offset());
        let (source_block, source_state) = world.get_block_and_block_state(&source_pos).await;

        if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(&source_block) {
            if let Some(level) = pumpkin_block
                .get_comparator_output(&source_block, world, &source_pos, &source_state)
                .await
            {
                return level;
            }
        }

        if redstone_level < 15 && source_state.is_solid() {
            let source_pos = source_pos.offset(facing.to_offset());
            let (source_block, source_state) = world.get_block_and_block_state(&source_pos).await;

            let itemframe_level = self
                .get_attached_itemframe_level(world, facing, source_pos)
                .await;
            let block_level = if let Some(pumpkin_block) =
                world.block_registry.get_pumpkin_block(&source_block)
            {
                pumpkin_block
                    .get_comparator_output(&source_block, world, &source_pos, &source_state)
                    .await
            } else {
                None
            };
            if let Some(level) = itemframe_level.max(block_level) {
                return level;
            }
        }
        redstone_level
    }

    fn get_update_delay_internal(&self, _state_id: BlockStateId, _block: &Block) -> u16 {
        2
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
        props.mode = match props.mode {
            ComparatorMode::Compare => ComparatorMode::Subtract,
            ComparatorMode::Subtract => ComparatorMode::Compare,
        };
        let state_id = props.to_state_id(block);
        world
            .set_block_state(&block_pos, state_id, BlockFlags::empty())
            .await;
        if let Some(state) = get_state_by_state_id(state_id) {
            self.update(world, block_pos, &state, block).await;
        }
    }

    async fn calculate_output_signal(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        let power = self.get_power(world, pos, state, block).await;
        let sub_power = self
            .get_max_input_level_sides(world, pos, state.id, block, false)
            .await;
        if sub_power >= power {
            return 0;
        }
        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        if props.mode == ComparatorMode::Subtract {
            power - sub_power
        } else {
            power
        }
    }

    async fn get_attached_itemframe_level(
        &self,
        world: &World,
        facing: HorizontalFacing,
        pos: BlockPos,
    ) -> Option<u8> {
        let mut itemframes = world
            .get_entities_at_box(&BoundingBox::from_block(&pos))
            .await
            .into_iter()
            .filter(|entity| {
                entity.get_entity().entity_type == EntityType::ITEM_FRAME
                    && entity.get_entity().get_horizontal_facing() == facing
            });
        if let Some(_itemframe) = itemframes.next() {
            if itemframes.next().is_none() {
                // TODO itemframe.getComparatorPower()
                return Some(1);
            }
        }
        None
    }

    async fn update(&self, world: &Arc<World>, pos: BlockPos, state: &BlockState, block: &Block) {
        let future_level = i32::from(self.calculate_output_signal(world, pos, state, block).await);
        let mut now_level = 0;
        if let Some((nbt, blockentity)) = world.get_block_entity(&pos).await {
            if blockentity.resource_location() == ComparatorBlockEntity::ID {
                let mut comparator = ComparatorBlockEntity::from_nbt(&nbt, pos);
                now_level = comparator.output_signal;
                comparator.output_signal = future_level;
                world.add_block_entity(Arc::new(comparator)).await;
            }
        }
        let mut props = ComparatorLikeProperties::from_state_id(state.id, block);
        if now_level != future_level || props.mode == ComparatorMode::Compare {
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
