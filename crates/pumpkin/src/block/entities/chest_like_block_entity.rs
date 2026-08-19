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
                // Read deferred loot-table fields first.
                let loot_table_key = nbt.get_string("LootTable").map(|s| s.to_string());
                let loot_table_seed = nbt.get_long("LootTableSeed").unwrap_or(0);

                let mut chest = Self {
                    position,
                    items: tokio::sync::RwLock::new(std::array::from_fn(|_| ItemStack::EMPTY.clone())),
                    dirty: std::sync::atomic::AtomicBool::new(false),
                    viewers: $crate::block::viewer::ViewerCountTracker::new(),
                    loot_table: StdMutex::new(loot_table_key),
                    loot_table_seed,
                };

                // Only read saved items when there is no pending loot table.
                let has_loot_table = chest
                    .loot_table
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .is_some();
                if !has_loot_table {
                    pumpkin_world::inventory::sync_read_items_from_nbt(nbt, chest.items.get_mut());
                }

                chest
            }

            fn write_nbt<'a>(
                &'a self,
                nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
                use pumpkin_world::inventory::Inventory;

                Box::pin(async move {
                    // Clone the loot table key without holding the lock across an await.
                    let loot_table_key = {
                        let guard = self.loot_table.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        guard.clone()
                    };

                    if let Some(key) = loot_table_key {
                        // Persist deferred loot: write the key and seed; skip items.
                        nbt.put_string("LootTable", key);
                        if self.loot_table_seed != 0 {
                            nbt.put_long("LootTableSeed", self.loot_table_seed);
                        }
                    } else {
                        // Loot has already been generated, so persist the actual items.
                        self.write_inventory_nbt(nbt, true).await;
                    }
                })
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

            fn chunk_data_nbt(&self) -> Option<pumpkin_nbt::compound::NbtCompound> {
                let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
                let loot_table_key = self.loot_table.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
                if let Some(key) = loot_table_key {
                    nbt.put_string("LootTable", key);
                    if self.loot_table_seed != 0 {
                        nbt.put_long("LootTableSeed", self.loot_table_seed);
                    }
                } else {
                    let items = futures::executor::block_on(self.items.read());
                    pumpkin_world::inventory::sync_write_items_to_nbt(&*items, &mut nbt);
                }
                Some(nbt)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn take_loot_table(&self) -> Option<(String, i64)> {
                let mut guard = self.loot_table.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.take().map(|key| (key, self.loot_table_seed))
            }

            fn has_loot_table(&self) -> bool {
                self.loot_table.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some()
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
                Self::INVENTORY_SIZE
            }

            fn is_empty(&self) -> pumpkin_world::inventory::InventoryFuture<'_, bool> {
                Box::pin(async move {
                    let items = self.items.read().await;
                    items.iter().all(|s| s.is_empty())
                })
            }

            fn get_stack(
                &self,
                slot: usize,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let items = self.items.read().await;
                    items[slot].clone()
                })
            }

            fn remove_stack(
                &self,
                slot: usize,
            ) -> pumpkin_world::inventory::InventoryFuture<'_, ItemStack> {
                Box::pin(async move {
                    let mut items = self.items.write().await;
                    let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
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
                    let mut items = self.items.write().await;
                    let res = if !items[slot].is_empty() && amount > 0 {
                        items[slot].split(amount)
                    } else {
                        ItemStack::EMPTY.clone()
                    };
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
                    let mut items = self.items.write().await;
                    items[slot] = stack;
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
                    let mut items = self.items.write().await;
                    items.fill_with(|| ItemStack::EMPTY.clone());
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
                use std::sync::Mutex as StdMutex;
                use std::sync::atomic::AtomicBool;

                Self {
                    position,
                    items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
                    dirty: AtomicBool::new(false),
                    viewers: $crate::block::viewer::ViewerCountTracker::new(),
                    loot_table: StdMutex::new(None),
                    loot_table_seed: 0,
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

                let (block, state) = world.get_block_and_state(&self.position);
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

                world.play_sound_fine(
                    sound,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    &position,
                    0.5,
                    pumpkin_util::random::RandomImpl::next_f32(&mut rng) * 0.1 + 0.9,
                );
                let bedrock_sound = match sound {
                    pumpkin_data::sound::Sound::BlockChestOpen => "chest.open",
                    pumpkin_data::sound::Sound::BlockChestClose => "chest.closed",
                    _ => return,
                };
                world.play_bedrock_level_sound(bedrock_sound, &position, 0);
            }
        }
    };
}
