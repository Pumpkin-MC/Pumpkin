use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

pub const HAPPY_GHAST_FOOD: &[&Item] = &[&Item::SNOWBALL];

/// Represents a Happy Ghast, a passive flying mob.
///
/// Wiki: <https://minecraft.wiki/w/Happy_Ghast>
pub struct HappyGhastEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub server_still_timeout: AtomicI32,
    pub leash_holder_time: AtomicI32,
    pub is_leash_holder: AtomicBool,
    pub stays_still: AtomicBool,
}

impl HappyGhastEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let happy_ghast = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            server_still_timeout: AtomicI32::new(0),
            leash_holder_time: AtomicI32::new(0),
            is_leash_holder: AtomicBool::new(false),
            stays_still: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(happy_ghast);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(TemptGoal::new(1.0, HAPPY_GHAST_FOOD)));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    pub fn set_server_still_timeout(&self, timeout: i32) {
        self.server_still_timeout.store(timeout, Ordering::Relaxed);
        self.sync_stay_still_flag();
    }

    #[must_use]
    pub fn is_on_still_timeout(&self) -> bool {
        self.stays_still.load(Ordering::Relaxed)
            || self.server_still_timeout.load(Ordering::Relaxed) > 0
    }

    fn set_leash_holder(&self, is_leash_holder: bool) {
        self.is_leash_holder
            .store(is_leash_holder, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
                is_leash_holder,
            )],
            None,
        );
    }

    fn sync_stay_still_flag(&self) {
        let stays_still = self.server_still_timeout.load(Ordering::Relaxed) > 0;
        self.stays_still.store(stays_still, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
                stays_still,
            )],
            None,
        );
    }
}

impl AgeableMob for HappyGhastEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for HappyGhastEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_HAPPY_GHAST_FOOD)
            || HAPPY_GHAST_FOOD.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for HappyGhastEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            "still_timeout",
            self.server_still_timeout.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(timeout) = nbt.get_int("still_timeout") {
            self.set_server_still_timeout(timeout);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) {
        self.ageable_ai_step();

        let leash_time = self.leash_holder_time.load(Ordering::Relaxed);
        if leash_time > 0 {
            self.leash_holder_time.fetch_sub(1, Ordering::Relaxed);
        }
        self.set_leash_holder(leash_time > 0);

        let still_timeout = self.server_still_timeout.load(Ordering::Relaxed);
        if still_timeout > 0 {
            let entity = self.get_entity();
            if entity.age.load(Ordering::Relaxed) > 60 {
                self.server_still_timeout.fetch_sub(1, Ordering::Relaxed);
            }
            self.sync_stay_still_flag();
        }

        // Continuous healing
        let entity = self.get_entity();
        if entity.is_alive() {
            let living = &self.mob_entity.living_entity;
            let current_health = living.health.load();
            let max_health = living.get_max_health();
            if current_health < max_health {
                let world = entity.world.load();
                let ticks = world.get_world_age();
                let heal_interval = 600;
                if ticks % heal_interval == 0 {
                    living.set_health(current_health + 1.0);
                }
            }
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::happy_ghast::BABY_ID,
                    true,
                )],
                None,
            );
        }
        entity.send_meta_data(
            &[
                Metadata::new(
                    pumpkin_data::tracked_data::happy_ghast::IS_LEASH_HOLDER,
                    self.is_leash_holder.load(Ordering::Relaxed),
                ),
                Metadata::new(
                    pumpkin_data::tracked_data::happy_ghast::STAYS_STILL,
                    self.stays_still.load(Ordering::Relaxed),
                ),
            ],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if self.is_baby() {
            return self.animal_interact(player, item_stack, Sound::EntityGhastlingAmbient);
        }

        self.animal_interact(player, item_stack, Sound::EntityHappyGhastAmbient)
    }
}
