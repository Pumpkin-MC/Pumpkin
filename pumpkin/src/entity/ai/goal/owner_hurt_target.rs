use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

const FOLLOW_RANGE: f64 = 16.0;

pub struct OwnerHurtTargetGoal {
    target: Option<Arc<dyn EntityBase>>,
    last_attack_time: i32,
}

impl OwnerHurtTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            target: None,
            last_attack_time: 0,
        })
    }
}

impl Goal for OwnerHurtTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.is_sitting() {
                return false;
            }

            let Some(owner_uuid) = mob.get_owner_uuid() else {
                return false;
            };

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let Some(owner) = world.get_player_by_uuid(owner_uuid) else {
                return false;
            };

            let attack_time = owner.living_entity.last_attack_time.load(Relaxed);
            if attack_time == self.last_attack_time {
                return false;
            }

            let attacking_id = owner.living_entity.last_attacking_id.load(Relaxed);
            if attacking_id == 0 {
                return false;
            }

            let Some(target) = world.get_entity_by_id(attacking_id) else {
                return false;
            };

            if !target.get_entity().is_alive() {
                return false;
            }

            if !mob.can_attack_with_owner(target.as_ref(), &*owner) {
                return false;
            }

            self.target = Some(target);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(t) = target.as_ref() else {
                return false;
            };
            if !t.get_entity().is_alive() {
                return false;
            }
            let my_pos = mob.get_entity().pos.load();
            let target_pos = t.get_entity().pos.load();
            my_pos.squared_distance_to_vec(&target_pos) <= FOLLOW_RANGE * FOLLOW_RANGE
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            *mob_entity.target.lock().await = self.target.clone();

            if let Some(owner_uuid) = mob.get_owner_uuid() {
                let world = mob_entity.living_entity.entity.world.load_full();
                if let Some(owner) = world.get_player_by_uuid(owner_uuid) {
                    self.last_attack_time = owner.living_entity.last_attack_time.load(Relaxed);
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
            *mob.get_mob_entity().target.lock().await = None;
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}
