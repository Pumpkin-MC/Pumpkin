use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct VexEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `Vex.owner` (`OwnableEntity`). Set by `EvokerSummonSpellGoal`.
    owner_id: AtomicCell<Option<i32>>,
    /// Vanilla: `Vex.boundOrigin`, the point `VexRandomMoveGoal` wanders around.
    bound_origin: AtomicCell<Option<BlockPos>>,
    /// Vanilla: `Vex.hasLimitedLife` / `limitedLifeTicks`.
    has_limited_life: AtomicBool,
    limited_life_ticks: AtomicI32,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vex = Self {
            mob_entity,
            owner_id: AtomicCell::new(None),
            bound_origin: AtomicCell::new(None),
            has_limited_life: AtomicBool::new(false),
            limited_life_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(vex);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
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

    /// Vanilla: `Vex#setOwner`.
    pub fn set_owner(&self, owner: &Entity) {
        self.owner_id.store(Some(owner.entity_id));
    }

    /// Vanilla: `Vex#setBoundOrigin`.
    pub fn set_bound_origin(&self, origin: BlockPos) {
        self.bound_origin.store(Some(origin));
    }

    /// Vanilla: `Vex#setLimitedLife`.
    pub fn set_limited_life(&self, life_ticks: i32) {
        self.has_limited_life.store(true, Relaxed);
        self.limited_life_ticks.store(life_ticks, Relaxed);
    }
}

impl NBTStorage for VexEntity {}

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `Vex#tick` -- while `hasLimitedLife`, deals 1 starvation damage every 20 ticks
    /// once the counter runs out, resetting it to keep ticking down.
    ///
    /// Scope reduction: the rest of vanilla's `Vex` AI (`VexMoveControl`'s no-physics flight,
    /// `VexRandomMoveGoal` wandering around `bound_origin`, `VexChargeAttackGoal`'s charge dash,
    /// and `VexCopyOwnerTargetGoal`) is not ported here -- Vex already falls back to the generic
    /// `WanderAroundGoal`/`MeleeAttackGoal` pair registered above, which is a pre-existing gap
    /// unrelated to evoker spellcasting.
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.has_limited_life.load(Relaxed) {
                let remaining = self.limited_life_ticks.fetch_sub(1, Relaxed) - 1;
                if remaining <= 0 {
                    self.limited_life_ticks.store(20, Relaxed);
                    caller
                        .damage(caller.as_ref(), 1.0, DamageType::STARVE)
                        .await;
                }
            }
        })
    }
}
