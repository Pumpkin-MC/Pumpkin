use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// A nautilus (`net/minecraft/world/entity/animal/nautilus/Nautilus.java`, behaviour in
/// `NautilusAi.java`, shared base in `AbstractNautilus.java`).
///
/// Vanilla drives this mob entirely from a `Brain`. Only the behaviours that map onto
/// Pumpkin's goal selector are wired here: `AnimalPanic(1.6F)` and the idle
/// `RandomStroll.swim(1.0F)` / `SetWalkTargetFromLookTarget` pair from `NautilusAi`'s CORE
/// and IDLE activities.
///
/// Not ported, none of which have an equivalent in Pumpkin today: taming, riding, the
/// saddle/body equipment slots and the 3-row mount inventory, dashing and its cooldown,
/// `AnimalMakeLove` breeding, `FollowTemptation`, the `ChargeAttack` FIGHT activity and
/// the `ANGRY_AT` / `ATTACK_TARGET_COOLDOWN` anger memories, the 300 tick air supply and
/// its `dryOut` damage on land, the baby dimensions, and every sound override.
///
/// `createAttributes` (`MAX_HEALTH` 15, `MOVEMENT_SPEED` 1.0, `ATTACK_DAMAGE` 3,
/// `KNOCKBACK_RESISTANCE` 0.3) already comes from the generated entity attribute table.
pub struct NautilusEntity {
    pub mob_entity: MobEntity,
}

impl NautilusEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let nautilus = Self { mob_entity };
        let mob_arc = Arc::new(nautilus);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // `NautilusAi`/`AbstractNautilus` has no float/swim behaviour: nautiluses are
            // always in water.
            // `NautilusAi.initCoreActivity`: `new AnimalPanic(1.6F)`.
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.6));
            // `NautilusAi.initIdleActivity`: `RandomStroll.swim(1.0F)`.
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for NautilusEntity {}

impl Mob for NautilusEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
