use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::LlamaEntity;

/// Llamas follow each other to form a caravan behind a leashed leader.
/// Mirrors vanilla Minecraft's `LlamaFollowCaravanGoal`.
pub struct LlamaFollowCaravanGoal {
    llama: Arc<LlamaEntity>,
    speed_modifier: f64,
    dist_check_counter: i32,
}

impl LlamaFollowCaravanGoal {
    #[must_use]
    pub fn new(llama: Arc<LlamaEntity>, speed_modifier: f64) -> Self {
        Self {
            llama,
            speed_modifier,
            dist_check_counter: 0,
        }
    }

    fn first_is_leashed(&self, current: &LlamaEntity, counter: i32) -> bool {
        if counter > 8 {
            false
        } else if let Some(head) = current.get_caravan_head() {
            if head.mob_entity.living_entity.entity.is_leashed() {
                true
            } else {
                self.first_is_leashed(&head, counter + 1)
            }
        } else {
            false
        }
    }
}

impl Goal for LlamaFollowCaravanGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        if self.llama.mob_entity.living_entity.entity.is_leashed() || self.llama.in_caravan() {
            return false;
        }

        let entity = &self.llama.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();

        let search_box =
            BoundingBox::new(pos, pos.add_raw(1.0, 1.0, 1.0)).expand(9.0, 4.0, 9.0);
        let entities = world.get_entities_at_box(&search_box);

        let mut closest_in_caravan: Option<Arc<LlamaEntity>> = None;
        let mut closest_dist_caravan = f64::MAX;

        let mut closest_leashed: Option<Arc<LlamaEntity>> = None;
        let mut closest_dist_leashed = f64::MAX;

        for e in entities {
            let ent = e.get_entity();
            if ent.entity_id == entity.entity_id {
                continue;
            }
            if ent.entity_type != &EntityType::LLAMA && ent.entity_type != &EntityType::TRADER_LLAMA {
                continue;
            }
            let Some(mob_ref) = e.get_mob() else {
                continue;
            };
            let Some(candidate) = mob_ref.as_llama() else {
                continue;
            };

            let dist_sq = pos.squared_distance_to_vec(&ent.pos.load());
            if candidate.in_caravan() && !candidate.has_caravan_tail() {
                if dist_sq < closest_dist_caravan {
                    closest_dist_caravan = dist_sq;
                    if let Some(candidate_arc) = candidate
                        .self_weak
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .as_ref()
                        .and_then(std::sync::Weak::upgrade)
                    {
                        closest_in_caravan = Some(candidate_arc);
                    }
                }
            } else if candidate.mob_entity.living_entity.entity.is_leashed() && !candidate.has_caravan_tail()
                && dist_sq < closest_dist_leashed
            {
                closest_dist_leashed = dist_sq;
                if let Some(candidate_arc) = candidate
                    .self_weak
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                    .and_then(std::sync::Weak::upgrade)
                {
                    closest_leashed = Some(candidate_arc);
                }
            }
        }

        let (chosen, chosen_dist) = if let Some(c) = closest_in_caravan {
            (Some(c), closest_dist_caravan)
        } else {
            (closest_leashed, closest_dist_leashed)
        };

        let Some(closest) = chosen else {
            return false;
        };

        if chosen_dist < 4.0 {
            return false;
        }

        if !closest.mob_entity.living_entity.entity.is_leashed() && !self.first_is_leashed(&closest, 1) {
            return false;
        }

        self.llama.join_caravan(&closest);
        true
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        if self.llama.in_caravan()
            && let Some(head) = self.llama.get_caravan_head()
            && head.get_entity().is_alive()
            && self.first_is_leashed(&self.llama, 0)
        {
            let dist_sq = self
                .llama
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&head.get_entity().pos.load());

            if dist_sq > 676.0 {
                if self.speed_modifier <= 3.0 {
                    self.speed_modifier *= 1.2;
                    self.dist_check_counter = 40;
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
        } else {
            false
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.llama.leave_caravan();
        self.speed_modifier = 2.1;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if self.llama.in_caravan()
            && let Some(follows) = self.llama.get_caravan_head()
        {
            let follows_pos = follows.get_entity().pos.load();
            let llama_pos = self.llama.get_entity().pos.load();
            let dx = follows_pos.x - llama_pos.x;
            let dy = follows_pos.y - llama_pos.y;
            let dz = follows_pos.z - llama_pos.z;
            let distance_to = (dx * dx + dy * dy + dz * dz).sqrt();
            if distance_to > 2.0 {
                let scale = (distance_to - 2.0).max(0.0) / distance_to;
                let dest = Vector3::new(
                    llama_pos.x + dx * scale,
                    llama_pos.y + dy * scale,
                    llama_pos.z + dz * scale,
                );
                let mut navigator = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                navigator.set_progress(NavigatorGoal {
                    current_progress: llama_pos,
                    destination: dest,
                    speed: self.speed_modifier,
                });
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
