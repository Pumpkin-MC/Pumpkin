use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{
    ai::pathfinder::{NavigatorGoal, pathfinding_context::PathfindingContext},
    mob::Mob,
};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;

const MAX_XZ_DISTANCE: f64 = 8.0;
const HOVER_VERTICAL_DISTANCE: i32 = 7;
const AIR_VERTICAL_DISTANCE: i32 = 4;
const HOVER_MIN_HEIGHT: i32 = 1;
const HOVER_MAX_HEIGHT: i32 = 3;
const AIR_FLYING_HEIGHT: i32 = -2;
const MAX_XZ_RADIANS_DIFF: f64 = std::f64::consts::FRAC_PI_2;
const RANDOM_POS_ATTEMPTS: usize = 10;

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

    async fn find_hover_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let yaw_rad = entity.yaw.load().to_radians();
        let view_x = -(yaw_rad.sin() as f64);
        let view_z = yaw_rad.cos() as f64;

        if let Some(target) = Self::find_best_directional_target(
            mob,
            pos,
            view_x,
            view_z,
            HOVER_VERTICAL_DISTANCE,
            true,
        )
        .await
        {
            Some(target)
        } else {
            Self::find_best_directional_target(
                mob,
                pos,
                view_x,
                view_z,
                AIR_VERTICAL_DISTANCE,
                false,
            )
            .await
        }
    }

    async fn find_best_directional_target(
        mob: &dyn Mob,
        origin: Vector3<f64>,
        x_dir: f64,
        z_dir: f64,
        vertical_dist: i32,
        hover_mode: bool,
    ) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let mut context = PathfindingContext::new(BlockPos::floored_v(origin).0, world.clone());
        let mut best_pos: Option<BlockPos> = None;
        let mut best_weight = f32::MIN;

        for _ in 0..RANDOM_POS_ATTEMPTS {
            let Some(offset) = ({
                let mut rng = mob.get_random();
                let flying_height = if hover_mode {
                    rng.random_range(HOVER_MIN_HEIGHT..=HOVER_MAX_HEIGHT)
                } else {
                    AIR_FLYING_HEIGHT
                };

                Self::sample_direction_offset(
                    &mut rng,
                    x_dir,
                    z_dir,
                    MAX_XZ_DISTANCE,
                    vertical_dist,
                    flying_height,
                )
            }) else {
                continue;
            };

            let mut candidate = BlockPos::floored(
                origin.x + f64::from(offset.x),
                origin.y + f64::from(offset.y),
                origin.z + f64::from(offset.z),
            );

            candidate = if hover_mode {
                let above_solid = {
                    let mut rng = mob.get_random();
                    rng.random_range(HOVER_MIN_HEIGHT..=HOVER_MAX_HEIGHT)
                };
                let Some(hover_pos) =
                    Self::move_up_to_above_solid(world.as_ref(), candidate, above_solid).await
                else {
                    continue;
                };
                hover_pos
            } else {
                let Some(open_pos) = Self::move_up_out_of_solid(world.as_ref(), candidate).await
                else {
                    continue;
                };
                open_pos
            };

            if !Self::is_valid_target(world.as_ref(), &mut context, candidate, hover_mode).await {
                continue;
            }

            let weight = Self::target_weight(world.as_ref(), candidate).await;
            if weight > best_weight {
                best_weight = weight;
                best_pos = Some(candidate);
            }
        }

        best_pos.map(|p| p.to_f64())
    }

    fn sample_direction_offset(
        rng: &mut rand::rngs::ThreadRng,
        x_dir: f64,
        z_dir: f64,
        max_horizontal_dist: f64,
        vertical_dist: i32,
        flying_height: i32,
    ) -> Option<Vector3<i32>> {
        let yaw_center = z_dir.atan2(x_dir) - std::f64::consts::FRAC_PI_2;
        let yaw = yaw_center + rng.random_range(-MAX_XZ_RADIANS_DIFF..=MAX_XZ_RADIANS_DIFF);
        let dist = rng.random_range(0.0f64..1.0f64).sqrt()
            * max_horizontal_dist
            * std::f64::consts::SQRT_2;
        let xt = -dist * yaw.sin();
        let zt = dist * yaw.cos();

        if xt.abs() > max_horizontal_dist || zt.abs() > max_horizontal_dist {
            return None;
        }

        let yt = f64::from(rng.random_range(-vertical_dist..=vertical_dist) + flying_height);
        Some(Vector3::new(
            xt.floor() as i32,
            yt.floor() as i32,
            zt.floor() as i32,
        ))
    }

    async fn move_up_out_of_solid(
        world: &crate::world::World,
        mut pos: BlockPos,
    ) -> Option<BlockPos> {
        while world.is_in_height_limit(pos.0.y) && Self::is_solid(world, pos).await {
            pos = pos.up();
        }

        world.is_in_height_limit(pos.0.y).then_some(pos)
    }

    async fn move_up_to_above_solid(
        world: &crate::world::World,
        pos: BlockPos,
        above_solid_amount: i32,
    ) -> Option<BlockPos> {
        if above_solid_amount < 0 {
            return None;
        }

        if !Self::is_solid(world, pos).await {
            return world.is_in_height_limit(pos.0.y).then_some(pos);
        }

        let mut current = pos.up();
        while world.is_in_height_limit(current.0.y) && Self::is_solid(world, current).await {
            current = current.up();
        }
        if !world.is_in_height_limit(current.0.y) {
            return None;
        }

        let first_non_solid_y = current.0.y;
        while world.is_in_height_limit(current.0.y)
            && current.0.y - first_non_solid_y < above_solid_amount
        {
            let next = current.up();
            if !world.is_in_height_limit(next.0.y) || Self::is_solid(world, next).await {
                break;
            }
            current = next;
        }

        Some(current)
    }

    async fn is_solid(world: &crate::world::World, pos: BlockPos) -> bool {
        world.get_block_state(&pos).await.is_full_cube()
    }

    async fn is_water(world: &crate::world::World, pos: BlockPos) -> bool {
        world
            .get_fluid(&pos)
            .await
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
    }

    async fn is_valid_target(
        world: &crate::world::World,
        context: &mut PathfindingContext,
        pos: BlockPos,
        reject_water: bool,
    ) -> bool {
        if !world.is_in_build_limit(pos) {
            return false;
        }
        if reject_water && Self::is_water(world, pos).await {
            return false;
        }
        if Self::is_solid(world, pos).await {
            return false;
        }

        context.get_land_node_type(pos.0).await.get_malus() == 0.0
    }

    async fn target_weight(world: &crate::world::World, pos: BlockPos) -> f32 {
        if world.get_block_state(&pos).await.is_air() {
            10.0
        } else {
            0.0
        }
    }
}

impl Goal for BeeWanderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            {
                let navigator = mob.get_mob_entity().navigator.lock().await;
                if !navigator.is_idle() {
                    return false;
                }
            }

            if mob.get_random().random_range(0..self.chance) != 0 {
                return false;
            }

            self.target = Self::find_hover_target(mob).await;
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
