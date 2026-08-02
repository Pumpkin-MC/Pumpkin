use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        skeleton_trap::SkeletonTrapGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Vanilla: `SkeletonHorse.TRAP_MAX_LIFE` -- an un-persistence-required trap horse despawns after
/// this many ticks if it's never triggered.
const TRAP_MAX_LIFE: i32 = 18000;

pub struct SkeletonHorseEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `SkeletonHorse.isTrap` -- set on horses spawned by the "lightning near a lightning
    /// rod" environmental trap mechanic. Nothing in Pumpkin currently spawns a skeleton horse
    /// with this set to `true` at construction time (the lightning-triggered spawn path in
    /// `World`'s chunk tick spawns a plain, non-trap `SkeletonHorseEntity` today), so in practice
    /// this stays `false` -- exactly like `MoveTowardsRestrictionGoal`'s dormant
    /// `position_target_range`, it's still correct to carry the field and the gated goal so that
    /// whichever spawn path is later updated to set it just works.
    is_trap: AtomicBool,
    trap_time: AtomicI32,
}

impl SkeletonHorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            is_trap: AtomicBool::new(false),
            trap_time: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(horse);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Vanilla priority 1 (dynamically added/removed via `setTrap`); see
            // `SkeletonTrapGoal`'s doc comment for why Pumpkin registers it unconditionally.
            goal_selector.add_goal(1, SkeletonTrapGoal::new(Arc::downgrade(&mob_arc)));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_trap(&self) -> bool {
        self.is_trap.load(Ordering::Relaxed)
    }

    pub fn set_trap(&self, trap: bool) {
        self.is_trap.store(trap, Ordering::Relaxed);
    }
}

impl NBTStorage for SkeletonHorseEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_bool("SkeletonTrap", self.is_trap.load(Ordering::Relaxed));
            nbt.put_int("SkeletonTrapTime", self.trap_time.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.is_trap.store(
                nbt.get_bool("SkeletonTrap").unwrap_or(false),
                Ordering::Relaxed,
            );
            self.trap_time.store(
                nbt.get_int("SkeletonTrapTime").unwrap_or(0),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for SkeletonHorseEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla: `SkeletonHorse.aiStep` -- an untriggered trap horse despawns after
            // `TRAP_MAX_LIFE` ticks. `isPersistenceRequired` gating is skipped (Pumpkin doesn't
            // expose that flag to entities generically here); this only matters once something
            // actually spawns a trap horse, which nothing does yet (see `is_trap`'s doc comment).
            if self.is_trap.load(Ordering::Relaxed) {
                let elapsed = self.trap_time.fetch_add(1, Ordering::Relaxed) + 1;
                if elapsed >= TRAP_MAX_LIFE {
                    self.mob_entity.living_entity.entity.remove().await;
                }
            }
        })
    }
}
