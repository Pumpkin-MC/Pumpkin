use std::sync::Weak;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    ai::control::{Control, MoveControlTrait, move_control::MoveControl},
    ai::goal::Goal,
    mob::Mob,
};

/// Movement and squish state shared by slimes, magma cubes and sulfur cubes.
pub struct CubeMobData {
    pub jump_delay: AtomicI32,
    pub target_yaw: AtomicCell<f32>,
    pub is_aggressive: AtomicBool,
    pub was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    pub speed_modifier: AtomicCell<Option<f64>>,
}

impl Default for CubeMobData {
    fn default() -> Self {
        Self {
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            is_aggressive: AtomicBool::new(false),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(None),
        }
    }
}

/// A cube mob that moves by jumping toward a chosen yaw.
pub trait CubeMob: Mob {
    fn cube_data(&self) -> &CubeMobData;
    fn get_jump_sound(&self) -> Sound;
    fn get_squish_sound(&self) -> Sound;

    fn get_size(&self) -> i32 {
        self.get_entity().data.load(Ordering::Relaxed)
    }

    fn is_tiny(&self) -> bool {
        self.get_size() <= 1
    }

    fn cube_sound_category(&self) -> SoundCategory {
        SoundCategory::Hostile
    }

    fn do_play_jump_sound(&self) -> bool {
        self.get_size() > 0
    }

    fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        (rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0 * pitch_adjuster
    }

    fn get_jump_delay(&self) -> i32 {
        rand::random_range(10..30)
    }
}

/// Runs the shared cube mob squish and landing sound tick.
pub fn tick_cube_mob(cube: &dyn CubeMob) {
    let data = cube.cube_data();
    let entity = cube.get_entity();

    data.o_squish.store(data.squish.load());
    data.squish
        .store(data.squish.load() + (data.target_squish.load() - data.squish.load()) * 0.5);

    let on_ground = entity.on_ground.load(Ordering::Relaxed);
    let was_on_ground = data.was_on_ground.load(Ordering::Relaxed);

    if on_ground && !was_on_ground {
        let world = entity.world.load();
        world.play_sound_fine(
            cube.get_squish_sound(),
            cube.cube_sound_category(),
            &entity.pos.load(),
            cube.get_sound_volume(),
            ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) / 0.8,
        );

        data.target_squish.store(-0.5);
    } else if !on_ground && was_on_ground {
        data.target_squish.store(1.0);
    }

    data.was_on_ground.store(on_ground, Ordering::Relaxed);
    data.target_squish.store(data.target_squish.load() * 0.6);

    data.is_aggressive.store(false, Ordering::Relaxed);
    data.speed_modifier.store(None);
}

pub struct CubeMobMoveControl {
    cube: Weak<dyn CubeMob>,
}

impl CubeMobMoveControl {
    #[must_use]
    pub const fn new(cube: Weak<dyn CubeMob>) -> Self {
        Self { cube }
    }
}

impl Control for CubeMobMoveControl {}

impl MoveControlTrait for CubeMobMoveControl {
    fn set_wanted_position(&mut self, _x: f64, _y: f64, _z: f64, speed_modifier: f64) {
        if let Some(cube) = self.cube.upgrade() {
            cube.cube_data().speed_modifier.store(Some(speed_modifier));
        }
    }

    fn strafe(&mut self, _forward: f32, _right: f32) {
        if let Some(cube) = self.cube.upgrade() {
            cube.cube_data().speed_modifier.store(None);
        }
    }

    fn has_wanted(&self) -> bool {
        self.cube
            .upgrade()
            .is_some_and(|cube| cube.cube_data().speed_modifier.load().is_some())
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        let data = cube.cube_data();
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;

        let current_yaw = entity.yaw.load();
        let new_yaw =
            MoveControl::rotlerp(current_yaw, data.target_yaw.load(), MoveControl::MAX_TURN);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let Some(speed_modifier) = data.speed_modifier.take() else {
            let input = living_entity.movement_input.load();
            living_entity
                .movement_input
                .store(Vector3::new(input.x, input.y, 0.0));
            return;
        };
        mob_entity.set_speed(
            (speed_modifier * living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED))
                as f32,
        );
        if entity.on_ground.load(Ordering::Relaxed) {
            let delay = data.jump_delay.fetch_sub(1, Ordering::Relaxed);
            if delay <= 0 {
                let mut next_delay = cube.get_jump_delay();
                if data.is_aggressive.load(Ordering::Relaxed) {
                    next_delay /= 3;
                }
                data.jump_delay.store(next_delay, Ordering::Relaxed);
                mob_entity
                    .jump_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .jump();
                if cube.do_play_jump_sound() {
                    let world = entity.world.load();
                    world.play_sound_fine(
                        cube.get_jump_sound(),
                        cube.cube_sound_category(),
                        &entity.pos.load(),
                        cube.get_sound_volume(),
                        cube.get_sound_pitch(),
                    );
                }
            } else {
                mob_entity.set_speed(0.0);
                let input = living_entity.movement_input.load();
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, input.y, 0.0));
            }
        }
    }
}

pub struct CubeMobFloatGoal {
    cube: Weak<dyn CubeMob>,
}

impl CubeMobFloatGoal {
    #[must_use]
    pub const fn new(cube: Weak<dyn CubeMob>) -> Self {
        Self { cube }
    }
}

impl Goal for CubeMobFloatGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        cube.get_entity().is_in_liquid()
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        if rand::random_range(0.0..1.0) < 0.8 {
            cube.get_mob_entity()
                .jump_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .jump();
        }
        cube.cube_data().speed_modifier.store(Some(1.2));
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::JUMP | crate::entity::ai::goal::Controls::MOVE
    }
}

pub struct CubeMobRandomDirectionGoal {
    cube: Weak<dyn CubeMob>,
    chosen_degrees: f32,
    next_randomize_time: i32,
}

impl CubeMobRandomDirectionGoal {
    #[must_use]
    pub const fn new(cube: Weak<dyn CubeMob>) -> Self {
        Self {
            cube,
            chosen_degrees: 0.0,
            next_randomize_time: 0,
        }
    }
}

impl Goal for CubeMobRandomDirectionGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        let entity = cube.get_entity();
        cube.get_mob_entity().get_target().is_none()
            && (entity.on_ground.load(Ordering::Relaxed)
                || entity.touching_water.load(Ordering::Relaxed)
                || entity.touching_lava.load(Ordering::Relaxed))
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        self.next_randomize_time -= 1;
        if self.next_randomize_time <= 0 {
            self.next_randomize_time = rand::random_range(40..100);
            self.chosen_degrees = rand::random_range(0.0..360.0);
        }
        let data = cube.cube_data();
        data.target_yaw.store(self.chosen_degrees);
        data.is_aggressive.store(false, Ordering::Relaxed);
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::LOOK
    }
}

pub struct CubeMobKeepOnJumpingGoal {
    cube: Weak<dyn CubeMob>,
}

impl CubeMobKeepOnJumpingGoal {
    #[must_use]
    pub const fn new(cube: Weak<dyn CubeMob>) -> Self {
        Self { cube }
    }
}

impl Goal for CubeMobKeepOnJumpingGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        !cube.get_entity().has_vehicle()
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        cube.cube_data().speed_modifier.store(Some(1.0));
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::JUMP | crate::entity::ai::goal::Controls::MOVE
    }
}
