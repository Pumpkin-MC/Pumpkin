//! Vanilla `LlamaFollowCaravanGoal` — llamas queue up behind a leashed leader.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::entity::EntityType;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::LlamaEntity;
use crate::entity::{Entity, EntityBase};

const NO_HEAD: i32 = -1;
/// Vanilla drops the caravan link beyond 26 blocks (676 squared).
const MAX_FOLLOW_DISTANCE_SQ: f64 = 676.0;
const SEARCH_RADIUS: f64 = 9.0;

pub struct FollowCaravanGoal {
    speed: f64,
}

impl FollowCaravanGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self { speed }
    }

    fn as_llama(mob: &dyn Mob) -> Option<&LlamaEntity> {
        mob.cast_any().downcast_ref::<LlamaEntity>()
    }

    fn resolve_llama(entity: &Arc<dyn EntityBase>) -> Option<&LlamaEntity> {
        entity.cast_any().downcast_ref::<LlamaEntity>()
    }

    async fn is_leashed(entity: &Entity) -> bool {
        entity.leashed_to.lock().await.is_some()
    }

    /// Vanilla `firstIsLeashed`: the caravan chain must end in a leashed llama
    /// within 8 hops.
    async fn chain_ends_leashed(llama: &LlamaEntity, world: &crate::world::World) -> bool {
        let mut head_id = llama.caravan_head.load(Ordering::Relaxed);
        for _ in 0..8 {
            if head_id == NO_HEAD {
                return false;
            }
            let Some(head) = world.get_entity_by_id(head_id) else {
                return false;
            };
            if Self::is_leashed(head.get_entity()).await {
                return true;
            }
            let Some(head_llama) = Self::resolve_llama(&head) else {
                return false;
            };
            head_id = head_llama.caravan_head.load(Ordering::Relaxed);
        }
        false
    }

    async fn leave_caravan(llama: &LlamaEntity, world: &crate::world::World) {
        let head_id = llama.caravan_head.swap(NO_HEAD, Ordering::Relaxed);
        if head_id != NO_HEAD
            && let Some(head) = world.get_entity_by_id(head_id)
            && let Some(head_llama) = Self::resolve_llama(&head)
        {
            head_llama.caravan_tail.store(false, Ordering::Relaxed);
        }
    }
}

impl Goal for FollowCaravanGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(llama) = Self::as_llama(mob) else {
                return false;
            };
            let entity = mob.get_entity();
            if Self::is_leashed(entity).await
                || llama.caravan_head.load(Ordering::Relaxed) != NO_HEAD
            {
                return false;
            }

            let world = entity.world.load();
            let pos = entity.pos.load();
            let mut best: Option<(Arc<dyn EntityBase>, f64, bool)> = None;
            for (_, candidate) in world.get_nearby_entities(pos, SEARCH_RADIUS) {
                if candidate.get_entity().entity_id == entity.entity_id
                    || candidate.get_entity().entity_type != &EntityType::LLAMA
                {
                    continue;
                }
                let Some(candidate_llama) = Self::resolve_llama(&candidate) else {
                    continue;
                };
                if candidate_llama.caravan_tail.load(Ordering::Relaxed) {
                    continue;
                }
                let in_caravan = candidate_llama.caravan_head.load(Ordering::Relaxed) != NO_HEAD;
                let leashed = Self::is_leashed(candidate.get_entity()).await;
                if !in_caravan && !leashed {
                    continue;
                }
                let distance = candidate
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos);
                // Vanilla prefers llamas already in a caravan over merely
                // leashed ones, then takes the nearest.
                let better = match &best {
                    None => true,
                    Some((_, best_distance, best_in_caravan)) => {
                        (in_caravan && !best_in_caravan)
                            || (in_caravan == *best_in_caravan && distance < *best_distance)
                    }
                };
                if better {
                    best = Some((candidate, distance, in_caravan));
                }
            }

            let Some((head, _, _)) = best else {
                return false;
            };
            let Some(head_llama) = Self::resolve_llama(&head) else {
                return false;
            };
            // Join the caravan (vanilla joinCaravan).
            llama
                .caravan_head
                .store(head.get_entity().entity_id, Ordering::Relaxed);
            head_llama.caravan_tail.store(true, Ordering::Relaxed);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(llama) = Self::as_llama(mob) else {
                return false;
            };
            let entity = mob.get_entity();
            let world = entity.world.load();
            let head_id = llama.caravan_head.load(Ordering::Relaxed);
            if head_id == NO_HEAD {
                return false;
            }
            let Some(head) = world.get_entity_by_id(head_id) else {
                return false;
            };
            if !head.get_entity().is_alive() {
                return false;
            }
            let distance_sq = entity
                .pos
                .load()
                .squared_distance_to_vec(&head.get_entity().pos.load());
            distance_sq <= MAX_FOLLOW_DISTANCE_SQ && Self::chain_ends_leashed(llama, &world).await
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(llama) = Self::as_llama(mob) else {
                return;
            };
            let entity = mob.get_entity();
            let world = entity.world.load();
            Self::leave_caravan(llama, &world).await;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(llama) = Self::as_llama(mob) else {
                return;
            };
            let entity = mob.get_entity();
            let world = entity.world.load();
            let head_id = llama.caravan_head.load(Ordering::Relaxed);
            if head_id == NO_HEAD {
                return;
            }
            let Some(head) = world.get_entity_by_id(head_id) else {
                return;
            };
            let mob_pos = entity.pos.load();
            let head_pos = head.get_entity().pos.load();
            // Stay a body length behind the leader instead of pushing into it.
            if mob_pos.squared_distance_to_vec(&head_pos) > 9.0 {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(mob_pos, head_pos, self.speed));
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
