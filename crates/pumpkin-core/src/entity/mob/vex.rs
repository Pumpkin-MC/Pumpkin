use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::raider;
use crate::entity::{
    Entity,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const VEX_HUNTS: &[Hunt] = &[Hunt::new(3, TargetMatch::Type(&EntityType::PLAYER), true)];

pub struct VexEntity {
    pub mob_entity: MobEntity,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vex = Self { mob_entity };
        let mob_arc = Arc::new(vex);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, raider::revenge_goal());
            // TODO: VexCopyOwnerTargetGoal at 2.
            hunting::apply(target_selector, &mob_arc.mob_entity, VEX_HUNTS);
        };

        mob_arc
    }
}

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
