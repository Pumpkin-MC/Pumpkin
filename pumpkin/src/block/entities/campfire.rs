use super::BlockEntity;
use pumpkin_data::block_properties::{BlockProperties, CampfireLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient};
use pumpkin_data::Block;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use tokio::sync::Mutex;

use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;

pub struct CampfireBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 4],
    pub cooking_times: [AtomicI32; 4],
    pub cooking_total_times: [AtomicI32; 4],
    pub dirty: AtomicBool,
    tick_counter: AtomicUsize,
}

impl BlockEntity for CampfireBlockEntity {
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
        let mut items = std::array::from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())));
        if let Some(list) = nbt.get_list("Items") {
            for tag in list {
                if let Some(compound) = tag.extract_compound() {
                    let slot = compound.get_byte("Slot").unwrap_or(0) as usize;
                    if slot < 4
                        && let Some(stack) = ItemStack::read_item_stack(compound)
                    {
                        items[slot] = Arc::new(Mutex::new(stack));
                    }
                }
            }
        }
        let mut cooking_times =
            [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
        if let Some(arr) = nbt.get_int_array("CookingTimes") {
            for (i, &val) in arr.iter().enumerate().take(4) {
                cooking_times[i] = AtomicI32::new(val);
            }
        }
        let mut cooking_total_times =
            [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
        if let Some(arr) = nbt.get_int_array("CookingTotalTimes") {
            for (i, &val) in arr.iter().enumerate().take(4) {
                cooking_total_times[i] = AtomicI32::new(val);
            }
        }

        Self {
            position,
            items,
            cooking_times,
            cooking_total_times,
            dirty: AtomicBool::new(false),
            tick_counter: AtomicUsize::new(0),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let mut list = Vec::new();
            for (i, item_mutex) in self.items.iter().enumerate() {
                let stack = item_mutex.lock().await;
                if !stack.is_empty() {
                    let mut item_nbt = NbtCompound::new();
                    item_nbt.put_byte("Slot", i as i8);
                    stack.write_item_stack(&mut item_nbt);
                    list.push(NbtTag::Compound(item_nbt));
                }
            }
            nbt.put_list("Items", list);

            let mut times = Vec::new();
            for ct in &self.cooking_times {
                times.push(ct.load(Ordering::Relaxed));
            }
            nbt.put("CookingTimes", NbtTag::IntArray(times));

            let mut total_times = Vec::new();
            for ctt in &self.cooking_total_times {
                total_times.push(ctt.load(Ordering::Relaxed));
            }
            nbt.put("CookingTotalTimes", NbtTag::IntArray(total_times));
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        let mut items_list = Vec::new();
        for (i, item_mutex) in self.items.iter().enumerate() {
            if let Ok(stack) = item_mutex.try_lock()
                && !stack.is_empty()
            {
                let mut item_nbt = NbtCompound::new();
                item_nbt.put_byte("Slot", i as i8);
                stack.write_item_stack(&mut item_nbt);
                items_list.push(NbtTag::Compound(item_nbt));
            }
        }
        nbt.put_list("Items", items_list);

        let mut times = Vec::new();
        for ct in &self.cooking_times {
            times.push(ct.load(Ordering::Relaxed));
        }
        nbt.put("CookingTimes", NbtTag::IntArray(times));

        let mut total_times = Vec::new();
        for ctt in &self.cooking_total_times {
            total_times.push(ctt.load(Ordering::Relaxed));
        }
        nbt.put("CookingTotalTimes", NbtTag::IntArray(total_times));
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    #[allow(clippy::too_many_lines)]
    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let block = world.get_block(&self.position);
            if *block != Block::CAMPFIRE && *block != Block::SOUL_CAMPFIRE {
                return;
            }

            let state_id = world.get_block_state(&self.position).id;
            let props = CampfireLikeProperties::from_state_id(state_id, block);

            if !props.lit {
                return;
            }

            let tick = self.tick_counter.fetch_add(1, Ordering::Relaxed);

            for slot in 0..4 {
                let (item_type, is_empty) = {
                    let guard = self.items[slot].lock().await;
                    (guard.item, guard.is_empty())
                };

                if is_empty {
                    if self.cooking_times[slot].load(Ordering::Relaxed) != 0 {
                        self.cooking_times[slot].store(0, Ordering::Relaxed);
                        self.dirty.store(true, Ordering::Relaxed);
                    }
                    if self.cooking_total_times[slot].load(Ordering::Relaxed) != 0 {
                        self.cooking_total_times[slot].store(0, Ordering::Relaxed);
                        self.dirty.store(true, Ordering::Relaxed);
                    }
                    continue;
                }

                if let Some(recipe) =
                    get_cooking_recipe_with_ingredient(item_type, CookingRecipeKind::CampfireCooking)
                {
                    let total = self.cooking_total_times[slot].load(Ordering::Relaxed);
                    if total == 0 {
                        self.cooking_total_times[slot]
                            .store(recipe.cookingtime, Ordering::Relaxed);
                        self.dirty.store(true, Ordering::Relaxed);
                    }
                    let total = self.cooking_total_times[slot].load(Ordering::Relaxed);

                    let prev = self.cooking_times[slot].fetch_add(1, Ordering::Relaxed);
                    let finished = prev + 1 >= total;

                    if finished {
                        let auto_pop = world
                            .server
                            .upgrade()
                            .is_none_or(|s| s.basic_config.auto_pop_off);

                        if auto_pop {
                            if let Some(result_item) = Item::from_registry_key(recipe.result.id) {
                                let result_stack =
                                    ItemStack::new(recipe.result.count, result_item);

                                let pos_down = self.position.down();
                                let inserted = if let Some(hopper_entity) =
                                    world.get_block_entity(&pos_down)
                                    && let Some(hopper_inv) = hopper_entity.get_inventory()
                                {
                                    crate::block::entities::hopper::HopperBlockEntity::add_one_item(
                                        self,
                                        hopper_inv.as_ref(),
                                        result_stack.clone(),
                                    )
                                    .await
                                } else {
                                    false
                                };

                                if !inserted {
                                    let spawn_pos = Vector3::new(
                                        f64::from(self.position.0.x) + 0.5,
                                        f64::from(self.position.0.y) + 0.7,
                                        f64::from(self.position.0.z) + 0.5,
                                    );
                                    let entity = Entity::new(
                                        world.clone(),
                                        spawn_pos,
                                        &EntityType::ITEM,
                                    );
                                    let item_entity =
                                        Arc::new(ItemEntity::new(entity, result_stack));
                                    world.spawn_entity(item_entity).await;
                                }
                            }
                            *self.items[slot].lock().await = ItemStack::EMPTY.clone();
                        } else if let Some(result_item) =
                            Item::from_registry_key(recipe.result.id)
                        {
                            let result_stack =
                                ItemStack::new(recipe.result.count, result_item);
                            *self.items[slot].lock().await = result_stack;
                        }
                        self.cooking_times[slot].store(0, Ordering::Relaxed);
                        self.cooking_total_times[slot].store(0, Ordering::Relaxed);
                        self.dirty.store(true, Ordering::Relaxed);

                        if let Some(entity_arc) = world.get_block_entity(&self.position) {
                            world.update_block_entity(&entity_arc);
                        }
                    }
                }
            }

            if tick.is_multiple_of(3) && let Some(particle) = if props.signal_fire {
                    Some(pumpkin_data::particle::Particle::CampfireSignalSmoke)
                } else if *block == Block::SOUL_CAMPFIRE {
                    Some(pumpkin_data::particle::Particle::Soul)
                } else {
                    Some(pumpkin_data::particle::Particle::CampfireCosySmoke)
                } {
                    let pos = pumpkin_util::math::vector3::Vector3::new(
                        f64::from(self.position.0.x) + 0.5,
                        f64::from(self.position.0.y) + 1.0,
                        f64::from(self.position.0.z) + 0.5,
                    );
                    world.spawn_particle(
                        pos,
                        pumpkin_util::math::vector3::Vector3::new(0.05, 0.0, 0.05),
                        0.01,
                        1,
                        particle,
                    );
                }
        })
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CampfireBlockEntity {
    pub const ID: &'static str = "minecraft:campfire";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: std::array::from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            cooking_times: [
                AtomicI32::new(0),
                AtomicI32::new(0),
                AtomicI32::new(0),
                AtomicI32::new(0),
            ],
            cooking_total_times: [
                AtomicI32::new(0),
                AtomicI32::new(0),
                AtomicI32::new(0),
                AtomicI32::new(0),
            ],
            dirty: AtomicBool::new(false),
            tick_counter: AtomicUsize::new(0),
        }
    }
}

impl Inventory for CampfireBlockEntity {
    fn size(&self) -> usize {
        4
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for item in &self.items {
                if !item.lock().await.is_empty() {
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
            let mut guard = self.items[slot].lock().await;
            if guard.is_empty() {
                return ItemStack::EMPTY.clone();
            }
            let result = guard.split(amount);
            self.mark_dirty();
            result
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
            self.mark_dirty();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn can_transfer_to(
        &self,
        _hopper_inventory: &dyn Inventory,
        _slot: usize,
        _stack: &ItemStack,
    ) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for CampfireBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for i in 0..4 {
                *self.items[i].lock().await = ItemStack::EMPTY.clone();
                self.cooking_times[i].store(0, Ordering::Relaxed);
                self.cooking_total_times[i].store(0, Ordering::Relaxed);
            }
            self.mark_dirty();
        })
    }
}
