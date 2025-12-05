use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, player::Player};
use pumpkin_data::entity::EntityType;
use rand::Rng;
use std::sync::{Arc, Weak};

#[allow(dead_code)]
pub struct LookAtEntityGoal {
    goal_control: Controls,
    target: Option<Arc<dyn EntityBase>>,
    range: f32,
    look_time: i32,
    chance: f32,
    look_forward: bool,
    target_type: &'static EntityType,
    target_predicate: TargetPredicate,
}

impl LookAtEntityGoal {
    #[must_use]
    pub fn new(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
        chance: f32,
        look_forward: bool,
    ) -> Self {
        let target_predicate = Self::create_target_predicate(mob_weak, target_type, range);
        Self {
            goal_control: Controls::LOOK,
            target: None,
            range,
            look_time: 0,
            chance,
            look_forward,
            target_type,
            target_predicate,
        }
    }

    #[must_use]
    pub fn with_default(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
    ) -> Box<Self> {
        Box::new(Self::new(mob_weak, target_type, range, 0.02, false))
    }

    fn create_target_predicate(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
    ) -> TargetPredicate {
        let mut target_predicate = TargetPredicate::non_attackable();
        target_predicate.base_max_distance = range;
        if target_type == &EntityType::PLAYER {
            target_predicate.set_predicate(move |living_entity, _world| {
                let mob_weak = mob_weak.clone();
                async move {
                    if let Some(mob_arc) = mob_weak.upgrade() {
                        let predicate = EntityPredicate::Rides(mob_arc.get_entity());
                        predicate.test(&living_entity.entity).await
                    } else {
                        // MobEntity is destroyed
                        false
                    }
                }
            });
        }
        target_predicate
    }
}

impl Goal for LookAtEntityGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if mob.get_random().random::<f32>() >= self.chance {
                return false;
            }

            let mob_entity = mob.get_mob_entity();

            {
                let mob_target = mob_entity.target.lock().await;
                if mob_target.is_some() {
                    self.target.clone_from(&mob_target);
                }
            }

            let world = &mob_entity.living_entity.entity.world;
            let mob_pos = mob_entity.living_entity.entity.pos.load();

            if *self.target_type == EntityType::PLAYER {
                self.target = world
                    .get_closest_player(mob_pos, self.range.into())
                    .await
                    .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
            } else {
                self.target = world
                    .get_closest_entity(mob_pos, self.range.into(), Some(&[self.target_type]))
                    .await;
            }

            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            if let Some(target) = &self.target {
                if !target.get_entity().is_alive() {
                    return false;
                }
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let target_pos = target.get_entity().pos.load();
                if mob_pos.squared_distance_to_vec(target_pos) as f32 > (self.range * self.range) {
                    return false;
                }
                return self.look_time > 0;
            }
            false
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.look_time = self.get_tick_count(40 + mob.get_random().random_range(0..40));
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            if let Some(target) = &self.target
                && target.get_entity().is_alive()
            {
                let target_pos = target.get_entity().pos.load();
                mob_entity.living_entity.entity.look_at(target_pos);
                self.look_time -= 1;
            }
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
