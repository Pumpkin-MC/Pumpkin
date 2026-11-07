use super::{Controls, Goal, active_target::ActiveTargetGoal};
use crate::entity::ageable::AgeableMob;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::world::World;
use pumpkin_data::entity::EntityType;

const CUB_SEARCH_HORIZONTAL: f64 = 8.0;
const CUB_SEARCH_VERTICAL: f64 = 4.0;

pub struct PolarBearAttackPlayersGoal {
    active_target_goal: Box<ActiveTargetGoal>,
}

impl PolarBearAttackPlayersGoal {
    #[must_use]
    pub fn new(mob: &MobEntity) -> Box<Self> {
        Box::new(Self {
            active_target_goal: Box::new(ActiveTargetGoal::new(
                mob,
                &EntityType::PLAYER,
                20,
                true,
                true,
                None::<fn(&LivingEntity, &World) -> bool>,
            ))
            .scale_range(0.5),
        })
    }
}

impl Goal for PolarBearAttackPlayersGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.as_ageable().is_some_and(AgeableMob::is_baby)
            || !self.active_target_goal.can_start(mob)
        {
            return false;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        let area = entity.bounding_box.load().expand(
            CUB_SEARCH_HORIZONTAL,
            CUB_SEARCH_VERTICAL,
            CUB_SEARCH_HORIZONTAL,
        );
        entity
            .world
            .load()
            .get_entities_at_box(&area)
            .iter()
            .any(|other| {
                other.get_entity().entity_type == &EntityType::POLAR_BEAR
                    && other
                        .get_mob()
                        .and_then(|mob| mob.as_ageable())
                        .is_some_and(AgeableMob::is_baby)
            })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.active_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.active_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.active_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.active_target_goal.controls()
    }
}
