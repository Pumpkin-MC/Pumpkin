use std::sync::Arc;

use tokio::sync::Mutex;

use crate::entity::{
    Entity,
    ai::{
        goal::{look_at_entity::LookAtEntityGoal, target_goal::TargetGoal},
        path::Navigator,
    },
    living::LivingEntity,
};

use super::MobEntity;

pub struct Zombie;

impl Zombie {
    pub fn make(entity: Entity) -> MobEntity {
        MobEntity {
            living_entity: LivingEntity::new(entity),
            goals: Mutex::new(vec![
                (Arc::new(LookAtEntityGoal::new(8.0)), false),
                (Arc::new(TargetGoal::new(16.0)), false),
            ]),
            navigator: Mutex::new(Navigator::default()),
        }
    }
}
