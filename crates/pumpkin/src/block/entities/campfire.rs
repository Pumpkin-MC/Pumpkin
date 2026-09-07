use super::BlockEntity;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::CampfireLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::io::Dirtiable;
use rand::{RngExt, rng};
use std::sync::{Arc, Mutex};

pub struct CampfireBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::SLOT_COUNT],
    pub cooking_times: [Mutex<i32>; Self::SLOT_COUNT],
    pub cooking_total_times: [Mutex<i32>; Self::SLOT_COUNT],
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
        let mut entity = Self::new(position);

        if let Some(list) = nbt.get_list("Items") {
            for tag in list {
                if let Some(compound) = tag.extract_compound() {
                    let slot = compound.get_byte("Slot").unwrap_or(0) as usize;
                    if slot < Self::SLOT_COUNT
                        && let Some(stack) = ItemStack::read_item_stack(compound)
                    {
                        entity.items[slot] = Arc::new(Mutex::new(stack));
                    }
                }
            }
        }

        if let Some(arr) = nbt.get_int_array("CookingTimes") {
            for (slot, &value) in arr.iter().enumerate().take(Self::SLOT_COUNT) {
                entity.cooking_times[slot] = Mutex::new(value);
            }
        }
        if let Some(arr) = nbt.get_int_array("CookingTotalTimes") {
            for (slot, &value) in arr.iter().enumerate().take(Self::SLOT_COUNT) {
                entity.cooking_total_times[slot] = Mutex::new(value);
            }
        }

        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_cooking_nbt(nbt);
    }

    fn tick(&self, world: &Arc<World>) {
        let block = world.get_block(&self.position);
        if block != &Block::CAMPFIRE && block != &Block::SOUL_CAMPFIRE {
            return;
        }

        let state_id = world.get_block_state(&self.position).id;
        let properties = CampfireLikeProperties::from_state_id(state_id);

        if !properties.lit {
            if self.cool_down() {
                self.mark_chunk_dirty(world);
            }
            return;
        }

        let mut changed = false;
        let mut completed = false;

        for slot in 0..Self::SLOT_COUNT {
            let stack = self.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if stack.is_empty() {
                continue;
            }

            let Some(recipe) =
                get_cooking_recipe_with_ingredient(stack.item, CookingRecipeKind::CampfireCooking)
            else {
                continue;
            };

            let total = {
                let mut total = self.cooking_total_times[slot]
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if *total <= 0 {
                    let mut cooking_time = recipe.cookingtime;
                    if let Some(server) = world.server.upgrade() {
                        let mut event = crate::plugin::api::events::block::campfire_start::CampfireStartEvent::new(
                            self.position,
                            world.clone(),
                            stack.clone(),
                            slot as u8,
                            cooking_time,
                        );
                        server.plugin_manager.fire_blocking(&server, &mut event);
                        if event.cancelled {
                            continue;
                        }
                        cooking_time = event.cooking_time;
                    }
                    *total = cooking_time;
                }
                *total
            };

            let finished = {
                let mut progress = self.cooking_times[slot]
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *progress += 1;
                changed = true;
                *progress >= total
            };

            if !finished {
                continue;
            }

            let Some(result_item) = Item::from_registry_key(recipe.result.id) else {
                continue;
            };

            let mut result = ItemStack::new(recipe.result.count, result_item);
            if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::api::events::block::block_cook::BlockCookEvent::new(
                    self.position,
                    world.clone(),
                    stack.clone(),
                    result.clone(),
                );
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    continue;
                }
                result = event.result;
            }
            Self::spawn_cooked_item(world, self.position, result);

            *self.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
            *self.cooking_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            *self.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            completed = true;
        }

        if completed {
            if let Some(block_entity) = world.get_block_entity(&self.position) {
                world.update_block_entity(&block_entity);
            }
        } else if changed {
            self.mark_chunk_dirty(world);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_cooking_nbt(&mut nbt);
        Some(nbt)
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, _position: &BlockPos) {
        for item in &self.items {
            let stack = std::mem::replace(
                &mut *item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                ItemStack::EMPTY.clone(),
            );
            if !stack.is_empty() {
                world.scatter_stack(
                    f64::from(self.position.0.x),
                    f64::from(self.position.0.y),
                    f64::from(self.position.0.z),
                    stack,
                );
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CampfireBlockEntity {
    pub const ID: &'static str = "minecraft:campfire";
    pub const SLOT_COUNT: usize = 4;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: std::array::from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            cooking_times: std::array::from_fn(|_| Mutex::new(0)),
            cooking_total_times: std::array::from_fn(|_| Mutex::new(0)),
        }
    }

    fn write_cooking_nbt(&self, nbt: &mut NbtCompound) {
        let mut items = Vec::new();
        for (slot, item) in self.items.iter().enumerate() {
            let stack = item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !stack.is_empty() {
                let mut item_nbt = NbtCompound::new();
                item_nbt.put_byte("Slot", slot as i8);
                stack.write_item_stack(&mut item_nbt);
                items.push(NbtTag::Compound(item_nbt));
            }
        }
        nbt.put_list("Items", items);

        let cooking_times = self
            .cooking_times
            .iter()
            .map(|time| {
                *time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .collect();
        nbt.put("CookingTimes", NbtTag::IntArray(cooking_times));

        let cooking_total_times = self
            .cooking_total_times
            .iter()
            .map(|time| {
                *time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .collect();
        nbt.put("CookingTotalTimes", NbtTag::IntArray(cooking_total_times));
    }

    fn cool_down(&self) -> bool {
        let mut changed = false;
        for progress in &self.cooking_times {
            let mut progress = progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let cooled = (*progress - 2).max(0);
            if cooled != *progress {
                *progress = cooled;
                changed = true;
            }
        }
        changed
    }

    fn mark_chunk_dirty(&self, world: &Arc<World>) {
        let chunk_position = self.position.chunk_position();
        let _ = world.level.read_chunk_sync(&chunk_position, |chunk| {
            chunk.mark_dirty(true);
        });
    }

    fn spawn_cooked_item(world: &Arc<World>, position: BlockPos, stack: ItemStack) {
        let spawn_position = Vector3::new(
            f64::from(position.0.x) + 0.5 + rng().random_range(-0.35..0.35),
            f64::from(position.0.y) + 1.01,
            f64::from(position.0.z) + 0.5 + rng().random_range(-0.35..0.35),
        );
        let entity = Entity::new(world.clone(), spawn_position, &EntityType::ITEM);
        world.spawn_entity(Arc::new(ItemEntity::new(entity, stack)));
    }
}
