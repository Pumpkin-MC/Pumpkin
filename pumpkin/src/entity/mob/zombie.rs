use super::{Mob, MobEntity};
use crate::entity::ai::goal::look_around_goal::LookAroundGoal;
use crate::entity::{
    Entity,
    ai::goal::{active_target_goal::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use pumpkin_data::entity::EntityType;
use std::sync::{Arc, Weak};

pub struct Zombie {
    mob_entity: MobEntity,
}

impl Zombie {
    pub async fn make(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zombie = Self { mob_entity };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // This is needed for goals because some of them needs the MobEntity fully initialized in the constructor
        // The Weak is stored to avoid memory leak and can be used if and where necessary
        {
            let mut goals = mob_arc.mob_entity.goals.lock().await;
            goals.push((
                Arc::new(LookAtEntityGoal::with_default(
                    mob_weak,
                    EntityType::PLAYER,
                    8.0,
                )),
                false,
            ));
            goals.push((Arc::new(LookAroundGoal::default()), false));

            goals.push((
                Arc::new(ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    EntityType::PLAYER,
                )),
                false,
            ));
        };

        mob_arc
    }
}

impl Mob for Zombie {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
