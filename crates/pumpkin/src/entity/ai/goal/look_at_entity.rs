use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, player::Player};
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::sync::{Arc, Weak};

pub struct LookAtEntityGoal {
    goal_control: Controls,
    target: Option<Arc<dyn EntityBase>>,
    range: f32,
    look_time: i32,
    chance: f32,
    look_forward: bool,
    /// `None` means vanilla's `Mob.class` lookAtType: any mob (excluding players), used by
    /// e.g. `Vex.java:92`'s `LookAtPlayerGoal(this, Mob.class, 8.0F)`.
    target_type: Option<&'static EntityType>,
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
        let target_predicate = Self::create_target_predicate(mob_weak, Some(target_type), range);
        Self {
            goal_control: Controls::LOOK,
            target: None,
            range,
            look_time: 0,
            chance,
            look_forward,
            target_type: Some(target_type),
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

    /// Vanilla `LookAtPlayerGoal(this, Mob.class, lookDistance, probability, onlyHorizontal)`:
    /// looks at the nearest mob of any type (not just players).
    #[must_use]
    pub fn new_any_mob(
        mob_weak: Weak<dyn Mob>,
        range: f32,
        chance: f32,
        look_forward: bool,
    ) -> Self {
        let target_predicate = Self::create_target_predicate(mob_weak, None, range);
        Self {
            goal_control: Controls::LOOK,
            target: None,
            range,
            look_time: 0,
            chance,
            look_forward,
            target_type: None,
            target_predicate,
        }
    }

    /// Vanilla default probability (`LookAtPlayerGoal.DEFAULT_PROBABILITY`).
    #[must_use]
    pub fn with_default_any_mob(mob_weak: Weak<dyn Mob>, range: f32) -> Box<Self> {
        Box::new(Self::new_any_mob(mob_weak, range, 0.02, false))
    }

    fn create_target_predicate(
        mob_weak: Weak<dyn Mob>,
        target_type: Option<&'static EntityType>,
        range: f32,
    ) -> TargetPredicate {
        let mut target_predicate = TargetPredicate::create_non_attackable();
        target_predicate.base_max_distance = range as f64; // TODO
        if target_type == Some(&EntityType::PLAYER) {
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

            let world = mob_entity.living_entity.entity.world.load();
            let mut mob_pos = mob_entity.living_entity.entity.pos.load();
            mob_pos.y += mob_entity.living_entity.entity.get_eye_height();

            let mut candidates: Vec<Arc<dyn EntityBase>> = match self.target_type {
                Some(target_type) if *target_type == EntityType::PLAYER => world
                    .get_nearby_players(mob_pos, self.range.into())
                    .into_iter()
                    .map(|p: Arc<Player>| p as Arc<dyn EntityBase>)
                    .collect(),
                Some(target_type) => world
                    .get_entities_at_box(
                        &mob_entity.living_entity.entity.bounding_box.load().expand(
                            self.range.into(),
                            3.0,
                            self.range.into(),
                        ),
                    )
                    .into_iter()
                    .filter(|candidate| {
                        candidate.get_entity().entity_type == target_type
                            && candidate.get_entity().entity_id
                                != mob_entity.living_entity.entity.entity_id
                    })
                    .collect(),
                // Vanilla `Mob.class`: any mob, players excluded (Player is not a Mob subclass).
                // The self-exclusion must happen here, inside the search: `get_closest_entity_where`
                // returns a single nearest match, and this mob itself (at ~0 distance) would
                // otherwise always win over any other candidate, leaving `target_predicate.test`'s
                // later `ptr::eq` self-check to reject the only candidate found every time.
                None => {
                    let own_id = mob_entity.living_entity.entity.entity_id;
                    world
                        .get_entities_at_box(
                            &mob_entity.living_entity.entity.bounding_box.load().expand(
                                self.range.into(),
                                3.0,
                                self.range.into(),
                            ),
                        )
                        .into_iter()
                        .filter(|candidate| {
                            candidate.get_entity().entity_id != own_id
                                && candidate.get_mob().is_some()
                        })
                        .collect()
                }
            };

            candidates.sort_by(|a, b| {
                let a_distance = a.get_entity().pos.load().squared_distance_to_vec(&mob_pos);
                let b_distance = b.get_entity().pos.load().squared_distance_to_vec(&mob_pos);
                a_distance.total_cmp(&b_distance)
            });

            // Vanilla runs candidates through the goal's `TargetingConditions`, which rejects
            // entities that are not part of the game (spectators) or out of range. It does so
            // while selecting the nearest entity, so a rejected nearest candidate must not hide a
            // farther valid candidate.
            self.target = None;
            for candidate in candidates {
                if let Some(living) = candidate.get_living_entity()
                    && self
                        .target_predicate
                        .test(&world, Some(&mob_entity.living_entity), living)
                        .await
                {
                    self.target = Some(candidate);
                    break;
                }
            }

            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            if let Some(target) = &self.target {
                if !target.get_entity().is_alive() {
                    return false;
                }
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let target_pos = target.get_entity().pos.load();
                if mob_pos.squared_distance_to_vec(&target_pos) as f32 > (self.range * self.range) {
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
                let target_entity = target.get_entity();
                let target_pos = target_entity.pos.load();
                let look_y = if self.look_forward {
                    mob_entity.living_entity.entity.get_eye_y()
                } else {
                    target_entity.get_eye_y()
                };
                mob_entity.look_control.lock().unwrap().look_at(
                    mob,
                    target_pos.x,
                    look_y,
                    target_pos.z,
                );
                self.look_time -= 1;
            }
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
