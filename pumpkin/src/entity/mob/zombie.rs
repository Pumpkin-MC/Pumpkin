use std::sync::Arc;

use super::MobEntity;
use crate::entity::{
    Entity,
    ai::{
        goal::{active_target_goal::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
        path::Navigator,
    },
    living::LivingEntity,
};
use pumpkin_data::entity::EntityType;
use tokio::sync::Mutex;

pub struct Zombie;

impl Zombie {
    pub async fn make(entity: Entity) -> Arc<MobEntity> {
        let mob_entity = MobEntity {
            living_entity: LivingEntity::new(entity),
            goals: Mutex::new(vec![]),
            navigator: Mutex::new(Navigator::default()),
            target: Mutex::new(None),
        };
        let mob_arc = Arc::new(mob_entity);
        let mob_weak = Arc::downgrade(&mob_arc);

        // This is needed for goals because some of them needs the MobEntity fully initialized in the constructor
        // The Weak is stored to avoid memory leak and can be used if and where necessary
        {
            let mut goals = mob_arc.goals.lock().await;
            goals.push((
                Arc::new(LookAtEntityGoal::with_default(
                    mob_weak.clone(),
                    EntityType::PLAYER,
                    8.0,
                )),
                false,
            ));
            goals.push((
                Arc::new(ActiveTargetGoal::with_default(
                    mob_weak.clone(),
                    EntityType::PLAYER,
                )),
                false,
            ));
        }

        mob_arc
    }
}
