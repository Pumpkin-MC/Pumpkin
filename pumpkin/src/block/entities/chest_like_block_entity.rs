/// Implements the `BlockEntity` trait for chest-like block entities.
/// Parameters:
/// - $`struct_name`: The type of the chest struct (e.g., `ChestBlockEntity`)
/// - $`resource_id`: The resource location string (e.g., "minecraft:chest")
#[macro_export]
macro_rules! impl_block_entity_for_chest {
    ($struct_name:ty) => {
        impl $crate::block::entities::BlockEntity for $struct_name {
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
                use pumpkin_world::inventory::Inventory;

                let chest = Self {
                    position,
                    items: std::array::from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
                    dirty: std::sync::atomic::AtomicBool::new(false),
                    viewers: $crate::block::viewer::ViewerCountTracker::new(),
                };

                chest.read_data(nbt, &chest.items);

                chest
            }

            fn write_nbt<'a>(
                &'a self,
                nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
                use pumpkin_world::inventory::Inventory;

                // Write inventory data to NBT
                self.write_inventory_nbt(nbt, true)
            }

            fn tick<'a>(
                &'a self,
                world: &'a Arc<$crate::world::World>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
                Box::pin(async move {
                    $crate::block::viewer::ViewerCountTrackerExt::update_viewer_count::<$struct_name>(
                        &self.viewers,
                        self,
                        world,
                        &self.position,
                    )
                    .await;
                })
            }

            fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn pumpkin_world::inventory::Inventory>> {
                Some(self)
            }

            fn is_dirty(&self) -> bool {
                self.dirty.load(std::sync::atomic::Ordering::Relaxed)
            }

            fn clear_dirty(&self) {
                self.dirty
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// Implements the Inventory trait for chest-like block entities.
#[macro_export]
macro_rules! impl_inventory_for_chest {
    ($struct_name:ty) => {
        impl pumpkin_world::inventory::Inventory for $struct_name {
            fn size(&self) -> usize {
                self.items.len()
            }

            fn is_empty(&self) -> pumpkin_world::inventory::InventoryFuture<'_, bool> {
                Box::pin(async move {
                    for slot in &self.items {
                        if !slot.lock().await.is_empty() {
                            return false;
                        }
                    }

                    true
                })
            }

            fn get_stack(
                &self,
                slot: usize,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
                Box::pin(async move { self.items[slot].clone() })
            }

            fn remove_stack(
                &self,
                slot: usize,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let mut removed = ItemStack::EMPTY.clone();
                    let mut guard = self.items[slot].lock().await;
                    std::mem::swap(&mut removed, &mut *guard);
                    self.mark_dirty();
                    removed
                })
            }

            fn remove_stack_specific(
                &self,
                slot: usize,
                amount: u8,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let res =
                        pumpkin_world::inventory::split_stack(&self.items, slot, amount).await;
                    self.mark_dirty();
                    res
                })
            }

            fn set_stack(
                &self,
                slot: usize,
                stack: ItemStack,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, ()> {
                Box::pin(async move {
                    *self.items[slot].lock().await = stack;
                    self.mark_dirty();
                })
            }

            fn on_open(&self) -> pumpkin_world::inventory::InventoryFuture<'_, ()> {
                Box::pin(async move {
                    self.viewers.open_container();
                })
            }

            fn on_close(&self) -> pumpkin_world::inventory::InventoryFuture<'_, ()> {
                Box::pin(async move {
                    self.viewers.close_container();
                })
            }

            fn mark_dirty(&self) {
                self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// Implements the Clearable trait for chest-like block entities.
#[macro_export]
macro_rules! impl_clearable_for_chest {
    ($struct_name:ty) => {
        impl pumpkin_world::inventory::Clearable for $struct_name {
            fn clear(
                &self,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
                Box::pin(async move {
                    for slot in &self.items {
                        *slot.lock().await = ItemStack::EMPTY.clone();
                    }
                    <$struct_name as pumpkin_world::inventory::Inventory>::mark_dirty(self);
                })
            }
        }
    };
}

/// Implements the `ViewerCountListener` trait for chest-like block entities.
///
/// The behavior is controlled by the `EMITS_REDSTONE` constant on the struct.
/// When `EMITS_REDSTONE` is true, updates neighbors for redstone signals when viewer count changes.
#[macro_export]
macro_rules! impl_viewer_count_listener_for_chest {
    ($struct_name:ty) => {
        impl $crate::block::viewer::ViewerCountListener for $struct_name {
            fn on_container_open<'a>(
                &'a self,
                world: &'a Arc<$crate::world::World>,
                _position: &'a pumpkin_util::math::position::BlockPos,
            ) -> $crate::block::viewer::ViewerFuture<'a, ()> {
                Box::pin(async move {
                    self.play_sound(world, pumpkin_data::sound::Sound::BlockChestOpen)
                        .await;
                })
            }

            fn on_container_close<'a>(
                &'a self,
                world: &'a Arc<$crate::world::World>,
                _position: &'a pumpkin_util::math::position::BlockPos,
            ) -> $crate::block::viewer::ViewerFuture<'a, ()> {
                Box::pin(async move {
                    self.play_sound(world, pumpkin_data::sound::Sound::BlockChestClose)
                        .await;
                })
            }

            fn on_viewer_count_update<'a>(
                &'a self,
                world: &'a Arc<$crate::world::World>,
                position: &'a pumpkin_util::math::position::BlockPos,
                old: u16,
                new: u16,
            ) -> $crate::block::viewer::ViewerFuture<'a, ()> {
                Box::pin(async move {
                    // Trigger block animation
                    world
                        .add_synced_block_event(
                            *position,
                            Self::LID_ANIMATION_EVENT_TYPE,
                            new as u8,
                        )
                        .await;

                    // Update neighbors for redstone signal when viewer count changes
                    // This is controlled by the EMITS_REDSTONE constant on the struct
                    if Self::EMITS_REDSTONE && old != new {
                        // Update direct neighbors
                        world.clone().update_neighbors(position, None).await;

                        // Also update neighbors of the block below (strongly powered block)
                        // This ensures redstone components adjacent to the block below are notified
                        let below_pos = position.down();
                        world.clone().update_neighbors(&below_pos, None).await;
                    }
                })
            }
        }
    };
}

/// Implements helper methods for chest-like block entities.
///
/// Includes the `play_sound` method which handles sound positioning for single and double chests,
/// as well as `new()` and `get_viewer_count()` methods.
#[macro_export]
macro_rules! impl_chest_helper_methods {
    ($struct_name:ty) => {
        impl $struct_name {
            /// Returns the number of players currently viewing this chest
            pub fn get_viewer_count(&self) -> u16 {
                self.viewers.get_viewer_count()
            }

            #[must_use]
            pub fn new(position: pumpkin_util::math::position::BlockPos) -> Self {
                use std::array::from_fn;
                use std::sync::atomic::AtomicBool;

                Self {
                    position,
                    items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
                    dirty: AtomicBool::new(false),
                    viewers: $crate::block::viewer::ViewerCountTracker::new(),
                }
            }

            async fn play_sound(
                &self,
                world: &Arc<$crate::world::World>,
                sound: pumpkin_data::sound::Sound,
            ) {
                let mut rng = pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(
                    pumpkin_util::random::get_seed(),
                );

                let (block, state) = world.get_block_and_state(&self.position).await;
                let properties = pumpkin_data::block_properties::ChestLikeProperties::from_state_id(
                    state.id, block,
                );
                let position = match properties.r#type {
                    pumpkin_data::block_properties::ChestType::Left => return,
                    pumpkin_data::block_properties::ChestType::Single => {
                        pumpkin_util::math::vector3::Vector3::new(
                            self.position.0.x as f64 + 0.5,
                            self.position.0.y as f64 + 0.5,
                            self.position.0.z as f64 + 0.5,
                        )
                    }
                    pumpkin_data::block_properties::ChestType::Right => {
                        let direction = pumpkin_data::HorizontalFacingExt::to_block_direction(
                            &properties.facing,
                        )
                        .to_offset();
                        pumpkin_util::math::vector3::Vector3::new(
                            self.position.0.x as f64 + 0.5 + direction.x as f64 * 0.5,
                            self.position.0.y as f64 + 0.5,
                            self.position.0.z as f64 + 0.5 + direction.z as f64 * 0.5,
                        )
                    }
                };

                world
                    .play_sound_fine(
                        sound,
                        pumpkin_data::sound::SoundCategory::Blocks,
                        &position,
                        0.5,
                        pumpkin_util::random::RandomImpl::next_f32(&mut rng) * 0.1 + 0.9,
                    )
                    .await;
            }
        }
    };
}
