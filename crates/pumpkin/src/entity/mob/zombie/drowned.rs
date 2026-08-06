use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::drowned_attack::DrownedAttackGoal;
use crate::entity::ai::goal::drowned_go_to_beach::DrownedGoToBeachGoal;
use crate::entity::ai::goal::drowned_go_to_water::DrownedGoToWaterGoal;
use crate::entity::ai::goal::drowned_swim_up::DrownedSwimUpGoal;
use crate::entity::ai::goal::drowned_util::is_bright_outside;
use crate::entity::ai::goal::ranged_trident_attack::DrownedTridentAttackGoal;
use crate::entity::living::LivingEntity;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, destroy_egg::DestroyEggGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

/// `Drowned#okTarget` (`Drowned.java:223-225`): a potential player target is only valid while
/// it's not bright outside, or while the target itself is in water.
async fn ok_target(target: Arc<LivingEntity>, world: Arc<World>) -> bool {
    !is_bright_outside(&world) || target.entity.touching_water.load(Relaxed)
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Deliberately not `ZombieEntityBase::new(entity)`: that constructor registers the
        // plain-Zombie goal set (SpearUseGoal, ZombieAttackGoal, the generic ActiveTargetGoal
        // quartet, etc.) into the same shared goal/target selectors this struct holds, and
        // `Drowned#addBehaviourGoals` (`Drowned.java:91-104`) replaces essentially all of that
        // with water-aware goals below. Building `MobEntity` directly and registering only
        // `Drowned`'s own goal set avoids double-registering (this is the fix the file's old
        // "Fix duplicated" TODO was pointing at).
        let mob_entity = MobEntity::new(entity);
        let base = Arc::new(ZombieEntityBase { mob_entity });
        let drowned = Self { entity: base };
        let mob_arc = Arc::new(drowned);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // `Drowned(EntityType, Level)` (`Drowned.java:75-79`):
        // `this.setPathfindingMalus(PathType.WATER, 0.0F)`. `MobData::new_zombie` (this
        // codebase's `WalkNodeEvaluator` cost table) otherwise costs water at 8.0, the same as
        // any other zombie, so without this override a Drowned would treat wading into water
        // as expensive terrain instead of free movement.
        mob_arc
            .entity
            .mob_entity
            .navigator
            .lock()
            .unwrap()
            .set_pathfinding_malus(crate::entity::ai::pathfinder::node::PathType::Water, 0.0);

        {
            let mut goal_selector = mob_arc.entity.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().unwrap();

            // Vanilla `Zombie#registerGoals` (not overridden by `Drowned`) still contributes
            // these three goals on top of `Drowned#addBehaviourGoals` below. Note the base
            // `Zombie`/`Husk`/`ZombieVillager` goal set in `ZombieEntityBase` includes a
            // `SwimGoal` at priority 0 that has no vanilla counterpart here -- neither
            // `Zombie#registerGoals` nor `Drowned#addBehaviourGoals` ever add a `FloatGoal`,
            // matching real zombies sinking (and eventually converting) in deep water instead
            // of floating. `Drowned` gets its own buoyancy from `DrownedSwimUpGoal` below.
            goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            // `Drowned#addBehaviourGoals` (`Drowned.java:90-104`).
            goal_selector.add_goal(1, DrownedGoToWaterGoal::new(1.0));
            goal_selector.add_goal(2, DrownedTridentAttackGoal::new(40, 10.0));
            goal_selector.add_goal(2, DrownedAttackGoal::new(1.0, false));
            goal_selector.add_goal(5, DrownedGoToBeachGoal::new(1.0));
            goal_selector.add_goal(6, DrownedSwimUpGoal::new(1.0));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));

            // `HurtByTargetGoal(this, Drowned.class).setAlertOthers(ZombifiedPiglin.class)`:
            // `RevengeGoal` here has no equivalent of vanilla's "ignore damage from own type" /
            // "also alert this other type" parameters, so it only alerts other `Drowned`
            // (matching `ZombieEntityBase`'s existing same-type-only alert simplification).
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.entity.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(ok_target),
                )),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::VILLAGER,
                    false,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::IRON_GOLEM,
                    true,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::AXOLOTL,
                    true,
                ),
            );
            // Vanilla restricts this to `Turtle.BABY_ON_LAND_SELECTOR` (baby turtles on land
            // only); no such predicate exists here, matching `ZombieEntityBase`'s existing
            // simplification of targeting any turtle.
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::TURTLE,
                    true,
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for DrownedEntity {}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}
