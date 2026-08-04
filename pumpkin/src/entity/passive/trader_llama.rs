use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        trader_llama_defend_wandering_trader::TraderLlamaDefendWanderingTraderGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::wandering_trader::WanderingTraderEntity,
};

/// Vanilla `TraderLlama::despawnDelay` default (`TraderLlama.java:28-29`).
const DEFAULT_DESPAWN_DELAY: i32 = 47999;

/// `TraderLlama.java`.
///
/// Most of vanilla's `canDespawn` (`TraderLlama.java:101-107`) is
/// unrepresentable here: `llama.rs` (this entity's non-trader sibling) carries no taming,
/// age-lock, or persistence-required state at all, so those guards are not ported --
/// documented gap, not silently invented as a `tamed` field that would do nothing.
/// `doPlayerRide`'s mount-block override (`TraderLlama.java:75-81`) is not implemented:
/// Pumpkin has no generic animal-mounting/riding system to attach it to (confirmed absent
/// from `entity/mod.rs` and `mob/mod.rs` -- only minecart-passenger and leash-follow logic
/// exist). `finalizeSpawn`'s spawn-reason age-forcing (`TraderLlama.java:117-130`) is also
/// not ported (no `EntitySpawnReason` concept confirmed in this pass).
pub struct TraderLlamaEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `despawnDelay` (`TraderLlama.java:28-29, 71-73`).
    pub despawn_delay: AtomicI32,
}

impl TraderLlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self {
            mob_entity,
            despawn_delay: AtomicI32::new(DEFAULT_DESPAWN_DELAY),
        };
        let mob_arc = Arc::new(llama);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));
        };

        {
            // Vanilla `TraderLlama.registerGoals` target selector, priority 1
            // (`TraderLlama.java:66`). The priority-2 zombie/`AbstractIllager`
            // `NearestAttackableTargetGoal`s from the same method are not ported here --
            // they target a multi-type class predicate that doesn't fit
            // `ActiveTargetGoal`'s single-`EntityType` shape, and are out of scope for "a
            // llama defends its wandering trader" specifically.
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, TraderLlamaDefendWanderingTraderGoal::new());
        };

        mob_arc
    }
}

impl NBTStorage for TraderLlamaEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            nbt.put_int("DespawnDelay", self.despawn_delay.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            if let Some(delay) = nbt.get_int("DespawnDelay") {
                self.despawn_delay.store(delay, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for TraderLlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `TraderLlama::maybeDespawn` (`TraderLlama.java:91-99`), with `canDespawn`
    /// (`TraderLlama.java:101-107`) reduced to just "not persistence-required" since the
    /// tamed/age-lock/passenger guards have nothing to check against on this entity (see
    /// struct doc). While leashed to a `WanderingTrader`, the countdown slaves itself to the
    /// trader's own `despawn_delay` minus one every tick (`TraderLlama.java:93-95`) so it
    /// automatically tracks any reset/extension of the trader's timer; otherwise it
    /// decrements independently.
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // `canDespawn`'s "not persistence-required" guard (`TraderLlama.java:107`) is not
            // checked: Pumpkin has no persistence-required/`isPersistenceRequired` concept on
            // any entity (confirmed absent from `entity/mod.rs`), not just this one -- a
            // pre-existing gap in the despawn system generally, not introduced here.
            let entity = &self.mob_entity.living_entity.entity;
            let holder = entity.leashed_to.lock().await.clone();
            let trader = holder
                .as_ref()
                .and_then(|h| h.cast_any().downcast_ref::<WanderingTraderEntity>());

            let new_delay = trader.map_or_else(
                || self.despawn_delay.load(Ordering::Relaxed) - 1,
                |trader| trader.despawn_delay.load(Ordering::Relaxed) - 1,
            );
            self.despawn_delay.store(new_delay, Ordering::Relaxed);

            if new_delay <= 0 {
                entity.unleash().await;
                let world = entity.world.load();
                world.remove_entity(self).await;
            }
        })
    }
}
