use std::sync::{Arc, Weak};

use tokio::sync::Mutex;
use pumpkin_data::entity::EntityType;
use crate::entity::{
    Entity,
    ai::{
        goal::{look_at_entity::LookAtEntityGoal, target_goal::TargetGoal},
        path::Navigator,
    },
    living::LivingEntity,
};
use crate::entity::ai::goal::Goal;
use super::MobEntity;

pub struct Zombie;

impl Zombie {
    pub fn make(entity: Entity) -> Arc<MobEntity> {
        let mob_arc = Arc::new_cyclic(|mob_weak: &Weak<MobEntity>| {
            let goals: Mutex<Vec<(Arc<dyn Goal>, bool)>> = Mutex::new(vec![
                (Arc::new(LookAtEntityGoal::with_default(mob_weak.clone(), EntityType::PLAYER, 8.0)), false),
                (Arc::new(TargetGoal::new(16.0)), false),
            ]);
            MobEntity {
                living_entity: LivingEntity::new(entity),
                goals,
                navigator: Mutex::new(Navigator::default()),
                target: Mutex::new(None),
            }
        });
        mob_arc
    }
}
