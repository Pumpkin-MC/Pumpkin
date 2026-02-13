use std::{
    any::Any,
    array::from_fn,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use pumpkin_data::{
    HorizontalFacingExt,
    block_properties::{BlockProperties, ChestLikeProperties, ChestType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use tokio::sync::Mutex;

use crate::{
    block::viewer::{ViewerCountListener, ViewerCountTracker, ViewerFuture},
    inventory::{Clearable, Inventory, InventoryFuture, split_stack},
    item::ItemStack,
    world::SimpleWorld,
};

use super::BlockEntity;

pub struct TrappedChestBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

impl BlockEntity for TrappedChestBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let chest = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        };

        chest.read_data(nbt, &chest.items);

        chest
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.write_data(nbt, &self.items, true).await;
        })
    }

    fn tick<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.viewers
                .update_viewer_count::<Self>(self, world, &self.position)
                .await;
        })
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ViewerCountListener for TrappedChestBlockEntity {
    fn on_container_open<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound(world, Sound::BlockChestOpen).await;
        })
    }

    fn on_container_close<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound(world, Sound::BlockChestClose).await;
        })
    }

    fn on_viewer_count_update<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        position: &'a BlockPos,
        old: u16,
        new: u16,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            // Trigger block animation
            world
                .add_synced_block_event(*position, Self::LID_ANIMATION_EVENT_TYPE, new as u8)
                .await;

            // Update neighbors for redstone signal when viewer count changes
            if old != new {
                world.clone().update_neighbors(position, None).await;
            }
        })
    }
}

impl TrappedChestBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:trapped_chest";

    /// Returns the number of players currently viewing this chest
    pub fn get_viewer_count(&self) -> u16 {
        self.viewers.get_viewer_count()
    }

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        }
    }

    async fn play_sound(&self, world: &Arc<dyn SimpleWorld>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        let block = world.get_block(&self.position).await;
        let state = world.get_block_state(&self.position).await;
        let properties = ChestLikeProperties::from_state_id(state.id, block);
        let position = match properties.r#type {
            ChestType::Left => return,
            ChestType::Single => Vector3::new(
                self.position.0.x as f64 + 0.5,
                self.position.0.y as f64 + 0.5,
                self.position.0.z as f64 + 0.5,
            ),
            ChestType::Right => {
                let direction = properties.facing.to_block_direction().to_offset();
                Vector3::new(
                    self.position.0.x as f64 + 0.5 + direction.x as f64 * 0.5,
                    self.position.0.y as f64 + 0.5,
                    self.position.0.z as f64 + 0.5 + direction.z as f64 * 0.5,
                )
            }
        };

        world
            .play_sound_fine(
                sound,
                SoundCategory::Blocks,
                &position,
                0.5,
                rng.next_f32() * 0.1 + 0.9,
            )
            .await;
    }
}

impl Inventory for TrappedChestBlockEntity {
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
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.viewers.open_container();
        })
    }

    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.viewers.close_container();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for TrappedChestBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}
