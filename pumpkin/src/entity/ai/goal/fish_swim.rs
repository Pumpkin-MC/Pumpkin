use std::sync::atomic::Ordering::SeqCst;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::{
    sound::Sound,
    tag::{self, Taggable},
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3, wrap_degrees};
use rand::RngExt;

const RETARGET_TICKS: i32 = 40;
const RETARGET_TICKS_ON_FAILURE: i32 = 10;
const TARGET_REACHED_DISTANCE_SQ: f64 = 1.0;
const WATER_SWIM_SPEED: f64 = 0.1;
const WATER_VELOCITY_BLEND: f64 = 0.125;
const WATER_UPWARD_DRIFT: f64 = 0.005;
const HORIZONTAL_RANGE: f64 = 8.0;
const VERTICAL_RANGE: i32 = 4;
const FLOP_CHANCE: f32 = 0.1;
const FLOP_XZ_PUSH: f64 = 0.05;
const FLOP_Y_PUSH: f64 = 0.4;

pub struct FishSwimGoal {
    goal_control: Controls,
    flop_sound: Sound,
    target: Option<Vector3<f64>>,
    retarget_cooldown: i32,
}

impl FishSwimGoal {
    #[must_use]
    pub const fn new(flop_sound: Sound) -> Self {
        Self {
            goal_control: Controls::MOVE,
            flop_sound,
            target: None,
            retarget_cooldown: 0,
        }
    }

    fn should_retarget(&self, position: Vector3<f64>) -> bool {
        self.target.is_none_or(|target| {
            position.squared_distance_to_vec(&target) <= TARGET_REACHED_DISTANCE_SQ
        })
    }

    async fn find_swim_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let position = entity.pos.load();
        let world = entity.world.load();

        for _ in 0..10 {
            let (dx, dy, dz) = {
                let mut rng = mob.get_random();
                (
                    rng.random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE),
                    rng.random_range(-VERTICAL_RANGE..=VERTICAL_RANGE),
                    rng.random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE),
                )
            };

            let candidate = BlockPos::new(
                (position.x + dx).floor() as i32,
                (position.y + dy as f64).floor() as i32,
                (position.z + dz).floor() as i32,
            );
            let fluid = world.get_fluid(&candidate).await;
            if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                continue;
            }

            let state = world.get_block_state(&candidate).await;
            if state.is_solid() {
                continue;
            }

            let above = BlockPos::new(candidate.0.x, candidate.0.y + 1, candidate.0.z);
            if world.get_block_state(&above).await.is_solid() {
                continue;
            }

            return Some(Vector3::new(
                f64::from(candidate.0.x) + 0.5,
                f64::from(candidate.0.y) + 0.5,
                f64::from(candidate.0.z) + 0.5,
            ));
        }

        None
    }

    fn steer_towards_target(mob: &dyn Mob, target: Vector3<f64>) {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        let position = entity.pos.load();
        let distance_sq = position.squared_distance_to_vec(&target);
        if distance_sq <= f64::EPSILON {
            return;
        }

        let distance = distance_sq.sqrt();
        let desired_x = (target.x - position.x) / distance * WATER_SWIM_SPEED;
        let desired_y = (target.y - position.y) / distance * WATER_SWIM_SPEED;
        let desired_z = (target.z - position.z) / distance * WATER_SWIM_SPEED;

        let mut velocity = entity.velocity.load();
        velocity.x += (desired_x - velocity.x) * WATER_VELOCITY_BLEND;
        velocity.y += (desired_y - velocity.y) * WATER_VELOCITY_BLEND + WATER_UPWARD_DRIFT;
        velocity.z += (desired_z - velocity.z) * WATER_VELOCITY_BLEND;
        entity.velocity.store(velocity);

        if desired_x != 0.0 || desired_z != 0.0 {
            let yaw = wrap_degrees((desired_z.atan2(desired_x) as f32).to_degrees() - 90.0);
            entity.yaw.store(yaw);
            entity.head_yaw.store(yaw);
            entity.body_yaw.store(yaw);
        }

        living.movement_input.store(Vector3::default());
    }

    async fn flop_on_land(&self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;

        if !entity.on_ground.load(SeqCst) || mob.get_random().random::<f32>() >= FLOP_CHANCE {
            return;
        }

        let (push_x, push_z) = {
            let mut rng = mob.get_random();
            (
                rng.random_range(-FLOP_XZ_PUSH..=FLOP_XZ_PUSH),
                rng.random_range(-FLOP_XZ_PUSH..=FLOP_XZ_PUSH),
            )
        };

        let mut velocity = entity.velocity.load();
        velocity.x += push_x;
        velocity.y = FLOP_Y_PUSH;
        velocity.z += push_z;
        entity.velocity.store(velocity);
        entity.on_ground.store(false, SeqCst);
        living.movement_input.store(Vector3::default());
        entity.play_sound(self.flop_sound).await;
    }
}

impl Goal for FishSwimGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            if entity.touching_water.load(SeqCst) {
                let position = entity.pos.load();
                self.retarget_cooldown -= 1;
                if self.retarget_cooldown <= 0 || self.should_retarget(position) {
                    self.target = Self::find_swim_target(mob).await;
                    self.retarget_cooldown = if self.target.is_some() {
                        RETARGET_TICKS
                    } else {
                        RETARGET_TICKS_ON_FAILURE
                    };
                }

                if let Some(target) = self.target {
                    Self::steer_towards_target(mob, target);
                }
            } else {
                self.target = None;
                self.retarget_cooldown = 0;
                self.flop_on_land(mob).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
