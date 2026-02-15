use crate::entity::attributes::AttributeBuilder;
use pumpkin_data::{attributes::Attributes, entity::EntityType};
use std::sync::{Arc, Weak};

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal},
    mob::{Mob, MobEntity},
};

pub struct WitherEntity {
    pub mob_entity: MobEntity,
}

impl WitherEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wither = Self { mob_entity };
        let mob_arc = Arc::new(wither);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            // TODO
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn create_attributes() -> AttributeBuilder {
        AttributeBuilder::new()
            .add(Attributes::ARMOR, 4.0)
            .add(Attributes::FLYING_SPEED, 0.6)
            .add(Attributes::FOLLOW_RANGE, 40.0)
            .add(Attributes::MOVEMENT_SPEED, 0.6)
    }
}

impl NBTStorage for WitherEntity {}

impl Mob for WitherEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
