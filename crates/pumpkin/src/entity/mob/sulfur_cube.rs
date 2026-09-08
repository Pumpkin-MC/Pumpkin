use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ageable::BABY_START_AGE;
use crate::entity::{
    Entity, EntityBase,
    ai::control::{Control, MoveControlTrait},
    ai::goal::{Controls, Goal},
    mob::{Mob, MobEntity},
};

pub struct SulfurCubeEntity {
    entity: Arc<MobEntity>,
    jump_delay: AtomicI32,
    target_yaw: AtomicCell<f32>,
    was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    speed_modifier: AtomicCell<f64>,
    is_baby: AtomicBool,
    has_split: AtomicBool,
}

impl SulfurCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cube = Self {
            entity: Arc::new(mob_entity),
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(0.0),
            is_baby: AtomicBool::new(false),
            has_split: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(cube);

        {
            let mut move_control = mob_arc
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *move_control = Box::new(SulfurCubeMoveControl::new(Arc::downgrade(&mob_arc)));

            let mut goal_selector = mob_arc
                .entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(
                1,
                Box::new(SulfurCubeFloatGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(
                4,
                Box::new(SulfurCubeRandomDirectionGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(
                5,
                Box::new(SulfurCubeKeepOnJumpingGoal::new(Arc::downgrade(&mob_arc))),
            );
        };

        mob_arc.set_size(2, true);

        mob_arc
    }

    pub fn set_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(1, 127);
        let entity = &self.entity.living_entity.entity;
        entity.data.store(actual_size, Ordering::Relaxed);
        entity.set_synced_data(tracked_data::sulfur_cube::ID_SIZE, actual_size);

        // Update attributes
        {
            let mut attributes = self
                .entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = (4 * actual_size) as f64;
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = (0.2 + 0.1 * actual_size as f32) as f64;
                speed.dirty.store(true, Ordering::Relaxed);
            }
        }

        if update_health {
            if actual_size == 1 && !self.is_baby() {
                self.set_baby(true);
            }
            let max_health = self
                .entity
                .living_entity
                .get_attribute_value(&Attributes::MAX_HEALTH) as f32;
            self.entity.living_entity.health.store(max_health);
        }

        // Refresh dimensions
        let scaled_dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * actual_size as f32,
            height: entity.entity_type.dimension[1] * actual_size as f32,
            eye_height: entity.entity_type.eye_height * actual_size as f32,
        };
        entity.entity_dimension.store(scaled_dimensions);

        let pos = entity.pos.load();
        let new_bb = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &scaled_dimensions);
        entity.bounding_box.store(new_bb);
    }

    pub fn get_size(&self) -> i32 {
        self.entity
            .living_entity
            .entity
            .data
            .load(Ordering::Relaxed)
    }

    pub fn is_tiny(&self) -> bool {
        self.get_size() <= 1
    }

    pub fn is_baby(&self) -> bool {
        self.is_baby.load(Ordering::Relaxed)
    }

    pub fn set_baby(&self, baby: bool) {
        self.is_baby.store(baby, Ordering::Relaxed);
        let entity = &self.entity.living_entity.entity;
        let old_age = entity
            .age
            .swap(if baby { BABY_START_AGE } else { 0 }, Ordering::Relaxed);
        if (old_age < 0) != baby {
            entity.set_synced_data(tracked_data::ageable_mob::DATA_BABY_ID, baby);
        }
    }

    pub(crate) const fn hurt_sound_for_size(size: i32) -> Sound {
        if size <= 1 {
            Sound::EntitySmallSulfurCubeHurt
        } else {
            Sound::EntitySulfurCubeHurt
        }
    }

    fn get_jump_delay() -> i32 {
        rand::random_range(10..30)
    }

    fn rot_lerp(start: f32, end: f32, max_step: f32) -> f32 {
        let mut diff = (end - start).rem_euclid(360.0);
        if diff > 180.0 {
            diff -= 360.0;
        }
        start + diff.clamp(-max_step, max_step)
    }

    fn get_jump_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySmallSulfurCubeJump
        } else {
            Sound::EntitySulfurCubeJump
        }
    }

    fn get_squish_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySmallSulfurCubeSquish
        } else {
            Sound::EntitySulfurCubeSquish
        }
    }

    fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        (rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0 * pitch_adjuster
    }
}

impl Mob for SulfurCubeEntity {
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", self.get_size() - 1);
        nbt.put_int(
            "Age",
            self.entity.living_entity.entity.age.load(Ordering::Relaxed),
        );
        nbt.put_bool("wasOnGround", self.was_on_ground.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_size(nbt.get_int("Size").unwrap_or(0) + 1, false);
        if nbt.get_int("Age").unwrap_or(0) < 0 {
            self.set_baby(true);
        }
        self.was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.o_squish.store(self.squish.load());
        self.squish
            .store(self.squish.load() + (self.target_squish.load() - self.squish.load()) * 0.5);

        let on_ground = self
            .entity
            .living_entity
            .entity
            .on_ground
            .load(Ordering::Relaxed);
        let was_on_ground = self.was_on_ground.load(Ordering::Relaxed);

        if on_ground && !was_on_ground {
            // TODO: particles

            let world = self.entity.living_entity.entity.world.load();
            world.play_sound_fine(
                self.get_squish_sound(),
                SoundCategory::Neutral,
                &self.entity.living_entity.entity.pos.load(),
                self.get_sound_volume(),
                ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) / 0.8,
            );

            self.target_squish.store(-0.5);
        } else if !on_ground && was_on_ground {
            self.target_squish.store(1.0);
        }

        self.was_on_ground.store(on_ground, Ordering::Relaxed);
        self.target_squish.store(self.target_squish.load() * 0.6);

        self.speed_modifier.store(0.0);
    }

    fn post_tick(&self) {
        if self.entity.living_entity.dead.load(Ordering::Relaxed)
            && self.get_size() > 1
            && self
                .has_split
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            let size = self.get_size();
            let world = self.entity.living_entity.entity.world.load();
            let pos = self.entity.living_entity.entity.pos.load();
            let half_size = size / 2;

            let width = self
                .entity
                .living_entity
                .entity
                .entity_dimension
                .load()
                .width;
            let xz_offset = width / 2.0;

            for i in 0..2 {
                let xd = ((i % 2) as f32 - 0.5) * xz_offset;
                let zd = ((i / 2) as f32 - 0.5) * xz_offset;

                let new_pos = Vector3::new(pos.x + xd as f64, pos.y + 0.5, pos.z + zd as f64);
                let new_entity = Entity::new(
                    world.clone(),
                    new_pos,
                    self.entity.living_entity.entity.entity_type,
                );
                let cube = Self::new(new_entity);
                cube.set_size(half_size, true);
                cube.entity
                    .living_entity
                    .entity
                    .yaw
                    .store(rand::random_range(0.0..360.0));
                world.spawn_entity_non_save(cube as Arc<dyn EntityBase>);
            }
        }
    }
}

pub struct SulfurCubeMoveControl {
    cube: Weak<SulfurCubeEntity>,
}

impl SulfurCubeMoveControl {
    #[must_use]
    pub const fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        Self { cube }
    }
}

impl Control for SulfurCubeMoveControl {}

impl MoveControlTrait for SulfurCubeMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;

        let current_yaw = entity.yaw.load();
        let new_yaw = SulfurCubeEntity::rot_lerp(current_yaw, cube.target_yaw.load(), 90.0);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let speed_modifier = cube.speed_modifier.load();
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);

        let on_ground = entity.on_ground.load(Ordering::Relaxed);

        if on_ground {
            if speed_modifier > 0.0 {
                let current_delay = cube.jump_delay.load(Ordering::Relaxed);
                if current_delay <= 0 {
                    // Start jump
                    let next_delay = SulfurCubeEntity::get_jump_delay();
                    cube.jump_delay.store(next_delay, Ordering::Relaxed);
                    living_entity.jumping.store(true, Ordering::SeqCst);
                    let world = entity.world.load();
                    world.play_sound_fine(
                        cube.get_jump_sound(),
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                        cube.get_sound_volume(),
                        cube.get_sound_pitch(),
                    );
                    movement_input.z = speed_modifier;
                } else {
                    cube.jump_delay.store(current_delay - 1, Ordering::Relaxed);
                    living_entity.jumping.store(false, Ordering::SeqCst);
                }
            } else {
                living_entity.jumping.store(false, Ordering::SeqCst);
            }
        } else {
            // In air: move forward but don't "jump" again
            if speed_modifier > 0.0 {
                movement_input.z = speed_modifier;
            }
            living_entity.jumping.store(false, Ordering::SeqCst);
        }
        living_entity.movement_input.store(movement_input);
    }
}

pub struct SulfurCubeFloatGoal {
    cube: Weak<SulfurCubeEntity>,
}

impl SulfurCubeFloatGoal {
    #[must_use]
    pub const fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        Self { cube }
    }
}

impl Goal for SulfurCubeFloatGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        let entity = &cube.entity.living_entity.entity;
        entity.touching_water.load(Ordering::Relaxed)
            || entity.touching_lava.load(Ordering::Relaxed)
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(cube) = self.cube.upgrade() else {
            return;
        };
        if rand::random_range(0.0..1.0) < 0.8 {
            cube.entity
                .living_entity
                .jumping
                .store(true, Ordering::SeqCst);
        }
        cube.speed_modifier.store(1.2);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}

pub struct SulfurCubeRandomDirectionGoal {
    cube: Weak<SulfurCubeEntity>,
    chosen_degrees: f32,
    next_randomize_time: i32,
}

impl SulfurCubeRandomDirectionGoal {
    #[must_use]
    pub const fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        Self {
            cube,
            chosen_degrees: 0.0,
            next_randomize_time: 0,
        }
    }
}

impl Goal for SulfurCubeRandomDirectionGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        let entity = &cube.entity.living_entity.entity;
        entity.on_ground.load(Ordering::Relaxed)
            || entity.touching_water.load(Ordering::Relaxed)
            || entity.touching_lava.load(Ordering::Relaxed)
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
        cube.target_yaw.store(self.chosen_degrees);
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

pub struct SulfurCubeKeepOnJumpingGoal {
    cube: Weak<SulfurCubeEntity>,
}

impl SulfurCubeKeepOnJumpingGoal {
    #[must_use]
    pub const fn new(cube: Weak<SulfurCubeEntity>) -> Self {
        Self { cube }
    }
}

impl Goal for SulfurCubeKeepOnJumpingGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(cube) = self.cube.upgrade() else {
            return false;
        };
        !cube.entity.living_entity.entity.has_vehicle()
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        if let Some(cube) = self.cube.upgrade() {
            cube.speed_modifier.store(1.0);
        }
    }

    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
