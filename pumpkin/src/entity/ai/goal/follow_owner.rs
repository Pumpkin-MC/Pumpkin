use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

const TELEPORT_DISTANCE_SQ: f64 = 144.0;

pub struct FollowOwnerGoal {
    speed: f64,
    min_distance_sq: f64,
    max_distance_sq: f64,
    update_countdown: i32,
    owner: Option<Arc<Player>>,
}

impl FollowOwnerGoal {
    #[must_use]
    pub fn new(speed: f64, min_distance: f32, max_distance: f32) -> Box<Self> {
        Box::new(Self {
            speed,
            min_distance_sq: (min_distance * min_distance) as f64,
            max_distance_sq: (max_distance * max_distance) as f64,
            update_countdown: 0,
            owner: None,
        })
    }

    fn can_follow(mob: &dyn Mob) -> bool {
        !mob.is_sitting()
    }

    fn find_owner(mob: &dyn Mob) -> Option<Arc<Player>> {
        let owner_uuid = mob.get_owner_uuid()?;
        let world = mob.get_mob_entity().living_entity.entity.world.load_full();
        let player = world.get_player_by_uuid(owner_uuid)?;
        if player.is_spectator() {
            return None;
        }
        Some(player)
    }

    fn distance_to_owner_sq(mob: &dyn Mob, owner: &Player) -> f64 {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let owner_pos = owner.living_entity.entity.pos.load();
        let dx = mob_pos.x - owner_pos.x;
        let dy = mob_pos.y - owner_pos.y;
        let dz = mob_pos.z - owner_pos.z;
        dx * dx + dy * dy + dz * dz
    }

    async fn try_teleport_to_owner(mob: &dyn Mob, owner: &Player) {
        let owner_pos = owner.living_entity.entity.pos.load();
        let mob_entity = &mob.get_mob_entity().living_entity.entity;
        let world = mob_entity.world.load_full();

        let offsets: [(i32, i32); 10] = {
            let mut rng = mob.get_random();
            std::array::from_fn(|_| (rng.random_range(-3..=3), rng.random_range(-3..=3)))
        };

        for (dx, dz) in offsets {
            let target_x = owner_pos.x + dx as f64;
            let target_z = owner_pos.z + dz as f64;
            let target_y = owner_pos.y.floor();

            let block_pos = BlockPos(Vector3::new(
                target_x as i32,
                target_y as i32,
                target_z as i32,
            ));

            let block = world.get_block_state(&block_pos).await;
            if block.is_air() {
                let above = BlockPos(Vector3::new(
                    target_x as i32,
                    target_y as i32 + 1,
                    target_z as i32,
                ));
                let above_block = world.get_block_state(&above).await;
                if above_block.is_air() {
                    mob_entity
                        .teleport(
                            Vector3::new(target_x + 0.5, target_y, target_z + 0.5),
                            None,
                            None,
                            world.clone(),
                        )
                        .await;

                    let mut navigator = mob.get_mob_entity().navigator.lock().await;
                    navigator.stop();
                    return;
                }
            }
        }
    }
}

impl Goal for FollowOwnerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !Self::can_follow(mob) {
                return false;
            }

            let Some(owner) = Self::find_owner(mob) else {
                return false;
            };

            let dist_sq = Self::distance_to_owner_sq(mob, &owner);
            if dist_sq < self.min_distance_sq {
                return false;
            }

            self.owner = Some(owner);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !Self::can_follow(mob) {
                return false;
            }

            let Some(owner) = &self.owner else {
                return false;
            };

            if owner.is_spectator() || !owner.living_entity.entity.is_alive() {
                return false;
            }

            let dist_sq = Self::distance_to_owner_sq(mob, owner);
            if dist_sq <= self.max_distance_sq {
                return false;
            }

            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.update_countdown = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let Some(owner) = &self.owner else {
                return;
            };

            let dist_sq = Self::distance_to_owner_sq(mob, owner);
            let should_teleport = dist_sq >= TELEPORT_DISTANCE_SQ;

            if !should_teleport {
                let owner_pos = owner.living_entity.entity.pos.load();
                let mob_entity = mob.get_mob_entity();
                let mut look_control = mob_entity.look_control.lock().await;
                look_control.look_at_position(mob, owner_pos);
            }

            self.update_countdown -= 1;
            if self.update_countdown <= 0 {
                self.update_countdown = to_goal_ticks(10);

                if should_teleport {
                    Self::try_teleport_to_owner(mob, owner).await;
                } else {
                    let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
                    let owner_pos = owner.living_entity.entity.pos.load();
                    let mut navigator = mob.get_mob_entity().navigator.lock().await;
                    navigator.set_progress(NavigatorGoal {
                        current_progress: mob_pos,
                        destination: owner_pos,
                        speed: self.speed,
                    });
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.owner = None;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.stop();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
