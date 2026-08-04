use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::atomic::Ordering::Relaxed;

pub struct DolphinJumpGoal {
    interval: i32,
    next_jump_tick: i32,
    /// Vanilla: `DolphinJumpGoal.breached`. Latches true the first time `tick()` observes the
    /// dolphin in water and is never reset (not even in `start`/`stop`) -- verified against
    /// `DolphinJumpGoal.java` (no other write site exists in the decompiled source). Since the
    /// goal starts while the dolphin is still in water, this usually latches (and plays the
    /// sound) on the very first tick after `start()`, then never again for the lifetime of this
    /// goal instance. That is vanilla's actual behavior, not a bug to "fix" here.
    breached: bool,
}

impl DolphinJumpGoal {
    #[must_use]
    pub const fn new(interval: i32) -> Self {
        Self {
            interval: to_goal_ticks(interval),
            next_jump_tick: 0,
            breached: false,
        }
    }

    const fn get_step_x(facing: HorizontalFacing) -> i32 {
        match facing {
            HorizontalFacing::South | HorizontalFacing::North => 0,
            HorizontalFacing::West => -1,
            HorizontalFacing::East => 1,
        }
    }

    const fn get_step_z(facing: HorizontalFacing) -> i32 {
        match facing {
            HorizontalFacing::South => 1,
            HorizontalFacing::North => -1,
            HorizontalFacing::West | HorizontalFacing::East => 0,
        }
    }

    fn water_is_clear(mob: &dyn Mob, step_x: i32, step_z: i32, current_step: i32) -> bool {
        let entity = mob.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let check_pos = pos.offset(Vector3::new(
            step_x * current_step,
            0,
            step_z * current_step,
        ));
        let block_state = world.get_block_state(&check_pos);
        let fluid = world.get_fluid(&check_pos);

        // Check if the block doesn't block motion and fluid is water
        // Vanilla: checks FluidTags.WATER which includes both water and flowing_water
        let is_water = fluid == &Fluid::WATER || fluid == &Fluid::FLOWING_WATER;
        !block_state.is_solid() && is_water
    }

    fn surface_is_clear(mob: &dyn Mob, step_x: i32, step_z: i32, current_step: i32) -> bool {
        let entity = mob.get_entity();
        let pos = entity.block_pos.load();
        let world = entity.world.load();

        let check_pos_1 = pos.offset(Vector3::new(
            step_x * current_step,
            1,
            step_z * current_step,
        ));
        let check_pos_2 = pos.offset(Vector3::new(
            step_x * current_step,
            2,
            step_z * current_step,
        ));

        let block_state_1 = world.get_block_state(&check_pos_1);
        let block_state_2 = world.get_block_state(&check_pos_2);

        block_state_1.is_air() && block_state_2.is_air()
    }
}

impl Goal for DolphinJumpGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.next_jump_tick = (self.next_jump_tick - 1).max(0);

            if self.next_jump_tick > 0 {
                return false;
            }

            if mob.get_random().random::<i32>().unsigned_abs() % self.interval as u32 != 0 {
                return false;
            }

            let facing = mob.get_entity().get_horizontal_facing();
            let step_x = Self::get_step_x(facing);
            let step_z = Self::get_step_z(facing);

            // Check the 6 steps defined in vanilla: 0, 1, 4, 5, 6, 7
            for step in [0, 1, 4, 5, 6, 7] {
                if !Self::water_is_clear(mob, step_x, step_z, step)
                    || !Self::surface_is_clear(mob, step_x, step_z, step)
                {
                    return false;
                }
            }

            self.next_jump_tick = self.interval;
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let velocity = entity.velocity.load();

            // Continue while dolphin is airborne and in water
            let y_velocity_sq = velocity.y * velocity.y;
            let is_falling = y_velocity_sq < 0.03
                || entity.pitch.load() == 0.0
                || (entity.pitch.load().abs() < 10.0);
            let is_in_water = entity.touching_water.load(Relaxed);

            !(is_falling) && !entity.on_ground.load(Relaxed) && is_in_water
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let facing = entity.get_horizontal_facing();
            let step_x = Self::get_step_x(facing) as f64;
            let step_z = Self::get_step_z(facing) as f64;

            let velocity = entity.velocity.load();
            let new_velocity = velocity.add_raw(step_x * 0.6, 0.7, step_z * 0.6);
            entity.velocity.store(new_velocity);

            // Stop navigation
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_entity().pitch.store(0.0);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let world = entity.world.load();

            // Detect the false->true transition of being back in water (vanilla: `breached`).
            let already_breached = self.breached;
            if !already_breached {
                let current_pos = entity.block_pos.load();
                let fluid = world.get_fluid(&current_pos);
                self.breached = fluid == &Fluid::WATER || fluid == &Fluid::FLOWING_WATER;
            }

            if self.breached && !already_breached {
                world.play_sound(
                    Sound::EntityDolphinJump,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
            }

            // Update pitch based on velocity
            let velocity = entity.velocity.load();
            let y_velocity = velocity.y;
            let y_velocity_sq = y_velocity * y_velocity;

            if y_velocity_sq < 0.03 && entity.pitch.load() != 0.0 {
                // Apply smooth rotation towards 0
                let current_pitch = entity.pitch.load();
                let new_pitch = lerp_rotation(0.2, current_pitch, 0.0);
                entity.pitch.store(new_pitch);
            } else if velocity.length() > 1.0e-5 {
                let horizontal_dist = velocity.x.hypot(velocity.z);
                let rotation = (-velocity.y).atan2(horizontal_dist) * 180.0 / std::f64::consts::PI;
                entity.pitch.store(rotation as f32);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

/// Vanilla's Mth.rotLerp equivalent: smoothly interpolates between two rotation values.
/// Used to smoothly update pitch angles.
fn lerp_rotation(progress: f32, current: f32, target: f32) -> f32 {
    let mut diff = target - current;
    while diff > 180.0 {
        diff -= 360.0;
    }
    while diff < -180.0 {
        diff += 360.0;
    }
    current + progress * diff
}
