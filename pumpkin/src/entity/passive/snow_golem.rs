use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        nearest_hostile_target::NearestHostileTargetGoal,
        ranged_snowball_attack::RangedSnowballAttackGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

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

            // `SnowGolem.java`'s `registerGoals`: priorities 1 (attack), 2 (wander), 3 (look at
            // player), 4 (random look around).
            goal_selector.add_goal(1, Box::new(RangedSnowballAttackGoal::new(20, 10.0)));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new_water_avoiding(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));

            // `SnowGolem.java`'s `targetSelector.addGoal(1, new NearestAttackableTargetGoal<>(
            // this, Mob.class, 10, true, false, (target, level) -> target instanceof Enemy))`
            // -- any hostile mob (including creepers), not just zombies.
            target_selector.add_goal(
                1,
                NearestHostileTargetGoal::new_for_snow_golem(&mob_arc.mob_entity),
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
