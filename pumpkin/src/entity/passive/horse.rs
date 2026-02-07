use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal, panic::PanicGoal,
        swim::SwimGoal, tempt, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Horse — a rideable passive mob.
///
/// Currently has basic AI: swims, panics, wanders, looks at players.
/// Taming, saddle, armor, and breeding are future additions.
pub struct HorseEntity {
    pub mob_entity: MobEntity,
}

impl HorseEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(1, PanicGoal::new(1.2));
            goal_selector.add_goal(3, tempt::TemptGoal::new(1.25, tempt::TEMPT_HORSE, 10.0));
            goal_selector.add_goal(6, WanderAroundGoal::new(0.7));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for HorseEntity {}

impl Mob for HorseEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
