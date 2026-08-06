use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::llama_data_of;

/// `LlamaFollowCaravanGoal.java`.
///
/// Vanilla's `LeashFenceKnotEntity` (a decorative fence-post leash
/// anchor) does not exist in Pumpkin, so the `tick` method's `!(getLeashHolder() instanceof
/// LeashFenceKnotEntity)` guard is always true here -- a llama leashed to a fence post would still
/// try to path toward its caravan head, which cannot currently arise since fence leashing itself
/// isn't implemented, so this is a latent gap rather than an observed behavior change.
///
/// The neighbor scan uses a spherical radius (`World::get_nearby_entities`) instead of vanilla's
/// `AABB.inflate(9.0, 4.0, 9.0)` box, which slightly under-includes the box's horizontal corners;
/// documented approximation, not expected to matter for the caravan-forming heuristic.
pub struct LlamaFollowCaravanGoal {
    speed: f64,
    dist_check_counter: i32,
}

const CARAVAN_SEARCH_RADIUS: f64 = 9.0;
const MIN_JOIN_DISTANCE_SQUARED: f64 = 4.0;
const MAX_FOLLOW_DISTANCE_SQUARED: f64 = 676.0;
const MAX_CARAVAN_CHAIN_DEPTH: i32 = 8;

impl LlamaFollowCaravanGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            dist_check_counter: 0,
        })
    }

    /// `LlamaFollowCaravanGoal.firstIsLeashed`: walks up the caravan chain from `start`, looking
    /// for a leashed llama within `MAX_CARAVAN_CHAIN_DEPTH` hops.
    async fn first_is_leashed(start: &dyn EntityBase, mut counter: i32) -> bool {
        let mut current_id = {
            let Some(data) = llama_data_of(start) else {
                return false;
            };
            data.caravan_head_id.load(Relaxed)
        };
        let world = start.get_entity().world.load();

        loop {
            if counter > MAX_CARAVAN_CHAIN_DEPTH || current_id == -1 {
                return false;
            }
            let Some(current) = world.get_entity_by_id(current_id) else {
                return false;
            };
            if Self::is_leashed(current.as_ref()).await {
                return true;
            }
            let Some(data) = llama_data_of(current.as_ref()) else {
                return false;
            };
            current_id = data.caravan_head_id.load(Relaxed);
            counter += 1;
        }
    }

    async fn is_leashed(entity: &dyn EntityBase) -> bool {
        entity.get_entity().leashed_to.lock().await.is_some()
    }
}

impl Goal for LlamaFollowCaravanGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let Some(data) = llama_data_of(mob as &dyn EntityBase) else {
                return false;
            };

            if Self::is_leashed(mob as &dyn EntityBase).await || data.in_caravan() {
                return false;
            }

            let pos = entity.pos.load();
            let world = entity.world.load();
            let candidates: Vec<_> = world
                .get_nearby_entities(pos, CARAVAN_SEARCH_RADIUS)
                .into_values()
                .filter(|e| {
                    e.get_entity().entity_id != entity.entity_id
                        && (e.get_entity().entity_type == &EntityType::LLAMA
                            || e.get_entity().entity_type == &EntityType::TRADER_LLAMA)
                })
                .collect();

            let mut closest: Option<(f64, std::sync::Arc<dyn EntityBase>)> = None;
            for candidate in &candidates {
                let Some(cdata) = llama_data_of(candidate.as_ref()) else {
                    continue;
                };
                if cdata.in_caravan() && !cdata.has_caravan_tail() {
                    let dist = pos.squared_distance_to_vec(&candidate.get_entity().pos.load());
                    if closest.as_ref().is_none_or(|(best, _)| dist < *best) {
                        closest = Some((dist, candidate.clone()));
                    }
                }
            }

            if closest.is_none() {
                for candidate in &candidates {
                    let Some(cdata) = llama_data_of(candidate.as_ref()) else {
                        continue;
                    };
                    if !cdata.has_caravan_tail() && Self::is_leashed(candidate.as_ref()).await {
                        let dist = pos.squared_distance_to_vec(&candidate.get_entity().pos.load());
                        if closest.as_ref().is_none_or(|(best, _)| dist < *best) {
                            closest = Some((dist, candidate.clone()));
                        }
                    }
                }
            }

            let Some((dist_sq, closest)) = closest else {
                return false;
            };

            if dist_sq < MIN_JOIN_DISTANCE_SQUARED {
                return false;
            }

            let closest_leashed = Self::is_leashed(closest.as_ref()).await;
            if !closest_leashed && !Self::first_is_leashed(closest.as_ref(), 1).await {
                return false;
            }

            let Some(closest_data) = llama_data_of(closest.as_ref()) else {
                return false;
            };
            data.caravan_head_id
                .store(closest.get_entity().entity_id, Relaxed);
            closest_data
                .caravan_tail_id
                .store(entity.entity_id, Relaxed);

            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let Some(data) = llama_data_of(mob as &dyn EntityBase) else {
                return false;
            };
            let head_id = data.caravan_head_id.load(Relaxed);
            if head_id == -1 {
                return false;
            }
            let world = entity.world.load();
            let Some(head) = world.get_entity_by_id(head_id) else {
                return false;
            };
            if !head.get_entity().is_alive() {
                return false;
            }
            if !Self::first_is_leashed(mob as &dyn EntityBase, 0).await {
                return false;
            }

            let dist_sqr = entity
                .pos
                .load()
                .squared_distance_to_vec(&head.get_entity().pos.load());
            if dist_sqr > MAX_FOLLOW_DISTANCE_SQUARED {
                if self.speed <= 3.0 {
                    self.speed *= 1.2;
                    self.dist_check_counter = super::to_goal_ticks(40);
                    return true;
                }
                if self.dist_check_counter == 0 {
                    return false;
                }
            }

            if self.dist_check_counter > 0 {
                self.dist_check_counter -= 1;
            }

            true
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(data) = llama_data_of(mob as &dyn EntityBase) {
                let head_id = data.caravan_head_id.swap(-1, Relaxed);
                if head_id != -1 {
                    let world = mob.get_entity().world.load();
                    if let Some(head) = world.get_entity_by_id(head_id)
                        && let Some(head_data) = llama_data_of(head.as_ref())
                    {
                        head_data.caravan_tail_id.store(-1, Relaxed);
                    }
                }
            }
            self.speed = 2.1;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let Some(data) = llama_data_of(mob as &dyn EntityBase) else {
                return;
            };
            let head_id = data.caravan_head_id.load(Relaxed);
            if head_id == -1 {
                return;
            }
            let world = entity.world.load();
            let Some(head) = world.get_entity_by_id(head_id) else {
                return;
            };

            let self_pos = entity.pos.load();
            let head_pos = head.get_entity().pos.load();
            let distance_to = self_pos.squared_distance_to_vec(&head_pos).sqrt();
            let wanted_distance = 2.0;
            let delta =
                (head_pos - self_pos).normalize() * (distance_to - wanted_distance).max(0.0);
            let destination = self_pos + delta;

            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal::new(self_pos, destination, self.speed));
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
