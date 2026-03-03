use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{effect::StatusEffect, entity::EntityType, item::Item};
use pumpkin_world::item::ItemStack;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, SunSensitive},
    passive::villager::{VillagerEntity, VillagerProfession, VillagerType},
    player::Player,
};

use super::ZombieEntityBase;

/// Conversion time range in ticks (vanilla: 3600-6000 ticks = 3-5 minutes)
const MIN_CONVERSION_TICKS: i32 = 3600;
const MAX_CONVERSION_TICKS: i32 = 6000;

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntityBase>,
    /// Ticks remaining until conversion completes. -1 = not converting.
    pub conversion_timer: AtomicI32,
    /// The profession the villager had before being zombified.
    pub villager_profession: AtomicI32,
    /// The biome type of the original villager.
    pub villager_type: AtomicI32,
}

impl ZombieVillagerEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = ZombieEntityBase::new(entity).await;
        let zombie = Self {
            mob_entity,
            conversion_timer: AtomicI32::new(-1),
            villager_profession: AtomicI32::new(VillagerProfession::None as i32),
            villager_type: AtomicI32::new(VillagerType::Plains as i32),
        };
        Arc::new(zombie)
    }

    /// Check if this zombie villager is currently being cured.
    pub fn is_converting(&self) -> bool {
        self.conversion_timer.load(Ordering::Relaxed) >= 0
    }

    /// Start the curing conversion process.
    pub fn start_conversion(&self) {
        // Random time between MIN and MAX conversion ticks
        let seed = self.mob_entity.mob_entity.living_entity.entity.entity_id as u64;
        let range = (MAX_CONVERSION_TICKS - MIN_CONVERSION_TICKS) as u64;
        let time = MIN_CONVERSION_TICKS
            + ((seed * 6364136223846793005 + 1442695040888963407) % range) as i32;
        self.conversion_timer.store(time, Ordering::Relaxed);
    }

    /// Complete the conversion: remove zombie villager and spawn a villager.
    async fn finish_conversion(&self) {
        let entity = &self.mob_entity.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let pos = entity.pos.load();

        // Create a new villager entity
        let villager_entity = Entity::new(world.clone(), pos, &EntityType::VILLAGER);

        let villager = VillagerEntity::new(villager_entity).await;

        // Set the profession and type from the original zombie villager
        let profession =
            VillagerProfession::from_i32(self.villager_profession.load(Ordering::Relaxed));
        villager.set_profession(profession);

        let vtype = VillagerType::from_i32(self.villager_type.load(Ordering::Relaxed));
        villager.set_villager_type(vtype);

        // Populate trades if the villager had a profession
        if profession != VillagerProfession::None && profession != VillagerProfession::Nitwit {
            villager.populate_all_trades().await;
        }

        // Sync metadata so clients see the correct appearance
        villager.sync_villager_data().await;

        // Remove the zombie villager
        entity.remove().await;

        // Spawn the new villager
        world.spawn_entity(villager).await;
    }
}

impl NBTStorage for ZombieVillagerEntity {}

impl Mob for ZombieVillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Sun burning logic
            self.sun_sensitive_tick().await;

            // Curing conversion countdown
            let timer = self.conversion_timer.load(Ordering::Relaxed);
            if timer >= 0 {
                let new_timer = timer - 1;
                if new_timer <= 0 {
                    // Conversion complete
                    self.finish_conversion().await;
                } else {
                    self.conversion_timer.store(new_timer, Ordering::Relaxed);
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        _player: &'a Player,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Check if already converting
            if self.is_converting() {
                return false;
            }

            // Check if player is holding a golden apple
            let is_golden_apple = item_stack.get_item().id == Item::GOLDEN_APPLE.id;
            if !is_golden_apple {
                return false;
            }

            // Check if the zombie villager has the weakness effect
            let has_weakness = self
                .mob_entity
                .mob_entity
                .living_entity
                .has_effect(&StatusEffect::WEAKNESS)
                .await;

            if !has_weakness {
                return false;
            }

            // Consume the golden apple
            item_stack.item_count = item_stack.item_count.saturating_sub(1);

            // Start conversion
            self.start_conversion();

            // TODO: Play cure sound effect (EntityStatus 16)
            // TODO: Show particles

            true
        })
    }
}

impl SunSensitive for ZombieVillagerEntity {}
