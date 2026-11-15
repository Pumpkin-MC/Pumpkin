use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::{LlamaEntity, as_llama};

const CARAVAN_LIMIT: i32 = 8;

pub struct LlamaFollowCaravanGoal {
    default_speed_modifier: f64,
    speed_modifier: f64,
    dist_check_counter: i32,
}

impl LlamaFollowCaravanGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            default_speed_modifier: speed_modifier,
            speed_modifier,
            dist_check_counter: 0,
        }
    }

    /// Ties go to the later candidate.
    fn closest(
        llamas: &[Arc<dyn EntityBase>],
        pos: Vector3<f64>,
        accept: impl Fn(&LlamaEntity, &dyn EntityBase) -> bool,
    ) -> Option<(&dyn EntityBase, f64)> {
        let mut closest = None;
        let mut closest_dist_sq = f64::MAX;
        for candidate in llamas {
            let Some(llama) = as_llama(candidate.as_ref()) else {
                continue;
            };
            if accept(llama, candidate.as_ref()) && !llama.has_caravan_tail() {
                let dist_sq = pos.squared_distance_to_vec(&candidate.get_entity().pos.load());
                if dist_sq <= closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest = Some(candidate.as_ref());
                }
            }
        }
        closest.map(|closest| (closest, closest_dist_sq))
    }

    fn first_is_leashed(llama: &LlamaEntity, counter: i32) -> bool {
        if counter > CARAVAN_LIMIT || !llama.in_caravan() {
            return false;
        }
        let Some(head) = llama.get_caravan_head() else {
            return false;
        };
        head.get_entity().is_leashed()
            || as_llama(head.as_ref()).is_some_and(|head| Self::first_is_leashed(head, counter + 1))
    }
}

impl Goal for LlamaFollowCaravanGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(llama) = as_llama(mob) else {
            return false;
        };
        let entity = mob.get_entity();
        if entity.is_leashed() || llama.in_caravan() {
            return false;
        }

        let pos = entity.pos.load();
        let area = entity.bounding_box.load().expand(9.0, 4.0, 9.0);
        let llamas: Vec<_> = entity
            .world
            .load()
            .get_entities_at_box(&area)
            .into_iter()
            .filter(|other| {
                let other = other.get_entity();
                other.entity_id != entity.entity_id
                    && (other.entity_type == &EntityType::LLAMA
                        || other.entity_type == &EntityType::TRADER_LLAMA)
            })
            .collect();

        let (closest, closest_dist_sq) = Self::closest(&llamas, pos, |llama, _| llama.in_caravan())
            .or_else(|| {
                Self::closest(&llamas, pos, |_, candidate| {
                    candidate.get_entity().is_leashed()
                })
            })
            .unzip();
        let closest_dist_sq = closest_dist_sq.unwrap_or(f64::MAX);
        let Some(closest) = closest else {
            return false;
        };
        let Some(closest_llama) = as_llama(closest) else {
            return false;
        };
        if closest_dist_sq < 4.0
            || (!closest.get_entity().is_leashed() && !Self::first_is_leashed(closest_llama, 1))
        {
            return false;
        }
        llama.join_caravan(closest_llama);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(llama) = as_llama(mob) else {
            return false;
        };
        let Some(head) = llama.get_caravan_head() else {
            return false;
        };
        if !head.get_entity().is_alive() || !Self::first_is_leashed(llama, 0) {
            return false;
        }

        let dist_sq = mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&head.get_entity().pos.load());
        if dist_sq > 676.0 {
            if self.speed_modifier <= 3.0 {
                self.speed_modifier *= 1.2;
                self.dist_check_counter = to_goal_ticks(40);
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
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(llama) = as_llama(mob) {
            llama.leave_caravan();
        }
        self.speed_modifier = self.default_speed_modifier;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(llama) = as_llama(mob) else {
            return;
        };
        let entity = mob.get_entity();
        let leashed_to_knot = entity
            .get_leash_holder()
            .is_some_and(|holder| holder.get_entity().entity_type == &EntityType::LEASH_KNOT);
        if leashed_to_knot {
            return;
        }
        let Some(head) = llama.get_caravan_head() else {
            return;
        };

        let pos = entity.pos.load();
        let head_pos = head.get_entity().pos.load();
        let distance = pos.squared_distance_to_vec(&head_pos).sqrt();
        let delta = (head_pos - pos).normalize() * (distance - 2.0).max(0.0);
        let target = pos + delta;
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.set_progress(NavigatorGoal::new(pos, target, self.speed_modifier));
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
