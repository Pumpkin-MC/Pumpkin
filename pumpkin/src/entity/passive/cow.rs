use std::sync::{Arc, Weak};

use pumpkin_data::{entity::EntityType, item::Item};

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

pub struct CowEntity {
    pub mob_entity: MobEntity,
}

impl CowEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cow = Self { mob_entity };
        let mob_arc = Arc::new(cow);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        mob_arc.mob_entity.living_entity.movement_speed.store(0.2);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for CowEntity {}

impl Mob for CowEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
