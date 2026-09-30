use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::abstract_cube_mob::ID_SIZE;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase};

pub const MIN_SIZE: i32 = 1;
pub const MAX_SIZE: i32 = 127;

pub struct CubeMobData {
    jump_delay: AtomicI32,
    target_yaw: AtomicCell<f32>,
    is_aggressive: AtomicBool,
    was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    speed_modifier: AtomicCell<f64>,
    has_split: AtomicBool,
}

impl CubeMobData {
    pub fn new(entity: &Entity) -> Self {
        entity.synched_data.define(ID_SIZE, MIN_SIZE);
        Self {
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            is_aggressive: AtomicBool::new(false),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(0.0),
            has_split: AtomicBool::new(false),
        }
    }
}

/// Shared behaviour of vanilla `AbstractCubeMob` (slimes, magma cubes, sulfur cubes).
pub trait CubeMob: Mob {
    fn get_cube_mob_data(&self) -> &CubeMobData;
    fn jump_sound(&self) -> Sound;
    fn squish_sound(&self) -> Sound;
    fn create_split_child(&self, entity: Entity, size: i32) -> Arc<dyn EntityBase>;

    fn sound_source(&self) -> SoundCategory {
        SoundCategory::Hostile
    }

    /// Mirrors `AbstractCubeMob.setCubeMobHealth`.
    fn cube_mob_health(&self, actual_size: i32) -> f64 {
        f64::from(actual_size * actual_size)
    }

    fn jump_delay(&self) -> i32 {
        rand::random_range(10..30)
    }

    fn can_deal_damage(&self) -> bool {
        !self.is_tiny()
    }

    fn split_count(&self) -> i32 {
        2 + rand::random_range(0..3)
    }

    fn register_common_goals(&self, mob: Weak<dyn CubeMob>) {
        let mob_entity = self.get_mob_entity();
        let mut move_control = mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *move_control = Box::new(CubeMobMoveControl::new(mob.clone()));
        drop(move_control);

        let mut goals = mob_entity
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        goals.add_goal(1, Box::new(CubeMobFloatGoal::new(mob.clone())));
        goals.add_goal(4, Box::new(CubeMobRandomDirectionGoal::new(mob.clone())));
        goals.add_goal(5, Box::new(CubeMobKeepOnJumpingGoal::new(mob)));
    }

    fn add_attack_goal(&self, mob: Weak<dyn CubeMob>) {
        self.get_mob_entity()
            .goals_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_goal(2, Box::new(CubeMobAttackGoal::new(mob)));
    }

    /// Mirrors `setSize`. Overrides call `set_cube_mob_size` first, like vanilla's `super.setSize`.
    fn set_size(&self, size: i32, update_health: bool) {
        self.set_cube_mob_size(size, update_health);
    }

    /// Mirrors `AbstractCubeMob.setSize`.
    fn set_cube_mob_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(MIN_SIZE, MAX_SIZE);
        let living_entity = &self.get_mob_entity().living_entity;
        let entity = &living_entity.entity;
        entity.data.store(actual_size, Ordering::Relaxed);
        entity.set_synced_data(ID_SIZE, actual_size);

        let mut attributes = living_entity
            .attributes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
            health.base_value = self.cube_mob_health(actual_size);
            health.dirty.store(true, Ordering::Relaxed);
        }
        if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            speed.base_value = (0.2 + 0.1 * actual_size as f32) as f64;
            speed.dirty.store(true, Ordering::Relaxed);
        }
        drop(attributes);

        if update_health {
            let max_health = living_entity.get_attribute_value(&Attributes::MAX_HEALTH) as f32;
            living_entity.health.store(max_health);
        }

        let scaled_dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * actual_size as f32,
            height: entity.entity_type.dimension[1] * actual_size as f32,
            eye_height: entity.entity_type.eye_height * actual_size as f32,
        };
        entity.entity_dimension.store(scaled_dimensions);
        let pos = entity.pos.load();
        entity.bounding_box.store(BoundingBox::new_from_pos(
            pos.x,
            pos.y,
            pos.z,
            &scaled_dimensions,
        ));
    }

    fn get_size(&self) -> i32 {
        self.get_entity().data.load(Ordering::Relaxed)
    }

    fn is_tiny(&self) -> bool {
        self.get_size() <= MIN_SIZE
    }

    fn write_cube_mob_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", self.get_size() - 1);
        nbt.put_bool(
            "wasOnGround",
            self.get_cube_mob_data()
                .was_on_ground
                .load(Ordering::Relaxed),
        );
    }

    fn read_cube_mob_nbt(&self, nbt: &NbtCompound) {
        self.set_size(nbt.get_int("Size").unwrap_or(0).wrapping_add(1), false);
        self.get_cube_mob_data().was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn cube_mob_tick(&self) {
        let data = self.get_cube_mob_data();
        // Cleared before AI runs so the float goal's jump survives the move control.
        self.get_mob_entity()
            .living_entity
            .jumping
            .store(false, Ordering::SeqCst);
        data.o_squish.store(data.squish.load());
        data.squish
            .store(data.squish.load() + (data.target_squish.load() - data.squish.load()) * 0.5);

        let entity = self.get_entity();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let was_on_ground = data.was_on_ground.load(Ordering::Relaxed);
        if on_ground && !was_on_ground {
            // Landing particles are client-side: vanilla's Level.addParticle is a no-op on the server.
            let world = entity.world.load();
            world.play_sound_fine(
                self.squish_sound(),
                self.sound_source(),
                &entity.pos.load(),
                self.get_sound_volume(),
                ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) / 0.8,
            );
            data.target_squish.store(-0.5);
        } else if !on_ground && was_on_ground {
            data.target_squish.store(1.0);
        }
        data.was_on_ground.store(on_ground, Ordering::Relaxed);
        data.target_squish.store(data.target_squish.load() * 0.6);
        data.is_aggressive.store(false, Ordering::Relaxed);
        data.speed_modifier.store(0.0);
    }

    fn cube_mob_post_tick(&self) {
        if self
            .get_mob_entity()
            .living_entity
            .dead
            .load(Ordering::Relaxed)
            && self.get_size() > MIN_SIZE
            && self
                .get_cube_mob_data()
                .has_split
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let half_size = self.get_size() / 2;
            let width = entity.entity_dimension.load().width;
            let xz_offset = width / 4.0;
            for i in 0..self.split_count() {
                let xd = (i % 2) as f32 * xz_offset - xz_offset / 2.0;
                let zd = (i / 2) as f32 * xz_offset - xz_offset / 2.0;
                let child_entity = Entity::new(
                    world.clone(),
                    Vector3::new(pos.x + xd as f64, pos.y + 0.5, pos.z + zd as f64),
                    entity.entity_type,
                );
                world.spawn_entity_non_save(self.create_split_child(child_entity, half_size));
            }
        }
    }

    fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) * pitch_adjuster
    }
}

pub struct CubeMobMoveControl {
    mob: Weak<dyn CubeMob>,
}

impl CubeMobMoveControl {
    #[must_use]
    pub const fn new(mob: Weak<dyn CubeMob>) -> Self {
        Self { mob }
    }
}

impl Control for CubeMobMoveControl {}

impl MoveControlTrait for CubeMobMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(hooks) = self.mob.upgrade() else {
            return;
        };
        let cube = hooks.get_cube_mob_data();
        let entity = &mob.get_mob_entity().living_entity.entity;
        let yaw = entity.yaw.load();
        let target_yaw = cube.target_yaw.load();
        let mut diff = (target_yaw - yaw).rem_euclid(360.0);
        if diff > 180.0 {
            diff -= 360.0;
        }
        let new_yaw = yaw + diff.clamp(-90.0, 90.0);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let speed_modifier = cube.speed_modifier.load();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);
        if on_ground {
            if speed_modifier > 0.0 {
                let current_delay = cube.jump_delay.load(Ordering::Relaxed);
                if current_delay <= 0 {
                    let mut next_delay = hooks.jump_delay();
                    if cube.is_aggressive.load(Ordering::Relaxed) {
                        next_delay /= 3;
                    }
                    cube.jump_delay.store(next_delay, Ordering::Relaxed);
                    mob.get_mob_entity()
                        .living_entity
                        .jumping
                        .store(true, Ordering::SeqCst);
                    let world = entity.world.load();
                    world.play_sound_fine(
                        hooks.jump_sound(),
                        hooks.sound_source(),
                        &entity.pos.load(),
                        hooks.get_sound_volume(),
                        hooks.get_sound_pitch(),
                    );
                    movement_input.z = speed_modifier;
                } else {
                    cube.jump_delay.store(current_delay - 1, Ordering::Relaxed);
                }
            }
        } else if speed_modifier > 0.0 {
            movement_input.z = speed_modifier;
        }
        mob.get_mob_entity()
            .living_entity
            .movement_input
            .store(movement_input);
    }
}

struct CubeMobFloatGoal {
    mob: Weak<dyn CubeMob>,
}
impl CubeMobFloatGoal {
    fn new(mob: Weak<dyn CubeMob>) -> Self {
        Self { mob }
    }
}
impl Goal for CubeMobFloatGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.mob.upgrade().is_some_and(|mob| {
            let e = &mob.get_entity();
            e.touching_water.load(Ordering::Relaxed) || e.touching_lava.load(Ordering::Relaxed)
        })
    }
    fn tick(&mut self, _mob: &dyn Mob) {
        if let Some(mob) = self.mob.upgrade() {
            if rand::random_range(0.0..1.0) < 0.8 {
                mob.get_mob_entity()
                    .living_entity
                    .jumping
                    .store(true, Ordering::SeqCst);
            }
            mob.get_cube_mob_data().speed_modifier.store(1.2);
        }
    }
    fn should_run_every_tick(&self) -> bool {
        true
    }
    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}

struct CubeMobAttackGoal {
    mob: Weak<dyn CubeMob>,
    grow_tired_timer: i32,
}
impl CubeMobAttackGoal {
    fn new(mob: Weak<dyn CubeMob>) -> Self {
        Self {
            mob,
            grow_tired_timer: 0,
        }
    }
}
impl Goal for CubeMobAttackGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.mob
            .upgrade()
            .is_some_and(|m| m.get_mob_entity().get_target().is_some())
    }
    fn start(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer = 300;
    }
    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.mob
            .upgrade()
            .is_some_and(|m| m.get_mob_entity().get_target().is_some() && self.grow_tired_timer > 0)
    }
    fn tick(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer -= 1;
        if let Some(mob) = self.mob.upgrade() {
            if let Some(target) = mob.get_mob_entity().get_target() {
                let target_pos = target.get_entity().pos.load();
                let pos = mob.get_entity().pos.load();
                // Vanilla lookAt yaw.
                mob.get_cube_mob_data().target_yaw.store(
                    ((target_pos.z - pos.z)
                        .atan2(target_pos.x - pos.x)
                        .to_degrees()
                        - 90.0) as f32,
                );
            }
            mob.get_cube_mob_data()
                .is_aggressive
                .store(true, Ordering::Relaxed);
        }
    }
    fn should_run_every_tick(&self) -> bool {
        true
    }
    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

struct CubeMobRandomDirectionGoal {
    mob: Weak<dyn CubeMob>,
    chosen_degrees: f32,
    next_randomize_time: i32,
}
impl CubeMobRandomDirectionGoal {
    fn new(mob: Weak<dyn CubeMob>) -> Self {
        Self {
            mob,
            chosen_degrees: 0.0,
            next_randomize_time: 0,
        }
    }
}
impl Goal for CubeMobRandomDirectionGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.mob.upgrade().is_some_and(|m| {
            m.get_mob_entity().get_target().is_none()
                && (m.get_entity().on_ground.load(Ordering::Relaxed)
                    || m.get_entity().touching_water.load(Ordering::Relaxed)
                    || m.get_entity().touching_lava.load(Ordering::Relaxed))
        })
    }
    fn tick(&mut self, _mob: &dyn Mob) {
        if let Some(mob) = self.mob.upgrade() {
            self.next_randomize_time -= 1;
            if self.next_randomize_time <= 0 {
                self.next_randomize_time = rand::random_range(40..100);
                self.chosen_degrees = rand::random_range(0.0..360.0);
            }
            mob.get_cube_mob_data()
                .target_yaw
                .store(self.chosen_degrees);
            mob.get_cube_mob_data()
                .is_aggressive
                .store(false, Ordering::Relaxed);
        }
    }
    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

struct CubeMobKeepOnJumpingGoal {
    mob: Weak<dyn CubeMob>,
}
impl CubeMobKeepOnJumpingGoal {
    fn new(mob: Weak<dyn CubeMob>) -> Self {
        Self { mob }
    }
}
impl Goal for CubeMobKeepOnJumpingGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.mob
            .upgrade()
            .is_some_and(|m| !m.get_entity().has_vehicle())
    }
    fn tick(&mut self, _mob: &dyn Mob) {
        if let Some(mob) = self.mob.upgrade() {
            mob.get_cube_mob_data().speed_modifier.store(1.0);
        }
    }
    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
