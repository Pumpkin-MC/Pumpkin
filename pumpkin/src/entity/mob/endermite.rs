use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// `Endermite::MAX_LIFE` (Endermite.java).
const MAX_LIFE: i32 = 2400;

pub struct EndermiteEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `Endermite::life`: ticks alive since spawn, while not persistence-required;
    /// self-discards at `MAX_LIFE`. Pumpkin has no general `Mob::isPersistenceRequired`
    /// flag (set on equipment pickup, explicit `setPersistenceRequired`, or the
    /// `PersistenceRequired` save tag), so only the custom-name case
    /// (`Mob::requiresCustomPersistence`'s sibling check in vanilla's `Mob::checkDespawn`,
    /// approximated here) is modeled: a named endermite never expires from this timer.
    life: AtomicI32,
}

impl EndermiteEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let endermite = Self {
            mob_entity,
            life: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(endermite);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, ClimbOnTopOfPowderSnowGoal::new());
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for EndermiteEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("Lifetime", self.life.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.life
                .store(nbt.get_int("Lifetime").unwrap_or(0), Ordering::Relaxed);
        })
    }
}

impl Mob for EndermiteEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `Endermite::aiStep`'s server-side branch: while not persistence-required,
    /// increments `life` each tick and discards the endermite once it reaches `MAX_LIFE`.
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            if entity.custom_name.load().is_some() {
                return;
            }

            let new_life = self.life.fetch_add(1, Ordering::Relaxed) + 1;
            if new_life >= MAX_LIFE {
                entity.world.load().remove_entity(caller.as_ref()).await;
            }
        })
    }
}
