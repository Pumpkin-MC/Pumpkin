use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, player::Player};
use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;

const TEMPT_RANGE: f64 = 10.0;
const STOP_DISTANCE: f64 = 2.5;
const SCARE_RANGE_SQUARED: f64 = 36.0;
const SCARE_MOVE_THRESHOLD_SQUARED: f64 = 0.01;
const SCARE_ROT_THRESHOLD: f32 = 5.0;

pub struct TemptGoal {
    goal_control: Controls,
    speed: f64,
    tempt_items: &'static [&'static Item],
    can_scare: bool,
    target_player: Option<Arc<Player>>,
    cooldown: i32,
    prev_pos: Vector3<f64>,
    prev_yaw: f32,
    prev_pitch: f32,
}

/// Vanilla `TemptGoal#canContinueToUse`'s scare check, factored out as a pure
/// function of the tracked previous player state and its current state.
/// Returns `false` (goal should abort) if the mob is within the scare range
/// and the player has moved or rotated more than the jitter threshold since
/// the last recorded checkpoint.
fn passes_scare_check(
    mob_to_player_dist_sq: f64,
    player_pos: Vector3<f64>,
    prev_pos: Vector3<f64>,
    player_yaw: f32,
    player_pitch: f32,
    prev_yaw: f32,
    prev_pitch: f32,
) -> bool {
    if mob_to_player_dist_sq >= SCARE_RANGE_SQUARED {
        return true;
    }
    if player_pos.squared_distance_to_vec(&prev_pos) > SCARE_MOVE_THRESHOLD_SQUARED {
        return false;
    }
    if (player_pitch - prev_pitch).abs() > SCARE_ROT_THRESHOLD
        || (player_yaw - prev_yaw).abs() > SCARE_ROT_THRESHOLD
    {
        return false;
    }
    true
}

impl TemptGoal {
    #[must_use]
    pub fn new(speed: f64, tempt_items: &'static [&'static Item], can_scare: bool) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            tempt_items,
            can_scare,
            target_player: None,
            cooldown: 0,
            prev_pos: Vector3::new(0.0, 0.0, 0.0),
            prev_yaw: 0.0,
            prev_pitch: 0.0,
        }
    }

    fn is_tempt_item(&self, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
        stack.item_count > 0 && self.tempt_items.iter().any(|i| i.id == stack.item.id)
    }

    async fn is_holding_tempt_item(&self, player: &Player) -> bool {
        let main = player.inventory.held_item();
        if self.is_tempt_item(&*main.lock().await) {
            return true;
        }
        let off = player.inventory.off_hand_item().await;
        self.is_tempt_item(&*off.lock().await)
    }

    async fn find_tempting_player(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let world = mob_entity.living_entity.entity.world.load();

        // Vanilla TemptGoal#canUse: `getNearestPlayer` selects the closest qualifying player,
        // not an arbitrary one in range.
        let mut nearest: Option<(Arc<Player>, f64)> = None;
        for player in world.get_nearby_players(pos, TEMPT_RANGE) {
            if !self.is_holding_tempt_item(&player).await {
                continue;
            }
            let dist = pos.squared_distance_to_vec(&player.get_entity().pos.load());
            if nearest.as_ref().is_none_or(|(_, best)| dist < *best) {
                nearest = Some((player, dist));
            }
        }
        nearest.map(|(player, _)| player)
    }

    async fn is_player_still_tempting(&self, player: &Player, mob: &dyn Mob) -> bool {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let player_pos = player.get_entity().pos.load();
        if mob_pos.squared_distance_to_vec(&player_pos) > TEMPT_RANGE * TEMPT_RANGE {
            return false;
        }
        self.is_holding_tempt_item(player).await
    }
}

impl Goal for TemptGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            self.target_player = self.find_tempting_player(mob).await;
            if let Some(player) = &self.target_player {
                self.prev_pos = player.get_entity().pos.load();
                true
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(player) = self.target_player.clone() else {
                return false;
            };

            if self.can_scare {
                let mob_entity = mob.get_mob_entity();
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let player_pos = player.get_entity().pos.load();
                let player_entity = player.get_entity();
                let player_yaw = player_entity.yaw.load();
                let player_pitch = player_entity.pitch.load();

                let dist_sq = mob_pos.squared_distance_to_vec(&player_pos);
                let ok = passes_scare_check(
                    dist_sq,
                    player_pos,
                    self.prev_pos,
                    player_yaw,
                    player_pitch,
                    self.prev_yaw,
                    self.prev_pitch,
                );

                if dist_sq >= SCARE_RANGE_SQUARED {
                    self.prev_pos = player_pos;
                }
                self.prev_yaw = player_yaw;
                self.prev_pitch = player_pitch;

                if !ok {
                    return false;
                }
            }

            self.is_player_still_tempting(&player, mob).await
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = &self.target_player {
                let mob_entity = mob.get_mob_entity();
                let player_pos = player.get_entity().pos.load();

                mob_entity.look_control.lock().unwrap().look_at(
                    mob,
                    player_pos.x,
                    player.get_entity().get_eye_y(),
                    player_pos.z,
                );

                let mob_pos = mob_entity.living_entity.entity.pos.load();
                if mob_pos.squared_distance_to_vec(&player_pos) > STOP_DISTANCE * STOP_DISTANCE {
                    let mut navigator = mob_entity.navigator.lock().unwrap();
                    navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target_player = None;
            self.cooldown = 100;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: f64, y: f64, z: f64) -> Vector3<f64> {
        Vector3::new(x, y, z)
    }

    #[test]
    fn far_away_always_passes() {
        // Outside the 6-block scare range, jitter never matters.
        assert!(passes_scare_check(
            100.0,
            pos(50.0, 0.0, 0.0),
            pos(0.0, 0.0, 0.0),
            180.0,
            180.0,
            0.0,
            0.0,
        ));
    }

    #[test]
    fn near_no_movement_passes() {
        assert!(passes_scare_check(
            4.0,
            pos(1.0, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            10.0,
            10.0,
            10.0,
            10.0,
        ));
    }

    #[test]
    fn near_small_jitter_passes() {
        // 0.05 blocks moved (sq = 0.0025), under the 0.01 sq threshold.
        assert!(passes_scare_check(
            4.0,
            pos(1.05, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            10.0,
            10.0,
            10.0,
            10.0,
        ));
    }

    #[test]
    fn near_movement_over_threshold_fails() {
        // 0.2 blocks moved (sq = 0.04), over the 0.01 sq threshold.
        assert!(!passes_scare_check(
            4.0,
            pos(1.2, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            10.0,
            10.0,
            10.0,
            10.0,
        ));
    }

    #[test]
    fn near_at_exact_range_boundary_is_far() {
        // dist_sq == 36.0 is treated as "far" (>=), matching vanilla's `< 36.0` gate.
        assert!(passes_scare_check(
            36.0,
            pos(500.0, 0.0, 0.0),
            pos(0.0, 0.0, 0.0),
            180.0,
            180.0,
            0.0,
            0.0,
        ));
    }

    #[test]
    fn near_yaw_rotation_over_threshold_fails() {
        assert!(!passes_scare_check(
            4.0,
            pos(1.0, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            10.0,
            20.0,
            10.0,
            10.0,
        ));
    }

    #[test]
    fn near_pitch_rotation_over_threshold_fails() {
        assert!(!passes_scare_check(
            4.0,
            pos(1.0, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            20.0,
            10.0,
            10.0,
            10.0,
        ));
    }

    #[test]
    fn near_rotation_at_exact_threshold_passes() {
        // abs diff == 5.0 is not > 5.0, so it should pass.
        assert!(passes_scare_check(
            4.0,
            pos(1.0, 0.0, 0.0),
            pos(1.0, 0.0, 0.0),
            15.0,
            15.0,
            10.0,
            10.0,
        ));
    }
}
