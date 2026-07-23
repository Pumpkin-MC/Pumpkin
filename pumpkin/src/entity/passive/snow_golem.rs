use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::{
        goal::{
            active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
            look_at_entity::LookAtEntityGoal, wander_around::WanderAroundGoal,
        },
        vanilla_enemy::{SNOW_GOLEM_ENEMY_EXCLUDES, SNOW_GOLEM_TARGET_CHANCE},
    },
    mob::{Mob, MobEntity},
};

/// Snow golem — throws snowballs at hostiles (vanilla `Enemy`).
///
/// # Vanilla targeting
/// ```text
/// NearestAttackableTargetGoal(Mob.class, 10, true, false,
///   e -> e instanceof Enemy)   // includes creepers; no golem-style exclude
/// ```
///
/// Wiki: <https://minecraft.wiki/w/Snow_Golem>
pub struct SnowGolemEntity {
    pub mob_entity: MobEntity,
}

impl SnowGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let snow_golem = Self { mob_entity };
        let mob_arc = Arc::new(snow_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // TODO: SnowballAttackGoal (ranged) — vanilla primary combat goal
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            // Vanilla: all Enemy (MONSTER), including creeper — not iron-golem rules.
            target_selector.add_goal(
                1,
                ActiveTargetGoal::for_enemies(
                    &mob_arc.mob_entity,
                    SNOW_GOLEM_ENEMY_EXCLUDES,
                    SNOW_GOLEM_TARGET_CHANCE,
                    true, // checkVisibility = true
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SnowGolemEntity {}

impl Mob for SnowGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
