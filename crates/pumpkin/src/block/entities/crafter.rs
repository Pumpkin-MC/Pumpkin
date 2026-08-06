use crate::block::entities::BlockEntity;
use pumpkin_data::BlockDirection;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::inventory::{
    Clearable, Inventory, InventoryFuture, split_stack, sync_write_items_to_nbt,
};
use std::any::Any;
use std::array::from_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use tokio::sync::Mutex;

pub struct CrafterBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub crafting_ticks_remaining: AtomicI32,
    pub triggered: AtomicBool,
    pub disabled_slots: [AtomicBool; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,
}

impl BlockEntity for CrafterBlockEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.write_inventory_nbt(nbt, true).await;
            nbt.put_int(
                "crafting_ticks_remaining",
                self.crafting_ticks_remaining.load(Ordering::Relaxed),
            );
            nbt.put_bool("triggered", self.triggered.load(Ordering::Relaxed));

            let disabled_indices: Vec<i32> = self
                .disabled_slots
                .iter()
                .enumerate()
                .filter(|(_, disabled)| disabled.load(Ordering::Relaxed))
                .map(|(slot, _)| slot as i32)
                .collect();
            if !disabled_indices.is_empty() {
                nbt.put("disabled_slots", NbtTag::IntArray(disabled_indices));
            }
        })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let disabled_slots: [AtomicBool; Self::INVENTORY_SIZE] =
            from_fn(|_| AtomicBool::new(false));
        if let Some(disabled_indices) = nbt.get_int_array("disabled_slots") {
            for &index in disabled_indices {
                if let Ok(slot) = usize::try_from(index)
                    && slot < Self::INVENTORY_SIZE
                {
                    disabled_slots[slot].store(true, Ordering::Relaxed);
                }
            }
        }

        let crafter = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            crafting_ticks_remaining: AtomicI32::new(
                nbt.get_int("crafting_ticks_remaining").unwrap_or(0),
            ),
            triggered: AtomicBool::new(nbt.get_bool("triggered").unwrap_or(false)),
            disabled_slots,
            dirty: AtomicBool::new(false),
        };

        crafter.read_data(nbt, &crafter.items);

        // Vanilla loadAdditional only keeps a loaded disabled flag if the slot is
        // still empty once items are loaded (slotCanBeDisabled gates both directions).
        for slot in 0..Self::INVENTORY_SIZE {
            if crafter.disabled_slots[slot].load(Ordering::Relaxed)
                && let Ok(item) = crafter.items[slot].try_lock()
                && !item.is_empty()
            {
                crafter.disabled_slots[slot].store(false, Ordering::Relaxed);
            }
        }

        crafter
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

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        sync_write_items_to_nbt(&self.items, &mut nbt);
        nbt.put_int(
            "crafting_ticks_remaining",
            self.crafting_ticks_remaining.load(Ordering::Relaxed),
        );
        nbt.put_bool("triggered", self.triggered.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CrafterBlockEntity {
    pub const INVENTORY_SIZE: usize = 9;
    pub const ID: &'static str = "minecraft:crafter";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            crafting_ticks_remaining: AtomicI32::new(0),
            triggered: AtomicBool::new(false),
            disabled_slots: from_fn(|_| AtomicBool::new(false)),
            dirty: AtomicBool::new(false),
        }
    }

    /// Vanilla `CrafterBlockEntity.slotCanBeDisabled`: a slot can only be toggled
    /// while it currently holds no item.
    pub async fn slot_can_be_disabled(&self, slot: usize) -> bool {
        slot < Self::INVENTORY_SIZE && self.items[slot].lock().await.is_empty()
    }

    /// Vanilla `CrafterBlockEntity.setSlotState`.
    pub async fn set_slot_state(&self, slot: usize, enabled: bool) {
        if self.slot_can_be_disabled(slot).await {
            self.disabled_slots[slot].store(!enabled, Ordering::Relaxed);
            self.mark_dirty();
        }
    }

    /// Vanilla `CrafterBlockEntity.isSlotDisabled`.
    #[must_use]
    pub fn is_slot_disabled(&self, slot: usize) -> bool {
        slot < Self::INVENTORY_SIZE && self.disabled_slots[slot].load(Ordering::Relaxed)
    }
}

impl Inventory for CrafterBlockEntity {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.items {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }

            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            self.mark_dirty();
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let res = split_stack(&self.items, slot, amount).await;
            self.mark_dirty();
            res
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            // Vanilla `setItem`: placing an item into a disabled slot re-enables it.
            if self.is_slot_disabled(slot) {
                self.set_slot_state(slot, true).await;
            }
            *self.items[slot].lock().await = stack;
            self.mark_dirty();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn can_insert_through_face<'a>(
        &'a self,
        slot: usize,
        _stack: &'a ItemStack,
        _direction: BlockDirection,
    ) -> InventoryFuture<'a, bool> {
        Box::pin(async move { !self.is_slot_disabled(slot) })
    }
}

impl Clearable for CrafterBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
            self.mark_dirty();
        })
    }
}
