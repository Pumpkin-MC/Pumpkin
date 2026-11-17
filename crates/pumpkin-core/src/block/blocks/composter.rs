use std::{
    any::Any,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    block::{
        BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnScheduledTickArgs,
        PathComputationType, PlacedArgs, UseWithItemArgs, registry::BlockActionResult,
    },
    entity::{Entity, EntityBase, item::ItemEntity},
    world::{World, game_event::GameEventContext},
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    block_properties::ComposterLikeProperties,
    data_component_impl::CompostableImpl,
    entity::EntityType,
    game_event::GameEvent,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
    world::WorldEvent,
};
use pumpkin_inventory::{Clearable, Inventory, screen_handler::InventoryPlayer};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

#[pumpkin_block("minecraft:composter")]
pub struct ComposterBlock;

impl BlockBehaviour for ComposterBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        if ComposterLikeProperties::from_state_id(args.state_id).level == 7 {
            args.world
                .schedule_block_tick(args.block, *args.position, 20, TickPriority::Normal);
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        if props.level == 8 {
            Self::extract_produce(None, state_id, args.world, args.position);
            return BlockActionResult::Success;
        }

        BlockActionResult::Pass
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let level = ComposterLikeProperties::from_state_id(state_id).level;

        let item_stack = &mut *args.item_stack;
        if item_stack.get_data_component::<CompostableImpl>().is_some() && level < 8 {
            if level < 7 {
                let count = item_stack.item_count;
                let new_state_id =
                    Self::insert_item(None, state_id, args.world, item_stack, args.position);
                if args.player.has_infinite_materials() {
                    item_stack.set_count(count);
                }
                args.world.sync_world_event(
                    WorldEvent::ComposterFill,
                    *args.position,
                    i32::from(new_state_id != state_id),
                );
            }
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = ComposterLikeProperties::from_state_id(state_id);
        let level = props.level;
        if level == 7 {
            self.update_level_composter(args.world, args.position, state_id, args.block, level + 1);
            args.world.play_sound_fine(
                Sound::BlockComposterReady,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
                1.0,
                1.0,
            );
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = ComposterLikeProperties::from_state_id(args.state.id);
        Some(props.level)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl ComposterBlock {
    pub fn insert_item(
        source: Option<&dyn EntityBase>,
        state_id: BlockStateId,
        world: &Arc<World>,
        stack: &mut ItemStack,
        pos: &BlockPos,
    ) -> BlockStateId {
        let level = ComposterLikeProperties::from_state_id(state_id).level;
        let Some(chance) = stack
            .get_data_component::<CompostableImpl>()
            .map(|compostable| compostable.chance)
        else {
            return state_id;
        };
        if level >= 7 {
            return state_id;
        }

        let increased = level == 0 || rand::rng().random_bool(f64::from(chance));
        stack.decrement(1);
        if !increased {
            return state_id;
        }

        let block = Block::from_state_id(state_id);
        Self::set_level_with_source(source, world, pos, state_id, block, level + 1)
    }

    fn set_level_with_source(
        source: Option<&dyn EntityBase>,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
        level: u8,
    ) -> BlockStateId {
        let mut props = ComposterLikeProperties::from_state_id(state_id);
        props.level = level;
        let new_state_id = props.to_state_id(block);
        world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
        world.emit_game_event_with_context(
            GameEvent::BlockChange.name(),
            location.to_centered_f64(),
            &GameEventContext {
                source_entity: source,
            },
        );
        if level == 7 {
            world.schedule_block_tick(block, *location, 20, TickPriority::Normal);
        }
        new_state_id
    }

    pub fn extract_produce(
        source: Option<&dyn EntityBase>,
        state_id: BlockStateId,
        world: &Arc<World>,
        location: &BlockPos,
    ) -> BlockStateId {
        let block = Block::from_state_id(state_id);
        let item_position = {
            let mut rng = rand::rng();
            location.to_centered_f64().add_raw(
                rng.random_range(-0.35..=0.35),
                rng.random_range(-0.35..=0.35) + 0.51,
                rng.random_range(-0.35..=0.35),
            )
        };

        let item_entity = ItemEntity::new(
            Entity::new(world.clone(), item_position, &EntityType::ITEM),
            ItemStack::new(1, &Item::BONE_MEAL),
        );

        world.spawn_entity(Arc::new(item_entity));
        let empty_state_id =
            Self::set_level_with_source(source, world, location, state_id, block, 0);
        world.play_sound_fine(
            Sound::BlockComposterEmpty,
            SoundCategory::Blocks,
            &location.to_centered_f64(),
            1.0,
            1.0,
        );
        empty_state_id
    }

    pub fn update_level_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
        level: u8,
    ) {
        Self::set_level_with_source(None, world, location, state_id, block, level);
    }

    #[must_use]
    pub fn get_container(
        state_id: BlockStateId,
        world: &Arc<World>,
        pos: &BlockPos,
    ) -> Arc<dyn Inventory> {
        let level = ComposterLikeProperties::from_state_id(state_id).level;
        if level == 8 {
            Arc::new(ComposterOutputContainer {
                state_id,
                world: world.clone(),
                pos: *pos,
                stack: Mutex::new(ItemStack::new(1, &Item::BONE_MEAL)),
                changed: AtomicBool::new(false),
            })
        } else if level < 7 {
            Arc::new(ComposterInputContainer {
                state_id,
                world: world.clone(),
                pos: *pos,
                stack: Mutex::new(ItemStack::EMPTY.clone()),
                changed: AtomicBool::new(false),
            })
        } else {
            Arc::new(ComposterEmptyContainer)
        }
    }
}

struct ComposterEmptyContainer;

impl Clearable for ComposterEmptyContainer {
    fn clear(&self) {}
}

impl Inventory for ComposterEmptyContainer {
    fn size(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn get_stack(&self, _slot: usize) -> ItemStack {
        ItemStack::EMPTY.clone()
    }

    fn remove_stack(&self, _slot: usize) -> ItemStack {
        ItemStack::EMPTY.clone()
    }

    fn remove_stack_specific(&self, _slot: usize, _amount: u8) -> ItemStack {
        ItemStack::EMPTY.clone()
    }

    fn set_stack(&self, _slot: usize, _stack: ItemStack) {}

    fn is_sided(&self) -> bool {
        true
    }

    fn get_slots_for_face(&self, _direction: BlockDirection) -> &'static [usize] {
        &[]
    }

    fn can_place_item_through_face(
        &self,
        _slot: usize,
        _stack: &ItemStack,
        _direction: Option<BlockDirection>,
    ) -> bool {
        false
    }

    fn can_take_item_through_face(
        &self,
        _slot: usize,
        _stack: &ItemStack,
        _direction: BlockDirection,
    ) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct ComposterInputContainer {
    state_id: BlockStateId,
    world: Arc<World>,
    pos: BlockPos,
    stack: Mutex<ItemStack>,
    changed: AtomicBool,
}

impl Clearable for ComposterInputContainer {
    fn clear(&self) {
        *self
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
    }
}

impl Inventory for ComposterInputContainer {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot == 0 {
            self.stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot != 0 {
            return ItemStack::EMPTY.clone();
        }
        let mut guard = self
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::replace(&mut *guard, ItemStack::EMPTY.clone())
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot != 0 || amount == 0 {
            return ItemStack::EMPTY.clone();
        }
        let mut guard = self
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.is_empty() {
            ItemStack::EMPTY.clone()
        } else {
            guard.split(amount)
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot == 0 {
            *self
                .stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
        }
    }

    fn get_max_count_per_stack(&self) -> u8 {
        1
    }

    fn is_sided(&self) -> bool {
        true
    }

    fn get_slots_for_face(&self, direction: BlockDirection) -> &'static [usize] {
        if direction == BlockDirection::Up {
            &[0]
        } else {
            &[]
        }
    }

    fn can_place_item_through_face(
        &self,
        _slot: usize,
        stack: &ItemStack,
        direction: Option<BlockDirection>,
    ) -> bool {
        !self.changed.load(Ordering::Relaxed)
            && direction == Some(BlockDirection::Up)
            && stack.get_data_component::<CompostableImpl>().is_some()
    }

    fn can_take_item_through_face(
        &self,
        _slot: usize,
        _stack: &ItemStack,
        _direction: BlockDirection,
    ) -> bool {
        false
    }

    fn mark_dirty(&self) {
        let mut contents = {
            let mut guard = self
                .stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if guard.is_empty() {
                return;
            }
            std::mem::replace(&mut *guard, ItemStack::EMPTY.clone())
        };
        if contents.get_data_component::<CompostableImpl>().is_some() {
            self.changed.store(true, Ordering::Relaxed);
            let new_state_id = ComposterBlock::insert_item(
                None,
                self.state_id,
                &self.world,
                &mut contents,
                &self.pos,
            );
            self.world.sync_world_event(
                WorldEvent::ComposterFill,
                self.pos,
                i32::from(new_state_id != self.state_id),
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct ComposterOutputContainer {
    state_id: BlockStateId,
    world: Arc<World>,
    pos: BlockPos,
    stack: Mutex<ItemStack>,
    changed: AtomicBool,
}

impl Clearable for ComposterOutputContainer {
    fn clear(&self) {
        *self
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
    }
}

impl Inventory for ComposterOutputContainer {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot == 0 {
            self.stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot != 0 {
            return ItemStack::EMPTY.clone();
        }
        let removed = {
            let mut guard = self
                .stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::replace(&mut *guard, ItemStack::EMPTY.clone())
        };
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot != 0 || amount == 0 {
            return ItemStack::EMPTY.clone();
        }
        let res = {
            let mut guard = self
                .stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if guard.is_empty() {
                return ItemStack::EMPTY.clone();
            }
            guard.split(amount)
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot == 0 {
            *self
                .stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
            self.mark_dirty();
        }
    }

    fn get_max_count_per_stack(&self) -> u8 {
        1
    }

    fn is_sided(&self) -> bool {
        true
    }

    fn get_slots_for_face(&self, direction: BlockDirection) -> &'static [usize] {
        if direction == BlockDirection::Down {
            &[0]
        } else {
            &[]
        }
    }

    fn can_place_item_through_face(
        &self,
        _slot: usize,
        _stack: &ItemStack,
        _direction: Option<BlockDirection>,
    ) -> bool {
        false
    }

    fn can_take_item_through_face(
        &self,
        _slot: usize,
        stack: &ItemStack,
        direction: BlockDirection,
    ) -> bool {
        !self.changed.load(Ordering::Relaxed)
            && direction == BlockDirection::Down
            && stack.item.id == Item::BONE_MEAL.id
    }

    fn mark_dirty(&self) {
        if !self.changed.swap(true, Ordering::Relaxed) {
            let block = Block::from_state_id(self.state_id);
            ComposterBlock::set_level_with_source(
                None,
                &self.world,
                &self.pos,
                self.state_id,
                block,
                0,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
