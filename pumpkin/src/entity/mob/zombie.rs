use crate::entity::ai::goal::{look_at_entity::LookAtEntityGoal, target_goal::TargetGoal};

use super::MobEntity;

pub struct Zombie;

impl Zombie {
    pub async fn make(mob: &MobEntity) {
        mob.goal(LookAtEntityGoal::new(8.0)).await;
        mob.goal(TargetGoal::new(16.0)).await;
    }
}
