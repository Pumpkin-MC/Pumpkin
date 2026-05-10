use futures::Future;
use pumpkin_data::data_component_impl::IDSetContent;
use pumpkin_data::tag::Taggable;
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::block::entities::BlockEntity;
use crate::world::World;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};

pub struct BeaconBlockEntity {
    pub position: BlockPos,
    pub primary_effect: AtomicI32,
    pub secondary_effect: AtomicI32,
    pub levels: AtomicI32,
    pub dirty: AtomicBool,
    pub payment: Arc<Mutex<ItemStack>>,

    // Vanilla Parity Fields
    pub custom_name: Mutex<Option<String>>,
    pub lock_key: Mutex<Option<String>>,
    pub last_check_y: AtomicI32,
}

impl BeaconBlockEntity {
    pub const ID: &'static str = "minecraft:beacon";

    // ContainerData Property Constants
    pub const DATA_LEVELS: usize = 0;
    pub const DATA_PRIMARY: usize = 1;
    pub const DATA_SECONDARY: usize = 2;
    pub const NUM_DATA_VALUES: usize = 3;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            primary_effect: AtomicI32::new(-1),
            secondary_effect: AtomicI32::new(-1),
            levels: AtomicI32::new(0),
            dirty: AtomicBool::new(false),
            payment: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            custom_name: Mutex::new(None),
            lock_key: Mutex::new(None),
            last_check_y: AtomicI32::new(position.0.y - 1),
        }
    }

    /// Replicates the Java `ContainerData` used to sync values to the `BeaconMenu`
    pub fn get_data(&self, id: usize) -> i32 {
        match id {
            Self::DATA_LEVELS => self.levels.load(Ordering::Relaxed),
            Self::DATA_PRIMARY => self.primary_effect.load(Ordering::Relaxed),
            Self::DATA_SECONDARY => self.secondary_effect.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    pub fn set_data(&self, id: usize, value: i32) {
        match id {
            Self::DATA_LEVELS => self.levels.store(value, Ordering::Relaxed),
            Self::DATA_PRIMARY => self.primary_effect.store(value, Ordering::Relaxed),
            Self::DATA_SECONDARY => self.secondary_effect.store(value, Ordering::Relaxed),
            _ => {}
        }
        self.mark_dirty();
    }

    /// Replicates Java's `updateBase` logic
    async fn update_base(&self, world: &Arc<World>) -> i32 {
        let mut levels = 0;
        let x = self.position.0.x;
        let y = self.position.0.y;
        let z = self.position.0.z;

        for step in 1..=4 {
            let ly = y - step;
            if ly < world.dimension.min_y {
                break;
            }

            let mut is_ok = true;
            for lx in (x - step)..=(x + step) {
                for lz in (z - step)..=(z + step) {
                    let pos = BlockPos::new(lx, ly, lz);
                    let block = world.get_block(&pos).await;
                    if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEACON_BASE_BLOCKS) {
                        is_ok = false;
                        break;
                    }
                }
                if !is_ok {
                    break;
                }
            }

            if !is_ok {
                break;
            }
            levels = step;
        }
        levels
    }

    /// Replicates Java's `applyEffects` bounding box mapping and duration mapping
    async fn apply_effects(&self, world: &Arc<World>, levels: i32) {
        if levels <= 0 {
            return;
        }

        let primary_id = self.primary_effect.load(Ordering::Relaxed);
        let secondary_id = self.secondary_effect.load(Ordering::Relaxed);

        if primary_id <= 0 {
            return;
        }

        let primary_effect = StatusEffect::from_id(primary_id as u16);
        let secondary_effect = if secondary_id > 0 {
            StatusEffect::from_id(secondary_id as u16)
        } else {
            None
        };

        // Vanilla: expandTowards(0.0, level.getHeight(), 0.0) -> Reaches across the entire Y axis
        let range = (levels * 10 + 10) as f64;
        let pos = self.position.0.to_f64();

        // Use the dimension height for vanilla parity (usually 384.0 in modern versions)
        let world_height = world.dimension.height as f64;

        let bounding_box = BoundingBox::new(pos, pos.add_raw(1.0, 1.0, 1.0))
            .expand(range, range, range)
            .expand_towards(0.0, world_height, 0.0);

        let players = world.get_players_at_box(&bounding_box);

        let duration_ticks = (9 + levels * 2) * 20;
        let base_amp = i32::from(levels >= 4 && primary_id == secondary_id);

        for player in players {
            if let Some(effect) = primary_effect {
                player
                    .add_effect(pumpkin_data::potion::Effect {
                        effect_type: effect,
                        duration: duration_ticks,
                        amplifier: base_amp as u8,
                        ambient: true,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }

            if levels >= 4
                && primary_id != secondary_id
                && let Some(effect) = secondary_effect
            {
                player
                    .add_effect(pumpkin_data::potion::Effect {
                        effect_type: effect,
                        duration: duration_ticks,
                        amplifier: 0,
                        ambient: true,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }
        }
    }
}

impl BlockEntity for BeaconBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        // Aligning to strict vanilla NBT tags
        let primary = nbt.get_int("primary_effect").unwrap_or(-1);
        let secondary = nbt.get_int("secondary_effect").unwrap_or(-1);
        let levels = nbt.get_int("Levels").unwrap_or(0); // Vanilla uses capital L
        let custom_name = nbt
            .get_string("CustomName")
            .map(std::string::ToString::to_string);
        let lock_key = nbt.get_string("Lock").map(std::string::ToString::to_string);

        Self {
            position,
            primary_effect: AtomicI32::new(primary),
            secondary_effect: AtomicI32::new(secondary),
            levels: AtomicI32::new(levels),
            dirty: AtomicBool::new(false),
            payment: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            custom_name: Mutex::new(custom_name),
            lock_key: Mutex::new(lock_key),
            last_check_y: AtomicI32::new(position.0.y - 1),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_int(
                "primary_effect",
                self.primary_effect.load(Ordering::Relaxed),
            );
            nbt.put_int(
                "secondary_effect",
                self.secondary_effect.load(Ordering::Relaxed),
            );
            nbt.put_int("Levels", self.levels.load(Ordering::Relaxed));

            if let Some(name) = &*self.custom_name.lock().await {
                nbt.put_string("CustomName", name.clone());
            }
            if let Some(lock) = &*self.lock_key.lock().await {
                nbt.put_string("Lock", lock.clone());
            }
        })
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Check properties every 80 ticks matching Java
            if world.get_time_of_day().await % 80 == 0 {
                let levels = self.update_base(world).await;
                self.levels.store(levels, Ordering::Relaxed);

                // TODO: Beam Section validation (scanning upward to heightmap to check for sky visibility)
                // is typically checked here before applying effects in Vanilla.

                if levels > 0 {
                    self.apply_effects(world, levels).await;
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self as Arc<dyn Inventory>)
    }
}

impl Inventory for BeaconBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move { self.payment.lock().await.is_empty() })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move {
            if slot == 0 {
                self.payment.clone()
            } else {
                Arc::new(Mutex::new(ItemStack::EMPTY.clone()))
            }
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot == 0 {
                let mut removed = ItemStack::EMPTY.clone();
                let mut guard = self.payment.lock().await;
                std::mem::swap(&mut removed, &mut *guard);
                self.mark_dirty();
                removed
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot == 0 {
                let mut stack = self.payment.lock().await;
                if stack.is_empty() {
                    return ItemStack::EMPTY.clone();
                }
                let res = stack.split(amount);
                self.mark_dirty();
                res
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            if slot == 0 {
                *self.payment.lock().await = stack;
                self.mark_dirty();
            }
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for BeaconBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if let Ok(mut payment) = self.payment.try_lock() {
                *payment = ItemStack::EMPTY.clone();
            }
        })
    }
}
