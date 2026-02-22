use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const MAX_XZ_DISTANCE: f64 = 8.0;
const HOVER_VERTICAL_DISTANCE: i32 = 7;
const AIR_VERTICAL_DISTANCE: i32 = 4;
const HOVER_MIN_HEIGHT: i32 = 1;
const HOVER_MAX_HEIGHT: i32 = 3;
const AIR_FLYING_HEIGHT: i32 = -2;
const MAX_XZ_RADIANS_DIFF: f64 = std::f64::consts::FRAC_PI_2;

pub struct BeeWanderGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    chance: i32,
}

impl BeeWanderGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            chance: to_goal_ticks(10),
        }
    }

    fn find_hover_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let yaw_rad = entity.yaw.load().to_radians();
        let mut rng = mob.get_random();
        let view_x = -(yaw_rad.sin() as f64);
        let view_z = yaw_rad.cos() as f64;
        let hover_height = rng.random_range(HOVER_MIN_HEIGHT..=HOVER_MAX_HEIGHT);

        // Match vanilla BeeWanderGoal sampling:
        // 1) HoverRandomPos-style directional hover
        // 2) AirAndWaterRandomPos-style fallback
        Self::sample_directional_target(
            &mut rng,
            pos,
            view_x,
            view_z,
            HOVER_VERTICAL_DISTANCE,
            hover_height,
        )
        .or_else(|| {
            Self::sample_directional_target(
                &mut rng,
                pos,
                view_x,
                view_z,
                AIR_VERTICAL_DISTANCE,
                AIR_FLYING_HEIGHT,
            )
        })
    }

    fn sample_directional_target(
        rng: &mut rand::rngs::ThreadRng,
        pos: Vector3<f64>,
        x_dir: f64,
        z_dir: f64,
        vertical_dist: i32,
        flying_height: i32,
    ) -> Option<Vector3<f64>> {
        let yaw_center = z_dir.atan2(x_dir) - std::f64::consts::FRAC_PI_2;
        let yaw = yaw_center + rng.random_range(-MAX_XZ_RADIANS_DIFF..=MAX_XZ_RADIANS_DIFF);
        let dist = rng.random_range(0.0..1.0).sqrt() * MAX_XZ_DISTANCE * std::f64::consts::SQRT_2;
        let xt = -dist * yaw.sin();
        let zt = dist * yaw.cos();

        if xt.abs() > MAX_XZ_DISTANCE || zt.abs() > MAX_XZ_DISTANCE {
            return None;
        }

        let yt = f64::from(rng.random_range(-vertical_dist..=vertical_dist) + flying_height);
        Some(Vector3::new(pos.x + xt, pos.y + yt, pos.z + zt))
    }
}

impl Goal for BeeWanderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_random().random_range(0..self.chance) != 0 {
                return false;
            }

            self.target = Self::find_hover_target(mob);
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
