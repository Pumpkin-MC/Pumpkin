use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal, witch_attack::WitchAttackGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct WitchEntity {
    pub mob_entity: MobEntity,
}

impl WitchEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let witch = Self { mob_entity };
        let mob_arc = Arc::new(witch);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Witch.registerGoals
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // 2 RangedAttackGoal (splash potions) + WaterAvoidingRandomStroll
            goal_selector.add_goal(2, WitchAttackGoal::new(1.0));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));

            // 1 HurtByTarget(Raider ignore) — no setAlertOthers
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // 2 NearestHealableRaiderTargetGoal TODO (raid heal allies)
            // 3 NearestAttackableWitchTargetGoal(Player) — only players, not villager/golem
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for WitchEntity {}

impl Mob for WitchEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
