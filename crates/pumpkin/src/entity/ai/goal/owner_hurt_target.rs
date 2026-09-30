use super::track_target::TrackTargetGoal;
use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

pub struct OwnerHurtTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
    timestamp: i32,
}

impl OwnerHurtTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            track_target_goal: TrackTargetGoal::with_default(false),
            target: None,
            target_predicate: TargetPredicate::create_attackable(),
            timestamp: 0,
        })
    }
}

impl Goal for OwnerHurtTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !mob.is_tamed() || mob.is_sitting() {
            return false;
        }

        let Some(owner_uuid) = mob.get_owner_uuid() else {
            return false;
        };

        let world = mob.get_mob_entity().living_entity.entity.world.load_full();
        let Some(owner) = world.get_player_by_uuid(owner_uuid) else {
            return false;
        };

        let timestamp = owner.living_entity.last_attack_time.load(Relaxed);
        if timestamp == self.timestamp {
            return false;
        }

        let other_id = owner.living_entity.last_attacking_id.load(Relaxed);
        if other_id == 0 {
            return false;
        }

        let Some(other) = world.get_entity_by_id(other_id) else {
            return false;
        };

        if !self
            .track_target_goal
            .can_track(mob, Some(other.as_ref()), &self.target_predicate)
            || !mob.can_attack_with_owner(other.as_ref(), &*owner)
        {
            return false;
        }

        self.target = Some(other);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());

        if let Some(owner_uuid) = mob.get_owner_uuid() {
            let world = mob.get_mob_entity().living_entity.entity.world.load_full();
            if let Some(owner) = world.get_player_by_uuid(owner_uuid) {
                self.timestamp = owner.living_entity.last_attack_time.load(Relaxed);
            }
        }

        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
