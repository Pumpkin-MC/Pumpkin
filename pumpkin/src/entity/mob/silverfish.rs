use std::sync::Arc;

use crate::entity::attributes::AttributeBuilder;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::active_target::ActiveTargetGoal,
    mob::{Mob, MobEntity},
};

pub struct SilverfishEntity {
    entity: Arc<MobEntity>,
}

impl SilverfishEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);

        {
            let mut target_selector = mob_arc.entity.target_selector.lock().await;

            // TODO
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn create_attributes() -> AttributeBuilder {
        AttributeBuilder::new()
            .add(Attributes::MAX_HEALTH, 8.0)
            .add(Attributes::MOVEMENT_SPEED, 0.25)
            .add(Attributes::ATTACK_DAMAGE, 1.0)
    }
}

impl NBTStorage for SilverfishEntity {}

impl Mob for SilverfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}
