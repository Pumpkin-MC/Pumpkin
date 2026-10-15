use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::item::ItemEntity;
use crate::entity::living::LivingEntity;
use crate::entity::{Entity, EntityBase};
use crate::server::Server;

pub const DEFAULT_MIN_SPAWN_TICKS: i64 = 60;
pub const DEFAULT_MAX_SPAWN_TICKS: i64 = 120;
pub const TICKS_BEFORE_ABOUT_TO_SPAWN_SOUND: i64 = 36;

pub struct OminousItemSpawnerEntity {
    pub entity: Entity,
    item_stack: Mutex<ItemStack>,
    spawn_item_after_ticks: AtomicI64,
    ticks: AtomicI64,
}

impl OminousItemSpawnerEntity {
    pub fn new(entity: Entity) -> Self {
        let delay = rand::random_range(DEFAULT_MIN_SPAWN_TICKS..=DEFAULT_MAX_SPAWN_TICKS);
        Self::new_with_delay(entity, ItemStack::new(0, &pumpkin_data::item::Item::AIR), delay)
    }

    pub fn new_with_delay(entity: Entity, item: ItemStack, delay: i64) -> Self {
        Self {
            entity,
            item_stack: Mutex::new(item),
            spawn_item_after_ticks: AtomicI64::new(delay),
            ticks: AtomicI64::new(0),
        }
    }

    pub fn item(&self) -> ItemStack {
        self.item_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_item(&self, item: ItemStack) {
        *self
            .item_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = item;
    }

    pub fn spawn_item_after_ticks(&self) -> i64 {
        self.spawn_item_after_ticks.load(Ordering::Relaxed)
    }

    pub fn set_spawn_item_after_ticks(&self, ticks: i64) {
        self.spawn_item_after_ticks.store(ticks, Ordering::Relaxed);
    }
}

impl EntityBase for OminousItemSpawnerEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let current_tick = self.ticks.fetch_add(1, Ordering::Relaxed) + 1;
        let spawn_delay = self.spawn_item_after_ticks.load(Ordering::Relaxed);
        let world = self.entity.world.load();

        if current_tick == spawn_delay - TICKS_BEFORE_ABOUT_TO_SPAWN_SOUND {
            let pos = self.entity.pos.load();
            world.play_sound(
                Sound::BlockTrialSpawnerAboutToSpawnItem,
                SoundCategory::Neutral,
                &pos,
            );
        }

        if current_tick >= spawn_delay {
            let item = self
                .item_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();

            if !item.is_empty() {
                let pos = self.entity.pos.load();
                let item_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
                let spawned = Arc::new(ItemEntity::new_with_velocity(
                    item_entity,
                    item,
                    Vector3::new(0.0, 0.0, 0.0),
                    10,
                ));
                world.spawn_entity(spawned);

                world.sync_world_event(
                    WorldEvent::ParticlesTrialSpawnerSpawnItem,
                    self.entity.block_pos.load(),
                    1,
                );
            }

            self.entity.remove();
        }
    }

    fn damage(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
    ) -> bool {
        // Matches vanilla hurtServer -> false
        false
    }

    fn get_gravity(&self) -> f64 {
        0.0
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
