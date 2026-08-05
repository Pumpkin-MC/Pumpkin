use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        defend_village_target::DefendVillageTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        nearest_hostile_target::NearestHostileTargetGoal, offer_flower::OfferFlowerGoal,
        revenge::RevengeGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use pumpkin_nbt::compound::NbtCompound;

/// Represents an Iron Golem, a powerful neutral mob that protects villagers and players.
///
/// Wiki: <https://minecraft.wiki/w/Iron_Golem>
pub struct IronGolemEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `IronGolem.DATA_PLAYER_CREATED_ID`/`isPlayerCreated` (`IronGolem.java:287-291`,
    /// persisted as `"PlayerCreated"` at line 147/154). Set by `CarvedPumpkinBlock` when a
    /// player assembles a golem out of iron blocks; village-spawned golems (e.g.
    /// `Villager::spawnGolemIfNeeded`) leave it `false`. Gates `DefendVillageTargetGoal` and
    /// `canAttack` (`IronGolem.java:136-141`): a player-created golem never attacks players,
    /// regardless of reputation.
    pub player_created: AtomicBool,
}

impl IronGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let iron_golem = Self {
            mob_entity,
            player_created: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(iron_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, OfferFlowerGoal::new());
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            // Vanilla `targetSelector.addGoal(1, new DefendVillageTargetGoal(this))`
            // (`IronGolem.java:75`): attack a player any nearby villager holds reputation
            // -100 or lower against. See `defend_village_target.rs` for full citation.
            target_selector.add_goal(1, DefendVillageTargetGoal::new());
            // Vanilla priority 2: `HurtByTargetGoal(this)`.
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            // Vanilla targets players through `NearestAttackableTargetGoal<>(..., this::isAngryAt)`,
            // so a golem only goes after a player it already holds a grudge against. We have no
            // per player anger state yet, so we leave players to `RevengeGoal` instead of
            // attacking every player on sight.
            //
            // Vanilla priority 3: `NearestAttackableTargetGoal<Mob>(this, Mob.class, 5, false,
            // false, (target, level) -> target instanceof Enemy && !(target instanceof Creeper))`
            // -- attacks the nearest hostile mob (excluding creepers) within follow range. This
            // has no village-proximity condition in vanilla; see `nearest_hostile_target.rs`.
            target_selector.add_goal(3, NearestHostileTargetGoal::new(&mob_arc.mob_entity));
        };

        mob_arc
    }
}

impl NBTStorage for IronGolemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            // `IronGolem.java:147`.
            nbt.put_bool("PlayerCreated", self.player_created.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            // `IronGolem.java:154`: `getBooleanOr("PlayerCreated", false)`.
            self.player_created.store(
                nbt.get_bool("PlayerCreated").unwrap_or(false),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for IronGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
