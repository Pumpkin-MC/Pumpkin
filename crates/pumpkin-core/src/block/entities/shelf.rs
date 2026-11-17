use crate::block::entities::BlockEntity;
use crate::world::World;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::{Clearable, Inventory, sync_write_items_to_nbt};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::array::from_fn;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock, Weak};

pub struct ShelfBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub align_items_to_bottom: AtomicBool,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
    world: OnceLock<Weak<World>>,
}

impl BlockEntity for ShelfBlockEntity {
    fn set_world(&self, world: Weak<World>) {
        let _ = self.world.set(world);
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_inventory_nbt(nbt, true);
        if self.align_items_to_bottom.load(Ordering::Relaxed) {
            nbt.put_bool("align_items_to_bottom", true);
        }
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let align_items_to_bottom = nbt.get_bool("align_items_to_bottom").unwrap_or(false);
        let mut shelf = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            align_items_to_bottom: AtomicBool::new(align_items_to_bottom),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            world: OnceLock::new(),
        };

        pumpkin_inventory::sync_read_items_from_nbt(
            nbt,
            shelf
                .items
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );

        shelf
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        }
        if self.align_items_to_bottom.load(Ordering::Relaxed) {
            nbt.put_bool("align_items_to_bottom", true);
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let items = self.items.try_read().ok()?;
        Some(
            super::bedrock::BedrockBlockActorBuilder::new("Shelf", self.position)
                .put_list(
                    "Items",
                    super::bedrock::build_bedrock_slot_items(items.as_slice()),
                )
                .put_bool(
                    "align_items_to_bottom",
                    self.align_items_to_bottom.load(Ordering::Relaxed),
                )
                .finish(),
        )
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        let Some(container) =
            stack.get_data_component::<pumpkin_data::data_component_impl::ContainerImpl>()
        else {
            return;
        };
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        for (slot, item) in &container.items {
            let idx = usize::from(*slot);
            if idx < Self::INVENTORY_SIZE {
                items[idx] = item.clone();
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ShelfBlockEntity {
    pub const INVENTORY_SIZE: usize = 3;
    pub const ID: &'static str = "minecraft:shelf";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            align_items_to_bottom: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            world: OnceLock::new(),
        }
    }

    /// Vanilla `ShelfBlockEntity.swapItemNoUpdate`.
    pub fn swap_item_no_update(&self, slot: usize, held_item_stack: ItemStack) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::replace(&mut items[slot], held_item_stack)
    }

    /// Vanilla `ShelfBlockEntity.setChanged(@Nullable Holder<GameEvent>)`.
    pub fn mark_dirty_with_event(&self, event: Option<GameEvent>) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
        if let Some(world) = self.world.get().and_then(Weak::upgrade) {
            if let Some(event) = event {
                world.emit_game_event(event.name(), self.position.to_centered_f64());
            }
            if let Some(entity) = world.get_block_entity(&self.position) {
                world.update_block_entity(&entity);
            }
        }
    }
}

impl Inventory for ShelfBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot].clone()
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let removed = {
            let mut items = self
                .items
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
        };
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let res = {
            let mut items = self
                .items
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !items[slot].is_empty() && amount > 0 {
                items[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            }
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[slot] = stack;
        self.mark_dirty();
    }

    fn update_stack(&self, slot: usize, update: &mut dyn FnMut(&mut ItemStack) -> bool) -> bool {
        pumpkin_inventory::update_locked_stack(&self.items, slot, update)
    }

    fn mark_dirty(&self) {
        self.mark_dirty_with_event(Some(GameEvent::BlockActivate));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for ShelfBlockEntity {
    fn clear(&self) {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}
